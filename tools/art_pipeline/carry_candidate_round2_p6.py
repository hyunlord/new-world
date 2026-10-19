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


def candidate_parts(frame: Frame) -> tuple[Image.Image, Image.Image, tuple[int, int]]:
    """Extract a source-area patch, enlarge, outline and attach at hip level."""
    body = frame.image()
    opaque = body.getbbox()
    if opaque is None:
        return body, Image.new("RGBA", (9, 7)), (0, 0)
    left, top, right, bottom = opaque
    height = bottom - top
    wood = frame.activity == "carry_wood"
    area = (left, top + round(height * .20), left + max(3, (right - left) // 3), top + round(height * .42)) if wood else (left + (right - left) // 3, top + round(height * .39), right - (right - left) // 4, top + round(height * .61))
    original = body.copy()
    middle = top + height // 2
    occupied = [x for x in range(body.width) if body.getpixel((x, middle))[3] > 0]
    torso_left, torso_right = min(occupied), max(occupied) + 1
    if wood:
        # Move the source's arm/load side pixels down to hip level. Both hands
        # remain the frame's own pixels; the torso/head stay at their original position.
        for y in range(top + round(height * .10), top + round(height * .5)):
            for x in range(body.width):
                if x < torso_left or x >= torso_right:
                    pixel = original.getpixel((x, y))
                    body.putpixel((x, y), (0, 0, 0, 0))
                    ny = y + round(height * .28)
                    nx = max(0, min(body.width - 1, x + (2 if x < torso_left else -2)))
                    if ny < body.height and pixel[3]:
                        body.putpixel((nx, ny), pixel)
    else:
        # A narrowly bounded chest replacement removes the old red food patch.
        # Sample a real skin pixel from the original face, never paint a new color.
        face = [original.getpixel((x, y)) for y in range(top + round(height * .2), top + round(height * .34))
                for x in range(torso_left, torso_right)
                if original.getpixel((x, y))[0] > 120 and original.getpixel((x, y))[1] > 65
                and original.getpixel((x, y))[0] > original.getpixel((x, y))[1] * 1.2]
        skin = sorted(face, key=lambda rgba: rgba[0])[len(face) // 2] if face else original.getpixel((torso_left + 2, middle))
        for y in range(top + round(height * .36), top + round(height * .60)):
            for x in range(torso_left + 3, torso_right - 3):
                pixel = body.getpixel((x, y))
                reddish = pixel[0] > pixel[1] * 1.5 and pixel[1] < 110
                inner_outline = max(pixel[:3]) < 60 and torso_left + 4 <= x < torso_right - 4
                if pixel[3] and (reddish or inner_outline):
                    body.putpixel((x, y), skin)
    pixels = original.crop(area).resize((7, 5), Image.Resampling.NEAREST)
    prop = Image.new("RGBA", (9, 7), (0, 0, 0, 0))
    for y in range(7):
        inset = 1 if y >= 5 else 0
        for x in range(inset, 9 - inset):
            border = y in (0, 6) or x in (inset, 8 - inset)
            color = OUTLINE if border else pixels.getpixel((min(6, x - 1), min(4, y - 1)))
            prop.putpixel((x, y), color if color[3] else OUTLINE)
    anchor = (torso_right - 3, top + round(height * .53))
    return body, prop, anchor


def candidate(frame: Frame) -> tuple[Image.Image, Image.Image, tuple[int, int]]:
    """Compose the exact reviewed body and prop for the board."""
    body, prop, anchor = candidate_parts(frame)
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
        draw.text((10, 10), "ROUND 2 | native / 2x / 4x | chest cleared / hip attachment", fill="white")
        for index, frame in enumerate(frames):
            picture, prop, anchor = candidate(frame)
            top = index * 160 + 40
            draw.text((10, top), frame.sprite_id, fill="white")
            for scale, center in ((1, 235), (2, 345), (4, 525)):
                resized = picture.resize((picture.width * scale, picture.height * scale), Image.Resampling.NEAREST)
                canvas.paste(resized, (center - resized.width // 2, top + 126 - resized.height), resized)
            records.append({"sprite_id": frame.sprite_id, "group": group, "anchor": anchor,
                            "size": prop.size, "source": "own_frame_load_area_crop",
                            "limitation": "Own-source chest skin replacement or relocated shoulder-side arm/load pixels; body hands retained",
                            "enabled": False})
        canvas.save(OUTPUT / "boards" / f"candidate_round2_{group}.png")
    (OUTPUT / "candidate_round2_manifest.json").write_text(json.dumps(records, indent=2) + "\n")
    print(f"candidate boards: {len(groups)} groups; all disabled pending review")


if __name__ == "__main__":
    main()
