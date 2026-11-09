# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "pytest"]
# ///
# How to run: dependency-complete uv command from README.md, targeting test_rooms.py.
"""Room composition checks against real existing texture inputs."""

from pathlib import Path

import pytest
from PIL import Image

from art.batch_a2.rooms import export_room
from art.batch_a2.rooms_schema import PropSpec, RecipeError, RoomSpec
from art.compositor.geometry import perimeter


def test_wall_layers_when_room_has_south_door(tmp_path: Path) -> None:
    # Given a three-by-three room with a south-facing entrance.
    spec = RoomSpec(
        id="test_room",
        era="neolithic",
        footprint=(3, 3),
        wall_style="log",
        material="oak",
        cells=tuple(sorted(perimeter((3, 3)))),
        door_cell=(1, 2),
    )
    # When exporting actual material layers.
    result = export_room(spec, tmp_path)
    # Then all standing layers keep exact grid bases and a sixty-pixel exposed wall.
    walls = [layer for layer in result.layers if layer.kind == "wall"]
    assert len(walls) == 8
    assert all(layer.anchor == (32, 59) for layer in walls)
    assert all(layer.world_base == (layer.cell[0] * 64 + 32, layer.cell[1] * 64 + 43) for layer in walls)
    assert all(layer.size[1] == 60 for layer in walls if layer.cell[1] == 2)
    assert result.exposed_wall_height == 60
    assert "art/compositor/legacy_walls.py" in {source.path for source in result.inputs}


def test_missing_prop_when_source_is_pending(tmp_path: Path) -> None:
    # Given a named missing semantic content asset.
    spec = RoomSpec(
        id="pending",
        era="bronze",
        footprint=(3, 3),
        wall_style="mud_brick",
        material="clay",
        cells=tuple(sorted(perimeter((3, 3)))),
        props=(PropSpec(source="art/style05/struct/a2/contents/absent.png", cell=(1, 1), role="grain"),),
    )
    # When exporting the shell.
    result = export_room(spec, tmp_path)
    # Then the shell is explicitly incomplete and never claims adoption.
    assert result.status == "DERIVED_INCOMPLETE"
    assert result.unresolved == ("art/style05/struct/a2/contents/absent.png",)


def test_floor_tiles_when_room_exceeds_texture_period(tmp_path: Path) -> None:
    # Given a five-tile-wide room whose floor wraps the existing four-tile source.
    spec = RoomSpec(
        id="floor",
        era="iron",
        footprint=(5, 4),
        wall_style="drystone",
        material="granite",
        cells=tuple(sorted(perimeter((5, 4)))),
    )
    # When exporting phase-locked floor layers.
    result = export_room(spec, tmp_path)
    # Then both ends repeat the same texture pixels without stretching.
    floors = {layer.cell: layer for layer in result.layers if layer.kind == "floor"}
    with (
        Image.open(tmp_path / floors[(0, 0)].path) as first,
        Image.open(tmp_path / floors[(4, 0)].path) as wrapped,
    ):
        assert first.size == wrapped.size == (64, 64)
        assert first.tobytes() == wrapped.tobytes()


def test_prop_pixels_when_existing_asset_is_placed(tmp_path: Path) -> None:
    # Given an existing small hearth with its published anchor.
    source = "art/style05/struct/hearth_unlit.png"
    spec = RoomSpec(
        id="hearth",
        era="neolithic",
        footprint=(3, 3),
        wall_style="log",
        material="oak",
        cells=tuple(sorted(perimeter((3, 3)))),
        props=(PropSpec(source=source, cell=(1, 1), anchor=(21, 23), role="home hearth"),),
    )
    # When composing the hearth into its own layer.
    result = export_room(spec, tmp_path)
    # Then the sprite is unchanged and its origin maps its anchor to the world base.
    layer = next(layer for layer in result.layers if layer.kind == "prop")
    with Image.open(source) as original, Image.open(tmp_path / layer.path) as exported:
        assert original.convert("RGBA").tobytes() == exported.convert("RGBA").tobytes()
    assert layer.world_origin == (75, 84)


def test_floor_ends_when_south_wall_base_is_reached(tmp_path: Path) -> None:
    # Given a roofless ring whose exposed wall ends at y171.
    spec = RoomSpec(
        id="bounded",
        era="neolithic",
        footprint=(3, 3),
        wall_style="log",
        material="oak",
        cells=tuple(sorted(perimeter((3, 3)))),
    )
    # When exporting its top-down floor.
    result = export_room(spec, tmp_path)
    # Then no floor pixel forms an apron below the wall's base row.
    assert max(row.world_origin[1] + row.size[1] for row in result.layers if row.kind == "floor") == 172


def test_unknown_material_style_when_recipe_names_it(tmp_path: Path) -> None:
    # Given a typo in an explicit material style.
    spec = RoomSpec(
        id="typo",
        era="bronze",
        footprint=(1, 1),
        wall_style="mud_brick_typo",
        material="clay",
        cells=((0, 0),),
        floor=False,
    )
    # When exporting the recipe.
    # Then the material resolver cannot silently substitute its generic fallback.
    with pytest.raises(RecipeError, match="unknown wall style"):
        _ = export_room(spec, tmp_path)
