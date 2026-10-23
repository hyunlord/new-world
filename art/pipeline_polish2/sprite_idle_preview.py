# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_idle_preview
"""Render stored south-idle masks beside untouched source and reviewed boxes."""
from __future__ import annotations

import numpy as np
from PIL import Image, ImageDraw

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_annotations import AnatomyAnnotations


def main() -> None:
    folder = ROOT.parent / "artpolish2-data/masks/all_idle"
    folder.mkdir(parents=True, exist_ok=True)
    annotations = AnatomyAnnotations.read(ROOT / "art/character/variant_anatomy.json")
    for look in FacePlan.read().looks:
        with Image.open(ROOT / "art/character/route_i" / look.look / "S/idle.png") as opened:
            source = opened.convert("RGBA")
        pixels = np.array(source)
        for region, colour in (("skin", (20, 220, 130)), ("hair", (235, 40, 200))):
            path = ROOT / "art/character/variant_masks" / look.look / "S" / f"idle_{region}.png"
            with Image.open(path) as opened:
                mask = np.asarray(opened) > 0
            pixels[mask, :3] = colour
        overlay = Image.fromarray(pixels)
        boxes = source.copy()
        draw = ImageDraw.Draw(boxes)
        annotation = annotations.for_look(look.look)
        for left, top, right, bottom in (annotation.face, *annotation.forearms):
            draw.rectangle((left, top, right - 1, bottom - 1), outline=(0, 180, 240))
        yy, xx = np.nonzero(np.asarray(source)[:, :, 3] >= 16)
        bounds = (int(np.min(xx)) - 2, int(np.min(yy)) - 2, int(np.max(xx)) + 3, int(np.max(yy)) + 3)
        width, height = bounds[2] - bounds[0], bounds[3] - bounds[1]
        panel = Image.new("RGB", (width * 18, height * 6 + 22), "#decba9")
        draw = ImageDraw.Draw(panel)
        frames = (source, overlay, boxes)
        labels = ("source", "stored skin/hair masks", "reviewed source boxes")
        for index, (frame, label) in enumerate(zip(frames, labels, strict=True)):
            tile = frame.crop(bounds).resize((width * 6, height * 6), Image.Resampling.NEAREST)
            panel.paste(tile, (index * width * 6, 22), tile)
            draw.text((index * width * 6 + 2, 2), label, fill="black")
        panel.save(folder / f"{look.look}.png")
    print(folder)


if __name__ == "__main__":
    main()
