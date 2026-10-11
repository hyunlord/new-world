# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/carry_candidate_p6.py
"""Preview attached load candidates made only from each frame's own load-area pixels."""
from __future__ import annotations

import json
from typing import Final

from PIL import Image, ImageDraw

from carry_boards_p6 import ROOT, Frame, installed_frames

OUTPUT: Final = ROOT / "docs/art/production6/carry"
OUTLINE: Final = (25, 19, 13, 255)


def candidate(frame: Frame) -> tuple[Image.Image, Image.Image, tuple[int, int]]:
    """Extract a source-area patch, enlarge, outline and attach at hip level."""
    body = frame.image()
    opaque = body.getbbox()
    if opaque is None:
        return body, Image.new("RGBA", (9, 7)), (0, 0)
    left, top, right, bottom = opaque
    height = bottom - top
    wood = frame.activity == "carry_wood"
    area = (left, top + round(height * .20), left + max(3, (right - left) // 3), top + round(height * .42)) if wood else (left + (right - left) // 3, top + round(height * .39), right - (right - left) // 4, top + round(height * .61))
    pixels = body.crop(area).resize((7, 5), Image.Resampling.NEAREST)
    prop = Image.new("RGBA", (9, 7), OUTLINE)
    prop.paste(pixels, (1, 1), pixels)
    anchor = (right - 4, top + round(height * .53))
    composite = Image.new("RGBA", (body.width + 9, body.height))
    composite.alpha_composite(body)
    composite.alpha_composite(prop, anchor)
    return composite, prop, anchor


def main() -> None:
    """Write only review evidence; runtime installation is a separate review gate."""
    groups: dict[str, list[Frame]] = {}
    for frame in installed_frames():
        if frame.activity != "eat" and frame.body != "child":
            groups.setdefault(f"{frame.body}_{frame.activity}_{frame.view}", []).append(frame)
    records = []
    for group, frames in groups.items():
        canvas = Image.new("RGB", (680, 160 * len(frames) + 36), "#73816e")
        draw = ImageDraw.Draw(canvas)
        draw.text((10, 10), "CANDIDATE ONLY | native / 2x / 4x | attached source-pixel load", fill="white")
        for index, frame in enumerate(frames):
            picture, prop, anchor = candidate(frame)
            top = index * 160 + 40
            draw.text((10, top), frame.sprite_id, fill="white")
            for scale, center in ((1, 235), (2, 345), (4, 525)):
                resized = picture.resize((picture.width * scale, picture.height * scale), Image.Resampling.NEAREST)
                canvas.paste(resized, (center - resized.width // 2, top + 126 - resized.height), resized)
            records.append({"sprite_id": frame.sprite_id, "group": group, "anchor": anchor,
                            "size": prop.size, "source": "own_frame_load_area_crop",
                            "limitation": "Original chest/shoulder load retained; own-load-area crop can include adjacent body colors",
                            "enabled": False})
        canvas.save(OUTPUT / "boards" / f"candidate_{group}.png")
    (OUTPUT / "candidate_manifest.json").write_text(json.dumps(records, indent=2) + "\n")
    print(f"candidate boards: {len(groups)} groups; all disabled pending review")


if __name__ == "__main__":
    main()
