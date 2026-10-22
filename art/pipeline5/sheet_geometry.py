# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy"]
# ///
# How to run: imported by python -m art.pipeline5.sheet_slice
"""Lossless panel division and one idle-derived Lanczos normalization."""
from dataclasses import dataclass
from itertools import pairwise
from typing import Final

import numpy as np
from PIL import Image

from art.pipeline3.structure import components
from art.pipeline5.panel_ownership import (
    FragmentOwnership,
    OwnershipEvidence,
    extract_owned,
)
from art.pipeline5.whole_checks import ALPHA_THRESHOLD

DUST_AREA_FRACTION: Final = 0.0001


@dataclass(frozen=True, slots=True)
class Panels:
    images: tuple[Image.Image, ...]
    boxes: tuple[tuple[int, int, int, int], ...]
    split_columns: tuple[int, ...]
    original_components: int
    original_major_components: int
    dust_area_limit: float
    attached_fragments: tuple[FragmentOwnership, ...]
    idle_height: int
    ownership: OwnershipEvidence


def split_panels(image: Image.Image) -> Panels:
    """Find five major components; retain every visible source pixel by ownership."""
    alpha = np.asarray(image)[:, :, 3]
    mask = alpha >= ALPHA_THRESHOLD
    raw_groups = components(mask)
    count = len(raw_groups)
    dust_limit = image.width * image.height * DUST_AREA_FRACTION
    dust = [group for group in raw_groups if len(group) < dust_limit]
    groups = [group for group in raw_groups if len(group) >= dust_limit]
    major_count = len(groups)
    cuts: list[int] = []
    if major_count < 5:
        width = image.width
        for boundary in range(1, 5):
            expected = width * boundary / 5
            crossing = any(min(x for x, _ in group) < expected <= max(x for x, _ in group)
                           for group in groups)
            if crossing:
                lo = max(1, round(expected - width / 50))
                hi = min(width - 1, round(expected + width / 50))
                counts = mask[:, lo:hi + 1].sum(axis=0)
                cut = min(range(lo, hi + 1), key=lambda x: (counts[x-lo], abs(x-expected), x))
                cuts.append(cut)
        edges = [0, *cuts, image.width]
        groups = []
        for left, right in pairwise(edges):
            groups.extend([[(x + left, y) for x, y in group]
                           for group in components(mask[:, left:right]) if len(group) >= dust_limit])
    if len(groups) != 5:
        message = f'panel_count: {count} raw components, {major_count} major initially, {len(groups)} major after splits {cuts}; need exactly5'
        raise ValueError(message)
    groups.sort(key=lambda group: min(x for x, _ in group))
    owned = extract_owned(image, groups, dust)
    idle_height = max(y for _, y in groups[0])-min(y for _, y in groups[0])+1
    return Panels(owned.images, owned.boxes, tuple(cuts), count, major_count,
                  dust_limit, owned.evidence.fragments, idle_height, owned.evidence)


@dataclass(frozen=True, slots=True)
class Normalization:
    scaled_size: tuple[int, int]
    trim_box: tuple[int, int, int, int]
    placement: tuple[int, int]
    resized_to_canvas_offset: tuple[int, int]
    visible_pixels: int


@dataclass(frozen=True, slots=True)
class NormalizedPanel:
    image: Image.Image
    transform: Normalization


def normalize_panel(panel: Image.Image, scale: float, sleep: bool) -> NormalizedPanel:
    """Trim only zero-alpha post-Lanczos padding, then register visible content."""
    size = (max(1, round(panel.width*scale)), max(1, round(panel.height*scale)))
    resized = panel.resize(size, Image.Resampling.LANCZOS)
    box = resized.getchannel('A').getbbox()
    if box is None:
        message = 'normalization: no visible pixels after Lanczos'
        raise ValueError(message)
    trimmed = resized.crop(box)
    alpha = np.asarray(trimmed)[:, :, 3]
    opaque_y, _ = np.nonzero(alpha >= ALPHA_THRESHOLD)
    if not len(opaque_y):
        message = 'normalization: no alpha128 contact pixels'
        raise ValueError(message)
    canvas_size = (96, 64) if sleep else (64, 96)
    x = (canvas_size[0]-trimmed.width)//2
    y = (56 if sleep else 88)-int(opaque_y.max())
    if x < 1 or y < 1 or x+trimmed.width >= canvas_size[0] or y+trimmed.height >= canvas_size[1]:
        message = (f'normalization: visible extent {trimmed.size} offset {(x,y)} would clip '
                   f'canvas {canvas_size}; scaled_size={size}, trim_box={box}')
        raise ValueError(message)
    canvas = Image.new('RGBA', canvas_size)
    canvas.paste(trimmed, (x, y))
    transform = Normalization(size, box, (x, y), (x-box[0], y-box[1]), int(np.count_nonzero(alpha)))
    return NormalizedPanel(canvas, transform)


def normalize(panel: Image.Image, scale: float, sleep: bool) -> Image.Image:
    """Compatibility image-only interface for tests and image consumers."""
    return normalize_panel(panel, scale, sleep).image
