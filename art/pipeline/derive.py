"""Permitted deterministic blueprint, canopy-shadow and river-bank derivatives."""
from collections.abc import Mapping
from dataclasses import dataclass
from math import sqrt

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from .models import RGB, PipelineError
from .outline import OUTLINE, silhouette_boundary


@dataclass(frozen=True, slots=True)
class BankTiles:
    images: Mapping[str, Image.Image]

def hut_ghost(hut: Image.Image) -> Image.Image:
    """Blue alpha-80 inner fill and boundary dashes, separated by a 1px moat."""
    pixels: NDArray[np.uint8] = np.array(hut.convert('RGBA'))
    mask: NDArray[np.bool_] = pixels[:, :, 3] > 0
    if not np.any(mask):
        raise PipelineError('Blueprint source has no silhouette')
    boundary = silhouette_boundary(mask)
    coordinates: NDArray[np.int64] = np.indices(mask.shape, dtype=np.int64)
    y, x = coordinates[0, :, :], coordinates[1, :, :]
    interior = mask & ~boundary
    moat = silhouette_boundary(interior)
    dashes = boundary & np.equal(((x + y) // 3) % 2, 0)
    visible = (interior & ~moat) | dashes
    output = np.zeros_like(pixels)
    output[visible] = (111, 176, 232, 80)
    return Image.fromarray(output)

def canopy_shadow(canopy: Image.Image) -> Image.Image:
    """Centered ellipse at 60% area of source alpha-bounds ellipse, alpha 90.

    Both radii scale by sqrt(0.6), not by 0.6. Pixel-center membership is exact
    for the declared ellipse; small pixel grids approximate its analytic area.
    """
    bbox = canopy.convert('RGBA').getbbox()
    if bbox is None:
        raise PipelineError('Canopy source has no silhouette')
    left, top, right, bottom = bbox
    cx, cy = (left + right) / 2, (top + bottom) / 2
    rx, ry = (right - left) * sqrt(0.6) / 2, (bottom - top) * sqrt(0.6) / 2
    coordinates: NDArray[np.int64] = np.indices((canopy.height, canopy.width), dtype=np.int64)
    y, x = coordinates[0, :, :], coordinates[1, :, :]
    mask = ((x + 0.5 - cx) / rx) ** 2 + ((y + 0.5 - cy) / ry) ** 2 <= 1
    output = np.zeros((canopy.height, canopy.width, 4), dtype=np.uint8)
    output[mask] = (*OUTLINE, 90)
    return Image.fromarray(output)

def bank_tiles(water: Image.Image, sand: RGB) -> BankTiles:
    """Overlay two-pixel straight lips and concave sand corners onto real water.

    Only geometry masks rotate. Source water pixels remain registered and are
    never interpolated. Corner sand occupies a 6px corner block outside a 4px
    circular bite centered at its inner corner, creating a concave water edge.
    """
    if water.size != (32, 32):
        raise PipelineError('Bank source must be a registered 32x32 water tile')
    pixels: NDArray[np.uint8] = np.array(water.convert('RGBA'))
    if np.any(np.not_equal(pixels[:, :, 3], 255)):
        raise PipelineError('Bank source water must be fully opaque')
    if any(channel < 0 or channel > 255 for channel in sand):
        raise PipelineError('Sand RGB channels must lie in 0..255')
    y, x = np.indices((32, 32))
    straight = y < 2
    corner = (x < 6) & (y < 6) & ((x + 0.5 - 6) ** 2 + (y + 0.5 - 6) ** 2 >= 16)
    images: dict[str, Image.Image] = {}
    for names, mask in ((('bank_n', 'bank_w', 'bank_s', 'bank_e'), straight),
                        (('bank_in_nw', 'bank_in_sw', 'bank_in_se', 'bank_in_ne'), corner)):
        for rotation, name in enumerate(names):
            output = pixels.copy()
            output[np.rot90(mask, rotation)] = (*sand, 255)
            images[name] = Image.fromarray(output)
    return BankTiles(images)
