# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m pytest art/compositor/test_material_walls.py -q.
"""Adversarial directional sources prove world phasing and face boundaries."""
from dataclasses import replace
from pathlib import Path

import numpy as np
import pytest
from PIL import Image

from art.compositor import walls
from art.materials.render import MaterialWallTextures
from art.materials.resolve import read_catalog, resolve


def sources() -> MaterialWallTextures:
    resolved = resolve(read_catalog(Path(__file__).resolve().parents[2]), "log", "oak")
    resolved = resolved.model_copy(update={"top": resolved.top.model_copy(update={"ink_edge_px":0})})
    ew = np.zeros((64, 256, 4), dtype=np.uint8)
    ns = np.zeros((256, 64, 4), dtype=np.uint8)
    ew[..., 0] = np.arange(256)[None, :]
    ns[..., 1] = np.arange(256)[:, None]
    ew[..., 3] = ns[..., 3] = 255
    return MaterialWallTextures(resolved, Image.new("RGBA", (256,44), "blue"),
        Image.fromarray(ew), Image.fromarray(ns), Image.new("RGBA", (24,44), "magenta"),
        Image.new("RGBA", (64,64), "yellow"), Image.new("RGBA", (64,60), "black"),
        Image.new("RGBA", (64,60)))


def test_direction_uses_independent_strip_and_correct_world_axis() -> None:
    # Given two strips carrying independent x/y channels.
    source = sources()
    # When rendering horizontal and vertical runs through negative coordinates.
    horizontal = walls.wall_tiles(frozenset((x, 0) for x in range(-2, 6)), source)
    vertical = walls.wall_tiles(frozenset((0, y) for y in range(-2, 6)), source)
    # Then each interior top samples its own axis with full-strip phase.
    for tile in horizontal:
        assert tile.image.getpixel((32,8)) == ((tile.cell[0]*64+32)%256,0,0,255)
    for tile in vertical:
        assert tile.image.getpixel((32,8)) == (0,(tile.cell[1]*64+8)%256,0,255)


@pytest.mark.parametrize("cells", [
    frozenset({(0,0)}), frozenset({(0,0),(1,0),(0,1)}),
    frozenset({(0,0),(-1,0),(1,0),(0,1)}),
    frozenset({(0,0),(-1,0),(1,0),(0,1),(0,-1)}),
])
def test_corner_junction_and_isolated_use_junction_source(cells: frozenset[tuple[int,int]]) -> None:
    # Given a unique junction source that cannot be confused with either strip.
    source = sources()
    # When assembling each nonstraight topology.
    tile = next(row for row in walls.wall_tiles(cells, source) if row.cell == (0,0))
    # Then the top comes from the actual junction image.
    assert tile.image.getpixel((32,8)) == (255,255,0,255)


def test_faces_keep_anchor_and_ends_never_cover_tops() -> None:
    # Given an L-shaped room edge and magenta face end pieces.
    cells = frozenset({(0,0),(0,1),(1,1),(2,1)})
    # When the material assembly draws its surfaces.
    tiles = walls.wall_tiles(cells, sources())
    # Then only exposed south faces exist and all end paint remains below the top.
    for tile in tiles:
        assert tile.face_visible == ((tile.cell[0],tile.cell[1]+1) not in cells)
        top = tile.image.crop((0,0,64,16)).tobytes()
        assert all(top[index:index+4] != bytes((255,0,255,255)) for index in range(0,len(top),4))
        if tile.face_visible:
            assert tile.base_y == tile.cell[1]*64+43
            assert tile.image.getpixel((32,59)) == (0,0,255,255)


def test_north_facing_tee_keeps_its_exposed_south_face() -> None:
    # Given a junction connected north, east and west, with its south side open.
    occupied = frozenset({(0,0),(0,-1),(-1,0),(1,0)})
    # When drawing its centre as one material subset of the global wall layout.
    tile = walls.wall_tiles(frozenset({(0,0)}), sources(), occupancy=occupied)[0]
    # Then junction degree cannot hide the exposed face or add internal ends.
    assert tile.face_visible
    assert tile.image.getpixel((32,59)) == (0,0,255,255)
    assert tile.image.getpixel((0,30)) == tile.image.getpixel((63,30)) == (0,0,255,255)


def test_width_is_centred_and_shadow_does_not_move_face() -> None:
    # Given the narrow material-top owner option on a vertical run.
    source = sources()
    spec = source.resolved.model_copy(update={"top":source.resolved.top.model_copy(update={"width_px":44,"side_shadow":.25})})
    # When rendering the middle of the vertical run.
    tile = walls.wall_tiles(frozenset({(0,0),(0,1),(0,2)}), replace(source,resolved=spec))[1]
    # Then the material begins at x10 and ends at x53 with a soft external shadow.
    assert tile.image.getchannel("A").crop((10,20,11,21)).tobytes()[0] == 255
    assert tile.image.getchannel("A").crop((53,20,54,21)).tobytes()[0] == 255
    assert 0 < tile.image.getchannel("A").crop((8,20,9,21)).tobytes()[0] < 255
    assert tile.image.getchannel("A").crop((0,20,1,21)).tobytes()[0] == 0


def test_narrow_corner_has_only_connected_arms() -> None:
    # Given an east/south corner with no north/west neighbours.
    source = sources()
    spec = source.resolved.model_copy(update={"top": source.resolved.top.model_copy(update={"width_px":44})})
    # When rendering the narrow corner without shadow or ink.
    tile = next(row for row in walls.wall_tiles(frozenset({(0,0),(1,0),(0,1)}), replace(source,resolved=spec)) if row.cell == (0,0))
    # Then only east and south connect; the omitted arms stay transparent.
    alpha = tile.image.getchannel("A")
    assert alpha.getpixel((0,8)) == alpha.getpixel((32,0)) == 0
    assert alpha.getpixel((63,8)) == alpha.getpixel((32,63)) == 255


def test_narrow_isolated_top_is_centred_without_four_stubs() -> None:
    # Given an isolated narrow top.
    source = sources()
    spec = source.resolved.model_copy(update={"top": source.resolved.top.model_copy(update={"width_px":44})})
    # When rendering its junction texture.
    tile = walls.wall_tiles(frozenset({(0,0)}), replace(source,resolved=spec))[0]
    # Then the top has a centred patch, with no arms to absent neighbours.
    assert tile.image.crop((0,0,64,16)).getbbox() == (10,2,54,13)
