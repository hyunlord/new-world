# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow"]
# ///
# How to run: python -m art.batch_r1.metrics compare REF CAND REGIONS --output FILE
"""Fail-closed R1 measurements from reviewed reference pixel annotations.

Rows sample the complete alpha>=128 silhouette. Clothing is a fixed, explicit
interior mask, never inferred from skin or colour. Each contiguous outline probe
starts outside the silhouette; effective ink integrates its first two occupied pixels.
"""

from __future__ import annotations

import argparse
import colorsys
import json
from collections.abc import Callable
from dataclasses import asdict, dataclass
from itertools import pairwise
from pathlib import Path
from statistics import mean
from typing import TypeAlias

from PIL import Image, ImageDraw

Point: TypeAlias = tuple[int, int]


class MeasurementError(ValueError):
    """Required annotation or measured evidence is missing or invalid."""


@dataclass(frozen=True, slots=True)
class FrameRegions:
    """Inspect these fixed reference annotations before evaluating candidates."""

    shoulder_rows: tuple[int, ...]
    hip_rows: tuple[int, ...]
    clothing: frozenset[Point]
    outline_probes: tuple[tuple[Point, ...], ...]
    outline_max_value: int


@dataclass(frozen=True, slots=True)
class FrameMetrics:
    shoulder_width: float
    hip_width: float
    clothing_saturation: float
    clothing_value: float
    outline_thickness: float


@dataclass(frozen=True, slots=True)
class Comparison:
    reference: FrameMetrics
    candidate: FrameMetrics
    ratios: FrameMetrics
    passed: bool


def _measure_functions(
    image: Image.Image, regions: FrameRegions
) -> tuple[Callable[[], float], ...]:
    """Share pixel reading across independently diagnosable measurements."""
    width, height = image.size
    pixels = image.convert("RGBA").tobytes()

    def pixel(point: Point) -> tuple[int, int, int, int]:
        x, y = point
        if not (0 <= x < width and 0 <= y < height):
            raise MeasurementError(f"out-of-frame annotation: {point}")
        offset = (y * width + x) * 4
        return (
            pixels[offset],
            pixels[offset + 1],
            pixels[offset + 2],
            pixels[offset + 3],
        )

    def row_width(rows: tuple[int, ...]) -> float:
        if not rows or len(set(rows)) != len(rows):
            raise MeasurementError("scan rows must be nonempty and unique")
        spans: list[int] = []
        for y in rows:
            xs = [x for x in range(width) if pixel((x, y))[3] >= 128]
            if not xs:
                raise MeasurementError(f"empty silhouette scanline: {y}")
            spans.append(xs[-1] - xs[0] + 1)
        return mean(spans)

    def clothing_hsv(channel: int) -> float:
        if not regions.clothing:
            raise MeasurementError("clothing mask must be nonempty")
        hsv: list[tuple[float, float, float]] = []
        for point in sorted(regions.clothing):
            r, g, b, alpha = pixel(point)
            if alpha < 128:
                continue
            hsv.append(colorsys.rgb_to_hsv(r / 255, g / 255, b / 255))
        if len(hsv) * 10 < len(regions.clothing) * 9:
            raise MeasurementError(
                f"clothing coverage below 90%: {len(hsv)}/{len(regions.clothing)}"
            )
        return mean(v[channel] for v in hsv)

    def binary_thickness() -> float:
        if not regions.outline_probes or not 0 <= regions.outline_max_value < 255:
            raise MeasurementError(
                "outline probes must be nonempty; cutoff must be 0..254"
            )
        thicknesses: list[int] = []
        for probe in regions.outline_probes:
            if len(probe) < 2:
                raise MeasurementError("outline probe needs ink and lighter fill")
            dx, dy = probe[1][0] - probe[0][0], probe[1][1] - probe[0][1]
            if abs(dx) + abs(dy) != 1 or any(
                (b[0] - a[0], b[1] - a[1]) != (dx, dy) for a, b in pairwise(probe)
            ):
                raise MeasurementError(
                    "outline probe must be a straight contiguous line"
                )
            outer = probe[0][0] - dx, probe[0][1] - dy
            if (
                0 <= outer[0] < width
                and 0 <= outer[1] < height
                and pixel(outer)[3] >= 128
            ):
                raise MeasurementError("outline probe must start at silhouette edge")
            run = 0
            found_fill = False
            started = False
            for point in probe:
                r, g, b, alpha = pixel(point)
                if alpha < 128:
                    if started:
                        raise MeasurementError(
                            "outline probe crosses transparent pixels after ink"
                        )
                    continue
                started = True
                if max(r, g, b) > regions.outline_max_value:
                    found_fill = True
                    break
                run += 1
            if run == 0 or not found_fill:
                raise MeasurementError(
                    "outline probe needs a nonempty ink run and lighter fill"
                )
            thicknesses.append(run)
        return mean(thicknesses)

    def effective_thickness() -> float:
        if not regions.outline_probes:
            raise MeasurementError("outline probes must be nonempty")
        thicknesses: list[float] = []
        for probe in regions.outline_probes:
            if len(probe) < 2:
                raise MeasurementError("outline probe needs two occupied edge pixels")
            dx, dy = probe[1][0] - probe[0][0], probe[1][1] - probe[0][1]
            if abs(dx) + abs(dy) != 1 or any(
                (b[0] - a[0], b[1] - a[1]) != (dx, dy) for a, b in pairwise(probe)
            ):
                raise MeasurementError(
                    "outline probe must be a straight contiguous line"
                )
            occupied = [pixel(point) for point in probe if pixel(point)[3] >= 16]
            if len(occupied) < 2:
                raise MeasurementError("outline probe needs two occupied edge pixels")
            thicknesses.append(
                sum(a / 255 * (1 - max(r, g, b) / 255) for r, g, b, a in occupied[:2])
            )
        return mean(thicknesses)

    return (
        lambda: row_width(regions.shoulder_rows),
        lambda: row_width(regions.hip_rows),
        lambda: clothing_hsv(1),
        lambda: clothing_hsv(2),
        effective_thickness,
        binary_thickness,
    )


