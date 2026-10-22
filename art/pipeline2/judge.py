# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run art/pipeline2/judge.py calibrate --output calibration-v2.json
"""Family-conditioned style judge; no global colour histogram or content palette.

Model evidence must independently establish two same-family references. Numeric
calibration is not semantic acceptance, and missing review always fails closed.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image
from scipy.ndimage import (
    binary_closing,
    binary_erosion,
    distance_transform_edt,
    uniform_filter,
)
from skimage.color import rgb2gray, rgb2hsv
from skimage.feature import canny
from skimage.measure import label
from skimage.morphology import skeletonize

SIZE: Final = 256
FloatArray = NDArray[np.float64]
FEATURES: Final = ("edge_density", "saturation_q25", "saturation_q75", "stroke_q25", "stroke_q50", "stroke_q75", "stroke_q90", "wash_flatness")
# Frozen before candidate scoring. Multiplicative factors allow texture variation;
# absolute floors avoid near-zero feature instability. Never fit to candidates.
FLOORS: Final = (0.04, 0.12, 0.12, 1.0, 2.0, 2.0, 1.0, 0.003)

@dataclass(frozen=True, slots=True)
class Metrics:
    edge_density: float
    saturation_q25: float
    saturation_q75: float
    stroke_q25: float
    stroke_q50: float
    stroke_q75: float
    stroke_q90: float
    wash_flatness: float
    foreground_pixels: int

@dataclass(frozen=True, slots=True)
class Anchor:
    name: str
    family: str
    file: str
    sha256: str
    metrics: Metrics

@dataclass(frozen=True, slots=True)
class FamilyRange:
    family: str
    lower: list[float]
    upper: list[float]

@dataclass(frozen=True, slots=True)
class Fold:
    name: str
    family: str
    passed: bool
    checks: dict[str, bool]

@dataclass(frozen=True, slots=True)
class Calibration:
    anchors: list[Anchor]
    ranges: list[FamilyRange]
    leave_one_out: list[Fold]
    comparison_size: int = SIZE
    version: int = 2

@dataclass(frozen=True, slots=True)
class ModelScores:
    medium: int
    line: int
    light: int
    palette: int
    sentence: str

    def __post_init__(self) -> None:
        scores = (self.medium, self.line, self.light, self.palette)
        if any(type(score) is not int or not 1 <= score <= 5 for score in scores):
            message = "Model scores must be four integers in 1..5"
            raise ValueError(message)
        if not self.sentence.strip():
            message = "Model judge must supply its actual explanation"
            raise ValueError(message)

    @property
    def passed(self) -> bool:
        return min(self.medium, self.line, self.light, self.palette) >= 4


@dataclass(frozen=True, slots=True)
class Verdict:
    candidate: str
    sha256: str
    family: str
    metrics: Metrics
    numeric_checks: dict[str, bool]
    numeric_pass: bool
    model_scores: ModelScores | None
    model_status: str
    passed: bool

def foreground(path: Path, comparison_size: int = SIZE) -> tuple[FloatArray, NDArray[np.bool_]]:
    """Remove alpha/chroma background for measurement only; never modify assets."""
    with Image.open(path) as source:
        rgba: NDArray[np.uint8] = np.asarray(source.convert("RGBA"))
    rgb = rgba[:, :, :3].astype(np.float64) / 255.0
    mask = (rgba[:, :, 3] >= 128) & ~(
        (rgb[:, :, 0] > 0.65) & (rgb[:, :, 2] > 0.65)
        & (rgb[:, :, 1] < 0.4) & (np.abs(rgb[:, :, 0] - rgb[:, :, 2]) < 0.3)
    )
    rows, cols = np.nonzero(mask)
    if rows.size < 16:
        message = f"Insufficient visible foreground: {path}"
        raise ValueError(message)
    bounds = np.s_[rows.min():rows.max() + 1, cols.min():cols.max() + 1]
    crop: NDArray[np.uint8] = rgba[bounds].copy()
    crop[:, :, 3] = mask[bounds] * 255
    height, width = crop.shape[:2]
    size = (max(1, round(width * comparison_size / max(width, height))),
            max(1, round(height * comparison_size / max(width, height))))
    scaled = np.asarray(Image.fromarray(crop).resize(size, Image.Resampling.LANCZOS))
    visible = scaled[:, :, 3] >= 128
    return scaled[:, :, :3].astype(np.float64) / 255.0, visible


def measure(path: Path) -> Metrics:
    """Measure local saturation, edge-bounded wash variance and ink widths.

    Stroke widths are distance-transform samples on the skeleton of dark ridges
    within four pixels of a Canny edge, normalized at 1024px. Canny itself is
    one pixel wide, so its direct transform would measure detector width only.
    """
    rgb, mask = foreground(path)
    gray: FloatArray = rgb2gray(rgb)
    interior = np.asarray(binary_erosion(mask, iterations=2), dtype=np.bool_)
    if int(interior.sum()) < 16:
        message = f"Insufficient foreground interior: {path}"
        raise ValueError(message)
    edges: NDArray[np.bool_] = canny(gray, sigma=1, low_threshold=0.08, high_threshold=0.20, mask=interior)
    weights: FloatArray = uniform_filter(mask.astype(np.float64), size=9)
    saturation: FloatArray = uniform_filter(rgb2hsv(rgb)[:, :, 1] * mask, size=9)
    sq = np.quantile(saturation[interior] / weights[interior], [0.25, 0.75])
    closed = np.asarray(binary_closing(edges, iterations=1), dtype=np.bool_)
    regions: NDArray[np.int64] = np.asarray(label(interior & ~closed, connectivity=1))
    count = int(regions.max())
    variances: list[tuple[int, float]] = []
    for region in range(1, count + 1):
        region_mask = regions == region
        # Retain region interiors: edge antialiasing is not pigment wash.
        region_interior = np.asarray(binary_erosion(region_mask, iterations=1), dtype=np.bool_)
        pixels = gray[region_interior]
        if pixels.size >= 16:
            variances.append((int(pixels.size), float(np.var(pixels))))
    flatness = sum(n * v for n, v in variances) / max(1, sum(n for n, _ in variances))
    srgb, smask = foreground(path, 1024)
    lum: FloatArray = rgb2gray(srgb)
    sedges: NDArray[np.bool_] = canny(lum, sigma=1, low_threshold=0.08, high_threshold=0.20, mask=smask)
    ink: NDArray[np.bool_] = (lum < uniform_filter(lum, size=15) - 0.12) & np.asarray(binary_erosion(smask, iterations=2), dtype=np.bool_)
    ink &= np.asarray(distance_transform_edt(~sedges), dtype=np.float64) <= 4
    widths = 2 * np.asarray(distance_transform_edt(ink), dtype=np.float64)[skeletonize(ink)]
    quantiles = np.quantile(widths, [0.25, 0.5, 0.75, 0.9]) if widths.size else np.zeros(4)
    return Metrics(float(edges.sum() / interior.sum()), float(sq[0]), float(sq[1]),
                   float(quantiles[0]), float(quantiles[1]), float(quantiles[2]), float(quantiles[3]),
                   flatness, int(mask.sum()))


def bounds(family: str, anchors: list[Anchor]) -> FamilyRange:
    """Fit only same-family references with preregistered tolerances."""
    values = np.asarray([[getattr(a.metrics, key) for key in FEATURES] for a in anchors])
    minimum, maximum = values.min(axis=0), values.max(axis=0)
    margin = np.maximum((maximum + minimum) * 0.25, FLOORS)
    # Region variance is positive and scale-like; permit a 120% ratio variation.
    lower, upper = np.maximum(0, minimum - margin), maximum + margin
    lower[-1], upper[-1] = minimum[-1] / 2.2, maximum[-1] * 2.2
    # Upper tail avoids the 2px quantization shared by fine-line medians.
    tail_margin = max(1.75, float((minimum[-2] + maximum[-2]) * 0.125))
    lower[-2], upper[-2] = minimum[-2] - tail_margin, maximum[-2] + tail_margin
    return FamilyRange(family, lower.tolist(), upper.tolist())


def compare(metrics: Metrics, reference: FamilyRange) -> dict[str, bool]:
    """All eight measured components must lie inside the reference bounds."""
    return {key: low - 1e-12 <= getattr(metrics, key) <= high + 1e-12
            for key, low, high in zip(FEATURES, reference.lower, reference.upper, strict=True)}


def calibrate(directory: Path) -> Calibration:
    """Fit family references and evaluate true held-out crops against the others."""
    manifest = json.loads((directory / "manifest.json").read_text())
    if manifest["version"] != 2:
        message = "Requires crop manifest version 2"
        raise ValueError(message)
    anchors: list[Anchor] = []
    for entry in manifest["crops"]:
        if entry["status"] != "active":
            continue
        path = directory / entry["file"]
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if digest != entry["sha256"]:
            message = f"Anchor hash changed: {path}"
            raise ValueError(message)
        anchors.append(Anchor(entry["name"], entry["family"], entry["file"], digest, measure(path)))
    ranges: list[FamilyRange] = []
    folds: list[Fold] = []
    for family in sorted({a.family for a in anchors}):
        group = [a for a in anchors if a.family == family]
        if len(group) < 2:
            message = f"Two same-family crops required: {family}"
            raise ValueError(message)
        ranges.append(bounds(family, group))
        for held in group:
            checks = compare(held.metrics, bounds(family, [a for a in group if a.name != held.name]))
            folds.append(Fold(held.name, family, all(checks.values()), checks))
    return Calibration(anchors, ranges, folds)


def judge(candidate: Path, calibration: Calibration, model: ModelScores | None = None,
          *, family: str, model_family_verified: bool = False) -> Verdict:
    """A supplied legacy score cannot establish same-family model acceptance."""
    references = [r for r in calibration.ranges if r.family == family]
    if len(references) != 1:
        message = f"Unknown or ambiguous anchor family: {family}"
        raise ValueError(message)
    metrics = measure(candidate)
    checks = compare(metrics, references[0])
    numeric = all(checks.values())
    status = "MISSING" if model is None else "SAME_FAMILY_VERIFIED" if model_family_verified else "LEGACY_REFERENCE_UNVERIFIED"
    return Verdict(str(candidate), hashlib.sha256(candidate.read_bytes()).hexdigest(), family,
                   metrics, checks, numeric, model, status,
                   numeric and model is not None and model.passed and model_family_verified
                   and len(calibration.leave_one_out) == len(calibration.anchors)
                   and bool(calibration.leave_one_out) and all(f.passed for f in calibration.leave_one_out))


def load_calibration(path: Path) -> Calibration:
    """Reject incompatible v1 global-histogram calibrations."""
    data = json.loads(path.read_text())
    if data["version"] != 2 or data["comparison_size"] != SIZE:
        message = "Calibration schema or comparison size mismatch"
        raise ValueError(message)
    return Calibration([Anchor(a["name"], a["family"], a["file"], a["sha256"], Metrics(**a["metrics"]))
                        for a in data["anchors"]], [FamilyRange(**r) for r in data["ranges"]],
                       [Fold(**f) for f in data["leave_one_out"]])


def main() -> int:
    """Emit reproducible calibration or fail-closed candidate JSON."""
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    cal = commands.add_parser("calibrate")
    cal.add_argument("--anchors", type=Path, default=Path("art/style05/anchors"))
    cal.add_argument("--output", required=True, type=Path)
    candidate = commands.add_parser("judge")
    candidate.add_argument("candidate", type=Path)
    candidate.add_argument("--family", required=True)
    candidate.add_argument("--calibration", required=True, type=Path)
    candidate.add_argument("--model-scores", type=Path)
    candidate.add_argument("--model-family-verified", action="store_true")
    candidate.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if args.command == "calibrate":
        result = calibrate(args.anchors)
        _ = args.output.write_text(json.dumps(asdict(result), indent=2) + "\n")
        return 0 if all(f.passed for f in result.leave_one_out) else 1
    model = ModelScores(**json.loads(args.model_scores.read_text())) if args.model_scores else None
    verdict = judge(args.candidate, load_calibration(args.calibration), model,
                    family=args.family, model_family_verified=args.model_family_verified)
    _ = args.output.write_text(json.dumps(asdict(verdict), indent=2) + "\n")
    return 0 if verdict.passed else 1

if __name__ == "__main__":
    raise SystemExit(main())
