# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python -m art.pipeline3.check_layer --whole-figure image.png adult idle
"""Frozen v3 whole-figure geometry; never modifies input pixels."""
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image

from art.pipeline3.structure import components, holes

ALPHA_THRESHOLD: Final = 128
FRAMES: Final = ('idle', 'walk_a', 'work_a', 'carry', 'sleep')


@dataclass(frozen=True, slots=True)
class WholeResult:
    passed: bool
    findings: tuple[str, ...]
    coverage: float
    component_count: int
    largest_hole: int
    contact_ok: bool
    limb_ok: str = 'NOT_MEASURED: whole figures have no semantic limb masks'


def check_whole(image: Image.Image, age: str, frame: str) -> WholeResult:
    """Check an unmodified final canvas under preregistered bounds."""
    if age not in ('adult', 'child') or frame not in FRAMES:
        message = f'Unsupported age/frame: {age}/{frame}'
        raise ValueError(message)
    sleep = frame == 'sleep'
    size = (96, 64) if sleep else (64, 96)
    if image.mode != 'RGBA' or image.size != size:
        return WholeResult(False, ('canvas: expected RGBA ' + str(size),), 0, 0, 0, False)
    alpha = np.asarray(image)[:, :, 3]
    mask = alpha >= ALPHA_THRESHOLD
    visible = alpha > 0
    findings: list[str] = []
    groups = components(mask)
    if len(groups) != 1:
        findings.append(f'connectivity: {len(groups)} components, expected 1')
    gaps = holes(mask, 1)
    largest = max((len(group) for group in gaps), default=0)
    if largest:
        findings.append(f'holes: enclosed gap {largest}px2 exceeds12')
    coverage = float(mask.mean())
    bounds = {(False, 'adult'): (.08, .55), (False, 'child'): (.04, .40),
              (True, 'adult'): (.04, .50), (True, 'child'): (.02, .35)}
    lower, upper = bounds[(sleep, age)]
    if not lower <= coverage <= upper:
        findings.append(f'coverage: {coverage:.6f} outside [{lower},{upper}]')
    oy, _ = np.nonzero(alpha >= ALPHA_THRESHOLD)
    contact = bool(len(oy) and int(oy.max()) == (56 if sleep else 88))
    if not contact:
        findings.append('contact_ok: lowest alpha128 pixel off baseline or missing')
    if any(np.any(edge) for edge in (visible[0], visible[-1], visible[:, 0], visible[:, -1])):
        findings.append('alpha_edge: visible pixels touch canvas border')
    if sleep and np.any(mask):
        ys, xs = np.nonzero(mask)
        if int(xs.max() - xs.min()) <= int(ys.max() - ys.min()):
            findings.append('sleep_axis: expected wider than tall')
    return WholeResult(not findings, tuple(findings), coverage, len(groups), largest, contact)


def check_whole_path(path: Path, age: str, frame: str) -> WholeResult:
    """Read source mode without converting a malformed image."""
    with Image.open(path) as image:
        return check_whole(image, age, frame)
