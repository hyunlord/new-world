# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
"""Build conforming PROD1 evidence exclusively from verified accepted frames."""
from __future__ import annotations

import itertools
import json
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.pipeline3.capture_lab import gif_from_frames

from .build import ROOT, sha
from .samples import FRAMES, STATES, Person

FACINGS: Final = ('S', 'E')
STATUS: Final = 'STRUCTURAL_PANEL_PASS_REVIEW_PENDING'


def load(route: Path, look: str, facing: str, state: str) -> Image.Image:
    """Reject missing, modified, rejected, or incorrectly normalized inputs."""
    path = route / look / facing / f'{state}.png'
    sidecar = json.loads(path.with_suffix('.json').read_text())
    if (sidecar['image_sha256'] != sha(path)
            or sidecar['checks']['passed'] is not True
            or sidecar['judge']['numeric_pass'] is not True
            or (sidecar['look'], sidecar['facing'], sidecar['frame']) != (look, facing, state)):
        raise ValueError(f'Unaccepted or modified PROD1 frame: {path}')
    with Image.open(path) as image:
        expected = (112, 80) if state == 'sleep' else (80, 112)
        if image.mode != 'RGBA' or image.size != expected:
            raise ValueError(f'Nonconforming PROD1 canvas: {path}')
        return image.copy()


def accepted(route: Path) -> tuple[Person, ...]:
    """Enumerate complete look identities; partial or rejected looks are excluded."""
    result = []
    for age, sex, top, hair in itertools.product(
            ('adult', 'child'), ('M', 'F'), ('loincloth', 'hide_wrap', 'fur_cloak'),
            ('short', 'long_tied', 'bun')):
        person = Person(sex, age, top, hair, 'idle')
        try:
            for facing, state in itertools.product(FACINGS, STATES):
                load(route, person.look, facing, state)
        except (FileNotFoundError, ValueError, KeyError):
            continue
        result.append(person)
    return tuple(result)


def choose(people: tuple[Person, ...]) -> tuple[Person, ...]:
    """Deterministic coverage-first selection, failing if quotas cannot be met."""
    chosen: list[Person] = []
    remaining = list(people)
    while remaining and len(chosen) < 12:
        def gain(person: Person) -> int:
            return (int(person.sex == 'F' and sum(p.sex == 'F' for p in chosen) < 3)
                    + int(person.age == 'child' and sum(p.age == 'child' for p in chosen) < 2)
                    + int(person.top not in {p.top for p in chosen})
                    + int(person.hair not in {p.hair for p in chosen}))
        person = max(remaining, key=gain)
        remaining.remove(person)
        chosen.append(person)
    if (len(chosen) != 12 or sum(p.sex == 'F' for p in chosen) < 3
            or sum(p.age == 'child' for p in chosen) < 2
            or len({p.top for p in chosen}) != 3 or len({p.hair for p in chosen}) != 3):
        raise ValueError('Accepted set cannot supply 12 looks, 3 female, 2 children, all tops/hairs')
    activities = ('idle', 'walk', 'work', 'carry', 'sleep')
    return tuple(Person(p.sex, p.age, p.top, p.hair, activities[i % 5]) for i, p in enumerate(chosen))


def ground(name: str) -> Image.Image:
    """Crop unchanged calm-ground source pixels to the production stage."""
    with Image.open(ROOT / f'art/style05/ground_v2/{name}.png') as image:
        return image.convert('RGBA').crop((0, 0, 112, 112))


