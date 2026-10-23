# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python3 -m art.compositor.render.
"""Validated scene and overlay inputs for the standalone ENV1 renderer."""
from __future__ import annotations

from enum import StrEnum
from pathlib import Path
from typing import ClassVar, Literal, Self

from pydantic import (
    AliasChoices,
    AliasPath,
    BaseModel,
    ConfigDict,
    Field,
    JsonValue,
    model_validator,
)


class InputModel(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")


class Activity(StrEnum):
    AWAKE = "awake"
    SLEEP = "sleep"
    CARRY = "carry"
    CHOP = "chop"


class Person(InputModel):
    id: int
    index: int
    x: float
    y: float
    female: bool | None
    glyph: str
    life_stage: Literal["adult", "child", "elder"]
    activity: Activity
    source: str
    carry_kind: int = -1
    carry_count: int = 0


class Scene(InputModel):
    schema_version: Literal[1]
    scene_id: str
    seed: int
    tick: int
    width: int = Field(gt=0)
    height: int = Field(gt=0)
    display_day: int
    clock_minute: int
    season: str
    camp: tuple[int, int]
    glyph_rows: tuple[str, ...]
    terrain_rows: tuple[str, ...]
    people: tuple[Person, ...]
    provenance: dict[str, JsonValue]
    limitations: tuple[str, ...] = ()

    @model_validator(mode="after")
    def check_shape(self) -> Self:
        """Reject missing map data and duplicate identities at the input edge."""
        for rows in (self.glyph_rows, self.terrain_rows):
            if len(rows) != self.height or any(len(row) != self.width for row in rows):
                message = f"{self.scene_id}: map dimensions disagree"
                raise ValueError(message)
        if len({person.id for person in self.people}) != len(self.people):
            message = f"{self.scene_id}: duplicate real person IDs"
            raise ValueError(message)
        return self

    @classmethod
    def read(cls, path: Path) -> Self:
        return cls.model_validate_json(path.read_text())


class Structure(InputModel):
    id: str
    offset: tuple[int, int]
    size: tuple[int, int]
    material: Literal["log", "stone"]
    floor: str
    door_side: Literal["south"] = "south"
    declared: Literal[True] = True


class OverlaySprite(InputModel):
    id: str
    asset: str
    offset: tuple[float, float]
    category: Literal["floor", "object", "animal"] = "object"
    declared: Literal[True] = True


class AppearanceOverride(InputModel):
    id: str
    offset: tuple[float, float]
    top: Literal["hide_wrap", "fur_cloak", "loincloth"]
    activity: Activity
    mark: str = ""
    declared: Literal[True] = True


class Overlay(InputModel):
    schema_version: Literal[1]
    coordinate_system: Literal["camp_relative_tiles"]
    interpretation: str
    structures: tuple[Structure, ...]
    sprites: tuple[OverlaySprite, ...]
    people: tuple[AppearanceOverride, ...]

    @classmethod
    def read(cls, path: Path) -> Self:
        return cls.model_validate_json(path.read_text())


class AssetEntry(InputModel):
    id: str
    path: str
    width: int = Field(gt=0, validation_alias=AliasChoices("width", AliasPath("canvas_px", 0)))
    height: int = Field(gt=0, validation_alias=AliasChoices("height", AliasPath("canvas_px", 1)))
    footprint: tuple[float, float] = Field(default=(1, 1), validation_alias=AliasChoices("footprint", "tile_footprint"))
    anchor: tuple[float, float] | None = Field(default=None, validation_alias=AliasChoices("anchor", "anchor_px", "anchor_point"))
    count_buckets: tuple[str, ...] = ()
    family: str = "item"
    status: str = "candidate"


class AssetManifest(InputModel):
    assets: tuple[AssetEntry, ...]


class LookEntry(InputModel):
    look: str


class LookManifest(InputModel):
    status: Literal["ADOPTED"]
    accepted_looks: tuple[LookEntry, ...]
    fallbacks: dict[str, str]
