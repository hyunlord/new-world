# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: imported by sprite_diagnostic.py and sprite_masks.py.
"""Source-reviewed polygon envelopes and bounded local skin-shadow fitting."""
from __future__ import annotations

from pathlib import Path
from typing import Final, Literal

import numpy as np
from PIL import Image, ImageDraw
from pydantic import BaseModel

from art.pipeline_polish2.sprite_colour import (
    BoolPixels,
    BytePixels,
    FloatPixels,
    nearest_label,
    rgb_to_lab,
    squared_distance,
)

Polygons = tuple[tuple[tuple[int, int], ...], ...]
LOCAL_SEEDS: Final = np.array(((135, 80, 45), (210, 155, 100), (91, 58, 33), (28, 18, 10), (116, 112, 99)), dtype=np.float64)


class SourceEnvelope(BaseModel):
    look: str
    source: str | None = None
    source_sha256: str
    facing: Literal["S", "E"] = "S"
    pose: Literal["idle", "walk_a", "work_a", "carry", "sleep"] = "idle"
    garment_exclusions: Polygons
    hair_envelopes: Polygons
    skin_supports: Polygons = ()
    prop_exclusions: Polygons = ()
    hair_exclusions: Polygons = ()
    facial_hair_envelopes: Polygons = ()

    @property
    def all_hair_envelopes(self) -> Polygons:
        return self.hair_envelopes + self.facial_hair_envelopes


class EnvelopeReport(BaseModel):
    frames: tuple[SourceEnvelope, ...]

    @classmethod
    def read(cls, path: Path) -> EnvelopeReport:
        return cls.model_validate_json(path.read_bytes())


def polygon_union(shape: tuple[int, int], polygons: Polygons) -> BoolPixels:
    image = Image.new("L", (shape[1], shape[0]), 0)
    draw = ImageDraw.Draw(image)
    for vertices in polygons:
        draw.polygon(vertices, fill=255)
    return np.asarray(image) > 0


def semantic_hair_support(envelope: SourceEnvelope, shape: tuple[int, int]) -> BoolPixels:
    """Declared skin wins over brown hair colour; observed beard is explicit."""
    hair = polygon_union(shape, envelope.all_hair_envelopes)
    skin = polygon_union(shape, envelope.skin_supports)
    forbidden_hair = polygon_union(shape, envelope.hair_exclusions)
    facial_hair = facial_hair_support(envelope, shape)
    excluded = polygon_union(shape, envelope.garment_exclusions + envelope.prop_exclusions)
    return hair & (~skin | facial_hair) & ~excluded & ~forbidden_hair


def facial_hair_support(envelope: SourceEnvelope, shape: tuple[int, int]) -> BoolPixels:
    """Explicit ear/cheek skin holes override a coarse beard envelope."""
    return polygon_union(shape, envelope.facial_hair_envelopes) & ~polygon_union(shape, envelope.hair_exclusions)


def source_hair_reference(rgba: BytePixels, lab: FloatPixels, envelope: SourceEnvelope) -> BoolPixels:
    """Independent source-colour candidate; semantic completeness needs review."""
    rgb = rgba[:, :, :3].astype(np.int16)
    colour = (lab[:, :, 0] <= 52) & (rgb[:, :, 0] - rgb[:, :, 1] < 55)
    return semantic_hair_support(envelope, (rgba.shape[0], rgba.shape[1])) & colour & (rgba[:, :, 3] >= 16)


def local_skin_shadow(rgba: BytePixels, lab: FloatPixels) -> BoolPixels:
    """Resolve warm skin shadows locally; callers must supply semantic support."""
    centres = rgb_to_lab(LOCAL_SEEDS)
    distances = squared_distance(lab[:, :, None, :] - centres[None, None, :, :])
    labels = nearest_label(distances)
    rgb = rgba[:, :, :3].astype(np.float64)
    warm = (rgb[:, :, 0] >= rgb[:, :, 1] * 1.3) & (rgb[:, :, 1] >= rgb[:, :, 2] * 1.3)
    return (np.equal(labels, 0) | np.equal(labels, 1)) & warm & (lab[:, :, 0] >= 32)
