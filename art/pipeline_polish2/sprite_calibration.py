# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_calibration --diagnostic diagnostic_v6 --revision calibration_v1
"""Compare frozen old anchors against corrected idle-mask means externally."""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw
from pydantic import JsonValue

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_build import sha
from art.pipeline_polish2.sprite_colour import (
    BoolPixels,
    lab_to_linear_rgb,
    lab_triple,
    mean_colour,
    rgb_to_lab,
    shifted,
)
from art.pipeline_polish2.sprite_diagnostic import snapshot
from art.pipeline_polish2.sprite_records import StoredTransforms, read_mask


class CalibrationArgs(argparse.Namespace):
    diagnostic: str = "diagnostic_v6"
    revision: str = "calibration_v1"


def error_record(source: Image.Image, tinted: Image.Image, mask: BoolPixels,
                 delta: tuple[float, float, float], target: tuple[int, int, int]) -> dict[str, JsonValue]:
    desired = rgb_to_lab(np.asarray(source)[:, :, :3][mask].astype(np.float64)) + np.array(delta)
    linear = lab_to_linear_rgb(desired)
    clipped = np.empty((len(desired),), dtype=np.bool_)
    _ = np.any((linear < 0) | (linear > 1), axis=-1, out=clipped)
    actual = mean_colour(rgb_to_lab(np.asarray(tinted)[:, :, :3][mask].astype(np.float64)))
    target_lab = rgb_to_lab(np.array(target, dtype=np.float64))
    difference = actual - target_lab
    squared_error: np.float64 = np.sum(difference ** 2)
    error = math.sqrt(float(squared_error))
    return {"selected_pixels": int(np.count_nonzero(mask)), "out_of_gamut_pixels": int(np.count_nonzero(clipped)),
            "actual_mean_lab": list(lab_triple(actual)), "target_lab": list(lab_triple(target_lab)), "mean_target_delta_e76": error}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument("--diagnostic", default="diagnostic_v6")
    _ = parser.add_argument("--revision", default="calibration_v1")
    args = parser.parse_args(namespace=CalibrationArgs())
    if any(Path(name).name != name for name in (args.diagnostic, args.revision)):
        raise ValueError("Names must each be a single directory component")
    base = ROOT.parent / "artpolish2-data/masks"
    diagnostic = base / args.diagnostic
    folder = base / args.revision
    folder.mkdir(exist_ok=False)
    inputs = folder / "inputs"
    inputs.mkdir()
    plan = FacePlan.model_validate_json(snapshot(diagnostic / "inputs/FACE_VARIANTS.json", inputs))
    stored = StoredTransforms.model_validate_json(snapshot(diagnostic / "inputs/variant_transforms.json", inputs))
    _ = snapshot(diagnostic / "summary.json", inputs)
    _ = snapshot(Path(__file__), inputs)
    _ = snapshot(ROOT / "art/pipeline_polish2/sprite_colour.py", inputs)
    old = {(entry.look, entry.variant): entry for entry in stored.transforms}
    transforms: list[dict[str, JsonValue]] = []
    anchors: list[dict[str, JsonValue]] = []
    for look in plan.looks:
        candidate = diagnostic / look.look
        target = folder / look.look
        target.mkdir()
        for filename in ("source.png", "skin.png", "hair.png"):
            _ = snapshot(candidate / filename, target)
        with Image.open(target / "source.png") as opened:
            source = opened.convert("RGBA")
        skin, hair = read_mask(target / "skin.png"), read_mask(target / "hair.png")
        lab = rgb_to_lab(np.asarray(source)[:, :, :3].astype(np.float64))
        skin_anchor, hair_anchor = mean_colour(lab[skin]), mean_colour(lab[hair])
        first = look.variants[0]
        old_skin = rgb_to_lab(np.array(first.skin_rgb, dtype=np.float64)) - np.array(old[(look.look, first.id)].skin_delta_lab)
        old_hair = rgb_to_lab(np.array(first.hair_rgb, dtype=np.float64)) - np.array(old[(look.look, first.id)].hair_delta_lab)
        anchors.append({"look": look.look, "old_skin_lab": list(lab_triple(old_skin)), "new_skin_lab": list(lab_triple(skin_anchor)),
                        "old_hair_lab": list(lab_triple(old_hair)), "new_hair_lab": list(lab_triple(hair_anchor)),
                        "source_sha256": sha(target / "source.png"), "skin_sha256": sha(target / "skin.png"), "hair_sha256": sha(target / "hair.png")})
        sheet = Image.new("RGB", (6 * 100, 2 * 136), "#decba9")
        draw = ImageDraw.Draw(sheet)
        for column, variant in enumerate(look.variants):
            previous = old[(look.look, variant.id)]
            skin_delta = lab_triple(rgb_to_lab(np.array(variant.skin_rgb, dtype=np.float64)) - skin_anchor)
            hair_delta = lab_triple(rgb_to_lab(np.array(variant.hair_rgb, dtype=np.float64)) - hair_anchor)
            old_tint = shifted(shifted(source, skin, previous.skin_delta_lab), hair, previous.hair_delta_lab)
            new_tint = shifted(shifted(source, skin, skin_delta), hair, hair_delta)
            old_tint.save(target / f"old_{variant.id}.png")
            new_tint.save(target / f"{variant.id}.png")
            for row, frame in enumerate((old_tint, new_tint)):
                sheet.paste(frame, (column * 100 + 10, row * 136 + 22), frame)
                draw.text((column * 100 + 3, row * 136 + 4), f"{variant.id} {'old' if row == 0 else 'new'}", fill="black")
            transforms.append({"look": look.look, "variant": variant.id, "source_skin_lab": list(lab_triple(skin_anchor)),
                               "source_hair_lab": list(lab_triple(hair_anchor)), "skin_delta_lab": list(skin_delta), "hair_delta_lab": list(hair_delta),
                               "old_skin_error": error_record(source, old_tint, skin, previous.skin_delta_lab, variant.skin_rgb),
                               "new_skin_error": error_record(source, new_tint, skin, skin_delta, variant.skin_rgb),
                               "old_hair_error": error_record(source, old_tint, hair, previous.hair_delta_lab, variant.hair_rgb),
                               "new_hair_error": error_record(source, new_tint, hair, hair_delta, variant.hair_rgb), "candidate_sha256": sha(target / f"{variant.id}.png")})
        sheet.save(target / "native.png")
        sheet.resize((1200, 544), Image.Resampling.NEAREST).save(target / "2x.png")
    summary = {"status": "ANCHOR_COMPARISON_ONLY_NOT_INTEGRATED", "diagnostic": str(diagnostic),
               "diagnostic_summary_sha256": sha(diagnostic / "summary.json"), "anchors": anchors, "transforms": transforms}
    _ = (folder / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(f"Compared {len(anchors)} anchors and {len(transforms)} transforms: {folder}")


if __name__ == "__main__":
    main()
