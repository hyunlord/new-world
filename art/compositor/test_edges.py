# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest"]
# ///
# How to run: uv run --with pillow --with numpy --with pytest python -m pytest art/compositor/test_edges.py
"""Observable transition, bank, zone and world-coordinate sampling contracts."""
from __future__ import annotations

import numpy as np
import pytest
from PIL import Image

from art.compositor import edges


def test_sampling_continues_across_world_tiles_and_wraps() -> None:
    # Given an unmistakable horizontal 256px texture ramp.
    ramp = np.zeros((256, 256, 4), dtype=np.uint8)
    ramp[:, :, 0] = np.arange(256)
    ramp[:, :, 3] = 255
    texture = Image.fromarray(ramp)
    # When sampling a tile across the wrap, including negative coordinates.
    output = edges.sample_texture(texture, (-32, 256))
    # Then the first/last pixels are source columns 224 and 31.
    assert output.getpixel((0, 0)) == (224, 0, 0, 255)
    assert output.getpixel((63, 63)) == (31, 0, 0, 255)


def test_blended_scene_is_byte_deterministic_and_not_a_straight_edge() -> None:
    # Given two flat textures whose unblended boundary is maximally square.
    textures = {"meadow": Image.new("RGBA", (256, 256), (40, 60, 30, 255)),
                "dirt": Image.new("RGBA", (256, 256), (160, 130, 80, 255))}
    terrain = {(x, y): "meadow" if x == 0 else "dirt" for y in range(3) for x in range(2)}
    field = edges.EdgeField(terrain, textures)
    # When composing the same six-tile scene twice.
    outputs = []
    for _ in range(2):
        image = Image.new("RGBA", (128, 192))
        for cell in terrain:
            image.alpha_composite(edges.render_tile(field, cell), (cell[0] * 64, cell[1] * 64))
        outputs.append(image)
    # Then bytes agree and the measured full boundary has no >24px constant step.
    assert outputs[0].tobytes() == outputs[1].tobytes()
    result = edges.boundary_steps(outputs[0], edges.Boundary((64, 0), 192, "vertical"))
    assert result.longest_identical_step <= 24
    assert result.nonzero_steps > 0
    assert len(set(result.steps)) > 8


def test_metric_rejects_an_unblended_square_boundary() -> None:
    # Given a flat sharp terrain boundary.
    image = Image.new("RGBA", (128, 64), (20, 20, 20, 255))
    image.paste((120, 120, 120, 255), (64, 0, 128, 64))
    # When measuring the actual luminance jump along it.
    result = edges.boundary_steps(image, edges.Boundary((64, 0), 64, "vertical"))
    # Then the entire 64px edge is detected, exceeding the 24px bar.
    assert result.longest_identical_step == 64


def test_wall_cells_and_adjacent_floor_edges_stay_crisp() -> None:
    # Given packed earth next to rock with a wall occupying the latter.
    textures = {"floor": Image.new("RGBA", (256, 256), (80, 60, 40, 255)),
                "rock": Image.new("RGBA", (256, 256), (180, 180, 180, 255))}
    field = edges.EdgeField({(0, 0): "floor", (1, 0): "rock"}, textures,
                            walls=frozenset({(1, 0)}))
    # When rendering both sides.
    floor = edges.render_tile(field, (0, 0))
    wall = edges.render_tile(field, (1, 0))
    # Then neither receives a transition under the wall.
    assert floor.getextrema() == ((80, 80), (60, 60), (40, 40), (255, 255))
    assert wall.getextrema() == ((180, 180), (180, 180), (180, 180), (255, 255))


def test_water_bank_falls_off_only_inside_ten_pixels() -> None:
    # Given water with land to the west.
    field = edges.EdgeField({(0, 0): "water", (-1, 0): "land"},
                            {"water": Image.new("RGBA", (256, 256), "blue"),
                             "land": Image.new("RGBA", (256, 256), "green")},
                            water=frozenset({(0, 0)}))
    # When deriving its declared alpha bank mask.
    alpha = np.asarray(edges.water_mask(field, (0, 0)))
    # Then the 10px falloff varies along the bank and interior is opaque.
    assert np.all(alpha[:, 10:] == 255)
    assert alpha[:, :10].min() < 128
    assert len(np.unique(alpha[:, 4])) > 8


def test_corner_falloff_uses_max_not_sum() -> None:
    # Given the same land on the north and west sides of a water corner.
    textures = {"water": Image.new("RGBA", (256, 256), "blue"),
                "land": Image.new("RGBA", (256, 256), "green")}
    one = edges.EdgeField({(0, 0): "water", (-1, 0): "land"}, textures,
                          water=frozenset({(0, 0)}))
    other = edges.EdgeField({(0, 0): "water", (0, -1): "land"}, textures,
                            water=frozenset({(0, 0)}))
    corner = edges.EdgeField({(0, 0): "water", (-1, 0): "land", (0, -1): "land"},
                             textures, water=frozenset({(0, 0)}))
    # When combining the two edge masks.
    result = np.asarray(edges.water_mask(corner, (0, 0)))
    # Then alpha equals the minimum opacity, equivalent to max falloff.
    assert np.array_equal(result, np.minimum(np.asarray(edges.water_mask(one, (0, 0))),
                                             np.asarray(edges.water_mask(other, (0, 0)))))


