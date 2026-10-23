# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by portrait_prepare and portrait_process for optional edit input verification.
"""Authenticate an edit reference through the immutable first-attempt processing record."""
from __future__ import annotations

from typing import ClassVar

from pydantic import BaseModel, ConfigDict

from art.pipeline_polish2.portrait_models import (
    AssetRef,
    Candidate,
    PreviousPortrait,
    Request,
)


class FirstAttemptRecord(BaseModel):
    """Project only provenance fields; the entire original record remains hash-bound."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    candidate: Candidate
    request: AssetRef
    raw: AssetRef
    tool_source: AssetRef


def verify_previous_portrait(binding: PreviousPortrait, original: AssetRef, image: AssetRef) -> None:
    """Require exact first request/record/raw provenance without reading mutable selection."""
    original.verify()
    first = Request.model_validate_json(original.path.read_bytes())
    if first.attempt != 1 or binding.processed.path.resolve() != (first.raw_directory / "try1/processed.json").resolve():
        message = "Previous portrait requires the immutable first-attempt processing record"
        raise ValueError(message)
    binding.processed.verify()
    record = FirstAttemptRecord.model_validate_json(binding.processed.path.read_bytes())
    if (binding.request != record.request or binding.raw != record.raw or image != record.raw
            or record.request.sha256 != original.sha256
            or record.raw.path.resolve() != (first.raw_directory / "try1/raw.png").resolve()
            or record.raw.sha256 != record.tool_source.sha256
            or record.candidate.attempt != 1 or record.candidate.key != f"{first.look}/{first.variant}"
            or record.candidate.sprite != first.references[0]):
        message = "Previous portrait does not match its original request and preserved first raw"
        raise ValueError(message)
    for reference in (record.request, record.raw, record.tool_source, first.face_plan, first.references[0]):
        reference.verify()


def verify_previous_for_retry(retry: Request) -> None:
    """Bind the immutable first history to the identity and frozen input consuming it."""
    binding, original = retry.previous_portrait, retry.previous_request
    if binding is None or original is None or retry.attempt != 2 or len(retry.references) != 5:
        message = "Previous portrait requires a bound second-attempt request"
        raise ValueError(message)
    verify_previous_portrait(binding, original, retry.references[4])
    first = Request.model_validate_json(original.path.read_bytes())
    if (retry.look != first.look or retry.variant != first.variant
            or retry.raw_directory.resolve() != first.raw_directory.resolve()
            or retry.canonical_path.resolve() != first.canonical_path.resolve()
            or retry.face_plan.sha256 != first.face_plan.sha256
            or retry.references[0].sha256 != first.references[0].sha256):
        message = "Previous portrait does not match the consuming retry identity, paths, plan and native input"
        raise ValueError(message)
    retry.face_plan.verify()
    retry.references[0].verify()


def bind_previous_portrait(original: AssetRef) -> PreviousPortrait:
    """Freeze the first record/raw bindings only after verifying their complete provenance chain."""
    original.verify()
    first = Request.model_validate_json(original.path.read_bytes())
    processed = AssetRef.of(first.raw_directory / "try1/processed.json")
    record = FirstAttemptRecord.model_validate_json(processed.path.read_bytes())
    binding = PreviousPortrait(processed=processed, request=record.request, raw=record.raw)
    verify_previous_portrait(binding, original, record.raw)
    return binding
