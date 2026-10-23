# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.compositor.wall_metrics strip.png [strip2.png ...].
"""Numeric four-repeat seam instrument from STRUCT_GOAL, without reviewers."""
from __future__ import annotations

import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image

SEAM_FACTOR: Final = 1.15


@dataclass(frozen=True, slots=True)
class StripMeasurement:
    width: int
    height: int
    interior_p95: float
    join_columns: tuple[int, int, int]
    join_mad: tuple[float, float, float]
    limit: float
    passed: bool


def measure_strip(strip: Image.Image) -> StripMeasurement:
    """Compare RGB MAD at each join with 1.15 times interior-column p95.

    Alpha is deliberately excluded by the registered RGB statistic. Source
    opacity must be checked independently; a seam pass does not imply that
    a texture contains no transparent holes or looks natural.
    """
    if strip.width < 2 or strip.height < 1:
        message = "Seam measurement requires at least two columns and one row"
        raise ValueError(message)
    rgb = np.asarray(strip.convert("RGB"), dtype=np.float64)
    interior = sorted(
        float(np.mean(np.abs(rgb[:, col] - rgb[:, col - 1])))
        for col in range(1, strip.width)
    )
    rank = .95 * (len(interior) - 1)
    lower = int(rank)
    fraction = rank - lower
    p95 = interior[lower] * (1 - fraction) + interior[min(lower + 1, len(interior) - 1)] * fraction
    tiled = np.tile(rgb, (1, 4, 1))
    columns = (strip.width, strip.width * 2, strip.width * 3)
    joins = tuple(float(np.mean(np.abs(tiled[:, col] - tiled[:, col - 1]))) for col in columns)
    return StripMeasurement(
        strip.width, strip.height, p95, columns,
        (joins[0], joins[1], joins[2]), p95 * SEAM_FACTOR,
        all(join <= p95 * SEAM_FACTOR for join in joins),
    )


def main() -> int:
    """Emit literal per-texture evidence; nonzero status for any failed seam."""
    paths = tuple(Path(argument) for argument in sys.argv[1:])
    if not paths:
        print("Usage: python3 -m art.compositor.wall_metrics STRIP.png [STRIP.png ...]", file=sys.stderr)
        return 2
    passed = True
    for path in paths:
        with Image.open(path) as strip:
            result = measure_strip(strip)
        print(json.dumps({"path": str(path), **asdict(result)}, sort_keys=True))
        passed = passed and result.passed
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
