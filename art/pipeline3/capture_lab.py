# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline3.capture_lab MANIFEST OUTPUT [all|grid|gifs|crowd]
"""Capture the real Godot lab after rechecking source bytes; encode GIF evidence."""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Literal

from PIL import Image

from .check_set import run as check_set
from .layer_schema import Layer, load_layer
from .set_checks import combinations

Activity = Literal['idle', 'walk', 'work', 'carry', 'sleep']
ACTIVITIES: tuple[Activity, ...] = ('idle', 'walk', 'work', 'carry', 'sleep', 'walk',
                                  'carry', 'work', 'sleep', 'idle', 'walk', 'carry')


def run_godot(manifest: Path, output: Path, arguments: list[str], seconds: int = 30) -> None:
    """Require a completion sentinel and clean renderer logs, not just exit zero."""
    repo = Path(__file__).resolve().parents[2]
    binary = os.environ.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot')
    route = json.loads(manifest.read_text())['route']
    command = [binary, '--path', str(repo / 'art/proto'), '--rendering-driver', 'opengl3',
               'res://scenes/character_lab.tscn', '--', '--route', route,
               '--root-' + route.lower(), str(manifest.parent), '--out', str(output), *arguments]
    output.mkdir(parents=True, exist_ok=True)
    (output / 'command.json').write_text(json.dumps(command, indent=2) + '\n')
    with (output / 'godot.log').open('w') as log_file:
        result = subprocess.run(command, stdout=log_file, stderr=subprocess.STDOUT,
                                text=True, timeout=seconds, check=False)
    log = (output / 'godot.log').read_text()
    if result.returncode or 'CHARACTER_LAB_DONE' not in log or any(
        marker in log for marker in ('SCRIPT ERROR', 'Parse Error', 'ERROR:')
    ):
        raise RuntimeError(f'Godot capture failed: {output / "godot.log"}')
    runtime = json.loads((output / 'runtime.json').read_text())
    if runtime['headless'] or runtime['rendered_frames'] <= 0:
        raise RuntimeError('No actual windowed rendering evidence')


def gif_from_frames(directory: Path, destination: Path) -> None:
    """Encode 8 fps as alternating 120/130 ms ticks (GIF uses 10 ms units).

    Palette encoding is GIF-only; the original RGBA PNG pixels stay unchanged.
    Eight frame delays sum to exactly 1000 ms, unlike rounded 125 ms delays.
    """
    frames: list[Image.Image] = []
    for index in range(8):
        with Image.open(directory / f'frame_{index:02d}.png') as source:
            frames.append(source.convert('RGB'))
    frames[0].save(destination, save_all=True, append_images=frames[1:], duration=[120, 130] * 4,
                   loop=0, disposal=2, optimize=False)
    if destination.stat().st_size > 2 * 1024 * 1024:
        raise RuntimeError(f'GIF exceeds 2 MB: {destination}')
    for frame in frames:
        frame.close()


def select_combos(layers: list[Layer]) -> list[tuple[int, Activity, tuple[str, ...]]]:
    """Pick twelve complete checked combinations, with activity-appropriate props."""
    by_id = {layer.id: layer for layer in layers}
    combos = combinations(layers)
    selected: list[tuple[int, Activity, tuple[str, ...]]] = []
    used: set[tuple[str, str, str, str, str]] = set()
    for activity in ACTIVITIES:
        choices: list[tuple[int, tuple[str, ...], tuple[str, str, str, str, str]]] = []
        for index, combo in enumerate(combos):
            body = by_id[combo.ids[0]]
            if body.body_type != 'adult':
                continue
            if activity == 'sleep':
                if body.frame == 'sleep':
                    choices.append((index, combo.ids, ('sleep', '', '', '', '')))
                continue
            if body.frame != 'idle':
                continue
            parts = [by_id[layer_id] for layer_id in combo.ids]
            has_axe = any(part.kind == 'item' for part in parts)
            has_carry = any(part.kind == 'carry' for part in parts)
            if has_axe != (activity == 'work') or has_carry != (activity == 'carry'):
                continue
            top = next(part.variant for part in parts if part.kind == 'top')
            hair = next(part.variant for part in parts if part.kind == 'hair')
            key = (activity, body.direction, top, hair, body.body_type)
            choices.append((index, combo.ids, key))
        if not choices:
            raise RuntimeError(f'No structurally accepted adult combination for {activity}')
        fresh = [choice for choice in choices if choice[2] not in used]
        choice = (fresh or choices)[len(selected) % len(fresh or choices)]
        used.add(choice[2])
        selected.append((choice[0], activity, choice[1]))
    return selected


