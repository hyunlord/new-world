# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: import transform_raw from art.batch_r1.retry in the retry recorder.
"""Uniform raw-sheet colour correction; callers record gains and review skin/hair."""

from __future__ import annotations

import math
from pathlib import Path

import numpy as np
from numpy.typing import NDArray
from PIL import Image


def transform_raw(
    source: Path,
    destination: Path,
    *,
    saturation_gain: float = 1.0,
    value_gain: float = 1.0,
    ink_gain: float = 1.0,
    ink_cutoff: float = 0.25,
    value_offset: float = 0.0,
) -> None:
    """Save a new RGBA sheet with spatially uniform HSV gains and unchanged alpha.

    Gain 1 is neutral. Ink gain scales V, blending quadratically below ink_cutoff;
    ordinary value gain fades out in that range. Transparent and near-white
    pixels retain their RGB. Value offset is added after gains; black becomes
    grayscale when lifted. No garment segmentation or fixed probes are used.
    The three independent gains are explicit to make recorded retries replayable.
    """
    if any(
        not math.isfinite(gain) or gain <= 0
        for gain in (saturation_gain, value_gain, ink_gain)
    ):
        message = "colour gains must be finite and positive"
        raise ValueError(message)
    if not math.isfinite(ink_cutoff) or not 0 < ink_cutoff <= 1:
        message = "ink cutoff must be finite and within (0, 1]"
        raise ValueError(message)
    if not math.isfinite(value_offset) or not -1 <= value_offset <= 1:
        message = "value offset must be finite and within [-1, 1]"
        raise ValueError(message)
    if source.resolve() == destination.resolve():
        message = "raw correction requires a separate destination"
        raise ValueError(message)
    with Image.open(source) as image:
        if image.mode != "RGBA" or image.size != (1536, 1024):
            message = "raw correction requires a 1536x1024 RGBA sheet"
            raise ValueError(message)
        pixels: NDArray[np.uint8] = np.array(image)
    if (saturation_gain, value_gain, ink_gain, value_offset) != (1.0, 1.0, 1.0, 0.0):
        rgb: NDArray[np.float64] = pixels[:, :, :3].astype(np.float64) / 255.0
        maximum: NDArray[np.float64] = np.maximum(
            np.maximum(rgb[:, :, 0], rgb[:, :, 1]), rgb[:, :, 2]
        )
        minimum: NDArray[np.float64] = np.minimum(
            np.minimum(rgb[:, :, 0], rgb[:, :, 1]), rgb[:, :, 2]
        )
        chroma: NDArray[np.float64] = maximum - minimum
        saturation: NDArray[np.float64] = np.divide(
            chroma, maximum, out=np.zeros_like(chroma), where=maximum > 0
        )
        scaled_saturation: NDArray[np.float64] = np.minimum(
            saturation * saturation_gain, 1.0
        )
        chroma_gain: NDArray[np.float64] = np.divide(
            scaled_saturation,
            saturation,
            out=np.ones_like(saturation),
            where=saturation > 0,
        )
        corrected: NDArray[np.float64] = (
            maximum[:, :, None] - (maximum[:, :, None] - rgb) * chroma_gain[:, :, None]
        )
        ink_weight: NDArray[np.float64] = (
            np.maximum(1.0 - maximum / ink_cutoff, 0.0) ** 2
        )
        brightness_gain: NDArray[np.float64] = (
            value_gain * (1.0 - ink_weight) + ink_gain * ink_weight
        )
        scaled_value: NDArray[np.float64] = np.clip(
            maximum * brightness_gain + value_offset, 0.0, 1.0
        )
        value_scale: NDArray[np.float64] = np.divide(
            scaled_value, maximum, out=np.ones_like(maximum), where=maximum > 0
        )
        corrected *= value_scale[:, :, None]
        corrected[maximum == 0] = scaled_value[maximum == 0, None]
        active: NDArray[np.bool_] = (pixels[:, :, 3] > 0) & (minimum < 245 / 255)
        pixels[:, :, :3][active] = np.rint(
            np.clip(corrected[active], 0.0, 1.0) * 255
        ).astype(np.uint8)
    with destination.open("xb") as output:
        Image.fromarray(pixels).save(output, format="PNG")
