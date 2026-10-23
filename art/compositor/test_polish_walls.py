# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "pydantic", "typer"]
# ///
# How to run: python3 -m pytest art/compositor/test_polish_walls.py -q.
"""POLISH cap and dug-pocket contracts, including adversarial source textures."""
from __future__ import annotations

from dataclasses import replace
from pathlib import Path

import numpy as np
from PIL import Image

from art.compositor import edges, walls
from art.compositor.test_walls import textures


def test_default_cap_darkens_value_and_saturation_without_face_changes() -> None:
    # Given a flat cap and a face with distinguishable values.
    source = textures()
    # When rendering a horizontal run with the default style.
    tile = walls.wall_tiles(frozenset({(0, 0), (1, 0), (2, 0)}), source)[1]
    # Then cap HSV V=180*.45 and S=(40/180)*.40, while the face is untouched.
    assert tile.image.getpixel((32, 8)) == (81, 77, 74, 255)
    assert tile.image.getpixel((32, 30)) == (80, 60, 40, 255)


def test_cap_outline_is_two_pixels_with_eighty_percent_ink() -> None:
    # Given an isolated wall and a flat cap.
    # When default dark rendering draws its four exposed edges.
    tile = walls.wall_tiles(frozenset({(0, 0)}), textures())[0]
    # Then north/west ink is composited once, not twice at the corner.
    assert tile.image.getpixel((32, 0)) == (32, 30, 28, 255)
    assert tile.image.getpixel((32, 1)) == (32, 30, 28, 255)
    assert tile.image.getpixel((32, 2)) == (81, 77, 74, 255)
    assert tile.image.getpixel((0, 0)) == (32, 30, 28, 255)


def test_highlight_is_one_pixel_only_where_south_face_hangs() -> None:
    # Given a vertical pair with only its south cell exposing a face.
    # When drawing both tiles.
    north, south = walls.wall_tiles(frozenset({(0, 0), (0, 1)}), textures())
    # Then the face edge receives V+.08; a connector's last row receives no highlight.
    assert south.image.getpixel((32, 15)) == (101, 96, 93, 255)
    assert south.image.getpixel((32, 14)) == (32, 30, 28, 255)
    assert north.image.getpixel((32, 63)) == (81, 77, 74, 255)


def test_connected_edges_have_no_internal_ink() -> None:
    # Given an interior tile with all four neighbours.
    cells = frozenset({(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)})
    # When its dark cap is drawn.
    middle = next(tile for tile in walls.wall_tiles(cells, textures()) if tile.cell == (1, 1))
    # Then neither an ink boundary nor an end/corner texture crosses the top.
    assert middle.image.getextrema() == ((81, 81), (77, 77), (74, 74), (255, 255))


def test_end_and_inner_corner_sources_never_cover_dark_caps() -> None:
    # Given bright magenta end/corner textures in an L-shaped run.
    source = textures()
    source = replace(source, end=Image.new("RGBA", (24, 60), "magenta"),
                     inner_corner=Image.new("RGBA", (16, 16), "magenta"))
    # When the L is rendered.
    tiles = walls.wall_tiles(frozenset({(0, 0), (0, 1), (1, 1)}), source)
    # Then magenta can occur only below a face-bearing cap.
    for tile in tiles:
        pixels = np.asarray(tile.image)
        magenta = (pixels[:, :, :3] == (255, 0, 255)).all(axis=2)
        assert not magenta[:16].any()
        if not tile.face_visible:
            assert not magenta.any()
    assert any((np.asarray(tile.image)[16:, :, :3] == (255, 0, 255)).all(axis=2).any()
               for tile in tiles if tile.face_visible)


def test_legacy_cap_style_preserves_original_strip_and_ends() -> None:
    # Given the frozen source strip.
    # When requesting the explicit STRUCT comparison path.
    tile = walls.wall_tiles(frozenset({(0, 0), (1, 0), (2, 0)}), textures(), cap_style="struct")[1]
    # Then old cap colour is unchanged.
    assert tile.image.getpixel((32, 8)) == (180, 160, 140, 255)


def test_box_blur_is_nine_pixels_not_nine_pixel_radius() -> None:
    # Given one white column in a black strip.
    source = textures()
    cap = Image.new("RGBA", (256, 16), (0, 0, 0, 255))
    cap.paste((255, 255, 255, 255), (32, 0, 33, 16))
    # When rendering with the declared default box blur.
    tile = walls.wall_tiles(frozenset({(0, 0)}), replace(source, cap=cap))[0]
    # Then exactly nine interior columns receive the impulse.
    nonzero = np.flatnonzero(np.asarray(tile.image)[8, 4:60, 0]) + 4
    assert nonzero.tolist() == list(range(28, 37))


