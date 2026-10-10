#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow"]
# ///
# How to run: python3 tools/art/gen_terrain_atlas.py
# Or: uv run tools/art/gen_terrain_atlas.py
"""Generate the fixed-seed WorldSim terrain atlas and supporting pixel sprites."""

from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from random import Random
from typing import Final, assert_never

from PIL import Image, ImageDraw


class Terrain(Enum):
    """Atlas row order, deliberately distinct from simulation terrain codes."""

    PLAIN = "Plain"
    FOREST = "Forest"
    HILL = "Hill"
    MOUNTAIN = "Mountain"
    RIVER = "River"


@dataclass(frozen=True, slots=True)
class Palette:
    """The four authored colors used for each terrain family."""

    base: str
    light: str
    dark: str
    accent: str


PALETTES: Final[tuple[Palette, ...]] = (
    Palette("#6E8B4E", "#84A35C", "#58723F", "#9AAE6A"),
    Palette("#3E5E36", "#4E7343", "#2E4628", "#5C8250"),
    Palette("#8C8A5A", "#A3A06B", "#6F6D46", "#5E5C3B"),
    Palette("#7A7672", "#918C87", "#5C5955", "#44413E"),
    Palette("#3F6E8C", "#5687A6", "#2F566F", "#8DB7CF"),
)
TILE_SIZE: Final = 16
VARIANTS: Final = 4
SEED: Final = 7
BERRY_POINTS: Final = ((5, 7), (10, 8), (8, 5), (4, 10), (11, 11), (8, 11))


def terrain_tile(terrain: Terrain, palette: Palette, rng: Random) -> Image.Image:
    """Paint one tile with a solid border and seeded interior details."""
    tile = Image.new("RGB", (TILE_SIZE, TILE_SIZE), palette.base)
    draw = ImageDraw.Draw(tile)
    for color, count in ((palette.light, 23), (palette.dark, 18)):
        for _ in range(count):
            x, y = rng.randint(1, 13), rng.randint(1, 14)
            draw.line((x, y, x + 1, y), fill=color)
    for _ in range(8):
        draw.point((rng.randint(1, 14), rng.randint(1, 14)), fill=palette.accent)
    match terrain:
        case Terrain.PLAIN:
            pass
        case Terrain.FOREST:
            for _ in range(3):
                radius = rng.randint(3, 5)
                x = rng.randint(radius + 1, 14 - radius)
                y = rng.randint(radius + 1, 14 - radius)
                draw.ellipse((x - radius, y - radius, x + radius, y + radius), fill=palette.dark)
                draw.ellipse((x - radius, y - radius, x + radius - 1, y + radius - 1), fill=palette.base)
                draw.arc((x - radius, y - radius, x + radius - 1, y + radius - 1), 190, 290, fill=palette.light)
        case Terrain.HILL:
            draw.arc((2, 3, 13, 12), 185, 345, fill=palette.accent)
        case Terrain.MOUNTAIN:
            for _ in range(rng.randint(2, 3)):
                x, y = rng.randint(3, 10), rng.randint(2, 9)
                draw.line([(x, y), (x + 2, y + 2), (x + 1, y + 4)], fill=palette.accent)
        case Terrain.RIVER:
            for y in rng.sample(range(1, 15), 2):
                x = rng.randint(1, 10)
                draw.line((x, y, x + rng.randint(2, 4), y), fill=palette.accent)
        case unreachable:
            assert_never(unreachable)
    return tile


def mean_saturation(image: Image.Image) -> float:
    """Measure mean HSV saturation from every RGB pixel, without quantization."""
    channels = image.tobytes()
    total = 0.0
    for red, green, blue in zip(channels[::3], channels[1::3], channels[2::3], strict=True):
        maximum = max(red, green, blue)
        if maximum:
            total += (maximum - min(red, green, blue)) / maximum
    return total / (image.width * image.height)


def generate(root: Path) -> None:
    """Write only the seven deterministic art files below the supplied root."""
    sprites = root / "assets" / "sprites"
    for folder in ("terrain", "resources", "fx"):
        (sprites / folder).mkdir(parents=True, exist_ok=True)
    rng = Random(SEED)
    atlas = Image.new("RGB", (TILE_SIZE * VARIANTS, TILE_SIZE * len(Terrain)))
    for row, (terrain, palette) in enumerate(zip(Terrain, PALETTES, strict=True)):
        for variant in range(VARIANTS):
            atlas.paste(terrain_tile(terrain, palette, rng), (variant * TILE_SIZE, row * TILE_SIZE))
        strip = atlas.crop((0, row * TILE_SIZE, TILE_SIZE * VARIANTS, (row + 1) * TILE_SIZE))
        saturation = mean_saturation(strip)
        print(f"{terrain.value}: mean saturation {saturation:.6f}")
    atlas.save(sprites / "terrain" / "terrain_v8.png")

    for stage in range(4):
        bush = Image.new("RGBA", (16, 16))
        draw = ImageDraw.Draw(bush)
        draw.ellipse((2, 4, 13, 13), fill="#24361F")
        draw.ellipse((3, 5, 12, 12), fill="#3E5E36")
        draw.arc((3, 5, 12, 12), 190, 280, fill="#5C8250")
        for position in BERRY_POINTS[:2 * stage]:
            draw.point(position, fill="#B0443A")
        bush.save(sprites / "resources" / f"berry_bush_{stage}.png")

    shadow = Image.new("RGBA", (12, 6))
    ImageDraw.Draw(shadow).ellipse((0, 0, 11, 5), fill=(0, 0, 0, 90))
    shadow.save(sprites / "fx" / "shadow_ellipse.png")
    glow = Image.new("RGBA", (64, 64))
    pixels: list[tuple[int, int, int, int]] = []
    for y in range(64):
        for x in range(64):
            radius_squared = ((x - 31.5) ** 2 + (y - 31.5) ** 2) / (31.5 ** 2)
            falloff = max(0.0, 1.0 - radius_squared)
            pixels.append((255, 255, 255, round(255.0 * falloff * falloff)))
    glow.putdata(pixels)
    glow.save(sprites / "fx" / "glow_radial.png")


if __name__ == "__main__":
    generate(Path(__file__).resolve().parents[2])
