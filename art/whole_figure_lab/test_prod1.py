"""Boundary tests use artificial pixels, never accepted production evidence."""
import itertools
import json
from pathlib import Path

import pytest
from PIL import Image

from art.whole_figure_lab.build import sha
from art.whole_figure_lab.prod1 import choose, load, stage
from art.whole_figure_lab.samples import Person


def test_selection_covers_required_attributes() -> None:
    people = tuple(Person(sex, age, top, hair, 'idle') for age, sex, top, hair in itertools.product(
        ('adult', 'child'), ('M', 'F'), ('loincloth', 'hide_wrap', 'fur_cloak'),
        ('short', 'long_tied', 'bun')))
    selected = choose(people)
    assert len({p.look for p in selected}) == 12
    assert sum(p.sex == 'F' for p in selected) >= 3
    assert sum(p.age == 'child' for p in selected) >= 2
    assert len({p.top for p in selected}) == len({p.hair for p in selected}) == 3
    assert {p.activity for p in selected} == {'idle', 'walk', 'work', 'carry', 'sleep'}


def test_selection_rejects_absent_or_unbalanced_sets() -> None:
    with pytest.raises(ValueError, match='Accepted set cannot'):
        choose(())


def test_sleep_canvas_retains_both_horizontal_ends() -> None:
    sprite = Image.new('RGBA', (112, 80))
    sprite.putpixel((0, 72), (255, 0, 255, 255))
    sprite.putpixel((111, 72), (255, 0, 255, 255))
    result = stage(sprite, 'meadow')
    assert result.getpixel((0, 104)) == result.getpixel((111, 104)) == (255, 0, 255)


def test_load_refuses_rejected_sidecar(tmp_path: Path) -> None:
    directory = tmp_path / 'adult_M_loincloth_short' / 'S'
    directory.mkdir(parents=True)
    path = directory / 'idle.png'
    Image.new('RGBA', (80, 112)).save(path)
    path.with_suffix('.json').write_text(json.dumps({'image_sha256': sha(path),
        'checks': {'passed': False}, 'judge': {'numeric_pass': True},
        'look': directory.parent.name, 'facing': 'S', 'frame': 'idle'}))
    with pytest.raises(ValueError, match='Unaccepted'):
        load(tmp_path, directory.parent.name, 'S', 'idle')
