# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: imported by pixel_normalize_v3.py
"""Production HSV, relative-size and per-view animation acceptance."""
from __future__ import annotations
from typing import TypedDict
import numpy as np
from PIL import Image

class StageMetrics(TypedDict):
    berry_counts: list[int]
    berry_pass: bool
    fire_counts: list[int]
    fire_pass: bool
    child_pass: bool
    basket_pass: bool
    passed: bool

class AnimationMetrics(TypedDict):
    feet_baseline_pass: bool
    head_top_span: int
    head_top_pass: bool
    color_jaccard: list[float]
    color_pass: bool
    walk_lower_differences: list[int]
    walk_pass: bool
    passed: bool


def semantic_counts(image: Image.Image) -> tuple[int, int]:
    """Count only opaque berry-red and bright orange/yellow fire texels."""
    rgba = np.asarray(image.convert('RGBA'))
    hsv = np.asarray(image.convert('RGB').convert('HSV'), dtype=np.float64)
    hue, sat, value = hsv[..., 0] * 360 / 255, hsv[..., 1] / 255, hsv[..., 2] / 255
    opaque = rgba[..., 3] == 255
    berry = ((hue >= 345) | (hue <= 15)) & (sat > .55) & (value > .45) & opaque
    fire = (hue >= 15) & (hue <= 55) & (sat > .6) & (value > .75) & opaque
    return int(berry.sum()), int(fire.sum())


def evaluate_stages(bushes: list[Image.Image], fires: list[Image.Image],
                    adults: list[Image.Image], child: Image.Image,
                    basket: Image.Image) -> StageMetrics:
    """Validate full/two/one/bare ordering and relative dimensions."""
    if len(bushes) != 4 or not adults or not fires:
        raise ValueError('Need four berry stages, at least one adult and lit fire')
    counts = [semantic_counts(image)[0] for image in bushes]
    flame = [semantic_counts(image)[1] for image in fires]
    berry_pass = counts[0] > counts[1] > counts[2] >= 1 and counts[3] == 0
    fire_pass = min(flame) >= 20
    child_pass = all(child.height < adult.height for adult in adults)
    basket_pass = all(basket.width <= bush.width and basket.height <= bush.height
                      and basket.width * basket.height < bush.width * bush.height for bush in bushes)
    return StageMetrics(berry_counts=counts, berry_pass=berry_pass, fire_counts=flame,
        fire_pass=fire_pass, child_pass=child_pass, basket_pass=basket_pass,
        passed=berry_pass and fire_pass and child_pass and basket_pass)


def evaluate_animation(idle: Image.Image, frames: list[Image.Image],
                       walk_frames: list[Image.Image] | None = None) -> AnimationMetrics:
    """Compare one character/view in a common foot-anchored native canvas."""
    sequence = [idle, *frames]
    boxes = [frame.getbbox() for frame in sequence]
    feet = all(box is not None and box[3] == frame.height for box, frame in zip(boxes, sequence))
    height = max(frame.height for frame in sequence)
    top = [height - frame.height + box[1] for frame, box in zip(sequence, boxes) if box is not None]
    span = max(top) - min(top) if top else height
    def colors(frame: Image.Image) -> set[tuple[int, int, int]]:
        rgba = np.asarray(frame.convert('RGBA'))
        return {tuple(map(int, color)) for color in rgba[rgba[..., 3] == 255, :3]}
    reference = colors(idle)
    similarities = [len(reference & colors(frame)) / max(1, len(reference | colors(frame))) for frame in frames]
    differences: list[int] = []
    if walk_frames is not None:
        if len(walk_frames) != 4:
            raise ValueError('Walk acceptance requires exactly four frames')
        width = max(frame.width for frame in walk_frames)
        height = max(frame.height for frame in walk_frames)
        aligned = []
        for frame in walk_frames:
            canvas = Image.new('RGBA', (width, height))
            canvas.paste(frame, ((width - frame.width) // 2, height - frame.height))
            aligned.append(np.asarray(canvas)[int(height * .6):])
        differences = [int(np.any(aligned[a] != aligned[b], axis=2).sum()) for a, b in ((0, 1), (0, 3), (2, 1), (2, 3))]
    color_pass = all(value >= .7 for value in similarities)
    walk_pass = all(value >= 6 for value in differences)
    return AnimationMetrics(feet_baseline_pass=feet, head_top_span=span,
        head_top_pass=span <= 1, color_jaccard=similarities, color_pass=color_pass,
        walk_lower_differences=differences, walk_pass=walk_pass,
        passed=feet and span <= 1 and color_pass and walk_pass)
