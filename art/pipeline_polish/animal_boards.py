# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: PYTHONPATH=. python -m art.pipeline_polish.animal_boards
"""Build native-size owner evidence and independent no-key animal packets."""
from __future__ import annotations

import random
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw
from pydantic import JsonValue

from art.compositor.tone import cap_ground, person_limit
from art.pipeline_env1.animal_normalize import digest
from art.pipeline_polish.animal_process import DATA, OUT, ROOT, SPECS, write_json

ROUND: Final = 1


@dataclass(frozen=True, slots=True)
class Frame:
    species: str
    facing: str
    pose: str
    path: Path


def frames() -> tuple[Frame, ...]:
    result: list[Frame] = []
    for spec in SPECS:
        for facing in ("S", "E"):
            for pose in ("stand", "walk", "run", spec.special_pose, "dead"):
                result.append(Frame(spec.species, facing, pose, OUT / spec.species / f"try{spec.attempt}" / facing / f"{pose}.png"))
    return tuple(result)


def ground(name: str) -> Image.Image:
    """Use compositor-wide calm-ground parameters, with no animal recolour."""
    limit = replace(person_limit(ROOT / "art/style05/anchors/person.png"), mean_max=.32, std_ratio=.25, saturation_max=.22)
    with Image.open(ROOT / f"art/style05/ground_v2/{name}.png") as source:
        result = cap_ground(source, limit)
    if not result.passed:
        message = f"Calm ground failed for {name}"
        raise ValueError(message)
    return result.image.resize((64, 64), Image.Resampling.LANCZOS).convert("RGBA")


def tile_background(size: tuple[int, int], tile: Image.Image, scale: int) -> Image.Image:
    pattern = tile.resize((64 * scale, 64 * scale), Image.Resampling.NEAREST)
    canvas = Image.new("RGBA", size)
    for y in range(0, size[1], pattern.height):
        for x in range(0, size[0], pattern.width):
            canvas.alpha_composite(pattern, (x, y))
    return canvas


def place(canvas: Image.Image, frame: Frame, center_x: int, baseline_y: int, scale: int) -> None:
    with Image.open(frame.path) as source:
        sprite = source.convert("RGBA").resize((source.width * scale, source.height * scale), Image.Resampling.NEAREST)
    baseline = 56 if frame.pose == "dead" else 72
    canvas.alpha_composite(sprite, (center_x - 48 * scale, baseline_y - baseline * scale))


def owner_board(rows: tuple[Frame, ...]) -> None:
    width, height = 1100, 2580
    canvas = Image.new("RGBA", (width, height), "#272923")
    draw = ImageDraw.Draw(canvas)
    draw.text((14, 10), "ART-POLISH animal candidates - owner decision pending; S = front 3/4, 35 degrees viewer-left", fill="#eee4ce")
    y = 36
    for terrain in ("meadow", "forest_floor"):
        for scale in (1, 2):
            band_height = 116 if scale == 1 else 202
            for start in range(0, 20, 5):
                group = rows[start:start + 5]
                band = tile_background((width, band_height), ground(terrain), scale)
                label = ImageDraw.Draw(band)
                for column, frame in enumerate(group):
                    x = column * 220
                    label.text((x + 8, 6), f"{frame.species} {frame.facing}/{frame.pose} {scale}x", fill="#f1e7ce")
                    place(band, frame, x + 110, band_height - 16, scale)
                canvas.alpha_composite(band, (0, y))
                y += band_height
            draw.text((10, y - 14), terrain, fill="#fff1c9")
    # Full native displays require 16 bands; do not downsample the owner board.
    path = ROOT / "art/boards/animals_owner.png"
    path.parent.mkdir(parents=True, exist_ok=True)
    canvas.convert("RGB").save(path, optimize=True)


def blind_packets(rows: tuple[Frame, ...]) -> None:
    key: list[JsonValue] = []
    packet_dir = DATA / f"round{ROUND}"
    packet_dir.mkdir(parents=True, exist_ok=True)
    for reviewer in ("a", "b"):
        for scale in (1, 2):
            order = list(rows)
            random.Random(20261024 + ROUND * 100 + scale + (0 if reviewer == "a" else 17)).shuffle(order)
            cell_w, cell_h = (128, 112) if scale == 1 else (212, 188)
            canvas = tile_background((cell_w * 5, cell_h * 4), ground("meadow"), scale)
            draw = ImageDraw.Draw(canvas)
            for index, frame in enumerate(order):
                x, y = index % 5 * cell_w, index // 5 * cell_h
                identifier = f"{reviewer.upper()}{scale}-{index + 1:02d}"
                draw.text((x + 7, y + 5), identifier, fill="#f7efd9")
                place(canvas, frame, x + cell_w // 2, y + cell_h - 10, scale)
                key.append({"id": identifier, "reviewer": reviewer, "scale": scale, "species": frame.species,
                            "pose": frame.pose, "facing": frame.facing, "path": str(frame.path.relative_to(ROOT)), "sha256": digest(frame.path)})
            canvas.convert("RGB").save(packet_dir / f"reviewer_{reviewer}_{scale}x.png", optimize=True)
    write_json(packet_dir / "answer_key.json", key)
    packet_text = "Inspect only your two supplied PNGs at their native image dimensions. For each identifier, give the animal species and visible action/pose, one answer each, no alternatives. Do not inspect source files, answer keys, prior reviews, filenames or repository. Report uncertainty honestly. Return JSON {responses:[{id,species,pose}]}.\n"
    _ = (packet_dir / "REVIEWER_INSTRUCTIONS.txt").write_text(packet_text)


if __name__ == "__main__":
    selected = frames()
    owner_board(selected)
    blind_packets(selected)
    print(f"Wrote 20-frame owner board and round{ROUND} blind packets to {DATA}")
