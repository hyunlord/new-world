# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "numpy"]
# ///
# How to run: python3 -m pytest art/compositor/test_material_walls.py -q.
"""Schema-driven wall assembly with independently lit along-run top strips.

The 16px south-visible top and 44px face retain the original projection.
Connected north/south tops extend to 64px. The frozen comparison renderer
remains available through its original input type, with no material lookup.
"""
from __future__ import annotations

from typing import Final, Literal, assert_never

import numpy as np
from PIL import Image, ImageChops, ImageDraw, ImageFilter

from art.compositor import legacy_walls
from art.compositor.legacy_walls import (
    ANCHOR,
    CAP,
    FACE,
    TILE,
    Cell,
    Door,
    WallRender,
    WallTextures,
    WallTile,
    hill_tiles,
)
from art.compositor.wall_style import CapStyle, WallParameters
from art.materials.render import MaterialWallTextures
from art.materials.schema import Top

Direction = Literal["ew", "ns", "junction"]
Neighbours = tuple[bool, bool, bool, bool]
EMPTY_CELLS: Final[frozenset[Cell]] = frozenset()

__all__ = ["ANCHOR", "CAP", "FACE", "TILE", "Cell", "Door", "WallRender", "WallTextures", "WallTile", "hill_tiles", "render_walls", "wall_tiles"]


def _direction(neighbours: Neighbours) -> Direction:
    north, east, south, west = neighbours
    horizontal, vertical = east or west, north or south
    if horizontal and not vertical:
        return "ew"
    if vertical and not horizontal:
        return "ns"
    return "junction"


def _top_image(textures: MaterialWallTextures, cell: Cell, direction: Direction,
               height: int) -> Image.Image:
    x, y = cell
    match direction:
        case "ew":
            offset = x * TILE % textures.top_ew.width
            return textures.top_ew.crop((offset, 0, offset + TILE, height))
        case "ns":
            offset = y * TILE % textures.top_ns.height
            return textures.top_ns.crop((0, offset, TILE, offset + height))
        case "junction":
            return textures.junction.crop((0, 0, TILE, height))
        case _:
            assert_never(direction)


def _top_mask(size: tuple[int, int], direction: Direction, neighbours: Neighbours, top: Top) -> Image.Image:
    width, height = size
    mask = Image.new("L", size)
    margin_x = (width - top.width_px) // 2
    transverse = max(1, round(height * top.width_px / TILE))
    margin_y = (height - transverse) // 2
    draw = ImageDraw.Draw(mask)
    match direction:
        case "ew":
            draw.rectangle((0, margin_y, width - 1, margin_y + transverse - 1), fill=255)
        case "ns":
            draw.rectangle((margin_x, 0, margin_x + top.width_px - 1, height - 1), fill=255)
        case "junction":
            cross_height = max(1, round(min(CAP, height) * top.width_px / TILE))
            cross_top = (min(CAP, height) - cross_height) // 2
            cross_bottom = cross_top + cross_height - 1
            left, right = margin_x, margin_x + top.width_px - 1
            draw.rectangle((left, cross_top, right, cross_bottom), fill=255)
            north, east, south, west = neighbours
            if north:
                draw.rectangle((left, 0, right, cross_bottom), fill=255)
            if south:
                draw.rectangle((left, cross_top, right, height - 1), fill=255)
            if east:
                draw.rectangle((left, cross_top, width - 1, cross_bottom), fill=255)
            if west:
                draw.rectangle((0, cross_top, right, cross_bottom), fill=255)
        case _:
            assert_never(direction)
    return mask


def _finish_top(source: Image.Image, direction: Direction, neighbours: Neighbours,
                top: Top) -> Image.Image:
    mask = _top_mask(source.size, direction, neighbours, top)
    image = Image.new("RGBA", source.size)
    if top.side_shadow:
        shadow = Image.new("RGBA", source.size, (20, 18, 16, 0))
        alpha = mask.filter(ImageFilter.GaussianBlur(3)).point([round(value * top.side_shadow) for value in range(256)])
        shadow.putalpha(alpha)
        image.alpha_composite(shadow)
    material = source.copy()
    material.putalpha(ImageChops.multiply(source.getchannel("A"), mask))
    edge = top.ink_edge_px
    if edge:
        padded = np.pad(np.asarray(mask, dtype=np.uint8), edge, mode="edge")
        north, east, south, west = neighbours
        if not north:
            padded[:edge, :] = 0
        if not east:
            padded[:, -edge:] = 0
        if not south:
            padded[-edge:, :] = 0
        if not west:
            padded[:, :edge] = 0
        inset = Image.fromarray(padded).filter(ImageFilter.MinFilter(2 * edge + 1))
        interior = inset.crop((edge, edge, edge + source.width, edge + source.height))
        ink_mask = ImageChops.subtract(mask, interior).point([round(value * .8) for value in range(256)])
        ink = Image.new("RGBA", source.size, (20, 18, 16, 0))
        ink.putalpha(ink_mask)
        material.alpha_composite(ink)
    image.alpha_composite(material)
    return image


