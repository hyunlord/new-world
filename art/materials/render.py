# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: imported by art.compositor.walls and art.materials.preview.
"""Load one resolved texture set, applying material tone without painting."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import assert_never

from PIL import Image

from art.materials.resolve import ResolvedMaterial
from art.materials.tone import shift_image, top_tone


@dataclass(frozen=True, slots=True)
class MaterialWallTextures:
    resolved: ResolvedMaterial
    face: Image.Image
    top_ew: Image.Image
    top_ns: Image.Image
    end: Image.Image
    junction: Image.Image
    door_closed: Image.Image
    door_open: Image.Image


def _read(root: Path, path: str | None, size: tuple[int, int]) -> Image.Image:
    if path is None:
        message = "Required resolved texture path is missing"
        raise ValueError(message)
    target = (root / path).resolve()
    if not target.is_relative_to(root.resolve()):
        message = f"Texture path escapes the repository root: {path}"
        raise ValueError(message)
    with Image.open(target) as source:
        image = source.convert("RGBA")
    if image.size != size:
        message = f"{path} must be {size}; received {image.size}"
        raise ValueError(message)
    return image


def load_textures(root: Path, resolved: ResolvedMaterial) -> MaterialWallTextures:
    """Load validated wall dimensions and transform a copy of each source."""
    if resolved.kind != "wall":
        message = f"Expected wall style, received {resolved.kind}"
        raise ValueError(message)
    paths = resolved.textures
    def tinted(path: str | None, size: tuple[int, int]) -> Image.Image:
        return shift_image(_read(root, path, size), resolved.tint)
    return MaterialWallTextures(
        resolved, tinted(paths.face, (256, 44)),
        top_tone(tinted(paths.top_ew, (256, 64)), resolved.top),
        top_tone(tinted(paths.top_ns, (64, 256)), resolved.top),
        tinted(paths.end, (24, 44)),
        top_tone(tinted(paths.junction, (64, 64)), resolved.top),
        tinted(paths.door_closed, (64, 60)), tinted(paths.door_open, (64, 60)),
    )


def load_surface(root: Path, resolved: ResolvedMaterial, *, opened: bool = False) -> Image.Image:
    """Use the same resolver for standalone floors and doors."""
    match resolved.kind:
        case "floor":
            path, size = resolved.textures.floor, (256, 256)
        case "door":
            path = resolved.textures.door_open if opened else resolved.textures.door_closed
            size = (64, 60)
        case "wall":
            message = "Wall styles require load_textures and topology assembly"
            raise ValueError(message)
        case _:
            assert_never(resolved.kind)
    return shift_image(_read(root, path, size), resolved.tint)
