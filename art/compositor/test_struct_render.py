# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "pydantic", "typer"]
# ///
# How to run: uv run --with pytest --with pillow --with numpy --with pydantic --with typer python -m pytest art/compositor/test_struct_render.py
"""STRUCT integration contracts exercised against frozen ENV1 inputs."""
from pathlib import Path

import pytest

from art.compositor.assets import load_catalog
from art.compositor.models import Overlay, Scene
from art.compositor.struct_layers import build_layout


def test_real_hut_becomes_ring_without_roof_sprite() -> None:
    # Given the authentic noon scene and unmodified declared ENV1 overlay.
    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    rows = list(scene.glyph_rows)
    rows[101] = rows[101][:91] + "H" + rows[101][92:]
    scene = scene.model_copy(update={"glyph_rows": tuple(rows)})
    overlay = Overlay.read(Path("art/compositor/scenes/m5_overlay.json"))
    catalog = load_catalog((Path("docs/v8/artenv1/built_assets.json").resolve(), Path("docs/v8/artenv1/items_manifest.json").resolve()))
    # When structural geometry is assembled.
    layout = build_layout(scene, overlay, catalog)
    # Then real huts have eight perimeter cells and a door, never a roof sprite.
    huts = [room for room in layout.rooms if room.id.startswith("real_hut_")]
    assert huts
    assert all(len(room.cells) == 8 and room.door in room.cells for room in huts)
    assert not any(sprite.asset == "hut" for sprite in layout.sprites)


def test_declared_floor_and_wall_positions_retain_env1_layout() -> None:
    # Given frozen ENV1 placements.
    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    overlay = Overlay.read(Path("art/compositor/scenes/m5_overlay.json"))
    catalog = load_catalog((Path("docs/v8/artenv1/built_assets.json").resolve(), Path("docs/v8/artenv1/items_manifest.json").resolve()))
    # When translating the same source into code walls.
    layout = build_layout(scene, overlay, catalog)
    # Then the house/store keep their exact footprint and shared doorway cell.
    house = next(room for room in layout.rooms if room.id == "declared_house")
    assert len(house.cells) == 20
    assert house.door == (82, 100)
    assert ((81, 98), "floor_packed_earth") in layout.floors
    assert layout.fords.issuperset({(88, 99), (89, 99)})
    assert not any(sprite.asset in {"dug_face", "hill_edge", "ford"} for sprite in layout.sprites)


def test_frozen_scene_composes_with_code_geometry(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given test-only textures, real ENV1 snapshots and an authentic HUD.
    from PIL import Image

    from art.compositor import struct_ground, struct_layers
    from art.compositor.render import RenderInputs, RenderSettings
    from art.compositor.struct_render import compose
    from art.compositor.tone import Condition

    root = tmp_path / "art/style05/struct"
    root.mkdir(parents=True)
    for material in ("log", "stone"):
        for part, size in (("face", (256, 44)), ("cap", (256, 16)), ("end", (24, 60)), ("inner_corner", (16, 16))):
            Image.new("RGBA", size, (110, 95, 75, 255)).save(root / f"{material}_{part}.png")
    Image.new("RGBA", (256, 44), (95, 95, 90, 255)).save(root / "rock_face.png")
    Image.new("RGBA", (256, 256), (45, 75, 85, 255)).save(root / "water.png")
    for number in range(1, 5):
        Image.new("RGBA", (10, 6), (120, 125, 115, 255)).save(root / f"ford_stone_{number}.png")
    monkeypatch.setattr(struct_ground, "REPO", tmp_path)
    monkeypatch.setattr(struct_layers, "REPO", tmp_path)
    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    overlay = Overlay.read(Path("art/compositor/scenes/m5_overlay.json"))
    catalog = load_catalog((Path("docs/v8/artenv1/built_assets.json").resolve(), Path("docs/v8/artenv1/items_manifest.json").resolve()))
    hud = Path("docs/v8/artenv1/reference/current_day6_noon_1x_hud.png").resolve()
    # When the complete composition runs with real source identities.
    result = compose(RenderInputs(scene, overlay, catalog, hud), RenderSettings(1, Condition.DAY, True))
    # Then HUD bytes, identities, and code wall evidence survive integration.
    with Image.open(hud) as original:
        assert result.image.crop((0, 0, 1600, 112)).tobytes() == original.convert("RGB").tobytes()
    assert result.evidence["source_population"] == len(scene.people)
    assert result.evidence["overlay_sha256"]
    assert result.evidence["boundary_metrics"]
    assert result.evidence["roof"] == "not drawn"


def test_owner_board_keeps_source_pixels_at_declared_scale(tmp_path: Path) -> None:
    # Given distinctive source pixels and the same crop in two panels.
    from PIL import Image

    from art.compositor.struct_boards import Panel, make_board

    source = Image.new("RGB", (12, 12))
    source.putpixel((4, 5), (17, 101, 209))
    path = tmp_path / "source.png"
    source.save(path)
    destination = tmp_path / "board.png"
    # When assembling native and doubled panels.
    make_board(destination, (Panel("1x", path, (2, 3, 8, 9)), Panel("2x", path, (2, 3, 8, 9), 2)))
    # Then 1x is byte-exact and 2x is exact replication, never a scaled overview.
    with Image.open(destination) as board:
        assert board.crop((0, 38, 6, 44)).tobytes() == source.crop((2, 3, 8, 9)).tobytes()
        assert board.getpixel((10, 42)) == (17, 101, 209)
        assert board.getpixel((11, 43)) == (17, 101, 209)


@pytest.mark.parametrize(("north_offset", "partly_hidden"), ((.5, True), (1.0, False)))
def test_actual_route_i_figure_occlusion_is_limited_to_wall_geometry(north_offset: float, partly_hidden: bool) -> None:
    # Given the real adopted figure and a 60px wall immediately south.
    from PIL import Image

    from art.compositor.assets import Catalog
    from art.compositor.drawing import ForegroundPainter
    from art.compositor.geometry import Viewport
    from art.compositor.models import Activity
    from art.compositor.placements import RenderPerson
    from art.compositor.struct_layers import StructureTile
    from art.compositor.struct_render import paint_structure
    from art.compositor.walls import WallTextures, wall_tiles

    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    viewport = Viewport(scene.camp, 1)
    image, coverage = Image.new("RGBA", (1600, 900)), Image.new("L", (1600, 900))
    painter = ForegroundPainter(image, viewport, Catalog({}), False, coverage)
    actor = RenderPerson(scene.people[0], (scene.camp[0], scene.camp[1] - north_offset), Activity.AWAKE)
    record = painter.person(actor, 1)
    assert record is not None
    before = coverage.histogram()[1]
    textures = WallTextures(Image.new("RGBA", (256, 44), (100, 90, 80, 255)), Image.new("RGBA", (256, 16), (140, 130, 120, 255)),
                            Image.new("RGBA", (24, 60)), Image.new("RGBA", (16, 16)))
    tile = wall_tiles(frozenset({scene.camp}), textures)[0]
    # When the wall is drawn later by the same production helper.
    _ = paint_structure(painter, StructureTile(tile, "test_wall"))
    # Then near-north feet are hidden but the head survives; the next row is untouched.
    after = coverage.histogram()[1]
    if partly_hidden:
        assert 0 < after < before
    else:
        assert after == before
    px, py = viewport.project(scene.camp)
    assert image.getchannel("A").getpixel((px, py + 11)) == 255
    assert image.getchannel("A").getpixel((px, py + 12)) == 0