def _material_tiles(cells: frozenset[Cell], textures: MaterialWallTextures,
                    doors: tuple[Door, ...], occupancy: frozenset[Cell],
                    openings: frozenset[Cell]) -> tuple[WallTile, ...]:
    replacements = {door.cell: door.image for door in doors}
    if len(replacements) != len(doors) or not replacements.keys() <= cells:
        message = "Door cells must be unique and present in the wall cell set"
        raise ValueError(message)
    apertures = openings | replacements.keys()
    faces = frozenset((x, y) for x, y in occupancy
                      if (x, y) not in apertures and (x, y + 1) not in occupancy)
    result: list[WallTile] = []
    for x, y in sorted(cells, key=lambda cell: (cell[1], cell[0])):
        if (x, y) in replacements:
            door = replacements[(x, y)]
            image = Image.new("RGBA", (TILE, CAP + FACE))
            image.alpha_composite(door, ((TILE - door.width) // 2, 0))
            result.append(WallTile((x, y), image, False))
            continue
        neighbours = ((x, y - 1) in occupancy, (x + 1, y) in occupancy,
                      (x, y + 1) in occupancy, (x - 1, y) in occupancy)
        height = TILE if neighbours[2] else CAP
        direction = _direction(neighbours)
        image = Image.new("RGBA", (TILE, max(height, CAP + FACE)))
        surface = _top_image(textures, (x, y), direction, height)
        image.alpha_composite(_finish_top(surface, direction, neighbours, textures.resolved.top))
        face_visible = (x, y) in faces
        if face_visible:
            offset = x * TILE % textures.face.width
            image.alpha_composite(textures.face.crop((offset, 0, offset + TILE, FACE)), (0, CAP))
            if (x - 1, y) not in faces:
                image.alpha_composite(textures.end, (0, CAP))
            if (x + 1, y) not in faces:
                image.alpha_composite(textures.end, (TILE - textures.end.width, CAP))
        result.append(WallTile((x, y), image, face_visible))
    return tuple(result)


def wall_tiles(cells: frozenset[Cell], textures: MaterialWallTextures | WallTextures,
               doors: tuple[Door, ...] = (), *, cap_style: CapStyle = "dark",
               params: WallParameters | None = None, occupancy: frozenset[Cell] | None = None,
               openings: frozenset[Cell] = EMPTY_CELLS) -> tuple[WallTile, ...]:
    """Draw selected cells using global topology, independent of material grouping."""
    occupied = cells if occupancy is None else occupancy
    if not cells <= occupied or not openings <= occupied:
        message = "Draw cells and openings must be present in global wall occupancy"
        raise ValueError(message)
    match textures:
        case MaterialWallTextures():
            return _material_tiles(cells, textures, doors, occupied, openings)
        case WallTextures():
            return tuple(tile for tile in legacy_walls.wall_tiles(occupied, textures, doors, cap_style=cap_style, params=params)
                         if tile.cell in cells)
        case _:
            assert_never(textures)


def render_walls(cells: frozenset[Cell], textures: MaterialWallTextures | WallTextures,
                 doors: tuple[Door, ...] = (), *, cap_style: CapStyle = "dark",
                 params: WallParameters | None = None, occupancy: frozenset[Cell] | None = None,
                 openings: frozenset[Cell] = EMPTY_CELLS) -> WallRender:
    """Bake a topology with an explicit world-pixel origin for composition."""
    tiles = wall_tiles(cells, textures, doors, cap_style=cap_style, params=params,
                       occupancy=occupancy, openings=openings)
    if not tiles:
        return WallRender(Image.new("RGBA", (1, 1)), (0, 0))
    left = min(tile.origin[0] for tile in tiles)
    upper = min(tile.origin[1] for tile in tiles)
    right = max(tile.origin[0] + tile.image.width for tile in tiles)
    lower = max(tile.origin[1] + tile.image.height for tile in tiles)
    image = Image.new("RGBA", (right - left, lower - upper))
    for tile in tiles:
        image.alpha_composite(tile.image, (tile.origin[0] - left, tile.origin[1] - upper))
    return WallRender(image, (left, upper))
