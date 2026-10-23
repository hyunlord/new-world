# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: python3 -m pytest art/materials/test_resolve.py -q.
"""Pure construction × simulation-material selection; file I/O is separate."""
from __future__ import annotations

from pathlib import Path
from typing import Literal

from art.materials.schema import (
    CategoriesDocument,
    Face,
    Kind,
    LabTint,
    MaterialCatalog,
    MaterialsDocument,
    Record,
    StylesDocument,
    TextureSet,
    Top,
)


class ResolvedMaterial(Record):
    style_id: str
    material_id: str | int
    material_key: str | None
    category: str | None
    kind: Kind
    textures: TextureSet
    tint: LabTint
    tint_source: Literal["material", "category", "style"]
    style_source: Literal["style", "blueprint", "category", "default"]
    top: Top
    face: Face


def read_catalog(root: Path) -> MaterialCatalog:
    """Parse repository JSON at the I/O boundary, without opening textures."""
    folder = root / "art/materials"
    return MaterialCatalog(
        construction=StylesDocument.model_validate_json((folder / "construction_styles.json").read_bytes()),
        materials=MaterialsDocument.model_validate_json((folder / "material_looks.json").read_bytes()),
        defaults=CategoriesDocument.model_validate_json((folder / "category_defaults.json").read_bytes()),
    )


def resolve(catalog: MaterialCatalog, style_id: str | None, material_id: str | int, *,
            category: str | None = None, kind: Kind = "wall") -> ResolvedMaterial:
    """Resolve without I/O or mutation; unknown ids never require new code.

    An unknown material supplies its category from simulation metadata. With no
    usable category its known style supplies the tint. Unknown blueprints use
    category defaults, then the document's kind default. Texture overrides are
    partial per-style replacements layered over the style's complete set.
    """
    material = next((row for row in catalog.materials.materials if row.id == material_id or row.sim_id == material_id), None)
    category_id = material.category if material is not None else category
    fallback = next((row for row in catalog.defaults.categories if row.category == category_id), None)
    styles = tuple(style for style in catalog.construction.styles if style.kind == kind)
    style = next((row for row in styles if row.id == style_id), None)
    style_source: Literal["style", "blueprint", "category", "default"] = "style"
    if style is None:
        style = next((row for row in styles if style_id in row.sim_blueprints), None)
        style_source = "blueprint"
    if style is None:
        default_id = fallback.kind_defaults.select(kind) if fallback else catalog.construction.default_styles.select(kind)
        style = next(row for row in styles if row.id == default_id)
        style_source = "category" if fallback else "default"
    tint, tint_source = style.default_tint, "style"
    if fallback is not None:
        tint, tint_source = fallback.tint, "category"
    if material is not None and material.tint is not None:
        tint, tint_source = material.tint, "material"
    textures = style.textures
    if material is not None and (override := material.texture_override.get(style.id)) is not None:
        textures = TextureSet.model_validate({**textures.model_dump(), **override.model_dump(exclude_none=True)})
    return ResolvedMaterial(
        style_id=style.id, material_id=material_id, material_key=material.id if material else None,
        category=category_id, kind=kind,
        textures=textures, tint=tint, tint_source=tint_source, style_source=style_source,
        top=style.top, face=style.face,
    )
