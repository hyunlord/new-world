# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python art/pipeline2/autotile.py RAW_DIR OUTPUT_DIR EARTH_PNG GRASS_PNG
"""Derive 4-bit wall connectivity from generated material, without drawing walls."""
from __future__ import annotations

import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image, ImageDraw

SIZE: Final = 64
BITS: Final = ((1, 90), (2, 0), (4, 270), (8, 180))


@dataclass(frozen=True, slots=True)
class Sources:
    straight: Image.Image
    corner: Image.Image
    end: Image.Image


@dataclass(frozen=True, slots=True)
class Recipe:
    mask: int
    core: str
    clockwise_arm_rotations: tuple[int, ...]


def recipe(mask: int) -> Recipe:
    """N=1, E=2, S=4, W=8; zero is an isolated terminal-material post."""
    if not 0 <= mask <= 15:
        message = f"Wall mask outside 0..15: {mask}"
        raise ValueError(message)
    adjacent = any(mask & pair == pair for pair in (3, 6, 12, 9))
    return Recipe(mask, "corner" if adjacent else "end",
                  tuple((-angle) % 360 for bit, angle in BITS if mask & bit))


def compose(mask: int, sources: Sources) -> Image.Image:
    """Crop half a straight, rotate its arms, then overlay the generated joint."""
    plan = recipe(mask)
    image = Image.new("RGBA", (SIZE, SIZE))
    horizontal = Image.new("RGBA", (SIZE, SIZE))
    horizontal.alpha_composite(sources.straight, (0, (SIZE - sources.straight.height) // 2))
    arm = Image.new("RGBA", (SIZE, SIZE))
    arm.alpha_composite(horizontal.crop((32, 0, 64, 64)), (32, 0))
    for angle in plan.clockwise_arm_rotations:
        image.alpha_composite(arm.rotate(-angle))
    core = sources.corner if plan.core == "corner" else sources.end
    image.alpha_composite(core, ((SIZE - core.width) // 2, (SIZE - core.height) // 2))
    return image


def extract(path: Path) -> Image.Image:
    """Remove magenta and mixed fringe alpha, preserving remaining RGB values."""
    with Image.open(path) as source:
        rgba = np.asarray(source.convert("RGBA")).copy()
    rgb = rgba[:, :, :3].astype(np.int16)
    key = (rgb[:, :, 0] - rgb[:, :, 1] > 20) & (rgb[:, :, 2] - rgb[:, :, 1] > 20)
    rgba[key] = 0
    cut = Image.fromarray(rgba)
    bounds = cut.getbbox()
    if bounds is None:
        message = f"Empty chroma cutout: {path}"
        raise ValueError(message)
    return cut.crop(bounds)


def build(raw: Path, output: Path, earth: Path, grass: Path) -> None:
    """Write candidates and exact source-crop recipes; acceptance stays pending."""
    output.mkdir(parents=True, exist_ok=True)
    straight = extract(raw / "straight_try1_raw.png")
    straight.thumbnail((64, 64), Image.Resampling.LANCZOS)
    corner = extract(raw / "corner_try1_raw.png")
    end = extract(raw / "end_try1_raw.png")
    # Generated corner elbow is bottom-left; terminal cap is bottom-center.
    corner_box = (0, corner.height - corner.width // 4, corner.width // 4, corner.height)
    end_box = (0, end.height - end.width, end.width, end.height)
    corner_core = corner.crop(corner_box).resize((16, 16), Image.Resampling.LANCZOS)
    end_core = end.crop(end_box).resize((16, 16), Image.Resampling.LANCZOS)
    sources = Sources(straight, corner_core, end_core)
    atlas = Image.new("RGBA", (256, 256))
    for mask in range(16):
        tile = compose(mask, sources)
        tile.save(output / f"wall_{mask:02d}.png")
        atlas.alpha_composite(tile, (mask % 4 * 64, mask // 4 * 64))
    atlas.save(output / "wall_atlas.png")
    with Image.open(earth) as source:
        floor = source.convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)
    with Image.open(grass) as source:
        meadow = source.convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)
    floor.save(output / "floor_00.png")
    edge = floor.copy()
    edge.alpha_composite(meadow.crop((0, 0, 64, 10)), (0, 0))
    edge.save(output / "floor_01.png")
    mapping = {
        "status": "PENDING", "bits": {"N": 1, "E": 2, "S": 4, "W": 8},
        "recipes": [asdict(recipe(mask)) for mask in range(16)],
        "operations": ["magenta alpha key R-G>20 and B-G>20; zero transparent RGB",
                       "crop alpha bounding box", "Lanczos downscale", "crop east half straight",
                       "rotate arms about tile center", "overlay 16px generated joint"],
        "corner_core_crop_after_alpha_bounds": corner_box,
        "end_core_crop_after_alpha_bounds": end_box,
        "sources": {str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                    for p in [raw / f"{name}_try1_raw.png" for name in ("straight", "corner", "end")] + [earth, grass]},
        "floor_operations": "floor00 earth Lanczos64; floor01 overlay top10px of grass Lanczos64",
        "limitations": ["Quarter rotations rotate baked light; not relit.",
                        "Generated straight uses upright posts; corner/end use long logs, a material geometry mismatch.",
                        "Core-to-arm junctions are overlap composites, requiring visual seam review.",
                        "Grass edge is a hard cropped strip, not a generated organic boundary."],
    }
    (output / "mapping.json").write_text(json.dumps(mapping, indent=2) + "\n")
    for mask in range(16):
        path = output / f"wall_{mask:02d}.png"
        sidecar = {"status": "PENDING", "working_size": [64, 64],
                   "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                   "recipe": asdict(recipe(mask)), "operations": mapping["operations"],
                   "sources": mapping["sources"], "model_judge": "PENDING",
                   "limitations": mapping["limitations"]}
        path.with_suffix(".json").write_text(json.dumps(sidecar, indent=2) + "\n")
    board = Image.new("RGBA", (384, 384), (211, 202, 178, 255))
    draw = ImageDraw.Draw(board)
    for mask in range(16):
        x, y = mask % 4 * 96, mask // 4 * 96
        board.alpha_composite(compose(mask, sources), (x + 16, y + 6))
        draw.text((x + 20, y + 76), f"{mask:02d}", fill=(20, 18, 15, 255))
    board.convert("RGB").save(output / "wall_board.png")


if __name__ == "__main__":
    if len(sys.argv) != 5:
        raise SystemExit("Usage: autotile.py RAW_DIR OUTPUT_DIR EARTH_PNG GRASS_PNG")
    build(*(Path(argument) for argument in sys.argv[1:]))
