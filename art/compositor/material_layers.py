# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "numpy"]
# ///
# How to run: imported by python3 -m art.compositor.polish2_boards.
"""Data-driven material bindings for the frozen standalone art scene."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from art.compositor.assets import REPO
from art.compositor.struct_layers import Layout, StructureTile
from art.compositor.walls import Door, wall_tiles
from art.materials.render import load_textures
from art.materials.resolve import ResolvedMaterial, resolve
from art.materials.schema import MaterialCatalog, Record, Top


class MaterialChoice(Record):
    style_id: str
    material_id: str
    label: str


class OwnerMode(Record):
    id: str
    label: str
    renderer: Literal["legacy", "material"]
    top: Top | None = None


class SceneMaterials(Record):
    schema_version: Literal[1]
    room_keys: dict[str, MaterialChoice]
    samples: tuple[MaterialChoice, ...]
    owner_modes: tuple[OwnerMode, ...]

    @classmethod
    def read(cls) -> SceneMaterials:
        return cls.model_validate_json(Path(__file__).with_name("polish2_scene_materials.json").read_bytes())


@dataclass(frozen=True, slots=True)
class MaterialScene:
    catalog: MaterialCatalog
    choices: SceneMaterials
    top: Top | None = None

    def resolve(self, choice: MaterialChoice) -> ResolvedMaterial:
        selected = resolve(self.catalog, choice.style_id, choice.material_id)
        return selected.model_copy(update={"top": self.top}) if self.top is not None else selected

    def input_paths(self) -> tuple[Path, ...]:
        """Bind rendered material parameters and generated source pixels."""
        paths = {"art/materials/construction_styles.json", "art/materials/material_looks.json",
                 "art/materials/category_defaults.json", "art/compositor/polish2_scene_materials.json"}
        for choice in self.choices.room_keys.values():
            textures = self.resolve(choice).textures
            paths.update(path for path in (textures.face, textures.top_ew, textures.top_ns,
                                           textures.end, textures.junction, textures.door_closed,
                                           textures.door_open) if path is not None)
        return tuple(REPO / path for path in sorted(paths))


def material_tiles(layout: Layout, material_scene: MaterialScene) -> tuple[StructureTile, ...]:
    """Keep connectivity and site masks while looking up every room key in data."""
    result: list[StructureTile] = []
    occupied = layout.wall_cells
    openings = frozenset(room.door for room in layout.rooms if room.door is not None)
    for key in sorted({room.material for room in layout.rooms}):
        choice = material_scene.choices.room_keys[key]
        resolved = material_scene.resolve(choice)
        textures = load_textures(REPO, resolved)
        rooms = tuple(room for room in layout.rooms if room.material == key)
        cells = frozenset(cell for room in rooms for cell in room.cells)
        doors = tuple(Door(room.door, textures.door_open if room.open_door else textures.door_closed)
                      for room in rooms if room.door is not None)
        tiles = wall_tiles(cells, textures, doors, occupancy=occupied, openings=openings)
        for site in (False, True):
            selected = frozenset(cell for room in rooms if room.site == site for cell in room.cells)
            result.extend(StructureTile(tile, f"{resolved.style_id}/{resolved.material_id}", site)
                          for tile in tiles if tile.cell in selected)
    return tuple(result)
