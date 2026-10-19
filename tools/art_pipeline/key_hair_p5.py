# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: imported by key_candidates_p5.py; pytest tools/art_pipeline/test_key_hair_p5.py
"""Extract exact reserved hair texels before natural-colour substitution."""
from __future__ import annotations

from collections import deque
from colorsys import rgb_to_hsv
from dataclasses import dataclass
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from despeckle import DespeckleResult, despeckle
from palette_tools import Color, rgb_to_lab

KEYS: Final = ((0, 220, 210), (0, 170, 162), (0, 125, 118))
NEIGHBOURS: Final = tuple((dy, dx) for dy in (-1, 0, 1) for dx in (-1, 0, 1) if dy or dx)


@dataclass(frozen=True, slots=True)
class KeyResult:
    """Both pre-substitution evidence and final natural pixels remain inspectable."""
    keyed: Image.Image
    natural: Image.Image
    mask: Image.Image
    errors: tuple[str, ...]
    key_count: int
    outside_count: int
    fringe_count: int
    delta_before: float
    delta_after: float
    cleaned: DespeckleResult


def key_pixels(rgba: NDArray[np.uint8]) -> NDArray[np.bool_]:
    """Find opaque exact members of the reserved key palette."""
    return (rgba[..., 3] > 0) & np.any(np.all(rgba[..., None, :3] == np.asarray(KEYS), axis=-1), axis=-1)


def protected_despeckle(image: Image.Image) -> DespeckleResult:
    """Restore protected source key texels and count only actual final changes."""
    source = np.asarray(image.convert('RGBA'))
    result = np.array(despeckle(image).image)
    protected = key_pixels(source)
    result[protected] = source[protected]
    # A neighbour vote cannot manufacture a new hair texel outside the key source.
    introduced = key_pixels(result) & ~protected
    result[introduced] = source[introduced]
    changed = int(np.any(result != source, axis=-1).sum())
    opaque = int((source[..., 3] > 0).sum())
    fraction = changed / max(1, opaque)
    return DespeckleResult(Image.fromarray(result), changed, opaque, fraction, fraction > .10)


def largest_component(mask: NDArray[np.bool_]) -> NDArray[np.bool_]:
    """Return largest eight-connected component, breaking ties by top-left seed."""
    remaining = mask.copy()
    best: list[tuple[int, int]] = []
    for y, x in np.argwhere(mask):
        if not remaining[y, x]:
            continue
        queue = deque([(int(y), int(x))])
        remaining[y, x] = False
        component: list[tuple[int, int]] = []
        while queue:
            point = queue.popleft()
            component.append(point)
            for dy, dx in NEIGHBOURS:
                ny, nx = point[0] + dy, point[1] + dx
                if 0 <= ny < mask.shape[0] and 0 <= nx < mask.shape[1] and remaining[ny, nx]:
                    remaining[ny, nx] = False
                    queue.append((ny, nx))
        if len(component) > len(best):
            best = component
    result = np.zeros_like(mask)
    for y, x in best:
        result[y, x] = True
    return result


def delta_e(sample: Image.Image, image: Image.Image) -> float:
    """Measure only common opaque source/native texels."""
    before, after = np.asarray(sample), np.asarray(image)
    common = (before[..., 3] == 255) & (after[..., 3] == 255)
    distances = np.linalg.norm(rgb_to_lab(before[common, :3]) - rgb_to_lab(after[common, :3]), axis=1)
    return float(distances.mean()) if len(distances) else 100.0


def convert_key(native: Image.Image, sample: Image.Image, browns: tuple[Color, Color, Color], *, asleep: bool = False) -> KeyResult:
    """Protect, validate and encode exact keys; ranking always uses keyed pixels."""
    cleaned = protected_despeckle(native)
    rgba = np.array(cleaned.image)
    key = key_pixels(rgba)
    component = largest_component(key)
    outside = int((key & ~component).sum())
    opaque_rows = np.flatnonzero(np.any(rgba[..., 3] > 0, axis=1))
    below = False
    if len(opaque_rows):
        cutoff = int(opaque_rows[0]) + round(.80 * (int(opaque_rows[-1] - opaque_rows[0]) + 1))
        below = bool(key[cutoff + 1:].any())
    adjacent = np.zeros_like(key)
    padded = np.pad(component, 1)
    for dy, dx in NEIGHBOURS:
        adjacent |= padded[1 + dy:1 + dy + key.shape[0], 1 + dx:1 + dx + key.shape[1]]
    fringe = 0
    for y, x in np.argwhere(adjacent & ~key & (rgba[..., 3] > 0)):
        hue, saturation, _ = rgb_to_hsv(*(float(c) / 255 for c in rgba[y, x, :3]))
        fringe += int(140 <= hue * 360 <= 200 and saturation >= .20)
    errors: list[str] = []
    if int(key.sum()) < 6 and not asleep:
        errors.append('key_hair_missing')
    if outside > 2 or below:
        errors.append('key_hair_outside_head')
    if fringe > 2:
        errors.append('key_fringe')
    mask = np.zeros_like(rgba)
    mask[..., 3] = rgba[..., 3]
    natural = rgba.copy()
    for shade, (color, brown) in enumerate(zip(KEYS, browns)):
        selected = key & np.all(rgba[..., :3] == color, axis=-1)
        mask[selected, 0] = 3
        mask[selected, 1] = shade
        natural[selected, :3] = brown
    image = Image.fromarray(natural)
    return KeyResult(cleaned.image, image, Image.fromarray(mask), tuple(errors), int(key.sum()), outside,
                     fringe, delta_e(sample, cleaned.image), delta_e(sample, image), cleaned)


def outlined_mask(mask: Image.Image, sprite: Image.Image) -> Image.Image:
    """Outline is a one-texel pad; never turn outline pixels into hair."""
    result = np.zeros((sprite.height, sprite.width, 4), dtype=np.uint8)
    source = np.asarray(mask)
    result[1:1 + mask.height, 1:1 + mask.width, :3] = source[..., :3]
    result[..., 3] = np.asarray(sprite)[..., 3]
    return Image.fromarray(result)
