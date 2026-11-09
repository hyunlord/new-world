# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: uv run --with pillow --with numpy --with pydantic python -m art.batch_a2.rooms
"""Derived roofless room recipes using the existing material renderer."""

from __future__ import annotations

import hashlib
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal

from PIL import Image

from art.batch_a2.rooms_schema import (
    AnchorMetadata,
    LayerRecord,
    Point,
    PropSpec,
    RecipeError,
    Recipes,
    RoomManifest,
    RoomSpec,
    SourceHash,
)
from art.compositor.walls import ANCHOR, CAP, FACE, TILE, Door, render_walls, wall_tiles
from art.materials.render import load_surface, load_textures
from art.materials.resolve import read_catalog, resolve

ROOT: Final = Path(__file__).resolve().parents[2]
EVIDENCE: Final = Path("/Users/rexxa/work/art-batch/A2/rooms")


@dataclass(frozen=True, slots=True)
class Layer:
    """Transient image with explicit placement; no resampling is permitted."""

    image: Image.Image
    kind: Literal["floor", "wall", "prop"]
    cell: Point
    anchor: Point
    origin: Point
    role: str
    sources: tuple[str, ...]
    operations: tuple[str, ...] = ("Unchanged source pixels at published anchor.",)
    footprint: Point = (1, 1)

    @property
    def base(self) -> Point:
        """Return world position of the source anchor."""
        return self.origin[0] + self.anchor[0], self.origin[1] + self.anchor[1]

    @property
    def order(self) -> tuple[int, int, int]:
        """Keep top-down floor below all base-sorted standing content."""
        return int(self.kind != "floor"), self.base[1], self.base[0]


def source_hash(relative: str) -> SourceHash:
    """Hash a repository input and reject paths outside the repository."""
    path = (ROOT / relative).resolve()
    if not path.is_relative_to(ROOT):
        raise RecipeError(relative, "source escapes repository")
    return SourceHash(path=str(path.relative_to(ROOT)), sha256=hashlib.sha256(path.read_bytes()).hexdigest())


def load_prop(prop: PropSpec) -> Layer | None:
    """Read one existing prop at its published anchor without modifying pixels."""
    path = ROOT / prop.source
    if not path.is_file():
        return None
    anchor = prop.anchor
    sources = (prop.source,)
    if anchor is None:
        metadata = AnchorMetadata.model_validate_json(path.with_suffix(".json").read_bytes())
        anchor = metadata.anchor or metadata.anchor_point
        sources += (str(path.with_suffix(".json").relative_to(ROOT)),)
    if anchor is None:
        raise RecipeError(prop.source, "published anchor missing")
    with Image.open(path) as image:
        origin = (
            prop.cell[0] * TILE + 32 + prop.offset[0] - anchor[0],
            prop.cell[1] * TILE + ANCHOR + prop.offset[1] - anchor[1],
        )
        return Layer(
            image.convert("RGBA"),
            "prop",
            prop.cell,
            anchor,
            origin,
            prop.role,
            sources,
            footprint=prop.footprint,
        )


