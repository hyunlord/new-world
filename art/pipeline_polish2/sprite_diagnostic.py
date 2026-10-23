# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_diagnostic
"""External-only envelope candidates, preserving every existing Lab transform."""
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
from art.pipeline_polish2.sprite_build import record
from art.pipeline_polish2.sprite_colour import BoolPixels, shifted
from art.pipeline_polish2.sprite_landmarks import FrameGuide
from art.pipeline_polish2.sprite_masks import detect
from art.pipeline_polish2.sprite_records import StoredTransforms
from art.pipeline_polish2.sprite_regions import EnvelopeReport, SourceEnvelope


class DiagnosticArgs(argparse.Namespace):
    revision: str | None = None


def stored_mask(look: str, region: str) -> BoolPixels:
    with Image.open(ROOT / "art/character/variant_masks" / look / "S" / f"idle_{region}.png") as opened:
        return np.asarray(opened) > 0


def overlay(source: Image.Image, masks: tuple[BoolPixels, BoolPixels]) -> Image.Image:
    pixels = np.array(source)
    pixels[masks[0], :3] = (20, 220, 130)
    pixels[masks[1], :3] = (235, 40, 200)
    return Image.fromarray(pixels)


def mask_panel(path: Path, frames: tuple[Image.Image, ...]) -> None:
    yy, xx = np.nonzero(np.asarray(frames[0])[:, :, 3] >= 16)
    bounds = (int(np.min(xx)) - 2, int(np.min(yy)) - 2, int(np.max(xx)) + 3, int(np.max(yy)) + 3)
    width, height = bounds[2] - bounds[0], bounds[3] - bounds[1]
    panel = Image.new("RGB", (width * 18, height * 6 + 22), "#decba9")
    draw = ImageDraw.Draw(panel)
    for index, (frame, label) in enumerate(zip(frames, ("source", "current stored masks", "diagnostic masks"), strict=True)):
        tile = frame.crop(bounds).resize((width * 6, height * 6), Image.Resampling.NEAREST)
        panel.paste(tile, (index * width * 6, 22), tile)
        draw.text((index * width * 6 + 2, 2), label, fill="black")
    panel.save(path)


