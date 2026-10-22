# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.whole_figure_lab.prod2 ROUTE OUTPUT
"""Build PROD2 certified inputs using the unchanged PROD1 rendering contract."""
from __future__ import annotations

import json
import sys
from dataclasses import asdict
from pathlib import Path

from PIL import Image

from art.pipeline3.capture_lab import gif_from_frames

from .build import sha
from .prod1 import FACINGS, STATUS, accepted, ground, load, panel
from .prod1_score import read_rows
from .prod2_selection import choose, validate
from .prod2_types import Texture
from .samples import FRAMES, STATES, Person


def build(route: Path, output: Path, fixed_key: Path | None = None) -> None:
    """Preflight the complete accepted set before emitting any evidence."""
    available = accepted(route)
    people = choose(available)
    if fixed_key is not None:
        rows = sorted(read_rows(fixed_key, key=True), key=lambda r: (r.number, r.zoom))
        people = tuple(Person(r.answers['sex'], r.answers['age'], r.answers['top'],
            r.answers['hair'], r.answers['activity']) for r in rows if r.zoom == 1)
        validate(people)
        if {p.look for p in people} - {p.look for p in available}:
            message = 'Fixed review cohort contains unaccepted looks'
            raise ValueError(message)
    for name in ('gifs', 'textures', 'stills', 'boards'):
        (output / name).mkdir(parents=True, exist_ok=True)
    answers: list[dict[str, str | int]] = []
    textures: list[Texture] = []
    hashes: dict[str, str] = {}
    for number, person in enumerate(people, 1):
        for facing in FACINGS:
            files: list[str] = []
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
    _ = (output / 'answer_key.private.json').write_text(json.dumps(answers, indent=2) + '\n')
    _ = (output / 'crowd_manifest.json').write_text(json.dumps({'dataset_status': STATUS, 'sprites': textures}, indent=2) + '\n')
    _ = (output / 'DATASET_STATUS.txt').write_text(STATUS + '\n')
    from .prod1_boards import boards
    boards(route, output / 'boards')
    _ = (output / 'provenance.json').write_text(json.dumps({'dataset_status': STATUS,
        'fixed_key_sha256': sha(fixed_key) if fixed_key is not None else None,
        'input_hashes': hashes, 'selection': [asdict(p) for p in people], 'frame_plan': FRAMES,
        'canvas': [80, 112], 'sleep_canvas': [112, 80], 'footline': 104, 'sleep_footline': 72,
        'shadow': [28, 8], 'pass': 'ART-PROD2', 'selection_rule': 'attribute-stratified deterministic subset; prefer 5 adult F + 5 adult M + child F + child M; fallback >=4 adult F and >=2 children; all tops/hairs; fixed before review', 'method': 'whole sprite translation; lab shadow; GIF encoding only'}, indent=2) + '\n')


if __name__ == '__main__':
    if len(sys.argv) not in (3, 4):
        raise SystemExit('Usage: python3 -m art.whole_figure_lab.prod2 ROUTE OUTPUT [FIXED_PREVIOUS_KEY]')
    build(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]) if len(sys.argv) == 4 else None)
