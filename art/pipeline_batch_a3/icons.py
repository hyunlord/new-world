# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by python -m art.pipeline_batch_a3.process.
"""Adopted ENV1 icon count composition and centered-canvas measurements."""

from __future__ import annotations

from typing import Final, Literal

from PIL import Image

from art.compositor.projection_check import ProjectionSpec, check_projection
from art.pipeline_batch_a3.checks import TileCheck, check_alpha

Bucket = Literal["1", "2_8", "9_plus"]
LAYOUTS: Final[tuple[tuple[Bucket, int, tuple[tuple[int, int], ...]], ...]] = (
    ("1", 64, ((0, 0),)),
    ("2_8", 42, ((3, 3), (16, 16), (3, 20))),
    (
        "9_plus",
        29,
        (
            (2, 3),
            (20, 3),
            (33, 7),
            (2, 20),
            (19, 20),
            (33, 23),
            (5, 34),
            (20, 34),
            (33, 34),
        ),
    ),
)
ICON_LIMIT: Final = 58
CANVAS_LIMIT: Final = 64


def icon_tiles(
    sprite: Image.Image, *, counts: bool
) -> tuple[tuple[Bucket, Image.Image], ...]:
    """Use ENV1 buckets unchanged, replacing only their hardwired output writes."""
    exports: list[tuple[Bucket, Image.Image]] = []
    for label, size, offsets in LAYOUTS if counts else LAYOUTS[:1]:
        result = Image.new("RGBA", (CANVAS_LIMIT, CANVAS_LIMIT))
        part = sprite.resize((size, size), Image.Resampling.LANCZOS)
        for offset in offsets:
            result.alpha_composite(part, offset)
        exports.append((label, result))
    return tuple(exports)


def inspect_icon(
    image: Image.Image, bucket: Bucket, expected: Image.Image
) -> TileCheck:
    """Check exact ENV1 placement; surviving-alpha center is diagnostic only."""
    alpha = check_alpha(image)
    bounds = alpha.visible_bounds
    opaque = alpha.opaque_bounds
    base = opaque[3] - 1 if opaque else 32
    projection = check_projection(
        image, ProjectionSpec("item", (32, base), (4, ICON_LIMIT))
    )
    width = opaque[2] - opaque[0] if opaque else 0
    center_offset = None
    centered = image.size == expected.size and image.convert("RGBA").tobytes() == (
        expected.convert("RGBA").tobytes()
    )
    safe = False
    if bounds:
        safe = min(bounds[:2]) >= 1 and max(bounds[2:]) <= CANVAS_LIMIT - 1
        if bucket == "1":
            center_offset = (
                (bounds[0] + bounds[2]) / 2 - 32,
                (bounds[1] + bounds[3]) / 2 - 32,
            )
            safe = (
                safe and max(bounds[2] - bounds[0], bounds[3] - bounds[1]) <= ICON_LIMIT
            )
    geometry = image.size == (64, 64) and alpha.passed and safe and centered
    return TileCheck(
        image.size,
        alpha,
        projection,
        width,
        geometry,
        "ENV1_CENTERED_ICON",
        center_offset,
        centered,
        safe,
    )
