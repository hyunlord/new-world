"""Native stone field on W2 thin runs and solid connected junction footprints."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

TILE: Final = 64
CAP: Final = 16
FACE_H: Final = 22
WALL_W: Final = 26
MARGIN: Final = 19
INK: Final = (20, 18, 16, 204)


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


def _ink(
    image: Image.Image, rectangle: tuple[int, int, int, int], sides: set[str]
) -> None:
    """Apply W2's2px exposed-edge ink, never ink an internal neighbour seam."""
    x, y, width, height = rectangle
    draw = ImageDraw.Draw(image)
    for side in sides:
        match side:
            case "n":
                box = (x, y, x + width - 1, y + 1)
            case "s":
                box = (x, y + height - 2, x + width - 1, y + height - 1)
            case "w":
                box = (x, y, x + 1, y + height - 1)
            case "e":
                box = (x + width - 2, y, x + width - 1, y + height - 1)
            case _:
                message = f"unknown ink side:{side}"
                raise ValueError(message)
        draw.rectangle(box, fill=INK)


def _shadow(image: Image.Image) -> None:
    """Retain the shipped W2 three-row contact shadow."""
    alpha_values = image.getchannel("A").tobytes()
    for x in range(64):
        for y in range(63, 0, -1):
            if alpha_values[y * 64 + x] >= 96:
                for step, alpha in ((1, 70), (2, 46), (3, 23)):
                    target = y + step
                    if target < 64 and alpha_values[target * 64 + x] < 10:
                        image.putpixel((x, target), (10, 8, 6, alpha))
                break


def compose_piece(mask: int, field: Image.Image) -> Image.Image:
    """Keep26px runs while filling the connected footprint of corner/T/X joins."""
    if not 0 <= mask <= 15 or field.size != (64, 64) or field.mode != "RGBA":
        message = "mask must be0..15 and native field must be64×64 RGBA"
        raise ValueError(message)
    north, east, south, west = (bool(mask & bit) for bit in (1, 2, 4, 8))
    horizontal, vertical = east or west, north or south
    x0, x1 = (0 if west else MARGIN), (64 if east else MARGIN + WALL_W)
    cap_height = 64 if south else CAP
    image = Image.new("RGBA", (64, 64))
    horizontal_rect = (x0, 0, x1 - x0, min(WALL_W, cap_height))
    vertical_rect = (MARGIN, 0, WALL_W, cap_height)
    rectangles: list[tuple[int, int, int, int]] = []
    if horizontal or not vertical:
        rectangles.append(horizontal_rect)
    if vertical:
        rectangles.append(vertical_rect)
    for x, y, width, height in rectangles:
        image.paste(field.crop((x, y, x + width, y + height)), (x, y))
    face_x0 = min(rect[0] for rect in rectangles)
    face_x1 = max(rect[0] + rect[2] for rect in rectangles)
    faces = [(face_x0, face_x1)] if not south else []
    # S-bearing side quadrants join the next tile; a short front leaves hollow L/T/X joins.
    if south and west:
        faces.append((0, MARGIN))
    if south and east:
        faces.append((MARGIN + WALL_W, TILE))
    face_height = TILE - CAP if south else FACE_H
    source_y = CAP if south else 0
    for left, right in faces:
        image.paste(
            field.crop((left, source_y, right, source_y + face_height)), (left, CAP)
        )
    for rect in rectangles:
        sides: set[str] = set()
        if not north:
            sides.add("n")
        if not west:
            sides.add("w")
        if not east:
            sides.add("e")
        if not south and rect == horizontal_rect:
            sides.add("s")
        _ink(image, rect, sides)
    if not south:
        sides = {"s"}
        if not west:
            sides.add("w")
        if not east:
            sides.add("e")
        _ink(image, (face_x0, CAP, face_x1 - face_x0, FACE_H), sides)
    else:
        for left, right in faces:
            _ink(image, (left, CAP, right - left, face_height), {"s"})
    _shadow(image)
    return image
