# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/merge_p4_selection.py --help
"""Recover fresh-approved candidates while preserving the post-audit manifest."""
from __future__ import annotations

import argparse
from collections import Counter
from collections.abc import Callable
from copy import deepcopy
from dataclasses import dataclass, replace
import json
from itertools import product
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import TypedDict

from hair_inventory_p4 import load_image
from hair_freeze_p4 import freeze_selected_hair
from install_v4 import (InstallError, InstallPlan, InstallRequest, PlannedSprite, Target,
                       build_plan, digest, fallback_for, validate_assembled, verify_file)
from merge_fill_selection import group_key, semantic
from merge_p3_selection import held_lineage
from salvage_v4 import CandidateRecord, guarded_candidate
from salvage_verdicts import REASONS, atomic_json, candidate_fingerprint
from v4_contracts import WorldScale, guard_frame


class Trial(TypedDict, total=False):
    target: str
    candidate_id: str
    accepted: bool
    reason: str
    untried_candidate_ids: list[str]


@dataclass(frozen=True, slots=True)
class RecoveryRequest:
    root: Path
    ledger: Path
    review_index: Path
    reviews: tuple[Path, ...]
    output: Path
    baseline_selection_paths: tuple[Path, ...] = ()


def choose_lowest(options: dict[str, list[CandidateRecord]],
                  verify: Callable[[dict[str, CandidateRecord]], None]
                  ) -> tuple[dict[str, CandidateRecord], list[Trial]]:
    """Check error-ranked complete combinations, recording each target's three-strike limit."""
    groups = {}
    for target, rows in options.items():
        groups.setdefault(group_key(rows[0]), {})[target] = rows
    chosen = {}
    trials = []
    for group in groups.values():
        targets = sorted(group)
        ranked = [sorted(group[target], key=lambda row: (row['metrics']['mean_delta_e'], row['id']))
                  for target in targets]
        failures = dict.fromkeys(targets, 0)
        attempted = {target: set() for target in targets}
        accepted = False
        for combination in product(*ranked):
            if any(failures[target] >= 3 for target in targets):
                break
            additions = dict(zip(targets, combination))
            trial = {**chosen, **additions}
            for target, row in additions.items():
                attempted[target].add(row['id'])
            try:
                verify(trial)
            except InstallError as error:
                for target, row in additions.items():
                    failures[target] += 1
                    trials.append(Trial(target=target, candidate_id=row['id'], accepted=False, reason=str(error)))
            else:
                chosen = trial
                accepted = True
                trials.extend(Trial(target=target, candidate_id=row['id'], accepted=True)
                              for target, row in additions.items())
                break
        if not accepted:
            for target, rows in zip(targets, ranked):
                trials.append(Trial(target=target, accepted=False,
                    reason='three_failed_trials' if failures[target] >= 3 else 'all_combinations_failed',
                    untried_candidate_ids=[row['id'] for row in rows if row['id'] not in attempted[target]]))
    return chosen, trials


