# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_hair_masks_v5.py
"""Align reviewed hair templates by head silhouette, never by prop colour alone."""
from __future__ import annotations

from collections import Counter, deque
from dataclasses import dataclass
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from hair_maps import LUMA, NEIGHBOURS, RGB

OUTLINE: Final = (3, 0, 0)


@dataclass(frozen=True, slots=True)
class Reference:
    """An independently approved outlined sprite and its exact hair map."""
    ident: str
    body: str
    view: str
    activity: str
    sprite: Image.Image
    mask: Image.Image


@dataclass(frozen=True, slots=True)
class Result:
    """A proposed map or an explicit failure to find sufficient head overlap."""
    mask: Image.Image | None
    reference_id: str
    iou: float
    shift: tuple[int, int]


def head_band(image: Image.Image) -> NDArray[np.bool_]:
    """Use opaque texels in the top 34 percent of the opaque bounding height."""
    alpha = np.asarray(image.convert('RGBA'))[..., 3] > 0
    rows = np.flatnonzero(alpha.any(axis=1))
    band = alpha.copy()
    if len(rows):
        band[rows[0] + round(.34 * (rows[-1] - rows[0] + 1)):] = False
    return band


def place(array: NDArray, shape: tuple[int, int], dx: int = 0, dy: int = 0) -> NDArray:
    """Bottom-centre then translate an array, clipping without wraparound."""
    result = np.zeros((*shape, *array.shape[2:]), dtype=array.dtype)
    x = (shape[1] - array.shape[1]) // 2 + dx
    y = shape[0] - array.shape[0] + dy
    left, top = max(0, x), max(0, y)
    right, bottom = min(shape[1], x + array.shape[1]), min(shape[0], y + array.shape[0])
    if right > left and bottom > top:
        result[top:bottom, left:right] = array[top-y:bottom-y, left-x:right-x]
    return result


def colour_counts(references: list[Reference], body: str) -> Counter[RGB]:
    """Count every colour inside approved masks, retaining rare long-hair shades."""
    counts: Counter[RGB] = Counter()
    for reference in references:
        if reference.body != body:
            continue
        pixels = np.asarray(reference.sprite.convert('RGBA'))
        hair = np.asarray(reference.mask.convert('RGBA'))[..., 0] == 3
        counts.update((int(r), int(g), int(b)) for r, g, b in pixels[hair, :3])
    counts.pop(OUTLINE, None)
    return counts


def neighbours(mask: NDArray[np.bool_]) -> NDArray[np.uint8]:
    """Count all eight neighbours from a frozen iteration snapshot."""
    padded = np.pad(mask, 1)
    counts = np.zeros(mask.shape, dtype=np.uint8)
    for dy, dx in NEIGHBOURS:
        counts += padded[1+dy:1+dy+mask.shape[0], 1+dx:1+dx+mask.shape[1]]
    return counts


def component_meeting(candidates: NDArray[np.bool_], template: NDArray[np.bool_]) -> NDArray[np.bool_]:
    """Choose the largest 8-connected component intersecting the template."""
    seen = np.zeros_like(candidates)
    best: list[tuple[int, int]] = []
    height, width = candidates.shape
    for y0, x0 in np.argwhere(candidates):
        if seen[y0, x0]:
            continue
        queue = deque([(int(y0), int(x0))])
        seen[y0, x0] = True
        points: list[tuple[int, int]] = []
        meets = False
        while queue:
            y, x = queue.popleft()
            points.append((y, x))
            meets |= bool(template[y, x])
            for dy, dx in NEIGHBOURS:
                ny, nx = y+dy, x+dx
                if 0 <= ny < height and 0 <= nx < width and candidates[ny, nx] and not seen[ny, nx]:
                    seen[ny, nx] = True
                    queue.append((ny, nx))
        if meets and len(points) > len(best):
            best = points
    result = np.zeros_like(candidates)
    for y, x in best:
        result[y, x] = True
    return result


def remask(sprite: Image.Image, native: Image.Image, body: str, view: str,
           activity: str, references: list[Reference], *, support_down_fraction: float = 0.0) -> Result:
    """Implement GOAL 7.3's exact alignment, selection, growth and fill rules."""
    target_band = head_band(sprite)
    ranked: list[tuple[float, str, int, int, Reference]] = []
    for ref in references:
        if ref.body != body or ref.view != view:
            continue
        band = head_band(ref.sprite)
        shifts = []
        for dy in range(-3, 4):
            for dx in range(-3, 4):
                moved = place(band, target_band.shape, dx, dy)
                union = int(np.count_nonzero(target_band | moved))
                score = float(np.count_nonzero(target_band & moved)) / union if union else 0.0
                shifts.append((-score, abs(dx)+abs(dy), dy, dx))
        score, _, dy, dx = min(shifts)
        ranked.append((-score, ref.ident, dx, dy, ref))
    if not ranked:
        return Result(None, '', 0.0, (0, 0))
    ranked.sort(key=lambda row: (-row[0], row[1]))
    same = [row for row in ranked if row[4].activity == activity]
    score, ident, dx, dy, ref = same[0] if same and same[0][0] >= .75 else ranked[0]
    if score < .60:
        return Result(None, ident, score, (dx, dy))
    rgba = np.asarray(sprite.convert('RGBA'))
    native_alpha = place(np.asarray(native.convert('RGBA'))[..., 3] > 0, target_band.shape,
                         dy=-((sprite.height-native.height+1)//2))
    allowed = native_alpha & (rgba[..., 3] > 0) & ~np.all(rgba[..., :3] == OUTLINE, axis=-1)
    template = place(np.asarray(ref.mask.convert('RGBA'))[..., 0] == 3, target_band.shape, dx, dy)
    dilated = template | (neighbours(template) > 0)
    opaque_rows = np.flatnonzero((rgba[..., 3] > 0).any(axis=1))
    if support_down_fraction and len(opaque_rows):
        reach = round(support_down_fraction * (opaque_rows[-1] - opaque_rows[0] + 1))
        for distance in range(1, reach + 1):
            dilated |= place(template, template.shape, dy=distance)
    colours = sorted(colour for colour, count in colour_counts(references, body).items() if count >= 3)
    candidates = np.zeros_like(target_band)
    for colour in colours:
        candidates |= np.all(rgba[..., :3] == colour, axis=-1)
    mask = component_meeting(candidates & dilated & allowed, template)
    for _ in range(2):
        mask |= template & allowed & (neighbours(mask) >= 5)
    output = np.zeros_like(rgba)
    output[..., 3] = rgba[..., 3]
    if colours:
        lower, upper = np.quantile(np.asarray(colours) @ np.asarray(LUMA) / 255., (1/3, 2/3))
        luma = rgba[..., :3] @ np.asarray(LUMA) / 255.
        shades = np.where(luma >= upper, 0, np.where(luma >= lower, 1, 2))
        output[..., 0][mask] = 3
        output[..., 1][mask] = shades[mask]
    return Result(Image.fromarray(output), ident, score, (dx, dy))
