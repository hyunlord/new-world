# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image"]
# ///
# How to run: see README.md for the dependency-complete uv command.
# python -m art.batch_a2.process RECORD.json
"""Export A2 candidates without changing historical manifests or adopting art."""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path
from typing import ClassVar, Final, Literal, Self

import numpy as np
from PIL import Image
from pydantic import BaseModel, ConfigDict, Field, TypeAdapter, model_validator

from art.compositor.projection_check import (
    ALPHA_THRESHOLD,
    HEIGHT_BANDS,
    ProjectionResult,
    ProjectionSpec,
    check_projection,
)
from art.pipeline2.judge import Verdict, judge, load_calibration
from art.pipeline_struct.normalize import normalize_sprite

ROOT: Final = Path(__file__).resolve().parents[2]
EVIDENCE: Final = Path("/Users/rexxa/work/art-batch/A2")
CALIBRATION: Final = ROOT / "docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json"
MIN_REFERENCES: Final = 2
CLI_ARGUMENT_COUNT: Final = 2
Bounds = tuple[int, int, int, int]


class Record(BaseModel):
    """A generation's declared geometry and provenance, before measurement."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    id: str = Field(pattern=r"^[a-z][a-z0-9_]*$")
    lane: str = Field(pattern=r"^[a-z][a-z0-9_]*$")
    era: Literal["paleolithic", "neolithic", "bronze", "iron"]
    attempt: int = Field(ge=1, le=3)
    raw: Path
    prompt: str = Field(min_length=1)
    subject: str | None = None
    references: tuple[Path, ...] = Field(min_length=2)
    target_height: int = Field(gt=0)
    projection_family: str
    height_band: tuple[int, int] | None = None
    height_band_reason: str | None = None
    footprint: tuple[float, float] = (1, 1)
    style_family: Literal["buildings", "plants", "ground", "items", "person"] = "buildings"
    generation_id: str | None = None
    seed: int | None = None
    generated_at: str | None = None

    @model_validator(mode="after")
    def geometry_is_declared(self) -> Self:
        """Reject silent band relaxation and unsafe or meaningless footprints."""
        if min(self.footprint) <= 0:
            message = "Footprint dimensions must be positive"
            raise ValueError(message)
        known = HEIGHT_BANDS.get(self.projection_family)
        if self.height_band is not None:
            low, high = self.height_band
            if not 0 < low <= high or not self.height_band_reason:
                message = "Height override needs a positive ordered band and justification"
                raise ValueError(message)
            if known is not None and (low < known[0] or high > known[1]):
                message = "An existing projection family band cannot be relaxed"
                raise ValueError(message)
        elif known is None:
            message = "An unlisted projection family needs a justified explicit height band"
            raise ValueError(message)
        if len(set(self.references)) < MIN_REFERENCES:
            message = "At least two distinct reference paths are required"
            raise ValueError(message)
        return self


class Reference(BaseModel):
    """Exact reference bytes used for generation."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    path: str
    sha256: str


class Result(BaseModel):
    """Measured candidate only; this tool can never emit adopted status."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    record: Record
    raw_preserved: str
    raw_sha256: str | None
    references: tuple[Reference, ...]
    calibration_sha256: str
    normalized: str | None
    normalized_sha256: str | None
    size: tuple[int, int] | None
    anchor: tuple[int, int] | None
    anchor_convention: str = "origin top-left; x right; y down; paste at world base minus anchor"
    operations: tuple[str, ...]
    projection: ProjectionResult | None
    judge_v2: Verdict | None
    raw_alpha_min: int | None
    raw_alpha_max: int | None
    clipped_at_raw_border: bool | None
    raw_nonzero_bbox: tuple[int, int, int, int] | None
    raw_opaque_bbox: tuple[int, int, int, int] | None
    raw_faint_far_ratio: float | None
    review_flags: tuple[str, ...]
    numeric_ready: bool
    status: Literal["NUMERIC_READY_AWAITING_REVIEW", "CANDIDATE_REJECTED"]
    failures: tuple[str, ...]
    review: None = None
    seed_note: str = "Null means the generation tool did not expose a seed; none was invented."


def digest(path: Path) -> str:
    """Hash file bytes without image re-encoding."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def preserve(source: Path, destination: Path) -> None:
    """Never overwrite a prior attempt with different raw image bytes."""
    content = source.read_bytes()
    if destination.exists() and destination.read_bytes() != content:
        message = f"Attempt raw identity conflict: {destination}"
        raise ValueError(message)
    destination.parent.mkdir(parents=True, exist_ok=True)
    if not destination.exists():
        _ = destination.write_bytes(content)


