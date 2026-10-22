# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: imported by python -m art.pipeline5.prod1_process
"""Typed request and immutable evidence records for paired-facing production."""
from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final, TypedDict

JsonValue = None | bool | int | float | str | list["JsonValue"] | dict[str, "JsonValue"]

from art.pipeline2.judge import Verdict
from art.pipeline5.sheet_geometry import Normalization
from art.pipeline5.whole_checks import WholeResult

FACINGS: Final = ('S', 'E')


@dataclass(frozen=True, slots=True)
class ProductionRequest:
    raw: Path
    look: str
    age: str
    attempt: int
    prompt_path: Path
    output_root: Path
    diagnostics_root: Path
    review_reason: str | None = None

    def __post_init__(self) -> None:
        """Constrain every filesystem identity before opening inputs."""
        if (self.age not in ('adult', 'child') or self.attempt not in (1, 2, 3)
                or re.fullmatch(r'(adult|child)_[MF]_(loincloth|hide_wrap|fur_cloak)_(short|long_tied|bun)', self.look) is None
                or not self.look.startswith(self.age + '_')):
            message = 'request: expected canonical look, matching age and attempt 1..3'
            raise ValueError(message)
        if self.review_reason is not None and not self.review_reason.strip():
            message = 'review retry requires a nonempty targeted misread reason'
            raise ValueError(message)
        if self.output_root.resolve() == self.diagnostics_root.resolve():
            message = 'request: delivery and evidence roots must differ'
            raise ValueError(message)

    @property
    def diagnostic(self) -> Path:
        return self.diagnostics_root / self.look / f'try{self.attempt}'


def parse_dispatch(path: Path, roots: tuple[Path, Path], review_retry: bool = False) -> ProductionRequest:
    """Parse the small JSON boundary without adding runtime dependencies."""
    data: JsonValue = json.loads(path.read_text())
    match data:  # noqa: MATCH_OK - untrusted JSON boundary requires rejection case
        case {'raw': str(raw), 'look': str(look), 'age': str(age),
              'attempt': int(attempt), 'prompt_path': str(prompt)} if type(attempt) is int:
            reason: str | None = None
            if review_retry:
                match data:  # noqa: MATCH_OK - untrusted JSON boundary requires rejection case
                    case {'review_reason': str(value)} if value.strip():
                        reason = value
                    case _:
                        message = 'review retry dispatch requires review_reason'
                        raise ValueError(message)
            return ProductionRequest(Path(raw), look, age, attempt, Path(prompt), *roots, reason)
        case _:
            message = 'dispatch: require raw/look/age/attempt/prompt_path with correct types'
            raise ValueError(message)


@dataclass(frozen=True, slots=True)
class FrameRecord:
    facing: str
    frame: str
    passed: bool
    checks: WholeResult
    judge: Verdict | None
    error: str | None = None


@dataclass(frozen=True, slots=True)
class Sidecar:
    look: str
    age: str
    facing: str
    frame: str
    attempt: int
    raw_sha256: str
    prompt_sha256: str
    image_sha256: str
    raw: str
    diagnostics: str
    panel_index: int
    calibration_sha256: str
    scale: float
    rotation: int
    source_box: tuple[int, int, int, int]
    normalization: Normalization
    checks: WholeResult
    judge: Verdict
    standing_scale: float
    sleep_scale: float
    template_version: int = 3


@dataclass(frozen=True, slots=True)
class SheetRecord:
    look: str
    age: str
    attempt: int
    raw: str
    raw_sha256: str
    prompt_sha256: str
    diagnostics: str
    passed: bool
    passing_panels: int
    frames: tuple[FrameRecord, ...]
    scale: float | None = None
    review_reason: str | None = None
    error: str | None = None


class AttemptIdentity(TypedDict):
    """Fields read back from our immutable attempt records."""
    look: str
    attempt: int
    passed: bool
    passing_panels: int
    review_reason: str | None


class CheckStatus(TypedDict):
    passed: bool


class NumericStatus(TypedDict):
    numeric_pass: bool


class SidecarIdentity(TypedDict):
    """Fields reread to validate copied delivery sidecars."""
    image_sha256: str
    checks: CheckStatus
    judge: NumericStatus
    rotation: int
