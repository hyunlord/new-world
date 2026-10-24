# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pytest", "jsonschema"]
# ///
# How to run: python3 -m pytest art/flora/test_resolve_flora.py -q
"""Selection, fallback, purity, schema and simulation-id contract tests."""
from __future__ import annotations

import json
from pathlib import Path
from unittest.mock import patch

import jsonschema
import pytest
from pydantic import ValidationError

from art.flora.resolve_flora import read_catalog, resolve_flora
from art.flora.schema import FloraCatalog, Habit, Species, SpeciesDocument

ROOT = Path(__file__).resolve().parents[2]


@pytest.fixture
def catalog() -> FloraCatalog:
    return read_catalog(ROOT)


def test_signed_hash_selects_variant(catalog: FloraCatalog) -> None:
    # Given a species with two variants; when a negative hash is selected.
    result = resolve_flora("oak", "large", "summer", -1, catalog=catalog)
    # Then modulo selection chooses the second frame reproducibly.
    assert result.path.endswith("oak_large_v2.png")
    assert result.tint.delta_lab == (0, 0, 0)


def test_deciduous_winter_swaps_to_bare(catalog: FloraCatalog) -> None:
    # Given a summer variant hash; when winter is requested.
    result = resolve_flora("birch", "small", "winter", 7, catalog=catalog)
    # Then the single bare frame, with unchanged color, is selected.
    assert result.path.endswith("birch_small_bare.png")
    assert result.tint.delta_lab == (0, 0, 0)


def test_evergreen_winter_retains_foliage(catalog: FloraCatalog) -> None:
    # Given evergreen data; when winter is requested.
    result = resolve_flora("pine", "large", "winter", 0, catalog=catalog)
    # Then darkening and bounded code snow are returned on the summer frame.
    assert result.path.endswith("pine_large_v1.png")
    assert result.tint.delta_lab == (-8, 0, 0)
    assert result.tint.snow_cap.alpha == 0.7


@pytest.mark.parametrize("habit", ["deciduous", "evergreen", "shrub"])
def test_unknown_id_uses_habit_defaults(catalog: FloraCatalog, habit: Habit) -> None:
    # Given a missing id with known habit; when resolving winter.
    result = resolve_flora("missing_species", "small", "winter", 0, catalog=catalog, habit=habit)
    # Then both the selected fallback and its habit tint follow data.
    assert result.species_id == catalog.defaults.habits[habit].fallback_species
    assert result.tint == catalog.defaults.habits[habit].seasons["winter"]
    assert result.fallback_used


def test_unknown_habit_policy_is_data(catalog: FloraCatalog) -> None:
    # Given an absent species with no supplied habit; when resolving.
    result = resolve_flora("missing_species", "small", "summer", 0, catalog=catalog)
    # Then the data document decides the fallback habit.
    assert result.habit == catalog.defaults.unknown_habit


def test_resolution_performs_no_io_or_mutation(catalog: FloraCatalog) -> None:
    # Given a loaded catalog; when file operations are forbidden during resolution.
    before = catalog.model_dump_json()
    with patch("builtins.open", side_effect=AssertionError("I/O in resolver")), patch.object(
        Path, "open", side_effect=AssertionError("I/O in resolver")
    ):
        result = resolve_flora("oak", "small", "spring", 12, catalog=catalog)
    # Then the selection works and input data remains unchanged.
    assert result.tint.delta_lab == (6, 0, 6)
    assert catalog.model_dump_json() == before


def test_data_only_species_selects_every_season(catalog: FloraCatalog) -> None:
    # Given a test-only data entry reusing an existing texture family.
    addition = Species.model_validate_json((ROOT / "art/flora/tests/test_alder.json").read_bytes())
    document = SpeciesDocument.model_validate({
        **catalog.species.model_dump(),
        "species": {**catalog.species.species, addition.id: addition},
    })
    expanded = FloraCatalog(species=document, defaults=catalog.defaults)
    # When all four seasons are selected without touching resolver code.
    results = [resolve_flora(addition.id, "large", season, 0, catalog=expanded)
               for season in addition.seasons]
    # Then each uses the additional species' own season record and reused frames.
    assert all(result.species_id == addition.id and not result.fallback_used for result in results)
    assert [result.tint for result in results] == list(addition.seasons.values())
    assert results[2].tint != catalog.species.species["oak"].seasons["autumn"]


def test_resolver_contains_no_species_names(catalog: FloraCatalog) -> None:
    # Given current data keys; when reading the algorithm source.
    source = (ROOT / "art/flora/resolve_flora.py").read_text()
    # Then none of the species names is embedded in code or documentation.
    assert all(species_id not in source for species_id in catalog.species.species)
    assert "test_alder" not in source


def test_tree_ids_match_simulation_material_names(catalog: FloraCatalog) -> None:
    # Given production tree records; when matching the simulation data files.
    trees = [row for row in catalog.species.species.values() if row.habit != "shrub"]
    # Then material keys are literal simulation ids, not art-only aliases.
    for row in trees:
        material = ROOT / "rust/crates/sim-core/data/material/wood" / f"{row.id}.ron"
        assert material.exists()
        assert f'"{row.id}"' in material.read_text()
        assert row.log_material == row.id


@pytest.mark.parametrize("stem", ["species", "habit_defaults"])
def test_documents_validate_against_json_schema(stem: str) -> None:
    # Given the shipped schema and document; when validating with Draft 2020-12.
    schema = json.loads((ROOT / f"art/flora/{stem}.schema.json").read_text())
    document = json.loads((ROOT / f"art/flora/{stem}.json").read_text())
    jsonschema.Draft202012Validator(schema).validate(document)
    # Then no validation error is raised.


def test_incorrect_tree_material_rejected(catalog: FloraCatalog) -> None:
    # Given a tree record whose log material diverges from its id.
    raw = {**catalog.species.species["oak"].model_dump(), "log_material": "wrong"}
    # When parsed; then its cross-field contract is rejected.
    with pytest.raises(ValidationError, match="log_material equals"):
        Species.model_validate(raw)


def test_missing_bare_frame_rejected(catalog: FloraCatalog) -> None:
    # Given a deciduous record with a missing winter size.
    raw = catalog.species.species["oak"].model_dump()
    raw["sizes"]["small"]["bare"] = []
    # When parsed; then resolution cannot silently pick a summer tree in winter.
    with pytest.raises(ValidationError, match="require bare"):
        Species.model_validate(raw)


def test_fallback_habit_mismatch_rejected(catalog: FloraCatalog) -> None:
    # Given an evergreen default pointing to a deciduous species.
    raw = catalog.model_dump()
    raw['defaults']['habits']['evergreen']['fallback_species'] = 'oak'
    # When parsed; then evergreen fallback cannot silently use a deciduous frame.
    with pytest.raises(ValidationError, match='Fallback habit must match'):
        FloraCatalog.model_validate(raw)
