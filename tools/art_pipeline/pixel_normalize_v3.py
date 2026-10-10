# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: import recover_cell_v3 and evaluate_sprite from this module.
"""Production native recovery; preserves v2 and excludes keyed-out fidelity texels."""
from __future__ import annotations
from typing import TypedDict
import numpy as np
from PIL import Image
from native_grid import KEY_RULE, _blocks, key_mask, remove_spill
from palette_tools import Color, rgb_to_lab
from v3_grid import detect_grid
from v3_semantics import evaluate_animation, evaluate_stages, semantic_counts

class RecoveryMetadata(TypedDict):
    key_rule: str
    source_bbox: list[int]
    detected_period_xy: list[float]
    period_xy: list[float]
    phase_xy: list[float]
    axis_periods_separate: bool
    axis_improvement_xy: list[float]
    native_size: list[int]
    sample_rgba: list[list[list[int]]]
    spill_removed: int
    fringe_repaired: int

class SpriteMetrics(TypedDict):
    native_size: list[int]
    size_pass: bool
    mean_delta_e: float
    fidelity_pass: bool
    common_opaque: int
    keyed_out: int
    palette_match_percent: float
    binary_alpha: bool
    berry_texels: int
    fire_texels: int
    format_pass: bool
    passed: bool


def recover_cell_v3(image: Image.Image) -> tuple[Image.Image, RecoveryMetadata]:
    """Recover native RGBA plus the same-sized pre-spill fidelity reference."""
    rgba = np.asarray(image.convert('RGBA'))
    mask = key_mask(rgba)
    yy, xx = np.nonzero(mask)
    if not len(xx):
        raise ValueError('Cell contains no foreground')
    bbox = [int(xx.min()), int(yy.min()), int(xx.max()) + 1, int(yy.max()) + 1]
    crop = rgba[bbox[1]:bbox[3], bbox[0]:bbox[2]]
    mask = mask[bbox[1]:bbox[3], bbox[0]:bbox[2]]
    grid = detect_grid(crop[..., :3].astype(np.float64) @ np.array([0.2126, 0.7152, 0.0722]), mask)
    columns = _blocks(crop.shape[1], grid.period_xy[0], grid.phase_xy[0])
    rows = _blocks(crop.shape[0], grid.period_xy[1], grid.phase_xy[1])
    sample = np.zeros((len(rows), len(columns), 4), dtype=np.uint8)
    for y, (y0, y1) in enumerate(rows):
        for x, (x0, x1) in enumerate(columns):
            coverage = mask[y0:y1, x0:x1]
            if coverage.sum() * 2 >= coverage.size:
                sample[y, x, :3] = np.median(crop[y0:y1, x0:x1, :3][coverage], axis=0).astype(np.uint8)
                sample[y, x, 3] = 255
    sampled = Image.fromarray(sample)
    retained = sampled.getbbox()
    if retained is None:
        raise ValueError('Native sampling retained no texels')
    sampled = sampled.crop(retained)
    cleaned, spill = remove_spill(sampled)
    return cleaned, RecoveryMetadata(key_rule=KEY_RULE, source_bbox=bbox,
        detected_period_xy=list(grid.detected_period_xy), period_xy=list(grid.period_xy),
        phase_xy=list(grid.phase_xy), axis_periods_separate=grid.separate_axes,
        axis_improvement_xy=list(grid.improvement_xy), native_size=list(cleaned.size),
        sample_rgba=np.asarray(sampled).tolist(), spill_removed=spill['spill_removed'],
        fringe_repaired=spill['fringe_repaired'])


def evaluate_sprite(sample: Image.Image, quantized: Image.Image, palette: list[Color],
                    maxsize: tuple[int, int]) -> SpriteMetrics:
    """Evaluate pre-outline geometry and common-opaque fidelity, never removed keys."""
    if sample.size != quantized.size:
        raise ValueError('Fidelity inputs must have identical pre-outline geometry')
    before, after = np.asarray(sample.convert('RGBA')), np.asarray(quantized.convert('RGBA'))
    original, final = before[..., 3] == 255, after[..., 3] == 255
    common = original & final
    delta = np.linalg.norm(rgb_to_lab(before[common, :3]) - rgb_to_lab(after[common, :3]), axis=1)
    mean = float(delta.mean()) if delta.size else 0.0
    allowed = set(palette)
    colors = after[final, :3]
    matched = sum(tuple(map(int, color)) in allowed for color in colors)
    membership = 100.0 * matched / len(colors) if len(colors) else 0.0
    binary = bool(np.isin(after[..., 3], [0, 255]).all())
    size_pass = quantized.width <= maxsize[0] and quantized.height <= maxsize[1]
    berry, fire = semantic_counts(quantized)
    fidelity_pass = bool(delta.size) and mean <= 6
    format_pass = membership == 100 and binary
    return SpriteMetrics(native_size=list(quantized.size), size_pass=size_pass,
        mean_delta_e=mean, fidelity_pass=fidelity_pass, common_opaque=int(common.sum()),
        keyed_out=int((original & ~final).sum()), palette_match_percent=membership,
        binary_alpha=binary, berry_texels=berry, fire_texels=fire, format_pass=format_pass,
        passed=size_pass and fidelity_pass and format_pass)


def quantize_native(image: Image.Image, palette: list[Color]) -> Image.Image:
    """Match retained native texels to the production palette in CIELAB."""
    rgba = np.array(image.convert('RGBA'))
    opaque = rgba[..., 3] == 255
    colors = np.asarray(palette, dtype=np.uint8)
    if not len(colors):
        raise ValueError('Production palette must contain colors')
    distances = np.sum((rgb_to_lab(rgba[opaque, :3])[:, None] - rgb_to_lab(colors)[None]) ** 2, axis=2)
    rgba[opaque, :3] = colors[np.argmin(distances, axis=1)]
    return Image.fromarray(rgba)
