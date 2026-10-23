# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python3 -m art.compositor.native_layers.
"""Validated packed bridge columns used by the exact-tick supplemental layer."""
from __future__ import annotations

from typing import Literal

from art.compositor.models import InputModel


class NativeClock(InputModel):
    tick: int


class NativePerson(InputModel):
    agent_id: int
    tile: tuple[int, int]
    activity_kind: int
    asleep: bool
    carry: int
    carry_kind: int
    job: int


class NativeBuilding(InputModel):
    kind: int
    tile: tuple[int, int]


class NativeDump(InputModel):
    people: tuple[NativePerson, ...]
    buildings: tuple[NativeBuilding, ...]


class NativeInventory(InputModel):
    item_counts: tuple[int, ...]
    item_kinds: tuple[int, ...]
    item_xs: tuple[int, ...]
    item_ys: tuple[int, ...]
    site_kinds: tuple[int, ...]
    site_xs: tuple[int, ...]
    site_ys: tuple[int, ...]
    zone_ids: tuple[int, ...]
    zone_xs: tuple[int, ...]
    zone_ys: tuple[int, ...]


class NativeSpace(InputModel):
    wall_kinds: tuple[int, ...]
    wall_xs: tuple[int, ...]
    wall_ys: tuple[int, ...]
    floor_kinds: tuple[int, ...]
    floor_xs: tuple[int, ...]
    floor_ys: tuple[int, ...]
    bed_xs: tuple[int, ...]
    bed_ys: tuple[int, ...]
    ford_xs: tuple[int, ...]
    ford_ys: tuple[int, ...]


class NativeAnimals(InputModel):
    ids: tuple[int, ...]
    kinds: tuple[int, ...]
    states: tuple[int, ...]
    xs: tuple[int, ...]
    ys: tuple[int, ...]


class NativeCapture(InputModel):
    scene_id: str
    camp: tuple[int, int]
    clock: NativeClock
    dump: NativeDump
    overlay: NativeInventory
    space: NativeSpace
    animals: NativeAnimals


class NativeSprite(InputModel):
    id: str
    asset: str
    position: tuple[int, int]
    category: Literal["floor", "object", "animal"] = "object"
    site: bool = False
    zone: bool = False
    native_kind: int | None = None
    native_count: int | None = None


class NativeLayer(InputModel):
    scene_id: str
    tick: int
    people: tuple[NativePerson, ...]
    sprites: tuple[NativeSprite, ...]


class NativeLayers(InputModel):
    source_path: str
    source_sha256: str
    source_commit: str
    derivation: str
    scenes: tuple[NativeLayer, ...]
