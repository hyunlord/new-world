# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: imported by python3 -m art.pipeline_polish2.portrait_prepare.
"""Typed portrait evidence boundaries and immutable source paths."""
from __future__ import annotations

import hashlib
from pathlib import Path
from typing import ClassVar, Final, Literal, Self

from pydantic import BaseModel, ConfigDict, Field, model_validator

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = ROOT.parent / "artpolish2-data/portraits"
ANCHOR: Final = ROOT / "art/style05/anchors/portrait.png"
PortraitVariant = Literal["v1", "v2", "v3", "v4", "v5", "v6"]


class Evidence(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")


class AssetRef(Evidence):
    path: Path
    sha256: str = Field(pattern=r"^[a-f0-9]{64}$")

    @classmethod
    def of(cls, path: Path) -> Self:
        return cls(path=path.resolve(), sha256=digest(path))

    def verify(self) -> None:
        if digest(self.path) != self.sha256:
            message = f"Asset changed after request was frozen: {self.path}"
            raise ValueError(message)


class Candidate(Evidence):
    look: str
    variant: PortraitVariant
    portrait: AssetRef
    sprite: AssetRef
    attempt: int = Field(ge=1, le=2)
    status: Literal["OWNER_CHOICE_PENDING"] = "OWNER_CHOICE_PENDING"

    @property
    def key(self) -> str:
        return f"{self.look}/{self.variant}"


class CandidateSet(Evidence):
    schema_version: Literal[1] = 1
    candidates: tuple[Candidate, ...]

    @model_validator(mode="after")
    def unique(self) -> Self:
        if len({row.key for row in self.candidates}) != len(self.candidates):
            message = "Portrait candidates must have unique look/variant keys"
            raise ValueError(message)
        return self


class SourceAppearance(Evidence):
    look: str
    source: AssetRef
    appearance_brief: str = Field(min_length=1)


class Request(Evidence):
    schema_version: Literal[1] = 1
    look: str
    variant: PortraitVariant
    description: str
    prompt: str
    references: tuple[AssetRef, ...] = Field(min_length=2, max_length=3)
    face_plan: AssetRef
    attempt: Literal[1, 2] = 1
    appearance_manifest: AssetRef | None = None
    source_appearance: SourceAppearance | None = None
    previous_request: AssetRef | None = None
    correction: str | None = None
    canonical_path: Path
    raw_directory: Path
    attempt_limit: Literal[2] = 2
    transparent_background: Literal[False] = False


class Requests(Evidence):
    schema_version: Literal[1] = 1
    requests: tuple[Request, ...]
    logical_fallbacks: dict[str, str]
    expected_count: int
    available_count: int
    complete: bool
    skipped_started_count: int = 0


class ToolRequest(Evidence):
    id: str
    look: str
    variant: PortraitVariant
    attempt: Literal[1, 2] = 1
    prompt: str
    referenced_image_paths: tuple[str, ...] = Field(min_length=2, max_length=3)
    references: tuple[AssetRef, ...] = Field(min_length=2, max_length=3)
    request_path: Path | None = None
    canonical_path: Path
    raw_directory: Path
    transparent_background: Literal[False] = False


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_evidence(path: Path, evidence: BaseModel) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    _ = path.write_text(evidence.model_dump_json(indent=2) + "\n")
