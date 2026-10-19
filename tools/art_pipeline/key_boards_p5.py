# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: invoked by key_candidates_p5.py
"""Candidate review boards pair natural identity with precise key-mask previews."""
from __future__ import annotations

import hashlib
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from natural_boards_p4 import GRASS, paste, save_board
from salvage_v4 import CandidateRecord


def review_boards(records: list[CandidateRecord], folder: Path, reference: Image.Image, ramps: Image.Image) -> list[dict[str, str | list[str]]]:
    """Freeze raw, 6x own/overlay, 2x grass identity and 4x white/flaxen + black."""
    pages: list[dict[str, str | list[str]]] = []
    ramp = np.asarray(ramps.convert('RGB'))
    for start in range(0, len(records), 4):
        chunk = records[start:start + 4]
        board = Image.new('RGB', (1800, len(chunk) * 300), (161, 158, 152))
        draw = ImageDraw.Draw(board)
        for offset, row in enumerate(chunk):
            y = offset * 300
            draw.text((8, y + 5), f'{row["id"]} | {row["activity"]} {row["view"]} frame {row["frame"]}', fill='black')
            draw.text((8, y + 20), 'RAW: teal is a mask key by design and replaced; identity judged against NATURAL idle only', fill='black')
            with Image.open(row['images']['raw']) as raw:
                thumb = raw.convert('RGB'); thumb.thumbnail((240, 235), Image.Resampling.NEAREST)
                board.paste(thumb, (8, y + 50))
            if 'sprite_hair_mask' not in row['images']:
                draw.text((270, y + 50), 'Conversion unavailable; reject retained', fill='black')
                continue
            with Image.open(row['images']['sprite']) as opened, Image.open(row['images']['sprite_hair_mask']) as source:
                sprite, mask = opened.convert('RGBA'), np.asarray(source.convert('RGBA'))
            rgba = np.asarray(sprite)
            hair = mask[..., 0] == 3
            overlay = rgba.copy(); overlay[hair, :3] = (70, 140, 255)
            preview = rgba.copy(); black = rgba.copy()
            hair_index = 4 if row['activity'] in ('carry_wood', 'gather_wood') else 6
            # Runtime hair rows start at four after the skin ramps.
            preview[hair, :3] = ramp[hair_index + 4, mask[..., 1][hair]]
            black[hair, :3] = ramp[4, mask[..., 1][hair]]
            for x, label, image, scale in [(270, 'own 6x', sprite, 6), (490, 'mask 6x', Image.fromarray(overlay), 6),
                                          (710, f'hair {hair_index} 4x', Image.fromarray(preview), 4),
                                          (870, 'black 4x', Image.fromarray(black), 4)]:
                draw.text((x, y + 38), label, fill='black'); paste(board, image, x, y + 60, scale)
            draw.rectangle((1050, y + 40, 1790, y + 295), fill=GRASS)
            draw.text((1070, y + 45), 'candidate 2x | natural identity idle 2x', fill='white')
            paste(board, sprite, 1070, y + 80, 2); paste(board, reference, 1220, y + 80, 2)
        path = save_board(board, folder / f'page_{start // 4 + 1:02}.png')
        pages.append({'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                      'candidate_ids': [r['id'] for r in chunk]})
    return pages