def raw_alpha(image: Image.Image) -> tuple[int, int, Bounds | None, Bounds, float, bool]:
    """Measure original fringe outside the opaque bounding box, without cleanup."""
    alpha = np.asarray(image.convert("RGBA"))[:, :, 3]
    channel = image.getchannel("A")
    alpha_bytes = channel.tobytes()
    ys, xs = np.nonzero(alpha >= ALPHA_THRESHOLD)
    bbox = (int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1)
    faint_far = (alpha > 0) & (alpha < ALPHA_THRESHOLD)
    left, top, right, bottom = bbox
    faint_far[max(0, top - 2) : bottom + 2, max(0, left - 2) : right + 2] = False
    ratio = float(np.count_nonzero(faint_far) / np.count_nonzero(alpha))
    clipped = bool(
        any(np.any(edge >= ALPHA_THRESHOLD) for edge in (alpha[0], alpha[-1], alpha[:, 0], alpha[:, -1]))
    )
    return min(alpha_bytes), max(alpha_bytes), channel.getbbox(), bbox, ratio, clipped


def process(record: Record, evidence: Path = EVIDENCE) -> Result:
    """Normalize once, measure unchanged export and retain every rejection reason."""
    directory = evidence / record.lane
    stem = f"{record.id}_try{record.attempt}"
    raw = directory / "raw" / f"{stem}.png"
    output = directory / "normalized" / f"{stem}.png"
    output.parent.mkdir(parents=True, exist_ok=True)
    source = record.raw if record.raw.is_absolute() else ROOT / record.raw
    reference_paths = tuple(p if p.is_absolute() else ROOT / p for p in record.references)
    references = tuple(Reference(path=str(p), sha256=digest(p)) for p in reference_paths)
    preserve(source, raw)
    projection = None
    verdict = None
    size = None
    anchor = None
    raw_min = None
    raw_max = None
    clipped = None
    nonzero_bbox = opaque_bbox = None
    faint_far_ratio = None
    review_flags: tuple[str, ...] = ()
    normalized_path = None
    normalized_hash = None
    operations: tuple[str, ...] = ()
    failures: list[str] = []
    try:
        with Image.open(raw) as image:
            normalized = normalize_sprite(image, record.target_height)
            raw_min, raw_max, nonzero_bbox, opaque_bbox, faint_far_ratio, clipped = raw_alpha(image)
            if faint_far_ratio > 0:
                review_flags = (
                    "Inspect raw halo: faint pixels extend beyond opaque bbox plus 2px; no cleanup applied",
                )
        normalized.image.save(output, optimize=True)
        normalized_path, normalized_hash = str(output), digest(output)
        size, anchor = normalized.image.size, normalized.anchor
        operations = (
            f"alpha>=128 bounds plus 2px fringe crop: {normalized.crop}",
            f"uniform Lanczos scale: {normalized.scale}",
            f"transparent 4px padding; source-canvas anchor: {anchor}",
        )
        projection = check_projection(
            normalized.image,
            ProjectionSpec(record.projection_family, anchor, record.height_band),
        )
        if clipped:
            failures.append("raw subject touches image border")
        if not projection.base_passed:
            failures.append("projection base does not match declared anchor")
        if not projection.height_passed:
            failures.append("projection height outside declared band")
        if not projection.light_passed:
            failures.append("projection upper-left luminance gradient failed")
        verdict = judge(raw, load_calibration(CALIBRATION), family=record.style_family)
        failures.extend(
            f"style metric: {key}" for key, passed in verdict.numeric_checks.items() if not passed
        )
    except (OSError, ValueError, KeyError) as failure:
        failures.append(f"{type(failure).__name__}: {failure}")
    ready = bool(projection and projection.passed and verdict and verdict.numeric_pass and not failures)
    result = Result(
        record=record,
        raw_preserved=str(raw),
        raw_sha256=digest(raw),
        references=references,
        calibration_sha256=digest(CALIBRATION),
        normalized=normalized_path,
        normalized_sha256=normalized_hash,
        size=size,
        anchor=anchor,
        operations=operations,
        projection=projection,
        judge_v2=verdict,
        raw_alpha_min=raw_min,
        raw_alpha_max=raw_max,
        clipped_at_raw_border=clipped,
        raw_nonzero_bbox=nonzero_bbox,
        raw_opaque_bbox=opaque_bbox,
        raw_faint_far_ratio=faint_far_ratio,
        review_flags=review_flags,
        numeric_ready=ready,
        status="NUMERIC_READY_AWAITING_REVIEW" if ready else "CANDIDATE_REJECTED",
        failures=tuple(failures),
    )
    _ = output.with_suffix(".json").write_text(result.model_dump_json(indent=2) + "\n")
    return result


def main() -> int:
    """Process a JSON record or array; rejected candidates produce exit status one."""
    if len(sys.argv) != CLI_ARGUMENT_COUNT:
        _ = sys.stderr.write("Usage: python -m art.batch_a2.process RECORD.json\n")
        return 2
    content = Path(sys.argv[1]).read_text()
    records = (
        TypeAdapter(tuple[Record, ...]).validate_json(content)
        if content.lstrip().startswith("[")
        else (Record.model_validate_json(content),)
    )
    results = tuple(process(record) for record in records)
    for result in results:
        prefix = f"{result.record.id} try{result.record.attempt}: {result.status}; "
        _ = sys.stdout.write(f"{prefix}{', '.join(result.failures)}\n")
    return 0 if results and all(result.numeric_ready for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
