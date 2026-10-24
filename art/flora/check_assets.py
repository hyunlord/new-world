# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy", "pillow", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: uv run --with scikit-image python -m art.flora.check_assets
"""Produce masks and evidence without retouching candidate image pixels."""
from __future__ import annotations

import colorsys
import hashlib
import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image
from pydantic import TypeAdapter

from art.compositor.projection_check import (
    ProjectionResult,
    ProjectionSpec,
    check_projection,
)
from art.flora.resolve_flora import read_catalog
from art.flora.schema import Frame
from art.flora.seasons import canopy_mask, foliage_clusters, read_palette
from art.pipeline3.structure import components

ROOT: Final = Path(__file__).resolve().parents[2]
REPORT: Final = ROOT / 'docs/v8/arttrees/CHECKS.json'
DATA: Final = Path('/Users/rexxa/work/arttrees-data')


@dataclass(frozen=True, slots=True)
class MaskMetrics:
    trunk_box_fraction: float
    foliage_cluster_coverage: float
    mask_pixels: int
    foliage_pixels: int
    passed: bool
    foliage_gate_applicable: bool = True


@dataclass(frozen=True, slots=True)
class SummerMetrics:
    mean_hue_degrees: float | None
    mean_saturation: float | None
    palette_hue_range_degrees: tuple[float, float]
    passed: bool


@dataclass(frozen=True, slots=True)
class StructureMetrics:
    components_alpha128: int
    clipping_alpha16: bool
    passed: bool
    components_alpha16: int = 0
    detached_pixels_alpha128: int = 0


@dataclass(frozen=True, slots=True)
class SpriteCheck:
    path: str
    sha256: str
    mask_path: str
    mask_sha256: str
    canvas_matches: bool
    projection: ProjectionResult
    structure: StructureMetrics
    mask: MaskMetrics
    summer_green: SummerMetrics | None
    passed: bool


def mask_metrics(source: Image.Image, mask: Image.Image, root: Path) -> MaskMetrics:
    """Measure full-canvas lower 30%, central 40% box and cluster coverage."""
    selected: NDArray[np.bool_] = np.asarray(mask.convert('L')) > 0
    height, width = mask.height, mask.width
    box = selected[int(np.floor(height * .7)):, int(np.floor(width * .3)):int(np.ceil(width * .7))]
    fraction = float(box.mean()) if box.size else 0.0
    foliage = foliage_clusters(source, read_palette(root))
    count = int(foliage.sum())
    covered = float((selected & foliage).sum() / count) if count else 1.0
    return MaskMetrics(fraction, covered, int(selected.sum()), count, fraction <= .02 and covered >= .85)


def summer_metrics(source: Image.Image, mask: Image.Image, root: Path) -> SummerMetrics:
    """Measure circular mean hue and mean saturation in the classified canopy."""
    palette = read_palette(root)
    colors = (palette.foliage_summer_mid, palette.foliage_summer_light, palette.foliage_summer_dark)
    hues = [colorsys.rgb_to_hsv(*(channel / 255 for channel in color))[0] * 360 for color in colors]
    band = (min(hues), max(hues))
    rgba: NDArray[np.uint8] = np.asarray(source.convert('RGBA'), dtype=np.uint8)
    selected: NDArray[np.bool_] = (np.asarray(mask.convert('L')) > 0) & (rgba[..., 3] >= 128)
    pixels: NDArray[np.float64] = rgba[selected, :3].astype(np.float64) / 255
    if not pixels.size:
        return SummerMetrics(None, None, band, False)
    hsv: NDArray[np.float64] = np.array([colorsys.rgb_to_hsv(float(pixel[0]), float(pixel[1]), float(pixel[2])) for pixel in pixels])
    angle: NDArray[np.float64] = hsv[:, 0] * 2 * np.pi
    hue = float(np.degrees(np.arctan2(np.sin(angle).mean(), np.cos(angle).mean())) % 360)
    saturation = float(hsv[:, 1].mean())
    return SummerMetrics(hue, saturation, band, band[0] <= hue <= band[1] and saturation >= .30)


