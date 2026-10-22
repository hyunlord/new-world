# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: uv run --with pillow --with numpy python -m art.pipeline2.autotile_v2 RAW OUTPUT EARTH MEADOW
"""Derive connected palisades from a generated strip and generated 2x2 elbow."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from art.pipeline2.autotile import BITS, SIZE, Sources, extract, recipe

STRAIGHT_CROP = (374, 510, 1094, 698)
CORNER_CROP = (-13, 712, 587, 1312)


def compose(mask: int, sources: Sources) -> Image.Image:
    """Keep the generated elbow intact; straight masks have no overlaid joint."""
    _ = recipe(mask)
    if mask in (5, 10):
        return sources.straight.copy() if mask == 10 else sources.straight.rotate(90)
    if mask == 0:
        result = Image.new("RGBA", (SIZE, SIZE))
        result.alpha_composite(sources.end, (24, 24))
        return result
    result = Image.new("RGBA", (SIZE, SIZE))
    result.alpha_composite(sources.end, (24, 24))
    remaining = mask
    for pair, rotation in ((3, 0), (6, -90), (12, 180), (9, 90)):
        if mask & pair == pair:
            result = sources.corner.rotate(rotation)
            remaining &= ~pair
            break
    arm = Image.new("RGBA", (SIZE, SIZE))
    arm.alpha_composite(sources.straight.crop((40, 0, 64, 64)), (40, 0))
    for bit, angle in BITS:
        if remaining & bit:
            result.alpha_composite(arm.rotate(angle))
    return result


def build(raw: Path, output: Path, earth: Path, meadow: Path) -> None:
    output.mkdir(parents=True, exist_ok=True)
    strip = extract(raw / "straight_try2_raw.png", crop_bounds=False)
    segment = strip.crop(STRAIGHT_CROP).resize((64, 16), Image.Resampling.LANCZOS)
    straight = Image.new("RGBA", (64, 64))
    straight.alpha_composite(segment, (0, 24))
    corner = extract(raw / "corner_try2_raw.png", crop_bounds=False).crop(CORNER_CROP)
    corner = corner.resize((64, 64), Image.Resampling.LANCZOS)
    sources = Sources(straight, corner, segment.crop((24, 0, 40, 16)))
    tiles = [compose(mask, sources) for mask in range(16)]
    source_paths = [raw / "straight_try2_raw.png", raw / "corner_try2_raw.png", earth, meadow,
                    raw / "floor_edge_try1_raw.png", raw / "floor_edge_working/candidate.png"]
    hashes = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in source_paths}
    operations = ["magenta alpha key only", "crop generated strip; Lanczos 64x16",
                  "crop generated 2x2 corner lower-left elbow; Lanczos 64x64",
                  "quarter rotations; partial generated strip overlay only for T/X and terminals",
                  "no painted outline; no quantization; no floor mirror/blend"]
    observations: list[dict[str, int | bool]] = []
    atlas = Image.new("RGBA", (256, 256))
    for mask, tile in enumerate(tiles):
        alpha = np.asarray(tile)[:, :, 3]
        edges = (alpha[0], alpha[:, -1], alpha[-1], alpha[:, 0])
        observed = sum(bit for bit, edge in zip((1, 2, 4, 8), edges, strict=True) if np.max(edge) > 128)
        observations.append({"mask": mask, "observed": observed, "pass": mask == observed})
        tile.save(output / f"wall_{mask:02d}.png")
        atlas.alpha_composite(tile, (mask % 4 * 64, mask // 4 * 64))
        sidecar = {"status": "reviewed_candidate", "mask": mask, "sources": hashes,
                   "operations": operations, "sha256": hashlib.sha256((output / f"wall_{mask:02d}.png").read_bytes()).hexdigest()}
        _ = (output / f"wall_{mask:02d}.json").write_text(json.dumps(sidecar, indent=2) + "\n")
    atlas.save(output / "wall_atlas.png")
    with Image.open(earth) as source:
        floor = source.convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)
    floor.save(output / "floor_00.png")
    with Image.open(raw / "floor_edge_working/candidate.png") as source:
        floor_edge = source.convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)
    floor_edge.save(output / "floor_01.png")
    mapping = {"version": 2, "bits": {"N": 1, "E": 2, "S": 4, "W": 8},
               "sources": hashes, "straight_raw_crop": STRAIGHT_CROP, "corner_raw_crop": CORNER_CROP,
               "operations": operations, "floor01": "generated earth-meadow transition; global ground saturation cap; no synthetic grass strip",
               "topology": observations, "visual_verdict": "see AUTOTILE_REVIEW_V2.md; topology is not visual acceptance"}
    _ = (output / "mapping.json").write_text(json.dumps(mapping, indent=2) + "\n")
    board = Image.new("RGBA", (1024, 1024), (216, 207, 184, 255))
    draw = ImageDraw.Draw(board)
    draw.text((10, 8), "TRIAL2b generated palisade - masks 1x / actual 10x8 topology 1x", fill="black")
    for mask, tile in enumerate(tiles):
        x, y = mask % 8 * 96 + 10, mask // 8 * 96 + 32
        board.alpha_composite(tile, (x, y))
        draw.text((x, y + 67), f"{mask:02}", fill="black")
    cells = {(x, y) for x in range(10) for y in range(8)
             if (x in (0, 9) or y in (0, 7)) and (x, y) not in ((4, 7), (5, 7))}
    cells.update((4, y) for y in range(2, 7))
    with Image.open(meadow) as source:
        grass = source.convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)
    enclosure = Image.new("RGBA", (640, 512))
    for y in range(8):
        for x in range(10):
            ground = grass if x in (0, 9) or y in (0, 7) else floor
            if y == 6 and 0 < x < 9:
                ground = floor_edge.transpose(Image.Transpose.FLIP_TOP_BOTTOM)
            enclosure.alpha_composite(ground, (x * 64, y * 64))
            if (x, y) in cells:
                mask = sum(bit for bit, dx, dy in ((1, 0, -1), (2, 1, 0), (4, 0, 1), (8, -1, 0)) if (x + dx, y + dy) in cells)
                enclosure.alpha_composite(tiles[mask], (x * 64, y * 64))
    enclosure.save(output / "enclosure_offline.png")
    board.alpha_composite(enclosure, (10, 260))
    draw.text((10, 790), "Join detail 2x: NW06/10/05    terminal04/05/01    T07/15 (offline composites)", fill="black")
    for index, mask in enumerate((6, 10, 5, 4, 5, 1, 7, 15)):
        board.alpha_composite(tiles[mask].resize((128, 128), Image.Resampling.NEAREST), (index * 128, 820))
    board.convert("RGB").save(output / "wall_board.png")
    if not all(item["pass"] for item in observations):
        raise ValueError(f"Generated alpha connectivity failed: {observations}")


if __name__ == "__main__":
    if len(sys.argv) != 5:
        raise SystemExit("Usage: autotile_v2.py RAW OUTPUT EARTH MEADOW")
    build(*(Path(argument) for argument in sys.argv[1:]))