def capture(manifest: Path, output: Path, mode: str, *, allow_incomplete: bool = False) -> None:
    """Collect one route without inventing visual, readability or timing results."""
    complete = check_set(manifest, manifest.parent)
    checks = json.loads((manifest.parent / 'CHECKS.json').read_text())
    diagnostic_valid = (
        allow_incomplete and checks.get('subset_passed') is True
        and checks.get('status') == 'NOT_COMPLETE'
        and checks.get('layer_count', 0) > 0
        and checks.get('layer_count') == checks.get('layer_pass_count')
        and bool(checks.get('combinations'))
        and all(result['passed'] or result['id'] == 'minimum_set' for result in checks['results'])
        and all(combo['passed'] for combo in checks['combinations'])
    )
    if not complete and not diagnostic_valid:
        raise RuntimeError('Structural checker rejected route; diagnostic capture also requires every supplied layer and combination to pass')
    diagnostic_args = ['--allow-incomplete', 'true'] if allow_incomplete else []
    raw = json.loads(manifest.read_text())
    route = raw['route'].lower()
    layers = [load_layer(manifest.parent / path) for path in raw['layers']]
    output.mkdir(parents=True, exist_ok=True)
    if mode in ('all', 'grid'):
        boards = output / 'boards'
        boards.mkdir(exist_ok=True)
        pages = []
        count = checks['combination_count']
        for zoom in (1, 2):
            capacity = (10 // zoom) * (896 // (112 * zoom))
            page_count = (count + capacity - 1) // capacity
            for ground in ('meadow', 'forest_floor'):
                covered: list[int] = []
                for page in range(page_count):
                    destination = output / f'grid_{ground}_{zoom}x_page{page + 1:03d}'
                    run_godot(manifest, destination,
                              [*diagnostic_args, '--mode', 'grid', '--zoom', str(zoom),
                               '--ground', ground, '--page', str(page), '--seconds', '1'])
                    runtime = json.loads((destination / 'runtime.json').read_text())
                    expected = list(range(page * capacity, min((page + 1) * capacity, count)))
                    if runtime['global_combination_indices'] != expected or runtime['draw_count'] != len(expected):
                        raise RuntimeError(f'Grid page coverage mismatch: {destination}')
                    if runtime['viewport'] != runtime['requested_viewport'] or runtime['viewport'][0] > 1200 or runtime['viewport'][1] > 900:
                        raise RuntimeError(f'Grid viewport clipped: {destination}')
                    source = destination / f'lab_{route}_{zoom}x.png'
                    if source.stat().st_size > 1_500_000:
                        raise RuntimeError(f'Grid PNG exceeds 1.5 MB without downscaling: {source}')
                    board = boards / f'lab_{route}_{zoom}x_{ground}_page{page + 1:03d}.png'
                    shutil.copyfile(source, board)
                    if page == 0 and ground == 'meadow':
                        shutil.copyfile(source, boards / f'lab_{route}_{zoom}x.png')
                    covered.extend(expected)
                    pages.append({'ground': ground, 'zoom': zoom, 'page': page,
                                  'indices': expected, 'draw_count': len(expected),
                                  'board': str(board), 'bytes': board.stat().st_size,
                                  'sha256': hashlib.sha256(board.read_bytes()).hexdigest(),
                                  'grid_rendered_fps': runtime['rendered_fps'],
                                  'dataset_status': runtime['dataset_status']})
                if covered != list(range(count)):
                    raise RuntimeError(f'Incomplete exhaustive grid coverage: {ground}/{zoom}')
        (boards / 'pages.json').write_text(json.dumps(pages, indent=2) + '\n')
    if mode in ('all', 'crowd'):
        run_godot(manifest, output / 'crowd', [*diagnostic_args, '--mode', 'crowd', '--zoom', '1', '--seconds', '60'], 90)
    if mode in ('all', 'gifs'):
        gif_dir = output / 'gifs'
        gif_dir.mkdir(exist_ok=True)
        answers = []
        for number, (combo, activity, ids) in enumerate(select_combos(layers), 1):
            for zoom in (1, 2):
                frames = output / f'combo_{number:02d}_{zoom}x'
                run_godot(manifest, frames, [*diagnostic_args, '--mode', 'capture', '--zoom', str(zoom),
                                            '--combo', str(combo), '--activity', activity])
                gif = gif_dir / f'{route}_{number:02d}_{zoom}x.gif'
                gif_from_frames(frames, gif)
                answers.append({'number': number, 'zoom': zoom, 'activity': activity, 'ids': ids,
                                'gif': str(gif), 'sha256': hashlib.sha256(gif.read_bytes()).hexdigest()})
        (output / 'answer_key.private.json').write_text(json.dumps(answers, indent=2) + '\n')


def main() -> int:
    """Minimal positional CLI; no new dependencies beyond the existing art pipeline."""
    arguments = sys.argv[1:]
    allow_incomplete = '--allow-incomplete' in arguments
    arguments = [argument for argument in arguments if argument != '--allow-incomplete']
    if len(arguments) not in (2, 3) or (len(arguments) == 3 and arguments[2] not in ('all', 'grid', 'gifs', 'crowd')):
        print(__doc__)
        print('usage: python3 -m art.pipeline3.capture_lab MANIFEST OUTPUT [all|grid|gifs|crowd] [--allow-incomplete]')
        return 2
    capture(Path(arguments[0]).resolve(), Path(arguments[1]).resolve(), arguments[2] if len(arguments) == 3 else 'all', allow_incomplete=allow_incomplete)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
