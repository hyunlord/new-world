# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy"]
# ///
# How to run: imported by python -m art.pipeline5.sheet_slice
"""Lossless panel division and explicit standing/sleep Lanczos normalization."""
from dataclasses import dataclass
from itertools import pairwise
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from art.pipeline3.structure import components
from art.pipeline5.panel_ownership import (
    FragmentOwnership,
    OwnershipEvidence,
    extract_owned,
)
from art.pipeline5.whole_checks import ALPHA_THRESHOLD, VISIBILITY_THRESHOLD

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


def split_panels(image: Image.Image, rows: int = 1) -> Panels:
    """Preserve legacy one-row splitting; PROD1 requires ten separate grid seeds."""
    if rows not in (1, 2):
        message = 'grid: rows must be 1 (legacy) or 2 (PROD1)'
        raise ValueError(message)
    alpha = np.asarray(image)[:, :, 3]
    mask = alpha >= ALPHA_THRESHOLD
    raw_groups = components(mask)
    count = len(raw_groups)
    dust_limit = image.width * image.height * DUST_AREA_FRACTION
    dust = [group for group in raw_groups if len(group) < dust_limit]
    groups: list[list[tuple[int, int]]] = [group for group in raw_groups if len(group) >= dust_limit]
    major_count = len(groups)
    cuts: list[int] = []
    if rows == 1 and major_count < 5:
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
    if len(groups) != 5 * rows:
        message = f'panel_count: {count} raw components, {major_count} major initially, {len(groups)} major after splits {cuts}; need exactly{5 * rows}'
        raise ValueError(message)
    if rows == 2:
        cells: dict[tuple[int, int], list[tuple[int, int]]] = {}
        for group in groups:
            cx = (min(x for x, _ in group) + max(x for x, _ in group)) / 2
            cy = (min(y for _, y in group) + max(y for _, y in group)) / 2
            cell = (min(1, int(cy * 2 / image.height)), min(4, int(cx * 5 / image.width)))
            if cell in cells:
                message = f'grid: multiple major figures in cell {cell}'
                raise ValueError(message)
            cells[cell] = group
        groups = [cells[(row, column)] for row in range(2) for column in range(5)]
    else:
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
    rotation_degrees: int = 0
    scale: float = 1.0
    discarded_pixels: int = 0
    discarded_max_alpha: int = 0
    visibility_threshold: int = VISIBILITY_THRESHOLD


@dataclass(frozen=True, slots=True)
class NormalizedPanel:
    image: Image.Image
    transform: Normalization


def sleep_body_scale(panel: Image.Image, target_length: int) -> float:
    """Calibrate the opaque body's long axis without trimming source alpha."""
    bounds = Image.fromarray(np.asarray(panel)[:, :, 3] >= ALPHA_THRESHOLD).getbbox()
    if bounds is None or target_length <= 0:
        message = 'normalization: sleep requires an opaque body and positive target length'
        raise ValueError(message)
    return target_length / max(bounds[2] - bounds[0], bounds[3] - bounds[1])


def normalize_panel(panel: Image.Image, scale: float, sleep: bool,
                    *, sleep_length: int | None = None) -> NormalizedPanel:
    """Register alpha16 extent; retain all RGBA within the canvas, logging clipped halos."""
    if sleep and sleep_length is not None:
        scale = sleep_body_scale(panel, sleep_length)
    oriented = panel
    rotation = 0
    if sleep:
        opaque = np.asarray(panel)[:, :, 3] >= ALPHA_THRESHOLD
        bounds = Image.fromarray(opaque).getbbox()
        if bounds is not None:
            alpha: NDArray[np.uint8] = np.asarray(panel.crop(bounds))[:, :, 3]
            vertical = bounds[3] - bounds[1] > bounds[2] - bounds[0]
            extent = bounds[3] - bounds[1] if vertical else bounds[2] - bounds[0]
            end = max(1, (extent * 3 + 9) // 10)
            first = alpha[:end, :].sum() if vertical else alpha[:, :end].sum()
            last = alpha[-end:, :].sum() if vertical else alpha[:, -end:].sum()
            if vertical:
                rotation = 90 if first >= last else 270
                operation = Image.Transpose.ROTATE_90 if rotation == 90 else Image.Transpose.ROTATE_270
                oriented = panel.transpose(operation)
            elif last > first:
                rotation = 180
                oriented = panel.transpose(Image.Transpose.ROTATE_180)
    size = (max(1, round(oriented.width*scale)), max(1, round(oriented.height*scale)))
    resized = oriented.resize(size, Image.Resampling.LANCZOS)
    alpha = np.asarray(resized)[:, :, 3]
    box = Image.fromarray(alpha >= VISIBILITY_THRESHOLD).getbbox()
    if box is None:
        message = 'normalization: no visible pixels after Lanczos'
        raise ValueError(message)
    width, height = box[2] - box[0], box[3] - box[1]
    canvas_size = (112, 80) if sleep else (80, 112)
    x = (canvas_size[0]-width)//2
    y = (72 if sleep else 104)-(height-1)
    if x < 1 or y < 1 or x+width >= canvas_size[0] or y+height >= canvas_size[1]:
        message = (f'normalization: visible extent {(width, height)} offset {(x,y)} would clip '
                   f'canvas {canvas_size}; scaled_size={size}, trim_box={box}')
        raise ValueError(message)
    dx, dy = x-box[0], y-box[1]
    discarded = alpha.copy()
    discarded[max(0, -dy):min(size[1], canvas_size[1]-dy),
              max(0, -dx):min(size[0], canvas_size[0]-dx)] = 0
    canvas = Image.new('RGBA', canvas_size)
    canvas.paste(resized, (dx, dy))
    transform = Normalization(size, box, (x, y), (dx, dy),
                              int(np.count_nonzero(alpha >= VISIBILITY_THRESHOLD)), rotation, scale,
                              int(np.count_nonzero(discarded)), int(discarded.max()))
    return NormalizedPanel(canvas, transform)


def normalize(panel: Image.Image, scale: float, sleep: bool) -> Image.Image:
    """Compatibility image-only interface for tests and image consumers."""
    return normalize_panel(panel, scale, sleep).image
