# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_reference_apply --expected-freeze-sha <reviewed-current-sha>
"""Install authorized exact idle references, preserving the full previous assets."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
from PIL import Image
from pydantic import BaseModel, JsonValue

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_build import sha
from art.pipeline_polish2.sprite_colour import lab_triple, rgb_to_lab, shifted
from art.pipeline_polish2.sprite_freeze import FreezeGuard
from art.pipeline_polish2.sprite_records import StoredTransforms, read_mask
from art.pipeline_polish2.sprite_release import file_record


class ApplyArgs(argparse.Namespace):
    expected_freeze_sha: str = ""
    revision: str = "final_reference_v1"


class MaskManifest(BaseModel):
    frames: tuple[dict[str, JsonValue], ...]


class IdleManifest(BaseModel):
    looks: tuple[dict[str, JsonValue], ...]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument("--expected-freeze-sha", required=True)
    _ = parser.add_argument("--revision", default="final_reference_v1")
    args = parser.parse_args(namespace=ApplyArgs())
    freeze_path = ROOT / "art/character/variant_reference_freeze.json"
    if sha(freeze_path) != args.expected_freeze_sha or Path(args.revision).name != args.revision:
        raise ValueError("Previous freeze changed or revision is invalid")
    _ = FreezeGuard.read()
    plan = FacePlan.read()
    base = ROOT.parent / "artpolish2-data/masks"
    diagnostic, calibration = base / "diagnostic_v6", base / "calibration_v1"
    receipt = base / args.revision
    receipt.mkdir(exist_ok=False)
    archive = receipt / "previous"
    paths = list((ROOT / "art/character/variant_masks").rglob("*"))
    for look in plan.looks:
        for variant in look.variants:
            paths.extend((ROOT / "art/character/route_i" / look.look / variant.id).rglob("*.png"))
    paths.extend((freeze_path, ROOT / "art/character/variant_transforms.json"))
    previous: list[JsonValue] = []
    for path in paths:
        if path.is_file():
            target = archive / path.relative_to(ROOT)
            target.parent.mkdir(parents=True, exist_ok=True)
            _ = target.write_bytes(path.read_bytes())
            previous.append(file_record(path, ROOT))
    _ = (receipt / "previous_files.json").write_text(json.dumps(previous, indent=2) + "\n")
    stored = StoredTransforms.model_validate_json((calibration / "summary.json").read_bytes())
    deltas = {(entry.look, entry.variant): entry for entry in stored.transforms}
    frozen: list[dict[str, JsonValue]] = []
    transforms: list[dict[str, JsonValue]] = []
    for look in plan.looks:
        candidate = calibration / look.look
        source_path = ROOT / "art/character/route_i" / look.look / "S/idle.png"
        if sha(source_path) != sha(candidate / "source.png"):
            raise ValueError(f"Original changed: {look.look}")
        with Image.open(source_path) as opened:
            source = opened.convert("RGBA")
        skin, hair = read_mask(candidate / "skin.png"), read_mask(candidate / "hair.png")
        mask_root = ROOT / "art/character/variant_masks" / look.look / "S"
        for region in ("skin", "hair"):
            path = candidate / f"{region}.png"
            if sha(path) != sha(diagnostic / look.look / f"{region}.png"):
                raise ValueError(f"Reviewed mask mismatch: {look.look}/{region}")
            _ = (mask_root / f"idle_{region}.png").write_bytes(path.read_bytes())
        variants: list[JsonValue] = []
        for variant in look.variants:
            delta = deltas[(look.look, variant.id)]
            reproduction = shifted(shifted(source, skin, delta.skin_delta_lab), hair, delta.hair_delta_lab)
            with Image.open(candidate / f"{variant.id}.png") as opened:
                expected = opened.convert("RGBA")
            if not np.array_equal(np.asarray(reproduction), np.asarray(expected)):
                raise ValueError(f"Calibrated bake mismatch: {look.look}/{variant.id}")
            target = ROOT / "art/character/route_i" / look.look / variant.id / "S/idle.png"
            _ = target.write_bytes((candidate / f"{variant.id}.png").read_bytes())
            variants.append({"variant": variant.id, **file_record(target, ROOT)})
            transforms.append({"look": look.look, "variant": variant.id, "skin_delta_lab": list(delta.skin_delta_lab),
                "hair_delta_lab": list(delta.hair_delta_lab),
                "source_skin_lab": list(lab_triple(rgb_to_lab(np.array(variant.skin_rgb, dtype=np.float64)) - np.array(delta.skin_delta_lab))),
                "source_hair_lab": list(lab_triple(rgb_to_lab(np.array(variant.hair_rgb, dtype=np.float64)) - np.array(delta.hair_delta_lab))),
                "target_skin_rgb": list(variant.skin_rgb), "target_hair_rgb": list(variant.hair_rgb), "frame_count": 10,
                "mask_pattern": f"art/character/variant_masks/{look.look}/{{facing}}/{{pose}}_{{region}}.png",
                "baked_pattern": f"art/character/route_i/{look.look}/{variant.id}/{{facing}}/{{pose}}.png"})
        frozen.append({"look": look.look, "source": file_record(source_path, ROOT),
                       "skin_mask": file_record(mask_root / "idle_skin.png", ROOT),
                       "hair_mask": file_record(mask_root / "idle_hair.png", ROOT), "variants": variants})
    transform_path = ROOT / "art/character/variant_transforms.json"
    _ = transform_path.write_text(json.dumps({"schema_version": 2, "colour_space": "CIELAB D65", "selection_rule": plan.rule,
        "fallbacks": plan.fallbacks, "alpha_preserved": True, "outline_L_max": 18, "outline_alpha_max": 127,
        "source_plan_sha256": sha(ROOT / "art/character/FACE_VARIANTS.json"), "transforms": transforms}, indent=2) + "\n")
    reviews = [ROOT / "docs/v8/artpolish2" / name for name in ("MASK_OWNER_V6_REVIEW.json", "IDLE_MASK_REVIEW.json")]
    records = [ROOT / "docs/v8/artpolish2" / name for name in ("OWNER_SOURCE_ENVELOPES.json", "IDLE_SOURCE_ENVELOPES.json")]
    manifest = {"schema_version": 2, "status": "ALL34_FINAL_REFERENCES_FROZEN", "looks": frozen,
        "protected_records": [file_record(path, ROOT) for path in (transform_path, *records)],
        "semantic_reviews": [file_record(path, ROOT) for path in reviews], "calibration": str(calibration),
        "calibration_summary_sha256": sha(calibration / "summary.json"), "archive": str(archive),
        "literal_coverage": {"face_pass": 29, "both_forearms_pass": 27, "total": 34, "status": "FAIL_RETAINED"},
        "gamut_clipping_variant_pixel_totals": {"old_skin": 1426, "new_skin": 2738, "old_hair": 28, "new_hair": 85},
        "nonidle_status_at_freeze": "REPAIR_AND_REBAKE_PENDING_WITH_THESE_IMMUTABLE_TRANSFORMS"}
    _ = freeze_path.write_text(json.dumps(manifest, indent=2) + "\n")
    masks_path = ROOT / "art/character/variant_masks/MANIFEST.json"
    old_rows = MaskManifest.model_validate_json(masks_path.read_bytes()).frames
    new_rows = {row["look"]: row for row in IdleManifest.model_validate_json((diagnostic / "summary.json").read_bytes()).looks if isinstance(row["look"], str)}
    rows = [new_rows[str(row["look"])] | {"facing": "S", "pose": "idle", "semantic_status": "INDEPENDENTLY_CLEARED"}
            if row["facing"] == "S" and row["pose"] == "idle" else row for row in old_rows]
    _ = masks_path.write_text(json.dumps({"schema_version": 2, "nonidle_status": "REPAIR_AND_REBAKE_PENDING", "frames": rows}, indent=2) + "\n")
    guard = FreezeGuard.read()
    _ = (receipt / "final_freeze.json").write_bytes(freeze_path.read_bytes())
    print(f"Frozen 34 looks /204 calibrated idle refs; verified {len(guard.hashes)} protected files")
    print(f"{freeze_path} SHA256 {sha(freeze_path)}")


if __name__ == "__main__":
    main()
