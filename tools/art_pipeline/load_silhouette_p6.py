# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/load_silhouette_p6.py
"""Measure installed load silhouettes without inventing missing region annotations."""
from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from typing import Final
from PIL import Image

import numpy as np

from carry_boards_p6 import ROOT, Frame, installed_frames

OUTPUT: Final = ROOT / "docs/art/production6/carry/source_measurements.json"


@dataclass(frozen=True, slots=True)
class Measurement:
    """Per-frame evidence, with inferred geometry explicitly non-certifying."""
    sprite_id: str
    activity: str
    opaque_height: int
    inferred_torso_columns: tuple[int, int]
    inferred_load_bounds: tuple[int, int, int, int] | None
    outside_torso_pixels: int
    load_top_fraction: float | None
    load_bottom_fraction: float | None
    dark_boundary_fraction: float | None
    geometry_matches: bool
    certified_pass: bool
    limitation: str


def measure(frame: Frame) -> Measurement:
    """Use documented color/location inference; absent clothing regions remain FAIL."""
    pixels = np.asarray(frame.image())
    opaque = pixels[:, :, 3] > 0
    ys, xs = np.where(opaque)
    top, bottom = int(ys.min()), int(ys.max())
    height = bottom - top + 1
    mid = top + height // 2
    torso = np.where(opaque[mid])[0]
    left, right = int(torso.min()), int(torso.max())
    yy = np.indices(opaque.shape)[0]
    colors = pixels[:, :, :3].astype(np.float64)
    red, green, blue = colors[:, :, 0], colors[:, :, 1], colors[:, :, 2]
    # Installed maps label hair only. Brown/red candidates in the load area
    # deliberately remain an inference; skin and clothing can share these hues.
    mask = opaque & (red > green * 1.25) & (red > blue * 1.8) & (green < 130)
    mask &= (yy >= top + height * .2) & (yy <= top + height * .7)
    ly, lx = np.where(mask)
    if not len(lx):
        return Measurement(frame.sprite_id, frame.activity, height, (left, right), None,
                           0, None, None, None, False, False, "No load pixels identified")
    bounds = (int(lx.min()), int(ly.min()), int(lx.max()) + 1, int(ly.max()) + 1)
    outside = int(np.count_nonzero(mask[:, :left]) + np.count_nonzero(mask[:, right + 1:]))
    first, last = (bounds[1] - top) / height, (bounds[3] - top) / height
    edge = opaque & ~(np.roll(opaque, 1, 0) & np.roll(opaque, -1, 0)
                      & np.roll(opaque, 1, 1) & np.roll(opaque, -1, 1))
    load_edge = edge & (yy >= bounds[1]) & (yy < bounds[3])
    dark = np.max(colors, axis=2) < 75
    boundary = float(np.count_nonzero(load_edge & dark) / max(1, np.count_nonzero(load_edge)))
    geometry = last <= .45 if frame.activity == "eat" else outside >= 2 and first >= .5 and last <= .8 and boundary == 1.0
    return Measurement(frame.sprite_id, frame.activity, height, (left, right), bounds,
                       outside, first, last, boundary, geometry, False,
                       "Installed region map encodes hair only; torso/load are color inference, not certified regions")


def main() -> None:
    """Write every installed carry/eat frame's measurements and evidence limits."""
    rows = [asdict(measure(frame)) for frame in installed_frames()]
    installed_path = ROOT / "assets/art/atlas/carry_p6.json"
    rendered = []
    if installed_path.exists():
        installed = json.loads(installed_path.read_text())
        with Image.open(ROOT / installed["prop_page"].removeprefix("res://")) as atlas:
            for frame, source in zip(installed_frames(), rows, strict=True):
                entry = installed["sprites"].get(frame.sprite_id)
                if entry is None:
                    rendered.append({"sprite_id": frame.sprite_id, "composition": "original",
                                     "source_measurement": source, "certified_pass": False})
                    continue
                x, y, width, height = entry["rect"]
                prop = np.asarray(atlas.crop((x, y, x + width, y + height)))
                mask = prop[:, :, 3] > 0
                py, px = np.where(mask)
                anchor_x, anchor_y = entry["anchor"]
                bbox = frame.image().getbbox()
                assert bbox is not None
                opaque_height = bbox[3] - bbox[1]
                lower, upper = source["inferred_torso_columns"]
                outside = int(np.count_nonzero(px + anchor_x > upper))
                outside = max(outside, int(np.count_nonzero(px + anchor_x < lower)))
                low = (anchor_y + int(py.min()) - bbox[1]) / opaque_height
                high = (anchor_y + int(py.max()) + 1 - bbox[1]) / opaque_height
                padded = np.pad(mask, 1)
                interior = padded[:-2, 1:-1] & padded[2:, 1:-1] & padded[1:-1, :-2] & padded[1:-1, 2:]
                edge = mask & ~interior
                boundary_dark = bool(np.all(np.max(prop[:, :, :3], axis=2)[edge] < 75))
                rendered.append({"sprite_id": frame.sprite_id, "composition": "approved_food_layer",
                    "native_load_size": [width, height], "outside_torso_pixels": outside,
                    "load_top_fraction": low, "load_bottom_fraction": high,
                    "dark_outline": boundary_dark, "geometry_matches": outside >= 2 and low >= .5 and high <= .8 and boundary_dark,
                    "certified_pass": False, "limitation": "Torso columns inferred; installed region map has no clothing label"})
        (OUTPUT.parent / "rendered_measurements.json").write_text(json.dumps({"frames": rendered}, indent=2) + "\n")
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(json.dumps({"schema": "load_silhouette_p6_source_v1", "frames": rows,
                                  "certified_pass_count": 0}, indent=2) + "\n")
    print(f"Measured {len(rows)} installed frames; certified source passes 0 (region evidence missing)")


if __name__ == "__main__":
    main()
