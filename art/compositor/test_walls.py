# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy"]
# ///
# How to run: python3 -m pytest art/compositor/test_walls.py -q.
"""Synthetic alpha and sampling checks for the declared wall geometry."""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import numpy as np
import pytest
from PIL import Image

from art.compositor import wall_metrics, walls


def textures() -> walls.WallTextures:
    return walls.WallTextures(
        Image.new("RGBA", (256, 44), (80, 60, 40, 255)),
        Image.new("RGBA", (256, 16), (180, 160, 140, 255)),
        Image.new("RGBA", (24, 60), (110, 90, 70, 255)),
        Image.new("RGBA", (16, 16), (180, 160, 140, 255)),
    )


@pytest.mark.parametrize("cells", [
    frozenset({(0, 0), (1, 0), (2, 0)}),
    frozenset({(0, 0), (0, 1), (1, 1)}),
    frozenset({(0, 0), (0, 1), (0, 2), (1, 2), (2, 2), (2, 1), (2, 0)}),
    frozenset({(0, 0), (1, 0), (2, 0), (1, 1)}),
    frozenset({(x, y) for y in range(3) for x in range(3) if x in (0, 2) or y in (0, 2)}),
])
def test_layout_expected_surfaces_have_no_alpha_gaps(cells: frozenset[tuple[int, int]]) -> None:
    # Given opaque strips and each required synthetic layout.
    # When compositing in world space.
    result = walls.render_walls(cells, textures())
    # Then every top band and every vertical connection is fully covered.
    for x, y in cells:
        px, py = x * 64 - result.origin[0], y * 64 - 16 - result.origin[1]
        assert result.image.getchannel("A").crop((px, py, px + 64, py + 16)).getextrema() == (255, 255)
        if (x, y + 1) in cells:
            assert result.image.getchannel("A").crop((px, py, px + 64, py + 64)).getextrema() == (255, 255)


def test_hidden_faces_and_junctions_never_draw_south_faces() -> None:
    # Given a T whose centre has no south neighbour, plus a vertical pair.
    cells = frozenset({(0, 1), (1, 1), (2, 1), (1, 0)})
    # When assembling each tile.
    tiles = {tile.cell: tile for tile in walls.wall_tiles(cells, textures())}
    # Then the T centre is cap only and the north cell has no face.
    assert not tiles[(1, 1)].face_visible
    assert tiles[(1, 1)].image.getbbox() == (0, 0, 64, 16)
    assert not tiles[(1, 0)].face_visible


def test_face_bottom_is_anchor_and_extent_is_exactly_44() -> None:
    # Given a three-cell horizontal run.
    # When drawing its middle without end caps.
    tile = walls.wall_tiles(frozenset({(0, 2), (1, 2), (2, 2)}), textures())[1]
    # Then face rows occupy world y=128..171, exactly the stated 44px.
    assert tile.origin == (64, 112)
    assert tile.base_y == 171
    face = np.asarray(tile.image)[16:, :, :3]
    assert face.shape == (44, 64, 3)
    assert tile.image.crop((0, 16, 64, 60)).getextrema() == ((80, 80), (60, 60), (40, 40), (255, 255))


def test_world_x_sampling_continues_across_positive_and_negative_tiles() -> None:
    # Given an asymmetric strip whose RGB encodes its source x.
    strip = np.zeros((16, 256, 4), dtype=np.uint8)
    strip[:, :, 0] = np.arange(256)
    strip[:, :, 3] = 255
    source = textures()
    source = walls.WallTextures(source.face, Image.fromarray(strip), source.end, source.inner_corner)
    cells = frozenset((x, 0) for x in range(-2, 6))
    # When rendering a run.
    result = walls.render_walls(cells, source, cap_style="struct")
    # Then interior cap pixels preserve world phase across wrap and crop boundaries.
    red = np.asarray(result.image)[0, 64:-64, 0]
    expected = np.arange(-64, 320) % 256
    assert np.array_equal(red, expected)


def test_open_door_keeps_transparent_aperture_and_wall_connectivity() -> None:
    # Given a complete ring with an open frame in its south edge.
    cells = frozenset((x, y) for y in range(3) for x in range(3) if x in (0, 2) or y in (0, 2))
    frame = Image.new("RGBA", (64, 60))
    frame.paste((90, 60, 30, 255), (0, 0, 64, 8))
    frame.paste((90, 60, 30, 255), (0, 0, 8, 60))
    frame.paste((90, 60, 30, 255), (56, 0, 64, 60))
    # When replacing one ring tile with the actual door frame.
    result = walls.render_walls(cells, textures(), (walls.Door((1, 2), frame),))
    # Then the opening remains transparent, and both neighbouring wall joins are opaque.
    alpha = np.asarray(result.image)[:, :, 3]
    row = 128 - result.origin[1] + 20
    assert alpha[row, 64 + 32] == 0
    assert alpha[row, 63] == 255
    assert alpha[row, 128] == 255


