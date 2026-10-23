# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by art/compositor/render.py; tested with test_tone.py.
"""Measured ground restraint and declared condition transforms for ENV1.

Luminance means Rec.709 weights applied to encoded sRGB, matching the existing
art pipeline. These are rendering transforms, never edits to source art.
"""
from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path
from typing import Final, assert_never

import numpy as np
from numpy.typing import NDArray
from PIL import Image

RGB: Final = np.array([0.2126, 0.7152, 0.0722], dtype=np.float64)
FloatPixels = NDArray[np.float64]


class Condition(StrEnum):
    DAY = "day"
    NIGHT = "night"
    WINTER = "winter"
    DROUGHT = "drought"


@dataclass(frozen=True, slots=True)
class ToneMetrics:
    luminance_mean: float
    luminance_std: float
    saturation_mean: float


@dataclass(frozen=True, slots=True)
class GroundLimit:
    person_luminance_std: float
    mean_max: float = 0.40
    std_ratio: float = 0.60
    saturation_max: float = 0.30


@dataclass(frozen=True, slots=True)
class ToneResult:
    image: Image.Image
    before: ToneMetrics
    after: ToneMetrics
    contrast_factor: float
    chroma_factor: float
    passed: bool


def metrics(pixels: FloatPixels) -> ToneMetrics:
    """Measure normalized RGB pixels, with no alpha/background dilution."""
    luminance = pixels @ RGB
    maximum = pixels.max(axis=-1)
    minimum = pixels.min(axis=-1)
    saturation = np.divide(maximum - minimum, maximum,
                           out=np.zeros_like(maximum), where=maximum > 0)
    return ToneMetrics(float(luminance.mean()), float(luminance.std()),
                       float(saturation.mean()))


def person_limit(anchor: Path) -> GroundLimit:
    """Use alpha/chroma foreground of the locked person anchor at native size."""
    with Image.open(anchor) as source:
        rgba = np.asarray(source.convert("RGBA"), dtype=np.float64) / 255.0
    rgb = rgba[:, :, :3]
    magenta = ((rgb[:, :, 0] > .65) & (rgb[:, :, 2] > .65)
               & (rgb[:, :, 1] < .4)
               & (np.abs(rgb[:, :, 0] - rgb[:, :, 2]) < .3))
    foreground = rgb[(rgba[:, :, 3] >= .5) & ~magenta]
    if foreground.size == 0:
        message = f"No person foreground in {anchor}"
        raise ValueError(message)
    return GroundLimit(metrics(foreground).luminance_std)


def cap_ground(source: Image.Image, limit: GroundLimit) -> ToneResult:
    """Apply one global contrast affine and one global chroma multiplier.

    The target leaves rounding margin below all limits. A monotone binary
    search selects one tile-wide chroma value; no local repainting occurs.
    """
    rgb = np.asarray(source.convert("RGB"), dtype=np.float64) / 255.0
    before = metrics(rgb)
    gray = rgb @ RGB
    target_mean = min(before.luminance_mean, limit.mean_max * .90)
    target_std = limit.person_luminance_std * limit.std_ratio * .90
    contrast = min(1.0, target_std / max(before.luminance_std, 1e-12))
    quiet_gray = np.clip((gray - gray.mean()) * contrast + target_mean, 0, 1)
    chroma = (rgb - gray[:, :, None]) * contrast
    low, high = 0.0, 1.0
    for _ in range(24):
        factor = (low + high) * .5
        candidate = np.clip(quiet_gray[:, :, None] + chroma * factor, 0, 1)
        if metrics(candidate).saturation_mean <= limit.saturation_max * .90:
            low = factor
        else:
            high = factor
    final = np.clip(quiet_gray[:, :, None] + chroma * low, 0, 1)
    image = Image.fromarray(np.rint(final * 255).astype(np.uint8))
    after = metrics(np.asarray(image, dtype=np.float64) / 255.0)
    passed = (after.luminance_mean <= limit.mean_max
              and after.luminance_std <= limit.person_luminance_std * limit.std_ratio
              and after.saturation_mean <= limit.saturation_max)
    return ToneResult(image, before, after, contrast, low, passed)


def season_ground(source: Image.Image, condition: Condition) -> Image.Image:
    """Only ground changes hue in winter/drought; foreground keeps its colours."""
    rgb = np.asarray(source.convert("RGB"), dtype=np.float64) / 255.0
    gray = (rgb @ RGB)[:, :, None]
    match condition:
        case Condition.DAY:
            result = rgb
        case Condition.NIGHT:
            result = rgb * np.array([.86, .94, 1.08])
        case Condition.WINTER:
            result = gray + (rgb - gray) * .35
        case Condition.DROUGHT:
            result = rgb * .65 + gray * np.array([1.25, 1.04, .65]) * .35
        case _:
            assert_never(condition)
    return Image.fromarray(np.rint(np.clip(result, 0, 1) * 255).astype(np.uint8))


@dataclass(frozen=True, slots=True)
class NightLight:
    hearth_pixels: tuple[tuple[float, float], ...]
    tile_pixels: int
    radius_tiles: float = 5.0


def night_value(source: Image.Image, light: NightLight) -> Image.Image:
    """Drop value by 45%; restore it linearly inside five hearth tiles.

    Apply before pasting the unmodified HUD. Same multiplier on RGB preserves
    people/item/building hues; ground blue shift happens in season_ground.
    """
    rgba: FloatPixels = np.asarray(source.convert("RGBA"), dtype=np.float64)
    yy = np.arange(source.height, dtype=np.float64).reshape(-1, 1)
    xx = np.arange(source.width, dtype=np.float64).reshape(1, -1)
    restore = np.zeros(rgba.shape[:2], dtype=np.float64)
    for x, y in light.hearth_pixels:
        distance = ((xx - x) ** 2 + (yy - y) ** 2) ** .5
        restore = np.maximum(restore, np.clip(1 - distance /
                             (light.tile_pixels * light.radius_tiles), 0, 1))
    rgba[:, :, :3] *= (.55 + .45 * restore)[:, :, None]
    return Image.fromarray(np.rint(rgba).astype(np.uint8))
