# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy", "scipy"]
# ///
# How to run: imported by python3 -m art.batch_b1.process
"""B1 input identities and immutable, hash-bound review records."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Annotated, ClassVar, Final, Literal, Self

from pydantic import BaseModel, ConfigDict, Field, model_validator

from art.pipeline5.sheet_geometry import Normalization
from art.pipeline5.whole_checks import WholeResult

Age = Literal["adult", "child"]
Facing = Literal["S", "E"]
Pose = Literal["idle", "walk_a", "work_a", "carry", "sleep"]
FrameId = Annotated[str, Field(pattern=r"^[a-z][a-z0-9_]*$")]
FACINGS: Final[tuple[Facing, ...]] = ("S", "E")
POSES: Final[tuple[Pose, ...]] = ("idle", "walk_a", "work_a", "carry", "sleep")
REPO: Final = Path(__file__).resolve().parents[2]


class Evidence(BaseModel):
    """Reject unknown fields rather than silently discarding provenance."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")


class Reference(Evidence):
    path: Path
    sha256: str = Field(pattern=r"^[a-f0-9]{64}$")


class Generation(Evidence):
    look: str = Field(
        pattern=r"^(adult|child)_[MF]_[a-z][a-z0-9_]*_(short|long_tied|bun)$"
    )
    age: Age
    raw: Path
    prompt: str = Field(min_length=1)
    seed: None
    attempt: int = Field(ge=1, le=3)
    references: tuple[Reference, ...] = ()
    layout: Literal["wardrobe", "tools"] = "wardrobe"
    frame_ids: tuple[FrameId, ...] = POSES

    @model_validator(mode="after")
    def matching_age(self) -> Self:
        """Keep filesystem identity and normalization age in agreement."""
        if not self.look.startswith(self.age + "_"):
            message = "look age must match normalization age"
            raise ValueError(message)
        if (
            len(self.frame_ids) != 5
            or len(set(self.frame_ids)) != 5
            or self.frame_ids[0] != "idle"
        ):
            message = "frame_ids must name five distinct columns, starting with idle"
            raise ValueError(message)
        if self.layout == "wardrobe" and self.frame_ids != POSES:
            message = "wardrobe layout must retain the five TEMPLATE_V3 poses"
            raise ValueError(message)
        if self.layout == "tools" and "sleep" in self.frame_ids:
            message = "tool sheets contain standing poses only"
            raise ValueError(message)
        return self


class Frame(Evidence):
    look: str
    age: Age
    attempt: int
    raw: Path
    raw_sha256: str
    prompt_sha256: str
    facing: Facing
    pose: FrameId
    check_pose: Pose
    path: str
    sha256: str
    source_box: tuple[int, int, int, int]
    normalization: Normalization
    checks: WholeResult
    standing_scale: float
    sleep_scale: float
    anchor: tuple[int, int]
    footprint: tuple[int, int] = (1, 1)
    template_version: int = 3


class Summary(Evidence):
    look: str
    age: Age
    attempt: int
    record_sha256: str
    raw_sha256: str
    prompt_sha256: str
    passed: bool
    frames: tuple[Frame, ...] = ()
    error: str | None = None
    layout: Literal["wardrobe", "tools"] = "wardrobe"
    frame_ids: tuple[FrameId, ...] = POSES
    template_version: int = 3
    numeric_pass: bool = False
    numeric_passing_panels: int = 0
    numeric_model_status: Literal["MISSING"] = "MISSING"
    numeric_report_sha256: str | None = None


class VisualVerdict(Evidence):
    score: int = Field(ge=0, le=100)
    verdict: Literal["pass", "revise", "fail"]
    category_match: bool
    differences: tuple[str, ...]
    suggestions: tuple[str, ...]
    reasoning: str = Field(min_length=1)
    summary_sha256: str = Field(pattern=r"^[a-f0-9]{64}$")


def digest(path: Path) -> str:
    """Hash stored bytes, including PNG metadata, not an in-memory preview."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path: Path, value: BaseModel) -> None:
    """Write deterministic readable JSON for independent auditing."""
    _ = path.write_text(value.model_dump_json(indent=2) + "\n")
