"""Vectorised sRGB (0..255) to CIE Lab conversion under D65 illumination."""
from __future__ import annotations

import numpy as np
from numpy.typing import NDArray


def rgb_to_lab(rgb: NDArray[np.uint8] | NDArray[np.float64]) -> NDArray[np.float64]:
    """Convert final-axis RGB triples to float64 Lab, preserving leading axes."""
    srgb = np.asarray(rgb, dtype=np.float64) / 255.0
    linear = np.where(srgb <= 0.04045, srgb / 12.92, ((srgb + 0.055) / 1.055) ** 2.4)
    matrix = np.array([[0.4124564, 0.3575761, 0.1804375],
                       [0.2126729, 0.7151522, 0.0721750],
                       [0.0193339, 0.1191920, 0.9503041]], dtype=np.float64)
    xyz = (linear @ matrix.T) / np.array([0.95047, 1.0, 1.08883])
    delta = 6.0 / 29.0
    f = np.where(xyz > delta ** 3, np.cbrt(xyz), xyz / (3 * delta ** 2) + 4 / 29)
    return np.stack((116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]),
                     200 * (f[..., 1] - f[..., 2])), axis=-1)


def squared_length(values: NDArray[np.float64], axis: int) -> NDArray[np.float64]:
    """Sum squared float64 coordinates along one axis with an explicit dtype."""
    return np.asarray(np.sum(np.square(values), axis=axis, dtype=np.float64), dtype=np.float64)


def vector_length(values: NDArray[np.float64], axis: int) -> NDArray[np.float64]:
    """Euclidean coordinate lengths preserving the float64 array contract."""
    return np.sqrt(squared_length(values, axis))