def binary_outline_thickness(image: Image.Image, regions: FrameRegions) -> float:
    """Legacy integer dark-run thickness, diagnostic only; not an acceptance metric."""
    return _measure_functions(image, regions)[5]()


def measure_frame(image: Image.Image, regions: FrameRegions) -> FrameMetrics:
    """Measure all fields strictly; missing evidence raises MeasurementError."""
    return FrameMetrics(
        *(measure() for measure in _measure_functions(image, regions)[:5])
    )


@dataclass(frozen=True, slots=True)
class DiagnosticMetrics:
    shoulder_width: float | None
    hip_width: float | None
    clothing_saturation: float | None
    clothing_value: float | None
    outline_thickness: float | None


@dataclass(frozen=True, slots=True)
class FrameDiagnostics:
    metrics: DiagnosticMetrics
    errors: tuple[str, ...]


def diagnose_frame(image: Image.Image, regions: FrameRegions) -> FrameDiagnostics:
    """Return independent field values or null plus errors; never certifies a pass."""
    values: list[float | None] = []
    errors: list[str] = []
    names = (
        "shoulder_width",
        "hip_width",
        "clothing_saturation",
        "clothing_value",
        "outline_thickness",
    )
    for name, measure in zip(
        names, _measure_functions(image, regions)[:5], strict=True
    ):
        try:
            values.append(measure())
        except MeasurementError as error:
            values.append(None)
            errors.append(f"{name}: {error}")
    return FrameDiagnostics(DiagnosticMetrics(*values), tuple(errors))


def compare_frames(
    reference: Image.Image, candidate: Image.Image, regions: FrameRegions
) -> Comparison:
    """Require every metric within the inclusive shipped-reference ±10% band."""
    if reference.size != candidate.size:
        raise MeasurementError("matching frames must have identical canvas dimensions")
    baseline, measured = (
        measure_frame(reference, regions),
        measure_frame(candidate, regions),
    )
    ratios: list[float] = []
    for before, after in zip(
        (
            baseline.shoulder_width,
            baseline.hip_width,
            baseline.clothing_saturation,
            baseline.clothing_value,
            baseline.outline_thickness,
        ),
        (
            measured.shoulder_width,
            measured.hip_width,
            measured.clothing_saturation,
            measured.clothing_value,
            measured.outline_thickness,
        ),
        strict=True,
    ):
        if before == 0:
            raise MeasurementError("reference metrics must be nonzero")
        ratios.append(after / before)
    return Comparison(
        baseline,
        measured,
        FrameMetrics(*ratios),
        all(0.9 - 1e-12 <= ratio <= 1.1 + 1e-12 for ratio in ratios),
    )


