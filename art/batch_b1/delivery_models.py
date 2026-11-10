# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python3 -m art.batch_b1.delivery
"""Typed B1 handoff records, separate from runtime activation claims."""

from __future__ import annotations

from pathlib import Path
from typing import ClassVar, Literal

from pydantic import BaseModel, ConfigDict, JsonValue, RootModel


class Record(BaseModel):
    """Parse the owned fields while retaining no unvalidated extension data."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")


class Grid(Record):
    ages: tuple[str, ...]
    sexes: tuple[str, ...]
    tops: tuple[str, ...]
    hairs: tuple[str, ...]
    fallbacks: dict[str, str]

    def logical_looks(self) -> tuple[str, ...]:
        """Enumerate the bridge's age, sex, top, hair index order."""
        return tuple(
            f"{age}_{sex}_{top}_{hair}"
            for age in self.ages
            for sex in self.sexes
            for top in self.tops
            for hair in self.hairs
        )


class CatalogLook(Record):
    id: str
    age: str
    sex: str
    top: str
    hair: str
    era: str


class Top(Record):
    id: str
    era: str
    wardrobe: str


class Catalog(Record):
    schema_version: Literal[1]
    scope: str
    baseline_commit: str
    reference_game_commit: str
    tops: tuple[Top, ...]
    proposed_look_grid: Grid
    expected_new_looks: int
    expected_new_frames: int
    expected_new_portraits: int
    looks: tuple[CatalogLook, ...]


class JsonDocument(RootModel[dict[str, JsonValue]]):
    """Preserve unrelated sections of the pinned import config unchanged."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)


class CopyPair(Record):
    source: Path
    destination: Path


class CopyRow(CopyPair):
    sha256: str
    existed: bool


class FileRow(Record):
    source: str
    sha256: str
    role: Literal["people_frame", "portrait"]
    look: str


class Inventory(Record):
    status: Literal["reviewed_candidates_not_runtime_activation"] = (
        "reviewed_candidates_not_runtime_activation"
    )
    actual_looks: tuple[str, ...]
    logical_looks: tuple[str, ...]
    files: tuple[FileRow, ...]
    baseline_copies: tuple[CopyRow, ...]
    pending: tuple[str, ...]
