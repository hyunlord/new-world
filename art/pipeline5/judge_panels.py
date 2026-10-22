# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: import judge_panels after ownership slicing, before normalization.
"""Judge source-resolution figures with frozen v2 person-family thresholds.

Legacy whole-sheet judging remains in pipeline2.judge and historical TRIAL5
processing. It is NOT a PROD1 acceptance instrument. Transparent 8% padding is
added per axis on each side; judge v2 internally applies its original foreground
crop and comparison scaling, unchanged. Model acceptance is never implied.
"""
from __future__ import annotations

import hashlib
import json
import math
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from PIL import Image

from art.pipeline2.judge import Verdict, judge, load_calibration

ROOT: Final = Path(__file__).resolve().parents[2]
CALIBRATION: Final = ROOT / 'docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json'
CALIBRATION_SHA256: Final = '7218ac77979065ec6398c77cb6156bce5bd523979ed48732945f2ab642a5d5fd'
PADDING_FRACTION: Final = 0.08


@dataclass(frozen=True, slots=True)
class PreparedPanel:
    """Unscaled source pixels plus a transparent measurement margin."""

    image: Image.Image
    crop_box: tuple[int, int, int, int]
    padding: tuple[int, int]


@dataclass(frozen=True, slots=True)
class PanelVerdict:
    """Bind every numeric result to the exact unscaled crop and saved PNG."""

    index: int
    source_size: tuple[int, int]
    source_rgba_sha256: str
    crop_box: tuple[int, int, int, int]
    padding: tuple[int, int]
    measured_size: tuple[int, int]
    verdict: Verdict


@dataclass(frozen=True, slots=True)
class SheetVerdict:
    """Numeric acceptance requires the entire expected panel population."""

    source: str
    source_sha256: str
    calibration_sha256: str
    expected_count: int
    panels: tuple[PanelVerdict, ...]
    numeric_pass: bool
    schema: str = 'prod1-source-panels-v1'
    family: str = 'person'
    measurement: str = 'source-resolution alpha>0 crop; ceil(8%) transparent pad per side per axis'
    legacy_whole_sheet: bool = False
    model_status: str = 'MISSING'


def prepare_panel(panel: Image.Image) -> PreparedPanel:
    """Crop alpha-zero margins only; preserve all visible RGBA without scaling."""
    rgba = panel.convert('RGBA')
    box = rgba.getchannel('A').getbbox()
    if box is None:
        message = 'panel_judge: empty alpha foreground'
        raise ValueError(message)
    crop = rgba.crop(box)
    padding = (math.ceil(crop.width * PADDING_FRACTION),
               math.ceil(crop.height * PADDING_FRACTION))
    padded = Image.new('RGBA', (crop.width + 2 * padding[0], crop.height + 2 * padding[1]))
    padded.paste(crop, padding)
    return PreparedPanel(padded, box, padding)


def judge_panels(source: Path, panels: tuple[Image.Image, ...], output: Path,
                 *, expected_count: int = 10) -> SheetVerdict:
    """Measure ownership-sliced source images, never normalized delivery sprites.

    The caller must supply panels straight from the source ownership slicer.
    Five panels are supported for historical calibration/recounts, ten for PROD1.
    One panel is supported solely for the historical I-1 positive control.
    """
    if expected_count not in (1, 5, 10) or len(panels) != expected_count:
        message = f'panel_judge: need exactly {expected_count} panels (allowed: 1, 5, 10), got {len(panels)}'
        raise ValueError(message)
    calibration_sha = hashlib.sha256(CALIBRATION.read_bytes()).hexdigest()
    if calibration_sha != CALIBRATION_SHA256:
        message = 'panel_judge: frozen calibration hash changed'
        raise ValueError(message)
    source_sha = hashlib.sha256(source.read_bytes()).hexdigest()
    calibration = load_calibration(CALIBRATION)
    prepared = tuple(prepare_panel(panel) for panel in panels)
    output.mkdir(parents=True, exist_ok=True)
    records: list[PanelVerdict] = []
    for index, (original, panel) in enumerate(zip(panels, prepared, strict=True)):
        path = output / f'panel_{index:02d}.png'
        panel.image.save(path)
        verdict = judge(path, calibration, family='person')
        records.append(PanelVerdict(index, original.size,
            hashlib.sha256(original.convert('RGBA').tobytes()).hexdigest(),
            panel.crop_box, panel.padding, panel.image.size, verdict))
    result = SheetVerdict(str(source), source_sha, calibration_sha, expected_count,
                         tuple(records), all(record.verdict.numeric_pass for record in records))
    _ = (output / 'panel_judge.json').write_text(json.dumps(asdict(result), indent=2) + '\n')
    return result
