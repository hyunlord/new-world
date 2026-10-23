# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by the STRUCT compositor; tests in test_edges.py.
"""World-aligned 64px terrain, noisy banks, stockpile tint and ford placement."""
from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from typing import Final, Literal, assert_never

import numpy as np
from numpy.typing import NDArray
from PIL import Image, ImageDraw

Cell = tuple[int, int]
TILE: Final = 64
SEED: Final = 5005
FloatGrid = NDArray[np.float64]
DIRECTIONS: Final = ((0, -1), (1, 0), (0, 1), (-1, 0))


@dataclass(frozen=True, slots=True)
class EdgeField:
    """World tile identifiers and original textures; render at 1x before LOD."""

    terrain: Mapping[Cell, str]
    textures: Mapping[str, Image.Image]
    walls: frozenset[Cell] = frozenset()
    water: frozenset[Cell] = frozenset()
    seed: int = SEED
    hills: frozenset[Cell] = frozenset()
    ground_blend_px: int = 12


@dataclass(frozen=True, slots=True)
class Boundary:
    """Pixels straddling x or y at start, followed along the boundary."""

    start: Cell
    length: int
    axis: Literal["vertical", "horizontal"]


@dataclass(frozen=True, slots=True)
class BoundaryMeasurement:
    """Signed RGB-derived luminance steps, rounded to 1/1000 display level."""

    steps: tuple[float, ...]
    longest_identical_step: int
    nonzero_steps: int


def sample_texture(texture: Image.Image, origin_px: Cell,
                   size: Cell = (TILE, TILE)) -> Image.Image:
    """Sample a repeating source using absolute world pixels, including negatives."""
    pixels = np.asarray(texture.convert("RGBA"))
    x = (np.arange(size[0]) + origin_px[0]) % texture.width
    y = (np.arange(size[1]) + origin_px[1]) % texture.height
    return Image.fromarray(pixels[y[:, None], x[None, :]])


def _noise(cell: Cell, seed: int) -> FloatGrid:
    """Bilinear fixed-seed value noise at 4px spacing plus bounded pixel dither."""
    y, x = np.indices((TILE, TILE), dtype=np.int64)
    x, y = x + cell[0] * TILE, y + cell[1] * TILE

    def hash_value(hx: NDArray[np.int64], hy: NDArray[np.int64]) -> FloatGrid:
        values = (hx * 73856093) ^ (hy * 19349663) ^ seed
        values = ((values ^ (values >> 13)) * 1274126177) & 0xFFFFFFFF
        return (values & 65535).astype(np.float64) / 65535

    gx, gy = x // 4, y // 4
    tx, ty = (x % 4) / 4, (y % 4) / 4
    smooth = ((1 - ty) * ((1 - tx) * hash_value(gx, gy) + tx * hash_value(gx + 1, gy))
              + ty * ((1 - tx) * hash_value(gx, gy + 1) + tx * hash_value(gx + 1, gy + 1)))
    return .8 * smooth + .2 * hash_value(x, y)


def _weights(cell: Cell, width: int, seed: int) -> tuple[FloatGrid, ...]:
    y, x = np.indices((TILE, TILE), dtype=np.float64)
    noise = _noise(cell, seed) - .5
    distances = (y, TILE - 1 - x, TILE - 1 - y, x)
    return tuple(np.clip((width - distance - .5) / width + noise * .6, 0, 1)
                 * (distance < width) for distance in distances)


def water_mask(field: EdgeField, cell: Cell) -> Image.Image:
    """10px inward alpha falloff; corners combine by maximum edge erosion."""
    erosion = np.zeros((TILE, TILE), dtype=np.float64)
    for offset, weight in zip(DIRECTIONS, _weights(cell, 10, field.seed), strict=True):
        neighbour = (cell[0] + offset[0], cell[1] + offset[1])
        if neighbour in field.terrain and neighbour not in field.water:
            erosion = np.maximum(erosion, weight)
    return Image.fromarray(np.rint((1 - erosion) * 255).astype(np.uint8))


def _bank_ink(field: EdgeField, cell: Cell) -> FloatGrid:
    """Two-pixel ink contour displaced 0–4px onto land; corners use max ink.

    The world-seeded value noise moves the stroke itself, not just its shade.
    This avoids a straight black tile border passing a luminance-only metric.
    """
    y, x = np.indices((TILE, TILE))
    noise = _noise(cell, field.seed)
    distances = (y, TILE - 1 - x, TILE - 1 - y, x)
    edge_noise = (noise[0:1, :], noise[:, -1:], noise[-1:, :], noise[:, 0:1])
    strength = np.zeros((TILE, TILE), dtype=np.float64)
    for offset, distance, along in zip(DIRECTIONS, distances, edge_noise, strict=True):
        neighbour = (cell[0] + offset[0], cell[1] + offset[1])
        if neighbour not in field.water or neighbour in field.walls:
            continue
        displacement = np.rint(along * 4)
        stroke = (distance >= displacement) & (distance < displacement + 2)
        strength = np.maximum(strength, stroke * (.20 + .15 * noise))
    return strength


