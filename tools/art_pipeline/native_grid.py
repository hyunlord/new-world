"""Recover generated sprite texels without scaling them to a target box."""
from __future__ import annotations

import math
import numpy as np
from PIL import Image


KEY_RULE = "R>150 and B>150 and G<110 and abs(R-B)<90; source alpha>=128"


def key_mask(rgba: np.ndarray) -> np.ndarray:
    """Return foreground coverage using the recorded lossless-sheet key."""
    rgb = rgba[..., :3].astype(np.int16)
    red, green, blue = np.moveaxis(rgb, -1, 0)
    key = (red > 150) & (blue > 150) & (green < 110) & (np.abs(red - blue) < 90)
    return ~key & (rgba[..., 3] >= 128)


def detect_axis(profile: np.ndarray) -> tuple[float, float, float]:
    """Maximize interpolated mean edge energy on the specified search lattice."""
    if profile.size < 3 or not np.any(profile):
        return 6.0, 0.0, 0.0
    coordinates = np.arange(1, profile.size + 1, dtype=np.float64)
    best = (6.0, 0.0, -math.inf)
    for period in np.arange(3.0, 9.0, 0.05):
        for phase in np.arange(0.0, period, 0.25):
            positions = np.arange(phase, profile.size + 1, period)
            positions = positions[(positions >= 1) & (positions <= profile.size)]
            if not positions.size:
                continue
            score = float(np.mean(np.interp(positions, coordinates, profile)))
            if score > best[2]:
                best = (float(period), float(phase), score)
    return best


def _phase_for_period(profile: np.ndarray, period: float) -> float:
    coordinates = np.arange(1, profile.size + 1, dtype=np.float64)
    phases = np.arange(0.0, period, 0.25)
    scores = []
    for phase in phases:
        positions = np.arange(phase, profile.size + 1, period)
        positions = positions[(positions >= 1) & (positions <= profile.size)]
        scores.append(float(np.mean(np.interp(positions, coordinates, profile))) if positions.size else -math.inf)
    return float(phases[int(np.argmax(scores))])


def _blocks(length: int, period: float, phase: float) -> list[tuple[int, int]]:
    # Pixel centres, rather than pixel indices, decide inner-half membership.
    starts = np.arange(phase - period, length, period)
    blocks = []
    for start in starts:
        lower = max(0, math.ceil(start + period * 0.25 - 0.5))
        upper = min(length, math.ceil(start + period * 0.75 - 0.5))
        if upper > lower:
            blocks.append((lower, upper))
    return blocks


def remove_spill(image: Image.Image) -> tuple[Image.Image, dict[str, int]]:
    """Remove native key spill, then repair only magenta-cast boundary texels."""
    rgba = np.array(image.convert("RGBA"))
    hsv = np.asarray(image.convert("HSV"), dtype=np.float64)
    hue = hsv[..., 0] * 360 / 255
    spill = (hue >= 285) & (hue <= 330) & (hsv[..., 1] / 255 > 0.35) & (rgba[..., 3] == 255)
    rgba[spill] = 0
    opaque = rgba[..., 3] == 255
    padded = np.pad(opaque, 1)
    interior = padded[:-2, 1:-1] & padded[2:, 1:-1] & padded[1:-1, :-2] & padded[1:-1, 2:]
    rgb = rgba[..., :3].astype(np.int16)
    cast = opaque & ~interior & (rgb[..., 0] > rgb[..., 1] + 40) & (rgb[..., 2] > rgb[..., 1] + 40)
    original = rgba.copy()
    repaired = 0
    for y, x in np.argwhere(cast):
        neighbors = [original[ny, nx, :3] for ny, nx in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1))
                     if 0 <= ny < rgba.shape[0] and 0 <= nx < rgba.shape[1] and opaque[ny, nx]]
        if neighbors:
            rgba[y, x, :3] = np.median(neighbors, axis=0).astype(np.uint8)
            repaired += 1
    return Image.fromarray(rgba), {"spill_removed": int(spill.sum()), "fringe_repaired": repaired}


def recover_cell(image: Image.Image) -> tuple[Image.Image, dict[str, object]]:
    """Return tight native RGBA after spill removal plus JSON-safe grid evidence.

    ``sample_rgba`` records the step-3 pre-spill sample for fidelity measurement;
    its dimensions equal the returned image, with no geometric resampling.
    """
    rgba = np.array(image.convert("RGBA"))
    mask = key_mask(rgba)
    yy, xx = np.nonzero(mask)
    if not len(xx):
        raise ValueError("Cell contains no foreground after chroma key")
    bbox = (int(xx.min()), int(yy.min()), int(xx.max()) + 1, int(yy.max()) + 1)
    crop = rgba[bbox[1]:bbox[3], bbox[0]:bbox[2]]
    foreground = mask[bbox[1]:bbox[3], bbox[0]:bbox[2]]
    luma = crop[..., :3].astype(np.float64) @ np.array([0.2126, 0.7152, 0.0722])
    xp = np.abs(np.diff(luma, axis=1)).sum(axis=0)
    yp = np.abs(np.diff(luma, axis=0)).sum(axis=1)
    sx, px, xscore = detect_axis(xp)
    sy, py, yscore = detect_axis(yp)
    independent = abs(sx - sy) > 0.3 + 1e-9
    detected = [round(sx, 4), round(sy, 4)]
    if not independent:
        sx = sy = (sx + sy) / 2
        px, py = _phase_for_period(xp, sx), _phase_for_period(yp, sy)
    columns, rows = _blocks(crop.shape[1], sx, px), _blocks(crop.shape[0], sy, py)
    sampled = np.zeros((len(rows), len(columns), 4), dtype=np.uint8)
    for y, (y0, y1) in enumerate(rows):
        for x, (x0, x1) in enumerate(columns):
            block_mask = foreground[y0:y1, x0:x1]
            if block_mask.sum() * 2 >= block_mask.size:
                sampled[y, x, :3] = np.median(crop[y0:y1, x0:x1, :3][block_mask], axis=0).astype(np.uint8)
                sampled[y, x, 3] = 255
    sample_image = Image.fromarray(sampled)
    retained = sample_image.getbbox()
    if retained is None:
        raise ValueError("Native majority sampling retained no texels")
    sample_image = sample_image.crop(retained)
    cleaned, spill = remove_spill(sample_image)
    metadata: dict[str, object] = {
        "key_rule": KEY_RULE, "source_bbox": list(bbox), "detected_period_xy": detected,
        "period_xy": [round(sx, 4), round(sy, 4)], "phase_xy": [px, py],
        "axis_periods_separate": independent, "edge_score_xy": [xscore, yscore],
        "native_size": list(cleaned.size), "sample_rgba": np.asarray(sample_image).tolist(), **spill,
    }
    return cleaned, metadata