def test_material_caps_share_texture_value_and_saturation() -> None:
    # Given actual log and stone materials.
    from art.compositor.struct_layers import load_wall_textures

    cells = frozenset({(0, 0), (1, 0), (2, 0)})
    # When their dark caps are rendered.
    log = walls.wall_tiles(cells, load_wall_textures("log"))[1].image.crop((2, 2, 62, 14)).convert("HSV")
    stone = walls.wall_tiles(cells, load_wall_textures("stone"))[1].image.crop((2, 2, 62, 14)).convert("HSV")
    # Then only hue changes; channel rounding permits at most two 8-bit S steps.
    left, right = np.asarray(log), np.asarray(stone)
    assert np.array_equal(left[:, :, 2], right[:, :, 2])
    assert np.abs(left[:, :, 1].astype(int) - right[:, :, 1].astype(int)).max() <= 2
    assert np.median(left[:, :, 0]) != np.median(right[:, :, 0])


def test_dug_floor_blends_twelve_pixels_only_against_ground() -> None:
    # Given hill north of a dug tile, with meadow to its east.
    field = edges.EdgeField(
        {(0, 0): "dug", (0, -1): "hill", (1, 0): "meadow"},
        {"dug": Image.new("RGBA", (64, 64), (80, 80, 80, 255)),
         "hill": Image.new("RGBA", (64, 64), (240, 240, 240, 255)),
         "meadow": Image.new("RGBA", (64, 64), (20, 100, 20, 255))},
        hills=frozenset({(0, -1)}),
    )
    # When rendering the pocket floor.
    pixels = np.asarray(edges.render_tile(field, (0, 0)))
    # Then the entire north edge stays rock except for the east ground blend.
    assert np.all(pixels[:, :52] == (80, 80, 80, 255))
    assert np.any(pixels[:, 52:, 1] > 80)


def test_dug_pocket_has_one_north_face_and_no_dug_tile_overlay() -> None:
    # Given a two-tile-wide pocket below unexcavated rock.
    hills = frozenset({(0, 0), (1, 0), (-1, 1), (2, 1)})
    dug = frozenset({(0, 1), (1, 1), (0, 2), (1, 2)})
    # When generating all hill-face overlays.
    tiles = walls.hill_tiles(hills, dug, textures())
    # Then only undug north cells draw, and no dug cell doubles the transition.
    assert {tile.cell for tile in tiles} == {(0, 0), (1, 0)}
    assert not any(tile.cell in dug for tile in tiles)


def test_dug_floor_wins_even_when_hill_marker_is_later() -> None:
    # Given a synthetic overlay with the hill marker after its dug floor.
    from art.compositor.models import Overlay, OverlaySprite, Scene
    from art.compositor.struct_layers import build_layout
    from art.compositor.struct_render import struct_catalog

    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    overlay = Overlay.read(Path("art/compositor/scenes/m5_overlay.json")).model_copy(update={
        "structures": (), "people": (), "sprites": (
            OverlaySprite(id="floor", asset="floor_dug_rock", offset=(0, 0), category="floor"),
            OverlaySprite(id="old_hill", asset="dug_face", offset=(0, 0)),
        ),
    })
    # When converting the overlay into topology.
    layout = build_layout(scene, overlay, struct_catalog())
    # Then excavation remains passable independent of input order.
    assert scene.camp not in layout.hills
    assert (scene.camp, "floor_dug_rock") in layout.floors


def test_face_end_is_drawn_beside_a_connected_cap_only_neighbour() -> None:
    # Given a horizontal face run meeting a south-connected cap at a T.
    source = replace(textures(), end=Image.new("RGBA", (24, 60), "magenta"))
    cells = frozenset({(0, 0), (1, 0), (2, 0), (1, 1)})
    # When generating the exposed west face immediately beside the T cap.
    tile = next(tile for tile in walls.wall_tiles(cells, source) if tile.cell == (0, 0))
    # Then the face terminates with its end piece while the cap stays uncontaminated.
    assert tile.image.getpixel((60, 30)) == (255, 0, 255, 255)
    assert tile.image.getpixel((60, 8)) == (81, 77, 74, 255)


def test_parameters_preserve_declared_owner_choices() -> None:
    # Given the committed parameter handoff rather than copied test defaults.
    from art.compositor.wall_style import WallParameters

    # When parsing the exact configuration used by the compositor.
    params = WallParameters.read()
    # Then all fixed quantities and the selectable brightness set remain explicit.
    assert params.cap_style == "dark"
    assert (params.box_blur_px, params.value_multiplier, params.saturation_multiplier) == (9, .45, .40)
    assert (params.ink_hex, params.ink_alpha, params.ink_width_px) == ("#141210", .8, 2)
    assert (params.south_highlight_width_px, params.south_highlight_value_add) == (1, .08)
    assert params.dug_floor_blend_px == 12
    assert params.owner_value_choices == (.35, .45, .60)
