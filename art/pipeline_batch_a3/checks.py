# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by python -m art.pipeline_batch_a3.process.
"""Immutable alpha and geometry checks for A3 item candidates."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Final, Literal

import numpy as np
from PIL import Image

from art.compositor.projection_check import (
    ALPHA_THRESHOLD,
    ProjectionResult,
    ProjectionSpec,
    check_projection,
)

if TYPE_CHECKING:
    from numpy.typing import NDArray

MAX_WIDTH: Final = 54


@dataclass(frozen=True, slots=True)
class AlphaCheck:
    """Source alpha facts, including faint pixels excluded by opaque bounds."""

    has_alpha: bool
    has_transparency: bool
    has_subject: bool
    border_clear: bool
    border_alpha_max: int
    passed: bool
    visible_pixels: int
    opaque_pixels: int
    fringe_pixels: int
    visible_bounds: tuple[int, int, int, int] | None
    opaque_bounds: tuple[int, int, int, int] | None


@dataclass(frozen=True, slots=True)
class TileCheck:
    """Declared tile geometry plus the unchanged projection instrument result."""

    canvas_px: tuple[int, int]
    alpha: AlphaCheck
    projection: ProjectionResult
    width_px: int
    structural_pass: bool
    geometry_contract: Literal["STRUCT_WORLD_ITEM", "ENV1_CENTERED_ICON"] = (
        "STRUCT_WORLD_ITEM"
    )
    center_offset_px: tuple[float, float] | None = None
    center_passed: bool | None = None
    safe_bounds_passed: bool | None = None


def check_alpha(image: Image.Image) -> AlphaCheck:
    """Require generated transparency, an opaque subject, and an unclipped edge."""
    pixels: NDArray[np.uint8] = np.asarray(image.convert("RGBA"))
    alpha: NDArray[np.uint8] = pixels[:, :, 3]
    has_alpha = "A" in image.getbands()
    transparency = bool(np.any(np.equal(alpha, 0)))
    subject = bool(np.any(alpha >= ALPHA_THRESHOLD))
    border_values = np.concatenate(
        (
            alpha[:1, :].ravel(),
            alpha[-1:, :].ravel(),
            alpha[:, :1].ravel(),
            alpha[:, -1:].ravel(),
        )
    )
    border_max = int(np.max(border_values))
    border = border_max < ALPHA_THRESHOLD
    visible = int(np.count_nonzero(alpha))
    opaque = int(np.count_nonzero(alpha >= ALPHA_THRESHOLD))
    opaque_mask = Image.fromarray(
        np.where(alpha >= ALPHA_THRESHOLD, 255, 0).astype(np.uint8)
    )
    return AlphaCheck(
        has_alpha,
        transparency,
        subject,
        border,
        border_max,
        has_alpha and transparency and subject and border,
        visible,
        opaque,
        visible - opaque,
        image.convert("RGBA").getchannel("A").getbbox(),
        opaque_mask.getbbox(),
    )


def inspect_tile(image: Image.Image) -> TileCheck:
    """Report geometry and lighting separately; never move a failing anchor."""
    alpha = check_alpha(image)
    projection = check_projection(image, ProjectionSpec("item", (32, 43)))
    pixels: NDArray[np.uint8] = np.asarray(image.convert("RGBA"))
    columns = np.nonzero(pixels[:, :, 3] >= ALPHA_THRESHOLD)[1]
    width = int(columns.max() - columns.min() + 1) if columns.size else 0
    geometry = (
        image.size == (64, 64)
        and alpha.passed
        and width <= MAX_WIDTH
        and projection.base_passed
        and projection.height_passed
    )
    return TileCheck(image.size, alpha, projection, width, geometry)
