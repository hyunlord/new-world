# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy", "pillow"]
# ///
# How to run: imported by art.climate_colour export; pytest art/climate_colour/test_colour.py.
"""Deterministic HSV transport; no spatial filtering, resampling or alpha edits."""

from __future__ import annotations

import colorsys
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image

FloatPixels = NDArray[np.float64]
QUANTILES: Final = (0.0, 0.1, 0.5, 0.9, 1.0)


def triple(values: FloatPixels) -> tuple[float, float, float]:
    """Extract scalar channels without erasing NumPy element types."""
    return (
        float(np.sum(values[:1])),
        float(np.sum(values[1:2])),
        float(np.sum(values[2:3])),
    )


def hsv_pixels(source: Image.Image) -> FloatPixels:
    """Read opaque colour, cutting circular hue at cyan rather than at red."""
    rgba = np.asarray(source.convert("RGBA"), dtype=np.uint8)
    pixels = rgba[rgba[..., 3] >= 128, :3].astype(np.float64) / 255
    if not pixels.size:
        message = "Colour distribution requires opaque pixels (alpha >= 128)"
        raise ValueError(message)
    hsv: FloatPixels = np.array(
        [
            colorsys.rgb_to_hsv(*triple(pixels[index : index + 1].reshape(3)))
            for index in range(len(pixels))
        ],
        dtype=np.float64,
    )
    hsv[:, 0] = (hsv[:, 0] + 0.5) % 1 - 0.5
    return hsv


def distribution(source: Image.Image) -> tuple[tuple[float, ...], ...]:
    """Return H/S/V q10, median, q90; hue is in turns in [-0.5, 0.5)."""
    values = np.quantile(hsv_pixels(source), (0.1, 0.5, 0.9), axis=0)
    return tuple(
        tuple(
            float(np.sum(values[index : index + 1, channel : channel + 1]))
            for index in range(3)
        )
        for channel in range(3)
    )


def recolour(source: Image.Image, reference: Image.Image) -> Image.Image:
    """Transport HSV marginal quantiles, retaining every pixel's location and alpha.

    Constant source channels map to the reference median. Nonconstant channels
    use monotone piecewise-linear knots; duplicate source knots share a target.
    shortcut: cyan-crossing palettes are outside this flora/earth family; use a
    family-specific hue origin before extending this transform to blue assets.
    """
    source_hsv, reference_hsv = hsv_pixels(source), hsv_pixels(reference)
    rgba = np.array(source.convert("RGBA"), dtype=np.uint8)
    if source.size == reference.size and np.array_equal(
        rgba, np.asarray(reference.convert("RGBA"))
    ):
        return source.copy()
    visible = rgba[..., 3] > 0
    values = rgba[visible, :3].astype(np.float64) / 255
    hsv: FloatPixels = np.array(
        [
            colorsys.rgb_to_hsv(*triple(values[index : index + 1].reshape(3)))
            for index in range(len(values))
        ],
        dtype=np.float64,
    )
    hsv[:, 0] = (hsv[:, 0] + 0.5) % 1 - 0.5
    source_knots = np.quantile(source_hsv, QUANTILES, axis=0)
    target_knots = np.quantile(reference_hsv, QUANTILES, axis=0)
    for channel in range(3):
        knots, indices, counts = np.unique(
            source_knots[:, channel], return_index=True, return_counts=True
        )
        targets = np.add.reduceat(target_knots[:, channel], indices) / counts
        if len(knots) == 1:
            targets[0] = np.median(reference_hsv[:, channel])
        hsv[:, channel] = np.interp(hsv[:, channel], knots, targets)
    hsv[:, 0] %= 1
    rgb: FloatPixels = np.array(
        [
            colorsys.hsv_to_rgb(*triple(hsv[index : index + 1].reshape(3)))
            for index in range(len(hsv))
        ],
        dtype=np.float64,
    )
    rgba[visible, :3] = np.rint(rgb * 255).clip(0, 255).astype(np.uint8)
    return Image.fromarray(rgba)
