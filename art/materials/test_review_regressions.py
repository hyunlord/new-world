# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic"]
# ///
# How to run: python3 -m pytest art/materials/test_review_regressions.py -q.
"""Independent review regressions for real rendering and external schema consumers."""
import json
import re
from dataclasses import replace
from pathlib import Path

import pytest
from pydantic import JsonValue, TypeAdapter, ValidationError

from art.compositor import walls
from art.compositor.material_layers import MaterialScene, SceneMaterials, material_tiles
from art.compositor.struct_layers import Layout, Room
from art.compositor.test_material_walls import sources
from art.materials.resolve import read_catalog, resolve
from art.materials.schema import MaterialCatalog, StylesDocument, TextureSet

ROOT = Path(__file__).resolve().parents[2]


def style_kind_rules() -> list[JsonValue]:
    schema = TypeAdapter(dict[str,JsonValue]).validate_python(StylesDocument.model_json_schema())
    definitions = schema["$defs"]
    assert isinstance(definitions,dict)
    construction = definitions["ConstructionStyle"]
    assert isinstance(construction,dict)
    rules = construction["allOf"]
    assert isinstance(rules,list)
    return rules


@pytest.mark.parametrize("cells", [
    frozenset({(0,0),(1,0),(0,1)}),
    frozenset({(0,0),(-1,0),(1,0),(0,1),(0,-1)}),
])
def test_inset_silhouette_is_inked_without_seams_at_connected_arms(cells: frozenset[tuple[int,int]]) -> None:
    # Given a yellow 44px corner/cross with every relevant tile neighbour.
    source = sources()
    top = source.resolved.top.model_copy(update={"width_px":44,"ink_edge_px":2})
    source = replace(source,resolved=source.resolved.model_copy(update={"top":top}))
    # When drawing through the public wall renderer.
    tile = next(row for row in walls.wall_tiles(cells,source) if row.cell == (0,0))
    # Then inset right/lower edges are inked, yet connected arm boundary centres are untouched.
    assert tile.image.getpixel((53,30)) == (67,65,13,255)
    assert tile.image.getpixel((60,12)) == (67,65,13,255)
    assert tile.image.getpixel((54,30)) == tile.image.getpixel((60,13)) == (0,0,0,0)
    assert tile.image.getpixel((63,8)) == tile.image.getpixel((32,63)) == (255,255,0,255)


@pytest.mark.parametrize("path", ["", " ", " art/a.png", "art/a.png ", ".", "..", "art/",
    "../a.png", "/a.png", "art/../a.png", "art//a.png", "art/./a.png", "art/a.png/",
    "C:/a.png", "art\\a.png", "art/a\x00.png", "art/a\n.png"])
def test_invalid_partial_override_path_fails_before_image_io(path: str) -> None:
    # Given real catalog data with a malformed partial material override.
    catalog = read_catalog(ROOT)
    replacement = '"texture_override":{"log":{"face":' + json.dumps(path) + '}}'
    raw = catalog.model_dump_json().replace('"texture_override":{}', replacement, 1)
    # When parsing the public boundary, then it rejects the value before any loader can open it.
    with pytest.raises(ValidationError):
        _ = MaterialCatalog.model_validate_json(raw)


@pytest.mark.parametrize("kind,roles", [
    ("wall",("face","top_ew","top_ns","end","junction","door_closed","door_open")),
    ("floor",("floor",)), ("door",("door_closed","door_open")),
])
def test_exported_json_schema_requires_non_null_textures_by_kind(kind: str,roles: tuple[str,...]) -> None:
    # Given the schema emitted for external consumers, independently of parser validators.
    rules = style_kind_rules()
    # When inspecting the per-kind clause, then every required role excludes null.
    assert {"if":{"properties":{"kind":{"const":kind}},"required":["kind"]},
        "then":{"properties":{"textures":{"required":list(roles),
            "properties":{role:{"type":"string"} for role in roles}}}}} in rules


def test_exported_json_schema_requires_wall_end_and_corner_paths() -> None:
    # Given the schema consumed without Pydantic's wall-alias validator.
    rules = style_kind_rules()
    # When selecting a wall, then both alias paths must be present and non-null.
    assert {"if":{"properties":{"kind":{"const":"wall"}},"required":["kind"]},
        "then":{"required":["ends","corner"],
            "properties":{"ends":{"type":"string"},"corner":{"type":"string"}}}} in rules


