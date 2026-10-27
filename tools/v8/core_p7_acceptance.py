"""Pure P7 exception boundary after an actual core_perf_check.check recomputation.

This module performs no IO. Manifest shape is not proof of raw member validity:
its caller must first run the existing checker, which authenticates all 66
accepted members, their loads, pair ordering, hashes and recomputed summaries.
Returned labels remain failures justified by P7 or existing report-only bars;
this function never converts them into an overall BARS_PASS or gate approval.
"""
from __future__ import annotations

import math
import re
from pathlib import PurePosixPath
from typing import Final

from core_perf_check import (
    CEILINGS, RSS_SCENES, SCENES, TICK_BAR, Document, EvidenceError, Json, require,
)

METRICS: Final = ('tick_mean_us', 'tick_p99_us')
REPORT_ONLY: Final = ('N1000 mind/old decision>0.5',
                      'N1000 tick ratio exceeds accepted arithmetic bar')


def positive(doc: Document, key: str) -> float:
    """Reject absent, boolean, nonfinite and nonpositive numeric evidence."""
    value = doc.number(key)
    require(value > 0, f'P7 nonpositive {key}')
    return value


def exact_integer(doc: Document, key: str, minimum: int) -> int:
    """Counts are integer JSON values, never floats or booleans."""
    value = doc.values.get(key)
    require(type(value) is int, f'P7 integer required: {key}')
    match value:
        case int() as count:
            require(count >= minimum, f'P7 invalid count: {key}')
            return count
        case _:
            raise EvidenceError(f'P7 integer required: {key}')


def ratio(doc: Document, old: str, new: str, field: str) -> float:
    """Require a finite positive ratio consistent with its numerator/denominator."""
    base, current, recorded = positive(doc, old), positive(doc, new), positive(doc, field)
    computed = current / base
    require(math.isfinite(computed) and math.isclose(recorded, computed, rel_tol=1e-12,
                                                   abs_tol=0.0),
            f'P7 inconsistent ratio: {field}')
    return computed


def accepted_manifest(receipt: Document) -> None:
    """Check all 33 pair slots without pretending file names prove valid loads."""
    expected = {(role, f'{kind}-{scene}-r{repeat}.json')
                for role in ('base', 'new')
                for kind, scenes, repeats in (('timed', SCENES, 3), ('rss', RSS_SCENES, 5))
                for scene in scenes for repeat in range(1, repeats + 1)}
    seen: set[tuple[str, str]] = set()
    paths: set[str] = set()
    roots: set[str] = set()
    for row in receipt.rows('files'):
        path = row.text('path')
        require(path not in paths, 'P7 duplicate manifest path')
        paths.add(path)
        require(re.fullmatch('[0-9a-f]{64}', row.text('sha256')) is not None,
                'P7 malformed manifest SHA256')
        parts = PurePosixPath(path).parts
        if 'accepted' not in parts:
            continue
        require(len(parts) >= 4 and parts[-3] == 'accepted' and '..' not in parts,
                'P7 malformed accepted path')
        slot = (parts[-2], parts[-1])
        require(slot in expected and slot not in seen, 'P7 extra/duplicate accepted member')
        roots.add(str(PurePosixPath(path).parents[2]))
        seen.add(slot)
    require(seen == expected and len(roots) == 1, 'P7 incomplete/mixed accepted matrix')


def check_paired(receipt: Json | Document) -> list[str]:
    """Return only authorized labels or raise EvidenceError; raw check is prerequisite."""
    doc = receipt if isinstance(receipt, Document) else Document.parse(receipt)
    require(exact_integer(doc, 'schema', 1) == 1, 'P7 unsupported receipt schema')
    require(exact_integer(doc, 'accepted_pairs', 0) == 33, 'P7 requires 33 pairs')
    exact_integer(doc, 'invalid_attempt_members', 0)
    require(doc.values.get('historical_ledger_used_as_base') is False,
            'P7 paired receipt must not use historical baseline')
    accepted_manifest(doc)
    require(doc.child('saturation_companion').text('status') == 'MEASURED',
            'P7 incomplete companion evidence')
    ceilings = doc.child('standing_c3').values
    require(set(ceilings) == set(CEILINGS) and all(v == 'PASS' for v in ceilings.values()),
            'P7 C3 missing/extra/failed')
    require(positive(doc, 'alloc_per_agent_ratio') <= 1.1, 'P7 memory regression')
    expected = {(s, m) for s in SCENES for m in METRICS}
    seen: set[tuple[str, str]] = set()
    derived: list[str] = []
    for row in doc.rows('standing_ratios'):
        scene, metric = row.text('scene'), row.text('metric')
        slot = (scene, metric)
        require(slot in expected and slot not in seen, 'P7 extra/duplicate standing metric')
        seen.add(slot)
        require(row.number('bar') == 1.1, 'P7 changed standing bar')
        measured = ratio(row, 'base', 'new', 'ratio')
        if measured > 1.1:
            require(scene == 'n200_side64', f'P7 non-side64 regression: {scene}/{metric}')
            derived.append(f'{scene}/{metric}>1.1')
    require(seen == expected, 'P7 incomplete standing matrix')
    require(doc.number('mind_ratio_bar') == .5 and doc.number('tick_ratio_bar') == TICK_BAR,
            'P7 changed report-only bars')
    comparisons: set[str] = set()
    for row in doc.rows('comparisons'):
        scene = row.text('scene')
        require(scene in ('n200', 'n1000', 'n2000') and scene not in comparisons,
                'P7 extra/duplicate comparison')
        comparisons.add(scene)
        mind = ratio(row, 'base_decision_mean_us', 'new_mind_mean_us', 'mind_ratio')
        tick = ratio(row, 'base_tick_mean_us', 'new_tick_mean_us', 'tick_ratio')
        if scene == 'n1000':
            if mind > .5:
                derived.append(REPORT_ONLY[0])
            if tick > TICK_BAR:
                derived.append(REPORT_ONLY[1])
    require(comparisons == {'n200', 'n1000', 'n2000'}, 'P7 incomplete comparisons')
    failures = doc.values.get('failures')
    require(isinstance(failures, list), 'P7 missing failure list')
    require(failures == derived, 'P7 missing/extra/reordered raw failures')
    require(doc.text('status') == ('FAIL' if derived else 'PASS'), 'P7 inconsistent raw status')
    return derived
