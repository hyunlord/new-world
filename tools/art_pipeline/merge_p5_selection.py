# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/merge_p5_selection.py prepare
"""Harvest fingerprint-bound approvals as whole, independently reviewed animations."""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
from copy import deepcopy
from dataclasses import replace
import json
from pathlib import Path

from PIL import Image
from harvest_boards_p5 import boards, completion_id
from harvest_p5 import Completion, binding_matches, completion_guard, completions, envelope_row, normalize
from install_v4 import (InstallError, InstallPlan, InstallRequest, PlannedSprite, Target,
                       build_plan, digest, validate_assembled, verify_file)
from production_sheets import SHEETS
from salvage_v4 import guard_data, guarded_candidate
from salvage_verdicts import atomic_json, candidate_fingerprint
from v4_contracts import FrameCandidate, FrameSpec, WorldScale, guard_chosen, guard_group
from v3_semantics import semantic_counts

DEFAULT_POOLS = (
    ('docs/art/production4/recovery/candidates.json', 'docs/art/production4/recovery/review_index.json',
     'docs/art/production4/recovery/reviews/final_reviews.json'),
    ('docs/art/production4/wood/processed/candidates.json', 'docs/art/production4/wood/processed/review_index.json',
     'docs/art/production4/wood/reviews/final_reviews.json'))
OUTPUT = Path('docs/art/production5/harvest')


def read(path: Path) -> dict:
    return json.loads(path.read_text())


def load_pools(pools: tuple[tuple[str, str, str], ...]) -> tuple[dict, dict, dict, dict]:
    """Compare original record fingerprints before changing any guard metadata."""
    records, approvals, bindings, exclusions = {}, {}, {}, {}
    for ledger_name, index_name, review_name in pools:
        ledger, index, reviews = read(Path(ledger_name)), read(Path(index_name)), read(Path(review_name))
        verdicts = {r['candidate_id']: r for r in reviews['verdicts']}
        for row in ledger['candidates']:
            if row.get('mask_scope') == 'c':
                continue
            identity = row['id']
            if not binding_matches(row, index['candidate_fingerprints']):
                exclusions[identity] = 'candidate_fingerprint_changed'
                continue
            if identity not in verdicts or verdicts[identity]['verdict'] != 'ok':
                exclusions[identity] = 'visual_rejection'
                continue
            if row['body'] == 'child' and row['activity'] in ('carry_wood', 'gather_wood'):
                exclusions[identity] = 'children_do_not_gather_wood'
                continue
            if identity == 'V_a1_01@1' or row.get('ancestor_id') == 'V_a1_01@1':
                exclusions[identity] = 'installation_hold'
                continue
            bindings[identity] = dict(fingerprint=candidate_fingerprint(row), review_index=index_name,
                                      review_path=review_name, ledger=ledger_name)
            approvals[identity] = verdicts[identity]
            records[identity] = normalize(row, WorldScale(**ledger.get('scale', {})))
    return records, approvals, bindings, exclusions


def current_sprites(root: Path) -> dict[str, PlannedSprite]:
    """Read the exact current drawable inventory, including audited fallbacks."""
    output = {}
    for row in read(root / 'assets/art/manifest.json')['assets']:
        if not row.get('sprite_id') or not row.get('drawable', True) or not row.get('native_path'):
            continue
        with Image.open(root / row['path']) as opened:
            sprite = opened.convert('RGBA')
        with Image.open(root / row['native_path']) as opened:
            native = opened.convert('RGBA')
        item = PlannedSprite(row['sprite_id'], row.get('body', ''), row.get('view', ''), row.get('activity', ''),
            row.get('frame', 0), row['qa_status'], row['source'], row.get('source_sprite', ''),
            row.get('contains_basket', False), row.get('fallback_kind', ''), row.get('activity_icon', ''),
            native, sprite, row.get('candidate_id', ''))
        output[item.slot_id] = item
    return output