def merge(request: RecoveryRequest) -> tuple[InstallPlan, dict]:
    """Write reviewable recovery evidence only; return the standard installer plan."""
    root = request.root.resolve()
    ledger_path, index_path = root / request.ledger, root / request.review_index
    ledger = json.loads(ledger_path.read_text())
    records = {row['id']: row for row in ledger.get('lineage_candidates', []) + ledger['candidates']}
    ids = ledger.get('recovery_ids', ledger.get('reprocessed_ids', [row['id'] for row in ledger['candidates']]))
    candidates = {identity: records[identity] for identity in ids}
    index = json.loads(index_path.read_text())
    if index['candidates_sha256'] != digest(ledger_path) or index['candidate_fingerprints'] != {
            identity: candidate_fingerprint(row) for identity, row in candidates.items()}:
        raise InstallError('Pre-review candidate pixels or ledger changed')
    if not index['images_sha256']:
        raise InstallError('Pre-review index has no boards')
    bindings = {str(index_path): digest(index_path), str(ledger_path): digest(ledger_path)}
    for supplied, expected in index['images_sha256'].items():
        path = root / supplied
        verify_file(path, expected)
        bindings[str(path)] = expected
    for supplied, expected in index.get('source_sha256', {}).items():
        path = root / supplied
        verify_file(path, expected)
        bindings[str(path)] = expected
    reviews = {}
    for supplied in request.reviews:
        path = root / supplied
        bindings[str(path)] = digest(path)
        payload = json.loads(path.read_text())
        rows = payload if isinstance(payload, list) else payload['verdicts']
        for row in rows:
            identity = row['candidate_id']
            if identity not in candidates or identity in reviews or row['sprite_id'] != candidates[identity]['slot_id']:
                raise InstallError(f'Unknown, duplicate or wrong-slot review: {identity}')
            if row['verdict'] not in ('ok', 'reject') or not row['observation'].strip():
                raise InstallError(f'Invalid review: {identity}')
            allowed = (REASONS - {'region_map_wrong'}) | {'hair_mask_wrong'}
            if (row['verdict'] == 'reject' and row['reason'] not in allowed) or (row['verdict'] == 'ok' and row['reason']):
                raise InstallError(f'Invalid review reason: {identity}')
            reviews[identity] = row
    if set(reviews) != set(candidates):
        raise InstallError('Fresh review must cover every recovery candidate')
    manifest_path = root / 'assets/art/manifest.json'
    manifest = json.loads(manifest_path.read_text())
    bindings[str(manifest_path)] = digest(manifest_path)
    selections = []
    defaults = [Path('docs/art/production3/reprocess/selection.json'), Path('docs/art/production3/installation_selection.json')]
    recovery = Path('docs/art/production4/recovery/selection_run/selection.json')
    if (root / recovery).exists(): defaults.insert(0, recovery)
    for relative in request.baseline_selection_paths or defaults:
        path = root / relative
        selections.append(json.loads(path.read_text()))
        bindings[str(path)] = digest(path)
    baseline = deepcopy(selections[-1])
    for entry in baseline['slots'].values():
        entry.update(disposition='unfilled', candidate_id=None)
    current: dict[str, PlannedSprite] = {}
    for row in manifest['assets']:
        if not row.get('sprite_id') or not row.get('drawable', True) or not row.get('native_path'):
            continue
        for field in ('path', 'native_path'):
            bindings[str(root / row[field])] = digest(root / row[field])
        sprite = PlannedSprite(row['sprite_id'], row.get('body', ''), row.get('view', ''), row.get('activity', ''),
            row.get('frame', 0), row['qa_status'], row['source'], row.get('source_sprite', ''),
            row.get('contains_basket', False), row.get('fallback_kind', ''), row.get('activity_icon', ''),
            load_image(root / row['native_path']), load_image(root / row['path']), row.get('candidate_id', ''))
        current[sprite.slot_id] = sprite
        if sprite.qa_status in ('accepted', 'derived'):
            decision = next((selection['slots'][sprite.slot_id] for selection in selections
                if selection['slots'].get(sprite.slot_id, {}).get('candidate_id') == sprite.candidate_id), None)
            if decision is None:
                raise InstallError(f'Missing current approved provenance: {sprite.slot_id}')
            baseline['slots'][sprite.slot_id] = deepcopy(decision)
    scale = WorldScale(**ledger['scale'])
    if ledger['scale'] != baseline['scale']:
        raise InstallError('Mixed world scales')
    options: dict[str, list[CandidateRecord]] = {}
    exclusions = {}
    holds_path = root / 'docs/art/production2/fill_sheets.json'
    holds = set(json.loads(holds_path.read_text()).get('installation_holds', {})) | {'V_a1_01@1'}
    for identity, row in candidates.items():
        despeckle = row.get('despeckle', {})
        over_limit = despeckle.get('changed_count', 0) > .10 * despeckle.get('opaque_count', 0)
        passed = (row['guards']['passed'] and not row['guards']['errors'] and not over_limit and
            not any(row.get(key, {}).get('errors') for key in ('group_guards', 'cross_guards')) and
            guard_frame(guarded_candidate(row), scale).passed)
        reason = ('visual_rejection' if reviews[identity]['verdict'] != 'ok' else
                  'held_lineage' if held_lineage(row, records, holds) else 'technical_guard' if not passed else '')
        if reason:
            exclusions[identity] = reason
        else:
            options.setdefault(row.get('target_slot', row['slot_id']), []).append(row)
    output = root / request.output
    output.mkdir(parents=True, exist_ok=True)
    selection_path, envelope_path = output / 'selection.json', output / 'envelope.json'
    installer_ledger = output / 'installer_candidates.json'
    atomic_json(installer_ledger, dict(ledger, candidates=list(records.values())))
    fingerprints = {identity: candidate_fingerprint(row) for identity, row in candidates.items()}

    def materialize(chosen: dict[str, CandidateRecord], paths: tuple[Path, Path]) -> InstallPlan:
        selection, envelope = paths
        if len({row['slot_id'] for row in chosen.values()}) != len(chosen):
            raise InstallError('Duplicate physical slots in recovery selection')
        atomic_json(envelope, dict(candidate_fingerprints={row['slot_id']: fingerprints[row['id']] for row in chosen.values()},
            verdicts=[reviews[row['id']] for row in chosen.values()], prereview_bindings_sha256=bindings))
        result = deepcopy(baseline)
        for target, row in chosen.items():
            meta = semantic(row)
            result['slots'][target] = dict(candidate_id=row['id'], disposition='accepted' if row['source_kind'] == 'generated' else 'derived',
                ledger=str(installer_ledger), ledger_sha256=digest(installer_ledger), verdict=str(envelope),
                verdict_sha256=digest(envelope), semantic=meta, **meta)
        result['accepted'] = {target: decision['candidate_id'] for target, decision in result['slots'].items()
                              if decision['disposition'] in ('accepted', 'derived')}
        atomic_json(selection, result)
        plan = build_plan(InstallRequest(root, selection, holds=holds_path))
        final = dict(current)
        fresh = {sprite.slot_id: sprite for sprite in plan.sprites if sprite.slot_id in chosen}
        final.update(fresh)
        sources = tuple(sprite for sprite in final.values() if sprite.qa_status == 'accepted')
        icons = frozenset(name for name in final if name.startswith('U1_'))
        for name, old in current.items():
            if name in chosen or old.activity != 'carry' or old.fallback_kind not in ('idle_icon', 'borrowed_carry_view'):
                continue
            if any(source.body == old.body and source.activity == 'carry' for source in fresh.values()):
                replacement = fallback_for(Target(name, old.body, old.view, old.activity, old.frame), sources, icons)
                if replacement is not None:
                    final[name] = replacement
        source_candidates = []
        for name, sprite in final.items():
            decision = result['slots'].get(name, {})
            if sprite.qa_status not in ('accepted', 'derived') or not decision.get('candidate_id'):
                continue
            source_ledger = json.loads((root / decision['ledger']).read_text())
            record = next(row for row in source_ledger['candidates'] if row['id'] == decision['candidate_id'])
            for kind, image in (('native', sprite.native), ('sprite', sprite.sprite)):
                original = load_image(root / record['images'][kind])
                if image.size != original.size or image.tobytes() != original.tobytes():
                    raise InstallError(f'Current approved pixels changed: {name}')
            source_candidates.append((Target(name, sprite.body, sprite.view, sprite.activity, sprite.frame), guarded_candidate(record)))
        diagnostics = validate_assembled(tuple(final.values()), tuple(source_candidates), scale)
        evidence = dict(plan.evidence)
        mutable_hair = {root / 'tools/art_pipeline/reviewed_hair_maps_p4.json',
                        *(root / f'tools/art_pipeline/hair_masks_p4/recovery/{target}.png' for target in chosen)}
        evidence.update({Path(path): value for path, value in bindings.items()})
        evidence = {path: value for path, value in evidence.items() if path not in mutable_hair}
        return replace(plan, sprites=tuple(final.values()), suppressed={name: row for name, row in manifest.get('suppressed', {}).items() if name not in final},
                       evidence=tuple(evidence.items()), fallback_guard_diagnostics=diagnostics)

    with TemporaryDirectory(prefix='p4-recovery-') as directory:
        paths = Path(directory) / 'selection.json', Path(directory) / 'envelope.json'
        def verify(chosen: dict[str, CandidateRecord]) -> None:
            materialize(chosen, paths)
        chosen, trials = choose_lowest(options, verify)
    for path, expected in bindings.items():
        verify_file(Path(path), expected)
    plan = materialize(chosen, (selection_path, envelope_path))
    report = dict(selected={target: row['id'] for target, row in chosen.items()}, trials=trials, exclusions=exclusions,
        before=dict(Counter(sprite.qa_status for sprite in current.values())), after=dict(Counter(sprite.qa_status for sprite in plan.sprites)),
        fallbacks_before=dict(Counter(sprite.fallback_kind for sprite in current.values() if sprite.qa_status == 'fallback')),
        fallbacks_after=dict(Counter(sprite.fallback_kind for sprite in plan.sprites if sprite.qa_status == 'fallback')),
        source_manifest_sha256=bindings[str(manifest_path)], selection_sha256=plan.selection_sha256,
        prereview_bindings_sha256=bindings, installer_validation='passed', applied=False)
    atomic_json(output / 'adjudication.json', report)
    return plan, report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--ledger', type=Path, required=True)
    parser.add_argument('--review-index', type=Path, required=True)
    parser.add_argument('--reviews', type=Path, nargs='+', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    _, report = merge(RecoveryRequest(args.root, args.ledger, args.review_index, tuple(args.reviews), args.output))
    print(json.dumps(report['selected'], indent=2))


if __name__ == '__main__':
    main()
