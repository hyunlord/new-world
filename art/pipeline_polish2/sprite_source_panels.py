# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_source_panels
"""Untinted source-only pose contacts and coordinate grids for region reviewers."""
from __future__ import annotations

import hashlib
import json

import numpy as np
from PIL import Image, ImageDraw
from pydantic import JsonValue

from art.compositor.face_variants import ROOT, FacePlan


def main() -> None:
    folder = ROOT.parent / "artpolish2-data/masks/source_poses"
    folder.mkdir(parents=True, exist_ok=True)
    rows: list[dict[str, JsonValue]] = []
    for look in FacePlan.read().looks:
        target = folder / look.look
        target.mkdir(exist_ok=True)
        sheet = Image.new("RGB", (1500, 660), "#decba9")
        draw = ImageDraw.Draw(sheet)
        for row, facing in enumerate(("S", "E")):
            for column, pose in enumerate(("idle", "walk_a", "work_a", "carry", "sleep")):
                path = ROOT / "art/character/route_i" / look.look / facing / f"{pose}.png"
                with Image.open(path) as opened:
                    source = opened.convert("RGBA")
                yy, xx = np.nonzero(np.asarray(source)[:, :, 3] >= 16)
                left, top = max(0, int(np.min(xx)) - 2), max(0, int(np.min(yy)) - 2)
                right, bottom = min(source.width, int(np.max(xx)) + 3), min(source.height, int(np.max(yy)) + 3)
                crop = source.crop((left, top, right, bottom))
                scale = min(4, 290 // crop.width, 300 // crop.height)
                tile = crop.resize((crop.width * scale, crop.height * scale), Image.Resampling.NEAREST)
                sheet.paste(tile, (column * 300 + 5, row * 330 + 24), tile)
                draw.text((column * 300 + 5, row * 330 + 5), f"{facing}/{pose} source only", fill="black")
                zoom, margin = 12, 26
                grid = Image.new("RGB", (crop.width * zoom + margin, crop.height * zoom + margin), "#decba9")
                large = crop.resize((crop.width * zoom, crop.height * zoom), Image.Resampling.NEAREST)
                grid.paste(large, (margin, margin), large)
                grid_draw = ImageDraw.Draw(grid)
                for x in range(left, right):
                    gx = (x - left) * zoom + margin
                    grid_draw.line((gx, margin, gx, grid.height), fill="#8e826f", width=1)
                    if x % 2 == 0:
                        grid_draw.text((gx, 4), str(x), fill="black")
                for y in range(top, bottom):
                    gy = (y - top) * zoom + margin
                    grid_draw.line((margin, gy, grid.width, gy), fill="#8e826f", width=1)
                    if y % 2 == 0:
                        grid_draw.text((2, gy), str(y), fill="black")
                grid.save(target / f"{facing}_{pose}_grid.png")
                rows.append({"look": look.look, "facing": facing, "pose": pose,
                             "source": str(path.relative_to(ROOT)), "source_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                             "grid": str(target / f"{facing}_{pose}_grid.png")})
        sheet.save(target / "contact.png")
    _ = (folder / "sources.json").write_text(json.dumps({"frames": rows}, indent=2) + "\n")
    print(folder)


if __name__ == "__main__":
    main()
