# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: invoked by merge_p5_selection.py prepare
"""Freeze frame-order animation boards without disclosing technical acceptance."""
from __future__ import annotations

import hashlib
from pathlib import Path
from PIL import Image, ImageDraw
from harvest_p5 import Completion


def completion_id(completion: Completion) -> str:
    """Stable identity includes target pixels and the exact copied-frame assignment."""
    text = '|'.join(f'{r["id"]}:{r.get("source_candidate", "")}' for r in completion.rows)
    suffix = hashlib.sha256(text.encode()).hexdigest()[:10]
    return '_'.join(completion.group) + '_' + suffix


def boards(completion: Completion, reference: Image.Image, directory: Path) -> dict[str, str]:
    """Write grey 4x, grass 2x, and a 2x 8/4-fps loop beside the reference."""
    directory.mkdir(parents=True, exist_ok=True)
    images = []
    for row in completion.rows:
        with Image.open(row['images']['sprite']) as opened:
            images.append(opened.convert('RGBA'))
    width = max([image.width for image in images] + [reference.width]) + 8
    height = max([image.height for image in images] + [reference.height]) + 8
    board = Image.new('RGB', ((len(images) + 1) * width * 4, height * 6 + 70), (110, 110, 110))
    draw = ImageDraw.Draw(board)
    draw.text((6, 4), '/'.join(completion.group) + ' | reference idle at right', fill='white')
    for scale, y, background in ((4, 30, (110, 110, 110)), (2, height * 4 + 48, (91, 120, 61))):
        for index, image in enumerate(images + [reference]):
            tile = Image.new('RGB', (width * scale, height * scale), background)
            enlarged = image.resize((image.width * scale, image.height * scale), Image.Resampling.NEAREST)
            tile.paste(enlarged, ((width * scale - enlarged.width) // 2, height * scale - enlarged.height), enlarged)
            board.paste(tile, (index * width * 4, y))
            label = 'reference' if index == len(images) else f'frame {completion.rows[index]["frame"]}' + (' COPY' if completion.rows[index]['source_kind'] == 'derived' else '')
            draw.text((index * width * 4 + 4, y - 13), label, fill='white')
    identity = completion_id(completion)
    path = directory / f'{identity}.png'
    board.save(path, optimize=True)
    gif_frames = []
    for image in images:
        frame = Image.new('RGB', (width * 4, height * 2 + 20), (91, 120, 61))
        for index, sprite in enumerate((image, reference)):
            enlarged = sprite.resize((sprite.width * 2, sprite.height * 2), Image.Resampling.NEAREST)
            frame.paste(enlarged, (index * width * 2 + (width * 2-enlarged.width)//2, height * 2-enlarged.height), enlarged)
        ImageDraw.Draw(frame).text((2, height * 2 + 3), 'animation       reference', fill='white')
        gif_frames.append(frame)
    gif = directory / f'{identity}.gif'
    duration = 125 if completion.group[2] in ('walk', 'run', 'carry', 'carry_wood') else 250
    gif_frames[0].save(gif, save_all=True, append_images=gif_frames[1:], duration=duration, loop=0, disposal=2)
    return {'board': str(path), 'gif': str(gif)}
