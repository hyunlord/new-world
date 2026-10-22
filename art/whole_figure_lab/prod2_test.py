# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy"]
# ///
# How to run: python3 -m pytest art/whole_figure_lab/prod2_test.py
"""Synthetic boundary checks; these fixtures never certify production imagery."""
import itertools
import json
from pathlib import Path

import pytest

from art.whole_figure_lab.prod2_score import score
from art.whole_figure_lab.prod2_selection import choose, validate
from art.whole_figure_lab.samples import Person


def population() -> tuple[Person, ...]:
    return tuple(Person(sex, age, top, hair, 'idle') for age, sex, top, hair in itertools.product(
        ('adult', 'child'), ('M', 'F'), ('loincloth', 'hide_wrap', 'fur_cloak'),
        ('short', 'long_tied', 'bun')))


def test_selection_is_order_independent_and_certified() -> None:
    chosen = choose(population())
    assert chosen == choose(tuple(reversed(population())))
    validate(chosen)
    assert len({p.look for p in chosen}) == 12
    assert sum(p.age == 'adult' and p.sex == 'F' for p in chosen) == 5
    assert sum(p.age == 'adult' and p.sex == 'M' for p in chosen) == 5
    assert sum(p.age == 'child' and p.sex == 'F' for p in chosen) == 1
    assert sum(p.age == 'child' and p.sex == 'M' for p in chosen) == 1
    assert sum(p.age == 'adult' and p.sex == 'F' for p in chosen) >= 4
    assert sum(p.age == 'child' for p in chosen) >= 2


@pytest.mark.parametrize('omission', ['children', 'adult_female', 'top', 'hair'])
def test_ineligible_populations_fail(omission: str) -> None:
    predicates = {
        'children': lambda p: p.age != 'child',
        'adult_female': lambda p: p.age != 'adult' or p.sex != 'F',
        'top': lambda p: p.top != 'fur_cloak',
        'hair': lambda p: p.hair != 'bun',
    }
    with pytest.raises(ValueError, match='cannot supply'):
        choose(tuple(p for p in population() if predicates[omission](p)))


def fixtures(tmp_path: Path) -> tuple[Path, Path, Path]:
    key, review = [], []
    for number, person in enumerate(choose(population()), 1):
        for zoom in (1, 2):
            values = {'activity': person.activity, 'item': 'none', 'top': person.top,
                      'hair': person.hair, 'sex': person.sex, 'age': person.age}
            key.append({'number': number, 'zoom': zoom, **values})
            review.append({'id': f'p1_{number:02d}', 'zoom': zoom, **values})
    paths = tuple(tmp_path / name for name in ('key.json', 'a.json', 'b.json'))
    for path, values in zip(paths, (key, review, review), strict=True):
        path.write_text(json.dumps(values))
    return paths[0], paths[1], paths[2]


def test_child_sex_is_unscored_and_not_regenerated(tmp_path: Path) -> None:
    key, a, b = fixtures(tmp_path)
    rows = json.loads(a.read_text())
    for row in rows:
        if row['age'] == 'child':
            row['sex'] = 'unknown'
    a.write_text(json.dumps(rows))
    score(key, (a, b), tmp_path / 'scores')
    result = json.loads((tmp_path / 'scores/scores.json').read_text())
    assert result['all_numeric_bars_met']
    assert result['misread_looks'] == []
    assert not result['targeted_retry_available']
    bar = result['reviewers'][0]['summaries'][0]['bars']['sex_adult_age']
    assert bar == {'correct': 22, 'denominator': 22, 'bar': 19, 'passed': True}
    assert (tmp_path / 'scores/reviewer_1.verbatim.json').read_bytes() == a.read_bytes()


@pytest.mark.parametrize('misses,passes', [(3, True), (4, False)])
def test_d1_threshold_uses_exact_83_percent(tmp_path: Path, misses: int, passes: bool) -> None:
    key, a, b = fixtures(tmp_path)
    rows = json.loads(a.read_text())
    for row in rows:
        if int(row['id'][-2:]) <= misses:
            row['age'] = 'unknown'
    a.write_text(json.dumps(rows))
    score(key, (a, b), tmp_path / 'scores')
    result = json.loads((tmp_path / 'scores/scores.json').read_text())
    assert result['all_numeric_bars_met'] is passes


def test_unknown_mapping_and_second_reviewer_failure_are_binding(tmp_path: Path) -> None:
    key, a, b = fixtures(tmp_path)
    rows = json.loads(b.read_text())
    for row in rows:
        if row['zoom'] == 2:
            row['item'] = 'stone axe or hammer'
            row['notes'] = 'stone_axe'
    b.write_text(json.dumps(rows))
    score(key, (a, b), tmp_path / 'scores', 1)
    result = json.loads((tmp_path / 'scores/scores.json').read_text())
    assert not result['all_numeric_bars_met']
    assert not result['targeted_retry_available']
    assert result['reviewers'][1]['summaries'][1]['bars']['item_top_hair']['correct'] == 24
    with pytest.raises(ValueError, match='Only initial'):
        score(key, (a, b), tmp_path / 'scores', 2)


def test_score_rejects_diagnostic_cohort_and_missing_review(tmp_path: Path) -> None:
    key, a, b = fixtures(tmp_path)
    rows = json.loads(a.read_text())
    a.write_text(json.dumps(rows[:-1]))
    with pytest.raises(ValueError, match='Missing specimen'):
        score(key, (a, b), tmp_path / 'scores')
    a.write_text(json.dumps(rows))
    truth = json.loads(key.read_text())
    for row in truth:
        row['age'] = 'adult'
    key.write_text(json.dumps(truth))
    with pytest.raises(ValueError, match='Certified cohort'):
        score(key, (a, b), tmp_path / 'scores')


def test_selection_fallback_keeps_minima_when_sex_balance_impossible() -> None:
    people = tuple(p for p in population() if p.sex == 'F')
    selected = choose(people)
    validate(selected)
    assert sum(p.age == 'child' for p in selected) >= 2
    assert sum(p.age == 'adult' for p in selected) >= 4
