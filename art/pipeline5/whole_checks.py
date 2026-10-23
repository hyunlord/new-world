# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python -m art.pipeline3.check_layer --whole-figure image.png adult idle
"""PROD1 whole-figure geometry; never modifies input pixels."""
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image

from art.pipeline3.structure import components

ALPHA_THRESHOLD: Final = 128
VISIBILITY_THRESHOLD: Final = 16
FRAMES: Final = ('idle', 'walk_a', 'work_a', 'carry', 'sleep')


@dataclass(frozen=True, slots=True)
class WholeResult:
    passed: bool
    findings: tuple[str, ...]
    coverage: float
    component_count: int
    largest_hole: int
    contact_ok: bool
    sleep_axis_ok: bool = True
    opaque_height: int = 0
    opaque_width: int = 0
    coverage_rule: str = 'idle height only; age-specific sleep length and height; posed extent constrained by canvas'
    limb_ok: str = 'NOT_MEASURED: whole figures have no semantic limb masks'
    visible_height: int = 0
    visible_width: int = 0
    visibility_threshold: int = VISIBILITY_THRESHOLD


def check_whole(image: Image.Image, age: str, frame: str) -> WholeResult:
    """Check an unmodified final canvas under preregistered bounds."""
    if age not in ('adult', 'child') or frame not in FRAMES:
        message = f'Unsupported age/frame: {age}/{frame}'
        raise ValueError(message)
    sleep = frame == 'sleep'
    size = (112, 80) if sleep else (80, 112)
    if image.mode != 'RGBA' or image.size != size:
        return WholeResult(False, ('canvas: expected RGBA ' + str(size),), 0, 0, 0, False)
    alpha = np.asarray(image)[:, :, 3]
    mask = alpha >= ALPHA_THRESHOLD
    visible = alpha >= VISIBILITY_THRESHOLD
    findings: list[str] = []
    groups = components(mask)
    if len(groups) != 1:
        findings.append(f'connectivity: {len(groups)} components, expected 1')
    # Negative spaces are anatomy, not layered-part holes. Keep the legacy field
    # zero for JSON readers; it is no longer an acceptance measurement.
    largest = 0
    coverage = float(mask.mean())
    oy, ox = np.nonzero(mask)
    opaque_height = int(oy.max()-oy.min()+1) if len(oy) else 0
    opaque_width = int(ox.max()-ox.min()+1) if len(ox) else 0
    vy, vx = np.nonzero(visible)
    height = int(vy.max()-vy.min()+1) if len(vy) else 0
    width = int(vx.max()-vx.min()+1) if len(vx) else 0
    if frame == 'idle':
        lower, upper = (37, 43) if age == 'child' else (52, 58)
        if not lower <= height <= upper:
            findings.append(f'coverage: idle visible height {height} outside [{lower},{upper}]')
    if sleep:
        lower, upper, max_height = (37, 43, 28) if age == 'child' else (52, 60, 36)
        if not lower <= width <= upper:
            findings.append(f'coverage: sleep visible width {width} outside [{lower},{upper}]')
        if height > max_height:
            findings.append(f'coverage: sleep visible height {height} exceeds {max_height}')
    contact = bool(len(vy) and abs(int(vy.max()) - (72 if sleep else 104)) <= 2)
    if not contact:
        findings.append('contact_ok: lowest alpha16 pixel off baseline or missing')
    if any(np.any(edge) for edge in (visible[0], visible[-1], visible[:, 0], visible[:, -1])):
        findings.append('alpha_edge: visible pixels touch canvas border')
    sleep_axis = not sleep or width > height
    if not sleep_axis:
        findings.append('sleep_axis_ok: expected wider than tall after auto-rotation')
    return WholeResult(not findings, tuple(findings), coverage, len(groups), largest,
                       contact, sleep_axis, opaque_height, opaque_width,
                       visible_height=height, visible_width=width)


def check_whole_path(path: Path, age: str, frame: str) -> WholeResult:
    """Read source mode without converting a malformed image."""
    with Image.open(path) as image:
        return check_whole(image, age, frame)
