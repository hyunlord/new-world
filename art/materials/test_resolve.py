# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pydantic"]
# ///
# How to run: python3 -m pytest art/materials/test_resolve.py -q.
"""Behavioral fallback and strict schema boundary tests."""
from pathlib import Path

import pytest
from pydantic import ValidationError

from art.materials.resolve import read_catalog, resolve
from art.materials.schema import StylesDocument

ROOT = Path(__file__).resolve().parents[2]


def test_known_material_overrides_category_tint() -> None:
    # Given the committed data and a known material.
    catalog = read_catalog(ROOT)
    # When resolving a style/material pair.
    result = resolve(catalog, "log", "birch")
    # Then the exact named material tint wins.
    assert result.style_id == "log"
    assert result.tint.dL == 12
    assert result.tint_source == "material"


def test_unknown_material_uses_supplied_category_and_unknown_blueprint() -> None:
    # Given a material the art has never seen, but a category the sim knows.
    catalog = read_catalog(ROOT)
    # When both material and blueprint are unfamiliar.
    result = resolve(catalog, "FutureWall", "unlisted_material", category="Stone")
    # Then category defaults choose the style and tint.
    assert result.style_id == "drystone"
    assert result.tint_source == "category"
    assert result.category == "Stone"


def test_unknown_category_uses_style_default_without_mutation() -> None:
    # Given a known style and completely foreign material metadata.
    catalog = read_catalog(ROOT)
    original = catalog.model_dump_json()
    # When resolving repeatedly.
    first = resolve(catalog, "log", "foreign", category="FutureCategory")
    second = resolve(catalog, "log", "foreign", category="FutureCategory")
    # Then the style's zero shift is stable and input data remains unchanged.
    assert first == second
    assert first.tint_source == "style"
    assert first.tint.dL == first.tint.da == first.tint.db == 0
    assert original == catalog.model_dump_json()


def test_blueprint_resolves_through_data() -> None:
    # Given declared simulation blueprint mapping.
    catalog = read_catalog(ROOT)
    # When resolving the blueprint instead of an art style id.
    result = resolve(catalog, "StoneWall", "sandstone")
    # Then no caller translation is necessary.
    assert result.style_id == "drystone"
    assert result.tint.db == 12


def test_kind_defaults_never_return_wall_for_floor() -> None:
    # Given an unknown floor blueprint.
    catalog = read_catalog(ROOT)
    # When resolving a floor in a known category.
    result = resolve(catalog, "unknown_floor", "granite", kind="floor")
    # Then floor textures are selected by data.
    assert result.kind == "floor"
    assert result.textures.floor is not None


def test_schema_rejects_unknown_fields_and_incomplete_wall_set() -> None:
    # Given a document that looks plausible but lacks required wall textures.
    malformed = '{"schema_version":1,"default_styles":{"wall":"x","floor":"x","door":"x"},"styles":[{"id":"x","kind":"wall","era":"test","textures":{},"prompt_template":"wall","sim_blueprints":[]}]}'
    # When parsing the public schema boundary, then missing assets fail.
    with pytest.raises(ValidationError):
        _ = StylesDocument.model_validate_json(malformed)


@pytest.mark.parametrize("category", ["Stone", "Wood", "Animal", "Mineral", "Plant", "Mod"])
def test_every_category_renders_an_unknown_material(category: str) -> None:
    # Given each simulation category and an unlisted material id.
    catalog = read_catalog(ROOT)
    fallback = next(row for row in catalog.defaults.categories if row.category == category)
    # When resolving an unknown blueprint/material pair.
    result = resolve(catalog, "future_blueprint", "unlisted_material", category=category)
    # Then the declared category default is used without new code.
    assert result.style_id == fallback.default_style
    assert result.tint == fallback.tint
    assert result.tint_source == "category"


def test_partial_texture_override_keeps_every_other_piece() -> None:
    # Given a material overriding only one source in a complete texture set.
    from art.materials.schema import TextureSet
    catalog = read_catalog(ROOT)
    original = next(row for row in catalog.materials.materials if row.id == "oak")
    altered = original.model_copy(update={"texture_override":{"log":TextureSet(face="art/materials/custom/face.png")}})
    entries = tuple(altered if row.id == original.id else row for row in catalog.materials.materials)
    catalog = catalog.model_copy(update={"materials":catalog.materials.model_copy(update={"materials":entries})})
    # When resolving the material override.
    result = resolve(catalog, "log", "oak")
    # Then only the selected face changes, not the independent directional tops.
    assert result.textures.face == "art/materials/custom/face.png"
    assert result.textures.top_ew == "art/materials/textures/log/top_ew.png"
    assert result.textures.top_ns == "art/materials/textures/log/top_ns.png"
    assert result.textures.door_open == "art/materials/textures/log/door_open.png"


def test_committed_json_schemas_match_parser_contract() -> None:
    # Given the machine-readable schemas shipped to the game implementer.
    import json

    from art.materials.schema import CategoriesDocument, MaterialsDocument
    # When comparing each schema with its actual parser contract.
    pairs = (("construction_styles",StylesDocument),("material_looks",MaterialsDocument),("category_defaults",CategoriesDocument))
    # Then a stale or hand-weakened schema cannot silently pass.
    for name, model in pairs:
        assert json.loads((ROOT / f"art/materials/{name}.schema.json").read_text()) == model.model_json_schema()


def test_resolver_and_assembly_contain_no_named_style_or_material_literal() -> None:
    # Given every construction/material id from the data, including future additions.
    import ast
    catalog = read_catalog(ROOT)
    forbidden = {row.id for row in catalog.construction.styles} | {row.id for row in catalog.materials.materials}
    # When inspecting all string literals in the pure resolver and new assembly.
    for relative in ("art/materials/resolve.py","art/materials/render.py","art/compositor/walls.py"):
        tree = ast.parse((ROOT / relative).read_text())
        literals = {node.value for node in ast.walk(tree) if isinstance(node,ast.Constant) and isinstance(node.value,str)}
        # Then style/material-specific branching cannot hide in the implementation.
        assert not forbidden & literals


def test_raw_simulation_id_resolves_without_display_name_guessing() -> None:
    # Given djb2("oak") = 193501184, computed from the pinned Rust hash contract.
    catalog = read_catalog(ROOT)
    # When the compositor receives the raw u32 from TileGrid.wall_material.
    result = resolve(catalog,"LogWall",193501184)
    # Then it reaches the same material entry as the canonical source key.
    assert result.material_key == "oak"
    assert result.category == "Wood"
    assert result.tint == resolve(catalog,"LogWall","oak").tint
