"""CIELAB D65 palette extraction and color matching."""
from __future__ import annotations

import colorsys
from pathlib import Path

import numpy as np
from numpy.typing import NDArray
from PIL import Image

Color = tuple[int, int, int]


def rgb_to_lab(rgb: NDArray[np.generic]) -> NDArray[np.float64]:
    """Convert 0..255 sRGB to CIELAB with the D65 reference white."""
    srgb = np.asarray(rgb, dtype=np.float64) / 255.0
    linear = np.where(srgb <= 0.04045, srgb / 12.92, ((srgb + 0.055) / 1.055) ** 2.4)
    xyz = linear @ np.array([[0.4124564, 0.3575761, 0.1804375],
                             [0.2126729, 0.7151522, 0.0721750],
                             [0.0193339, 0.1191920, 0.9503041]]).T
    ratio = xyz / np.array([0.95047, 1.0, 1.08883])
    delta = 6.0 / 29.0
    f = np.where(ratio > delta**3, np.cbrt(ratio), ratio / (3 * delta**2) + 4 / 29)
    return np.stack([116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]),
                     200 * (f[..., 1] - f[..., 2])], axis=-1)


def extract_palette(image: Image.Image, count: int) -> list[Color]:
    """Median-cut, merge DeltaE76 < 3 clusters, then hue/lightness sort."""
    quantized = image.convert("RGB").quantize(colors=count, method=Image.Quantize.MEDIANCUT)
    palette = quantized.getpalette()
    counts = quantized.getcolors()
    if palette is None or counts is None:
        raise RuntimeError("Pillow did not return a quantized palette")
    weighted = [(weight, np.array(palette[index * 3:index * 3 + 3], dtype=np.float64))
                for weight, index in counts]
    while len(weighted) > 1:
        labs = rgb_to_lab(np.array([color for _, color in weighted]))
        distances = np.linalg.norm(labs[:, None] - labs[None, :], axis=-1)
        distances[np.tril_indices(len(weighted))] = np.inf
        a, b = np.unravel_index(np.argmin(distances), distances.shape)
        if distances[a, b] >= 3:
            break
        wa, ca = weighted[a]
        wb, cb = weighted[b]
        merged = (wa + wb, np.rint((ca * wa + cb * wb) / (wa + wb)))
        weighted = [entry for i, entry in enumerate(weighted) if i not in (a, b)] + [merged]
    colors = [(int(c[0]), int(c[1]), int(c[2])) for _, c in weighted]
    return sorted(colors, key=lambda c: (colorsys.rgb_to_hsv(*(v / 255 for v in c))[0],
                                        float(rgb_to_lab(np.array(c))[0])))


def load_palette(path: Path) -> list[Color]:
    """Read one #RRGGBB value per nonempty line."""
    colors: list[Color] = []
    for line in path.read_text().splitlines():
        value = line.strip().removeprefix("#")
        if value:
            if len(value) != 6:
                raise ValueError(f"Invalid palette color: {line}")
            colors.append((int(value[0:2], 16), int(value[2:4], 16), int(value[4:6], 16)))
    if not colors:
        raise ValueError(f"Empty palette: {path}")
    return colors


def save_palette(colors: list[Color], path: Path) -> None:
    """Write the palette as canonical hex lines."""
    path.write_text("".join(f"#{r:02X}{g:02X}{b:02X}\n" for r, g, b in colors))
