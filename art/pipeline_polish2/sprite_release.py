# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_release --revision release_v1
"""Stage a complete external tint release from immutable diagnostic evidence."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
from PIL import Image
from pydantic import JsonValue

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_build import sha
from art.pipeline_polish2.sprite_colour import lab_triple, rgb_to_lab, shifted
from art.pipeline_polish2.sprite_diagnostic import snapshot
from art.pipeline_polish2.sprite_records import (
    FrameEvidence,
    StoredTransforms,
    field,
    read_mask,
)


class ReleaseArgs(argparse.Namespace):
    frames: str = "frames_diagnostic_v1"
    calibration: str = "calibration_v1"
    revision: str = "release_v1"


def file_record(path: Path, root: Path) -> dict[str, JsonValue]:
    return {"path": str(path.relative_to(root)), "sha256": sha(path)}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument("--frames", default="frames_diagnostic_v1")
    _ = parser.add_argument("--calibration", default="calibration_v1")
    _ = parser.add_argument("--revision", default="release_v1")
    args = parser.parse_args(namespace=ReleaseArgs())
    if any(Path(name).name != name for name in (args.frames, args.calibration, args.revision)):
        raise ValueError("Names must be single directory components")
    base = ROOT.parent / "artpolish2-data/masks"
    frames, calibration, folder = base / args.frames, base / args.calibration, base / args.revision
    folder.mkdir(exist_ok=False)
    inputs = folder / "inputs"
    inputs.mkdir()
    for kind, source in (("frames", frames), ("calibration", calibration)):
        (inputs / kind).mkdir()
        _ = snapshot(source / "summary.json", inputs / kind)
    _ = snapshot(Path(__file__), inputs)
    plan = FacePlan.model_validate_json(snapshot(ROOT / "art/character/FACE_VARIANTS.json", inputs))
    evidence = FrameEvidence.model_validate_json((inputs / "frames/summary.json").read_bytes())
    transforms = StoredTransforms.model_validate_json((inputs / "calibration/summary.json").read_bytes())
    deltas = {(entry.look, entry.variant): entry for entry in transforms.transforms}
    if len(deltas) != 204 or len(evidence.frames) != 340:
        raise ValueError("Complete 204 transforms and 340 frames required")
    output = folder / "art/character"
    output.mkdir(parents=True)
    rows: list[dict[str, JsonValue]] = []
    frozen: list[dict[str, JsonValue]] = []
    for entry in evidence.frames:
        row = entry.copy()
        look, facing, pose = (field(row, key) for key in ("look", "facing", "pose"))
        source_path = ROOT / "art/character/route_i" / look / facing / f"{pose}.png"
        source_candidate = frames / look / facing / pose
        if sha(source_path) != field(row, "source_sha256"):
            raise ValueError(f"Source changed: {source_path}")
        with Image.open(source_path) as opened:
            source = opened.convert("RGBA")
        mask_folder = output / "variant_masks" / look / facing
        mask_folder.mkdir(parents=True, exist_ok=True)
        for region in ("skin", "hair"):
            path = source_candidate / f"{region}.png"
            if sha(path) != field(row, f"{region}_sha256"):
                raise ValueError(f"Candidate mask changed: {path}")
            _ = (mask_folder / f"{pose}_{region}.png").write_bytes(path.read_bytes())
            if facing == "S" and pose == "idle" and sha(path) != sha(calibration / look / f"{region}.png"):
                raise ValueError(f"Calibration and final idle masks disagree: {look}/{region}")
        skin, hair = read_mask(mask_folder / f"{pose}_skin.png"), read_mask(mask_folder / f"{pose}_hair.png")
        original = np.asarray(source)
        outline = (rgb_to_lab(original[:, :, :3].astype(np.float64))[:, :, 0] <= 18) | (original[:, :, 3] < 128)
        if np.any(skin & hair) or np.any((skin | hair) & outline):
            raise ValueError(f"Mask overlap or protected outline selection: {look}/{facing}/{pose}")
        baked: dict[str, JsonValue] = {}
        variant_files: list[JsonValue] = []
        for variant in plan.for_look(look).variants:
            delta = deltas[(look, variant.id)]
            tinted = shifted(shifted(source, skin, delta.skin_delta_lab), hair, delta.hair_delta_lab)
            result = np.asarray(tinted)
            if not np.array_equal(original[:, :, 3], result[:, :, 3]) or not np.array_equal(original[~(skin | hair)], result[~(skin | hair)]):
                raise ValueError(f"Tint changed protected bytes: {look}/{variant.id}/{facing}/{pose}")
            destination = output / "route_i" / look / variant.id / facing / f"{pose}.png"
            destination.parent.mkdir(parents=True, exist_ok=True)
            tinted.save(destination)
            baked[variant.id] = sha(destination)
            variant_files.append({"variant": variant.id, **file_record(destination, folder)})
            if facing == "S" and pose == "idle" and sha(destination) != sha(calibration / look / f"{variant.id}.png"):
                raise ValueError(f"Calibration tint differs: {look}/{variant.id}")
        if facing == "S" and pose == "idle":
            frozen.append({"look": look, "source": file_record(source_path, ROOT),
                           "skin_mask": file_record(mask_folder / "idle_skin.png", folder),
                           "hair_mask": file_record(mask_folder / "idle_hair.png", folder), "variants": variant_files})
        row.update({"source": str(source_path.relative_to(ROOT)), "baked_sha256": baked,
                    "calibration_summary_sha256": sha(calibration / "summary.json")})
        rows.append(row)
    transform_rows: list[dict[str, JsonValue]] = []
    for look in plan.looks:
        for variant in look.variants:
            delta = deltas[(look.look, variant.id)]
            transform_rows.append({"look": look.look, "variant": variant.id,
                "skin_delta_lab": list(delta.skin_delta_lab), "hair_delta_lab": list(delta.hair_delta_lab),
                "source_skin_lab": list(lab_triple(rgb_to_lab(np.array(variant.skin_rgb, dtype=np.float64)) - np.array(delta.skin_delta_lab))),
                "source_hair_lab": list(lab_triple(rgb_to_lab(np.array(variant.hair_rgb, dtype=np.float64)) - np.array(delta.hair_delta_lab))),
                "target_skin_rgb": list(variant.skin_rgb), "target_hair_rgb": list(variant.hair_rgb), "frame_count": 10,
                "mask_pattern": f"art/character/variant_masks/{look.look}/{{facing}}/{{pose}}_{{region}}.png",
                "baked_pattern": f"art/character/route_i/{look.look}/{variant.id}/{{facing}}/{{pose}}.png"})
    _ = (output / "variant_transforms.json").write_text(json.dumps({"schema_version": 2, "colour_space": "CIELAB D65", "selection_rule": plan.rule,
            "fallbacks": plan.fallbacks, "alpha_preserved": True, "outline_L_max": 18, "outline_alpha_max": 127,
            "source_plan_sha256": sha(ROOT / "art/character/FACE_VARIANTS.json"), "transforms": transform_rows}, indent=2) + "\n")
    _ = (output / "variant_masks/MANIFEST.json").write_text(json.dumps({"schema_version": 2, "source_reports": evidence.source_reports,
            "semantic_status": "AWAITING_FULL_FRAME_REVIEW", "frames": rows}, separators=(",", ":")) + "\n")
    _ = (output / "variant_reference_freeze.json").write_text(json.dumps({"schema_version": 2, "status": "CANDIDATE_NOT_APPLIED",
            "strict_coverage_status": "FAIL_RETAINED", "source_diagnostic": str(frames), "calibration": str(calibration), "looks": frozen}, indent=2) + "\n")
    print(f"Staged 680 masks + 2040 bakes + 204 transforms; live files unchanged: {folder}")


if __name__ == "__main__":
    main()
