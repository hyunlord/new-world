#!/usr/bin/env python3
"""Freeze a numbered, shuffled source-icon sheet; never claim a game capture."""
from __future__ import annotations

import argparse
import hashlib
import json
import random
from pathlib import Path

from PIL import Image, ImageDraw


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("icons", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--seed", type=int, required=True)
    args = parser.parse_args()
    sources = [args.icons / f"icon-{kind}.png" for kind in range(7)]
    images = []
    for path in sources:
        with Image.open(path) as source:
            icon = source.convert("RGBA")
        if icon.width > 22 or icon.height > 22 or min(icon.size) < 20:
            raise ValueError(f"Unexpected source badge dimensions: {path}: {icon.size}")
        images.append(icon.resize((icon.width * 4, icon.height * 4), Image.Resampling.NEAREST))
    order = list(range(7))
    random.Random(args.seed).shuffle(order)
    args.output.mkdir(parents=True, exist_ok=False)
    sheet = Image.new("RGBA", (784, 144), "#808080")
    draw = ImageDraw.Draw(sheet)
    for index, kind in enumerate(order):
        icon = images[kind]
        draw.text((index * 112 + 56, 16), str(index + 1), fill="white", anchor="mt", font_size=18)
        sheet.alpha_composite(icon, (index * 112 + (112 - icon.width) // 2, 44))
    target = args.output / "stage-a.png"
    sheet.convert("RGB").save(target)
    manifest = {
        "evidence_type": "source_texture_sheet_at_z4_scale_not_framebuffer",
        "scale": 4,
        "background": "#808080",
        "seed": args.seed,
        "private_answer_key": {str(i + 1): kind for i, kind in enumerate(order)},
        "source_sha256": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},
        "display_dimensions": {str(k): list(im.size) for k, im in enumerate(images)},
        "sheet_sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
        "reader_result": "PENDING",
    }
    (args.output / "private-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(target)


if __name__ == "__main__":
    main()
