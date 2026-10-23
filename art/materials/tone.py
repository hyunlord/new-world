# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy", "pillow"]
# ///
# How to run: python3 -m pytest art/materials/test_material_tone.py -q.
"""Deterministic D65 CIELAB shifts and periodic top-surface tone transforms."""
from __future__ import annotations

from math import ceil
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image, ImageEnhance, ImageFilter

from art.materials.schema import LabTint, Top

FloatPixels = NDArray[np.float64]
RGB_TO_XYZ: Final = np.array([[.4124564, .3575761, .1804375],
                              [.2126729, .7151522, .0721750],
                              [.0193339, .1191920, .9503041]])
XYZ_TO_RGB: Final = np.linalg.inv(RGB_TO_XYZ)
WHITE: Final = np.array([.95047, 1., 1.08883])
DELTA: Final = 6 / 29


def rgb_to_lab(rgb: FloatPixels) -> FloatPixels:
    """Convert encoded sRGB in [0,1] to L* [0,100], a*, b* under D65."""
    linear = np.where(rgb <= .04045, rgb / 12.92, ((rgb + .055) / 1.055) ** 2.4)
    xyz = (linear @ RGB_TO_XYZ.T) / WHITE
    f = np.where(xyz > DELTA ** 3, np.cbrt(xyz), xyz / (3 * DELTA ** 2) + 4 / 29)
    return np.stack((116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]),
                     200 * (f[..., 1] - f[..., 2])), axis=-1)


def lab_to_rgb(lab: FloatPixels) -> FloatPixels:
    """Convert CIELAB to clipped encoded sRGB in [0,1]."""
    fy = (lab[..., 0] + 16) / 116
    f = np.stack((fy + lab[..., 1] / 500, fy, fy - lab[..., 2] / 200), axis=-1)
    xyz = np.where(f > DELTA, f ** 3, 3 * DELTA ** 2 * (f - 4 / 29)) * WHITE
    linear = np.clip(xyz @ XYZ_TO_RGB.T, 0, 1)
    return np.where(linear <= .0031308, linear * 12.92, 1.055 * linear ** (1 / 2.4) - .055)


def shift_image(source: Image.Image, tint: LabTint) -> Image.Image:
    """Apply one declared Lab shift; preserve source alpha byte-for-byte."""
    image = source.convert("RGBA")
    if tint.dL == 0 and tint.da == 0 and tint.db == 0:
        return image.copy()
    rgba = np.array(image, dtype=np.uint8)
    lab = rgb_to_lab(rgba[..., :3].astype(np.float64) / 255)
    shifted = lab + np.array([tint.dL, tint.da, tint.db])
    rgba[..., :3] = np.rint(lab_to_rgb(shifted) * 255).astype(np.uint8)
    return Image.fromarray(rgba)


def top_tone(source: Image.Image, top: Top) -> Image.Image:
    """Blur periodically in both axes, then apply declared saturation/value."""
    rgba = source.convert("RGBA")
    radius = ceil(top.blur_px * 3)
    if radius:
        pixels = np.asarray(rgba)
        padded = np.pad(pixels, ((radius, radius), (radius, radius), (0, 0)), mode="wrap")
        blurred = Image.fromarray(padded).filter(ImageFilter.GaussianBlur(top.blur_px))
        rgba = blurred.crop((radius, radius, radius + source.width, radius + source.height))
    saturated = ImageEnhance.Color(rgba).enhance(top.saturation)
    pixels = np.array(saturated, dtype=np.uint8)
    pixels[..., :3] = np.rint(pixels[..., :3].astype(np.float64) * top.value).astype(np.uint8)
    pixels[..., 3] = np.asarray(source.convert("RGBA"))[..., 3]
    return Image.fromarray(pixels)
