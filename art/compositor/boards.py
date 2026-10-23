# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow"]
# ///
# How to run: python3 -m art.compositor.boards.
"""Owner comparison boards assembled from immutable full-resolution renders."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw, ImageFont

REPO: Final = Path(__file__).resolve().parents[2]
DOCS: Final = REPO / "docs/v8/artenv1"
FONT: Final = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 20)
SMALL: Final = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 16)


def board(destination: Path, panels: tuple[tuple[str, Path], ...]) -> None:
    image = Image.new("RGB", (800 * len(panels), 520), (21, 26, 24))
    draw = ImageDraw.Draw(image)
    sources: list[dict[str, str]] = []
    for index, (label, path) in enumerate(panels):
        with Image.open(path) as source:
            if source.size != (1600, 900):
                message = f"Board source is not the full game window: {path}"
                raise ValueError(message)
            image.paste(source.convert("RGB").resize((800, 450), Image.Resampling.LANCZOS), (index * 800, 40))
        draw.text((index * 800 + 16, 10), label, font=FONT, fill=(236, 230, 209))
        sources.append({"path": str(path.relative_to(REPO)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
    draw.text((16, 497), "Owner review candidate. After panels add declared west buildings, fixtures, animals and actions to the native scene. No owner approval claimed.",
              font=SMALL, fill=(197, 202, 188))
    destination.parent.mkdir(parents=True, exist_ok=True)
    image.save(destination, optimize=True, compress_level=9)
    if destination.stat().st_size > 1_500_000:
        message = f"Owner board exceeds file cap: {destination}"
        raise ValueError(message)
    _ = destination.with_suffix(".json").write_text(json.dumps({"sources": sources, "operation": "uniform 0.5 Lanczos panel resize, no quantization",
                                                          "owner_approval": "NOT_REQUESTED"}, indent=2) + "\n")


def main() -> None:
    renders = DOCS / "renders"
    reference = DOCS / "reference"
    board(REPO / "art/boards/env1_owner_board.png", (
        ("Current renderer | day 6 noon | 1x", reference / "current_day6_noon_1x.png"),
        ("05 | day 6 noon | 1x", renders / "scene_day6_noon_day_1x_figures.png"),
        ("05 | day 6 noon | 1/2x + LOD", renders / "scene_day6_noon_day_half_lod.png"),
        ("05 | same source scene | night transform", renders / "scene_day6_noon_night_1x_figures.png"),
    ))
    board(REPO / "art/boards/env1_winter_overlay_board.png", (
        ("Current renderer | first winter noon | 1x", reference / "current_winter_noon_1x.png"),
        ("05 | winter | 1x", renders / "scene_winter_noon_winter_1x_figures.png"),
        ("05 | winter | 1/2x + LOD", renders / "scene_winter_noon_winter_half_lod.png"),
        ("05 | declared M3-M5 overlay | day", renders / "scene_winter_noon_day_1x_figures.png"),
    ))


if __name__ == "__main__":
    main()
