# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline3.b_lab_composites MANIFEST CHECKS A_LAB OUTPUT
"""Compose actual B layers for diagnostic review without weakening lab admission."""
from __future__ import annotations

import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from itertools import product
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from .capture_lab import ACTIVITIES, gif_from_frames
from .layer_schema import Layer, load_layer

ROOT: Final = Path(__file__).resolve().parents[2]


@dataclass(frozen=True, slots=True)
class Combo:
    """One direction and wardrobe selection, including optional independent props."""

    direction: str
    top: str
    hair: str
    axe: bool
    carry: bool


def selected(a_lab: Path) -> list[Combo]:
    """Match A's existing twelve samples' attributes for direct comparison."""
    answers = json.loads((a_lab / 'answer_key.private.json').read_text())
    result: list[Combo] = []
    for number in range(1, 13):
        row = next(row for row in answers if row['number'] == number and row['zoom'] == 1)
        if row['activity'] != ACTIVITIES[number - 1]:
            message = f'A activity schedule differs at sample {number}'
            raise ValueError(message)
        ids = row['ids']
        body = next(item for item in ids if item.startswith('body_'))
        direction = next((part for part in body.split('_') if part in ('S', 'E', 'N')), 'S')
        top = 'fur_cloak' if any('fur_cloak' in item for item in ids) else 'hide_wrap'
        hair = 'long_tied' if any('long_tied' in item for item in ids) else 'short'
        result.append(Combo(direction, top, hair, any('stone_axe' in item for item in ids),
                            any('log_bundle' in item for item in ids)))
    return result


def parts(layers: list[Layer], combo: Combo, frame: str) -> list[Layer]:
    """Select already-positioned rendered passes; apply no anchor translation."""
    if frame == 'sleep':
        return [next(layer for layer in layers if layer.kind == 'body' and layer.frame == 'sleep')]
    chosen = [next(layer for layer in layers if layer.kind == 'body'
                   and layer.direction == combo.direction and layer.frame == frame)]
    for kind, variant in [('top', combo.top), ('hair', combo.hair),
                          *([('item', 'stone_axe')] if combo.axe else []),
                          *([('carry', 'log_bundle')] if combo.carry else [])]:
        chosen.append(next(layer for layer in layers if layer.kind == kind
                           and layer.variant == variant and layer.direction == combo.direction))
    return chosen


