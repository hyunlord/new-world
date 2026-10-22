# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run art/pipeline2/ground_v2.py DATA_ROOT PERSON_ANCHOR
"""Apply the authorized global ground tone cap, preserving original seam limits."""
from __future__ import annotations

import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

import numpy as np
from numpy.typing import NDArray
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from art.pipeline2.ground import Seam, measure
from art.pipeline2.judge import foreground


@dataclass(frozen=True, slots=True)
class Tone:
    luminance_std: float
    saturation_mean: float


@dataclass(frozen=True, slots=True)
class Result:
    name: str
    source: str
    source_sha256: str
    person_sha256: str
    person_luminance_std: float
    cap: float
    before: Tone
    after: Tone
    contrast_scale: float
    saturation_scale: float
    seam: Seam
    tone_pass: bool
    mirror_blend: bool = False
    style_acceptance: str = "NOT_VERIFIED"
    comparison_size: int = 512


def luminance(rgb: NDArray[np.float64]) -> NDArray[np.float64]:
    return rgb[:, :, 0] * 0.2125 + rgb[:, :, 1] * 0.7154 + rgb[:, :, 2] * 0.0721


def saturation_mean(rgb: NDArray[np.float64]) -> float:
    maximum = rgb.max(axis=2)
    saturation = np.divide(maximum - rgb.min(axis=2), maximum,
                           out=np.zeros_like(maximum), where=maximum > 0)
    return float(saturation.mean())


def tone(image: Image.Image) -> Tone:
    """Measure full-bleed ground using normalized sRGB luminance and HSV."""
    pixels = np.asarray(image.convert("RGB"), dtype=np.float64) / 255
    return Tone(float(luminance(pixels).std()), saturation_mean(pixels))


def cap_tone(image: Image.Image, cap: float) -> tuple[Image.Image, float, float]:
    """Reduce global contrast and chroma only; never quantize or force contours."""
    pixels = np.asarray(image.convert("RGB"), dtype=np.float64) / 255
    gray = luminance(pixels)
    contrast = min(1.0, max(0.0, cap - 1 / 255) / max(float(gray.std()), 1e-12))
    pixels = pixels.mean(axis=(0, 1)) + contrast * (pixels - pixels.mean(axis=(0, 1)))
    gray = luminance(pixels)[:, :, None]
    # Evaluate final 8-bit pixels inside bisection: flat-color rounding can exceed 0.35.
    low, high = 0.0, 1.0
    for _ in range(24):
        middle = (low + high) / 2
        trial = np.rint((gray + middle * (pixels - gray)) * 255).clip(0, 255) / 255
        if saturation_mean(trial) <= 0.348:
            low = middle
        else:
            high = middle
    final = np.rint((gray + low * (pixels - gray)) * 255).clip(0, 255)
    return Image.fromarray(final.astype(np.uint8)), contrast, low


def process(source: Path, anchor: Path, destination: Path) -> Result:
    """Write a candidate and repeat with reproducible metrics, never auto-accept."""
    anchor_rgb, anchor_mask = foreground(anchor, 512)
    person_std = float(luminance(anchor_rgb)[anchor_mask].std())
    with Image.open(source) as original:
        working = original.convert("RGB").resize((512, 512), Image.Resampling.LANCZOS)
    tile, contrast, saturation = cap_tone(working, person_std * 0.6)
    after = tone(tile)
    result = Result(source.parent.name, str(source), hashlib.sha256(source.read_bytes()).hexdigest(),
                    hashlib.sha256(anchor.read_bytes()).hexdigest(), person_std, person_std * 0.6,
                    tone(working), after, contrast, saturation, measure(tile),
                    after.luminance_std <= person_std * 0.6 and after.saturation_mean <= 0.35)
    destination.mkdir(parents=True, exist_ok=True)
    tile.save(destination / "candidate.png")
    repeat = Image.new("RGB", (1536, 1536))
    for y in range(3):
        for x in range(3):
            repeat.paste(tile, (x * 512, y * 512))
    repeat.save(destination / "repeat_3x3.png")
    _ = (destination / "metrics.json").write_text(json.dumps(asdict(result), indent=2) + "\n")
    return result


if __name__ == "__main__":
    root, person = Path(sys.argv[1]), Path(sys.argv[2])
    (root / "working").mkdir(exist_ok=True)
    results: list[Result] = []
    for folder in sorted(root.iterdir()):
        candidates = sorted(folder.glob("try[1-3]_raw.png"))
        for raw in candidates:
            output = folder / f"{raw.stem.removesuffix('_raw')}_capped"
            result = process(raw, person, output)
            results.append(result)
            if raw == candidates[-1]:
                _ = (root / "working" / f"{result.name}.png").write_bytes((output / "candidate.png").read_bytes())
    _ = (root / "tone_results.json").write_text(json.dumps([asdict(row) for row in results], indent=2) + "\n")
    print(json.dumps([asdict(row) for row in results], indent=2))
