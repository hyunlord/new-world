# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
"""Readable natural-colour contact sheets with exact per-frame review indexes."""
from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
GRASS = (92, 111, 67)


@dataclass(frozen=True)
class BoardFrame:
    ident: str
    body: str
    activity: str
    view: str
    frame: int
    sprite: Image.Image
    mask: Image.Image
    peers: tuple[Image.Image, ...] = ()


def paste(board: Image.Image, sprite: Image.Image, x: int, y: int, scale: int) -> None:
    enlarged = sprite.resize((sprite.width * scale, sprite.height * scale), Image.Resampling.NEAREST)
    board.paste(enlarged, (x, y), enlarged)


def hair_visuals(sprite: Image.Image, mask: Image.Image) -> tuple[Image.Image, Image.Image]:
    rgba = np.array(sprite.convert('RGBA'))
    indices = np.asarray(mask.convert('RGBA'))
    hair = indices[..., 0] == 3
    overlay = rgba.copy()
    overlay[hair, :3] = (70, 140, 255)
    with Image.open(ROOT / 'assets/art/villager_ramps.png') as image:
        ramp = np.asarray(image.convert('RGB'))
    rgba[hair, :3] = ramp[4 + 6, np.clip(indices[..., 1][hair], 0, 2)]
    return Image.fromarray(overlay), Image.fromarray(rgba)


def compose(rows: list[BoardFrame], reference: Image.Image) -> Image.Image:
    max_width = max((r.sprite.width for r in rows), default=30)
    width = max(580, max_width * 18 + 90)
    reference_x = max(88, 20 + 2 * max_width)
    peers_x = max(166, reference_x + 2 * reference.width + 12)
    peer_width = max(64, max((p.width for r in rows for p in r.peers), default=28) * 2 + 8)
    peer_columns = max(1, (width - peers_x - 8) // peer_width)
    peer_height = max((p.height for r in rows for p in r.peers), default=30) * 2 + 6
    peer_rows = max(((len(r.peers) + peer_columns - 1) // peer_columns for r in rows), default=1)
    grass_height = max(84, 26 + peer_rows * peer_height)
    height = max(300, max((r.sprite.height for r in rows), default=32) * 6 + 66 + grass_height)
    board = Image.new('RGB', (width * 3, height * ((len(rows) + 2) // 3)), (161, 158, 152))
    draw = ImageDraw.Draw(board)
    for offset, row in enumerate(rows):
        x, y = (offset % 3) * width, (offset // 3) * height
        draw.text((x + 8, y + 5), row.ident, fill='black')
        draw.text((x + 8, y + 20), f'{row.activity} / {row.view} / frame {row.frame}', fill='black')
        draw.text((x + 8, y + 35), 'own 6x   | blue hair overlay 6x | white hair 4x', fill='black')
        paste(board, row.sprite, x + 8, y + 54, 6)
        if row.body != 'child':
            overlay, white = hair_visuals(row.sprite, row.mask)
            paste(board, overlay, x + 24 + max_width * 6, y + 54, 6)
            paste(board, white, x + 40 + max_width * 12, y + 54, 4)
        grass_y = y + height - grass_height
        draw.rectangle((x + 4, grass_y, x + width - 5, y + height - 3), fill=GRASS)
        paste(board, row.sprite, x + 12, grass_y + 8, 2)
        paste(board, reference, x + reference_x, grass_y + 8, 2)
        draw.text((x + peers_x, grass_y + 2), 'installed cycle 2x', fill='white')
        for index, peer in enumerate(row.peers):
            paste(board, peer, x + peers_x + (index % peer_columns) * peer_width, grass_y + 20 + (index // peer_columns) * peer_height, 2)
    return board


def save_board(board: Image.Image, path: Path) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    board.save(path, optimize=True)
    if path.stat().st_size > 1_000_000:
        destination = path.with_suffix('.jpg')
        board.save(destination, quality=92, subsampling=0)
        path.unlink()
        return destination
    return path


def atlas_frames() -> list[BoardFrame]:
    manifest = json.loads((ROOT / 'assets/art/atlas/world_atlas.json').read_text())
    registry = json.loads((ROOT / 'tools/art_pipeline/reviewed_hair_maps_p4.json').read_text())['frames']
    rows: list[BoardFrame] = []
    for ident, entry in manifest['sprites'].items():
        if entry.get('body') not in ('adult_a', 'adult_b', 'child') or not entry.get('drawable', True):
            continue
        page = entry['page']
        if isinstance(page, int):
            page = manifest['pages'][page]
        x, y, w, h = entry['rect']
        with Image.open(ROOT / page.removeprefix('res://')) as image:
            sprite = image.convert('RGBA').crop((x, y, x + w, y + h))
        with Image.open(ROOT / registry[ident]['mask_path']) as image:
            mask = image.convert('RGBA')
        rows.append(BoardFrame(ident, entry['body'], entry['activity'], entry['view'], entry['frame'], sprite, mask))
    return sorted(rows, key=lambda row: (row.body, row.activity, row.view, row.frame, row.ident))


def main() -> None:
    output = ROOT / 'docs/art/production4/natural_audit'
    frames = atlas_frames()
    index: dict[str, list[dict[str, object]]] = {}
    for body in ('adult_a', 'adult_b', 'child'):
        rows = [row for row in frames if row.body == body]
        with Image.open(ROOT / f'docs/art/production3/reprocess/references/{body}.png') as image:
            reference = image.convert('RGBA')
        master = save_board(compose(rows, reference), output / f'{body}.png')
        index[body] = []
        for start in range(0, len(rows), 12):
            chunk = rows[start:start + 12]
            page = save_board(compose(chunk, reference), output / f'pages/{body}_{start // 12 + 1:02}.png')
            index[body].append({'path': str(page.relative_to(ROOT)), 'sha256': hashlib.sha256(page.read_bytes()).hexdigest(), 'frames': [r.ident for r in chunk]})
        print(body, len(rows), master.relative_to(ROOT))
    (output / 'index.json').write_text(json.dumps(index, indent=2) + '\n')


if __name__ == '__main__':
    main()
