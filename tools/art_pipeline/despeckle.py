# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_despeckle.py
"""Single-pass isolated-texel correction with a strict broken-cell rejection cap."""
from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
from typing import Final

import numpy as np
from PIL import Image

from palette_tools import rgb_to_lab

TRANSPARENT: Final = (0, 0, 0, 0)


@dataclass(frozen=True, slots=True)
class DespeckleResult:
    """Corrected evidence is retained even when the change fraction rejects it."""
    image: Image.Image
    changed_count: int
    opaque_count: int
    change_fraction: float
    rejected: bool


def despeckle(image: Image.Image) -> DespeckleResult:
    """Replace rare isolated opaque colours using original four-neighbour votes."""
    source = np.asarray(image.convert('RGBA'))
    result = source.copy()
    opaque = source[..., 3] > 0
    opaque_count = int(opaque.sum())
    counts = Counter(tuple(int(v) for v in pixel) for pixel in source[opaque, :3])
    labs = rgb_to_lab(source[..., :3])
    changed = 0
    for y, x in zip(*np.nonzero(opaque)):
        color = tuple(int(v) for v in source[y, x, :3])
        if counts[color] > 2:
            continue
        neighbours = [(ny, nx) for ny in range(max(0, y - 1), min(image.height, y + 2))
                      for nx in range(max(0, x - 1), min(image.width, x + 2))
                      if (ny != y or nx != x) and opaque[ny, nx]]
        if any(np.linalg.norm(labs[ny, nx] - labs[y, x]) <= 12 for ny, nx in neighbours):
            continue
        votes = []
        for dy, dx in ((-1, 0), (0, -1), (0, 1), (1, 0)):
            ny, nx = y + dy, x + dx
            if 0 <= ny < image.height and 0 <= nx < image.width and opaque[ny, nx]:
                votes.append(tuple(int(v) for v in source[ny, nx]))
            else:
                votes.append(TRANSPARENT)
        tally = Counter(votes)
        # Transparency requires a strict majority; remaining ties are RGBA lexical.
        eligible = [value for value in tally if value != TRANSPARENT]
        replacement = (TRANSPARENT if tally[TRANSPARENT] > 2 or not eligible else
                       min(eligible, key=lambda value: (-tally[value], value)))
        result[y, x] = replacement
        changed += 1
    fraction = changed / opaque_count if opaque_count else 0.0
    return DespeckleResult(Image.fromarray(result), changed, opaque_count, fraction,
                           changed * 100 > opaque_count * 3)
