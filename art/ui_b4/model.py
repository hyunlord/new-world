# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python3 -m art.ui_b4.check ROOT REPORT.json
"""Typed boundaries for B4 candidate provenance and audit results."""

from pathlib import Path
from typing import Annotated, ClassVar, Literal

from pydantic import BaseModel, ConfigDict, Field, JsonValue

Digest = Annotated[str, Field(pattern=r"^[0-9a-f]{64}$")]
Identifier = Annotated[str, Field(pattern=r"^[a-z][a-z0-9_]*$")]
Lane = Literal["jobs", "cards", "alerts", "needs", "governance"]
FamilyName = Literal["icon", "chapter"]


class Evidence(BaseModel):
    """Parse audited fields while allowing extra production commentary."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")


class Coverage(Evidence):
    required_ids: tuple[Identifier, ...]
    variant_ids: tuple[Identifier, ...]


class Catalogue(Evidence):
    runtime_source_commit: Annotated[str, Field(pattern=r"^[0-9a-f]{40}$")]
    lanes: dict[Lane, Coverage]


class Reference(Evidence):
    path: Path
    sha256: Digest


class Generation(Evidence):
    id: Identifier
    family: FamilyName
    attempt: Annotated[int, Field(ge=1, le=3)]
    prompt: Annotated[str, Field(min_length=1)]
    raw_path: Path
    original_tool_path: Path
    raw_sha256: Digest
    references: Annotated[tuple[Reference, ...], Field(min_length=1)]
    tool: Annotated[str, Field(min_length=1)]
    seed: int | None


class Asset(Evidence):
    id: Identifier
    lane: Lane
    family: FamilyName
    era: Annotated[str, Field(min_length=1)]
    canvas: tuple[int, int]
    anchor: tuple[int, int]
    footprint: None
    path: Path
    sha256: Digest
    attempt_record: Path
    attempt_record_sha256: Digest
    normalization: Path
    status: Literal["candidate_selected"]
    semantic_note: Annotated[str, Field(min_length=1)]
    source_runtime_ids: JsonValue


class Decision(Evidence):
    record: Path
    reason: Annotated[str, Field(min_length=1)]


class Comparison(Evidence):
    id: Identifier
    before_record: Path
    after_record: Path
    decision: Annotated[str, Field(min_length=1)]
    reason: Annotated[str, Field(min_length=1)]


class SelfReview(Evidence):
    weakest_ids: tuple[Identifier, ...]
    regenerated_ids: tuple[Identifier, ...]
    before_after: tuple[Comparison, ...]


class Manifest(Evidence):
    schema_version: Literal[1]
    lane: Lane
    assets: Annotated[tuple[Asset, ...], Field(min_length=1)]
    attempts: Annotated[tuple[Path, ...], Field(min_length=1)]
    rejected: tuple[Decision, ...]
    superseded: tuple[Decision, ...]
    coverage: Coverage
    self_review: SelfReview


class LaneResult(Evidence):
    lane: Lane
    assets: int
    attempts: int
    rejected: int
    superseded: int
    regenerated: int
    errors: tuple[str, ...]


class AuditResult(Evidence):
    root: Path
    runtime_source_commit: str | None
    passed: bool
    lanes: tuple[LaneResult, ...]
    errors: tuple[str, ...]
