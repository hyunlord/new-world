# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "typer", "numpy", "scipy", "scikit-image"]
# ///
# How to run: python3 -m art.pipeline_polish2.portrait_process REQUEST RAW --attempt 1
"""Preserve portrait originals and normalize full frames without pixel retouching."""
from __future__ import annotations

import shutil
from dataclasses import asdict
from pathlib import Path
from typing import Literal

import typer
from PIL import Image
from pydantic import Field, JsonValue

from art.pipeline2.judge import judge, load_calibration
from art.pipeline_env1.portrait_finalize import Inspection, geometry
from art.pipeline_polish2.portrait_appearance import (
    verified_appearance,
    verify_bust_detail,
)
from art.pipeline_polish2.portrait_evidence import preserve_input, record_revision
from art.pipeline_polish2.portrait_models import (
    DATA,
    ROOT,
    AssetRef,
    Candidate,
    CandidateSet,
    Evidence,
    Request,
    write_evidence,
)
from art.pipeline_polish2.portrait_previous import verify_previous_for_retry


class Processed(Evidence):
    candidate: Candidate
    request: AssetRef
    raw: AssetRef
    tool_source: AssetRef
    operations: tuple[str, ...]
    numeric_report_only: dict[str, JsonValue]
    structure_status: Literal["PENDING_OBSERVATION", "PASS", "FAIL"]
    structural: dict[str, JsonValue] | None
    inspection: AssetRef | None = None
    observation: Inspection | None = None
    owner_adopted: Literal[False] = False


class ProcessInput(Evidence):
    request_path: Path
    raw_path: Path
    attempt: int = Field(ge=1, le=2)
    inspection_path: Path | None = None


def process(spec: ProcessInput) -> Processed:
    """Save one attempt with literal provenance; observations remain independently supplied."""
    request = Request.model_validate_json(spec.request_path.read_bytes())
    if spec.attempt != request.attempt:
        message = "Attempt must match the separately prepared request"
        raise ValueError(message)
    for reference in (*request.references, request.face_plan):
        reference.verify()
    if request.previous_request is not None:
        request.previous_request.verify()
        if request.previous_portrait is not None:
            verify_previous_for_retry(request)
    if request.appearance_manifest is not None:
        _ = verified_appearance(request.appearance_manifest)
    if request.source_appearance is not None:
        request.source_appearance.source.verify()
    if request.bust_detail is not None:
        _ = verify_bust_detail(request.bust_detail, request.references[0], request.references[3])
    attempt_dir = request.raw_directory / f"try{spec.attempt}"
    raw_path = attempt_dir / "raw.png"
    with Image.open(spec.raw_path) as raw:
        if raw.width != raw.height:
            message = f"Portrait source must be square, received {raw.size}"
            raise ValueError(message)
        normalized = raw.convert("RGB").resize((128, 128), Image.Resampling.LANCZOS)
    attempt_dir.mkdir(parents=True, exist_ok=True)
    source = AssetRef.of(spec.raw_path)
    if raw_path.exists() and AssetRef.of(raw_path).sha256 != source.sha256:
        message = f"Attempt already contains a different raw image: {raw_path}"
        raise ValueError(message)
    if spec.raw_path.resolve() != raw_path.resolve():
        _ = shutil.copyfile(spec.raw_path, raw_path)
    request.canonical_path.parent.mkdir(parents=True, exist_ok=True)
    normalized.save(request.canonical_path, optimize=True)
    normalized.save(attempt_dir / "canonical.png", optimize=True)
    numeric = judge(raw_path, load_calibration(ROOT / "docs/v8/artenv1/portrait_calibration.json"), family="portrait")
    structural = None
    observation = None
    inspection = None
    structure_status: Literal["PENDING_OBSERVATION", "PASS", "FAIL"] = "PENDING_OBSERVATION"
    if spec.inspection_path is not None:
        inspection = preserve_input(spec.inspection_path, attempt_dir / "observations")
        observation = Inspection.model_validate_json(inspection.path.read_bytes())
        if observation.look != f"{request.look}/{request.variant}" or observation.attempt != spec.attempt:
            message = "Inspection must name the exact look/variant and attempt"
            raise ValueError(message)
        structural = geometry(observation)
        structure_status = "PASS" if structural["passed"] else "FAIL"
    record = Processed(
        candidate=Candidate(look=request.look, variant=request.variant, attempt=spec.attempt,
                            portrait=AssetRef.of(request.canonical_path), sprite=request.references[0]),
        request=AssetRef.of(spec.request_path), raw=AssetRef.of(raw_path), tool_source=source,
        operations=("full-frame RGB conversion", "full-frame Lanczos resize to 128x128", "lossless PNG encoding"),
        numeric_report_only=asdict(numeric), structure_status=structure_status, structural=structural,
        inspection=inspection, observation=observation,
    )
    revision = record_revision(attempt_dir / "processed.json", record)
    write_evidence(request.raw_directory / "selected_revision.json", revision)
    write_evidence(request.raw_directory / "selected.json", record)
    return record


def collect(data: Path = DATA) -> CandidateSet:
    """Collect explicitly selected, hashed candidates without claiming acceptance."""
    records = tuple(Processed.model_validate_json(path.read_bytes())
                    for path in sorted((data / "generation").glob("*/*/selected.json")))
    for row in records:
        row.candidate.portrait.verify()
        row.candidate.sprite.verify()
        row.raw.verify()
        if row.inspection is not None:
            row.inspection.verify()
    result = CandidateSet(candidates=tuple(row.candidate for row in records))
    write_evidence(data / "candidates.json", result)
    return result


def main(request: Path, raw: Path, attempt: int = 1, inspection: Path | None = None) -> None:
    result = process(ProcessInput(request_path=request, raw_path=raw, attempt=attempt, inspection_path=inspection))
    print(f"{result.candidate.key}: normalized; structure={result.structure_status}; owner choice pending")


if __name__ == "__main__":
    typer.run(main)
