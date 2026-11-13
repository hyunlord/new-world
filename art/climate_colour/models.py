# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python -m art.climate_colour.export
"""Typed boundaries for the inherited B2 handoff; preserve unrelated fields."""

from pathlib import Path
from typing import ClassVar, Self

from pydantic import (
    AliasChoices,
    BaseModel,
    ConfigDict,
    Field,
    JsonValue,
    model_validator,
)


class Row(BaseModel):
    """Import metadata retains B2 extras without inventing runtime selectors."""

    model_config: ClassVar[ConfigDict] = ConfigDict(extra="allow", frozen=True)
    id: str
    path: str = Field(validation_alias=AliasChoices("path", "source"))
    sha256: str
    canvas: tuple[int, int]
    anchor: tuple[int, int]
    footprint: tuple[int, int]
    lane: str = ""
    species: str = ""
    size: str = ""
    state: str = "full"
    season: str | None = "summer"
    climate: tuple[str, ...] | None = None
    kind: str = ""
    dest: str = ""
    mask: str = ""
    sidecar: str = ""
    tint: dict[str, JsonValue] = Field(default_factory=dict)

    @model_validator(mode="after")
    def tree_geometry(self) -> Self:
        """Older olive proposals omit lane/size; their frozen canvas is explicit."""
        if "/trees/" in self.path and not self.lane:
            sizes = {(128, 112): "small", (192, 192): "large"}
            return self.model_copy(update={"lane": "trees", "size": sizes[self.canvas]})
        return self


class Draft(BaseModel):
    """B2 active and proposed sources remain distinct."""

    model_config: ClassVar[ConfigDict] = ConfigDict(extra="allow", frozen=True)
    files: tuple[Row, ...]
    proposed_files: tuple[Row, ...]
    existing_references: tuple[Row, ...]


class Ground(BaseModel):
    """One already generated tile, never a new generation attempt."""

    model_config: ClassVar[ConfigDict] = ConfigDict(extra="ignore", frozen=True)
    id: str
    attempt: int
    tile: Path
    tile_sha256: str
    raw: Path
    raw_sha256: str
    style: dict[str, JsonValue]


class ShrubFamily(BaseModel):
    """B2 catalog owns shrub climate membership."""

    model_config: ClassVar[ConfigDict] = ConfigDict(extra="ignore", frozen=True)
    id: str
    climate: tuple[str, ...]
