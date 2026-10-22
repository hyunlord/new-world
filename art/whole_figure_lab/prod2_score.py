# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m art.whole_figure_lab.prod2_score KEY REVIEW_A REVIEW_B OUTPUT [ROUND]
"""PROD2 D1 scoring; all language equivalences remain the imported PROD1 rules."""
from __future__ import annotations

import json
import sys
from pathlib import Path

from .prod1_score import (
    ALIAS_PROVENANCE,
    CANONICAL,
    FIELDS,
    HISTORICAL_ALIASES,
    HISTORICAL_SHA256,
    HISTORICAL_SOURCE,
    MAPPING,
    Row,
    digest,
    mapped,
    read_rows,
)
from .prod2_selection import validate
from .prod2_types import Case, FieldScore, Review, Scale
from .samples import Person


def validate_key(rows: tuple[Row, ...]) -> None:
    """Reject diagnostic or unbalanced identities before computing formal bars."""
    validate(tuple(Person(r.answers['sex'], r.answers['age'], r.answers['top'],
                          r.answers['hair'], r.answers['activity']) for r in rows if r.zoom == 1))


def score(key_path: Path, reviewer_paths: tuple[Path, Path], output: Path, round_number: int = 0) -> None:
    """Write exact raw evidence and all per-field decisions; never authorize a third round."""
    if round_number not in (0, 1):
        raise ValueError('Only initial round 0 and one targeted round 1 are allowed')
    if reviewer_paths[0].resolve() == reviewer_paths[1].resolve():
        raise ValueError('Two distinct reviewer response files required')
    truth = {(r.number, r.zoom): r for r in read_rows(key_path, key=True)}
    validate_key(tuple(truth.values()))
    reviews = [read_rows(p) for p in reviewer_paths]
    summaries: list[Review] = []
    targets: dict[str, set[str]] = {}
    for reviewer, (path, rows) in enumerate(zip(reviewer_paths, reviews, strict=True), 1):
        cases: list[Case] = []
        for row in rows:
            expected = truth[(row.number, row.zoom)]
            fields: dict[str, FieldScore] = {}
            for field in FIELDS:
                raw = row.answers[field]
                value = mapped(field, raw, expected.answers['age'])
                correct = value != 'unknown' and value == expected.answers[field]
                fields[field] = {'raw': raw, 'mapped': value, 'truth': expected.answers[field], 'correct': correct,
                    'mapping_status': 'REGISTERED' if value != 'unknown' else 'UNKNOWN_PENDING_EXPLICIT_EQUIVALENCE_REVIEW'}
                if not correct and (field != 'sex' or expected.answers['age'] == 'adult'):
                    targets.setdefault(expected.look, set()).add(field)
            cases.append({'id': f'p1_{row.number:02d}', 'zoom': row.zoom, 'look': expected.look, 'fields': fields})
        scales: list[Scale] = []
        for zoom in (1, 2):
            counts = {field: sum(c['fields'][field]['correct'] for c in cases if c['zoom'] == zoom) for field in FIELDS}
            adults = sum(c['fields']['age']['truth'] == 'adult' for c in cases if c['zoom'] == zoom)
            adult_sex = sum(c['fields']['sex']['correct'] for c in cases if c['zoom'] == zoom and c['fields']['age']['truth'] == 'adult')
            bars = {'activity': (counts['activity'], 12, 8 if zoom == 1 else 10),
                    'item_top_hair': (counts['item'] + counts['top'] + counts['hair'], 36, 30),
                    'sex_adult_age': (adult_sex + counts['age'], adults + 12, (83 * (adults + 12) + 99) // 100)}
            scales.append({'zoom': zoom, 'field_correct': counts,
                'bars': {name: {'correct': values[0], 'denominator': values[1], 'bar': values[2],
                               'passed': values[0] >= values[2]} for name, values in bars.items()},
                'all_numeric_bars_met': all(values[0] >= values[2] for values in bars.values())})
        summaries.append({'reviewer': reviewer, 'raw_path': str(path.resolve()), 'raw_sha256': digest(path),
                          'verbatim_copy': f'reviewer_{reviewer}.verbatim.json', 'summaries': scales, 'cases': cases})
    output.mkdir(parents=True, exist_ok=True)
    for number, path in enumerate(reviewer_paths, 1):
        _ = (output / f'reviewer_{number}.verbatim.json').write_bytes(path.read_bytes())
    result = {'schema': 2, 'planner_decision': 'D1: sex adults only; age all; ratio >= 0.83', 'round': round_number, 'answer_key': str(key_path.resolve()),
        'answer_key_sha256': digest(key_path), 'historical_mapping_source': HISTORICAL_SOURCE,
        'historical_mapping_sha256': HISTORICAL_SHA256, 'mapping_dictionary': MAPPING,
        'canonical_aliases': CANONICAL, 'historical_aliases': HISTORICAL_ALIASES,
        'alias_provenance': ALIAS_PROVENANCE, 'reviewers': summaries,
        'all_numeric_bars_met': all(s['all_numeric_bars_met'] for r in summaries for s in r['summaries']),
        'eligibility': 'Numeric scores only; freshness, blind access, and accepted-source provenance require independent audit',
        'notes_policy': 'Notes never alter classifications; unregistered phrases are unknown and wrong',
        'targeted_retry_available': round_number == 0 and bool(targets),
        'misread_looks': [{'look': look, 'facings': ['S', 'E'], 'fields': sorted(fields)} for look, fields in sorted(targets.items())],
        'retry_rule': 'One targeted round only; generation try ceiling must be independently checked; no third review round'}
    _ = (output / 'scores.json').write_text(json.dumps(result, indent=2) + '\n')


if __name__ == '__main__':
    if len(sys.argv) not in (5, 6):
        raise SystemExit('Usage: python3 -m art.whole_figure_lab.prod2_score KEY REVIEW_A REVIEW_B OUTPUT [ROUND_0_OR_1]')
    score(Path(sys.argv[1]), (Path(sys.argv[2]), Path(sys.argv[3])), Path(sys.argv[4]), int(sys.argv[5]) if len(sys.argv) == 6 else 0)
