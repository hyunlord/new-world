# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: imported by remask_candidates_p5.py.
"""Blind review cards with natural, outline-aware mask, black and light hair views."""
from __future__ import annotations

import numpy as np
from PIL import Image, ImageDraw
from natural_boards_p4 import paste, GRASS

BLACK = np.array(((63,60,54),(41,38,33),(16,11,3)), dtype=np.uint8)
WHITE = np.array(((242,241,240),(217,216,217),(187,185,183)), dtype=np.uint8)
FLAXEN = np.array(((202,196,141),(174,169,115),(142,137,85)), dtype=np.uint8)


def recolour(sprite: Image.Image, mask: Image.Image, colours: np.ndarray) -> Image.Image:
    """Emulate the region-ramp lookup without changing unmasked pixels."""
    pixels = np.array(sprite)
    indices = np.asarray(mask)
    hair = indices[..., 0] == 3
    pixels[hair, :3] = colours[indices[..., 1][hair]]
    return Image.fromarray(pixels)


def card(ident: str, activity: str, view: str, sprite: Image.Image, mask: Image.Image,
         reference: Image.Image) -> Image.Image:
    """Render all GOAL 7.4 scales, including a grass reference comparison."""
    width = max(620, sprite.width * 20 + 60)
    height = max(290, sprite.height * 6 + 100)
    board = Image.new('RGB', (width, height), (161,158,152))
    draw = ImageDraw.Draw(board)
    draw.text((8, 6), ident, fill='black')
    draw.text((8, 21), f'{activity} / {view} | natural 6x | mask 6x | light 4x | black 4x', fill='black')
    overlay = np.array(sprite)
    overlay[np.asarray(mask)[..., 0] == 3, :3] = (70,140,255)
    light = FLAXEN if activity in ('carry_wood', 'gather_wood') else WHITE
    paste(board, sprite, 8, 40, 6)
    paste(board, Image.fromarray(overlay), 18+sprite.width*6, 40, 6)
    paste(board, recolour(sprite, mask, light), 28+sprite.width*12, 40, 4)
    paste(board, recolour(sprite, mask, BLACK), 38+sprite.width*16, 40, 4)
    draw.rectangle((0,height-76,width,height), fill=GRASS)
    paste(board, sprite, 12, height-72, 2)
    paste(board, reference, 35+sprite.width*2, height-72, 2)
    draw.text((70+sprite.width*2+reference.width*2,height-65),'2x on grass; adjacent reference idle',fill='white')
    return board
