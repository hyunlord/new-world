# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "numpy"]
# ///
# How to run: imported by python -m art.compositor.struct_render.
"""Translate unchanged ENV1 observations into explicit STRUCT geometry."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from PIL import Image

from art.compositor.assets import REPO, Catalog, read_image
from art.compositor.geometry import perimeter
from art.compositor.models import Overlay, Scene
from art.compositor.native_layers import bridge_sprites, verified_layer
from art.compositor.placements import Sprite, overlay_sprites, real_sprites
from art.compositor.wall_style import CapStyle, WallParameters, material_hue
from art.compositor.walls import Door, WallTextures, WallTile, hill_tiles, wall_tiles

Cell = tuple[int, int]


@dataclass(frozen=True, slots=True)
class Room:
    id: str
    cells: frozenset[Cell]
    material: Literal["log", "stone"]
    door: Cell | None = None
    open_door: bool = True
    site: bool = False


@dataclass(frozen=True, slots=True)
class Layout:
    sprites: tuple[Sprite, ...]
    floors: tuple[tuple[Cell, str], ...]
    rooms: tuple[Room, ...]
    hills: frozenset[Cell]
    fords: frozenset[Cell]
    zones: tuple[Sprite, ...]
    water: frozenset[Cell]

    @property
    def wall_cells(self) -> frozenset[Cell]:
        return frozenset(cell for room in self.rooms for cell in room.cells)


@dataclass(frozen=True, slots=True)
class StructureTile:
    tile: WallTile
    source: str
    site: bool = False

    @property
    def position(self) -> tuple[float, float]:
        return self.tile.cell


def build_layout(scene: Scene, overlay: Overlay, catalog: Catalog) -> Layout:
    """Keep scene/overlay coordinates; replace roofs and generated wall cases."""
    native = verified_layer(scene)
    original = tuple(row for row in real_sprites(scene) if row.asset in {"oak_large", "stump", "hut"})
    all_rows = original + bridge_sprites(native, catalog) + overlay_sprites(scene, overlay)
    rooms: list[Room] = []
    floors: list[tuple[Cell, str]] = []
    sprites: list[Sprite] = []
    zones: list[Sprite] = []
    hills = {(x, y) for y, row in enumerate(scene.terrain_rows) for x, glyph in enumerate(row) if glyph in "#^o"}
    fords: set[Cell] = set()
    for structure in overlay.structures:
        ox, oy = scene.camp[0] + structure.offset[0], scene.camp[1] + structure.offset[1]
        cells = frozenset((ox + x, oy + y) for x, y in perimeter(structure.size))
        rooms.append(Room(structure.id, cells, structure.material,
                          (ox + structure.size[0] // 2, oy + structure.size[1] - 1)))
    for row in all_rows:
        cell = (round(row.position[0]), round(row.position[1]))
        if row.zone:
            zones.append(row)
        elif row.asset == "hut":
            ox, oy = cell[0] - 1, cell[1] - 1
            cells = frozenset((ox + x, oy + y) for x, y in perimeter((3, 3)))
            rooms.append(Room(row.id, cells, "log", (ox + 1, oy + 2), site=row.site))
            if not row.site:
                floors.extend(((ox + x, oy + y), "floor_packed_earth") for y in range(3) for x in range(3))
        elif row.asset.startswith(("wall_", "door_")):
            if not row.declared:
                material: Literal["log", "stone"] = "stone" if "stone" in row.asset else "log"
                rooms.append(Room(row.id, frozenset({cell}), material,
                                  cell if row.asset.startswith("door_") else None, False, row.site))
        elif row.asset in {"dug_face", "hill_edge"}:
            hills.add(cell)
        elif row.asset == "ford":
            fords.add(cell)
        elif row.category == "floor" and not row.site:
            floors.append((cell, row.asset))
            hills.discard(cell)
        else:
            sprites.append(row)
    hills.difference_update(cell for cell, _ in floors)
    water = {(x, y) for y, row in enumerate(scene.terrain_rows) for x, glyph in enumerate(row) if glyph in "~_"}
    water.difference_update(cell for cell, _ in floors)
    water.update(fords)
    return Layout(tuple(sprites), tuple(floors), tuple(rooms), frozenset(hills), frozenset(fords), tuple(zones), frozenset(water))


def load_wall_textures(material: str) -> WallTextures:
    root = REPO / "art/style05/struct"
    params = WallParameters.read()
    palette_root = Path(__file__).resolve().parents[2]
    return WallTextures(*(read_image(root / f"{material}_{part}.png") for part in ("face", "cap", "end", "inner_corner")),
                        dark_cap=read_image(REPO / params.shared_cap_source),
                        cap_hue=material_hue(palette_root, material, params))


def structural_tiles(layout: Layout, catalog: Catalog, dimensions: tuple[int, int], *,
                     cap_style: CapStyle = "dark", params: WallParameters | None = None) -> tuple[StructureTile, ...]:
    """Assemble wall connectivity globally within each material/site group."""
    result: list[StructureTile] = []
    for material in ("log", "stone"):
        textures = load_wall_textures(material)
        rooms = tuple(room for room in layout.rooms if room.material == material)
        cells = frozenset(cell for room in rooms for cell in room.cells)
        for site in (False, True):
            selected = frozenset(cell for room in rooms if room.site == site for cell in room.cells)
            doors = tuple(Door(room.door, wall_door(catalog, f"door_{material}_{'open' if room.open_door else 'closed'}"))
                          for room in rooms if room.door is not None)
            result.extend(StructureTile(tile, f"{material}_wall", site) for tile in wall_tiles(cells, textures, doors, cap_style=cap_style, params=params) if tile.cell in selected)
    rock = catalog.image("hill_rock").resize((256, 64), Image.Resampling.LANCZOS)
    rock_textures = WallTextures(read_image(REPO / "art/style05/struct/rock_face.png"),
                                rock.crop((0, 0, 256, 16)), rock.resize((24, 60)), rock.crop((0, 0, 16, 16)))
    blocked = layout.hills | layout.water | layout.wall_cells
    passable = frozenset((x, y) for y in range(dimensions[1]) for x in range(dimensions[0]) if (x, y) not in blocked)
    result.extend(StructureTile(tile, "hill_face") for tile in hill_tiles(layout.hills, passable, rock_textures))
    return tuple(result)


def wall_door(catalog: Catalog, asset_id: str) -> Image.Image:
    """Place declared door anchor into the fixed 64x60 wall sprite window."""
    source = catalog.image(asset_id)
    anchor = catalog.asset(asset_id).anchor or (source.width / 2, source.height - 1)
    image = Image.new("RGBA", (64, 60))
    image.alpha_composite(source, (32 - round(anchor[0]), 59 - round(anchor[1])))
    return image


def bake_rooms(catalog: Catalog) -> None:
    """Bake the exact code wall routine into three FPS chunks with provenance."""
    import hashlib
    import json

    from art.compositor.walls import render_walls

    destination = REPO / "art/proto/struct/rooms"
    destination.mkdir(parents=True, exist_ok=True)
    rows: list[dict[str, str | list[int]]] = []
    inputs: dict[str, str] = {}
    layouts = (("house", (7, 5), "log", (30, 100)), ("storehouse", (5, 4), "stone", (550, 100)), ("hut", (3, 3), "log", (1100, 100)))
    for name, size, material, position in layouts:
        cells = perimeter(size)
        door_id = f"door_{material}_open"
        baked = render_walls(cells, load_wall_textures(material), (Door((size[0] // 2, size[1] - 1), wall_door(catalog, door_id)),))
        path = destination / f"{name}.png"
        baked.image.save(path, optimize=True)
        rows.append({"path": str(path.relative_to(REPO)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "position": list(position)})
        input_paths = [REPO / "art/compositor/walls.py", Path(__file__), REPO / catalog.asset(door_id).path]
        input_paths.extend(REPO / f"art/style05/struct/{material}_{part}.png" for part in ("face", "cap", "end", "inner_corner"))
        for source in input_paths:
            inputs[str(source.relative_to(REPO))] = hashlib.sha256(source.read_bytes()).hexdigest()
    _ = (destination.parent / "rooms.json").write_text(json.dumps({"rooms": rows, "bake_inputs": inputs,
        "layouts": [{"name": name, "size": list(size), "material": material} for name, size, material, _ in layouts]}, indent=2) + "\n")
