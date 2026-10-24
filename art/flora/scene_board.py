# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "typer"]
# ///
# How to run: python3 -m art.flora.boards
"""Recompose the frozen ENV1 observations with data-resolved seasonal flora."""
from __future__ import annotations

from dataclasses import dataclass, replace
from functools import lru_cache
from typing import assert_never

from PIL import Image, ImageDraw, ImageFilter

from art.compositor.assets import REPO
from art.compositor.drawing import ForegroundPainter
from art.compositor.geometry import HUD_HEIGHT, WINDOW, Viewport
from art.compositor.ground import GroundPainter
from art.compositor.placements import RenderPerson, Sprite, render_people
from art.compositor.polish2_walls import frozen_inputs
from art.compositor.render import FONT, RenderProduct
from art.compositor.struct_ground import paint_ground
from art.compositor.struct_layers import StructureTile, build_layout, structural_tiles
from art.compositor.struct_render import paint_structure
from art.compositor.tone import Condition
from art.flora.resolve_flora import ResolvedFlora, resolve_flora
from art.flora.schema import FloraCatalog, Season


@dataclass(frozen=True, slots=True)
class TreePlacement:
    source: Sprite
    resolved: ResolvedFlora

    @property
    def position(self) -> tuple[float, float]:
        return self.source.position


def tree_selection(row: Sprite, catalog: FloraCatalog, season: Season) -> TreePlacement:
    """Hash only observed coordinates; species assignment is declared art metadata."""
    choices = sorted(key for key, species in catalog.species.species.items()
                     if species.habit != "shrub")
    x, y = round(row.position[0]), round(row.position[1])
    key = (x * 73856093) ^ (y * 19349663)
    species = choices[key % len(choices)]
    size = "small" if (key >> 3) % 4 == 0 else "large"
    frame = resolve_flora(species, size, season, key >> 6, catalog=catalog)
    return TreePlacement(row, frame)


def seasonal_image(frame: ResolvedFlora) -> Image.Image:
    """Cache by canonical serialized data rather than mutable model hash semantics."""
    return cached_image(frame.model_dump_json())


@lru_cache(maxsize=256)
def cached_image(serialized: str) -> Image.Image:
    """Render one immutable resolution record and reuse its unmodified pixels."""
    from art.flora.seasons import render_frame
    return render_frame(ResolvedFlora.model_validate_json(serialized), REPO)


def shadow(size: tuple[int, int]) -> Image.Image:
    """Keep contact/canopy shadow entirely separate from generated tree pixels."""
    width, height = size
    image = Image.new("RGBA", size)
    ImageDraw.Draw(image).ellipse((2, 2, width - 3, height - 3), fill=(18, 25, 17, 55))
    return image.filter(ImageFilter.GaussianBlur(max(1, height / 8)))


def paint_tree(image: Image.Image, tree: TreePlacement, viewport: Viewport) -> None:
    """Place exactly at the declared base, with a soft procedural shadow beneath."""
    source = seasonal_image(tree.resolved)
    scale = viewport.zoom
    frame = source.resize((max(1, round(source.width * scale)), max(1, round(source.height * scale))),
                          Image.Resampling.NEAREST)
    px, py = viewport.project(tree.position)
    py += round(11 * scale)
    shade = shadow((max(8, round(source.width * .7 * scale)), max(6, round(18 * scale))))
    image.alpha_composite(shade, (px - shade.width // 2, py - shade.height // 2))
    image.alpha_composite(frame, (px - round(tree.resolved.anchor[0] * scale),
                                  py - round(tree.resolved.anchor[1] * scale)))


def dense_cells(trees: tuple[TreePlacement, ...]) -> frozenset[tuple[int, int]]:
    """Use canopy mass only where a tree has two or more adjacent tree cells."""
    cells = {(round(tree.position[0]), round(tree.position[1])) for tree in trees}
    return frozenset((x, y) for x, y in cells if sum(
        (x + dx, y + dy) in cells for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1))) >= 2)


@lru_cache(maxsize=64)
def seasonal_mass(serialized: str) -> Image.Image:
    """Apply the resolved season to foliage clusters, preserving mass-texture ink."""
    from art.flora.seasons import apply_tint, canopy_mask
    frame = ResolvedFlora.model_validate_json(serialized)
    with Image.open(REPO / frame.map_canopy_path) as source:
        texture = source.convert("RGBA")
    return apply_tint(texture, canopy_mask(texture, REPO), frame.tint)


