#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: uv run art/techmap/assets.py FAMILY OUTPUT_DIR CHECK_DIR INPUT...
# FAMILY is icon/banner/chapter. INPUT is a PNG or generation JSON record.
"""Mechanical A4 image normalization and evidence; never adoption decisions."""

from __future__ import annotations

import hashlib
import sys
from dataclasses import dataclass, replace
from enum import StrEnum
from pathlib import Path
from typing import ClassVar, Final, assert_never

from PIL import Image
from pydantic import BaseModel, ConfigDict, Field, TypeAdapter

ALPHA_VISIBLE: Final = 16
ALPHA_SOLID: Final = 128


class Family(StrEnum):
    """Declared A4 UI canvases, distinct from world sprite footprints."""

    ICON = "icon"
    BANNER = "banner"
    CHAPTER = "chapter"

    @property
    def canvas(self) -> tuple[int, int]:
        match self:
            case Family.ICON:
                return (64, 64)
            case Family.BANNER:
                return (264, 88)
            case Family.CHAPTER:
                return (520, 260)
            case unreachable:
                assert_never(unreachable)

    @property
    def padding(self) -> int:
        match self:
            case Family.ICON:
                return 3
            case Family.BANNER | Family.CHAPTER:
                return 2
            case unreachable:
                assert_never(unreachable)


@dataclass(frozen=True, slots=True)
class Expectation:
    canvas: tuple[int, int] | None = None
    sha256: str | None = None


DEFAULT_EXPECTATION: Final = Expectation()


@dataclass(frozen=True, slots=True)
class Measurement:
    path: Path
    sha256: str
    canvas: tuple[int, int]
    has_alpha: bool
    alpha_extrema: tuple[int, int]
    bbox1: tuple[int, int, int, int] | None
    bbox16: tuple[int, int, int, int] | None
    bbox128: tuple[int, int, int, int] | None
    opaque_pixels: int
    occupancy: float
    border_alpha_max: int
    expected_canvas: tuple[int, int] | None
    canvas_matches: bool
    expected_sha256: str | None
    hash_matches: bool | None
    passed: bool


class AssetRejected(ValueError):
    """A source or export failed a concrete file-level check."""

    measurement: Measurement

    def __init__(self, measurement: Measurement) -> None:
        self.measurement = measurement
        super().__init__(f"Image check failed: {measurement.path}")


@dataclass(frozen=True, slots=True)
class Normalization:
    raw: Measurement
    output: Measurement
    crop: tuple[int, int, int, int]
    scale: float
    resized: tuple[int, int]
    offset: tuple[int, int]
    anchor: tuple[int, int]
    discarded_alpha_pixels: int
    discarded_alpha_max: int
    footprint: None = None
    operations: tuple[str, ...] = (
        "preserve generated alpha; no repainting",
        "alpha>=128 bounds plus 2px fringe; discarded fringe measured",
        "uniform LANCZOS fit; integer raster rounding",
        "center on transparent canvas",
    )
    status: str = "numeric_checked_visual_review_pending"


