# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.whole_figure_lab.build ROUTE OUTPUT
"""Build source-hashed whole-figure evidence; reject absent or unchecked sprites."""
from __future__ import annotations

import hashlib
import json
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.pipeline3.capture_lab import gif_from_frames

from .samples import FRAMES, PEOPLE, STATES

ROOT: Final = Path(__file__).resolve().parents[2]


def sha(path: Path) -> str:
    """Hash current source bytes."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def ground(name: str) -> Image.Image:
    """Use unchanged calm-ground pixels and historical 104×112 stage."""
    with Image.open(ROOT / f'art/style05/ground_v2/{name}.png') as image:
        return image.convert('RGBA').crop((0, 0, 104, 112))


def stage(sprite: Image.Image, name: str, bob: int = 0) -> Image.Image:
    """Translate complete sprite only; never introduce shadow or part edits."""
    result = ground(name)
    result.alpha_composite(sprite, ((104 - sprite.width) // 2, 104 - sprite.height + bob))
    return result.convert('RGB')


def source_directory(route: Path, look: str, facing: str, diagnostic: bool) -> Path:
    """Diagnostic selection explicitly maps each identity to one attempt directory."""
    if diagnostic:
        selection = json.loads(route.read_text())
        directory = Path(selection['sheets'][f'{look}/{facing}'])
        if not directory.is_absolute():
            directory = route.parent / directory
        summary = json.loads((directory / 'sheet_checks.json').read_text())
        if (summary['look'], summary['facing']) != (look, facing):
            message = f'Diagnostic sheet identity mismatch: {directory}'
            raise ValueError(message)
        return directory
    return route / look / facing


def load(route: Path, look: str, facing: str, state: str, diagnostic: bool = False) -> Image.Image:
    """Require sidecar checks and image hash to describe these exact bytes."""
    path = source_directory(route, look, facing, diagnostic) / f'{state}.png'
    sidecar = json.loads(path.with_suffix('.json').read_text())
    if (sidecar['image_sha256'] != sha(path) or (not diagnostic and (not sidecar['checks']['passed']
            or sidecar['judge'].get('numeric_pass') is not True))
            or (sidecar['look'], sidecar['facing'], sidecar['frame']) != (look, facing, state)):
        message = f'Unchecked or modified whole figure: {path}'
        raise ValueError(message)
    if diagnostic:
        summary = json.loads((path.parent / 'sheet_checks.json').read_text())
        if (sidecar['attempt'], sidecar['raw_sha256']) != (summary['attempt'], summary['raw_sha256']):
            message = f'Diagnostic attempt does not match sheet: {path}'
            raise ValueError(message)
    with Image.open(path) as image:
        expected_size = (96, 64) if state == 'sleep' else (64, 96)
        if image.mode != 'RGBA' or image.size != expected_size:
            message = f'Not a normalized RGBA whole frame: {path}'
            raise ValueError(message)
        return image.copy()


def panel(route: Path, look: str, state: str, bob: int = 0, diagnostic: bool = False) -> Image.Image:
    """S/E columns and meadow/forest rows at exact native scale."""
    result = Image.new('RGB', (208, 240 if diagnostic else 224), '#e0bd8e')
    for col, facing in enumerate(('S', 'E')):
        sprite = load(route, look, facing, state, diagnostic)
        for row, name in enumerate(('meadow', 'forest_floor')):
            result.paste(stage(sprite, name, bob), (col * 104, row * 112))
    if diagnostic:
        ImageDraw.Draw(result).text((2, 225), 'DIAGNOSTIC - NOT ACCEPTED', fill='#18110a')
    return result


def boards(route: Path, output: Path, diagnostic: bool = False) -> None:
    """Cover 240 cells per scale with readable bounded pages, plus summary."""
    records = []
    panel_height = 240 if diagnostic else 224
    status = "DIAGNOSTIC_NOT_ACCEPTED" if diagnostic else "review pending"
    for zoom in (1, 2):
        for number, person in enumerate(PEOPLE, 1):
            board = Image.new('RGB', (1040 * zoom, (panel_height + 24) * zoom), '#e0bd8e')
            for column, state in enumerate(STATES):
                tile = panel(route, person.look, state, diagnostic=diagnostic).resize((208 * zoom, panel_height * zoom), Image.Resampling.NEAREST)
                board.paste(tile, (column * 208 * zoom, 24 * zoom))
            ImageDraw.Draw(board).text((4, 4), f'{status} | {number:02d} | idle / walk / work / carry / sleep | S,E; meadow,forest', fill='#18110a')
            target = output / f'lab_t5_{zoom}x_page{number:02d}.png'
            board.save(target, optimize=True)
            records.append({'file': target.name, 'number': number, 'zoom': zoom, 'states': STATES,
                            'facings': ['S', 'E'], 'grounds': ['meadow', 'forest_floor'], 'sha256': sha(target)})
        summary = Image.new('RGB', (832 * zoom, (panel_height * 3 + 24) * zoom), '#e0bd8e')
        for index, person in enumerate(PEOPLE):
            state, bob = FRAMES[person.activity][1]
            tile = panel(route, person.look, state, bob, diagnostic).resize((208 * zoom, panel_height * zoom), Image.Resampling.NEAREST)
            summary.paste(tile, (index % 4 * 208 * zoom, 24 * zoom + index // 4 * panel_height * zoom))
        ImageDraw.Draw(summary).text((4, 4), f'TRIAL5 {zoom}x | 01..12 row-major | {status}', fill='#18110a')
        summary.save(output / f'lab_t5_{zoom}x.png', optimize=True)
    (output / 'pages.json').write_text(json.dumps(records, indent=2) + '\n')


def build(route: Path, output: Path, diagnostic: bool = False) -> None:
    """Fail before writing outputs if any preregistered input is unavailable."""
    sources = []
    input_status = []
    sheet_status = []
    status = "diagnostic_not_accepted" if diagnostic else "STRUCTURAL_PASS_REVIEW_PENDING"
    for person in PEOPLE:
        for facing in ('S', 'E'):
            for state in STATES:
                load(route, person.look, facing, state, diagnostic)
                path = source_directory(route, person.look, facing, diagnostic) / f'{state}.png'
                sources.extend((path, path.with_suffix('.json')))
                input_status.append(json.loads(path.with_suffix('.json').read_text()))
            if diagnostic:
                summary_path = source_directory(route, person.look, facing, diagnostic) / 'sheet_checks.json'
                sources.append(summary_path)
                sheet_status.append(json.loads(summary_path.read_text()))
    hashes = {str(path.resolve()): sha(path) for path in sources}
    for name in ('gifs', 'stills', 'boards', 'textures'):
        (output / name).mkdir(parents=True, exist_ok=True)
    answers = []
    textures = []
    for number, person in enumerate(PEOPLE, 1):
        state, bob = FRAMES[person.activity][1]
        if number <= 6:
            load(route, person.look, 'S', state, diagnostic).save(output / 'stills' / f'{number:02d}.png')
        for facing in ('S', 'E'):
            files = []
            for state in STATES:
                destination = output / 'textures' / f't5_{number:02d}_{facing}_{state}.png'
                load(route, person.look, facing, state, diagnostic).save(destination)
                files.append(destination.name)
            textures.append({'number': number, 'facing': facing, 'activity': person.activity, 'files': files})
        for zoom in (1, 2):
            directory = output / f'frames_{number:02d}_{zoom}x'
            directory.mkdir(exist_ok=True)
            for index, (state, bob) in enumerate(FRAMES[person.activity]):
                frame = panel(route, person.look, state, bob, diagnostic)
                frame.resize((frame.width * zoom, frame.height * zoom), Image.Resampling.NEAREST).save(directory / f'frame_{index:02d}.png')
            gif = output / 'gifs' / f't5_{number:02d}_{zoom}x.gif'
            gif_from_frames(directory, gif)
            answers.append({'number': number, 'zoom': zoom, **asdict(person),
                            'item': {'work': 'stone_axe', 'carry': 'log_bundle'}.get(person.activity, 'none'),
                            'file': gif.name, 'sha256': sha(gif)})
    for name in ('meadow', 'forest_floor'):
        ground(name).save(output / f'{name}.png')
    (output / 'answer_key.private.json').write_text(json.dumps(answers, indent=2) + '\n')
    (output / 'crowd_manifest.json').write_text(json.dumps({'dataset_status': status, 'sprites': textures}, indent=2) + '\n')
    (output / 'DATASET_STATUS.txt').write_text(status + '\n')
    (output / 'provenance.json').write_text(json.dumps({'dataset_status': status, 'input_status': input_status, 'sheet_status': sheet_status, 'input_hashes': hashes,
        'frame_plan': FRAMES, 'selection_sha256': sha(Path(__file__).with_name('samples.py')),
        'diagnostic_selection_sha256': sha(route) if diagnostic else None,
        'ground_hashes': {name: sha(ROOT / f'art/style05/ground_v2/{name}.png') for name in ('meadow', 'forest_floor')},
        'method': 'whole sprite translation; no extra shadow; PNG unchanged; GIF palette encoding only'}, indent=2) + '\n')
    boards(route, output / 'boards', diagnostic)
    oversized = [str(path) for path in output.rglob('*') if path.suffix in ('.gif', '.png') and path.stat().st_size > 1_500_000]
    if oversized:
        message = f'File ceiling exceeded: {oversized}'
        raise ValueError(message)


if __name__ == '__main__':
    arguments = sys.argv[1:]
    diagnostic = '--diagnostic' in arguments
    if diagnostic:
        arguments.remove('--diagnostic')
    if len(arguments) != 2:
        raise SystemExit('Usage: python3 -m art.whole_figure_lab.build ROUTE_OR_SELECTION_JSON OUTPUT [--diagnostic]')
    build(Path(arguments[0]), Path(arguments[1]), diagnostic)
