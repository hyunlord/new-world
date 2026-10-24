# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: python3 -m pytest art/flora/test_resolve_flora.py -q
"""Pure flora selection; only read_catalog performs file I/O."""
from pathlib import Path
from typing import assert_never

from art.flora.schema import (
    DefaultsDocument,
    FloraCatalog,
    Habit,
    Record,
    Season,
    SeasonTint,
    Size,
    SpeciesDocument,
)


class ResolvedFlora(Record):
    requested_id: str
    species_id: str
    habit: Habit
    fallback_used: bool
    path: str
    canopy_mask: str
    tint: SeasonTint
    anchor: tuple[int, int]
    canvas: tuple[int, int]
    map_canopy: str
    map_canopy_path: str
    log_material: str | None


def read_catalog(root: Path) -> FloraCatalog:
    """Load and parse the two documents once, outside the pure resolver."""
    directory = root / "art/flora"
    return FloraCatalog(
        species=SpeciesDocument.model_validate_json((directory / "species.json").read_bytes()),
        defaults=DefaultsDocument.model_validate_json((directory / "habit_defaults.json").read_bytes()),
    )


def resolve_flora(species_id: str, size: Size, season: Season, variant_hash: int, *,
                  catalog: FloraCatalog, habit: Habit | None = None) -> ResolvedFlora:
    """Select frame and season without I/O, RNG, mutation or species literals.

    Four explicit selector arguments are the owner contract; catalog is injected.
    An unknown id cannot reveal its habit: callers supply simulation metadata,
    otherwise the document's unknown_habit applies. Unknown species use habit
    seasons, never the fallback species' overrides. Python modulo makes signed
    hashes stable as well. The resolved material is the fallback's material;
    requested_id is retained so the caller can report the missing art entry.
    """
    known = catalog.species.species.get(species_id)
    selected_habit = known.habit if known else habit or catalog.defaults.unknown_habit
    default = catalog.defaults.habits[selected_habit]
    selected = known or catalog.species.species[default.fallback_species]
    tint = selected.seasons[season] if known else default.seasons[season]
    frames = selected.sizes[size]
    match tint.frame:
        case "foliage":
            variants = frames.variants
        case "bare":
            variants = frames.bare
        case unreachable:
            assert_never(unreachable)
    frame = variants[variant_hash % len(variants)]
    return ResolvedFlora(
        requested_id=species_id, species_id=selected.id, habit=selected_habit,
        fallback_used=known is None, path=frame.path, canopy_mask=frame.canopy_mask,
        tint=tint, anchor=frame.anchor, canvas=frame.canvas,
        map_canopy=selected.map_canopy,
        map_canopy_path=catalog.species.map_canopies[selected.map_canopy],
        log_material=selected.log_material,
    )
