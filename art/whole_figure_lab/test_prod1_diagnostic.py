"""Diagnostic fallbacks retain absent/rejected truth and never install assets."""
import json
from pathlib import Path

import pytest
from PIL import Image

from art.whole_figure_lab.build import sha
from art.whole_figure_lab.prod1_crowd_inputs import build as build_crowd
from art.whole_figure_lab.prod1_diagnostic import STATUS, Selection, load, select
from art.whole_figure_lab.samples import PEOPLE


def test_missing_frame_uses_labeled_conforming_blank() -> None:
    selection = Selection(PEOPLE[0].look, None, None, 0, '{}')
    image, record = load(selection, 'S', 'sleep')
    assert image.size == (112, 80)
    assert image.getbbox() is not None
    assert record['display'] == 'MISSING'
    assert record['formal_R1_eligible'] is False
    assert record['dataset_status'] == STATUS


def test_best_attempt_uses_panel_count_then_earliest(tmp_path: Path) -> None:
    person = PEOPLE[0]
    for attempt, count in ((1, 5), (2, 8), (3, 8)):
        directory = tmp_path / person.look / f'try{attempt}'
        directory.mkdir(parents=True)
        (directory / 'sheet_checks.json').write_text(json.dumps({
            'look': person.look, 'attempt': attempt, 'passing_panels': count, 'passed': False}))
    chosen = select(tmp_path, person)
    assert chosen.attempt == 2
    assert json.loads(chosen.summary_json)['passed'] is False


def test_rejected_pixels_and_checks_preserved_exactly(tmp_path: Path) -> None:
    person = PEOPLE[0]
    directory = tmp_path / person.look / 'try1'
    (directory / 'S').mkdir(parents=True)
    path = directory / 'S' / 'idle.png'
    original = Image.new('RGBA', (80, 112), (30, 40, 50, 128))
    original.save(path)
    metadata = {'look': person.look, 'facing': 'S', 'frame': 'idle', 'attempt': 1,
                'image_sha256': sha(path), 'checks': {'passed': False, 'findings': ['coverage failed']},
                'judge': {'numeric_pass': False}}
    raw = json.dumps(metadata, indent=2) + '\n'
    path.with_suffix('.json').write_text(raw)
    selection = Selection(person.look, directory, 1, 0, '{}')
    image, record = load(selection, 'S', 'idle')
    assert image.tobytes() == original.tobytes()
    assert record['sidecar_json_verbatim'] == raw
    assert path.with_suffix('.json').read_text() == raw
    assert record['formal_R1_eligible'] is False


def test_raw_rejected_source_never_substitutes_for_missing_normalized(tmp_path: Path) -> None:
    (tmp_path / 'S').mkdir()
    Image.new('RGBA', (160, 224), 'red').save(tmp_path / 'S' / 'idle_rejected_source.png')
    image, record = load(Selection(PEOPLE[0].look, tmp_path, 1, 0, '{}'), 'S', 'idle')
    assert image.size == (80, 112)
    assert record['display'] == 'MISSING'
    assert record['source_sha256'] is None


def test_crowd_zero_accepted_fails_before_writes(tmp_path: Path) -> None:
    output = tmp_path / 'output'
    with pytest.raises(ValueError, match='at least one accepted'):
        build_crowd(tmp_path / 'absent', output)
    assert not output.exists()


def test_crowd_accepts_complete_subset_without_demographic_quota(tmp_path: Path) -> None:
    from art.whole_figure_lab.samples import STATES
    person = PEOPLE[0]
    route = tmp_path / 'route'
    for facing in ('S', 'E'):
        directory = route / person.look / facing
        directory.mkdir(parents=True)
        for state in STATES:
            path = directory / f'{state}.png'
            Image.new('RGBA', (112, 80) if state == 'sleep' else (80, 112)).save(path)
            path.with_suffix('.json').write_text(json.dumps({
                'look': person.look, 'facing': facing, 'frame': state,
                'image_sha256': sha(path), 'checks': {'passed': True},
                'judge': {'numeric_pass': True}}))
    output = tmp_path / 'crowd'
    build_crowd(route, output)
    record = json.loads((output / 'crowd_manifest.json').read_text())
    assert record['dataset_status'] == 'ACCEPTED_SUBSET_CROWD_ONLY'
    assert record['accepted_looks'] == [person.look]
    assert {r['activity'] for r in record['sprites']} == {'idle', 'walk', 'work', 'carry', 'sleep'}
    assert len(record['input_hashes']) == 20
    assert record['formal_R1_eligible'] is False