def paint_mass(image: Image.Image, trees: tuple[TreePlacement, ...], viewport: Viewport) -> frozenset[tuple[int, int]]:
    """World-sample seasonal canopy; bare deciduous trees stay as branch sprites."""
    candidates = dense_cells(trees)
    dense = frozenset((round(tree.position[0]), round(tree.position[1])) for tree in trees
                      if tree.resolved.tint.frame != "bare"
                      and (round(tree.position[0]), round(tree.position[1])) in candidates)
    for tree in trees:
        cell = (round(tree.position[0]), round(tree.position[1]))
        if cell not in dense or not viewport.visible(tree.position, 32):
            continue
        texture = seasonal_mass(tree.resolved.model_dump_json())
        left, top = (cell[0] * 64) % 256, (cell[1] * 64) % 256
        tile = texture.crop((left, top, left + 64, top + 64))
        tile = tile.resize((viewport.tile, viewport.tile), Image.Resampling.LANCZOS)
        px, py = viewport.project(tree.position)
        image.alpha_composite(tile, (px - viewport.tile // 2, py - viewport.tile // 2))
    return dense


def compose_scene(catalog: FloraCatalog, season: Season, zoom: float = 1) -> RenderProduct:
    """Preserve all frozen non-tree placements, terrain, camera and declared overlay."""
    inputs = frozen_inputs()
    scene, assets = inputs.scene, inputs.catalog
    camera = (scene.camp[0] + 8, scene.camp[1]) if zoom == 1 else scene.camp
    viewport = Viewport(camera, zoom)
    layout = build_layout(scene, inputs.overlay, assets)
    condition = Condition.WINTER if season == "winter" else Condition.DAY
    ground = paint_ground(scene, layout, GroundPainter(assets, viewport, condition))
    image = ground.image
    trees = tuple(tree_selection(row, catalog, season) for row in layout.sprites if row.asset == "oak_large")
    dense: frozenset[tuple[int, int]] = paint_mass(image, trees, viewport) if zoom == .25 else frozenset()
    actors = render_people(scene, inputs.overlay)
    painter = ForegroundPainter(image, viewport, assets, zoom < 1)
    walls = structural_tiles(layout, assets, (scene.width, scene.height))
    layers: list[TreePlacement | Sprite | RenderPerson | StructureTile] = [
        *(row for row in layout.sprites if row.asset != "oak_large"), *trees, *actors, *walls]
    for row in sorted(layers, key=lambda item: (item.position[1], item.position[0])):
        match row:
            case TreePlacement():
                cell = (round(row.position[0]), round(row.position[1]))
                if cell not in dense and viewport.visible(row.position, round(220 * zoom)):
                    paint_tree(image, row, viewport)
                continue
            case StructureTile():
                _ = paint_structure(painter, row)
                continue
            case Sprite():
                placed = row if row.category == "animal" else replace(row, position=(row.position[0], row.position[1] + 11 / 64))
                _ = painter.sprite(placed)
                continue
            case RenderPerson():
                _ = painter.person(row, 1)
                continue
        assert_never(row)
    # Keep original clock out of a cross-season rehearsal, which is not a simulated date.
    image.paste((27, 31, 26, 255), (0, 0, WINDOW[0], HUD_HEIGHT))
    draw = ImageDraw.Draw(image)
    draw.text((18, 18), f"ART-TREES / {season.upper()} / {zoom:g}x / OWNER VERDICT PENDING", font=FONT, fill=(231, 232, 212))
    draw.text((18, 44), "Frozen ENV1 day 6, tick 7440; same terrain and people. Season and species are declared art previews.", font=FONT, fill=(205, 212, 193))
    draw.text((18, 70), "Existing declared ENV1 structures/actions retained. Quarter view: dense forest canopy mass; sparse trees retain silhouettes.", font=FONT, fill=(205, 212, 193))
    return RenderProduct(image.convert("RGB"), {
        "scene": "art/compositor/scenes/scene_day6_noon.json", "tick": scene.tick,
        "season_preview": season, "zoom": zoom, "tree_count": len(trees),
        "camera_center": list(camera), "source_camp": list(scene.camp),
        "camera_note": "1x pans 8 tiles east to include forest and camp; quarter retains camp center",
        "dense_mass_cells": len(dense), "source_population": len(scene.people),
        "tree_assignment": "sorted species; coordinate xor hash; size/hash variant; art-only, not simulated species",
        "overlay": "art/compositor/scenes/m5_overlay.json", "owner_approval": "PENDING",
        "depth": "base y then x; base offset 11px within 64px tile",
    })
