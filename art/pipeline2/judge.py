# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: python art/pipeline2/judge.py calibrate --output calibration.json
"""Frozen-anchor numeric style measurements plus a fail-closed external model vote.

All images use the same foreground crop and 256-px longest-edge normalization.
This removes canvas-size bias, not information loss in tiny working sprites.
Wash is local HSV saturation, not physically measurable pigment opacity.
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
from scipy.ndimage import binary_erosion, distance_transform_edt, uniform_filter
from skimage.color import lab2rgb, rgb2gray, rgb2hsv, rgb2lab
from skimage.feature import canny
from skimage.morphology import skeletonize

NAMES: Final = ("person", "hut", "props", "ground", "scene", "portrait")
SIZE: Final = 256
FloatArray = NDArray[np.float64]


@dataclass(frozen=True, slots=True)
class Metrics:
    histogram: list[float]
    edge_density: float
    wash: float
    stroke_width_at_1024: float
    foreground_pixels: int


@dataclass(frozen=True, slots=True)
class Anchor:
    name: str
    sha256: str
    metrics: Metrics


@dataclass(frozen=True, slots=True)
class Calibration:
    anchors: list[Anchor]
    histogram: list[float]
    histogram_limit: float
    palette_lab: list[list[float]]
    loo_distances: list[float]
    edge_range: tuple[float, float]
    wash_range: tuple[float, float]
    comparison_size: int = SIZE
    version: int = 1


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
    metrics: Metrics
    histogram_distance: float
    numeric_checks: dict[str, bool]
    numeric_pass: bool
    model_scores: ModelScores | None
    model_status: str
    passed: bool


def foreground(path: Path, comparison_size: int = SIZE) -> tuple[FloatArray, NDArray[np.bool_]]:
    """Remove alpha/chroma background for measurement only; never modify assets."""
    with Image.open(path) as source:
        rgba = np.asarray(source.convert("RGBA"))
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
    crop = rgba[bounds].copy()
    crop[:, :, 3] = mask[bounds] * 255
    height, width = crop.shape[:2]
    size = (max(1, round(width * comparison_size / max(width, height))),
            max(1, round(height * comparison_size / max(width, height))))
    scaled = np.asarray(Image.fromarray(crop).resize(size, Image.Resampling.LANCZOS))
    visible = scaled[:, :, 3] >= 128
    return scaled[:, :, :3].astype(np.float64) / 255.0, visible


def measure(path: Path, palette: list[list[float]] | None = None) -> Metrics:
    """Canny luminance edges, Lab histogram, local saturation and stroke estimate."""
    rgb, mask = foreground(path)
    luminance = rgb2gray(rgb)
    interior = np.asarray(binary_erosion(mask, iterations=2), dtype=np.bool_)
    if int(interior.sum()) < 16:
        message = f"Insufficient foreground interior: {path}"
        raise ValueError(message)
    edges = canny(luminance, sigma=1.0, low_threshold=0.08,
                  high_threshold=0.20, mask=interior)
    histogram, _ = np.histogramdd(rgb2lab(rgb)[mask], bins=(8, 8, 8),
                                range=((0, 100), (-128, 128), (-128, 128)))
    histogram = histogram.ravel() / histogram.sum()
    if palette is not None:
        histogram = palette_histogram(rgb2lab(rgb)[mask], palette)
    weights = uniform_filter(mask.astype(np.float64), size=9)
    saturation = uniform_filter(rgb2hsv(rgb)[:, :, 1] * mask, size=9)
    wash = float(np.mean(saturation[interior] / weights[interior]))
    # Measure strokes independently at 1024 px; 256-px edges lose fine ink.
    stroke_rgb, stroke_mask = foreground(path, 1024)
    stroke_luminance = rgb2gray(stroke_rgb)
    stroke_edges = canny(stroke_luminance, sigma=1.0, low_threshold=0.08,
                         high_threshold=0.20, mask=stroke_mask)
    local = uniform_filter(stroke_luminance, size=15)
    stroke_interior = np.asarray(binary_erosion(stroke_mask, iterations=2), dtype=np.bool_)
    ink = (stroke_luminance < local - 0.12) & stroke_interior
    edge_distance = np.asarray(distance_transform_edt(~stroke_edges), dtype=np.float64)
    ink &= edge_distance <= 4
    skeleton = skeletonize(ink)
    ink_distance = np.asarray(distance_transform_edt(ink), dtype=np.float64)
    widths = 2 * ink_distance[skeleton]
    stroke = float(np.mean(widths)) if widths.size else 0.0
    return Metrics(histogram.tolist(), float(edges.sum() / interior.sum()),
                   wash, stroke, int(mask.sum()))


def chi_square(left: list[float], right: list[float]) -> float:
    """Symmetric normalized chi-square in [0, 1]."""
    a, b = np.asarray(left), np.asarray(right)
    return float(0.5 * np.sum((a - b) ** 2 / (a + b + 1e-15)))


def dominant_palette(samples: FloatArray) -> list[list[float]]:
    """Select 24 most populous Lab cells and their measured colour centroids."""
    bins = np.floor((samples + [0, 128, 128]) / [100 / 12, 256 / 12, 256 / 12]).astype(int)
    bins = np.clip(bins, 0, 11)
    keys = bins[:, 0] * 144 + bins[:, 1] * 12 + bins[:, 2]
    counts = np.bincount(keys, minlength=1728)
    ranked = np.argsort(-counts, kind="stable")[:24]
    return [samples[keys == key].mean(axis=0).tolist() for key in ranked if counts[key] > 0]


def palette_histogram(samples: FloatArray, palette: list[list[float]]) -> FloatArray:
    """Assign measured colours to nearest reference Lab centroid, without editing art."""
    distance = np.sum((samples[:, None, :] - np.asarray(palette)[None, :, :]) ** 2, axis=2)
    counts = np.bincount(distance.argmin(axis=1), minlength=len(palette)).astype(np.float64)
    return counts / counts.sum()


def calibrate(directory: Path) -> Calibration:
    """Rebuild a 24-colour palette on the other five anchors for every LOO fold."""
    samples = []
    for name in NAMES:
        rgb, mask = foreground(directory / f"{name}.png")
        samples.append(rgb2lab(rgb)[mask])
    palette = dominant_palette(np.concatenate(samples))
    anchors = [Anchor(name, hashlib.sha256((directory / f"{name}.png").read_bytes()).hexdigest(),
                      measure(directory / f"{name}.png", palette)) for name in NAMES]
    loo = []
    for index in range(6):
        others = [sample for position, sample in enumerate(samples) if position != index]
        fold_palette = dominant_palette(np.concatenate(others))
        reference = np.mean([palette_histogram(sample, fold_palette) for sample in others], axis=0)
        loo.append(chi_square(palette_histogram(samples[index], fold_palette).tolist(), reference.tolist()))
    edges = [anchor.metrics.edge_density for anchor in anchors]
    washes = [anchor.metrics.wash for anchor in anchors]
    histograms = np.asarray([anchor.metrics.histogram for anchor in anchors])
    return Calibration(anchors, histograms.mean(axis=0).tolist(), max(loo), palette, loo,
                       (min(edges) * 0.7, max(edges) * 1.3), (min(washes), max(washes)))


def judge(candidate: Path, calibration: Calibration,
          model: ModelScores | None = None) -> Verdict:
    """Both independent signals must pass; absent model review cannot pass."""
    metrics = measure(candidate, calibration.palette_lab)
    distance = chi_square(metrics.histogram, calibration.histogram)
    checks = {
        "lab_histogram": distance <= calibration.histogram_limit + 1e-12,
        "line_density": calibration.edge_range[0] <= metrics.edge_density <= calibration.edge_range[1],
        "local_saturation": calibration.wash_range[0] - 1e-12 <= metrics.wash <= calibration.wash_range[1] + 1e-12,
    }
    numeric = all(checks.values())
    return Verdict(str(candidate), hashlib.sha256(candidate.read_bytes()).hexdigest(),
                   metrics, distance, checks, numeric, model,
                   "MISSING" if model is None else "SUPPLIED",
                   numeric and model is not None and model.passed)


def load_calibration(path: Path) -> Calibration:
    """Read the generated calibration including immutable anchor provenance."""
    data = json.loads(path.read_text())
    anchors = [Anchor(item["name"], item["sha256"], Metrics(**item["metrics"]))
               for item in data["anchors"]]
    if len(anchors) != 6 or data["comparison_size"] != SIZE or data["version"] != 1:
        message = "Calibration schema or comparison size mismatch"
        raise ValueError(message)
    return Calibration(anchors, data["histogram"], data["histogram_limit"], data["palette_lab"],
                       data["loo_distances"], tuple(data["edge_range"]), tuple(data["wash_range"]))


def main() -> int:
    """Emit reproducible calibration or candidate JSON; rejection exits 1."""
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    calibration_parser = commands.add_parser("calibrate")
    calibration_parser.add_argument("--anchors", type=Path, default=Path("art/style05/anchors"))
    calibration_parser.add_argument("--output", required=True, type=Path)
    calibration_parser.add_argument("--palette-output", type=Path)
    candidate_parser = commands.add_parser("judge")
    candidate_parser.add_argument("candidate", type=Path)
    candidate_parser.add_argument("--calibration", required=True, type=Path)
    candidate_parser.add_argument("--model-scores", type=Path)
    candidate_parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if args.command == "calibrate":
        result = calibrate(args.anchors)
        args.output.write_text(json.dumps(asdict(result), indent=2) + "\n")
        if args.palette_output is not None:
            rgb = np.round(lab2rgb(np.asarray(result.palette_lab)[None, :, :])[0] * 255).astype(np.uint8)
            palette_data = {"purpose": "judging only; never quantize assets", "lab": result.palette_lab,
                            "rgb": rgb.tolist(), "anchor_sha256": {a.name: a.sha256 for a in result.anchors}}
            args.palette_output.write_text(json.dumps(palette_data, indent=2) + "\n")
            swatch = np.repeat(np.repeat(rgb.reshape(4, 6, 3), 64, axis=0), 64, axis=1)
            Image.fromarray(swatch).save(args.palette_output.with_suffix(".png"))
        return 0
    model = ModelScores(**json.loads(args.model_scores.read_text())) if args.model_scores else None
    verdict = judge(args.candidate, load_calibration(args.calibration), model)
    args.output.write_text(json.dumps(asdict(verdict), indent=2) + "\n")
    return 0 if verdict.passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
