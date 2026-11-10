# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: uv run --with pillow --with numpy --with pydantic python -m art.batch_b3.rooms.render RECIPES.json OUTPUT_DIR
"""Render review-only lifecycle compositions through the existing material system."""

from __future__ import annotations

import hashlib
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal

from PIL import Image

from art.batch_b3.rooms.layers import course_image, course_origin
from art.batch_b3.rooms.schema import (
    LayerRecord,
    Manifest,
    Point,
    RecipeError,
    Recipes,
    Room,
    Source,
)
from art.compositor.walls import ANCHOR, TILE, Door, wall_tiles
from art.materials.render import load_surface, load_textures
from art.materials.resolve import read_catalog, resolve

ROOT: Final = Path(__file__).resolve().parents[3]


@dataclass(frozen=True, slots=True)
class Layer:
    """A transient independently depth-sortable surface."""

    image: Image.Image
    kind: Literal["floor", "wall", "prop"]
    cell: Point
    anchor: Point
    origin: Point
    sources: tuple[str, ...]
    operations: tuple[str, ...]

    @property
    def sort_key(self) -> tuple[int, int, int]:
        """Ground precedes all standing content; source anchors determine depth."""
        return (
            int(self.kind != "floor"),
            self.origin[1] + self.anchor[1],
            self.origin[0] + self.anchor[0],
        )


def digest(path: Path) -> str:
    """Hash exact bytes, not decoded image pixels."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def collect(spec: Room) -> tuple[Layer, ...]:
    """Build layers without editing any generated texture or prior asset."""
    catalog = read_catalog(ROOT)
    wall = resolve(catalog, spec.wall_style, spec.material)
    if wall.style_id != spec.wall_style:
        raise RecipeError(spec.id, "unknown wall style")
    floor = resolve(catalog, "packed_earth", "soil", kind="floor")
    textures = load_textures(ROOT, wall)
    wall_sources = tuple(
        p
        for p in (
            wall.textures.face,
            wall.textures.top_ew,
            wall.textures.top_ns,
            wall.textures.end,
            wall.textures.junction,
            wall.textures.door_open,
            wall.textures.door_closed,
        )
        if p is not None
    )
    layers: list[Layer] = []
    if spec.floor:
        floor_source = floor.textures.floor
        if floor_source is None:
            raise RecipeError(spec.id, "floor texture missing")
        surface = load_surface(ROOT, floor)
        for y in range(spec.footprint[1]):
            for x in range(spec.footprint[0]):
                left, top = x * TILE % surface.width, y * TILE % surface.height
                height = ANCHOR + 1 if y == spec.footprint[1] - 1 else TILE
                crop = (left, top, left + TILE, top + height)
                layers.append(
                    Layer(
                        surface.crop(crop),
                        "floor",
                        (x, y),
                        (32, ANCHOR),
                        (x * TILE, y * TILE),
                        (floor_source,),
                        (f"World-phased floor crop {crop}",),
                    )
                )
    for course in spec.courses:
        doors = (
            ()
            if course.door_cell is None
            else (
                Door(
                    course.door_cell,
                    textures.door_open if course.door_open else textures.door_closed,
                ),
            )
        )
        for tile in wall_tiles(frozenset(course.cells), textures, doors):
            layers.append(
                Layer(
                    course_image(tile.image, course.height),
                    "wall",
                    tile.cell,
                    (32, course.height - 1),
                    course_origin(tile.cell, course.height),
                    wall_sources,
                    (
                        f"Existing wall_tiles topology; exposed height {course.height}; retain full N/S cap depth",
                        "Exact top/first-face-row crop only; no resampling, paint or recolour",
                    ),
                )
            )
    for prop in spec.props:
        path = (ROOT / prop.source).resolve()
        if not path.is_relative_to(ROOT) or not path.is_file():
            raise RecipeError(spec.id, f"missing or escaping prop: {prop.source}")
        with Image.open(path) as image:
            layers.append(
                Layer(
                    image.convert("RGBA"),
                    "prop",
                    prop.cell,
                    prop.anchor,
                    (
                        prop.cell[0] * TILE + 32 + prop.offset[0] - prop.anchor[0],
                        prop.cell[1] * TILE + ANCHOR + prop.offset[1] - prop.anchor[1],
                    ),
                    (prop.source,),
                    ("Unchanged source pixels at published anchor",),
                )
            )
    if not layers:
        raise RecipeError(spec.id, "empty composition")
    return tuple(sorted(layers, key=lambda layer: layer.sort_key))


def render(spec: Room, output: Path) -> Manifest:
    """Write a reproducible layered recipe; this function never adopts it."""
    layers = collect(spec)
    folder = output / spec.id
    (folder / "layers").mkdir(parents=True, exist_ok=True)
    left = min(layer.origin[0] for layer in layers)
    top = min(layer.origin[1] for layer in layers)
    right = max(layer.origin[0] + layer.image.width for layer in layers)
    bottom = max(layer.origin[1] + layer.image.height for layer in layers)
    preview = Image.new("RGBA", (right - left, bottom - top))
    records: list[LayerRecord] = []
    for index, layer in enumerate(layers):
        path = (
            folder
            / "layers"
            / f"{index:03}_{layer.kind}_{layer.cell[0]}_{layer.cell[1]}.png"
        )
        layer.image.save(path)
        preview.alpha_composite(
            layer.image, (layer.origin[0] - left, layer.origin[1] - top)
        )
        records.append(
            LayerRecord(
                path=str(path.relative_to(output)),
                sha256=digest(path),
                kind=layer.kind,
                cell=layer.cell,
                size=layer.image.size,
                anchor=layer.anchor,
                origin=layer.origin,
                operations=layer.operations,
            )
        )
    preview.save(folder / "preview.png")
    preview.resize(
        (preview.width * 2, preview.height * 2), Image.Resampling.NEAREST
    ).save(folder / "preview_2x.png")
    inputs = {source for layer in layers for source in layer.sources}
    inputs.update(
        (
            "art/materials/construction_styles.json",
            "art/materials/material_looks.json",
            "art/materials/category_defaults.json",
            "art/materials/render.py",
            "art/materials/resolve.py",
            "art/materials/tone.py",
            "art/compositor/walls.py",
            "art/compositor/legacy_walls.py",
            "art/batch_b3/rooms/render.py",
            "art/batch_b3/rooms/layers.py",
        )
    )
    result = Manifest(
        recipe=spec,
        layers=tuple(records),
        sources=tuple(
            Source(path=source, sha256=digest(ROOT / source))
            for source in sorted(inputs)
        ),
        preview_origin=(left, top),
        preview_anchor=(
            spec.footprint[0] * 32 - left,
            (spec.footprint[1] - 1) * TILE + ANCHOR - top,
        ),
        preview_sha256=digest(folder / "preview.png"),
    )
    _ = (folder / "manifest.json").write_text(result.model_dump_json(indent=2) + "\n")
    return result


def main() -> int:
    """Read explicit recipes and destination; no historical output defaults."""
    recipes = Recipes.model_validate_json(Path(sys.argv[1]).read_bytes())
    output = Path(sys.argv[2])
    for room in recipes.rooms:
        _ = render(room, output)
    _ = sys.stdout.write(f"Rendered {len(recipes.rooms)} layered recipes to {output}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
