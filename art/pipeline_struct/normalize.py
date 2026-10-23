# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image"]
# ///
# How to run: uv run --with pillow --with numpy --with pydantic --with scikit-image python -m art.pipeline_struct.normalize [LEDGER]
"""Deterministic candidate exports; numeric readiness never means art acceptance."""
from __future__ import annotations

import hashlib
import json
import shutil
import sys
from dataclasses import dataclass, replace
from pathlib import Path
from typing import ClassVar, Final, Literal, assert_never

import numpy as np
from numpy.typing import NDArray
from PIL import Image
from pydantic import BaseModel, ConfigDict, Field, TypeAdapter

from art.compositor.models import AssetEntry, AssetManifest
from art.compositor.projection_check import (
    ProjectionResult,
    ProjectionSpec,
    check_projection,
)
from art.compositor.wall_metrics import StripMeasurement, measure_strip
from art.pipeline2.judge import Verdict, judge, load_calibration

REPO: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path("/Users/rexxa/work/artstruct-data/generations")
CALIBRATION: Final = REPO / "docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json"


class Generation(BaseModel):
    """Parse only relevant immutable fields; retain the complete raw ledger separately."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    asset: str = Field(pattern=r"^[a-zA-Z0-9_-]+$")
    attempt: int = Field(ge=1, le=3)
    kind: Literal["strip", "sprite", "component"]
    source: Path
    size: tuple[int, int] | None = None
    height: int | None = Field(default=None, gt=0)
    family: str = "buildings"
    projection_family: str | None = None
    footprint: tuple[float, float] = (1, 1)


class AssetReading(BaseModel):
    """Actual executor reading, separate from numeric measurements and owner approval."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    asset: str
    attempt: int = Field(ge=1, le=3)
    readable: bool
    reading: str = Field(min_length=1)


@dataclass(frozen=True, slots=True)
class NormalizedSprite:
    image: Image.Image
    anchor: tuple[int, int]
    crop: tuple[int, int, int, int]
    scale: float


@dataclass(frozen=True, slots=True)
class NormalizedComponent:
    image: Image.Image
    crop: tuple[int, int, int, int]


@dataclass(frozen=True, slots=True)
class AttemptCheck:
    asset: str
    attempt: int
    kind: str
    source: str
    source_sha256: str
    export: str
    export_sha256: str
    operations: tuple[str, ...]
    projection: ProjectionResult | None
    seam_x: StripMeasurement | None
    seam_y: StripMeasurement | None
    judge_v2: Verdict | None
    numeric_ready: bool
    error: str | None
    entry: AssetEntry | None


@dataclass(frozen=True, slots=True)
class ExportEvidence:
    calibration_sha256: str
    ledger_sha256: str
    acceptance: str
    attempts: tuple[AttemptCheck, ...]
    readings_sha256: str | None = None
    selections: tuple[AttemptCheck, ...] = ()