def render_tile(field: EdgeField, cell: Cell) -> Image.Image:
    """Blend the strongest 12px edge, or water over its strongest bank ground."""
    origin = (cell[0] * TILE, cell[1] * TILE)
    source = sample_texture(field.textures[field.terrain[cell]], origin)
    if cell in field.walls or cell in field.hills:
        return source
    pixels = np.asarray(source, dtype=np.float64).copy()
    strongest = np.zeros((TILE, TILE), dtype=np.float64)
    replacement = pixels.copy()
    is_water = cell in field.water
    weights = _weights(cell, 10 if is_water else field.ground_blend_px, field.seed)
    for offset, weight in zip(DIRECTIONS, weights, strict=True):
        neighbour = (cell[0] + offset[0], cell[1] + offset[1])
        if neighbour not in field.terrain or neighbour in field.walls or neighbour in field.hills:
            continue
        if field.terrain[neighbour] == field.terrain[cell]:
            continue
        if neighbour in field.water:
            continue
        active = weight > strongest
        other = np.asarray(sample_texture(field.textures[field.terrain[neighbour]], origin),
                           dtype=np.float64)
        replacement[active] = other[active]
        strongest = np.maximum(strongest, weight)
    # Water fades fully into land; ordinary ground meets halfway at the edge.
    alpha = strongest[:, :, None] * (1 if is_water else .5)
    result = pixels * (1 - alpha) + replacement * alpha
    if not is_water:
        result[:, :, :3] *= 1 - _bank_ink(field, cell)[:, :, None]
    return Image.fromarray(np.rint(result).astype(np.uint8))


def zone_overlay(size: Cell, colour: tuple[int, int, int], tick_px: int = 5) -> Image.Image:
    """Exactly 8% colour coverage with small outer-corner ticks, never a grid."""
    image = Image.new("RGBA", size, (*colour, round(.08 * 255)))
    draw = ImageDraw.Draw(image)
    for x, y, sx, sy in ((0, 0, 1, 1), (size[0] - 1, 0, -1, 1),
                          (0, size[1] - 1, 1, -1), (size[0] - 1, size[1] - 1, -1, -1)):
        draw.line((x + sx * tick_px, y, x, y, x, y + sy * tick_px), fill=(*colour, 115))
    return image


def ford_overlay(stones: tuple[Image.Image, ...], cell: Cell) -> Image.Image:
    """Place four supplied, unmodified stone sprites along a shallow diagonal."""
    if len(stones) != 4:
        message = "A ford requires exactly four stepping-stone sprites"
        raise ValueError(message)
    image = Image.new("RGBA", (TILE, TILE))
    shift = (cell[0] * 7 + cell[1] * 11) % 5 - 2
    for stone, centre in zip(stones, ((9, 38), (24, 30), (39, 35), (54, 26)), strict=True):
        image.alpha_composite(stone.convert("RGBA"),
                              (centre[0] - stone.width // 2, centre[1] - stone.height // 2 + shift))
    return image


def boundary_steps(image: Image.Image, boundary: Boundary) -> BoundaryMeasurement:
    """Measure actual adjacent-pixel luminance steps; zero steps are not edges."""
    rgb = np.asarray(image.convert("RGB"), dtype=np.float64)
    luminance: FloatGrid = rgb @ np.array([.2126, .7152, .0722])
    x, y = boundary.start
    values: FloatGrid
    match boundary.axis:
        case "vertical":
            valid = 0 < x < image.width and 0 <= y and y + boundary.length <= image.height
            if not valid or boundary.length <= 0:
                message = "Vertical boundary must have two columns and a positive in-image span"
                raise ValueError(message)
            values = luminance[y:y + boundary.length, x] - luminance[y:y + boundary.length, x - 1]
        case "horizontal":
            valid = 0 < y < image.height and 0 <= x and x + boundary.length <= image.width
            if not valid or boundary.length <= 0:
                message = "Horizontal boundary must have two rows and a positive in-image span"
                raise ValueError(message)
            values = luminance[y, x:x + boundary.length] - luminance[y - 1, x:x + boundary.length]
        case unreachable:
            assert_never(unreachable)
    rounded: FloatGrid = np.round(values, 3)
    steps = tuple(float(rounded[index]) for index in range(len(rounded)))
    longest = current = 0
    previous: float | None = None
    for value in steps:
        current = current + 1 if value != 0 and value == previous else int(value != 0)
        longest = max(longest, current)
        previous = value
    return BoundaryMeasurement(steps, longest, sum(value != 0 for value in steps))
