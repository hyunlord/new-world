# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by art.batch_b3.rooms.render.
"""Explicit layered construction recipes and their export evidence."""

from __future__ import annotations

from typing import ClassVar, Literal, Self

from pydantic import BaseModel, ConfigDict, Field, model_validator

Point = tuple[int, int]
Stage = Literal["blueprint", "foundation", "half_built", "built", "damaged", "ruined"]


class Record(BaseModel):
    """Reject unknown authored fields and keep parsed inputs immutable."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")


class RecipeError(ValueError):
    """Identify a recipe whose geometry or dependency is incomplete."""

    def __init__(self, subject: str, reason: str) -> None:
        self.subject: str = subject
        self.reason: str = reason
        super().__init__(f"{subject}: {reason}")


class Prop(Record):
    """An unchanged adopted sprite placed at its declared source anchor."""

    source: str
    cell: Point
    role: str
    anchor: Point
    offset: Point = (0, 0)
    footprint: Point = (1, 1)


class Course(Record):
    """One contiguous course with the original top depth and cropped facade."""

    height: Literal[24, 60]
    cells: tuple[Point, ...] = Field(min_length=1)
    door_cell: Point | None = None
    door_open: bool = True


class Room(Record):
    """A proposed visual lifecycle state; does not create simulation state."""

    id: str
    family: str
    era: str
    state: Stage
    source_recipe: str
    source_commit: str
    footprint: Point
    wall_style: str
    material: str
    floor: bool
    courses: tuple[Course, ...]
    props: tuple[Prop, ...] = ()
    notes: tuple[str, ...] = ()

    @model_validator(mode="after")
    def geometry(self) -> Self:
        """Reject off-grid cells and doors unsupported by a full-height wall."""
        width, height = self.footprint
        cells = tuple(cell for course in self.courses for cell in course.cells)
        cells += tuple(prop.cell for prop in self.props)
        if min(self.footprint) < 1 or any(
            not (0 <= x < width and 0 <= y < height) for x, y in cells
        ):
            raise RecipeError(self.id, "cell lies outside declared footprint")
        for course in self.courses:
            if len(set(course.cells)) != len(course.cells):
                raise RecipeError(self.id, "duplicate course cell")
            if course.door_cell is not None and (
                course.door_cell not in course.cells or course.height != 60
            ):
                raise RecipeError(self.id, "door requires full-height occupied cell")
        return self


class Recipes(Record):
    """The full explicitly authored construction matrix."""

    schema_version: Literal[1] = 1
    rooms: tuple[Room, ...]


class LayerRecord(Record):
    """One importable layer and its unchanged grid placement."""

    path: str
    sha256: str
    kind: Literal["floor", "wall", "prop"]
    cell: Point
    size: Point
    anchor: Point
    origin: Point
    operations: tuple[str, ...]


class Source(Record):
    """Immutable source bytes used in a composition."""

    path: str
    sha256: str


class Manifest(Record):
    """The preview is evidence, while layers remain separately composable."""

    recipe: Room
    status: Literal["RECIPE_AWAITING_VISUAL_REVIEW"] = "RECIPE_AWAITING_VISUAL_REVIEW"
    provenance: str = "existing_material_composition_no_new_generation"
    layers: tuple[LayerRecord, ...]
    sources: tuple[Source, ...]
    preview_origin: Point
    preview_anchor: Point
    preview_sha256: str