def stage(sprite: Image.Image, name: str, bob: int = 0) -> Image.Image:
    """Anchor complete frames at ground y104 with a lab-drawn 28×8 shadow."""
    result = ground(name)
    shadow = Image.new('RGBA', result.size)
    ImageDraw.Draw(shadow).ellipse((42, 100, 69, 107), fill=(24, 17, 10, 64))
    result.alpha_composite(shadow)
    foot = 72 if sprite.size == (112, 80) else 104
    result.alpha_composite(sprite, ((112 - sprite.width) // 2, 104 - foot + bob))
    return result.convert('RGB')


def panel(route: Path, person: Person, state: str, bob: int = 0) -> Image.Image:
    """S/E columns, meadow/forest rows; no scaling of normalized sprites."""
    result = Image.new('RGB', (224, 224))
    for column, facing in enumerate(FACINGS):
        sprite = load(route, person.look, facing, state)
        for row, name in enumerate(('meadow', 'forest_floor')):
            result.paste(stage(sprite, name, bob), (112 * column, 112 * row))
    return result


def build(route: Path, output: Path) -> None:
    """Preflight the complete accepted set before emitting any evidence."""
    people = choose(accepted(route))
    for name in ('gifs', 'textures', 'stills', 'boards'):
        (output / name).mkdir(parents=True, exist_ok=True)
    answers = []
    textures = []
    hashes: dict[str, str] = {}
    for number, person in enumerate(people, 1):
        for facing in FACINGS:
            files = []
            for state in STATES:
                source = route / person.look / facing / f'{state}.png'
                hashes[str(source.resolve())] = sha(source)
                hashes[str(source.with_suffix('.json').resolve())] = sha(source.with_suffix('.json'))
                destination = output / 'textures' / f'p1_{number:02d}_{facing}_{state}.png'
                load(route, person.look, facing, state).save(destination)
                files.append(destination.name)
            textures.append({'number': number, 'facing': facing, 'activity': person.activity, 'files': files})
        if number <= 6:
            state, _ = FRAMES[person.activity][1]
            load(route, person.look, 'S', state).save(output / 'stills' / f'{number:02d}.png')
        for zoom in (1, 2):
            directory = output / f'frames_{number:02d}_{zoom}x'
            directory.mkdir(exist_ok=True)
            for index, (state, bob) in enumerate(FRAMES[person.activity]):
                image = panel(route, person, state, bob)
                image.resize((224 * zoom, 224 * zoom), Image.Resampling.NEAREST).save(directory / f'frame_{index:02d}.png')
            target = output / 'gifs' / f'p1_{number:02d}_{zoom}x.gif'
            gif_from_frames(directory, target)
            answers.append({'number': number, 'zoom': zoom, **asdict(person),
                            'item': {'work': 'stone_axe', 'carry': 'log_bundle'}.get(person.activity, 'none'),
                            'file': target.name, 'sha256': sha(target)})
    for zoom in (1, 2):
        board = Image.new('RGB', (896 * zoom, 672 * zoom))
        for index, person in enumerate(people):
            state, bob = FRAMES[person.activity][1]
            tile = panel(route, person, state, bob).resize((224 * zoom, 224 * zoom), Image.Resampling.NEAREST)
            board.paste(tile, (index % 4 * 224 * zoom, index // 4 * 224 * zoom))
        board.save(output / 'boards' / f'lab_p1_{zoom}x.png', optimize=True)
    for name in ('meadow', 'forest_floor'):
        ground(name).save(output / f'{name}.png')
    (output / 'answer_key.private.json').write_text(json.dumps(answers, indent=2) + '\n')
    (output / 'crowd_manifest.json').write_text(json.dumps({'dataset_status': STATUS, 'sprites': textures}, indent=2) + '\n')
    (output / 'DATASET_STATUS.txt').write_text(STATUS + '\n')
    from .prod1_boards import boards
    boards(route, output / 'boards')
    (output / 'provenance.json').write_text(json.dumps({'dataset_status': STATUS,
        'input_hashes': hashes, 'selection': [asdict(p) for p in people], 'frame_plan': FRAMES,
        'canvas': [80, 112], 'sleep_canvas': [112, 80], 'footline': 104, 'sleep_footline': 72,
        'shadow': [28, 8], 'method': 'whole sprite translation; lab shadow; GIF encoding only'}, indent=2) + '\n')


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('Usage: python3 -m art.whole_figure_lab.prod1 ROUTE OUTPUT')
    build(Path(sys.argv[1]), Path(sys.argv[2]))
