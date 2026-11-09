#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image"]
# ///
# How to run (PYTHONPATH=.):
# python -m art.pipeline_batch_a3.process RECORD OUT [COUNTS]
# Reuse the documented existing art Python environment; COUNTS is true or false.
"""Export external A3 review candidates using the adopted ENV1 icon convention."""

from __future__ import annotations

import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import ClassVar, Final

from PIL import Image
from pydantic import AliasChoices, BaseModel, ConfigDict, Field

from art.pipeline2.judge import Verdict, judge, load_calibration
from art.pipeline_batch_a3.checks import (
    AlphaCheck,
    TileCheck,
    check_alpha,
)
from art.pipeline_batch_a3.icons import Bucket, icon_tiles, inspect_icon
from art.pipeline_env1.items_prepare import normalize
from art.pipeline_struct.normalize import CALIBRATION, digest

REPO: Final = Path(__file__).resolve().parents[2]
CLI_FIELDS: Final = ("record", "output", "counts")
REQUIRED_ARGS: Final = 2


class GenerationRecord(BaseModel):
    """Only parse export inputs; the immutable full generation record stays external."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    id: str = Field(pattern=r"^[a-z0-9_]+$")
    attempt: int = Field(ge=1)
    prompt: str = Field(min_length=1)
    raw: Path = Field(validation_alias=AliasChoices("raw", "source", "path"))
    era: str = ""


class ExportRequest(BaseModel):
    """One source record and its declared export policy."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    record: Path
    output: Path
    counts: bool = False
    asset_id: str | None = Field(default=None, pattern=r"^[a-z0-9_]+$")


@dataclass(frozen=True, slots=True)
class ExportedTile:
    """One immutable count-bucket export and its measured geometry."""

    bucket: Bucket
    path: str
    sha256: str
    check: TileCheck


@dataclass(frozen=True, slots=True)
class Candidate:
    """Evidence required for review, with adoption deliberately left pending."""

    id: str
    attempt: int
    era: str
    generation_record: str
    generation_record_sha256: str
    raw: str
    raw_sha256: str
    source_alpha: AlphaCheck
    height_target_px: int
    canvas_px: tuple[int, int]
    anchor_px: tuple[int, int]
    tile_footprint: tuple[int, int]
    operations: tuple[str, ...]
    exports: tuple[ExportedTile, ...]
    calibration: str
    calibration_sha256: str
    style_diagnostic: Verdict | None
    status: str
    seam_check: str = "NOT_APPLICABLE_ISOLATED_ITEM"
    visual_acceptance: str = "PENDING_INDEPENDENT_REVIEW"
    generation_id: str = ""
    projection_contract: str = "STRUCT_WORLD_ITEM"


def export_record(request: ExportRequest) -> Candidate:
    """Preserve raw bytes and report instrument failures without auto-adoption."""
    record_path = request.record.resolve()
    row = GenerationRecord.model_validate_json(record_path.read_text())
    asset_id = request.asset_id or row.id
    raw = row.raw if row.raw.is_absolute() else record_path.parent / row.raw
    raw = raw.resolve()
    output = request.output.resolve()
    if output.is_relative_to(REPO):
        message = "A3 candidates and check evidence must remain outside the repository"
        raise ValueError(message)
    output.mkdir(parents=True, exist_ok=True)
    exports: list[ExportedTile] = []
    style = None
    with Image.open(raw) as opened:
        alpha = check_alpha(opened)
        if alpha.passed:
            normalized = normalize(raw)
            for label, sprite in icon_tiles(normalized, counts=request.counts):
                path = output / f"{asset_id}_try{row.attempt}_count_{label}.png"
                sprite.save(path, optimize=True)
                with Image.open(path) as saved:
                    check = inspect_icon(saved, label, sprite)
                exports.append(ExportedTile(label, str(path), digest(path), check))
            style = judge(raw, load_calibration(CALIBRATION), family="items")
    status = (
        "CANDIDATE_REVIEW_REQUIRED"
        if exports and all(e.check.structural_pass for e in exports)
        else "REJECTED_STRUCTURE"
    )
    candidate = Candidate(
        asset_id,
        row.attempt,
        row.era,
        str(record_path),
        digest(record_path),
        str(raw),
        digest(raw),
        alpha,
        58,
        (64, 64),
        (32, 32),
        (1, 1),
        (
            "ENV1 normalize: nonzero-alpha crop, proportional Lanczos fit to58",
            "transparent 64x64 padding centered at item icon anchor(32,32)",
            "ENV1 ordinal layouts:1/3/9 whole-icon copies at64/42/29px",
        ),
        tuple(exports),
        str(CALIBRATION.relative_to(REPO)),
        digest(CALIBRATION),
        style,
        status,
        generation_id=row.id,
        projection_contract="ICON_ONLY:4-58px; measured bottom is not ground anchor",
    )
    check_path = output / f"{asset_id}_try{row.attempt}.json"
    _ = check_path.write_text(
        json.dumps(asdict(candidate), indent=2, allow_nan=False) + "\n"
    )
    return candidate


def main() -> int:
    """Process one record; structural rejection returns exit 1, never adoption."""
    args = sys.argv[1:]
    if not REQUIRED_ARGS <= len(args) <= len(CLI_FIELDS):
        print(
            "Usage: process RECORD.json OUT_DIR [COUNTS=false]",
            file=sys.stderr,
        )
        return 2
    values = dict(zip(CLI_FIELDS, args, strict=False))
    request = ExportRequest.model_validate(values)
    result = export_record(request)
    print(
        f"{result.id} try{result.attempt}: {result.status};",
        f"{len(result.exports)} exports",
    )
    return 0 if result.status == "CANDIDATE_REVIEW_REQUIRED" else 1


if __name__ == "__main__":
    raise SystemExit(main())