def normalize_sprite(source: Image.Image, target_height: int) -> NormalizedSprite:
    """Crop alpha bounds plus fringe, uniformly resample, and pad without painting."""
    if "A" not in source.getbands():
        message = "Sprite lacks generated alpha; background removal is not permitted"
        raise ValueError(message)
    rgba = source.convert("RGBA")
    pixels: NDArray[np.uint8] = np.asarray(rgba)
    alpha = pixels[:, :, 3]
    if int(alpha.min()) != 0 or int(alpha.max()) < 128:
        message = "Sprite requires transparent background and an opaque subject"
        raise ValueError(message)
    ys, xs = np.nonzero(alpha >= 128)
    crop = (max(0, int(xs.min()) - 2), max(0, int(ys.min()) - 2),
            min(source.width, int(xs.max()) + 3), min(source.height, int(ys.max()) + 3))
    scale = target_height / int(ys.max() - ys.min() + 1)
    cut = rgba.crop(crop)
    scaled = cut.resize((max(1, round(cut.width * scale)), max(1, round(cut.height * scale))), Image.Resampling.LANCZOS)
    visible = np.asarray(scaled)[:, :, 3] >= 128
    rows = np.nonzero(visible)[0]
    if not rows.size:
        message = "Resampling eliminated opaque subject"
        raise ValueError(message)
    pad = 4
    canvas = Image.new("RGBA", (scaled.width + pad * 2, scaled.height + pad * 2))
    canvas.paste(scaled, (pad, pad))
    return NormalizedSprite(canvas, (canvas.width // 2, int(rows.max()) + pad), crop, scale)


def digest(path: Path) -> str:
    """Hash literal bytes, including failed raw generations."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def normalize_component(source: Image.Image, size: tuple[int, int]) -> NormalizedComponent:
    """Crop an alpha component and globally resample to its declared block geometry."""
    if "A" not in source.getbands() or min(size) <= 0:
        message = "Component requires generated alpha and positive dimensions"
        raise ValueError(message)
    rgba = source.convert("RGBA")
    pixels: NDArray[np.uint8] = np.asarray(rgba)
    alpha = pixels[:, :, 3]
    if int(alpha.min()) != 0 or int(alpha.max()) < 128:
        message = "Component requires a transparent background and opaque subject"
        raise ValueError(message)
    ys, xs = np.nonzero(alpha >= 128)
    crop = (int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1)
    return NormalizedComponent(rgba.crop(crop).resize(size, Image.Resampling.LANCZOS), crop)


def export_attempt(row: Generation, directory: Path) -> AttemptCheck:
    """Measure an immutable attempt export before selecting a catalogue candidate."""
    raw = DATA / f"{row.asset}_try{row.attempt}.png"
    if not raw.exists():
        _ = shutil.copyfile(row.source, raw)
    if digest(raw) != digest(row.source):
        message = f"Raw attempt identity mismatch: {raw}"
        raise ValueError(message)
    output = directory / f"{row.asset}_try{row.attempt}.png"
    projection = None
    seam_x = None
    seam_y = None
    verdict = None
    entry = None
    operations: tuple[str, ...] = ()
    error = None
    numeric_ready = False
    try:
        with Image.open(raw) as source:
            match row.kind:
                case "strip":
                    if row.size is None or min(row.size) <= 0:
                        message = f"Strip requires positive dimensions: {row.asset}"
                        raise ValueError(message)
                    image = source.convert("RGBA").resize(row.size, Image.Resampling.LANCZOS)
                    anchor = (image.width // 2, image.height - 1)
                    operations = (f"fullframe LANCZOS {source.size} -> {row.size}",)
                    seam_x = measure_strip(image) if image.width == 256 else None
                    seam_y = measure_strip(image.transpose(Image.Transpose.TRANSPOSE)) if image.height == 256 else None
                    opaque = bool(np.all(np.asarray(image)[:, :, 3] >= 128))
                    numeric_ready = opaque and all(measurement.passed for measurement in (seam_x, seam_y) if measurement is not None)
                case "sprite":
                    if row.height is None:
                        message = f"Sprite requires target height: {row.asset}"
                        raise ValueError(message)
                    normalized = normalize_sprite(source, row.height)
                    image, anchor = normalized.image, normalized.anchor
                    operations = (f"alpha>=128 bounds+2 fringe crop {normalized.crop}",
                                  f"uniform LANCZOS scale {normalized.scale}", f"transparent pad4; declared anchor {anchor}")
                    projection = check_projection(image, ProjectionSpec(row.projection_family or row.asset, anchor))
                case "component":
                    if row.size is None:
                        message = f"Component requires declared dimensions: {row.asset}"
                        raise ValueError(message)
                    component = normalize_component(source, row.size)
                    image = component.image
                    anchor = (image.width // 2, image.height - 1)
                    operations = (f"alpha>=128 bbox crop {component.crop}",
                                  f"global LANCZOS resize to exact structural dimensions {row.size}",
                                  "structural component: no standing projection acceptance")
                    numeric_ready = True
                case unreachable:
                    assert_never(unreachable)
        image.save(output, optimize=True)
        entry = AssetEntry(id=row.asset, path=f"art/style05/struct/{row.asset}.png", width=image.width,
                           height=image.height, footprint=row.footprint, family=row.family, anchor=anchor,
                           status="CANDIDATE_FAILED")
        if row.kind == "sprite":
            operations += (f"judge_v2 metric_input={raw}; sha256={digest(raw)}; projection_input={output}",)
            verdict = judge(raw, load_calibration(CALIBRATION), family=row.family)
            numeric_ready = bool(projection and projection.passed and verdict.numeric_pass)
        status = "NUMERIC_READY_AWAITING_READING" if numeric_ready else "CANDIDATE_FAILED"
        entry = entry.model_copy(update={"status": status})
    except (ValueError, KeyError) as failure:
        error = str(failure)
    return AttemptCheck(row.asset, row.attempt, row.kind, str(raw), digest(raw), str(output),
                        digest(output) if output.exists() else "", operations, projection, seam_x,
                        seam_y, verdict, numeric_ready, error, entry)


def select_attempt(candidates: tuple[AttemptCheck, ...], readings: tuple[AssetReading, ...]) -> AttemptCheck | None:
    """Visual rejections override numeric preference without changing measured evidence."""
    available = sorted((row for row in candidates if row.entry is not None), key=lambda row: row.attempt)
    notes = {(row.asset, row.attempt): row for row in readings}
    readable = [row for row in available if (note := notes.get((row.asset, row.attempt))) and note.readable]
    ready = [row for row in readable if row.numeric_ready]
    unreviewed = [row for row in available if (row.asset, row.attempt) not in notes]
    pool = ready or readable or unreviewed or available
    if not pool:
        return None
    chosen = pool[-1]
    note = notes.get((chosen.asset, chosen.attempt))
    status = "UNREVIEWED_NUMERIC_PASS" if chosen.numeric_ready else "UNREVIEWED_NUMERIC_FAIL"
    if note is not None:
        status = "VISUAL_FAIL" if not note.readable else (
            "NUMERIC_READY_READABLE_OWNER_PENDING" if chosen.numeric_ready else "READABLE_NUMERIC_FAIL")
        status += f": {note.reading}"
    if chosen.entry is not None:
        return replace(chosen, entry=chosen.entry.model_copy(update={"status": status}))
    return None


def run(ledger: Path, readings_path: Path | None = None) -> tuple[AttemptCheck, ...]:
    """Process a snapshot of completed attempts, preserving all measured candidates."""
    rows = TypeAdapter(list[Generation]).validate_json(ledger.read_bytes())
    identities = [(row.asset, row.attempt) for row in rows]
    if len(identities) != len(set(identities)):
        message = "Ledger repeats an asset/attempt identity"
        raise ValueError(message)
    output = REPO / "art/style05/struct"
    output.mkdir(parents=True, exist_ok=True)
    attempts_dir = DATA / "exports"
    attempts_dir.mkdir(parents=True, exist_ok=True)
    checks = tuple(export_attempt(row, attempts_dir) for row in sorted(rows, key=lambda item: (item.asset, item.attempt)))
    docs = REPO / "docs/v8/artstruct"
    readings_path = readings_path or docs / "asset_readings.json"
    readings = TypeAdapter(tuple[AssetReading, ...]).validate_json(readings_path.read_bytes()) if readings_path.exists() else ()
    if len({(row.asset, row.attempt) for row in readings}) != len(readings):
        message = "Readings repeat an asset/attempt identity"
        raise ValueError(message)
    entries: list[AssetEntry] = []
    selections: list[AttemptCheck] = []
    for asset in sorted({row.asset for row in rows}):
        selected = select_attempt(tuple(check for check in checks if check.asset == asset), readings)
        if selected is not None and selected.entry is not None:
            _ = shutil.copyfile(selected.export, REPO / selected.entry.path)
            entries.append(selected.entry)
            selections.append(selected)
    _ = (docs / "assets.json").write_text(AssetManifest(assets=tuple(entries)).model_dump_json(indent=2) + "\n")
    evidence = ExportEvidence(digest(CALIBRATION), digest(ledger),
                              "Numeric and executor reading evidence only; owner approval remains pending", checks,
                              digest(readings_path) if readings_path.exists() else None, tuple(selections))
    _ = (docs / "asset_checks.json").write_bytes(TypeAdapter(ExportEvidence).dump_json(evidence, indent=2) + b"\n")
    return checks


if __name__ == "__main__":
    results = run(Path(sys.argv[1]) if len(sys.argv) > 1 else DATA / "ledger.json")
    print(json.dumps({"attempts": len(results), "numeric_ready": sum(row.numeric_ready for row in results)}))
