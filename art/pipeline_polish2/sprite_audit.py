# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_audit
"""Prepare 40 fixed source/mask pairs for independent visual audit."""
from __future__ import annotations

import json
from typing import Final

import numpy as np
from PIL import Image, ImageDraw

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_annotations import AnatomyAnnotations
from art.pipeline_polish2.sprite_freeze import FreezeGuard
from art.pipeline_polish2.sprite_landmarks import FrameGuide
from art.pipeline_polish2.sprite_masks import detect

DATA: Final = ROOT.parent / "artpolish2-data/masks/audit40"


def main() -> None:
    DATA.mkdir(parents=True, exist_ok=True)
    plan = FacePlan.read()
    freeze = FreezeGuard.read()
    annotations = AnatomyAnnotations.read(ROOT / "art/character/variant_anatomy.json")
    paths: list[str] = []
    for index in (0, 2, 4, 6):
        look = plan.owner_sample[index]
        sheet = Image.new("RGB", (2400, 650), "#decba9")
        draw = ImageDraw.Draw(sheet)
        for row, facing in enumerate(("S", "E")):
            for column, pose in enumerate(("idle", "walk_a", "work_a", "carry", "sleep")):
                path = ROOT / "art/character/route_i" / look / facing / f"{pose}.png"
                with Image.open(path) as opened:
                    source = opened.convert("RGBA")
                with Image.open(ROOT / "art/character/route_i" / look / facing / "idle.png") as ref:
                    reference = ref.convert("RGBA")
                masks = detect(source, FrameGuide(look.startswith("child_"), "_loincloth_" in look, pose, reference if pose not in ("idle", "sleep") else None, annotations.for_look(look) if facing == "S" and pose == "idle" else None, freeze.masks_for(look) if facing == "S" and pose == "idle" else None))
                rgba = np.array(source)
                rgba[masks.skin, :3] = (20, 220, 130)
                rgba[masks.hair, :3] = (235, 40, 200)
                overlay = Image.fromarray(rgba)
                silhouette_pixels = np.array(source)
                silhouette_pixels[masks.hair_silhouette, :3] = (80, 130, 255)
                silhouette_pixels[masks.hair_silhouette & masks.outline, :3] = (255, 220, 0)
                silhouette = Image.fromarray(silhouette_pixels)
                boxes = source.copy()
                box_draw = ImageDraw.Draw(boxes)
                for box in (masks.anatomy.face, *masks.anatomy.forearms):
                    box_draw.rectangle((box.left, box.top, box.right - 1, box.bottom - 1), outline=(0, 180, 240))
                visible = np.asarray(source)[:, :, 3] >= 16
                vy, vx = np.nonzero(visible)
                bounds = (int(np.min(vx)), int(np.min(vy)), int(np.max(vx)) + 1, int(np.max(vy)) + 1)
                left, top, right, bottom = bounds
                bounds = (max(0, left - 2), max(0, top - 2), min(source.width, right + 2), min(source.height, bottom + 2))
                for offset, frame in enumerate((source, overlay, silhouette, boxes)):
                    tile = frame.crop(bounds)
                    tile.thumbnail((112, 275), Image.Resampling.NEAREST)
                    scale = min(112 // tile.width, 275 // tile.height)
                    tile = tile.resize((tile.width * scale, tile.height * scale), Image.Resampling.NEAREST)
                    sheet.paste(tile, (column * 480 + offset * 118, row * 325 + 28), tile)
                draw.text((column * 480 + 4, row * 325 + 4), f"{facing}/{pose}: source / mask / hair ref / boxes", fill="black")
                cropped = [frame.crop(bounds) for frame in (source, overlay, silhouette, boxes)]
                width, height = cropped[0].size
                panel = Image.new("RGB", (width * 24, height * 6 + 22), "#decba9")
                panel_draw = ImageDraw.Draw(panel)
                for offset, (frame, label) in enumerate(zip(cropped, ("source", "skin / hair mask", "hair ref (yellow=ink)", "source boxes"), strict=True)):
                    large = frame.resize((width * 6, height * 6), Image.Resampling.NEAREST)
                    panel.paste(large, (offset * width * 6, 22), large)
                    panel_draw.text((offset * width * 6 + 2, 2), label, fill="black")
                (DATA / look).mkdir(exist_ok=True)
                panel.save(DATA / look / f"{facing}_{pose}.png")
                paths.append(str(path.relative_to(ROOT)))
        sheet.save(DATA / f"{look}.png")
    _ = (DATA / "sample.json").write_text(json.dumps({"rule": "first accepted look in each age-sex cohort; all ten frames", "frames": paths}, indent=2) + "\n")
    print(DATA)


if __name__ == "__main__":
    main()
