# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "numpy", "typer"]
# ///
# How to run: imported by python3 -m art.compositor.polish2_boards.
"""Lossless board layout with native-pixel panel provenance."""
from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import ClassVar, Final

from PIL import Image, ImageDraw, ImageFont
from pydantic import BaseModel, ConfigDict

from art.compositor.assets import REPO
from art.compositor.render import sha256
from art.compositor.struct_render import save_lossless

FONT: Final = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 16)
SMALL: Final = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 12)
BACKGROUND: Final = (242, 236, 219)


@dataclass(frozen=True, slots=True)
class BoardPanel:
    source: Path
    crop: tuple[int, int, int, int]
    origin: tuple[int, int]
    label: str
    scale: int = 1


class PanelRecord(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    path: str
    sha256: str
    crop: tuple[int, int, int, int]
    origin: tuple[int, int]
    label: str
    integer_scale: int


class BoardRecord(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    path: str
    sha256: str
    bytes: int
    dimensions: tuple[int, int]
    title: str
    panels: tuple[PanelRecord, ...]
    owner_approval: str = "PENDING"


@dataclass(frozen=True, slots=True)
class BoardLayout:
    size: tuple[int, int]
    title: str
    panels: tuple[BoardPanel, ...]


def make_board(destination: Path, layout: BoardLayout, *, enforce_cap: bool = True,
               optimize_oversize: bool = False) -> BoardRecord:
    """Copy at 1× or exact integer 2×; reject empty or out-of-bounds panels."""
    if not layout.panels:
        message = "An owner board must contain actual image panels"
        raise ValueError(message)
    board = Image.new("RGB", layout.size, BACKGROUND)
    draw = ImageDraw.Draw(board)
    draw.text((12, 10), layout.title, font=FONT, fill=(47, 39, 31))
    records: list[PanelRecord] = []
    for panel in layout.panels:
        if panel.scale not in (1, 2):
            message = "Owner panels permit only native or exact double scale"
            raise ValueError(message)
        with Image.open(panel.source) as source:
            if not (0 <= panel.crop[0] < panel.crop[2] <= source.width and
                    0 <= panel.crop[1] < panel.crop[3] <= source.height):
                message = f"Crop exceeds source: {panel.source}: {panel.crop}"
                raise ValueError(message)
            cropped = source.convert("RGBA").crop(panel.crop)
        if panel.scale == 2:
            cropped = cropped.resize((cropped.width * 2, cropped.height * 2), Image.Resampling.NEAREST)
        x, y = panel.origin
        if min(x, y) < 0 or x + cropped.width > board.width or y + cropped.height > board.height - 28:
            message = f"Panel exceeds board: {panel.label}"
            raise ValueError(message)
        board.paste(cropped, panel.origin, cropped)
        if panel.label:
            draw.text((x, y - 19), panel.label, font=SMALL, fill=(47, 39, 31))
        records.append(PanelRecord(path=str(panel.source), sha256=sha256(panel.source), crop=panel.crop,
                                   origin=panel.origin, label=panel.label, integer_scale=panel.scale))
    draw.text((12, board.height - 20), "CANDIDATE / OWNER CHOICE PENDING / exact native pixels and nearest 2x", font=SMALL, fill=(47, 39, 31))
    destination.parent.mkdir(parents=True, exist_ok=True)
    save_lossless(board, destination, enforce_cap=enforce_cap, optimize_oversize=optimize_oversize)
    record = BoardRecord(path=str(destination.relative_to(REPO)) if destination.is_relative_to(REPO) else str(destination),
                         sha256=sha256(destination), bytes=destination.stat().st_size, dimensions=board.size,
                         title=layout.title, panels=tuple(records))
    _ = destination.with_suffix(".json").write_text(json.dumps(record.model_dump(mode="json"), indent=2) + "\n")
    return record