def snapshot(path: Path, folder: Path) -> bytes:
    data = path.read_bytes()
    _ = (folder / path.name).write_bytes(data)
    return data


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument("--revision", help="New external directory name; existing revisions are immutable")
    args = parser.parse_args(namespace=DiagnosticArgs())
    base = ROOT.parent / "artpolish2-data/masks"
    number = 2
    while (base / f"diagnostic_v{number}").exists():
        number += 1
    revision = args.revision if isinstance(args.revision, str) else f"diagnostic_v{number}"
    if Path(revision).name != revision:
        message = "Revision must be one directory name"
        raise ValueError(message)
    folder = base / revision
    folder.mkdir(parents=True, exist_ok=False)
    inputs = folder / "inputs"
    inputs.mkdir()
    plan = FacePlan.model_validate_json(snapshot(ROOT / "art/character/FACE_VARIANTS.json", inputs))
    annotations = AnatomyAnnotations.model_validate_json(snapshot(ROOT / "art/character/variant_anatomy.json", inputs))
    stored = StoredTransforms.model_validate_json(snapshot(ROOT / "art/character/variant_transforms.json", inputs))
    for code in sorted((ROOT / "art/pipeline_polish2").glob("sprite_*.py")):
        _ = snapshot(code, inputs)
    transforms = {(entry.look, entry.variant): entry for entry in stored.transforms}
    envelopes: dict[str, SourceEnvelope] = {}
    reports: dict[str, JsonValue] = {}
    for name in ("OWNER_SOURCE_ENVELOPES.json", "IDLE_SOURCE_ENVELOPES.json"):
        path = ROOT / "docs/v8/artpolish2" / name
        if path.is_file():
            data = snapshot(path, inputs)
            reports[name] = hashlib.sha256(data).hexdigest()
            envelopes.update((entry.look, entry) for entry in EnvelopeReport.model_validate_json(data).frames)
    rows: list[dict[str, JsonValue]] = []
    for look in plan.looks:
        envelope = envelopes.get(look.look)
        if envelope is None:
            continue
        path = ROOT / "art/character/route_i" / look.look / "S/idle.png"
        if hashlib.sha256(path.read_bytes()).hexdigest() != envelope.source_sha256:
            message = f"Source envelope hash mismatch: {look.look}"
            raise ValueError(message)
        with Image.open(path) as opened:
            source = opened.convert("RGBA")
        guide = FrameGuide(look.look.startswith("child_"), "_loincloth_" in look.look,
                           annotation=annotations.for_look(look.look), envelope=envelope, refine_skin=True, refine_hair=True)
        masks = detect(source, guide)
        skin, hair = stored_mask(look.look, "skin"), stored_mask(look.look, "hair")
        target = folder / look.look
        target.mkdir(exist_ok=True)
        _ = (target / "source.png").write_bytes(path.read_bytes())
        Image.fromarray(skin.astype(np.uint8) * 255).save(target / "current_skin.png")
        Image.fromarray(hair.astype(np.uint8) * 255).save(target / "current_hair.png")
        Image.fromarray(masks.skin.astype(np.uint8) * 255).save(target / "skin.png")
        Image.fromarray(masks.hair.astype(np.uint8) * 255).save(target / "hair.png")
        mask_panel(target / "masks.png", (source, overlay(source, (skin, hair)), overlay(source, (masks.skin, masks.hair))))
        evidence = record(masks)
        evidence.update({"look": look.look, "source_sha256": envelope.source_sha256,
                         "skin_added": int(np.count_nonzero(masks.skin & ~skin)),
                         "skin_removed": int(np.count_nonzero(skin & ~masks.skin)),
                         "hair_added": int(np.count_nonzero(masks.hair & ~hair)),
                         "hair_removed": int(np.count_nonzero(hair & ~masks.hair))})
        changes: list[JsonValue] = []
        sheet = Image.new("RGB", (1200, 520), "#decba9")
        draw = ImageDraw.Draw(sheet)
        for index, variant in enumerate(look.variants):
            transform = transforms[(look.look, variant.id)]
            candidate = shifted(shifted(source, masks.skin, transform.skin_delta_lab), masks.hair, transform.hair_delta_lab)
            candidate.save(target / f"{variant.id}.png")
            current_path = ROOT / "art/character/route_i" / look.look / variant.id / "S/idle.png"
            _ = (target / f"current_{variant.id}.png").write_bytes(current_path.read_bytes())
            with Image.open(current_path) as opened:
                current = opened.convert("RGBA")
            candidate_pixels = np.asarray(candidate, dtype=np.uint8)
            current_pixels = np.asarray(current, dtype=np.uint8)
            changed = np.empty(skin.shape, dtype=np.bool_)
            different = np.empty(candidate_pixels.shape, dtype=np.bool_)
            _ = np.not_equal(candidate_pixels, current_pixels, out=different)
            _ = np.any(different, axis=-1, out=changed)
            head, face = masks.anatomy.head.mask(skin.shape), masks.anatomy.face.mask(skin.shape)
            changes.append({"variant": variant.id, "face_changed_pixels": int(np.count_nonzero(changed & face)),
                            "head_changed_pixels": int(np.count_nonzero(changed & head)),
                            "body_changed_pixels": int(np.count_nonzero(changed & ~head)),
                            "candidate_sha256": hashlib.sha256((target / f"{variant.id}.png").read_bytes()).hexdigest()})
            for row, frame in enumerate((current, candidate)):
                tile = frame.resize((160, 224), Image.Resampling.NEAREST)
                sheet.paste(tile, (index * 200 + 20, row * 260 + 22), tile)
                draw.text((index * 200 + 5, row * 260 + 4), f"{variant.id} {'current' if row == 0 else 'candidate'}", fill="black")
        evidence["variant_changes"] = changes
        rows.append(evidence)
        sheet.save(target / "tints.png")
    summary = {"schema_version": 1, "status": "DIAGNOSTIC_ONLY_NOT_INTEGRATED", "anchors": "existing transforms unchanged",
               "source_reports": reports, "looks": rows}
    _ = (folder / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(f"External candidates: {len(rows)}/34; face passes: {sum(row['face_pass'] is True for row in rows)}; forearm passes: {sum(row['forearms_pass'] is True for row in rows)}")
    print(folder)


if __name__ == "__main__":
    main()
