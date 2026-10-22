# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline5.build_part0
"""Compose the Part 0 owner board from unchanged candidate and historic pixels."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.pipeline4.pawn_gen import PawnLook, compose_layers

ROOT: Final = Path(__file__).resolve().parents[2]
NAMES: Final = ('I-1', 'I-2', 'I-3', 'I-4', 'B')
OUT: Final = ROOT / 'art/character/boards'
MEADOW: Final = ROOT / 'art/style05/ground_v2/meadow.png'
WIDTH: Final = 300


def read(path: Path) -> Image.Image:
    """Own image bytes independently of the file handle."""
    with Image.open(path) as source:
        return source.convert('RGBA')


def crop(image: Image.Image) -> Image.Image:
    """Trim only fully transparent margins, preserving every visible pixel."""
    bounds = image.getchannel('A').getbbox()
    if bounds is None:
        message = 'Empty foreground cannot appear on the review board'
        raise ValueError(message)
    return image.crop(bounds)


def scaled(image: Image.Image, height: int) -> Image.Image:
    """Scale the complete alpha crop with the specified Lanczos filter."""
    figure = crop(image)
    return figure.resize((round(figure.width * height / figure.height), height), Image.Resampling.LANCZOS)


def paste_at_feet(board: Image.Image, figure: Image.Image, x: int, foot_y: int) -> None:
    """Place an intact figure centred on its shared ground line."""
    board.alpha_composite(figure, (x - figure.width // 2, foot_y - figure.height))


def ground(size: tuple[int, int]) -> Image.Image:
    """Tile unchanged pixels from the frozen calm meadow at native scale."""
    tile = read(MEADOW)
    canvas = Image.new('RGBA', size)
    for y in range(0, size[1], tile.height):
        for x in range(0, size[0], tile.width):
            canvas.alpha_composite(tile, (x, y))
    return canvas


def historical_a() -> tuple[Image.Image, list[Path]]:
    """Compose the existing TRIAL3-A adult idle with its registered overlays."""
    paths = [ROOT / f'art/character/route_a/{name}.png' for name in
             ('body_S_idle', 'top_hide_wrap_S', 'hair_short_S')]
    image = read(paths[0])
    for path in paths[1:]:
        image.alpha_composite(read(path))
    return crop(image), paths


def main() -> None:
    """Emit three-row five-column board, equal-position crowds and provenance."""
    OUT.mkdir(parents=True, exist_ok=True)
    paths = [ROOT / f'art/character/candidates/{name}.png' for name in NAMES]
    figures = [read(path) for path in paths]
    pawn_root = ROOT / 'art/character/route_p'
    current = crop(compose_layers(pawn_root, PawnLook(top='hide_wrap')))
    old_a, a_paths = historical_a()
    board = Image.new('RGBA', (WIDTH * 5, 754), '#e4d2ad')
    draw = ImageDraw.Draw(board)
    board.alpha_composite(ground((WIDTH * 5, 294)), (0, 460))
    crowd = ground((WIDTH * 5, 380))
    crowd_draw = ImageDraw.Draw(crowd)
    for column, (name, image) in enumerate(zip(NAMES, figures, strict=True)):
        left = column * WIDTH
        draw.text((left + 12, 8), name + ' | whole figure | REVIEW ONLY', fill='#241b14')
        paste_at_feet(board, scaled(image, 384), left + 150, 430)
        draw.text((left + 12, 440), '384 px alpha crop', fill='#241b14')
        paste_at_feet(board, scaled(image, 112), left + 150, 600)
        draw.text((left + 12, 610), '2x | 112 px', fill='#241b14')
        paste_at_feet(board, scaled(image, 56), left + 56, 724)
        paste_at_feet(board, current, left + 153, 724)
        paste_at_feet(board, old_a, left + 247, 724)
        draw.text((left + 12, 738), name + ' 56px      CURRENT       TRIAL3-A', fill='#241b14')
        crowd_draw.text((left + 12, 8), name + ' | 12 figures | 56 px each', fill='#241b14')
        for row in range(3):
            for index in range(4):
                x = left + 40 + 73 * index + ((row % 2) * 5)
                y = 107 + row * 112 + (index % 2) * 14
                paste_at_feet(crowd, scaled(image, 56), x, y)
        draw.line((left, 0, left, 754), fill='#8c7759')
        crowd_draw.line((left, 0, left, 380), fill='#8c7759')
    outputs = [('look_candidates.png', board), ('look_candidates_1x_crowd.png', crowd)]
    for name, image in outputs:
        target = OUT / name
        image.convert('RGB').save(target, optimize=True)
        if target.stat().st_size > 1_500_000:
            message = f'Board exceeds committed file cap: {target}'
            raise ValueError(message)
    pawn_paths = [pawn_root / file for file in (
        'BODY/body_adult_M_0_S_idle.png', 'TOP/top_hide_wrap_0_S.png',
        'HAIR/hair_short_0_S.png', 'ITEM/item_none_S_idle.png')]
    sources = paths + [MEADOW] + a_paths + pawn_paths
    provenance = {
        'method': 'Alpha>0 bounding box crop, Lanczos scale, alpha composite; no quantization, recolour or pixel surgery.',
        'candidate_heights': [384, 112, 56], 'crowd_count_per_candidate': 12,
        'crowd_positions': 'Identical 4 by 3 staggered positions in every candidate column.',
        'historical_comparators': 'Native pixel sizes; composed adult male S idle, hide wrap, short hair; crop transparent margins only.',
        'comparator_alpha_heights': {'CURRENT': current.height, 'TRIAL3-A': old_a.height},
        'sources_sha256': {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in sources},
        'outputs': {name: {'dimensions': image.size, 'sha256': hashlib.sha256((OUT / name).read_bytes()).hexdigest(),
                           'bytes': (OUT / name).stat().st_size} for name, image in outputs},
    }
    _ = (OUT / 'look_candidates.provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
    print(json.dumps(provenance['outputs'], indent=2))


if __name__ == '__main__':
    main()