def check_frame(frame: Frame, band: tuple[int, int], *, summer: bool, empty_mask: bool = False) -> SpriteCheck:
    """Freeze one mask, then inspect the unchanged native sprite and metadata."""
    path = ROOT / frame.path
    mask_path = ROOT / frame.canopy_mask
    with Image.open(path) as original:
        source = original.convert('RGBA')
    mask = Image.new('L', source.size, 0) if empty_mask else canopy_mask(source, ROOT)
    mask_path.parent.mkdir(parents=True, exist_ok=True)
    mask.save(mask_path, optimize=True)
    alpha: NDArray[np.uint8] = np.asarray(source, dtype=np.uint8)[..., 3]
    groups = components(alpha >= 128)
    count = len(groups)
    visible_count = len(components(alpha >= 16))
    detached = sum(len(group) for group in groups) - max((len(group) for group in groups), default=0)
    clipped = any(bool(np.any(edge >= 16)) for edge in (alpha[0], alpha[-1], alpha[:, 0], alpha[:, -1]))
    structure = StructureMetrics(count, clipped, count == 1 and not clipped, visible_count, detached)
    family = 'stump' if band == (14, 24) else 'tree'
    projection = check_projection(source, ProjectionSpec(family, frame.anchor, band))
    measured_mask = mask_metrics(source, mask, ROOT)
    if empty_mask:
        measured_mask = MaskMetrics(measured_mask.trunk_box_fraction, measured_mask.foliage_cluster_coverage,
                                    measured_mask.mask_pixels, measured_mask.foliage_pixels, True, False)
    green = summer_metrics(source, mask, ROOT) if summer else None
    canvas_matches = source.size == frame.canvas
    passed = canvas_matches and structure.passed and projection.passed and measured_mask.passed and (green is None or green.passed)
    return SpriteCheck(frame.path, hashlib.sha256(path.read_bytes()).hexdigest(), frame.canopy_mask,
                       hashlib.sha256(mask_path.read_bytes()).hexdigest(), canvas_matches,
                       projection, structure, measured_mask, green, passed)


def main() -> int:
    """Record negative controls first, candidate reports second; never waive failures."""
    from art.pipeline2.judge import judge, load_calibration

    calibration = load_calibration(ROOT / 'docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json')
    controls = [Path('/Users/rexxa/work/arttrial2-data') / name for name in
                ('negative_painterly.png', 'negative_woodcut.png', 'negative_photo_try1.png')]
    control_results = [judge(path, calibration, family='plants') for path in controls]
    gating = all(not result.numeric_pass for result in control_results)
    catalog = read_catalog(ROOT)
    frames: list[tuple[Frame, tuple[int, int], bool, bool]] = []
    for species in catalog.species.species.values():
        if species.habit == 'shrub':
            continue
        for size, entries in species.sizes.items():
            band = (64, 96) if size == 'small' else (128, 192)
            frames.extend((frame, band, True, False) for frame in entries.variants)
            frames.extend((frame, band, False, True) for frame in entries.bare)
        if species.stump is not None:
            frames.append((species.stump, (14, 24), False, True))
    missing = [frame.path for frame, _, _, _ in frames if not (ROOT / frame.path).is_file()]
    results = [check_frame(frame, band, summer=summer, empty_mask=empty)
               for frame, band, summer, empty in frames if (ROOT / frame.path).is_file()]
    history_path = REPORT.with_name('ATTEMPT_CHECKS.json')
    history = TypeAdapter(list[SpriteCheck]).validate_json(history_path.read_bytes()) if history_path.exists() else []
    seen = {(row.path, row.sha256) for row in history}
    history.extend(row for row in results if (row.path, row.sha256) not in seen)
    _ = history_path.write_text(json.dumps([asdict(row) for row in history], indent=2) + '\n')
    verdicts = [judge(ROOT / result.path, calibration, family='plants') for result in results]
    payload = {
        'schema_version': 1,
        'thresholds': {'trunk_box_max_fraction': .02, 'foliage_cluster_min_coverage': .85,
                       'canopy_min_saturation': .30, 'single_component_alpha': 128, 'clip_alpha': 16},
        'mask_method': 'Nearest Lab palette clusters including legacy olive; exclude L*<=24 ink; no spatial masking. Cluster coverage measures extraction fidelity, not human semantic accuracy.',
        'trunk_box_definition': 'lowest 30% of full canvas height, central 40% of full canvas width',
        'projection_band_note': 'Owner sizes: large128..192, small64..96; stump14..24 from PROJECTION. Explicit override recorded, unchanged images.',
        'calibration': {'family': 'plants', 'controls': [asdict(result) for result in control_results],
                        'gating_enabled': gating, 'condition': 'all three numeric negative controls rejected'},
        'sprites': [asdict(result) for result in results],
        'style_reports': [asdict(result) for result in verdicts],
        'missing': missing,
        'technical_passed': not missing and all(result.passed for result in results),
        'style_gate_passed': all(result.numeric_pass for result in verdicts) if gating else None,
        'owner_verdict': 'PENDING',
    }
    _ = REPORT.write_text(json.dumps(payload, indent=2) + '\n')
    DATA.mkdir(parents=True, exist_ok=True)
    _ = (DATA / 'checks_latest.json').write_text(json.dumps(payload, indent=2) + '\n')
    print(json.dumps({'checked': len(results), 'missing': len(missing), 'technical_passed': payload['technical_passed'], 'style_gating': gating}))
    return 0 if payload['technical_passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
