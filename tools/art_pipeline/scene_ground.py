"""Deterministic sixteen-texel ground and continuous two-tile river."""
from __future__ import annotations

import colorsys
from dataclasses import dataclass

import numpy as np
from PIL import Image, ImageDraw

from palette_tools import Color, rgb_to_lab


@dataclass(frozen=True, slots=True)
class GroundRoles:
    grass: tuple[Color, ...]
    earth: tuple[Color, ...]
    water: tuple[Color, ...]
    shore: Color


def ground_roles(palette: list[Color]) -> GroundRoles:
    """Select actual source colors, never introduce new environment colors."""
    def nearest(target: Color, candidates: list[Color]) -> Color:
        labs = rgb_to_lab(np.asarray(candidates))
        index = int(np.argmin(np.linalg.norm(labs - rgb_to_lab(np.asarray(target)), axis=1)))
        return candidates[index]
    def ramp(targets: tuple[Color, ...], candidates: list[Color]) -> tuple[Color, ...]:
        available = list(dict.fromkeys(candidates or palette))
        selected: list[Color] = []
        for target in targets:
            unused = [color for color in available if color not in selected]
            # A short source ramp repeats its non-base entries, never the base.
            pool = unused or [color for color in available if color != selected[0]] or available
            selected.append(nearest(target, pool))
        return tuple(selected)
    greens = [c for c in palette if 0.14 < colorsys.rgb_to_hsv(*c)[0] < 0.48]
    blues = [c for c in palette if 0.46 < colorsys.rgb_to_hsv(*c)[0] < 0.70]
    earth = [c for c in palette if 0.035 < colorsys.rgb_to_hsv(*c)[0] < 0.15]
    return GroundRoles(
        ramp(((102,115,61),(106,120,64),(100,112,59),(109,121,67),(69,89,48)), greens),
        ramp(((135,111,73),(142,120,83),(130,108,74),(147,124,86)), earth),
        ramp(((45,92,111),(51,102,122),(66,113,131),(88,130,141)), blues),
        nearest((173,158,116), earth or palette),
    )


def tile_variant(x: int, y: int) -> int:
    return ((x * 73856093) ^ (y * 19349663)) % 4


def river_left(y: int, direction: str) -> int:
    """Quantized bends remain exactly thirty-two texels wide at every row."""
    bend = (0, 1, 1, 0, -1, -1, 0, 1)[(y // 96) % 8]
    return 368 + bend * 8 + (8 if direction == "D2" else 0)


def ground_image(roles: GroundRoles, direction: str) -> Image.Image:
    result = Image.new("RGB", (1024, 1024))
    draw = ImageDraw.Draw(result)
    for ty in range(64):
        for tx in range(64):
            variant = tile_variant(tx, ty)
            x, y = tx * 16, ty * 16
            packed = ((x + 8 - 512) / 88) ** 2 + ((y + 8 - 512) / 68) ** 2 < 1
            ramp = roles.earth if packed else roles.grass
            draw.rectangle((x, y, x+15, y+15), fill=ramp[0])
            if packed:
                draw.line((x+4+variant, y+11, x+6+variant, y+11), fill=ramp[1+variant%3])
            elif variant != 3:
                px, py = x+3+variant*3, y+7+variant
                draw.line((px,py,px,py-2), fill=roles.grass[4])
                draw.point((px-1,py-1), fill=roles.grass[1+variant%3])
    for y in range(1024):
        left = river_left(y, direction)
        draw.line((left-1,y,left-1,y), fill=roles.shore)
        draw.line((left,y,left+31,y), fill=roles.water[0])
        draw.point((left+32,y), fill=roles.shore)
        if y % 16 in (5, 11):
            for offset in (5, 21):
                draw.line((left+offset,y,left+offset+2,y), fill=roles.water[1+(y//16)%3])
    return result.convert("RGBA")
