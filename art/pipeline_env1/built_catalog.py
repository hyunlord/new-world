# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run --with pillow --with numpy --with scikit-image python -m art.pipeline_env1.built_process
"""Frozen ENV1 built-asset export dimensions and source classifications."""
from dataclasses import dataclass
from typing import Final


@dataclass(frozen=True, slots=True)
class Spec:
    """An authored asset's simulation footprint and measurement family."""

    name: str
    family: str
    footprint: tuple[int, int]
    full_bleed: bool = False


SPECS: Final = (
    Spec("wall_log_straight", "buildings", (4, 1)),
    Spec("wall_log_corner", "buildings", (2, 2)),
    Spec("wall_stone_straight", "buildings", (4, 1)),
    Spec("wall_stone_corner", "buildings", (2, 2)),
    Spec("door_log_closed", "buildings", (1, 1)),
    Spec("door_log_open", "buildings", (1, 1)),
    Spec("door_stone_closed", "buildings", (1, 1)),
    Spec("door_stone_open", "buildings", (1, 1)),
    Spec("floor_packed_earth", "ground", (1, 1), True),
    Spec("floor_dug_rock", "ground", (1, 1), True),
    Spec("bedroll", "items", (1, 2)),
    Spec("drying_rack_empty", "buildings", (2, 1)),
    Spec("drying_rack_hide", "buildings", (2, 1)),
    Spec("pottery_bench", "buildings", (2, 1)),
    Spec("grinding_stone", "buildings", (1, 1)),
    Spec("hearth_pots", "items", (1, 1)),
    Spec("hearth_unlit", "items", (1, 1)),
    Spec("claybank_full", "ground", (2, 2)),
    Spec("claybank_depleted", "ground", (2, 2)),
    Spec("ford", "ground", (1, 1), True),
    Spec("hill_edge", "ground", (1, 1), True),
    Spec("dug_face", "ground", (1, 1), True),
    Spec("snow_overlay", "ground", (1, 1), True),
)
