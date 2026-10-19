"""Learn reviewed hair colours and grow head-connected hair-only region maps."""
from __future__ import annotations

from collections import Counter, deque
from collections.abc import Iterable
from dataclasses import dataclass
from typing import Final

import numpy as np
from PIL import Image

from palette_tools import rgb_to_lab

RGB = tuple[int, int, int]
LUMA: Final = (0.2126, 0.7152, 0.0722)
NEIGHBOURS: Final = tuple((dy, dx) for dy in (-1, 0, 1) for dx in (-1, 0, 1)
                         if dy or dx)


@dataclass(frozen=True, slots=True)
class HairPalette:
    """Body-specific training vocabulary; known non-hair colours remain excluded."""

    body: str
    seen_rgb: frozenset[RGB]
    hair_rgb: frozenset[RGB]


def build_palette(pairs: Iterable[tuple[Image.Image, Image.Image]], body: str) -> HairPalette:
    """Count opaque reviewed texels, selecting colours seen ≥3 times and ≥40% hair."""
    totals: Counter[RGB] = Counter()
    hair: Counter[RGB] = Counter()
    for image, reviewed in pairs:
        rgba = np.asarray(image.convert('RGBA'))
        regions = np.asarray(reviewed.convert('RGBA'))
        if image.size != reviewed.size:
            message = f'Hair training size mismatch: {image.size} != {reviewed.size}'
            raise ValueError(message)
        for y, x in np.argwhere(rgba[..., 3] > 0):
            color = (int(rgba[y, x, 0]), int(rgba[y, x, 1]), int(rgba[y, x, 2]))
            totals[color] += 1
            hair[color] += int(regions[y, x, 0] == 3)
    selected = frozenset(color for color, count in totals.items()
                         if count >= 3 and hair[color] * 5 >= count * 2)
    return HairPalette(body, frozenset(totals), selected)


def classify(image: Image.Image, palette: HairPalette, activity: str = '') -> Image.Image:
    """Encode connected hair in R=3/G=tercile shade, preserving source alpha."""
    rgba = np.asarray(image.convert('RGBA'))
    output = np.zeros_like(rgba)
    output[..., 3] = rgba[..., 3]
    opaque = rgba[..., 3] > 0
    points = np.argwhere(opaque)
    excluded = palette.body == 'child' or activity in ('asleep', 'lying')
    if excluded or not len(points) or not palette.hair_rgb:
        return Image.fromarray(output)
    top, left = points.min(axis=0)
    bottom, right = points.max(axis=0)
    height = int(bottom - top + 1)
    if right - left + 1 > height:
        return Image.fromarray(output)
    head_end = int(top) + round(0.34 * height)
    reach_end = int(top) + round(0.62 * height)
    hair_colors = np.asarray(sorted(palette.hair_rgb), dtype=np.float64)
    hair_labs = rgb_to_lab(hair_colors)
    accepted = set(palette.hair_rgb)
    for values in np.unique(rgba[opaque, :3], axis=0):
        color = (int(values[0]), int(values[1]), int(values[2]))
        if color in palette.seen_rgb:
            continue
        distance = np.linalg.norm(hair_labs - rgb_to_lab(values), axis=1)
        if float(distance.min()) <= 10.0:
            accepted.add(color)
    candidates = np.zeros_like(opaque)
    for color in accepted:
        candidates |= opaque & np.all(rgba[..., :3] == color, axis=-1)
    candidates[reach_end:] = False
    mask = candidates.copy()
    mask[head_end:] = False
    queue = deque((int(y), int(x)) for y, x in np.argwhere(mask))
    while queue:
        y, x = queue.popleft()
        for dy, dx in NEIGHBOURS:
            ny, nx = y + dy, x + dx
            if 0 <= ny < reach_end and 0 <= nx < image.width:
                if candidates[ny, nx] and not mask[ny, nx]:
                    mask[ny, nx] = True
                    queue.append((ny, nx))
    for _ in range(2):
        padded = np.pad(mask, 1)
        count = np.zeros_like(rgba[..., 0])
        for dy, dx in NEIGHBOURS:
            count += padded[1 + dy:1 + dy + image.height,
                            1 + dx:1 + dx + image.width]
        filled = opaque & (count >= 6)
        filled[reach_end:] = False
        mask |= filled
    lower, upper = np.quantile(hair_colors @ np.asarray(LUMA) / 255.0, (1 / 3, 2 / 3))
    luma = rgba[..., :3] @ np.asarray(LUMA) / 255.0
    shades = np.where(luma >= upper, 0, np.where(luma >= lower, 1, 2))
    output[..., 0][mask] = 3
    output[..., 1][mask] = shades[mask]
    return Image.fromarray(output)
