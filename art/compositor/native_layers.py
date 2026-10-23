# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "typer"]
# ///
# How to run: python3 -m art.compositor.native_layers.
"""Normalize verified bridge captures without changing native simulation state."""
from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Final

import typer
from pydantic import TypeAdapter

from art.compositor.assets import REPO, Catalog
from art.compositor.geometry import wall_mask
from art.compositor.models import Scene
from art.compositor.native_models import (
    NativeCapture,
    NativeLayer,
    NativeLayers,
    NativeSprite,
)
from art.compositor.placements import Sprite

SOURCE: Final = Path("/Users/rexxa/work/artenv1-data/baseline/baseline_capture.json")
DESTINATION: Final = REPO / "art/compositor/scenes/native_layers.json"
ITEMS: Final = ("log_pile_small", "firewood", "stone_pile", "flint_nodule", "flint_blade", "stone_axe", "hide",
                "clay", "raw_pot", "clay_pot_empty", "ground_axe", "meat", "cured_hide", "hide_wrap", "fur_cloak", "spear", "wolf_hide")
SITES: Final = ("hut", "knapping_spot", "woodworking_bench", "drying_rack_empty", "wall_log_00", "wall_stone_00",
               "door_log_closed", "floor_packed_earth", "bedroll", "grinding_stone", "pottery_bench", "hearth_pots")


def normalize(capture: NativeCapture) -> NativeLayer:
    result: list[NativeSprite] = []
    inventory, space = capture.overlay, capture.space
    walls = frozenset(zip(space.wall_xs, space.wall_ys, strict=True))
    planned = frozenset((x, y) for x, y, kind in zip(inventory.site_xs, inventory.site_ys, inventory.site_kinds, strict=True) if kind in (4, 5, 6))
    for index, (x, y, kind) in enumerate(zip(space.wall_xs, space.wall_ys, space.wall_kinds, strict=True)):
        asset = "door_log_closed" if kind == 2 else f"wall_{'log' if kind == 0 else 'stone'}_{wall_mask((x, y), walls | planned):02d}"
        result.append(NativeSprite(id=f"bridge_wall_{index}", asset=asset, position=(x, y), native_kind=kind))
    for index, (x, y, kind) in enumerate(zip(inventory.site_xs, inventory.site_ys, inventory.site_kinds, strict=True)):
        asset = SITES[kind]
        if kind in (4, 5):
            asset = f"wall_{'log' if kind == 4 else 'stone'}_{wall_mask((x, y), walls | planned):02d}"
        result.append(NativeSprite(id=f"bridge_site_{index}", asset=asset, position=(x, y), site=True, native_kind=kind))
    for index, (x, y, kind, count) in enumerate(zip(inventory.item_xs, inventory.item_ys, inventory.item_kinds, inventory.item_counts, strict=True)):
        asset = ITEMS[kind]
        if kind == 0:
            asset = "log_pile_small" if count == 1 else "log_pile_medium" if count <= 8 else "log_pile_large"
        result.append(NativeSprite(id=f"bridge_item_{index}", asset=asset, position=(x, y), native_kind=kind, native_count=count))
    for index, (x, y, kind) in enumerate(zip(space.floor_xs, space.floor_ys, space.floor_kinds, strict=True)):
        result.append(NativeSprite(id=f"bridge_floor_{index}", asset="floor_packed_earth" if kind == 0 else "floor_dug_rock",
                                   position=(x, y), category="floor", native_kind=kind))
    for prefix, asset, xs, ys in (("bed", "bedroll", space.bed_xs, space.bed_ys),
                                 ("ford", "ford", space.ford_xs, space.ford_ys)):
        for index, (x, y) in enumerate(zip(xs, ys, strict=True)):
            result.append(NativeSprite(id=f"bridge_{prefix}_{index}", asset=asset, position=(x, y), category="floor" if prefix == "ford" else "object"))
    for index, (x, y, zone_id) in enumerate(zip(inventory.zone_xs, inventory.zone_ys, inventory.zone_ids, strict=True)):
        result.append(NativeSprite(id=f"bridge_zone_{index}", asset="floor_packed_earth", position=(x, y), zone=True, native_kind=zone_id))
    animals = capture.animals
    for animal_id, kind, state, x, y in zip(animals.ids, animals.kinds, animals.states, animals.xs, animals.ys, strict=True):
        species = "wolf" if kind == 0 else "deer"
        pose = "run" if state in (1, 5) else "attack" if state == 4 and kind == 0 else "walk" if state == 3 else "stand"
        result.append(NativeSprite(id=f"bridge_animal_{animal_id}", asset=f"{species}_S_{pose}", position=(x, y), category="animal", native_kind=kind))
    for index, building in enumerate(capture.dump.buildings):
        if building.kind in (0, 1):
            result.append(NativeSprite(id=f"bridge_building_{index}", asset="hearth_1" if building.kind == 0 else "lean_to", position=building.tile, native_kind=building.kind))
    return NativeLayer(scene_id=capture.scene_id, tick=capture.clock.tick, people=capture.dump.people, sprites=tuple(result))


def verified_layer(scene: Scene, path: Path = DESTINATION) -> NativeLayer:
    layers = NativeLayers.model_validate_json(path.read_text())
    layer = next(row for row in layers.scenes if row.scene_id == scene.scene_id.removeprefix("scene_"))
    if layer.tick != scene.tick:
        raise ValueError("Supplemental bridge tick differs from native source scene")
    native = {person.agent_id: person.tile for person in layer.people}
    if native != {person.id: (person.x, person.y) for person in scene.people}:
        raise ValueError("Supplemental bridge IDs/positions differ from native source scene")
    return layer


def bridge_sprites(layer: NativeLayer, catalog: Catalog) -> tuple[Sprite, ...]:
    return tuple(Sprite(row.id, catalog.stack_asset(row.asset, row.native_count), row.position, False, row.category, row.site,
                        source="verified_bridge", zone=row.zone, native_kind=row.native_kind, native_count=row.native_count)
                 for row in layer.sprites)


def main(source: Path = SOURCE, destination: Path = DESTINATION) -> None:
    captures = TypeAdapter(dict[str, NativeCapture]).validate_json(source.read_bytes())
    selected = [normalize(capture) for key, capture in sorted(captures.items()) if key.endswith("_1x.png")]
    result = NativeLayers(source_path=str(source), source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                          source_commit="3a3b236c125c0c25e9b74261be88536473fc01e9",
                          derivation="Exact frozen-save read-only bridge capture. Native codes checked in sim-core colony/items.rs, orders.rs, wildlife.rs and sim-bridge ffi/space.rs. Resource-node and extra routine animation layers remain outside this normalized subset.", scenes=tuple(selected))
    _ = destination.write_text(result.model_dump_json(indent=2) + "\n")
    for layer in selected:
        print(f"NATIVE {layer.scene_id} tick={layer.tick} sprites={len(layer.sprites)} people={len(layer.people)}")


if __name__ == "__main__":
    typer.run(main)
