# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: python3 -m pytest art/materials/test_resolve.py -q.
"""Versioned, strict construction and simulation-material input contracts."""
from __future__ import annotations

from pathlib import PurePosixPath
from typing import Annotated, ClassVar, Final, Literal, Self, assert_never

from pydantic import BaseModel, ConfigDict, Field, JsonValue, model_validator

Kind = Literal["wall", "floor", "door"]
PATH_PATTERN: Final = r"^(?!/)(?!.*(?:^|/)\.{1,2}(?:/|$))(?!.*//)(?!.*(?:\s|\\|:))[^\x00-\x1f\x7f]+\.png$"
TexturePath = Annotated[str, Field(min_length=1, json_schema_extra={"pattern": PATH_PATTERN})]
STYLE_ROLES: Final = (
    ("wall", ("face", "top_ew", "top_ns", "end", "junction", "door_closed", "door_open")),
    ("floor", ("floor",)), ("door", ("door_closed", "door_open")),
)

STYLE_KIND_RULES: Final[list[JsonValue]] = [
    {"if": {"properties": {"kind": {"const": kind}}, "required": ["kind"]},
     "then": {"properties": {"textures": {"required": [role for role in roles],
               "properties": {role: {"type": "string"} for role in roles}}}}}
    for kind, roles in STYLE_ROLES
]
STYLE_SCHEMA_EXTRA: Final[dict[str, JsonValue]] = {
    "allOf": [*STYLE_KIND_RULES,
        {"if": {"properties": {"kind": {"const": "wall"}}, "required": ["kind"]},
         "then": {"required": ["ends", "corner"],
                  "properties": {"ends": {"type": "string"}, "corner": {"type": "string"}}}}],
}


class Record(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid", allow_inf_nan=False)


class LabTint(Record):
    dL: float = Field(default=0, ge=-100, le=100)
    da: float = Field(default=0, ge=-128, le=128)
    db: float = Field(default=0, ge=-128, le=128)


class Top(Record):
    value: float = Field(default=.75, ge=0, le=1)
    saturation: float = Field(default=.85, ge=0, le=1)
    blur_px: float = Field(default=2, ge=0, le=8)
    ink_edge_px: int = Field(default=2, ge=0, le=8)
    width_px: int = Field(default=64, ge=1, le=64)
    side_shadow: float = Field(default=0, ge=0, le=1)
    orientation: Literal["along_run"] = "along_run"


class Face(Record):
    height_px: Literal[44] = 44


class TextureSet(Record):
    face: TexturePath | None = None
    top_ew: TexturePath | None = None
    top_ns: TexturePath | None = None
    end: TexturePath | None = None
    junction: TexturePath | None = None
    door_closed: TexturePath | None = None
    door_open: TexturePath | None = None
    floor: TexturePath | None = None

    @model_validator(mode="after")
    def safe_paths(self) -> Self:
        for path in (self.face, self.top_ew, self.top_ns, self.end, self.junction,
                     self.door_closed, self.door_open, self.floor):
            if path is None:
                continue
            normalized = PurePosixPath(path)
            invalid = (normalized.is_absolute() or ".." in normalized.parts
                       or normalized.as_posix() != path or normalized.suffix != ".png"
                       or any(character.isspace() or ord(character) < 32 or ord(character) == 127 for character in path)
                       or "\\" in path or ":" in path)
            if invalid:
                message = f"Texture path must be a canonical repository-relative PNG path: {path!r}"
                raise ValueError(message)
        return self


class ConstructionStyle(Record):
    model_config: ClassVar[ConfigDict] = ConfigDict(json_schema_extra=STYLE_SCHEMA_EXTRA)
    id: str = Field(min_length=1)
    kind: Kind
    era: str
    textures: TextureSet
    top: Top = Top()
    face: Face = Face()
    ends: TexturePath | None = None
    corner: TexturePath | None = None
    prompt_template: str
    sim_blueprints: tuple[str, ...]
    default_tint: LabTint = LabTint()

    @model_validator(mode="after")
    def required_textures(self) -> Self:
        match self.kind:
            case "wall":
                required = (self.textures.face, self.textures.top_ew, self.textures.top_ns,
                            self.textures.end, self.textures.junction,
                            self.textures.door_closed, self.textures.door_open)
                if self.ends != self.textures.end or self.corner != self.textures.junction:
                    message = "Wall ends/corner must reference the declared end/junction textures"
                    raise ValueError(message)
            case "floor":
                required = (self.textures.floor,)
            case "door":
                required = (self.textures.door_closed, self.textures.door_open)
            case _:
                assert_never(self.kind)
        if not all(required):
            message = f"Incomplete texture set for {self.kind} style {self.id}"
            raise ValueError(message)
        return self


class KindDefaults(Record):
    wall: str
    floor: str
    door: str

    def select(self, kind: Kind) -> str:
        match kind:
            case "wall":
                return self.wall
            case "floor":
                return self.floor
            case "door":
                return self.door
            case _:
                assert_never(kind)


class StylesDocument(Record):
    schema_version: Literal[1]
    default_styles: KindDefaults
    styles: tuple[ConstructionStyle, ...] = Field(min_length=1)


class MaterialLook(Record):
    id: str
    sim_id: int = Field(ge=0, le=4294967295)
    category: str
    tint: LabTint | None = None
    texture_override: dict[str, TextureSet] = Field(default_factory=dict)


class MaterialsDocument(Record):
    schema_version: Literal[1]
    source_revision: str
    source_path: str
    materials: tuple[MaterialLook, ...] = Field(min_length=1)


class CategoryDefault(Record):
    category: str
    default_style: str
    kind_defaults: KindDefaults
    tint: LabTint


class CategoriesDocument(Record):
    schema_version: Literal[1]
    categories: tuple[CategoryDefault, ...] = Field(min_length=1)


class MaterialCatalog(Record):
    construction: StylesDocument
    materials: MaterialsDocument
    defaults: CategoriesDocument

    @model_validator(mode="after")
    def references(self) -> Self:
        styles = {style.id: style for style in self.construction.styles}
        categories = {row.category for row in self.defaults.categories}
        if len(styles) != len(self.construction.styles) or len(categories) != len(self.defaults.categories):
            message = "Duplicate style or category id"
            raise ValueError(message)
        if (len({row.id for row in self.materials.materials}) != len(self.materials.materials)
                or len({row.sim_id for row in self.materials.materials}) != len(self.materials.materials)):
            message = "Duplicate material id"
            raise ValueError(message)
        blueprints: set[tuple[Kind, str]] = set()
        for style in self.construction.styles:
            for blueprint in style.sim_blueprints:
                if (style.kind, blueprint) in blueprints:
                    message = f"Ambiguous blueprint: {blueprint}"
                    raise ValueError(message)
                blueprints.add((style.kind, blueprint))
        for selection in (self.construction.default_styles, *(row.kind_defaults for row in self.defaults.categories)):
            for kind in ("wall", "floor", "door"):
                selected = styles.get(selection.select(kind))
                if selected is None or selected.kind != kind:
                    message = f"Default style missing or wrong kind: {kind}"
                    raise ValueError(message)
        for row in self.defaults.categories:
            if row.default_style != row.kind_defaults.wall:
                message = "Category default_style and wall kind default disagree"
                raise ValueError(message)
        for row in self.materials.materials:
            if row.category not in categories or not row.texture_override.keys() <= styles.keys():
                message = f"Unknown category or texture override style: {row.id}"
                raise ValueError(message)
        return self