def test_hill_only_exposes_faces_to_passable_south_cells() -> None:
    # Given hill interior, a dug pocket and an impassable map boundary.
    hills = frozenset({(0, 0), (1, 0), (0, 1)})
    # When extracting visible cliff surfaces.
    tiles = walls.hill_tiles(hills, frozenset({(1, 1)}), textures())
    # Then only the pocket north edge gets a 44px rock face.
    assert len(tiles) == 1
    assert tiles[0].cell == (1, 0)
    assert tiles[0].image.size == (64, 60)
    assert tiles[0].face_visible


def test_seam_instrument_rejects_gradient_wrap_and_accepts_periodic_strip() -> None:
    # Given a ramp with a 255-level discontinuity, and a periodic 0/255 stripe.
    ramp = np.repeat(np.arange(256, dtype=np.uint8)[None, :, None], 44, axis=0)
    ramp = np.repeat(ramp, 3, axis=2)
    # When measuring all three joins of a four-repeat strip.
    failed = wall_metrics.measure_strip(Image.fromarray(ramp))
    periodic = wall_metrics.measure_strip(Image.fromarray((ramp % 2) * 255))
    # Then literal known statistics, not a self-derived threshold, determine acceptance.
    assert failed.join_mad == (255.0, 255.0, 255.0)
    assert failed.interior_p95 == 1.0
    assert failed.limit == 1.15
    assert not failed.passed
    assert periodic.join_mad == (255.0, 255.0, 255.0)
    assert periodic.interior_p95 == 255.0
    assert periodic.passed


def test_render_is_identical_when_input_insertion_order_changes() -> None:
    # Given equivalent sets built in opposite order.
    points = [(0, 0), (1, 0), (0, 1), (1, 1)]
    # When rendering both inputs.
    first = walls.render_walls(frozenset(points), textures())
    second = walls.render_walls(frozenset(reversed(points)), textures())
    # Then bytes and world origin are identical.
    assert first.origin == second.origin
    assert first.image.tobytes() == second.image.tobytes()


def test_closed_door_blocks_aperture_at_full_wall_height() -> None:
    # Given a closed 60px slab replacing the centre of a run.
    cells = frozenset({(0, 0), (1, 0), (2, 0)})
    door = walls.Door((1, 0), Image.new("RGBA", (64, 60), (1, 2, 3, 255)))
    # When rendering it in the wall run.
    result = walls.render_walls(cells, textures(), (door,))
    # Then every pixel of its aperture is the actual opaque slab.
    assert result.image.crop((64, 0, 128, 60)).getextrema() == ((1, 1), (2, 2), (3, 3), (255, 255))


def test_cross_draws_only_connected_cap_surface() -> None:
    # Given a four-way junction with distinguishable face, cap and end colours.
    cells = frozenset({(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)})
    # When assembling its middle.
    tile = next(tile for tile in walls.wall_tiles(cells, textures(), cap_style="struct") if tile.cell == (1, 1))
    # Then no face or end texture contaminates any of the junction top.
    assert not tile.face_visible
    assert tile.image.getextrema() == ((180, 180), (160, 160), (140, 140), (255, 255))


def test_invalid_door_placement_is_rejected() -> None:
    # Given a door outside the wall topology.
    door = walls.Door((3, 0), Image.new("RGBA", (64, 60)))
    # When rendering it, then silent disconnected placements are rejected.
    with pytest.raises(ValueError, match="present in the wall cell set"):
        _ = walls.render_walls(frozenset({(0, 0)}), textures(), (door,))


def test_invalid_texture_dimensions_are_rejected() -> None:
    # Given a source cap that would leave a gap after a 64px crop.
    source = textures()
    # When crossing the texture boundary, then incompatible strips are rejected.
    with pytest.raises(ValueError, match="cap must be"):
        _ = walls.WallTextures(source.face, Image.new("RGBA", (63, 16)), source.end, source.inner_corner)


def test_constant_strip_zero_p95_has_no_division_error() -> None:
    # Given a constant texture with zero interior variation.
    strip = Image.new("RGB", (256, 16), (40, 50, 60))
    # When measuring the literal registered inequality.
    result = wall_metrics.measure_strip(strip)
    # Then a zero seam passes exactly without inventing epsilon headroom.
    assert result.limit == 0
    assert result.join_mad == (0, 0, 0)
    assert result.passed


def test_metric_cli_outputs_evidence_and_fails_bad_seams(tmp_path: Path) -> None:
    # Given one good strip followed by a non-seamless gradient.
    good, bad = tmp_path / "good.png", tmp_path / "bad.png"
    Image.new("RGB", (256, 16), (30, 40, 50)).save(good)
    ramp = np.tile(np.arange(256, dtype=np.uint8)[None, :, None], (16, 1, 3))
    Image.fromarray(ramp).save(bad)
    # When invoking the real CLI on both source files.
    completed = subprocess.run(
        [sys.executable, "-m", "art.compositor.wall_metrics", str(good), str(bad)],
        check=False, capture_output=True, text=True,
    )
    # Then output includes both independent records and the exit code reports failure.
    records = [json.loads(line) for line in completed.stdout.splitlines()]
    assert completed.returncode == 1
    assert len(records) == 2
    assert records[0]["passed"] is True
    assert records[1]["passed"] is False
    assert records[1]["join_columns"] == [256, 512, 768]
    assert records[1]["join_mad"] == [255.0, 255.0, 255.0]
