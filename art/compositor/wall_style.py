# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: python3 -m pytest art/compositor/test_polish_walls.py -q.
"""Declared HSV cap treatment; all material variation is palette hue only."""
from __future__ import annotations

import colorsys
from pathlib import Path
from typing import Final, Literal

from PIL import Image, ImageColor, ImageDraw, ImageFilter
from pydantic import Field

from art.compositor.models import InputModel

CapStyle = Literal["dark", "struct"]
PARAMS_PATH: Final = Path(__file__).with_name("wall_params.json")


class WallParameters(InputModel):
    cap_style: CapStyle
    box_blur_px: Literal[9]
    value_multiplier: float = Field(gt=0, le=1)
    saturation_multiplier: float = Field(ge=0, le=1)
    ink_hex: str
    ink_alpha: float = Field(ge=0, le=1)
    ink_width_px: Literal[2]
    south_highlight_width_px: Literal[1]
    south_highlight_value_add: float = Field(ge=0, le=1)
    shared_cap_source: str
    palette_source: str
    material_palette_indices: dict[str, int]
    dug_floor_blend_px: Literal[12]
    owner_value_choices: tuple[float, ...]

    @classmethod
    def read(cls) -> WallParameters:
        return cls.model_validate_json(PARAMS_PATH.read_bytes())


class Palette(InputModel):
    rgb: tuple[tuple[int, int, int], ...]


def material_hue(root: Path, material: str, params: WallParameters) -> float:
    palette = Palette.model_validate_json((root / params.palette_source).read_bytes())
    colour = palette.rgb[params.material_palette_indices[material]]
    return colorsys.rgb_to_hsv(*(value / 255 for value in colour))[0]


def dark_strip(source: Image.Image, params: WallParameters, hue: float | None) -> Image.Image:
    """Periodic horizontal 9px box; HSV scale before any boundary ink."""
    radius = (params.box_blur_px - 1) // 2
    padded = Image.new("RGBA", (source.width + 2 * radius, source.height))
    padded.paste(source.crop((source.width - radius, 0, source.width, source.height)))
    padded.paste(source, (radius, 0))
    padded.paste(source.crop((0, 0, radius, source.height)), (source.width + radius, 0))
    blurred = padded.filter(ImageFilter.BoxBlur(radius)).crop((radius, 0, source.width + radius, source.height))
    pixels = blurred.tobytes()
    output = Image.new("RGBA", source.size)
    for y in range(source.height):
        for x in range(source.width):
            offset = (y * source.width + x) * 4
            r, g, b, a = pixels[offset:offset + 4]
            h, s, v = colorsys.rgb_to_hsv(r / 255, g / 255, b / 255)
            rgb = colorsys.hsv_to_rgb(h if hue is None else hue,
                                      s * params.saturation_multiplier, v * params.value_multiplier)
            output.putpixel((x, y), (*[round(channel * 255) for channel in rgb], a))
    return output


def outline_cap(image: Image.Image, cap_height: int, neighbours: tuple[bool, bool, bool, bool],
                face_visible: bool, params: WallParameters) -> None:
    """One alpha union for exposed edges; south highlight derives from uninked cap."""
    north, east, south, west = neighbours
    original = image.crop((0, cap_height - 1, image.width, cap_height)).tobytes()
    ink = Image.new("RGBA", image.size)
    draw = ImageDraw.Draw(ink)
    rgb = ImageColor.getrgb(params.ink_hex)
    colour = (*rgb, round(params.ink_alpha * 255))
    width = params.ink_width_px
    for exposed, rectangle in ((not north, (0, 0, image.width - 1, width - 1)),
                               (not east, (image.width - width, 0, image.width - 1, cap_height - 1)),
                               (not south, (0, cap_height - width, image.width - 1, cap_height - 1)),
                               (not west, (0, 0, width - 1, cap_height - 1))):
        if exposed:
            draw.rectangle(rectangle, fill=colour)
    image.alpha_composite(ink)
    if face_visible:
        for x in range(width if not west else 0, image.width - (width if not east else 0)):
            r, g, b, a = original[x * 4:x * 4 + 4]
            h, s, v = colorsys.rgb_to_hsv(r / 255, g / 255, b / 255)
            rgb_float = colorsys.hsv_to_rgb(h, s, min(1, v + params.south_highlight_value_add))
            image.putpixel((x, cap_height - 1), (*[round(channel * 255) for channel in rgb_float], a))