def cross_guard(completion: Completion, current: dict[str, PlannedSprite]) -> tuple[str, ...]:
    """Check the proposed whole group against the current world-sized drawables."""
    final = {}
    for target, sprite in current.items():
        sheet, cell = target.rsplit('_', 1)
        berry, fire = semantic_counts(sprite.native)
        final[target] = FrameCandidate(FrameSpec(sheet, int(cell), 1, sprite.body, sprite.view,
            sprite.activity, sprite.native.size), sprite.native, 0., 100., True, berry, fire, sprite.frame)
    for row in completion.rows:
        final[row.get('target_slot', row['slot_id'])] = guarded_candidate(row)
    report = guard_chosen(
        bushes=[final[k] for k in ('N2_01', 'N2_02', 'N2_03', 'N2_04') if k in final],
        adults=[c for c in final.values() if c.frame.body.startswith('adult')],
        children=[c for c in final.values() if c.frame.body == 'child'],
        baskets=[final[k] for k in ('S2_10', 'S2_11') if k in final])
    return report.errors


def prepare(root: Path = Path.cwd(), output: Path = OUTPUT,
            pools: tuple[tuple[str, str, str], ...] = DEFAULT_POOLS) -> dict:
    """Freeze all ranked completions and first reviewable proposals without installation."""
    if (output / 'adjudication.json').exists():
        raise InstallError('Use a separate output directory; prior installation evidence is immutable')
    output.mkdir(parents=True, exist_ok=True)
    originals, approvals, bindings, exclusions = load_pools(pools)
    current = current_sprites(root)
    manifest = read(root / 'assets/art/manifest.json')
    atomic_json(output / 'baseline_manifest.json', manifest)
    expected = defaultdict(dict)
    for row in manifest['assets']:
        if row.get('sprite_id') and row.get('body'):
            expected[(row['body'], row['view'], row['activity'])][row['frame']] = row['sprite_id']
    for sheet, cells in SHEETS.items():
        body = {'a': 'adult_a', 'b': 'adult_b', 'c': 'child'}.get(sheet[2:3], '')
        for index, cell in enumerate(cells, 1):
            if body and cell.view:
                expected[(body, cell.view, cell.activity)].setdefault(cell.frame, f'{sheet}_{index:02d}')
    for ledger_name, _, _ in pools:
        for row in read(Path(ledger_name))['candidates']:
            if row['body']:
                expected[(row['body'], row['view'], row['activity'])].setdefault(row['frame'], row.get('target_slot', row['slot_id']))
    palette = [tuple(row['rgb']) for row in read(root / 'tools/art_pipeline/production_palette.json')['colors']]
    sets, objects = defaultdict(list), defaultdict(list)
    for row in originals.values():
        if row['body']:
            sets[(row['body'], row['view'], row['activity'], row.get('source_sheet', row['sheet']), row['attempt'])].append(row)
        else:
            objects[row.get('target_slot', row['slot_id'])].append(row)
    ranked, search, all_records = defaultdict(list), [], dict(originals)
    for key, rows in sorted(sets.items()):
        targets = expected.get(key[:3], {})
        if not targets:
            for row in rows:
                targets[row['frame']] = row.get('target_slot', row['slot_id'])
        alternatives = completions(rows, targets, output / 'derived', palette)
        if not alternatives:
            search.append(dict(group=list(key[:3]), source=list(key[3:]), reason='no_complete_usable_set'))
        for completion in alternatives:
            identity = completion_id(completion)
            failures = completion_guard(completion, WorldScale()) + cross_guard(completion, current)
            old = [s for s in current.values() if (s.body, s.view, s.activity) == key[:3]]
            accepted = sum(s.qa_status == 'accepted' for s in old)
            real = len(completion.rows) - completion.copies
            replaceable = all(s.qa_status in ('procedural', 'derived', 'fallback') for s in old) or real > accepted
            reason = ','.join(failures) if failures else '' if replaceable else 'does_not_improve_real_frame_count'
            search.append(dict(id=identity, group=list(key[:3]), source=list(key[3:]), kind=completion.kind,
                               copies=completion.copies, rank=list(completion.rank), reason=reason,
                               candidate_ids=[r['id'] for r in completion.rows], status='guard_rejected' if failures else 'untried'))
            if failures or not replaceable:
                continue
            report = guard_data(guard_group([guarded_candidate(r) for r in completion.rows]))
            if completion.kind == 'C2':
                report['warnings'].append('two_pose_cycle')
            for row in completion.rows:
                row['group_guards'] = report
                all_records[row['id']] = row
            ranked[key[:3]].append(completion)
    for ledger_name, _, _ in tuple(dict.fromkeys((*DEFAULT_POOLS, *pools))):
        for row in read(Path(ledger_name)).get('lineage_candidates', []):
            all_records.setdefault(row['id'], row)
    proposals = []
    for group, alternatives in sorted(ranked.items()):
        alternatives.sort(key=lambda c: c.rank)
        for rank, completion in enumerate(alternatives):
            reference = next((s.sprite for s in current.values() if s.body == group[0] and s.activity == 'idle' and s.view == 'S'), completion.rows[0])
            paths = boards(completion, reference, output / 'animation')
            proposals.append(dict(id=completion_id(completion), group=list(group), source=list(completion.source),
                kind=completion.kind, rank=rank, copies=completion.copies,
                candidate_ids=[r['id'] for r in completion.rows], **paths))
    selected_objects = {}
    for target, rows in objects.items():
        usable = [r for r in rows if r['guards']['passed']]
        if usable:
            selected_objects[target] = min(usable, key=lambda r: (len(r['guards']['warnings']), r['metrics']['mean_delta_e']))['id']
    atomic_json(output / 'installer_candidates.json', dict(schema_version=1, scale=dict(adult_height=28, child_height=21), candidates=list(all_records.values())))
    payload = dict(proposals=proposals, objects=selected_objects, original_bindings=bindings,
                   approvals=approvals, exclusions=exclusions, trials=search,
                   baseline_manifest_sha256=digest(root / 'assets/art/manifest.json'))
    atomic_json(output / 'search.json', payload)
    atomic_json(output / 'animation/review_queue.json', {'groups': [p for p in proposals if p['rank'] == 0]})
    return payload


