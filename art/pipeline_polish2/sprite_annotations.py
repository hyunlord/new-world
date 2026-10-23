# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: read by sprite_build.py and sprite_audit.py.
"""Independent source-image anatomy annotations for the south-idle mask gate."""
from __future__ import annotations

from pathlib import Path
from typing import ClassVar, Literal

from pydantic import BaseModel, ConfigDict


class SourceAnatomy(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    look: str
    source: str
    source_sha256: str
    face: tuple[int, int, int, int]
    forearms: tuple[tuple[int, int, int, int], tuple[int, int, int, int]]
    hair_region: tuple[int, int, int, int] | None = None
    hair_regions: tuple[tuple[int, int, int, int], ...] = ()
    clothing_regions: tuple[tuple[int, int, int, int], ...] = ()
    convention: Literal["left_top_right_bottom_exclusive"]


class AnatomyAnnotations(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    schema_version: Literal[1]
    method: str
    annotated_by: str
    review_status: str
    frames: tuple[SourceAnatomy, ...]

    @classmethod
    def read(cls, path: Path) -> AnatomyAnnotations:
        return cls.model_validate_json(path.read_bytes())

    def for_look(self, look: str) -> SourceAnatomy:
        return next(frame for frame in self.frames if frame.look == look)
