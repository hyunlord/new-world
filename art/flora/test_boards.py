# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic", "numpy", "typer"]
# ///
# How to run: python3 -m pytest art/flora/test_boards.py -q
"""Bounded board contracts independent of generation timing."""
import importlib.util
from pathlib import Path


def test_board_modules_exist() -> None:
    # Given the required offline board surface, when located, then both exist.
    assert importlib.util.find_spec("art.flora.boards") is not None
    assert importlib.util.find_spec("art.flora.scene_board") is not None


def test_owner_schedule_covers_every_native_variant_twice() -> None:
    # Given the production catalog, when scheduled, then all seasons and scales exist.
    from art.flora.boards import owner_schedule
    from art.flora.resolve_flora import read_catalog
    rows = owner_schedule(read_catalog(Path.cwd()))
    assert len(rows) == 96
    assert len({(r.species_id, r.size, r.variant, r.season, r.scale) for r in rows}) == 96


def test_sheet_crops_are_exact_source_panels() -> None:
    # Given the frozen sheet, when selecting comparison crops, then no caption enters.
    from art.flora.boards import REFERENCE_CROPS
    assert REFERENCE_CROPS == ((976, 8, 1936, 548), (976, 3068, 1936, 3608))


def test_dense_mass_keeps_isolated_trees_as_silhouettes() -> None:
    # Given one dense triplet and an isolated tree, when grouped, then only centre is mass.
    from art.compositor.placements import Sprite
    from art.flora.resolve_flora import read_catalog
    from art.flora.scene_board import dense_cells, tree_selection
    catalog = read_catalog(Path.cwd())
    trees = tuple(tree_selection(Sprite(str(x), "oak_large", (x, 5), False), catalog, "summer")
                  for x in (1, 2, 3, 9))
    assert dense_cells(trees) == frozenset({(2, 5)})


def test_tree_assignment_retains_source_coordinates_across_seasons() -> None:
    # Given a native tree, when seasons change, then only its resolved seasonal art changes.
    from art.compositor.placements import Sprite
    from art.flora.resolve_flora import read_catalog
    from art.flora.scene_board import tree_selection
    catalog = read_catalog(Path.cwd())
    row = Sprite("observed", "oak_large", (91, 101), False)
    summer = tree_selection(row, catalog, "summer")
    winter = tree_selection(row, catalog, "winter")
    assert summer.position == winter.position == row.position
    assert summer.resolved.species_id == winter.resolved.species_id


def test_fixture_catalog_adds_data_without_mutating_production() -> None:
    # Given an arbitrary valid species fixture, when merged, then production stays unchanged.
    from art.flora.boards import fixture_catalog
    from art.flora.resolve_flora import read_catalog
    catalog = read_catalog(Path.cwd())
    fixture = Path("art/flora/tests/test_alder.json")
    expanded = fixture_catalog(catalog, fixture, "test_alder")
    assert "test_alder" not in catalog.species.species
    assert expanded.species.species["test_alder"].seasons["autumn"].delta_lab == (2, 24, 16)


def test_quarter_canopy_uses_seasons_and_omits_bare_mass() -> None:
    # Given a dense deciduous patch, when seasons change, then the mass changes/vanishes.
    from PIL import Image

    from art.compositor.geometry import Viewport
    from art.compositor.placements import Sprite
    from art.flora.resolve_flora import read_catalog, resolve_flora
    from art.flora.scene_board import TreePlacement, paint_mass
    from art.flora.schema import Season
    catalog = read_catalog(Path.cwd())
    seasons: tuple[Season, ...] = ("summer", "autumn", "winter")
    outputs: list[bytes] = []
    for season in seasons:
        frame = resolve_flora("oak", "large", season, 0, catalog=catalog)
        trees = tuple(TreePlacement(Sprite(str(x), "oak_large", (x, 101), False), frame) for x in (90, 91, 92))
        image = Image.new("RGBA", (1600, 900))
        mass = paint_mass(image, trees, Viewport((91, 101), .25))
        outputs.append(image.tobytes())
        assert bool(mass) == (season != "winter")
    assert len(set(outputs)) == 3


def test_evergreen_autumn_mass_is_identical_to_summer() -> None:
    # Given evergreen season data, when composited, then autumn keeps the summer pixels.
    from art.flora.resolve_flora import read_catalog, resolve_flora
    from art.flora.scene_board import seasonal_mass
    catalog = read_catalog(Path.cwd())
    summer = resolve_flora("pine", "large", "summer", 0, catalog=catalog)
    autumn = resolve_flora("pine", "large", "autumn", 0, catalog=catalog)
    assert seasonal_mass(summer.model_dump_json()).tobytes() == seasonal_mass(autumn.model_dump_json()).tobytes()


def test_transparent_strip_panels_keep_board_background() -> None:
    # Given a transparent sprite panel, when pasted, then its void stays the board colour.
    from PIL import Image

    from art.flora.boards import panel_board
    source = Image.new("RGBA", (20, 20), (0, 0, 0, 0))
    result = panel_board((("preview", source),), 1)
    assert result.getpixel((20, 45)) == (31, 35, 28)
