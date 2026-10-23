# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run --with pillow --with numpy --with scikit-image python -m art.pipeline_env1.built_boards
"""Build geometry-only comparison boards and freeze judge input identities."""
from __future__ import annotations

import json
from pathlib import Path

from PIL import Image, ImageDraw

from art.pipeline_env1.built_process import DOC, OUT, RAW, digest
from art.pipeline_env1.items_records import JsonRecord

GROUPS = {
    "buildings_a": ["wall_log_straight", "wall_log_corner", "wall_stone_straight", "wall_stone_corner"],
    "buildings_b": ["door_log_closed", "door_log_open", "door_stone_closed", "door_stone_open"],
    "buildings_c": ["drying_rack_empty", "drying_rack_hide", "pottery_bench", "grinding_stone"],
    "ground_a": ["floor_packed_earth", "floor_dug_rock", "claybank_full", "claybank_depleted"],
    "ground_b": ["ford", "hill_edge", "dug_face", "snow_overlay"],
}


def main() -> None:
    manifest = JsonRecord.read(DOC / "built_assets.json")
    rows = {row.text("id"): row for row in manifest.records("assets")}
    for group, names in GROUPS.items():
        path = RAW / f"judge_{group}_candidates.png"
        if path.exists():
            continue
        board = Image.new("RGB", (1200, 1200), (218, 212, 196))
        draw = ImageDraw.Draw(board)
        for index, name in enumerate(names):
            tile = Image.open(rows[name].text("raw")).convert("RGBA")
            tile.thumbnail((580, 550), Image.Resampling.LANCZOS)
            x, y = index % 2 * 600, index // 2 * 600
            board.paste(tile, (x + (600 - tile.width) // 2, y + (550 - tile.height) // 2), tile)
            draw.text((x + 20, y + 565), name, fill="black")
        board.save(path, optimize=True)
        frozen = {"board": str(path), "sha256": digest(path), "candidates": [
            {"id": name, "raw": rows[name].text("raw"), "sha256": rows[name].text("raw_sha256")} for name in names]}
        _ = path.with_suffix(".json").write_text(json.dumps(frozen, indent=2) + "\n")
    repeat = Image.new("RGB", (1152, 832), (82, 80, 70))
    draw = ImageDraw.Draw(repeat)
    names = ["floor_packed_earth", "floor_dug_rock", "ford", "hill_edge", "dug_face", "snow_overlay"]
    for index, name in enumerate(names):
        tile = Image.open(rows[name].text("path")).convert("RGBA")
        patch = Image.new("RGBA", (192, 192), (82, 80, 70, 255))
        for y in range(3):
            for x in range(3):
                patch.alpha_composite(tile, (x * 64, y * 64))
        x, y = index % 3 * 384, index // 3 * 416
        repeat.paste(patch.resize((384, 384), Image.Resampling.NEAREST), (x, y + 24))
        draw.text((x + 10, y + 5), name + " 3x3 at2x", fill="white")
    repeat.save(OUT / "repeat_board.png", optimize=True)
    scale = Image.new("RGBA", (960, 400), (82, 80, 70, 255))
    draw = ImageDraw.Draw(scale)
    for name, pos in (("hut", (10, 30)), ("knapping_spot", (220, 30)), ("woodworking_bench", (300, 30))):
        tile = Image.open(Path("art/style05/env/buildings") / f"{name}.png").convert("RGBA")
        scale.alpha_composite(tile, pos)
        draw.text((pos[0], pos[1] - 18), name, fill="white")
    for material, y in (("log", 250), ("stone", 320)):
        for x in range(6):
            tile = Image.open(OUT / f"wall_{material}_10.png").convert("RGBA")
            scale.alpha_composite(tile, (x * 64 + 10, y))
        draw.text((420, y + 24), material + " wall: six64px tiles,16px band", fill="white")
    scale.convert("RGB").save(OUT / "reuse_scale_board.png", optimize=True)


if __name__ == "__main__":
    main()
