# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: python3 -m pytest art/flora/test_resolve_flora.py -q
"""Typed, versioned flora documents; no image loading or species branches."""
from __future__ import annotations

from typing import Annotated, ClassVar, Literal, Self, assert_never

from pydantic import BaseModel, ConfigDict, Field, model_validator

Habit = Literal["deciduous", "evergreen", "shrub"]
Size = Literal["small", "large"]
Season = Literal["spring", "summer", "autumn", "winter"]
TexturePath = Annotated[str, Field(pattern=r"^art/(?!.*\.\.)(?!.*//)[A-Za-z0-9_./-]+\.png$")]


class Record(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid", allow_inf_nan=False, regex_engine="python-re")


class SnowCap(Record):
    depth_px: int = Field(default=0, ge=0, le=16)
    alpha: float = Field(default=0, ge=0, le=0.7)


class SeasonTint(Record):
    delta_lab: tuple[float, float, float] = (0, 0, 0)
    frame: Literal["foliage", "bare"] = "foliage"
    snow_cap: SnowCap = SnowCap()


class Frame(Record):
    path: TexturePath
    canopy_mask: TexturePath
    anchor: tuple[int, int]
    canvas: tuple[int, int]

    @model_validator(mode="after")
    def anchor_in_canvas(self) -> Self:
        if any(length <= 0 for length in self.canvas) or any(
            position < 0 or position >= length
            for position, length in zip(self.anchor, self.canvas, strict=True)
        ):
            message = "Frame anchor must lie within a positive canvas"
            raise ValueError(message)
        return self


class SizeFrames(Record):
    variants: tuple[Frame, ...] = Field(min_length=1)
    bare: tuple[Frame, ...] = ()


class Species(Record):
    id: str = Field(pattern=r"^[a-z][a-z0-9_]*$")
    habit: Habit
    sizes: dict[Size, SizeFrames]
    stump: Frame | None
    seasons: dict[Season, SeasonTint]
    map_canopy: str = Field(min_length=1)
    log_material: str | None

    @model_validator(mode="after")
    def complete_species(self) -> Self:
        if set(self.sizes) != {"small", "large"}:
            message = "Both small and large sizes are required"
            raise ValueError(message)
        if set(self.seasons) != {"spring", "summer", "autumn", "winter"}:
            message = "All four season records are required"
            raise ValueError(message)
        match self.habit:
            case "deciduous":
                if any(not frames.bare for frames in self.sizes.values()):
                    message = "Deciduous sizes require bare frames"
                    raise ValueError(message)
                if self.seasons["winter"].frame != "bare":
                    message = "Deciduous winter must select bare frames"
                    raise ValueError(message)
            case "evergreen" | "shrub":
                if any(frames.bare for frames in self.sizes.values()):
                    message = "Only deciduous species have bare variants"
                    raise ValueError(message)
                if any(tint.frame == "bare" for tint in self.seasons.values()):
                    message = "Bare selection requires deciduous habit"
                    raise ValueError(message)
            case unreachable:
                assert_never(unreachable)
        if self.habit != "shrub" and (self.log_material != self.id or self.stump is None):
            message = "Tree log_material equals species id, with a stump required"
            raise ValueError(message)
        return self


class SpeciesDocument(Record):
    schema_version: Literal[1]
    species: dict[str, Species] = Field(min_length=1)
    map_canopies: dict[str, TexturePath]

    @model_validator(mode="after")
    def keyed_ids(self) -> Self:
        for key, species in self.species.items():
            if key != species.id or species.map_canopy not in self.map_canopies:
                message = "Species keys must equal ids and map canopy ids must resolve"
                raise ValueError(message)
        return self


class HabitDefault(Record):
    fallback_species: str
    seasons: dict[Season, SeasonTint]

    @model_validator(mode="after")
    def four_seasons(self) -> Self:
        if set(self.seasons) != {"spring", "summer", "autumn", "winter"}:
            message = "Habit defaults require all four seasons"
            raise ValueError(message)
        return self


class DefaultsDocument(Record):
    schema_version: Literal[1]
    unknown_habit: Habit = "deciduous"
    habits: dict[Habit, HabitDefault]

    @model_validator(mode="after")
    def all_habits(self) -> Self:
        if set(self.habits) != {"deciduous", "evergreen", "shrub"}:
            message = "Defaults require deciduous, evergreen and shrub habits"
            raise ValueError(message)
        return self


class FloraCatalog(Record):
    species: SpeciesDocument
    defaults: DefaultsDocument

    @model_validator(mode="after")
    def fallback_references(self) -> Self:
        for habit, default in self.defaults.habits.items():
            fallback = self.species.species.get(default.fallback_species)
            if fallback is None:
                message = "Fallback species must exist in the catalog"
                raise ValueError(message)
            if fallback.habit != habit:
                message = "Fallback habit must match its habit-default key"
                raise ValueError(message)
            if any(tint.frame == "bare" for tint in default.seasons.values()) and any(
                not frames.bare for frames in fallback.sizes.values()
            ):
                message = "A bare default requires bare fallback frames"
                raise ValueError(message)
        return self
