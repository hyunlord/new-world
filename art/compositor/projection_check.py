# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: uv run art/compositor/projection_check.py IMAGE FAMILY ANCHOR_X ANCHOR_Y
"""Read-only projection measurements against declared sprite-space anchors."""
from __future__ import annotations

import json
import math
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from types import MappingProxyType
from typing import Final

import numpy as np
from PIL import Image

HEIGHT_BANDS: Final = MappingProxyType({
    "wall": (60, 60), "door": (58, 62), "bedroll": (12, 16),
    "workbench": (30, 40), "knapping_spot": (30, 40),
    "wood_bench": (30, 40), "pottery_bench": (30, 40),
    "grinding_stone": (30, 40), "drying_rack": (46, 50),
    "hearth": (18, 22), "hearth_lit": (20, 40), "hearth_firing": (20, 40),
    "boulder": (24, 36), "flint_outcrop": (18, 22), "clay_bank": (14, 18),
    "tree": (128, 192), "person": (54, 58), "child": (38, 42),
    "deer": (40, 60), "wolf": (28, 42), "stump": (14, 24),
    "carcass": (12, 24), "lean_to": (54, 66), "item": (4, 24),
})
ALPHA_THRESHOLD: Final = 128
BASE_TOLERANCE: Final = 2
LIGHT_EPSILON: Final = 1e-6


@dataclass(frozen=True, slots=True)
class ProjectionSpec:
    """Anchor belongs to the original canvas; overrides remain visible in output."""

    family: str
    anchor: tuple[int, int]
    height_band: tuple[int, int] | None = None


@dataclass(frozen=True, slots=True)
class ProjectionResult:
    """Measurements do not certify visual perspective or anatomical landmarks."""

    family: str
    anchor: tuple[int, int]
    height_band: tuple[int, int]
    height_band_override: bool
    opaque_pixels: int
    base_row: int | None
    base_error: int | None
    height: int
    gradient_x: float
    gradient_y: float
    light_angle_degrees: float | None
    base_passed: bool
    height_passed: bool
    light_passed: bool
    passed: bool


def check_projection(image: Image.Image, spec: ProjectionSpec) -> ProjectionResult:
    """Measure alpha silhouette and mean interior adjacent-pixel luminance slopes.

    Screen coordinates have +y down. A negative x and y slope means brightness
    increases toward the upper left. Transparent pixels never enter gradients.
    """
    band = spec.height_band if spec.height_band is not None else HEIGHT_BANDS[spec.family]
    if not 0 < band[0] <= band[1]:
        message = f"Invalid height band: {band}"
        raise ValueError(message)
    rgba = np.asarray(image.convert("RGBA"), dtype=np.float64)
    opaque = rgba[:, :, 3] >= ALPHA_THRESHOLD
    rows = np.nonzero(opaque)[0]
    count = int(np.count_nonzero(opaque))
    base = int(rows.max()) if count else None
    height = int(rows.max() - rows.min() + 1) if count else 0
    base_error = base - spec.anchor[1] if base is not None else None
    luminance = rgba[:, :, :3] @ np.array([0.2126, 0.7152, 0.0722])
    valid_x = opaque[:, 1:] & opaque[:, :-1]
    valid_y = opaque[1:, :] & opaque[:-1, :]
    dx = np.diff(luminance, axis=1)[valid_x]
    dy = np.diff(luminance, axis=0)[valid_y]
    gx = float(dx.mean()) if dx.size else 0.0
    gy = float(dy.mean()) if dy.size else 0.0
    measurable = bool(dx.size and dy.size and math.hypot(gx, gy) > LIGHT_EPSILON)
    angle = math.degrees(math.atan2(gy, gx)) % 360.0 if measurable else None
    base_ok = base_error is not None and abs(base_error) <= BASE_TOLERANCE
    height_ok = band[0] <= height <= band[1]
    light_ok = measurable and gx < -LIGHT_EPSILON and gy < -LIGHT_EPSILON
    return ProjectionResult(
        spec.family, spec.anchor, band, spec.height_band is not None, count,
        base, base_error, height, gx, gy, angle, base_ok, height_ok, light_ok,
        base_ok and height_ok and light_ok,
    )


def main() -> int:
    """Emit machine-readable measurements; rejected candidates return exit 1."""
    if len(sys.argv) != 5:
        print("Usage: projection_check.py IMAGE FAMILY ANCHOR_X ANCHOR_Y", file=sys.stderr)
        return 2
    image_path = Path(sys.argv[1])
    family = sys.argv[2]
    if family not in HEIGHT_BANDS:
        print(f"Unknown family {family!r}; choose from {', '.join(HEIGHT_BANDS)}", file=sys.stderr)
        return 2
    spec = ProjectionSpec(family, (int(sys.argv[3]), int(sys.argv[4])))
    with Image.open(image_path) as source:
        result = check_projection(source, spec)
    print(json.dumps(asdict(result), indent=2, allow_nan=False))
    return 0 if result.passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
