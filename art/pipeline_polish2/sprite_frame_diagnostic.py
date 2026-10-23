# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_frame_diagnostic --revision frames_diagnostic_v1
"""Immutable all-frame candidates with source geometry and actual stored masks."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw
from pydantic import JsonValue

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_annotations import AnatomyAnnotations
from art.pipeline_polish2.sprite_build import record, sha
from art.pipeline_polish2.sprite_colour import shifted
from art.pipeline_polish2.sprite_diagnostic import (
    mask_panel,
    overlay,
    snapshot,
)
from art.pipeline_polish2.sprite_freeze import FreezeGuard
from art.pipeline_polish2.sprite_landmarks import FrameGuide
from art.pipeline_polish2.sprite_masks import Masks, detect
from art.pipeline_polish2.sprite_records import StoredTransforms, read_mask
from art.pipeline_polish2.sprite_sources import load_envelopes


class FrameArgs(argparse.Namespace):
    revision: str = "frames_diagnostic_v1"
    scope: str = "all"


def audit_panel(path: Path, source: Image.Image, masks: Masks, tint: Image.Image) -> None:
    reference = np.array(source)
    reference[masks.hair_silhouette, :3] = (20, 155, 240)
    reference[masks.hair, :3] = (235, 40, 200)
    frames = (source, overlay(source, (masks.skin, masks.hair)), Image.fromarray(reference), tint)
    sheet = Image.new("RGB", (source.width * 16, source.height * 4 + 24), "#decba9")
    draw = ImageDraw.Draw(sheet)
    for column, (frame, title) in enumerate(zip(frames, ("source", "skin / hair masks", "hair reference blue / selected pink", "v4 stored transform"), strict=True)):
        tile = frame.resize((source.width * 4, source.height * 4), Image.Resampling.NEAREST)
        sheet.paste(tile, (column * source.width * 4, 24), tile)
        draw.text((column * source.width * 4 + 2, 4), title, fill="black")
    sheet.save(path)


def contact(folder: Path, frames: list[tuple[str, str]]) -> None:
    sheet = Image.new("RGB", (5 * 680, 2 * 250), "#decba9")
    draw = ImageDraw.Draw(sheet)
    for index, (facing, pose) in enumerate(frames):
        x, y = index % 5 * 680, index // 5 * 250
        target = folder / facing / pose
        with Image.open(target / "source.png") as opened:
            source = opened.convert("RGBA")
        skin, hair = read_mask(target / "skin.png"), read_mask(target / "hair.png")
        with Image.open(target / "v4.png") as opened:
            tint = opened.convert("RGBA")
        for column, frame in enumerate((source, overlay(source, (skin, hair)), tint)):
            tile = frame.resize((frame.width * 2, frame.height * 2), Image.Resampling.NEAREST)
            sheet.paste(tile, (x + column * 225, y + 22), tile)
        draw.text((x + 3, y + 3), f"{facing}/{pose}: source | masks | v4 stored transform", fill="black")
    sheet.save(folder / "contact.png")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument("--revision", default="frames_diagnostic_v1")
    _ = parser.add_argument("--scope", choices=("all", "nonidle"), default="all")
    args = parser.parse_args(namespace=FrameArgs())
    if Path(args.revision).name != args.revision:
        raise ValueError("Revision must be one directory name")
    folder = ROOT.parent / "artpolish2-data/masks" / args.revision
    folder.mkdir(parents=True, exist_ok=False)
    inputs = folder / "inputs"
    inputs.mkdir()
    plan = FacePlan.model_validate_json(snapshot(ROOT / "art/character/FACE_VARIANTS.json", inputs))
    annotations = AnatomyAnnotations.model_validate_json(snapshot(ROOT / "art/character/variant_anatomy.json", inputs))
    stored = StoredTransforms.model_validate_json(snapshot(ROOT / "art/character/variant_transforms.json", inputs))
    freeze = FreezeGuard.read()
    for code in sorted((ROOT / "art/pipeline_polish2").glob("sprite_*.py")):
        _ = snapshot(code, inputs)
    index = load_envelopes(inputs)
    transforms = {(entry.look, entry.variant): entry for entry in stored.transforms}
    rows: list[dict[str, JsonValue]] = []
    audit: list[dict[str, JsonValue]] = []
    audit_looks = {next(look.look for look in plan.looks if look.look.startswith(prefix))
                   for prefix in ("adult_F_", "adult_M_", "child_F_", "child_M_")}
    for look in plan.looks:
        completed: list[tuple[str, str]] = []
        for facing in ("S", "E"):
            with Image.open(ROOT / "art/character/route_i" / look.look / facing / "idle.png") as opened:
                idle = opened.convert("RGBA")
            for pose in ("idle", "walk_a", "work_a", "carry", "sleep"):
                if args.scope == "nonidle" and facing == "S" and pose == "idle":
                    continue
                key = (look.look, facing, pose)
                envelope = index.frames[key]
                source_path = ROOT / "art/character/route_i" / look.look / facing / f"{pose}.png"
                with Image.open(source_path) as opened:
                    source = opened.convert("RGBA")
                guide = FrameGuide(look.look.startswith("child_"), "_loincloth_" in look.look,
                                   envelope.pose, idle if pose not in ("idle", "sleep") else None,
                                   annotations.for_look(look.look) if facing == "S" and pose == "idle" else None,
                                   freeze.masks_for(look.look) if facing == "S" and pose == "idle" else None,
                                   envelope=envelope, refine_skin=True, refine_hair=True)
                masks = detect(source, guide)
                target = folder / look.look / facing / pose
                target.mkdir(parents=True)
                _ = (target / "source.png").write_bytes(source_path.read_bytes())
                mask_root = ROOT / "art/character/variant_masks" / look.look / facing
                current_skin, current_hair = read_mask(mask_root / f"{pose}_skin.png"), read_mask(mask_root / f"{pose}_hair.png")
                for name, pixels in (("skin", masks.skin), ("hair", masks.hair), ("hair_reference", masks.hair_silhouette),
                                     ("outline", masks.outline), ("current_skin", current_skin), ("current_hair", current_hair)):
                    Image.fromarray(pixels.astype(np.uint8) * 255).save(target / f"{name}.png")
                mask_panel(target / "masks.png", (source, overlay(source, (current_skin, current_hair)), overlay(source, (masks.skin, masks.hair))))
                baked: dict[str, JsonValue] = {}
                original = np.asarray(source)
                preservation = True
                for variant in look.variants:
                    transform = transforms[(look.look, variant.id)]
                    candidate = shifted(shifted(source, masks.skin, transform.skin_delta_lab), masks.hair, transform.hair_delta_lab)
                    candidate.save(target / f"{variant.id}.png")
                    result = np.asarray(candidate)
                    preservation &= bool(np.array_equal(original[:, :, 3], result[:, :, 3])
                                         and np.array_equal(original[~(masks.skin | masks.hair)], result[~(masks.skin | masks.hair)])
                                         and np.array_equal(original[masks.outline], result[masks.outline]))
                    baked[variant.id] = sha(target / f"{variant.id}.png")
                    if variant.id == "v4":
                        audit_panel(target / "audit.png", source, masks, candidate)
                if not preservation or np.any(masks.skin & masks.hair):
                    raise ValueError(f"Mechanical preservation failed: {key}")
                evidence = record(masks)
                evidence.update({"look": look.look, "facing": facing, "pose": pose,
                                 "source_sha256": envelope.source_sha256, "source_report": index.frame_reports[key],
                                 "source_report_sha256": index.report_hashes[index.frame_reports[key]],
                                 "source_envelope_sha256": hashlib.sha256(envelope.model_dump_json().encode()).hexdigest(),
                                 "skin_box_gate_applicable": facing == "S" and pose == "idle",
                                 "mechanical_preservation_pass": preservation, "semantic_status": "AWAITING_INDEPENDENT_REVIEW",
                                 "path": str(target), "skin_sha256": sha(target / "skin.png"), "hair_sha256": sha(target / "hair.png"),
                                 "hair_reference_sha256": sha(target / "hair_reference.png"), "audit_panel_sha256": sha(target / "audit.png"),
                                 "candidate_sha256": baked})
                rows.append(evidence)
                if look.look in audit_looks:
                    audit.append(evidence)
                completed.append((facing, pose))
        contact(folder / look.look, completed)
        print(f"diagnostic {look.look}: {len(completed)} frames", flush=True)
    summary = {"schema_version": 1, "status": "DIAGNOSTIC_ONLY_NOT_INTEGRATED", "scope": args.scope,
               "anchors": "Frozen variant_transforms.json values unchanged across all10 frames; S/idle masks fixed to final reference freeze",
               "hair_reference": {"status": "INDEPENDENT_SOURCE_COLOUR_CANDIDATE_NOT_GROUND_TRUTH",
                                  "Lab_L_max": 52, "red_minus_green_exclusive_max": 55, "alpha_min": 16,
                                  "geometry": "source scalp/backfall/beard minus declared face/neck/garment/prop supports",
                                  "selected_mask_or_Lloyd_labels_used": False},
               "source_reports": index.report_hashes, "frames": rows}
    _ = (folder / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    _ = (folder / "audit40.json").write_text(json.dumps({"status": "AWAITING_INDEPENDENT_REVIEW", "frames": audit}, indent=2) + "\n")
    print(f"Locked {len(rows)} frames; fixed audit {len(audit)}; {folder}")


if __name__ == "__main__":
    main()
