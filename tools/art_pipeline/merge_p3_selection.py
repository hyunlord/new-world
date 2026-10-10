# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/merge_p3_selection.py --help
from __future__ import annotations

import argparse
from copy import deepcopy
from dataclasses import dataclass
import json
from pathlib import Path
from tempfile import TemporaryDirectory

from install_v4 import InstallError, InstallRequest, build_plan
from merge_fill_selection import group_key, semantic
from salvage_v4 import CandidateRecord, guarded_candidate
from salvage_verdicts import REASONS, Verdict, atomic_json, candidate_fingerprint, sha
from v4_contracts import WorldScale, guard_frame


class MergeError(ValueError):
    pass


@dataclass(frozen=True, slots=True)
class Outputs:
    selection: Path
    envelope: Path
    adjudication: Path


def validate_reviews(records: dict[str, CandidateRecord], rows: list[Verdict]) -> dict[str, Verdict]:
    result: dict[str, Verdict] = {}
    for row in rows:
        identity = row['candidate_id']
        if identity not in records or identity in result:
            raise MergeError(f'Unknown or duplicate review identity: {identity}')
        if row['sprite_id'] != records[identity]['slot_id']:
            raise MergeError(f'Review physical slot mismatch: {identity}')
        if row['verdict'] not in ('ok', 'reject', 'unavailable') or not row['observation'].strip():
            raise MergeError(f'Invalid verdict or empty observation: {identity}')
        if row['verdict'] == 'reject' and row['reason'] not in REASONS:
            raise MergeError(f'Invalid rejection reason: {identity}')
        if row['verdict'] in ('ok', 'unavailable') and row['reason']:
            raise MergeError(f'Non-rejection reason must be empty: {identity}')
        if row['reason'] == 'detail_destroyed_by_fit' and type(row['unfitted_acceptable']) is not bool:
            raise MergeError(f'Detail rejection needs boolean unfitted judgement: {identity}')
        if row['verdict'] == 'unavailable' and not (records[identity].get('conversion_error')
                or not records[identity].get('images', {}).get('sprite')):
            raise MergeError(f'Unavailable verdict for usable candidate: {identity}')
        if row['unfitted_acceptable'] is not None and type(row['unfitted_acceptable']) is not bool:
            raise MergeError(f'Invalid unfitted judgement: {identity}')
        result[identity] = row
    if set(result) != set(records):
        raise MergeError(f'Missing physical candidate reviews: {sorted(set(records) - set(result))}')
    return result


def verify_review_index(index_path: Path, ledger_path: Path,
                        candidates: dict[str, CandidateRecord]) -> dict[str, str]:
    index = json.loads(index_path.read_text())
    if index['candidates_sha256'] != sha(ledger_path):
        raise MergeError('Pre-review ledger changed')
    fingerprints = {identity: candidate_fingerprint(row) for identity, row in candidates.items()}
    if index['candidate_fingerprints'] != fingerprints:
        raise MergeError('Pre-review candidate pixels or records changed')
    if not index['images_sha256']:
        raise MergeError('Pre-review index has no board hashes')
    bindings = {str(index_path): sha(index_path), str(ledger_path): sha(ledger_path)}
    for name, expected in index['images_sha256'].items():
        path = (index_path.parent / name).resolve()
        if sha(path) != expected:
            raise MergeError(f'Pre-review board changed: {path}')
        bindings[str(path)] = expected
    return bindings


def held_lineage(record: CandidateRecord, records: dict[str, CandidateRecord], holds: set[str]) -> bool:
    seen: set[str] = set()
    while True:
        identity = record['id']
        if identity in seen:
            raise MergeError(f'Cyclic lineage: {identity}')
        if identity in holds:
            return True
        seen.add(identity)
        ancestor = record.get('source_candidate')
        if not ancestor:
            return False
        if ancestor not in records:
            raise MergeError(f'Missing ancestor: {ancestor}')
        record = records[ancestor]