@pytest.mark.parametrize("path", ["", " ", "../a.png", "/a.png", "art//a.png", "art/./a.png", "art/a.png/", "art/a\x00.png"])
def test_exported_json_schema_path_pattern_rejects_invalid_paths(path: str) -> None:
    # Given the lexical path clause emitted for every texture role.
    schema = TypeAdapter(dict[str,JsonValue]).validate_python(TextureSet.model_json_schema())
    properties = schema["properties"]
    assert isinstance(properties,dict)
    for role in ("face","top_ew","top_ns","end","junction","door_closed","door_open","floor"):
        property_schema = properties[role]
        assert isinstance(property_schema,dict)
        alternatives = property_schema["anyOf"]
        assert isinstance(alternatives,list)
        string_clause = next(clause for clause in alternatives if isinstance(clause,dict) and clause["type"] == "string")
        pattern = string_clause["pattern"]
        assert isinstance(pattern,str)
        # When testing malformed paths directly against the exported regex, then they fail.
        assert string_clause["minLength"] == 1
        assert re.search(pattern,path) is None
        assert re.search(pattern,"art/materials/asset.png") is not None


def test_mixed_material_rooms_share_global_wall_topology() -> None:
    # Given adjacent cells whose only difference is their material binding.
    scene = MaterialScene(read_catalog(ROOT),SceneMaterials.read())
    layout = Layout((),(),(Room("north",frozenset({(0,0)}),"log"),
        Room("south",frozenset({(0,1)}),"stone")),frozenset(),frozenset(),(),frozenset())
    # When the real material scene adapter assembles them.
    tiles = {row.tile.cell:row.tile for row in material_tiles(layout,scene)}
    # Then the north cell connects across the material boundary without drawing a south face.
    assert not tiles[(0,0)].face_visible
    assert tiles[(0,0)].image.height == 64
    assert tiles[(0,1)].face_visible


def test_packed_floor_blueprint_is_resolved_by_mapping_not_accidental_default() -> None:
    # Given the pinned simulation's actual BlueprintKind name.
    catalog = read_catalog(ROOT)
    # When selecting that floor blueprint.
    result = resolve(catalog,"PackedFloor","clay",kind="floor")
    # Then the declared mapping is used, independently of category default coincidence.
    assert result.style_source == "blueprint"


def test_texture_symlink_cannot_escape_repository_root(tmp_path: Path) -> None:
    # Given a lexically valid path whose symlink points outside the asset repository.
    from PIL import Image

    from art.materials.render import load_surface
    root = tmp_path / "repo"
    (root / "art").mkdir(parents=True)
    outside = tmp_path / "outside.png"
    Image.new("RGBA",(256,256),"red").save(outside)
    (root / "art/linked.png").symlink_to(outside)
    resolved = resolve(read_catalog(ROOT),None,"clay",kind="floor")
    resolved = resolved.model_copy(update={"textures":TextureSet(floor="art/linked.png")})
    # When loading the surface, then resolution stops before reading external image bytes.
    with pytest.raises(ValueError,match="repository root"):
        _ = load_surface(root,resolved)


def test_full_width_connected_cross_has_no_new_internal_ink() -> None:
    # Given the original full-width cross with all four sides connected.
    source = sources()
    top = source.resolved.top.model_copy(update={"width_px":64,"ink_edge_px":2})
    source = replace(source,resolved=source.resolved.model_copy(update={"top":top}))
    cells = frozenset({(0,0),(-1,0),(1,0),(0,-1),(0,1)})
    # When rendering the middle using silhouette ink.
    tile = next(row for row in walls.wall_tiles(cells,source) if row.cell == (0,0))
    # Then its source remains byte-identical because no top boundary is exposed.
    assert tile.image.tobytes() == source.junction.tobytes()


def test_mixed_material_horizontal_subset_has_no_internal_end_piece_or_ink() -> None:
    # Given a draw subset with east/west neighbours owned by other material batches.
    source = sources()
    top = source.resolved.top.model_copy(update={"ink_edge_px":2})
    source = replace(source,resolved=source.resolved.model_copy(update={"top":top}))
    occupied = frozenset({(-1,0),(0,0),(1,0)})
    # When drawing the selected tile against global occupancy.
    tile = walls.wall_tiles(frozenset({(0,0)}),source,occupancy=occupied)[0]
    # Then neither magenta end pieces nor side ink appears on connected material joins.
    assert tile.image.getpixel((0,30)) == tile.image.getpixel((63,30)) == (0,0,255,255)
    assert tile.image.getpixel((0,8)) == (0,0,0,255)
    assert tile.image.getpixel((63,8)) == (63,0,0,255)


def test_exported_json_schema_constrains_version_and_extra_keys() -> None:
    # Given the external schema without relying on the parser's version validation.
    schema = StylesDocument.model_json_schema()
    # When inspecting its boundary clauses, then version one and known keys are mandatory.
    assert schema["properties"]["schema_version"]["const"] == 1
    assert "schema_version" in schema["required"]
    assert schema["additionalProperties"] is False
