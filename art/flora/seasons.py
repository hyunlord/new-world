# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy", "pillow", "pydantic"]
# ///
# How to run: python3 -m pytest art/flora/test_seasons.py -q
"""Palette-cluster canopy masks and deterministic, alpha-preserving seasons."""
from __future__ import annotations

from pathlib import Path
from typing import ClassVar, Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image
from pydantic import BaseModel, ConfigDict

from art.flora.resolve_flora import ResolvedFlora
from art.flora.schema import SeasonTint, SnowCap
from art.materials.tone import lab_to_rgb, rgb_to_lab

BoolPixels = NDArray[np.bool_]
FloatPixels = NDArray[np.float64]
# These indices are the existing olive foliage judging colors, not bark.
LEGACY_FOLIAGE: Final = (9, 12, 15, 17, 18, 19)


class MaskPalette(BaseModel):
    """Only consume the declared foliage and retained original judging palette."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    rgb: tuple[tuple[int, int, int], ...]
    foliage_summer_mid: tuple[int, int, int]
    foliage_summer_light: tuple[int, int, int]
    foliage_summer_dark: tuple[int, int, int]


def read_palette(root: Path) -> MaskPalette:
    """Parse the project palette at the I/O boundary."""
    return MaskPalette.model_validate_json((root / "art/style05/palette_05.json").read_bytes())


def foliage_clusters(source: Image.Image, palette: MaskPalette) -> BoolPixels:
    """Nearest CIELAB palette clusters; legacy olive is recognized before repair.

    Each pixel joins its closest foliage, bark, or ink prototype. No spatial
    exclusion hides low foliage from the separately measured trunk-box gate.
    Dark ink (L* <= 24) is excluded even inside a green silhouette.
    """
    rgba = np.asarray(source.convert("RGBA"), dtype=np.uint8)
    foliage = [palette.rgb[index] for index in LEGACY_FOLIAGE]
    foliage.extend((palette.foliage_summer_mid, palette.foliage_summer_light, palette.foliage_summer_dark))
    other = [rgb for index, rgb in enumerate(palette.rgb) if index not in LEGACY_FOLIAGE]
    centers = rgb_to_lab(np.array(foliage + other, dtype=np.float64) / 255)
    lab = rgb_to_lab(rgba[..., :3].astype(np.float64) / 255)
    distance: FloatPixels = np.sum((lab[..., None, :] - centers) ** 2, axis=-1)
    labels: NDArray[np.int64] = np.argmin(distance, axis=-1)
    return (labels < len(foliage)) & (lab[..., 0] > 24) & (rgba[..., 3] > 0)


def canopy_mask(source: Image.Image, root: Path) -> Image.Image:
    """Return a binary L mask; do not change the canonical RGBA image."""
    selected = foliage_clusters(source, read_palette(root))
    return Image.fromarray(selected.astype(np.uint8) * 255)


def snow_layer(mask: Image.Image, cap: SnowCap) -> Image.Image:
    """Directional erosion from the top envelope, never internal lower holes.

    The white layer is separate from source alpha. Its alpha is floored to
    preserve the <=70 percent limit after 8-bit quantization (178/255).
    """
    support: BoolPixels = np.asarray(mask.convert("L"), dtype=np.uint8) > 0
    layer = np.zeros((*support.shape, 4), dtype=np.uint8)
    if cap.depth_px == 0 or cap.alpha == 0:
        return Image.fromarray(layer)
    top: NDArray[np.int64] = np.argmax(support, axis=0)
    ys: NDArray[np.int64] = np.arange(mask.height, dtype=np.int64)[:, None]
    band: BoolPixels = support & (ys < top[None, :] + cap.depth_px)
    layer[band, :3] = 255
    layer[band, 3] = int(cap.alpha * 255)
    return Image.fromarray(layer)


def apply_tint(source: Image.Image, mask: Image.Image, tint: SeasonTint) -> Image.Image:
    """Shift masked RGB in D65 Lab; preserve every unmasked and alpha byte."""
    if source.size != mask.size:
        message = f"Frame and canopy mask dimensions differ: {source.size}, {mask.size}"
        raise ValueError(message)
    rgba = np.array(source.convert("RGBA"), dtype=np.uint8)
    support = (np.asarray(mask.convert("L")) > 0) & (rgba[..., 3] > 0)
    if any(tint.delta_lab) and np.any(support):
        lab = rgb_to_lab(rgba[support, :3].astype(np.float64) / 255)
        shifted = lab + np.array(tint.delta_lab, dtype=np.float64)
        rgba[support, :3] = np.rint(lab_to_rgb(shifted) * 255).astype(np.uint8)
    snow = np.asarray(snow_layer(Image.fromarray(support.astype(np.uint8) * 255), tint.snow_cap))
    snowy = snow[..., 3] > 0
    weight = snow[snowy, 3:4].astype(np.float64) / 255
    rgba[snowy, :3] = np.rint(rgba[snowy, :3] * (1 - weight) + 255 * weight).astype(np.uint8)
    return Image.fromarray(rgba)


def render_frame(resolved: ResolvedFlora, repo_root: Path) -> Image.Image:
    """Load a resolved frame and its mask, then apply the data season record."""
    with Image.open(repo_root / resolved.path) as source, Image.open(repo_root / resolved.canopy_mask) as mask:
        if source.size != resolved.canvas:
            message = f"Declared canvas differs from PNG: {resolved.path}"
            raise ValueError(message)
        return apply_tint(source, mask, resolved.tint)
