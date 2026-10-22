"""Scoring boundaries and raw-evidence preservation, with artificial answers."""
import json
from pathlib import Path

import pytest

from art.whole_figure_lab.prod1_score import mapped, read_rows, score


def fixture_files(tmp_path: Path) -> tuple[Path, Path, Path]:
    key = []
    review = []
    for number in range(1, 13):
        for zoom in (1, 2):
            fields = {'activity': 'work', 'item': 'stone_axe', 'top': 'hide_wrap',
                      'hair': 'bun', 'sex': 'F', 'age': 'adult'}
            key.append({'number': number, 'zoom': zoom, **fields})
            review.append({'id': f'p1_{number:02d}', 'zoom': zoom, **fields})
    paths = (tmp_path / 'key.json', tmp_path / 'a.json', tmp_path / 'b.json')
    for path, rows in zip(paths, (key, review, review), strict=True):
        path.write_text(json.dumps(rows, indent=2) + '\n')
    return paths


@pytest.mark.parametrize('field,raw', [
    ('item', 'stone axe or hammer'), ('top', 'cloak or blanket'),
    ('hair', 'tied-up hair'), ('sex', 'unknown'), ('top', 'new apparently reasonable wrap'),
])
def test_unregistered_ambiguity_is_wrong(field: str, raw: str) -> None:
    assert mapped(field, raw, 'adult') == 'unknown'


def test_historical_equivalence_preserves_age_boundary() -> None:
    assert mapped('top', 'bare chest, tan waist wrap', 'adult') == 'loincloth'
    assert mapped('top', 'bare chest, tan waist wrap', 'child') == 'unknown'
    assert mapped('hair', 'dark brown topknot or bun', 'adult') == 'bun'


@pytest.mark.parametrize('mutation', ['missing', 'duplicate', 'field'])
def test_incomplete_review_fails(tmp_path: Path, mutation: str) -> None:
    _, source, _ = fixture_files(tmp_path)
    rows = json.loads(source.read_text())
    match mutation:
        case 'missing':
            rows.pop()
        case 'duplicate':
            rows[-1] = rows[0]
        case 'field':
            del rows[0]['item']
    source.write_text(json.dumps(rows))
    with pytest.raises(ValueError):
        read_rows(source)


def test_unknown_stays_wrong_despite_notes_and_raw_is_preserved(tmp_path: Path) -> None:
    key, a, b = fixture_files(tmp_path)
    rows = json.loads(a.read_text())
    for row in rows:
        row['item'] = 'unknown'
        row['notes'] = 'Definitely a stone_axe and all other answers are certain'
    a.write_text(json.dumps(rows) + '\n')
    output = tmp_path / 'scores'
    score(key, (a, b), output)
    result = json.loads((output / 'scores.json').read_text())
    assert not result['all_numeric_bars_met']
    assert result['targeted_retry_available']
    assert result['misread_looks'] == [{'look': 'adult_F_hide_wrap_bun', 'facings': ['S', 'E'], 'fields': ['item']}]
    first = result['reviewers'][0]
    assert first['cases'][0]['fields']['item'] == {'raw': 'unknown', 'mapped': 'unknown',
        'truth': 'stone_axe', 'correct': False, 'mapping_status': 'UNKNOWN_PENDING_EXPLICIT_EQUIVALENCE_REVIEW'}
    assert first['summaries'][0]['bars']['item_top_hair']['correct'] == 24
    assert first['summaries'][0]['bars']['item_top_hair']['denominator'] == 36
    assert (output / 'reviewer_1.verbatim.json').read_bytes() == a.read_bytes()


def test_each_reviewer_each_scale_must_pass_and_no_third_round(tmp_path: Path) -> None:
    key, a, b = fixture_files(tmp_path)
    output = tmp_path / 'scores'
    score(key, (a, b), output, 1)
    result = json.loads((output / 'scores.json').read_text())
    assert result['all_numeric_bars_met']
    assert not result['targeted_retry_available']
    for reviewer in result['reviewers']:
        assert [s['bars']['activity']['bar'] for s in reviewer['summaries']] == [8, 10]
    with pytest.raises(ValueError, match='Only initial'):
        score(key, (a, b), output, 2)