def finalize(root: Path = Path.cwd(), output: Path = OUTPUT) -> tuple[InstallPlan, dict]:
    """Make a standard installer plan only from whole groups with frozen animation approval."""
    search = read(output / 'search.json')
    ledger_path = output / 'installer_candidates.json'
    ledger = read(ledger_path)
    for original_id, binding in search['original_bindings'].items():
        if binding.get('ledger'):
            original = next(r for r in read(Path(binding['ledger']))['candidates'] if r['id'] == original_id)
            if candidate_fingerprint(original) != binding['fingerprint']:
                raise InstallError(f'Original candidate pixels changed: {original_id}')
        index = read(Path(binding['review_index']))
        if index['candidate_fingerprints'].get(original_id) != binding['fingerprint']:
            raise InstallError(f'Original review binding changed: {original_id}')
    records = {r['id']: r for r in ledger['candidates']}
    reviews_path = output / 'animation/final_reviews.json'
    reviews = {r['id']: r for r in read(reviews_path)['verdicts']}
    chosen = {target: records[identity] for target, identity in search['objects'].items()}
    groups, approved_groups = defaultdict(list), []
    for proposal in search['proposals']:
        groups[tuple(proposal['group'])].append(proposal)
    for group, alternatives in groups.items():
        strikes = 0
        for proposal in sorted(alternatives, key=lambda p: p['rank']):
            verdict = reviews.get(proposal['id'])
            if verdict is None:
                break
            if verdict['verdict'] != 'ok':
                strikes += 1
                if strikes >= 3:
                    break
                continue
            for identity in proposal['candidate_ids']:
                row = records[identity]
                chosen[row.get('target_slot', row['slot_id'])] = row
            approved_groups.append(dict(proposal, animation_review=str(reviews_path)))
            break
    # Bind the rewritten guard records, while carrying their unchanged original verdict fingerprints.
    envelope = dict(candidate_fingerprints={r['slot_id']: candidate_fingerprint(r) for r in chosen.values()},
                    verdicts=[envelope_row(r, search['approvals']) for r in chosen.values()],
                    original_bindings=search['original_bindings'], animation_reviews_sha256=digest(reviews_path))
    envelope_path = output / 'envelope.json'
    atomic_json(envelope_path, envelope)
    prior_paths = sorted((root / 'docs/art/production5').glob('**/selection.json'))
    selections = [read(p) for p in prior_paths if p.resolve() != (output / 'selection.json').resolve()]
    selections += [read(root / p) for p in ('docs/art/production4/wood/selection_run/selection.json', 'docs/art/production4/recovery/selection_run/selection.json',
                  'docs/art/production3/reprocess/selection.json', 'docs/art/production3/installation_selection.json') if (root / p).exists()]
    baseline = deepcopy(selections[-1])
    for entry in baseline['slots'].values():
        entry.update(disposition='unfilled', candidate_id=None)
    current = current_sprites(root)
    replaced_groups = {tuple(p['group']) for p in approved_groups}
    for name, sprite in current.items():
        if name in chosen or sprite.qa_status not in ('accepted', 'derived') or (sprite.body, sprite.view, sprite.activity) in replaced_groups:
            continue
        decision = next((s['slots'][name] for s in selections if s['slots'].get(name, {}).get('candidate_id') == sprite.candidate_id), None)
        if decision is None:
            raise InstallError(f'Missing baseline approved provenance: {name}')
        baseline['slots'][name] = deepcopy(decision)
    for target, row in chosen.items():
        meta = {key: row[key] for key in ('body', 'view', 'activity', 'frame', 'name')}
        baseline['slots'][target] = dict(candidate_id=row['id'], disposition='derived' if row['source_kind'] == 'derived' else 'accepted',
            ledger=str(ledger_path), ledger_sha256=digest(ledger_path), verdict=str(envelope_path),
            verdict_sha256=digest(envelope_path), semantic=meta, **meta)
    selection_path = output / 'selection.json'
    atomic_json(selection_path, baseline)
    plan = build_plan(InstallRequest(root, selection_path))
    final = dict(current)
    final.update({s.slot_id: s for s in plan.sprites if s.slot_id in chosen})
    candidates = []
    for name, sprite in final.items():
        entry = baseline['slots'].get(name, {})
        if sprite.qa_status in ('accepted', 'derived') and entry.get('candidate_id'):
            row = next(r for r in read(root / entry['ledger'])['candidates'] if r['id'] == entry['candidate_id'])
            candidates.append((Target(name, sprite.body, sprite.view, sprite.activity, sprite.frame), guarded_candidate(row)))
    diagnostics = validate_assembled(tuple(final.values()), tuple(candidates), WorldScale())
    plan = replace(plan, sprites=tuple(final.values()), fallback_guard_diagnostics=diagnostics,
        suppressed={key: value for key, value in plan.suppressed.items() if key not in final})
    report = dict(selected={target: row['id'] for target, row in chosen.items()}, selection_sha256=plan.selection_sha256,
        groups=approved_groups, before=dict(Counter(s.qa_status for s in current.values())),
        after=dict(Counter(s.qa_status for s in final.values())),
        fallbacks_before=dict(Counter(s.fallback_kind for s in current.values() if s.qa_status == 'fallback')),
        fallbacks_after=dict(Counter(s.fallback_kind for s in final.values() if s.qa_status == 'fallback')),
        installer_validation='passed', applied=False)
    approved_ids = {p['id'] for p in approved_groups}
    for trial in search['trials']:
        identity = trial.get('id')
        if identity in approved_ids:
            trial.update(status='installed', reason='animation_ok', review_strikes=0)
        elif identity in reviews:
            trial.update(status='review_rejected', reason=reviews[identity].get('reason', 'animation_rejected'))
        elif not trial.get('reason'):
            trial.update(status='untried', reason='higher_ranked_completion_approved' if tuple(trial['group']) in replaced_groups else 'awaiting_animation_review')
    atomic_json(output / 'search.json', search)
    atomic_json(output / 'adjudication.json', report)
    atomic_json(output / 'installed_groups.json', {'groups': [dict(group=p['group'], source=p['source'],
        copied_frames=[records[i]['frame'] for i in p['candidate_ids'] if records[i]['source_kind'] == 'derived'],
        candidate_ids=p['candidate_ids'], animation_review=p['id']) for p in approved_groups]})
    return plan, report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('phase', choices=('prepare', 'finalize'))
    args = parser.parse_args()
    if args.phase == 'prepare':
        report = prepare()
        print(json.dumps({'proposals': len(report['proposals']), 'objects': report['objects']}))
    else:
        _, report = finalize()
        print(json.dumps(report['after']))


if __name__ == '__main__':
    main()
