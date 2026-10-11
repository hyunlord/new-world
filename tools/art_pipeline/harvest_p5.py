# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: imported by merge_p5_selection.py; tests: python3 -m unittest discover -s tools/art_pipeline -p test_harvest_p5.py
"""Enumerate immutable-review-bound physical sets, never Cartesian slot mixtures."""
from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass
from pathlib import Path
from typing import Final, TypedDict

from palette_tools import Color
from salvage_v4 import CandidateRecord, _derive, guarded_candidate, guard_data
from salvage_verdicts import candidate_fingerprint
from v4_contracts import CYCLES, WorldScale, guard_frame, guard_group

GROUP: Final = tuple[str, str, str]


class OriginalBinding(TypedDict):
    fingerprint: str
    review_index: str
    review_path: str


@dataclass(frozen=True, slots=True)
class Completion:
    """One complete semantic animation from one physical sheet and attempt."""
    kind: str
    rows: tuple[CandidateRecord, ...]
    copies: int

    @property
    def group(self) -> tuple[str, str, str]:
        row = self.rows[0]
        return row['body'], row['view'], row['activity']

    @property
    def source(self) -> tuple[str, int]:
        row = self.rows[0]
        return row.get('source_sheet', row['sheet']), row['attempt']

    @property
    def rank(self) -> tuple[int, float, float, int, int]:
        errors = [row['metrics']['mean_delta_e'] for row in self.rows]
        return (self.copies, max(errors), sum(errors) / len(errors), self.source[1],
                ('C0', 'C1', 'C2').index(self.kind))


def binding_matches(record: CandidateRecord, fingerprints: dict[str, str]) -> bool:
    """A changed record or image loses the inherited verdict."""
    return fingerprints.get(record['id']) == candidate_fingerprint(record)


def normalize(record: CandidateRecord, scale: WorldScale) -> CandidateRecord:
    """Copy ledger history before replacing obsolete guard results with v5 results."""
    result = deepcopy(record)
    result['original_candidate_id'] = record['id']
    result['original_candidate_fingerprint'] = candidate_fingerprint(record)
    result['guards_p4'] = {key: deepcopy(record.get(key, {}))
                           for key in ('guards', 'group_guards', 'cross_guards')}
    result['guards'] = guard_data(guard_frame(guarded_candidate(result), scale))
    result['group_guards'] = guard_data(guard_group([guarded_candidate(result)], scale))
    result['cross_guards'] = {'errors': [], 'warnings': [], 'passed': True}
    return result


def copy_phase(target: CandidateRecord, source: CandidateRecord, cache: Path,
               palette: list[Color], scale: WorldScale) -> CandidateRecord:
    """Call the legacy derivation only when its same-phase branch is guaranteed."""
    if (target['activity'] not in CYCLES or target['frame'] % 2 != source['frame'] % 2 or
            (target.get('source_sheet', target['sheet']), target['attempt']) !=
            (source.get('source_sheet', source['sheet']), source['attempt'])):
        raise HarvestError('copy_requires_same_physical_set_and_phase')
    safe_target = deepcopy(target)
    safe_target['guards'] = deepcopy(source['guards'])
    result = _derive(safe_target, [source], cache, palette, scale)
    if result['derivation_rule'] != 'same_cycle_phase_copy':
        raise HarvestError('procedural_derivation_forbidden')
    # Source mask, fidelity and cleanup counts bind the copied pixels, not the bad target.
    if 'despeckle' in source:
        result['despeckle'] = deepcopy(source['despeckle'])
    result['original_candidate_id'] = source.get('original_candidate_id', source['id'])
    result['original_candidate_fingerprint'] = source.get('original_candidate_fingerprint', '')
    result['guards'] = guard_data(guard_frame(guarded_candidate(result), scale))
    return result


class HarvestError(ValueError):
    """Physical provenance or completion contract was violated."""


def completions(records: list[CandidateRecord], expected: dict[int, str], cache: Path,
                palette: list[Color], scale: WorldScale = WorldScale()) -> tuple[Completion, ...]:
    """Enumerate C0, each possible single same-phase repair, and carry two-pose sets."""
    if not records:
        return ()
    sources = {(r.get('source_sheet', r['sheet']), r['attempt']) for r in records}
    groups = {(r['body'], r['view'], r['activity']) for r in records}
    if len(sources) != 1 or len(groups) != 1:
        raise HarvestError('mixed_physical_set')
    usable = {r['frame']: r for r in records if guard_frame(guarded_candidate(r), scale).passed}
    original = {r['frame']: r for r in records}
    output = []
    if set(expected).issubset(usable):
        output.append(Completion('C0', tuple(usable[f] for f in sorted(expected)), 0))
    activity = records[0]['activity']
    if activity not in CYCLES or set(expected) != {0, 1, 2, 3}:
        return tuple(output)

    def target(frame: int) -> CandidateRecord:
        if frame in original:
            return original[frame]
        seed = deepcopy(records[0])
        physical_cell = seed.get('source_cell', seed['cell']) - seed['frame'] + frame
        sheet = seed.get('source_sheet', seed['sheet'])
        seed.update(id=f'{sheet}_{physical_cell:02d}@{seed["attempt"]}:missing',
                    slot_id=f'{sheet}_{physical_cell:02d}', cell=physical_cell,
                    source_cell=physical_cell, frame=frame, target_slot=expected[frame])
        return seed

    for frame in range(4):
        twin = (frame + 2) % 4
        if twin in usable and all(f in usable for f in expected if f != frame):
            replacement = copy_phase(target(frame), usable[twin], cache, palette, scale)
            output.append(Completion('C1', tuple(replacement if f == frame else usable[f]
                                                 for f in sorted(expected)), 1))
    if activity in ('carry', 'carry_wood'):
        for even in sorted(f for f in usable if f % 2 == 0):
            for odd in sorted(f for f in usable if f % 2 == 1):
                rows = []
                for frame in sorted(expected):
                    source = usable[even if frame % 2 == 0 else odd]
                    rows.append(source if frame == source['frame'] else copy_phase(
                        target(frame), source, cache, palette, scale))
                output.append(Completion('C2', tuple(rows), 2))
    return tuple(sorted(output, key=lambda completion: completion.rank))


def completion_guard(completion: Completion, scale: WorldScale) -> tuple[str, ...]:
    """Return all computed failures without consuming an animation-review strike."""
    candidates = [guarded_candidate(r) for r in completion.rows]
    report = guard_group(candidates, scale)
    errors = [error for candidate in candidates for error in guard_frame(candidate, scale).errors]
    errors.extend(report.errors)
    if not report.ready_for_review:
        errors.extend(report.warnings)
    return tuple(dict.fromkeys(errors))


def envelope_row(record: CandidateRecord, approvals: dict[str, dict]) -> dict:
    """An inherited approval names its reviewed source and the installed identity."""
    source = record.get('original_candidate_id', record['id'])
    row = deepcopy(approvals[source])
    if row['verdict'] != 'ok':
        raise HarvestError('unapproved_copy_source')
    row.update(sprite_id=record['slot_id'], candidate_id=record['id'], inherited_from=source)
    return row
