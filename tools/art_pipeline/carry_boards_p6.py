# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/carry_boards_p6.py
"""Review all installed carry and eat frames at native, 2x and 4x scales."""
from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

ROOT: Final = Path(__file__).resolve().parents[2]
OUTPUT: Final = ROOT / "docs/art/production6/carry/boards"


@dataclass(frozen=True, slots=True)
class Frame:
    """Installed atlas frame and its activity group."""
    sprite_id: str
    body: str
    activity: str
    view: str
    page: Path
    box: tuple[int, int, int, int]

    def image(self) -> Image.Image:
        """Return an independent RGBA crop of original installed pixels."""
        with Image.open(self.page) as atlas:
            return atlas.convert("RGBA").crop(self.box)


def installed_frames() -> list[Frame]:
    """Read the installed atlas, omitting explicitly suppressed frames."""
    document = json.loads((ROOT / "assets/art/atlas/world_atlas.json").read_text())
    result: list[Frame] = []
    for sprite_id, entry in sorted(document["sprites"].items()):
        if entry.get("activity") not in ("carry", "carry_wood", "eat"):
            continue
        if sprite_id in document.get("suppressed", {}) or not entry.get("drawable", True):
            continue
        x, y, width, height = entry["rect"]
        result.append(Frame(sprite_id, entry["body"], entry["activity"], entry["view"],
                            ROOT / entry["page"].removeprefix("res://"),
                            (x, y, x + width, y + height)))
    return result


def board(frames: list[Frame], destination: Path) -> None:
    """Produce original-frame review boards with nearest-neighbor scale only."""
    canvas = Image.new("RGB", (660, 160 * len(frames) + 36), "#73816e")
    draw = ImageDraw.Draw(canvas)
    draw.text((10, 10), "INSTALLED SOURCE | native / 2x / 4x | no new compositing", fill="white")
    for index, frame in enumerate(frames):
        top = index * 160 + 40
        draw.text((10, top), frame.sprite_id, fill="white")
        picture = frame.image()
        for scale, center in ((1, 235), (2, 345), (4, 505)):
            resized = picture.resize((picture.width * scale, picture.height * scale), Image.Resampling.NEAREST)
            canvas.paste(resized, (center - resized.width // 2, top + 126 - resized.height), resized)
    canvas.save(destination)


def main() -> None:
    """Write one board per body/activity/view group."""
    OUTPUT.mkdir(parents=True, exist_ok=True)
    groups: dict[str, list[Frame]] = {}
    for frame in installed_frames():
        groups.setdefault(f"{frame.body}_{frame.activity}_{frame.view}", []).append(frame)
    for group, frames in groups.items():
        board(frames, OUTPUT / f"source_{group}.png")
    print(f"source boards: {len(groups)} groups, {sum(map(len, groups.values()))} frames")


if __name__ == "__main__":
    main()
