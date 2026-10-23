# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: imported by python3 -m art.compositor.render.
"""Measure every visible ground tile after seasonal and lighting transforms."""
from __future__ import annotations

from dataclasses import asdict

import numpy as np
from PIL import Image
from pydantic import JsonValue

from art.compositor.geometry import HUD_HEIGHT, WINDOW, Viewport
from art.compositor.models import Scene
from art.compositor.tone import GroundLimit, metrics


def ground_evidence(image: Image.Image, scene: Scene, viewport: Viewport,
                    limit: GroundLimit) -> dict[str, JsonValue]:
    rows: list[dict[str, JsonValue]] = []
    xs, ys = viewport.tile_bounds((scene.width, scene.height))
    for y in ys:
        for x in xs:
            px, py = viewport.project((x, y))
            half = viewport.tile // 2
            box = (max(0, px - half), max(HUD_HEIGHT, py - half),
                   min(WINDOW[0], px + half), min(WINDOW[1] - 23, py + half))
            if box[0] >= box[2] or box[1] >= box[3]:
                continue
            measured = metrics(np.asarray(image.crop(box).convert("RGB"), dtype=np.float64) / 255)
            registered = (measured.luminance_mean <= .4 and measured.saturation_mean <= .3
                          and measured.luminance_std <= limit.person_luminance_std * .6)
            strict = (measured.luminance_mean <= limit.mean_max and measured.saturation_mean <= limit.saturation_max
                      and measured.luminance_std <= limit.person_luminance_std * limit.std_ratio)
            rows.append({"tile": [x, y], "screen_crop": list(box), "metrics": asdict(measured),
                         "registered_pass": registered, "strict_initial_pass": strict})
    return {"measurement_stage": "ground-only pixels after season, snow, cap and hearth lighting; before foreground/HUD",
            "tile_count": len(rows), "registered_pass": all(row["registered_pass"] for row in rows),
            "strict_initial_pass": all(row["strict_initial_pass"] for row in rows),
            "tiles": [row for row in rows]}
