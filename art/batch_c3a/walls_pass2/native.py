"""Native stone RGB inside immutable shipped W2 alpha stencils."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

TILE: Final = 64


@dataclass(frozen=True, slots=True)
class CropScore:
    """One native crop's unchanged wrap discontinuities."""

    x: int
    y: int
    horizontal: float
    vertical: float
    width: int = 64
    height: int = 32


def crop_scores(source: Path) -> tuple[CropScore, ...]:
    """Compare every64×32 native crop; never change its pixels to close a seam."""
    with Image.open(source) as opened:
        image = opened.convert("RGBA")
    if (
        image.width < 64
        or image.height < 32
        or min(image.getchannel("A").tobytes()) != 255
    ):
        message = "course source must be opaque and at least64×32"
        raise ValueError(message)
    result: list[CropScore] = []
    for y in range(image.height - 31):
        for x in range(image.width - 63):
            crop = image.crop((x, y, x + 64, y + 32)).convert("RGB")
            left = crop.crop((0, 0, 1, 32)).tobytes()
            right = crop.crop((63, 0, 64, 32)).tobytes()
            upper = crop.crop((0, 0, 64, 1)).tobytes()
            lower = crop.crop((0, 31, 64, 32)).tobytes()
            horizontal = sum(
                abs(a - b) for a, b in zip(left, right, strict=True)
            ) / len(left)
            vertical = sum(abs(a - b) for a, b in zip(upper, lower, strict=True)) / len(
                upper
            )
            result.append(CropScore(x, y, horizontal, vertical))
    return tuple(result)


def course_field(source: Path) -> tuple[Image.Image, CropScore]:
    """Repeat the least-discontinuous native64×32 crop twice without filtering."""
    selected = min(
        crop_scores(source),
        key=lambda s: (
            max(s.horizontal, s.vertical),
            s.horizontal + s.vertical,
            s.y,
            s.x,
        ),
    )
    with Image.open(source) as opened:
        crop = opened.convert("RGBA").crop(
            (selected.x, selected.y, selected.x + 64, selected.y + 32)
        )
    field = Image.new("RGBA", (64, 64))
    field.paste(crop, (0, 0))
    field.paste(crop, (0, 32))
    return field, selected


def compose_piece(
    stencil: Image.Image, field: Image.Image, *, tile: tuple[int, int] = (0, 0)
) -> Image.Image:
    """Sample world-periodic stone RGB inside exact shipped alpha, retaining contact."""
    if stencil.width != 64 or stencil.height not in (60, 64) or field.size != (64, 64):
        message = "stencil must be64×64 or shipped64×60 door; field must be64×64"
        raise ValueError(message)
    result = bytearray(stencil.convert("RGBA").tobytes())
    alpha = stencil.getchannel("A").tobytes()
    texture = field.convert("RGB").tobytes()
    for y in range(stencil.height):
        for x in range(stencil.width):
            # Contact/shadow pixels use the shipped low-alpha black/brown unchanged.
            if alpha[y * 64 + x] < 96:
                continue
            neighbours = [
                (x + dx, y + dy)
                for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1))
                if 0 <= x + dx < stencil.width and 0 <= y + dy < stencil.height
            ]
            edge = any(alpha[point[1] * 64 + point[0]] < 96 for point in neighbours)
            # Tile boundaries are never outlined: their adjacent tile continues the band.
            sample_x = (tile[0] * TILE + x) % field.width
            sample_y = (tile[1] * TILE + y) % field.height
            offset = (sample_y * field.width + sample_x) * 3
            rgb = texture[offset : offset + 3]
            lit_edge = any(
                alpha[ny * 64 + nx] < 96 for nx, ny in neighbours if nx < x or ny < y
            )
            shade = (24 if lit_edge else 48) if edge else 0
            offset = (y * 64 + x) * 4
            result[offset : offset + 3] = bytes(max(0, value - shade) for value in rgb)
    return Image.frombytes("RGBA", stencil.size, bytes(result))


def door_frame(name: str) -> Image.Image:
    """Conservative hand-inspected frame mask excludes every wooden moving leaf."""
    if name not in {
        f"door_{orientation}_{state}"
        for orientation in ("ew", "ns")
        for state in ("closed", "open")
    }:
        message = f"unknown shipped door: {name}"
        raise ValueError(message)
    frame = Image.new("L", (64, 64))
    draw = ImageDraw.Draw(frame)
    if "_ew_" in name:
        draw.rectangle((0, 0, 19, 63), fill=255)
        draw.rectangle((0, 0, 63, 7), fill=255)
        if name.endswith("closed"):
            draw.rectangle((50, 0, 63, 63), fill=255)
        else:
            draw.rectangle((61, 0, 63, 63), fill=255)
    else:
        draw.rectangle((0, 0, 63, 14 if name.endswith("closed") else 6), fill=255)
        draw.rectangle((0, 48 if name.endswith("closed") else 47, 63, 63), fill=255)
        draw.rectangle((0, 24, 8, 56), fill=255)
    return frame


def compose_door(name: str, stencil: Image.Image, field: Image.Image) -> Image.Image:
    """Recolour only fixed frame pixels; wood and hardware remain shipped RGBA."""
    return Image.composite(
        compose_piece(stencil, field),
        stencil,
        door_frame(name).crop((0, 0, *stencil.size)),
    )