def merge(root: Path, baseline_path: Path, ledger_path: Path, review_paths: list[Path],
          outputs: Outputs, holds_path: Path | None = None, *, review_index: Path) -> None:
    root = root.resolve()
    baseline_path = (root / baseline_path).resolve()
    ledger_path = (root / ledger_path).resolve()
    review_paths = [(root / path).resolve() for path in review_paths]
    outputs = Outputs(*[(root / path).resolve() for path in
                        (outputs.selection, outputs.envelope, outputs.adjudication)])
    if len({outputs.selection, outputs.envelope, outputs.adjudication}) != 3:
        raise MergeError('Output paths must be distinct')
    if any(path.exists() for path in (outputs.selection, outputs.envelope, outputs.adjudication)):
        raise MergeError('Refusing to overwrite existing output')
    baseline = json.loads(baseline_path.read_text())
    ledger = json.loads(ledger_path.read_text())
    if ledger['scale'] != baseline['scale']:
        raise MergeError('Mixed world scales')
    records = {row['id']: row for row in ledger['candidates']}
    if len(records) != len(ledger['candidates']):
        raise MergeError('Duplicate candidate IDs in ledger')
    review_ids = ledger.get('reprocessed_ids', [key for key in records if key.endswith(':p3')])
    candidates = {key: records[key] for key in review_ids}
    if not candidates:
        raise MergeError('No :p3 candidates in ledger')
    review_index = (root / review_index).resolve()
    prereview_bindings = verify_review_index(review_index, ledger_path, candidates)
    rows = []
    for path in review_paths:
        rows.extend(json.loads(path.read_text()))
    reviews = validate_reviews(candidates, rows)
    hold_file = (root / holds_path).resolve() if holds_path else root / 'docs/art/production2/fill_sheets.json'
    holds = set(baseline.get('installation_holds', {}))
    if hold_file.exists():
        holds.update(json.loads(hold_file.read_text()).get('installation_holds', {}))
    holds.add('V_a1_01@1')
    scale = WorldScale(**baseline['scale'])
    preserved = {slot: entry for slot, entry in baseline['slots'].items()
                 if entry['disposition'] in ('accepted', 'derived')}
    existing = {}
    for slot, entry in preserved.items():
        path = root / entry['ledger']
        existing[slot] = next(row for row in json.loads(path.read_text())['candidates']
                              if row['id'] == entry['candidate_id'])
    options = {}
    excluded = {}
    for identity, record in candidates.items():
        target = record.get('target_slot', record['slot_id'])
        reason = ''
        if target in preserved:
            reason = 'preserve_existing_approval'
        elif reviews[identity]['verdict'] != 'ok':
            reason = 'visual_rejection'
        elif held_lineage(record, records, holds):
            reason = 'held_lineage'
        elif (not record['guards']['passed'] or record['guards']['errors']
              or any(record.get(key, {}).get('errors') for key in ('group_guards', 'cross_guards'))
              or not guard_frame(guarded_candidate(record), scale).passed):
            reason = 'technical_guard'
        if reason:
            excluded[identity] = reason
            continue
        key = group_key(record) if record['body'] else ('', target, '')
        source = (record.get('source_sheet', record['sheet']), record['attempt'])
        peers = [row for row in existing.values() if record['body'] and group_key(row) == key]
        if any((row.get('source_sheet', row['sheet']), row['attempt']) != source for row in peers):
            excluded[identity] = 'physical_source_conflicts_with_preserved_group'
            continue
        options.setdefault(key, {}).setdefault(source, []).append(record)
    chosen = {}
    trials = []
    source_hashes = {str(path): sha(path) for path in review_paths}
    fingerprints = {identity: candidate_fingerprint(record) for identity, record in candidates.items()}

    def materialize(selection_path: Path, envelope_path: Path) -> None:
        envelope = dict(candidates_sha256=sha(ledger_path), round=1,
                        candidate_fingerprints={row['slot_id']: fingerprints[row['id']] for row in chosen.values()},
                        verdicts=[reviews[row['id']] for row in chosen.values()], review_sources_sha256=source_hashes,
                        prereview_bindings_sha256=prereview_bindings)
        atomic_json(envelope_path, envelope)
        result = deepcopy(baseline)
        for target, row in chosen.items():
            meta = semantic(row)
            result['slots'][target] = dict(candidate_id=row['id'],
                disposition='accepted' if row['source_kind'] == 'generated' else 'derived',
                ledger=str(ledger_path), ledger_sha256=sha(ledger_path),
                verdict=str(envelope_path), verdict_sha256=sha(envelope_path), semantic=meta, **meta)
        result['accepted'] = {slot: entry['candidate_id'] for slot, entry in result['slots'].items()
                              if entry['disposition'] in ('accepted', 'derived')}
        result['merge_provenance'] = dict(baseline=str(baseline_path), baseline_sha256=sha(baseline_path),
                                        review_sources_sha256=source_hashes, preserved_slots=sorted(preserved))
        if any(result['slots'][slot] != entry for slot, entry in preserved.items()):
            raise MergeError('Existing approval changed')
        atomic_json(selection_path, result)

    with TemporaryDirectory(prefix='p3-selection-') as temporary:
        trial_selection = Path(temporary) / 'selection.json'
        trial_envelope = Path(temporary) / 'envelope.json'
        materialize(trial_selection, trial_envelope)
        build_plan(InstallRequest(root, trial_selection, holds=hold_file))
        for key, alternatives in sorted(options.items()):
            ranked = []
            for source, members in alternatives.items():
                by_target = {}
                for row in sorted(members, key=lambda row: (row['metrics']['mean_delta_e'], row['id'])):
                    by_target.setdefault(row.get('target_slot', row['slot_id']), row)
                ranked.append((source, by_target))
            ranked.sort(key=lambda option: (-len(option[1]),
                sum(row['metrics']['mean_delta_e'] for row in option[1].values()) / len(option[1]), option[0]))
            for source, additions in ranked[:3]:
                before = dict(chosen)
                physical = [row['slot_id'] for row in [*chosen.values(), *additions.values()]]
                if len(physical) != len(set(physical)):
                    trials.append(dict(group=key, source=source, accepted=False, reason='duplicate_physical_slot'))
                    continue
                chosen.update(additions)
                materialize(trial_selection, trial_envelope)
                try:
                    build_plan(InstallRequest(root, trial_selection, holds=hold_file))
                except InstallError as error:
                    chosen.clear()
                    chosen.update(before)
                    trials.append(dict(group=key, source=source, accepted=False, reason=str(error)))
                else:
                    trials.append(dict(group=key, source=source, accepted=True, targets=sorted(additions)))
                    break
    for identity in candidates:
        if identity not in {row['id'] for row in chosen.values()}:
            excluded.setdefault(identity, 'not_adopted_after_bounded_group_trials')
    verify_review_index(review_index, ledger_path, candidates)
    materialize(outputs.selection, outputs.envelope)
    build_plan(InstallRequest(root, outputs.selection, holds=hold_file))
    atomic_json(outputs.adjudication, dict(authoritative_sha256=sha(outputs.envelope),
        candidates_sha256=sha(ledger_path), review_sources_sha256=source_hashes,
        reviewed_candidate_fingerprints=fingerprints, full_review_verdicts=rows,
        reviewed_count=len(reviews), unavailable_count=sum(row['verdict'] == 'unavailable' for row in rows),
        prereview_bindings_sha256=prereview_bindings, accepted={row['slot_id']: row['id'] for row in chosen.values()},
        accepted_targets={target: row['id'] for target, row in chosen.items()}, exclusions=excluded,
        trials=trials, preserved_slots=sorted(preserved), installer_validation='passed'))


def main() -> None:
    parser = argparse.ArgumentParser(description='Add freshly approved P3 cells without replacing prior approvals.')
    parser.add_argument('--root', type=Path, default=Path.cwd())
    parser.add_argument('--baseline', type=Path, required=True)
    parser.add_argument('--ledger', type=Path, required=True)
    parser.add_argument('--review-index', type=Path, required=True)
    parser.add_argument('--review', type=Path, action='append', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--envelope', type=Path, required=True)
    parser.add_argument('--adjudication', type=Path, required=True)
    parser.add_argument('--holds', type=Path)
    args = parser.parse_args()
    merge(args.root, args.baseline, args.ledger, args.review,
          Outputs(args.output, args.envelope, args.adjudication), args.holds, review_index=args.review_index)


if __name__ == '__main__':
    main()