def export_room(spec: RoomSpec, output: Path) -> RoomManifest:
    """Export independent layers and review-only composites with frozen inputs."""
    catalog = read_catalog(ROOT)
    wall = resolve(catalog, spec.wall_style, spec.material)
    if wall.style_id != spec.wall_style:
        raise RecipeError(spec.id, "unknown wall style")
    floor = resolve(catalog, "packed_earth", "soil", kind="floor")
    textures = load_textures(ROOT, wall)
    wall_sources = tuple(
        path
        for path in (
            wall.textures.face,
            wall.textures.top_ew,
            wall.textures.top_ns,
            wall.textures.end,
            wall.textures.junction,
            wall.textures.door_closed,
            wall.textures.door_open,
        )
        if path is not None
    )
    floor_source = floor.textures.floor
    if floor_source is None:
        raise RecipeError(spec.id, "floor texture missing")
    floor_image = load_surface(ROOT, floor)
    cells = frozenset(spec.cells)
    doors = (
        ()
        if spec.door_cell is None
        else (Door(spec.door_cell, textures.door_open if spec.door_open else textures.door_closed),)
    )
    layers: list[Layer] = []
    if spec.floor:
        for y in range(spec.footprint[1]):
            for x in range(spec.footprint[0]):
                left, top = x * TILE % floor_image.width, y * TILE % floor_image.height
                height = ANCHOR + 1 if y == spec.footprint[1] - 1 else TILE
                crop = (left, top, left + TILE, top + height)
                image = floor_image.crop(crop)
                layers.append(
                    Layer(
                        image,
                        "floor",
                        (x, y),
                        (32, ANCHOR),
                        (x * TILE, y * TILE),
                        "top-down floor",
                        (floor_source,),
                        (f"World-phased material floor crop: {crop}; no resampling.",),
                    )
                )
    layers.extend(
        Layer(
            tile.image,
            "wall",
            tile.cell,
            (32, CAP + FACE - 1),
            tile.origin,
            "roofless wall or door",
            wall_sources,
            ("Existing wall_tiles topology/material renderer; no resizing.",),
        )
        for tile in wall_tiles(cells, textures, doors)
    )
    unresolved: list[str] = []
    for prop in spec.props:
        layer = load_prop(prop)
        if layer is None:
            unresolved.append(prop.source)
        else:
            layers.append(layer)
    folder = output / spec.id
    (folder / "layers").mkdir(parents=True, exist_ok=True)
    records: list[LayerRecord] = []
    left, top = min(row.origin[0] for row in layers), min(row.origin[1] for row in layers)
    right = max(row.origin[0] + row.image.width for row in layers)
    bottom = max(row.origin[1] + row.image.height for row in layers)
    preview = Image.new("RGBA", (right - left, bottom - top))
    for index, layer in enumerate(sorted(layers, key=lambda row: row.order)):
        path = folder / "layers" / f"{index:03}_{layer.kind}_{layer.cell[0]}_{layer.cell[1]}.png"
        layer.image.save(path)
        preview.alpha_composite(layer.image, (layer.origin[0] - left, layer.origin[1] - top))
        records.append(
            LayerRecord(
                path=str(path.relative_to(output)),
                sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                kind=layer.kind,
                cell=layer.cell,
                size=layer.image.size,
                anchor=layer.anchor,
                world_origin=layer.origin,
                world_base=layer.base,
                sort_key=layer.order,
                role=layer.role,
                sources=tuple(source_hash(source) for source in layer.sources),
                operations=layer.operations,
                footprint=layer.footprint,
            )
        )
    preview.save(folder / "preview.png")
    preview.resize((preview.width * 2, preview.height * 2), Image.Resampling.NEAREST).save(
        folder / "preview_2x.png"
    )
    walls = render_walls(cells, textures, doors)
    walls.image.save(folder / "walls_reference.png")
    paths = {source.path for record in records for source in record.sources}
    paths.update(
        (
            "art/materials/construction_styles.json",
            "art/materials/material_looks.json",
            "art/materials/category_defaults.json",
            "art/materials/render.py",
            "art/materials/resolve.py",
            "art/materials/tone.py",
            "art/compositor/walls.py",
            "art/compositor/legacy_walls.py",
            "art/batch_a2/rooms.py",
            "art/batch_a2/rooms_schema.py",
            "art/batch_a2/rooms.json",
        )
    )
    result = RoomManifest(
        recipe=spec,
        status="DERIVED_INCOMPLETE" if unresolved else "DERIVED_AWAITING_REVIEW",
        resolved_wall=wall,
        resolved_floor=floor,
        layers=tuple(records),
        inputs=tuple(source_hash(path) for path in sorted(paths)),
        unresolved=tuple(unresolved),
        preview_origin=(left, top),
        walls_reference_origin=walls.origin,
        preview_sha256=hashlib.sha256((folder / "preview.png").read_bytes()).hexdigest(),
    )
    _ = (folder / "manifest.json").write_text(result.model_dump_json(indent=2) + "\n")
    return result


def main() -> int:
    """Render all recipe variants; incomplete content yields a nonzero exit."""
    recipe_path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_suffix(".json")
    arguments = sys.argv[1:]
    output = Path(arguments[1]) if len(arguments) > 1 else EVIDENCE
    recipes = Recipes.model_validate_json(recipe_path.read_bytes())
    results = tuple(export_room(spec, output) for spec in recipes.rooms)
    for result in results:
        _ = sys.stdout.write(f"{result.recipe.id}: {result.status}; {','.join(result.unresolved)}\n")
    return int(any(result.unresolved for result in results))


if __name__ == "__main__":
    raise SystemExit(main())
