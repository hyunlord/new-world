# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: imported by sprite_build.py.
"""Prevent reviewed portrait-reference pixels from drifting during later repairs."""
from __future__ import annotations

import hashlib
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path

import numpy as np
from PIL import Image
from pydantic import BaseModel

from art.compositor.face_variants import ROOT
from art.pipeline_polish2.sprite_colour import BoolPixels


class FrozenFile(BaseModel):
    path: str
    sha256: str


class FrozenLook(BaseModel):
    look: str
    source: FrozenFile
    skin_mask: FrozenFile
    hair_mask: FrozenFile
    variants: tuple[FrozenFile, ...]


class ReferenceFreeze(BaseModel):
    looks: tuple[FrozenLook, ...]
    protected_records: tuple[FrozenFile, ...] = ()


@dataclass(frozen=True, slots=True)
class FreezeGuard:
    hashes: dict[Path, str]

    @classmethod
    def read(cls) -> FreezeGuard:
        path = ROOT / "art/character/variant_reference_freeze.json"
        if not path.is_file():
            return cls({})
        freeze = ReferenceFreeze.model_validate_json(path.read_bytes())
        hashes: dict[Path, str] = {}
        for look in freeze.looks:
            for entry in (look.source, look.skin_mask, look.hair_mask, *look.variants):
                file = ROOT / entry.path
                if hashlib.sha256(file.read_bytes()).hexdigest() != entry.sha256:
                    message = f"Frozen reference already changed: {entry.path}"
                    raise ValueError(message)
                hashes[file] = entry.sha256
        for entry in freeze.protected_records:
            file = ROOT / entry.path
            if hashlib.sha256(file.read_bytes()).hexdigest() != entry.sha256:
                message = f"Frozen transform record changed: {entry.path}"
                raise ValueError(message)
            hashes[file] = entry.sha256
        return cls(hashes)

    def masks_for(self, look: str) -> tuple[BoolPixels, BoolPixels] | None:
        folder = ROOT / "art/character/variant_masks" / look / "S"
        skin_path, hair_path = folder / "idle_skin.png", folder / "idle_hair.png"
        if skin_path not in self.hashes or hair_path not in self.hashes:
            return None
        with Image.open(skin_path) as opened:
            skin = np.asarray(opened) > 0
        with Image.open(hair_path) as opened:
            hair = np.asarray(opened) > 0
        return skin, hair

    def save_png(self, image: Image.Image, path: Path, *, optimize: bool = False) -> None:
        buffer = BytesIO()
        image.save(buffer, format="PNG", optimize=optimize)
        data = buffer.getvalue()
        expected = self.hashes.get(path)
        if expected is not None and hashlib.sha256(data).hexdigest() != expected:
            message = f"Refusing to change frozen portrait reference: {path}"
            raise ValueError(message)
        _ = path.write_bytes(data)
