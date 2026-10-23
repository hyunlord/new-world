# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by sprite_masks.py; reviewed via sprite_audit.py.
"""Translation-only head matching against the same look/facing's idle source."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

import numpy as np
from PIL import Image

from art.pipeline_polish2.sprite_annotations import SourceAnatomy
from art.pipeline_polish2.sprite_colour import (
    BoolPixels,
    FloatPixels,
    rgb_to_lab,
    squared_distance,
)
from art.pipeline_polish2.sprite_regions import SourceEnvelope

Pose = Literal["idle", "walk_a", "work_a", "carry", "sleep"]


@dataclass(frozen=True, slots=True)
class FrameGuide:
    child: bool
    bare_chest: bool
    pose: Pose = "idle"
    idle_reference: Image.Image | None = None
    annotation: SourceAnatomy | None = None
    fixed_masks: tuple[BoolPixels, BoolPixels] | None = None
    envelope: SourceEnvelope | None = None
    refine_skin: bool = False
    refine_hair: bool = False


def locate_head(source: Image.Image, reference: Image.Image, box: tuple[int, int, int, int]) -> tuple[int, int]:
    """Match alpha plus Lab appearance; warm face pixels get stronger weight."""
    ref = np.asarray(reference.convert("RGBA"))
    src = np.asarray(source.convert("RGBA"))
    left, top, right, bottom = box
    template = ref[top:bottom, left:right]
    lab_template = rgb_to_lab(template[:, :, :3].astype(np.float64))
    lab_source = rgb_to_lab(src[:, :, :3].astype(np.float64))
    solid = template[:, :, 3] >= 128
    rgb = template[:, :, :3].astype(np.int16)
    skin = (rgb[:, :, 0] > 145) & (rgb[:, :, 0] - rgb[:, :, 1] > 50)
    weights = solid.astype(np.float64) * (1 + skin * 3)
    width, height = right - left, bottom - top
    best_score = float("inf")
    best = (left, top)
    for yy in range(max(0, top - 13), min(source.height - height, top + 25) + 1):
        for xx in range(max(0, left - 18), min(source.width - width, left + 18) + 1):
            patch = lab_source[yy:yy + height, xx:xx + width]
            squared: FloatPixels = squared_distance(patch - lab_template)
            error: FloatPixels = np.minimum(squared, 4000)
            alpha_error = np.abs(src[yy:yy + height, xx:xx + width, 3].astype(np.float64) - template[:, :, 3]) / 255
            weighted_sum: np.float64 = np.sum(error * weights)
            weight_sum: np.float64 = np.sum(weights)
            alpha_mean: np.float64 = np.mean(alpha_error, dtype=np.float64)
            score = float(weighted_sum / weight_sum + alpha_mean * 1200)
            if score < best_score:
                best_score, best = score, (xx, yy)
    return best
