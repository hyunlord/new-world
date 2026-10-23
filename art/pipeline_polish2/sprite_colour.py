# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m pytest art/pipeline_polish2/test_sprite_variants.py -q.
"""D65 CIELAB transforms that preserve source alpha and all unselected bytes."""
from __future__ import annotations

from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image

FloatPixels = NDArray[np.float64]
BytePixels = NDArray[np.uint8]
BoolPixels = NDArray[np.bool_]
IntPixels = NDArray[np.intp]


def lab_triple(values: FloatPixels) -> tuple[float, float, float]:
    return float(np.sum(values[:1])), float(np.sum(values[1:2])), float(np.sum(values[2:3]))


def squared_distance(values: FloatPixels) -> FloatPixels:
    result = np.empty(values.shape[:-1], dtype=np.float64)
    _ = np.sum(values ** 2, axis=-1, out=result)
    return result


def nearest_label(distances: FloatPixels) -> IntPixels:
    labels = np.empty(distances.shape[:-1], dtype=np.intp)
    _ = np.argmin(distances, axis=-1, out=labels)
    return labels


def mean_colour(values: FloatPixels) -> FloatPixels:
    result = np.empty((3,), dtype=np.float64)
    _ = np.mean(values, axis=0, out=result)
    return result


RGB_TO_XYZ: Final = np.array(((.4124564, .3575761, .1804375), (.2126729, .7151522, .0721750), (.0193339, .1191920, .9503041)))
WHITE: Final = np.array((.95047, 1., 1.08883))


def rgb_to_lab(rgb: FloatPixels) -> FloatPixels:
    """Convert encoded sRGB 0..255 to ordinary CIELAB units (D65)."""
    encoded = rgb / 255.
    linear = np.where(encoded <= .04045, encoded / 12.92, ((encoded + .055) / 1.055) ** 2.4)
    xyz = (linear @ RGB_TO_XYZ.T) / WHITE
    f = np.where(xyz > (6 / 29) ** 3, np.cbrt(xyz), xyz / (3 * (6 / 29) ** 2) + 4 / 29)
    return np.stack((116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]), 200 * (f[..., 1] - f[..., 2])), axis=-1)


def lab_to_linear_rgb(lab: FloatPixels) -> FloatPixels:
    """Return unclipped linear sRGB so callers can count gamut loss."""
    fy = (lab[..., 0] + 16) / 116
    f = np.stack((fy + lab[..., 1] / 500, fy, fy - lab[..., 2] / 200), axis=-1)
    xyz = np.where(f > 6 / 29, f ** 3, 3 * (6 / 29) ** 2 * (f - 4 / 29)) * WHITE
    return xyz @ np.linalg.inv(RGB_TO_XYZ).T


def lab_to_rgb(lab: FloatPixels) -> BytePixels:
    """Convert CIELAB to encoded sRGB with deterministic gamut clipping."""
    linear = np.clip(lab_to_linear_rgb(lab), 0, 1)
    encoded = np.where(linear <= .0031308, linear * 12.92, 1.055 * linear ** (1 / 2.4) - .055)
    return np.rint(np.clip(encoded, 0, 1) * 255).astype(np.uint8)


def shifted(source: Image.Image, mask: BoolPixels, delta: tuple[float, float, float]) -> Image.Image:
    """Write only selected RGB, never alpha, geometry, outline or other pixels."""
    rgba = np.array(source.convert("RGBA"), dtype=np.uint8)
    if mask.shape != rgba.shape[:2]:
        message = "Mask dimensions must equal source dimensions"
        raise ValueError(message)
    rgba[mask, :3] = lab_to_rgb(rgb_to_lab(rgba[mask, :3].astype(np.float64)) + np.array(delta))
    return Image.fromarray(rgba)
