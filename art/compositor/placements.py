# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python3 -m art.compositor.render.
"""Separate real glyph observations from explicitly declared overlay placements."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from art.compositor.geometry import perimeter, wall_mask
from art.compositor.models import Activity, Overlay, Person, Scene


@dataclass(frozen=True, slots=True)
class Sprite:
    id: str
    asset: str
    position: tuple[float, float]
    declared: bool
    category: Literal["floor", "object", "animal"] = "object"
    site: bool = False
    source: str = "native_text"
    zone: bool = False
    native_kind: int | None = None
    native_count: int | None = None


@dataclass(frozen=True, slots=True)
class RenderPerson:
    person: Person
    position: tuple[float, float]
    activity: Activity
    top: str | None = None
    declared: bool = False
    mark: str = ""


def overlay_sprites(scene: Scene, overlay: Overlay) -> tuple[Sprite, ...]:
    result: list[Sprite] = []
    cx, cy = scene.camp
    for structure in overlay.structures:
        ox, oy = structure.offset
        width, height = structure.size
        walls = perimeter(structure.size)
        door = (width // 2, height - 1)
        for y in range(height):
            for x in range(width):
                position = (cx + ox + x, cy + oy + y)
                result.append(Sprite(f"{structure.id}_floor_{x}_{y}", structure.floor,
                                     position, True, "floor"))
                if (x, y) in walls:
                    asset = (f"door_{structure.material}_open" if (x, y) == door else
                             f"wall_{structure.material}_{wall_mask((x, y), walls):02d}")
                    result.append(Sprite(f"{structure.id}_{x}_{y}", asset, position, True))
    result.extend(Sprite(row.id, row.asset, (cx + row.offset[0], cy + row.offset[1]),
                         True, row.category) for row in overlay.sprites)
    return tuple(result)


def real_sprites(scene: Scene) -> tuple[Sprite, ...]:
    result: list[Sprite] = []
    hut_cells: set[tuple[int, int]] = set()
    mapping = {"*": "hearth_1", "h": "lean_to", "t": "stump", "T": "oak_large",
               "x": "oak_large", "l": "log_pile_medium", "L": "log_pile_large"}
    for y, row in enumerate(scene.glyph_rows):
        for x, glyph in enumerate(row):
            if glyph in mapping:
                result.append(Sprite(f"real_{glyph}_{x}_{y}", mapping[glyph], (x, y), False))
            if glyph == "H":
                hut_cells.add((x, y))
            if glyph == "b":
                result.append(Sprite(f"real_site_{x}_{y}", "floor_packed_earth", (x, y), False,
                                     "floor", True))
    while hut_cells:
        first = min(hut_cells)
        component = {first}
        pending = [first]
        hut_cells.remove(first)
        while pending:
            x, y = pending.pop()
            for adjacent in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                if adjacent in hut_cells:
                    hut_cells.remove(adjacent)
                    component.add(adjacent)
                    pending.append(adjacent)
        xs, ys = zip(*component, strict=True)
        result.append(Sprite(f"real_hut_{first[0]}_{first[1]}", "hut",
                             ((min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2), False))
    return tuple(result)


def render_people(scene: Scene, overlay: Overlay) -> tuple[RenderPerson, ...]:
    ordered = sorted(scene.people, key=lambda p: ((p.x - scene.camp[0]) ** 2 +
                                                (p.y - scene.camp[1]) ** 2, p.id))
    if len(ordered) < len(overlay.people):
        message = f"Need {len(overlay.people)} real IDs for declared actor overrides"
        raise ValueError(message)
    overrides = {person.id: row for person, row in zip(ordered, overlay.people, strict=False)}
    result: list[RenderPerson] = []
    grouped: dict[tuple[float, float], int] = {}
    for person in sorted(scene.people, key=lambda row: row.id):
        if person.id in overrides:
            row = overrides[person.id]
            position = (scene.camp[0] + row.offset[0], scene.camp[1] + row.offset[1])
            result.append(RenderPerson(person, position, row.activity, row.top, True, row.mark))
        else:
            slot = grouped.get((person.x, person.y), 0)
            grouped[(person.x, person.y)] = slot + 1
            offset = ((slot + 1) // 2) * .5 * (1 if slot % 2 else -1) if slot else 0
            result.append(RenderPerson(person, (person.x + offset, person.y),
                                       person.activity))
    return tuple(result)
