# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow"]
# ///
# How to run: python3 -m pytest art/compositor/test_walls.py -q.
"""World-phased wall strips, open doors and south-facing hill cuts.

The exposed cap is 16px and the face is 44px, ending at tile y+43.
For a south-connected tile the cap's top surface continues for 64px to
meet the next cap: rows repeat, never stretch or retouch generated art.
This top-down connector prevents 48px gaps along vertical runs. T/cross
junctions have caps only. Door sprites own their transparency completely.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

from PIL import Image

Cell = tuple[int, int]
TILE: Final = 64
CAP: Final = 16
FACE: Final = 44
ANCHOR: Final = 43


@dataclass(frozen=True, slots=True)
class WallTextures:
    """Unmodified, normalized source strips; dimensions checked at boundary."""

    face: Image.Image
    cap: Image.Image
    end: Image.Image
    inner_corner: Image.Image

    def __post_init__(self) -> None:
        for name, image, size in (
            ("face", self.face, (256, FACE)),
            ("cap", self.cap, (256, CAP)),
            ("end", self.end, (24, CAP + FACE)),
            ("inner_corner", self.inner_corner, (CAP, CAP)),
        ):
            if image.size != size:
                message = f"{name} must be {size}; received {image.size}"
                raise ValueError(message)


@dataclass(frozen=True, slots=True)
class Door:
    """A 60px standing slab/frame, centred within its wall cell."""

    cell: Cell
    image: Image.Image

    def __post_init__(self) -> None:
        if self.image.height != CAP + FACE or not 1 <= self.image.width <= TILE:
            message = f"Door must be 1..64 wide and 60 high; received {self.image.size}"
            raise ValueError(message)


@dataclass(frozen=True, slots=True)
class WallTile:
    cell: Cell
    image: Image.Image
    face_visible: bool

    @property
    def origin(self) -> Cell:
        return self.cell[0] * TILE, self.cell[1] * TILE - CAP

    @property
    def base_y(self) -> int:
        return self.cell[1] * TILE + ANCHOR


@dataclass(frozen=True, slots=True)
class WallRender:
    image: Image.Image
    origin: Cell


def _strip_tile(strip: Image.Image, world_x: int) -> Image.Image:
    offset = world_x * TILE % strip.width
    return strip.crop((offset, 0, offset + TILE, strip.height)).convert("RGBA")


def wall_tiles(
    cells: frozenset[Cell], textures: WallTextures, doors: tuple[Door, ...] = (),
) -> tuple[WallTile, ...]:
    """Return tiles sorted by base y then x, retaining doors as neighbours."""
    replacements = {door.cell: door.image for door in doors}
    if len(replacements) != len(doors) or not replacements.keys() <= cells:
        message = "Door cells must be unique and present in the wall cell set"
        raise ValueError(message)
    result: list[WallTile] = []
    for x, y in sorted(cells, key=lambda cell: (cell[1], cell[0])):
        if (x, y) in replacements:
            door = replacements[(x, y)]
            image = Image.new("RGBA", (TILE, CAP + FACE))
            image.alpha_composite(door.convert("RGBA"), ((TILE - door.width) // 2, 0))
            result.append(WallTile((x, y), image, False))
            continue
        north, east, south, west = (
            (x + dx, y + dy) in cells for dx, dy in ((0, -1), (1, 0), (0, 1), (-1, 0))
        )
        junction = sum((north, east, south, west)) >= 3
        face_visible = not south and not junction
        image = Image.new("RGBA", (TILE, TILE if south else CAP + FACE))
        cap = _strip_tile(textures.cap, x)
        for top in range(0, TILE if south else CAP, CAP):
            image.alpha_composite(cap, (0, top))
        if face_visible:
            image.alpha_composite(_strip_tile(textures.face, x), (0, CAP))
        if not junction:
            end = textures.end.convert("RGBA")
            if not face_visible:
                end = end.crop((0, 0, end.width, CAP))
            if not west:
                image.alpha_composite(end, (0, 0))
            if not east:
                image.alpha_composite(end, (TILE - end.width, 0))
            if south and (east or west):
                corner_x = TILE - CAP if east else 0
                image.alpha_composite(textures.inner_corner.convert("RGBA"), (corner_x, 0))
        result.append(WallTile((x, y), image, face_visible))
    return tuple(result)


def render_walls(
    cells: frozenset[Cell], textures: WallTextures, doors: tuple[Door, ...] = (),
) -> WallRender:
    """Bake a wall chunk with an explicit world-pixel origin for placement."""
    tiles = wall_tiles(cells, textures, doors)
    if not tiles:
        return WallRender(Image.new("RGBA", (1, 1)), (0, 0))
    left = min(tile.origin[0] for tile in tiles)
    top = min(tile.origin[1] for tile in tiles)
    right = max(tile.origin[0] + tile.image.width for tile in tiles)
    bottom = max(tile.origin[1] + tile.image.height for tile in tiles)
    canvas = Image.new("RGBA", (right - left, bottom - top))
    for tile in tiles:
        canvas.alpha_composite(tile.image, (tile.origin[0] - left, tile.origin[1] - top))
    return WallRender(canvas, (left, top))


def hill_tiles(
    hills: frozenset[Cell], passable: frozenset[Cell], textures: WallTextures,
) -> tuple[WallTile, ...]:
    """Overlay cliffs only where an undug hill's south cell is passable.

    The caller retains its existing top-down rock ground for every hill
    interior. Unlike wall runs, continuous terrain cliffs have no log ends.
    """
    if hills & passable:
        message = "Hill and passable sets must be disjoint"
        raise ValueError(message)
    result: list[WallTile] = []
    for x, y in sorted(hills, key=lambda cell: (cell[1], cell[0])):
        if (x, y + 1) not in passable:
            continue
        image = Image.new("RGBA", (TILE, CAP + FACE))
        image.alpha_composite(_strip_tile(textures.cap, x))
        image.alpha_composite(_strip_tile(textures.face, x), (0, CAP))
        result.append(WallTile((x, y), image, True))
    return tuple(result)
