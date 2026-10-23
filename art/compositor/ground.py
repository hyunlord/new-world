# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: imported by python3 -m art.compositor.render.
"""Measured tile-wide tone transforms after seasonal and snow composition."""
from __future__ import annotations

from dataclasses import asdict, dataclass, field, replace
from typing import Final

import numpy as np
from PIL import Image
from pydantic import JsonValue

from art.compositor.assets import ROOT, Catalog
from art.compositor.geometry import WINDOW, Viewport
from art.compositor.models import Scene
from art.compositor.placements import Sprite
from art.compositor.tone import (
    Condition,
    GroundLimit,
    cap_ground,
    person_limit,
    season_ground,
)

TERRAIN: Final = {".": "dark_grass", ",": "forest_floor", "#": "hill_rock",
                 "^": "hill_rock", "~": "river_bank", "_": "river_bank",
                 ":": "packed_earth", "-": "dark_grass", '"': "forest_floor",
                 "o": "hill_rock"}


@dataclass(slots=True)
class GroundPainter:
    """Mutable image/measurement cache shared by one render condition and zoom."""
    catalog: Catalog
    viewport: Viewport
    condition: Condition
    limit: GroundLimit = field(default_factory=lambda: replace(person_limit(ROOT / "style05/anchors/person.png"),
                                                             mean_max=.32, std_ratio=.25, saturation_max=.22))
    cache: dict[str, Image.Image] = field(default_factory=dict)
    measurements: dict[str, dict[str, JsonValue]] = field(default_factory=dict)

    def tile(self, asset_id: str) -> Image.Image:
        if asset_id in self.cache:
            return self.cache[asset_id]
        size = (self.viewport.tile, self.viewport.tile)
        source = self.catalog.image(asset_id).resize(size, Image.Resampling.LANCZOS)
        if asset_id == "river_bank":
            blue = np.asarray(source.convert("RGB"), dtype=np.float64) * np.array([.55, .85, 1.30])
            source = Image.fromarray(np.rint(np.clip(blue, 0, 255)).astype(np.uint8))
        seasonal = season_ground(source, self.condition)
        if self.condition is Condition.WINTER:
            snow = self.catalog.image("snow_overlay").resize(size, Image.Resampling.LANCZOS)
            rgba = np.asarray(snow).copy()
            rgba[:, :, 3] = np.minimum(rgba[:, :, 3], 153)
            seasonal = Image.alpha_composite(seasonal.convert("RGBA"), Image.fromarray(rgba))
        capped = cap_ground(seasonal, self.limit)
        if not capped.passed:
            message = f"Ground cap failed: {asset_id}/{self.condition}/{self.viewport.zoom}"
            raise ValueError(message)
        self.cache[asset_id] = capped.image.convert("RGBA")
        self.measurements[asset_id] = {
            "asset": asset_id, "condition": self.condition.value, "tile_px": self.viewport.tile,
            "source_asset_status": self.catalog.asset(asset_id).status,
            "snow_source_asset_status": self.catalog.asset("snow_overlay").status if self.condition is Condition.WINTER else None,
            "before": asdict(capped.before), "after": asdict(capped.after),
            "contrast_factor": capped.contrast_factor, "chroma_factor": capped.chroma_factor,
            "person_luminance_std": self.limit.person_luminance_std,
            "limits": {"mean": self.limit.mean_max, "person_std_ratio": self.limit.std_ratio,
                       "saturation": self.limit.saturation_max},
            "river_rgb_multiplier": [.55, .85, 1.30] if asset_id == "river_bank" else [1, 1, 1],
            "snow_alpha_max": .6 if self.condition is Condition.WINTER else 0,
            "passed": capped.passed,
        }
        return self.cache[asset_id]

    def paint(self, scene: Scene, floors: tuple[Sprite, ...]) -> Image.Image:
        image = Image.new("RGBA", WINDOW, (30, 32, 29, 255))
        xs, ys = self.viewport.tile_bounds((scene.width, scene.height))
        for y in ys:
            for x in xs:
                glyph = scene.terrain_rows[y][x]
                if glyph not in TERRAIN:
                    message = f"Unmapped actual terrain glyph {glyph!r} at {x},{y}"
                    raise ValueError(message)
                px, py = self.viewport.project((x, y))
                image.alpha_composite(self.tile(TERRAIN[glyph]),
                                      (px - self.viewport.tile // 2, py - self.viewport.tile // 2))
        for floor in floors:
            if self.viewport.visible(floor.position, self.viewport.tile):
                px, py = self.viewport.project(floor.position)
                image.alpha_composite(self.tile(floor.asset),
                                      (px - self.viewport.tile // 2, py - self.viewport.tile // 2))
        return image
