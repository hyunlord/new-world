#!/usr/bin/env python3
"""Contact sheet of game screenshots for the owner (V8-PLAY goal 1.2).

Each shot is scaled to 960x540 (nearest neighbour, so pixel art stays sharp at
the 2:1 step from 1920x1080), with its file name and a one-line caption in a
strip under it, two shots per row. The caption font is the game's Galmuri11 at
24 px (its 12-px design grid), so Korean captions render as in the game.

usage: contact_sheet.py --manifest shots.json --out sheet.png [--shots DIR] [--columns N]
  shots.json: [{"file": "m1_01_start_2x.png", "caption": "..."}, ...]
  (file paths are relative to --shots, or to the manifest's folder by default; an entry with an empty "file"
  leaves its cell blank, so rows can hold before / after pairs with a third column used only by some rows)

The output is a palette PNG (adaptive, 256 colours) so a 12-shot sheet stays
well under the 4 MB limit; exits 1 when it would not.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

TILE_W, TILE_H = 960, 540
STRIP_H = 64
GAP = 8
COLUMNS = 2
BACKGROUND = (24, 20, 16)
NAME_COLOUR = (216, 178, 110)
TEXT_COLOUR = (236, 230, 216)
MAX_BYTES = 4 * 1024 * 1024
FONT = Path(__file__).resolve().parents[2] / "assets/fonts/galmuri/Galmuri11.ttf"


def sheet_size(count: int, columns: int = COLUMNS) -> tuple[int, int]:
    """Pixel size of a sheet holding `count` cells in `columns` columns."""
    rows = max(1, (count + columns - 1) // columns)
    return (columns * TILE_W + (columns + 1) * GAP, rows * (TILE_H + STRIP_H) + (rows + 1) * GAP)


def _font(size: int) -> ImageFont.ImageFont:
    try:
        return ImageFont.truetype(str(FONT), size)
    except OSError:
        return ImageFont.load_default()


def build(items: list[tuple[Path | None, str]], columns: int = COLUMNS) -> Image.Image:
    """Lay out `items` (image path or None for a blank cell, caption) and return the RGB sheet."""
    if not any(path is not None for path, _ in items):
        raise ValueError("no shots")
    sheet = Image.new("RGB", sheet_size(len(items), columns), BACKGROUND)
    draw = ImageDraw.Draw(sheet)
    font = _font(24)
    for index, (path, caption) in enumerate(items):
        if path is None:
            continue
        col, row = index % columns, index // columns
        x = GAP + col * (TILE_W + GAP)
        y = GAP + row * (TILE_H + STRIP_H + GAP)
        with Image.open(path) as shot:
            sheet.paste(shot.convert("RGB").resize((TILE_W, TILE_H), Image.Resampling.NEAREST), (x, y))
        draw.text((x + 4, y + TILE_H + 4), path.name, font=font, fill=NAME_COLOUR)
        draw.text((x + 4, y + TILE_H + 32), caption, font=font, fill=TEXT_COLOUR)
    return sheet


def save(sheet: Image.Image, out: Path) -> int:
    """Save `sheet` as an adaptive-palette PNG; return its size in bytes."""
    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.quantize(colors=256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save(out, optimize=True)
    return out.stat().st_size


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--shots", type=Path)
    parser.add_argument("--columns", type=int, default=COLUMNS)
    args = parser.parse_args(argv)
    base = args.shots or args.manifest.parent
    entries = json.loads(args.manifest.read_text(encoding="utf-8"))
    items = [((base / e["file"]) if e.get("file") else None, str(e.get("caption", ""))) for e in entries]
    missing = [str(p) for p, _ in items if p is not None and not p.is_file()]
    if missing:
        print("contact_sheet: missing " + ", ".join(missing), file=sys.stderr)
        return 1
    size = save(build(items, args.columns), args.out)
    w, h = sheet_size(len(items), args.columns)
    print(f"contact_sheet: {args.out} {w}x{h} {size} bytes, {sum(p is not None for p, _ in items)} shots")
    if size > MAX_BYTES:
        print(f"contact_sheet: {size} bytes exceeds {MAX_BYTES}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