def test_bank_ink_is_two_pixels_wide_on_a_noisy_land_side_contour() -> None:
    # Given land immediately east of water.
    textures = {"water": Image.new("RGBA", (256, 256), (0, 0, 200, 255)),
                "land": Image.new("RGBA", (256, 256), (120, 100, 60, 255))}
    field = edges.EdgeField({(0, 0): "water", (1, 0): "land"}, textures,
                            water=frozenset({(0, 0)}))
    # When drawing land.
    tile = np.asarray(edges.render_tile(field, (1, 0)))
    # Then every row has exactly two ink pixels, but their land-side location varies.
    ink = tile[:, :, 0] < 120
    assert np.all(ink.sum(axis=1) == 2)
    assert len(np.unique(np.argmax(ink, axis=1))) >= 3
    assert np.all(tile[:, 6:, :3] == np.array([120, 100, 60]))


def test_bank_corner_does_not_multiply_ink_darkness() -> None:
    # Given water north and west of the same land corner.
    textures = {"water": Image.new("RGBA", (256, 256), (0, 0, 200, 255)),
                "land": Image.new("RGBA", (256, 256), (120, 100, 60, 255))}
    field = edges.EdgeField({(0, 0): "land", (-1, 0): "water", (0, -1): "water"},
                            textures, water=frozenset({(-1, 0), (0, -1)}))
    # When two ink strokes meet.
    tile = np.asarray(edges.render_tile(field, (0, 0)))
    # Then their maximum35% ink never compounds into a darker corner.
    assert tile[:, :, 0].min() >= round(120 * .65)


def test_zone_is_eight_percent_with_only_outer_corner_ticks() -> None:
    # Given a three-tile stockpile rectangle.
    # When drawing its overlay.
    tile = np.asarray(edges.zone_overlay((192, 128), (100, 160, 200)))
    # Then tile-grid coordinates and centre share the same 8% alpha.
    assert tile[64, 64, 3] == round(.08 * 255)
    assert tile[64, 96, 3] == round(.08 * 255)
    assert tile[0, 0, 3] > tile[64, 64, 3]
    assert np.all(tile[10:-10, 10:-10, 3] == round(.08 * 255))


def test_ford_places_four_distinct_stones_without_mutating_sources() -> None:
    # Given four differently coloured stepping stones.
    stones = tuple(Image.new("RGBA", (8, 8), colour) for colour in
                   ((200, 0, 0, 255), (0, 200, 0, 255), (0, 0, 200, 255), (200, 200, 0, 255)))
    # When arranging one ford tile.
    output = edges.ford_overlay(stones, (12, 9))
    # Then all four source colours survive in a transparent tile.
    colours = {tuple(pixel) for pixel in np.asarray(output).reshape(-1, 4)}
    assert all(stone.getpixel((0, 0)) in colours for stone in stones)
    assert output.getpixel((0, 0)) == (0, 0, 0, 0)


def test_terrain_blending_stops_at_twelve_pixels() -> None:
    # Given flat meadow and dirt meeting on the east side.
    textures = {"meadow": Image.new("RGBA", (256, 256), (20, 40, 20, 255)),
                "dirt": Image.new("RGBA", (256, 256), (160, 120, 80, 255))}
    field = edges.EdgeField({(0, 0): "meadow", (1, 0): "dirt"}, textures)
    # When blending their boundary.
    pixels = np.asarray(edges.render_tile(field, (0, 0)))
    # Then the 52 interior columns are unchanged, and the border is blended.
    assert np.all(pixels[:, :52] == np.array([20, 40, 20, 255]))
    assert np.any(pixels[:, 52:, 0] > 20)


def test_horizontal_metric_and_invalid_span() -> None:
    # Given a 64px long horizontal edge.
    image = Image.new("RGBA", (64, 128), (20, 20, 20, 255))
    image.paste((120, 120, 120, 255), (0, 64, 64, 128))
    # When measuring it and requesting a nonexistent adjacent row.
    result = edges.boundary_steps(image, edges.Boundary((0, 64), 64, "horizontal"))
    # Then all 64 steps are detected and an invalid measurement fails explicitly.
    assert result.longest_identical_step == 64
    with pytest.raises(ValueError, match="two rows"):
        edges.boundary_steps(image, edges.Boundary((0, 0), 64, "horizontal"))
