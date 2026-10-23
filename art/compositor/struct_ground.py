# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: imported by python -m art.compositor.struct_render.
"""Use ENV1 tones with STRUCT world-sampled banks, transitions and zone tint."""
from __future__ import annotations

from dataclasses import dataclass

from PIL import Image
from pydantic import JsonValue

from art.compositor.assets import REPO, read_image
from art.compositor.edges import (
    Boundary,
    EdgeField,
    boundary_steps,
    ford_overlay,
    render_tile,
    zone_overlay,
)
from art.compositor.geometry import WINDOW, Viewport
from art.compositor.ground import TERRAIN, GroundPainter
from art.compositor.models import Scene
from art.compositor.struct_layers import Layout
from art.compositor.tone import (
    Condition,
    NightLight,
    cap_ground,
    night_value,
    season_ground,
)


@dataclass(frozen=True, slots=True)
class GroundProduct:
    image: Image.Image
    boundary_metrics: tuple[dict[str, JsonValue], ...]
    field: EdgeField


def paint_ground(scene: Scene, layout: Layout, painter: GroundPainter) -> GroundProduct:
    """Blend before zoom reduction, and measure unoccluded terrain boundaries."""
    terrain = {(x, y): TERRAIN[glyph] for y, row in enumerate(scene.terrain_rows) for x, glyph in enumerate(row)}
    terrain.update(layout.floors)
    terrain.update((cell, "hill_rock") for cell in layout.hills)
    terrain.update((cell, "water") for cell in layout.fords)
    water = frozenset(cell for cell, kind in terrain.items() if kind in {"river_bank", "water"})
    texture_painter = GroundPainter(painter.catalog, Viewport(scene.camp, 1), painter.condition)
    textures = {kind: texture_painter.tile(kind) for kind in set(terrain.values()) - {"river_bank", "water"}}
    source = read_image(REPO / "art/style05/struct/water.png")
    calm = cap_ground(season_ground(source, painter.condition), painter.limit).image.convert("RGBA")
    textures.update({"river_bank": calm, "water": calm})
    field = EdgeField(terrain, textures, layout.wall_cells, water)
    viewport = painter.viewport
    image = Image.new("RGBA", WINDOW, (30, 32, 29, 255))
    xs, ys = viewport.tile_bounds((scene.width, scene.height))
    stones = tuple(read_image(REPO / f"art/style05/struct/ford_stone_{n}.png") for n in range(1, 5))
    tiles: dict[tuple[int, int], Image.Image] = {}
    tile_px = viewport.tile
    hearths = tuple(viewport.project(row.position) for row in layout.sprites if row.asset in {"hearth_1", "hearth_pots"})
    for y in ys:
        for x in xs:
            cell = (x, y)
            tile = render_tile(field, cell)
            px, py = viewport.project(cell)
            measured = tile.resize((tile_px, tile_px), Image.Resampling.LANCZOS)
            if painter.condition is Condition.NIGHT:
                local_hearths = tuple((hx - px + tile_px // 2, hy - py + tile_px // 2) for hx, hy in hearths)
                measured = night_value(measured, NightLight(local_hearths, tile_px))
            tiles[cell] = measured
            if cell in layout.fords:
                tile = Image.alpha_composite(tile, ford_overlay(stones, cell))
            scaled = tile.resize((viewport.tile, viewport.tile), Image.Resampling.LANCZOS)
            px, py = viewport.project(cell)
            image.alpha_composite(scaled, (px - viewport.tile // 2, py - viewport.tile // 2))
    metrics: list[dict[str, JsonValue]] = []
    segments: dict[tuple[str, int], list[tuple[int, Image.Image]]] = {}
    for (x, y), tile in sorted(tiles.items()):
        for dx, dy, axis in ((1, 0, "vertical"), (0, 1, "horizontal")):
            adjacent = (x + dx, y + dy)
            if adjacent not in tiles or terrain[(x, y)] == terrain[adjacent]:
                continue
            if (x, y) in layout.wall_cells or adjacent in layout.wall_cells:
                continue
            strip = Image.new("RGBA", (2 * tile_px, tile_px) if dx else (tile_px, 2 * tile_px))
            strip.paste(tile, (0, 0))
            strip.paste(tiles[adjacent], (tile_px * dx, tile_px * dy))
            boundary = Boundary((tile_px, 0), tile_px, "vertical") if dx else Boundary((0, tile_px), tile_px, "horizontal")
            result = boundary_steps(strip, boundary)
            key = (axis, x + 1) if dx else (axis, y + 1)
            segments.setdefault(key, []).append((y if dx else x, strip))
            metrics.append({"cell": [x, y], "axis": axis, "from": terrain[(x, y)], "to": terrain[adjacent],
                            "longest_identical_step": result.longest_identical_step, "passed": result.longest_identical_step <= 24})
    for (axis, coordinate), spans in sorted(segments.items()):
        groups: list[list[tuple[int, Image.Image]]] = []
        for span in sorted(spans, key=lambda item: item[0]):
            if not groups or span[0] != groups[-1][-1][0] + 1:
                groups.append([])
            groups[-1].append(span)
        for group in groups:
            vertical = axis == "vertical"
            length = tile_px * len(group)
            joined = Image.new("RGBA", (2 * tile_px, length) if vertical else (length, 2 * tile_px))
            for index, (_, strip) in enumerate(group):
                joined.paste(strip, (0, index * tile_px) if vertical else (index * tile_px, 0))
            boundary = Boundary((tile_px, 0), length, "vertical") if vertical else Boundary((0, tile_px), length, "horizontal")
            result = boundary_steps(joined, boundary)
            metrics.append({"axis": axis, "coordinate": coordinate, "start_tile": group[0][0],
                            "continuous_length_px": length, "longest_identical_step": result.longest_identical_step,
                            "passed": result.longest_identical_step <= 24})
    paint_zones(image, layout, viewport)
    return GroundProduct(image, tuple(metrics), field)


def paint_zones(image: Image.Image, layout: Layout, viewport: Viewport) -> None:
    """Draw one tint rectangle per contiguous native zone, never per-tile grids."""
    grouped: dict[int | None, set[tuple[int, int]]] = {}
    for row in layout.zones:
        grouped.setdefault(row.native_kind, set()).add((round(row.position[0]), round(row.position[1])))
    for cells in grouped.values():
        pending = set(cells)
        while pending:
            component = {min(pending)}
            queue = list(component)
            pending.difference_update(component)
            while queue:
                x, y = queue.pop()
                for cell in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                    if cell in pending:
                        pending.remove(cell)
                        component.add(cell)
                        queue.append(cell)
            left, top = min(x for x, _ in component), min(y for _, y in component)
            right, bottom = max(x for x, _ in component), max(y for _, y in component)
            tint = zone_overlay(((right - left + 1) * 64, (bottom - top + 1) * 64), (174, 172, 126))
            # Preserve non-rectangular zone coverage without interior grid lines.
            for y in range(top, bottom + 1):
                for x in range(left, right + 1):
                    if (x, y) not in component:
                        tint.paste((0, 0, 0, 0), ((x - left) * 64, (y - top) * 64, (x - left + 1) * 64, (y - top + 1) * 64))
            px, py = viewport.project((left, top))
            size = (round(tint.width * viewport.zoom), round(tint.height * viewport.zoom))
            image.alpha_composite(tint.resize(size, Image.Resampling.NEAREST), (px - viewport.tile // 2, py - viewport.tile // 2))