class Generation(BaseModel):
    """Parse the fields needed for export; the original prompt record is retained."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    id: str = Field(pattern=r"^[a-zA-Z0-9_-]+$")
    family: Family
    attempt: int = Field(default=1, ge=1)
    raw_path: Path
    raw_sha256: str | None = Field(default=None, pattern=r"^[0-9a-f]{64}$")


def measure(path: Path, expected: Expectation = DEFAULT_EXPECTATION) -> Measurement:
    """Check original file bytes and alpha, never an inferred opaque conversion."""
    sha = hashlib.sha256(path.read_bytes()).hexdigest()
    with Image.open(path) as original:
        has_alpha = "A" in original.getbands()
        rgba = original.convert("RGBA")
    alpha = rgba.getchannel("A")
    histogram = alpha.histogram()
    minimum = next(i for i, count in enumerate(histogram) if count)
    maximum = max(i for i, count in enumerate(histogram) if count)
    boxes = tuple(
        alpha.point([255 if i >= threshold else 0 for i in range(256)]).getbbox()
        for threshold in (1, ALPHA_VISIBLE, ALPHA_SOLID)
    )
    edges = (
        (0, 0, rgba.width, 1),
        (0, rgba.height - 1, rgba.width, rgba.height),
        (0, 0, 1, rgba.height),
        (rgba.width - 1, 0, rgba.width, rgba.height),
    )
    border = max(
        max(i for i, count in enumerate(alpha.crop(edge).histogram()) if count)
        for edge in edges
    )
    count = sum(histogram[ALPHA_SOLID:])
    canvas_matches = expected.canvas is None or rgba.size == expected.canvas
    hash_matches = None if expected.sha256 is None else sha == expected.sha256
    passed = (
        has_alpha
        and minimum == 0
        and maximum >= ALPHA_SOLID
        and border < ALPHA_VISIBLE
        and canvas_matches
        and hash_matches is not False
    )
    return Measurement(
        path,
        sha,
        rgba.size,
        has_alpha,
        (minimum, maximum),
        boxes[0],
        boxes[1],
        boxes[2],
        count,
        count / (rgba.width * rgba.height),
        border,
        expected.canvas,
        canvas_matches,
        expected.sha256,
        hash_matches,
        passed,
    )


def normalize(raw: Path, output: Path, family: Family) -> Normalization:
    """Reject clipping first; use the established solid-alpha crop plus soft fringe."""
    source_check = measure(raw)
    bounds = source_check.bbox128
    if not source_check.passed or bounds is None:
        raise AssetRejected(source_check)
    width, height = source_check.canvas
    crop = (
        max(0, bounds[0] - 2),
        max(0, bounds[1] - 2),
        min(width, bounds[2] + 2),
        min(height, bounds[3] + 2),
    )
    canvas_width, canvas_height = family.canvas
    scale = min(
        (canvas_width - 2 * family.padding) / (crop[2] - crop[0]),
        (canvas_height - 2 * family.padding) / (crop[3] - crop[1]),
    )
    resized = (
        max(1, round((crop[2] - crop[0]) * scale)),
        max(1, round((crop[3] - crop[1]) * scale)),
    )
    offset = ((canvas_width - resized[0]) // 2, (canvas_height - resized[1]) // 2)
    with Image.open(raw) as source:
        rgba = source.convert("RGBA")
        cropped = rgba.crop(crop)
        whole_histogram = rgba.getchannel("A").histogram()
        kept_histogram = cropped.getchannel("A").histogram()
        discarded = tuple(
            total - kept
            for total, kept in zip(whole_histogram, kept_histogram, strict=True)
        )
        discarded_count = sum(discarded[1:])
        discarded_max = max(
            (i for i, count in enumerate(discarded) if i and count), default=0
        )
        cutout = cropped.resize(resized, Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", family.canvas)
    canvas.paste(cutout, offset)
    output.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(output, optimize=True)
    output_check = measure(output, Expectation(canvas=family.canvas))
    if not output_check.passed:
        raise AssetRejected(output_check)
    return Normalization(
        source_check,
        output_check,
        crop,
        scale,
        resized,
        offset,
        (canvas_width // 2, canvas_height // 2),
        discarded_count,
        discarded_max,
    )


def main(arguments: list[str]) -> int:
    """Normalize a snapshot of PNGs or generation records; save every verdict."""
    if len(arguments) < 4:
        _ = sys.stderr.write(
            "Usage: assets.py FAMILY OUTPUT_DIR CHECK_DIR PNG_OR_JSON...\n"
        )
        return 2
    family, output_dir, check_dir = (
        Family(arguments[0]),
        Path(arguments[1]),
        Path(arguments[2]),
    )
    check_dir.mkdir(parents=True, exist_ok=True)
    failed = 0
    for source in (Path(value) for value in arguments[3:]):
        record = (
            Generation.model_validate_json(source.read_bytes())
            if source.suffix == ".json"
            else Generation(id=source.stem, family=family, raw_path=source)
        )
        if record.family != family:
            _ = sys.stderr.write(f"Family mismatch: {source}\n")
            return 2
        name = (
            f"{record.id}-{record.attempt:02d}"
            if source.suffix == ".json"
            else record.id
        )
        raw_check = measure(record.raw_path, Expectation(sha256=record.raw_sha256))
        try:
            if not raw_check.passed:
                raise AssetRejected(raw_check)
            result = normalize(record.raw_path, output_dir / f"{name}.png", family)
            result = replace(result, raw=raw_check)
            payload = TypeAdapter(Normalization).dump_json(result, indent=2)
        except AssetRejected as rejection:
            failed += 1
            payload = TypeAdapter(Measurement).dump_json(
                rejection.measurement, indent=2
            )
        _ = (check_dir / f"{name}.json").write_bytes(payload + b"\n")
    _ = sys.stdout.write(
        f"Processed {len(arguments) - 3}; failed {failed}; visual review still required.\n"
    )
    return int(failed > 0)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
