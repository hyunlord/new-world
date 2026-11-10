# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "pytest"]
# ///
# How to run: uv run --with pillow --with numpy --with pydantic --with pytest python -m pytest art/batch_b3/rooms
"""Check semantic stage differences and actual exported layer geometry."""

from pathlib import Path

import pytest
from PIL import Image

from art.batch_b3.rooms.render import render
from art.batch_b3.rooms.schema import RecipeError, Recipes

ROOT = Path(__file__).resolve().parents[3]


def test_damage_removes_a_wall_but_preserves_door() -> None:
    # Given all authored construction states, including the3x3 doorway regression.
    rows = Recipes.model_validate_json(
        (ROOT / "art/batch_b3/rooms/recipes.json").read_bytes()
    ).rooms
    lookup = {row.id: row for row in rows}
    # When comparing each damaged state with its built counterpart.
    for damaged in (row for row in rows if row.state == "damaged"):
        intact = lookup[f"{damaged.family}_built"].courses[0]
        damaged_course = damaged.courses[0]
        # Then exactly one explicit wall is missing and the doorway remains valid.
        assert len(damaged_course.cells) == len(intact.cells) - 1
        assert damaged_course.door_cell == intact.door_cell


def test_foundation_actual_export_keeps_declared_anchor(tmp_path: Path) -> None:
    # Given the real mud-brick recipe and adopted material textures.
    rows = Recipes.model_validate_json(
        (ROOT / "art/batch_b3/rooms/recipes.json").read_bytes()
    ).rooms
    room = next(row for row in rows if row.id == "mud_brick_house_foundation")
    # When rendering through the existing compositor and the new course adapter.
    manifest = render(room, tmp_path)
    # Then every facade still ends on its declared tile base; the preview is real RGBA.
    for layer in (layer for layer in manifest.layers if layer.kind == "wall"):
        assert layer.origin[1] + layer.anchor[1] == layer.cell[1] * 64 + 43
    with Image.open(tmp_path / room.id / "preview.png") as image:
        assert image.mode == "RGBA"
        assert image.getbbox() is not None


def test_missing_prop_fails_instead_of_exporting_incomplete_room(
    tmp_path: Path,
) -> None:
    # Given an otherwise valid room whose referenced furniture is absent.
    rows = Recipes.model_validate_json(
        (ROOT / "art/batch_b3/rooms/recipes.json").read_bytes()
    ).rooms
    original = next(row for row in rows if row.id == "mud_brick_house_built")
    missing = original.props[0].model_copy(update={"source": "missing_b3_prop.png"})
    room = original.model_copy(update={"props": (missing,)})
    # When the actual renderer resolves the sources.
    # Then it rejects missing data rather than silently omitting furniture.
    with pytest.raises(RecipeError, match="missing or escaping prop"):
        _ = render(room, tmp_path)