def compose(chosen: list[Layer], ground: str, bob: int = 0) -> Image.Image:
    """Alpha-composite original RGBA passes and the specified lab contact ellipse."""
    with Image.open(ROOT / f'art/style05/env/ground/{ground}.png') as source:
        stage = source.convert('RGBA').crop((0, 0, 104, 112))
    body = chosen[0]
    size = (96, 64) if body.frame == 'sleep' else (64, 96)
    figure = Image.new('RGBA', size)
    if body.frame != 'sleep':
        ImageDraw.Draw(figure).ellipse((18, 84, 46, 92), fill=(0, 0, 0, 110))
    for layer in chosen:
        with Image.open(layer.image) as source:
            if layer.scale != 1 or source.size != size:
                message = f'Requires original 1x pass on declared canvas: {layer.id}'
                raise ValueError(message)
            figure.alpha_composite(source.convert('RGBA'))
    stage.alpha_composite(figure, ((104 - size[0]) // 2, 112 - size[1] - 8 - bob))
    return stage.convert('RGB')


def make_gifs(layers: list[Layer], a_lab: Path, output: Path) -> None:
    """Retain eight unquantized PNG frames before GIF-only palette encoding."""
    (output / 'gifs').mkdir(exist_ok=True)
    answers = []
    for number, (combo, activity) in enumerate(zip(selected(a_lab), ACTIVITIES, strict=True), 1):
        frames: tuple[str, ...] = ('idle',) * 8
        if activity in ('walk', 'carry'):
            frames = ('walk_a', 'walk_b') * 4
        if activity == 'work':
            frames = ('work_a', 'work_b') * 4
        if activity == 'sleep':
            frames = ('sleep',) * 8
        for zoom in (1, 2):
            directory = output / f'combo_{number:02d}_{zoom}x'
            directory.mkdir(exist_ok=True)
            for index, frame in enumerate(frames):
                image = compose(parts(layers, combo, frame), 'meadow', index % 2 if activity == 'idle' else 0)
                if zoom == 2:
                    image = image.resize((208, 224), Image.Resampling.NEAREST)
                image.save(directory / f'frame_{index:02d}.png')
            gif = output / 'gifs' / f'b_{number:02d}_{zoom}x.gif'
            gif_from_frames(directory, gif)
            answers.append({'number': number, 'zoom': zoom, 'activity': activity,
                            'top': 'none' if activity == 'sleep' else combo.top,
                            'hair': 'none' if activity == 'sleep' else combo.hair,
                            'carried': 'axe' if combo.axe else 'logs' if combo.carry else 'none',
                            'direction': combo.direction, 'frames': frames,
                            'layer_ids': [layer.id for layer in parts(layers, combo, frames[0])],
                            'sha256': hashlib.sha256(gif.read_bytes()).hexdigest()})
    (output / 'answer_key.private.json').write_text(json.dumps(answers, indent=2) + '\n')


def boards(layers: list[Layer], output: Path, status: str) -> None:
    """Cover all 48 upright combinations on both grounds in bounded-size pages."""
    combos = [Combo(*values) for values in product(('S', 'E', 'N'),
              ('hide_wrap', 'fur_cloak'), ('short', 'long_tied'), (False, True), (False, True))]
    directory = output / 'boards'
    directory.mkdir(exist_ok=True)
    pages = []
    for zoom, ground in product((1, 2), ('meadow', 'forest_floor')):
        cell_w, cell_h = 104 * zoom, 112 * zoom + 20
        columns, rows = 1040 // cell_w, 820 // cell_h
        capacity = columns * rows
        for first in range(0, len(combos), capacity):
            subset = combos[first:first + capacity]
            board = Image.new('RGB', (columns * cell_w, ((len(subset) + columns - 1) // columns) * cell_h + 40), '#e0bd8e')
            draw = ImageDraw.Draw(board)
            draw.text((8, 8), f'{status} | B {zoom}x {ground} | combinations {first + 1}-{first + len(subset)}/48', fill='#18110a')
            for index, combo in enumerate(subset):
                image = compose(parts(layers, combo, 'idle'), ground)
                if zoom == 2:
                    image = image.resize((208, 224), Image.Resampling.NEAREST)
                x, y = index % columns * cell_w, index // columns * cell_h + 40
                board.paste(image, (x, y))
                draw.text((x + 4, y + 112 * zoom), f'{first + index + 1:02d}', fill='#18110a')
            filename = f'lab_b_{zoom}x_{ground}_page{first // capacity + 1:03d}.png'
            board.save(directory / filename)
            if first == 0 and ground == 'meadow':
                board.save(directory / f'lab_b_{zoom}x.png')
            pages.append({'file': filename, 'ground': ground, 'zoom': zoom,
                          'indices': list(range(first, first + len(subset))), 'status': status})
    (directory / 'pages.json').write_text(json.dumps(pages, indent=2) + '\n')
    (directory / 'combinations.json').write_text(json.dumps([asdict(combo) for combo in combos], indent=2) + '\n')


def compare(a_lab: Path, output: Path, status: str) -> None:
    """Place original A stills beside matched B composites without rescaling either."""
    for zoom in (1, 2):
        cell_w, cell_h = 208 * zoom, 112 * zoom + 20
        columns, rows = 1040 // cell_w, 820 // cell_h
        capacity = columns * rows
        for first in range(0, 12, capacity):
            count = min(capacity, 12 - first)
            board = Image.new('RGB', (columns * cell_w, ((count + columns - 1) // columns) * cell_h + 40), '#e0bd8e')
            draw = ImageDraw.Draw(board)
            draw.text((8, 8), f'{status} | existing A (left) / B (right) | {zoom}x', fill='#18110a')
            for index in range(count):
                number = first + index + 1
                x, y = index % columns * cell_w, index // columns * cell_h + 40
                for offset, root in enumerate((a_lab, output)):
                    with Image.open(root / f'combo_{number:02d}_{zoom}x/frame_00.png') as source:
                        if source.size != (104 * zoom, 112 * zoom):
                            message = f'Unexpected A/B still dimensions: {source.size}'
                            raise ValueError(message)
                        board.paste(source, (x + offset * 104 * zoom, y))
                draw.text((x + 4, y + 112 * zoom), f'{number:02d}: A | B', fill='#18110a')
            board.save(output / 'boards' / f'compare_AB_{zoom}x_page{first // capacity + 1:03d}.png')
            if first == 0:
                board.save(output / 'boards' / f'compare_AB_{zoom}x.png')


def main() -> None:
    """Require actual checker evidence; diagnostic composites never imply acceptance."""
    if len(sys.argv) != 5:
        raise SystemExit('Usage: MANIFEST CHECKS A_LAB OUTPUT')
    manifest, checks_path, a_lab, output = map(Path, sys.argv[1:])
    checks = json.loads(checks_path.read_text())
    raw = json.loads(manifest.read_text())
    layers = [load_layer(manifest.parent / name) for name in raw['layers']]
    expected = {str(path.resolve()): hashlib.sha256(path.read_bytes()).hexdigest()
                for path in [manifest, *[manifest.parent / name for name in raw['layers']],
                             *[layer.image for layer in layers], *[layer.mask for layer in layers],
                             *[layer.anatomical_mask for layer in layers if layer.anatomical_mask is not None]]}
    if checks['route'] != 'B' or any(checks['input_hashes'].get(path) != digest for path, digest in expected.items()):
        raise SystemExit('Checker evidence does not match these B inputs')
    status = 'STRUCTURAL_PASS_REVIEW_PENDING' if checks['passed'] else 'DIAGNOSTIC_REJECTED'
    output.mkdir(parents=True, exist_ok=True)
    (output / 'DATASET_STATUS.txt').write_text(status + '\nNo structural acceptance is inferred from composite/GIF generation.\n')
    make_gifs(layers, a_lab, output)
    boards(layers, output, status)
    compare(a_lab, output, status)
    (output / 'provenance.json').write_text(json.dumps({'status': status, 'inputs': expected,
        'checks_sha256': hashlib.sha256(checks_path.read_bytes()).hexdigest(),
        'method': 'original 1x passes, alpha composition, contact shadow; no pixel repair; nearest 2x',
        'scope': '48 upright independent-prop combinations plus bare sleep; idle 1px composition bob matching lab; sleep static source pose',
        'runtime': 'Pillow composites; browser playback separate; NOT Godot P1'}, indent=2) + '\n')
    print(status)


if __name__ == '__main__':
    main()
