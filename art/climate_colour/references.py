# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy", "pillow", "pydantic"]
# ///
# How to run: imported by python -m art.climate_colour.export
"""Resolve shipped colour controls with explicit nearest-habit herb proxies."""

from pathlib import Path
from typing import Final

from PIL import Image

from art.climate_colour.models import Row
from art.flora.schema import SeasonTint
from art.flora.seasons import apply_tint

ROOT: Final = Path(__file__).resolve().parents[2]
GROUND_REFERENCES: Final = {
    "tundra": "hill_rock",
    "taiga_floor": "forest_floor",
    "steppe": "meadow",
    "temperate_meadow": "meadow",
    "mediterranean_scrub": "dark_grass",
    "marsh": "dark_grass",
    "river_gravel": "river_bank",
    "highland_rock": "hill_rock",
}


def read(path: Path) -> Image.Image:
    """Own image file resources and return detached RGBA pixels."""
    with Image.open(path) as image:
        return image.convert("RGBA")


def reference(row: Row) -> tuple[Image.Image, tuple[str, ...], str]:
    """Choose semantic controls, never substitute a summer tree for autumn."""
    if row.lane == "trees":
        species = "oak" if row.species == "wild_olive" else row.species
        variant = "v2" if "_v2" in row.id else "v1"
        path = f"art/flora/sprites/{species}_{row.size}_{variant}.png"
        mask = path.replace("/sprites/", "/masks/")
        image = read(ROOT / path)
        tint = SeasonTint.model_validate(row.tint) if row.tint else SeasonTint()
        if row.species == "wild_olive":
            delta = {
                "spring": (6, 0, 6),
                "summer": (0, 0, 0),
                "autumn": (0, 0, 0),
                "winter": (-8, 0, 0),
            }
            tint = SeasonTint(delta_lab=delta[row.season or "summer"])
        image = apply_tint(image, read(ROOT / mask), tint)
        return (
            image,
            (path, mask),
            "same_species_size_variant_season"
            if species == row.species
            else "olive_canopy_palette_control_oak",
        )
    dry = any(
        word in row.id
        for word in ("barley", "grain", "einkorn", "millet", "rye", "reeds")
    )
    path = (
        "art/style05/env/plants/reeds.png"
        if dry
        else "art/flora/sprites/berry_bush_bare.png"
    )
    if row.lane == "shrubs" and row.id.startswith("berry_"):
        path = (
            "art/flora/sprites/berry_bush_full.png"
            if row.state == "full"
            else "art/flora/sprites/berry_bush_bare.png"
        )
    if row.lane == "herbaceous" and row.state == "winter":
        path = "art/flora/sprites/oak_small_bare.png"
    image = read(ROOT / path)
    if row.state == "winter" and row.lane == "shrubs" and "/sprites/" in path:
        mask = read(ROOT / path.replace("/sprites/", "/masks/"))
        image = apply_tint(image, mask, SeasonTint(delta_lab=(-8, 0, 0)))
    exact = row.id.startswith(("berry_", "reeds_"))
    return (
        image,
        (path,),
        "same_family" if exact else "nearest_habit_proxy_no_shipped_crop_family",
    )