Json: TypeAlias = str | int | float | bool | None | list["Json"] | dict[str, "Json"]
_decode: Callable[[str], Json] = json.loads


def read_regions(path: Path) -> FrameRegions:
    """Read integer-only annotations; reject malformed input at this boundary."""
    raw = _decode(path.read_text())
    if not isinstance(raw, dict):
        raise MeasurementError("region JSON must be an object")

    def integers(value: Json) -> tuple[int, ...]:
        if not isinstance(value, list):
            raise MeasurementError("expected an integer array")
        result: list[int] = []
        for item in value:
            if not isinstance(item, int) or isinstance(item, bool):
                raise MeasurementError("coordinates and cutoff must be integers")
            result.append(item)
        return tuple(result)

    def points(value: Json) -> tuple[Point, ...]:
        if not isinstance(value, list):
            raise MeasurementError("expected a point array")
        result: list[Point] = []
        for item in value:
            point = integers(item)
            if len(point) != 2:
                raise MeasurementError("point must contain exactly x,y")
            result.append((point[0], point[1]))
        return tuple(result)

    try:
        cutoff = raw["outline_max_value"]
        probes = raw["outline_probes"]
        if (
            not isinstance(cutoff, int)
            or isinstance(cutoff, bool)
            or not isinstance(probes, list)
        ):
            raise MeasurementError("invalid cutoff or outline probe array")
        return FrameRegions(
            integers(raw["shoulder_rows"]),
            integers(raw["hip_rows"]),
            frozenset(points(raw["clothing"])),
            tuple(points(p) for p in probes),
            cutoff,
        )
    except KeyError as error:
        raise MeasurementError(f"missing annotation key: {error}") from error


def write_regions(path: Path, regions: FrameRegions) -> None:
    """Persist annotations deterministically for review and reuse."""
    _ = path.write_text(
        json.dumps(
            {
                "shoulder_rows": regions.shoulder_rows,
                "hip_rows": regions.hip_rows,
                "clothing": sorted(regions.clothing),
                "outline_probes": regions.outline_probes,
                "outline_max_value": regions.outline_max_value,
            },
            indent=2,
        )
        + "\n"
    )


def render_masks(image: Image.Image, regions: FrameRegions) -> Image.Image:
    """Overlay clothing in cyan, rows in green/magenta, and probes in yellow."""
    result = image.convert("RGBA")
    draw = ImageDraw.Draw(result)
    for y in regions.shoulder_rows:
        draw.line((0, y, image.width - 1, y), fill=(0, 255, 0, 255))
    for y in regions.hip_rows:
        draw.line((0, y, image.width - 1, y), fill=(255, 0, 255, 255))
    for point in regions.clothing:
        draw.point(point, fill=(0, 255, 255, 255))
    for probe in regions.outline_probes:
        for point in probe:
            draw.point(point, fill=(255, 255, 0, 255))
    return result


class Arguments(argparse.Namespace):
    command: str = ""
    reference: Path = Path()
    candidate: Path = Path()
    regions: Path = Path()
    output: Path = Path()


def main() -> int:
    """Compare a matching pair or export a review overlay."""
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    compare = commands.add_parser("compare")
    _ = compare.add_argument("reference", type=Path)
    _ = compare.add_argument("candidate", type=Path)
    _ = compare.add_argument("regions", type=Path)
    _ = compare.add_argument("--output", type=Path, required=True)
    overlay = commands.add_parser("render-masks")
    _ = overlay.add_argument("reference", type=Path)
    _ = overlay.add_argument("regions", type=Path)
    _ = overlay.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(namespace=Arguments())
    regions = read_regions(args.regions)
    with Image.open(args.reference) as reference:
        if args.command == "render-masks":
            render_masks(reference, regions).save(args.output)
            return 0
        with Image.open(args.candidate) as candidate:
            result = compare_frames(reference, candidate, regions)
    _ = args.output.write_text(json.dumps(asdict(result), indent=2) + "\n")
    return 0 if result.passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
