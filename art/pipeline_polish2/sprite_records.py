# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: imported by sprite diagnostic and integration modules.
"""Shared typed records for immutable sprite evidence and stored transforms."""
from __future__ import annotations

from pathlib import Path

import numpy as np
from PIL import Image
from pydantic import BaseModel, JsonValue

from art.pipeline_polish2.sprite_colour import BoolPixels


class StoredTransform(BaseModel):
    look: str
    variant: str
    skin_delta_lab: tuple[float, float, float]
    hair_delta_lab: tuple[float, float, float]


class StoredTransforms(BaseModel):
    transforms: tuple[StoredTransform, ...]


class FrameEvidence(BaseModel):
    frames: tuple[dict[str, JsonValue], ...]
    source_reports: dict[str, str]


def field(row: dict[str, JsonValue], name: str) -> str:
    value = row[name]
    if not isinstance(value, str):
        raise TypeError(f"Expected string {name}")
    return value


def read_mask(path: Path) -> BoolPixels:
    with Image.open(path) as opened:
        return np.asarray(opened) > 0
