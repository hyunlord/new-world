# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python -m art.batch_a2.rooms.
"""Typed boundaries for derived room recipes and evidence."""

from __future__ import annotations

from typing import ClassVar, Literal, Self

from pydantic import BaseModel, ConfigDict, Field, model_validator

from art.materials.resolve import ResolvedMaterial  # noqa: TC001 - Pydantic resolves model fields at runtime.
from art.materials.schema import Record

Point = tuple[int, int]


class PropSpec(Record):
    """One existing image, placed without resampling."""

    source: str
    cell: Point
    role: str
    anchor: Point | None = None
    offset: Point = (0, 0)
    footprint: Point = (1, 1)


class RoomSpec(Record):
    """Grid topology and existing contents, never a generated building image."""

    id: str = Field(pattern=r"^[a-z][a-z0-9_]*$")
    era: str
    footprint: Point
    wall_style: str
    material: str
    cells: tuple[Point, ...] = Field(min_length=1)
    floor: bool = True
    door_cell: Point | None = None
    door_open: bool = True
    props: tuple[PropSpec, ...] = ()
    notes: tuple[str, ...] = ()

    @model_validator(mode="after")
    def grid_bounds(self) -> Self:
        """Reject ambiguous duplicate topology and out-of-footprint placements."""
        width, height = self.footprint
        points = (*self.cells, *(prop.cell for prop in self.props))
        if width < 1 or height < 1 or any(not (0 <= x < width and 0 <= y < height) for x, y in points):
            raise RecipeError(self.id, "cell outside footprint")
        if len(set(self.cells)) != len(self.cells) or (
            self.door_cell is not None and self.door_cell not in self.cells
        ):
            raise RecipeError(self.id, "duplicate wall or door absent from topology")
        return self


class Recipes(Record):
    """Versioned collection of explicit composition recipes."""

    schema_version: Literal[1] = 1
    rooms: tuple[RoomSpec, ...]


class AnchorMetadata(BaseModel):
    """Read only the published anchor from an existing full sidecar."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    anchor: Point | None = None
    anchor_point: Point | None = None


class RecipeError(ValueError):
    """Invalid recipe with a named subject and concrete reason."""

    def __init__(self, subject: str, reason: str) -> None:
        """Attach the recipe subject to a boundary failure."""
        self.subject: str = subject
        self.reason: str = reason
        super().__init__(f"{subject}: {reason}")


class SourceHash(Record):
    """Exact input bytes, relative to the repository."""

    path: str
    sha256: str


class LayerRecord(Record):
    """One independently placeable image and its source-canvas anchor."""

    path: str
    sha256: str
    kind: Literal["floor", "wall", "prop"]
    cell: Point
    size: Point
    anchor: Point
    world_origin: Point
    world_base: Point
    sort_key: tuple[int, int, int]
    role: str
    sources: tuple[SourceHash, ...]
    operations: tuple[str, ...]
    footprint: Point


class RoomManifest(Record):
    """Derived composition evidence, with no adoption claim."""

    recipe: RoomSpec
    provenance: Literal["derived_existing_material_composition"] = "derived_existing_material_composition"
    status: Literal["DERIVED_AWAITING_REVIEW", "DERIVED_INCOMPLETE"]
    exposed_wall_height: Literal[60] = 60
    tile_size: Literal[64] = 64
    resolved_wall: ResolvedMaterial
    resolved_floor: ResolvedMaterial
    layers: tuple[LayerRecord, ...]
    inputs: tuple[SourceHash, ...]
    unresolved: tuple[str, ...]
    preview_origin: Point
    preview_sha256: str
    walls_reference_origin: Point
    operations: tuple[str, ...] = (
        "Existing material resolver tint/top-tone and wall topology renderer; no new painting.",
        "World-phased 64px crops of existing top-down floor; unchanged prop pixels.",
        "South floor row stops at wall base(y+43), retaining44px of its64px source crop.",
        "Floor layers first; standing layers sorted by base_y then base_x.",
        "Flattened previews are review-only; runtime must place the individual layers.",
        "60px exposed wall is16px cap+44px face; north-south connectors have64px top surface.",
    )
