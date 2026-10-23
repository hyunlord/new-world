# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pydantic", "pillow", "numpy"]
# ///
# How to run: python3 -m pytest art/compositor/test_renderer.py -q.
"""Boundary regressions for real-scene identity and map geometry."""
from pathlib import Path

import pytest
from PIL import Image, ImageDraw
from pydantic import ValidationError

from art.compositor.assets import Catalog, MissingAssetError
from art.compositor.drawing import ForegroundPainter, activity_glyph
from art.compositor.geometry import Viewport, perimeter, wall_mask
from art.compositor.models import Activity, Overlay, Scene
from art.compositor.native_layers import verified_layer
from art.compositor.placements import RenderPerson, overlay_sprites, render_people


def test_scene_rejects_wrong_dimensions() -> None:
    # Given a real frozen snapshot with one missing terrain row.
    source = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    broken = source.model_dump()
    broken["terrain_rows"] = source.terrain_rows[:-1]
    # When parsing it, then absent terrain cannot silently become grass.
    with pytest.raises(ValidationError, match="dimensions"):
        _ = Scene.model_validate(broken)


def test_scene_rejects_duplicate_identities() -> None:
    # Given a real person repeated in the input.
    source = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    broken = source.model_dump()
    broken["people"] = (source.people[0], source.people[0])
    # When parsing it, then a double-drawn identity is rejected.
    with pytest.raises(ValidationError, match="duplicate"):
        _ = Scene.model_validate(broken)


def test_world_projection_keeps_camp_below_hud() -> None:
    # Given the required half-size viewport.
    viewport = Viewport((91, 101), .5)
    # When projecting camp and a one-tile offset, then positions use 32px.
    assert viewport.project((91, 101)) == (800, 506)
    assert viewport.project((92, 101)) == (832, 506)
    assert viewport.visible((91, 101))
    assert not viewport.visible((91, 70))


def test_wall_masks_preserve_rectangular_corners() -> None:
    # Given the required 7x5 walled house perimeter.
    walls = perimeter((7, 5))
    # When resolving corner/straight masks, then no mirrored case is invented.
    assert len(walls) == 20
    assert wall_mask((0, 0), walls) == 6
    assert wall_mask((6, 0), walls) == 12
    assert wall_mask((3, 0), walls) == 10
    assert wall_mask((0, 2), walls) == 5


def test_missing_art_fails_without_placeholder() -> None:
    catalog = Catalog({})
    with pytest.raises(MissingAssetError, match="missing_wolf"):
        _ = catalog.image("missing_wolf")


def test_half_marker_is_exactly_ten_pixels_including_ring() -> None:
    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    image = Image.new("RGBA", (1600, 900))
    actor = RenderPerson(scene.people[0], scene.camp, scene.people[0].activity)
    painter = ForegroundPainter(image, Viewport(scene.camp, .5), Catalog({}), True)
    _ = painter.person(actor)
    bbox = image.getbbox()
    assert bbox is not None
    assert (bbox[2] - bbox[0], bbox[3] - bbox[1]) == (10, 10)


@pytest.mark.parametrize(("activity", "extent"), ((Activity.CHOP, (14, 14)), (Activity.CARRY, (14, 8))))
def test_repair_glyphs_keep_registered_extents(activity: Activity, extent: tuple[int, int]) -> None:
    image = Image.new("RGBA", (32, 32))
    activity_glyph(ImageDraw.Draw(image), activity, (8, 8))
    bbox = image.getbbox()
    assert bbox is not None
    assert (bbox[2] - bbox[0], bbox[3] - bbox[1]) == extent


def test_colocated_native_sleepers_receive_distinct_display_positions() -> None:
    scene = Scene.read(Path("art/compositor/scenes/scene_day6_night.json"))
    overlay = Overlay.read(Path("art/compositor/scenes/m5_overlay.json"))
    actors = render_people(scene, overlay)
    sleepers = [row for row in actors if not row.declared]
    assert len(sleepers) == 3
    assert len({row.position for row in sleepers}) == 3
    assert len({(row.person.x, row.person.y) for row in sleepers}) == 2


def test_native_items_preserve_kind_instead_of_text_log_proxy() -> None:
    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    layer = verified_layer(scene)
    items = [row for row in layer.sprites if row.id.startswith("bridge_item_")]
    assert len(items) == 14
    assert {row.asset for row in items} == {"firewood", "stone_pile", "flint_nodule"}
    assert sum(row.native_count or 0 for row in items) == 162


def test_native_join_rejects_different_tick() -> None:
    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    with pytest.raises(ValueError, match="tick"):
        _ = verified_layer(scene.model_copy(update={"tick": scene.tick + 1}))


def test_declared_layout_avoids_native_objects_and_river_actor_positions() -> None:
    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    overlay = Overlay.read(Path("art/compositor/scenes/m5_overlay.json"))
    native = verified_layer(scene)
    occupied = {row.position for row in native.sprites if not row.zone and row.category != "animal"}
    assert not ({row.position for row in overlay_sprites(scene, overlay)} & occupied)
    for actor in render_people(scene, overlay):
        if actor.declared:
            assert scene.terrain_rows[int(actor.position[1])][int(actor.position[0])] != "~"
