#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: uv run art/techmap/sheets.py FAMILY OUTPUT.png INPUT.png...
# FAMILY: icon (32/48/64px), banner (264x88), chapter (520x260).
"""Contact sheets place unchanged A4 exports beside current adopted item art."""

from __future__ import annotations

import hashlib
import json
import math
import sys
from pathlib import Path
from typing import Final, Literal, TypedDict, assert_never

from PIL import Image, ImageDraw, ImageFont
from pydantic import TypeAdapter

ROOT: Final = Path(__file__).resolve().parents[2]
REFERENCES: Final = (
    "art/style05/polish/items/stone_axe_try1.png",
    "art/style05/polish/items/ground_axe_try3.png",
    "art/style05/polish/items/berry_basket_try1.png",
    "art/style05/polish/items/clay_try3.png",
    "art/style05/polish/items/firewood_try3.png",
    "art/style05/env1/items/clay_pot_empty.png",
)
LIGHT: Final = (224, 216, 195, 255)
DARK: Final = (34, 37, 34, 255)
INK: Final = (31, 31, 26, 255)
SheetFamily = Literal["icon", "banner", "chapter"]


class SheetInput(TypedDict):
    """An unchanged file used to assemble a review board."""

    path: str
    sha256: str
    role: Literal["reference", "candidate"]


def main(arguments: list[str]) -> int:
    """Render the supplied nonempty snapshot, recording every input byte identity."""
    if len(arguments) < 3:
        _ = sys.stderr.write(
            "Usage: sheets.py icon|banner|chapter OUTPUT.png INPUT.png...\n"
        )
        return 2
    family = TypeAdapter[SheetFamily](SheetFamily).validate_python(arguments[0])
    destination = Path(arguments[1])
    match family:
        case "icon":
            cell_width, cell_height, columns = 228, 198, 5
            native_size, previews, band_height = (
                (64, 64),
                ((32, 32), (48, 48), (64, 64)),
                78,
            )
        case "banner":
            cell_width, cell_height, columns = 292, 238, 4
            native_size, previews, band_height = (264, 88), ((264, 88),), 96
        case "chapter":
            cell_width, cell_height, columns = 548, 584, 2
            native_size, previews, band_height = (520, 260), ((520, 260),), 268
        case unreachable:
            assert_never(unreachable)
    paths = tuple(Path(value) for value in arguments[2:])
    header = 218
    width = max(1100, cell_width * min(columns, len(paths)))
    height = header + cell_height * math.ceil(len(paths) / columns)
    sheet = Image.new("RGBA", (width, height), (246, 242, 230, 255))
    draw = ImageDraw.Draw(sheet)
    font = ImageFont.load_default(size=13)
    draw.text(
        (14, 10),
        f"A4 {family}: file-level candidates; visual review required",
        fill=INK,
        font=font,
    )
    draw.text(
        (14, 32),
        "Adopted WorldSim references (same master shown on light / dark):",
        fill=INK,
        font=font,
    )
    inputs: list[SheetInput] = []
    for i, relative in enumerate(REFERENCES):
        path = ROOT / relative
        with Image.open(path) as original:
            reference = original.convert("RGBA")
        reference.thumbnail((64, 64), Image.Resampling.LANCZOS)
        x = 14 + i * 178
        for column, background in enumerate((LIGHT, DARK)):
            draw.rectangle(
                (x + column * 76, 58, x + column * 76 + 72, 134), fill=background
            )
            sheet.alpha_composite(reference, (x + column * 76 + 4, 64))
        draw.text((x, 142), path.stem[:23], fill=INK, font=font)
        inputs.append(
            {
                "path": str(path),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "role": "reference",
            }
        )
    draw.text(
        (14, 183),
        "Icons: 32 / 48 / 64 px on each ground. Banners/chapters: native target canvas.",
        fill=INK,
        font=font,
    )
    for i, path in enumerate(paths):
        with Image.open(path) as original:
            source = original.convert("RGBA")
        if source.size != native_size:
            _ = sys.stderr.write(
                f"Wrong {family} canvas: {path}: {source.size}, expected {native_size}\n"
            )
            return 1
        x, y = (i % columns) * cell_width, header + (i // columns) * cell_height
        draw.text((x + 10, y + 5), path.stem[:36], fill=INK, font=font)
        for row, background in enumerate((LIGHT, DARK)):
            top = y + 28 + row * (band_height + 7)
            draw.rectangle(
                (x + 8, top, x + cell_width - 8, top + band_height), fill=background
            )
            left = x + 16
            for size in previews:
                shown = source.resize(size, Image.Resampling.LANCZOS)
                sheet.alpha_composite(
                    shown, (left, top + (band_height - shown.height) // 2)
                )
                left += size[0] + 16
        inputs.append(
            {
                "path": str(path),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "role": "candidate",
            }
        )
    destination.parent.mkdir(parents=True, exist_ok=True)
    sheet.convert("RGB").save(destination, optimize=True)
    ledger = {
        "family": family,
        "inputs": inputs,
        "canvas": [width, height],
        "sheet_sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
        "note": "Review composite only; source sprites are unchanged.",
    }
    _ = destination.with_suffix(".json").write_text(json.dumps(ledger, indent=2) + "\n")
    _ = sys.stdout.write(
        f"Wrote {destination}: {len(paths)} candidates plus {len(REFERENCES)} references.\n"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
