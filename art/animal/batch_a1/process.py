# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "pydantic"]
# ///
# How to run: python3 -m art.animal.batch_a1.process SPEC.json
"""Preserve generated animal pixels while registering batch A1 candidates."""
from __future__ import annotations

import json
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image, ImageDraw
from pydantic import JsonValue, TypeAdapter

from art.animal.batch_a1.check import Spec, inspect_frame, split_attack, verify
from art.compositor.projection_check import ProjectionSpec, check_projection
from art.pipeline5.sheet_geometry import split_panels
from art.pipeline_env1.animal_normalize import digest, normalize

ROOT: Final = Path(__file__).resolve().parents[3]
JSON: Final = TypeAdapter[JsonValue](JsonValue)



def write_json(path: Path, value: JsonValue) -> None:
    """Write stable, human-readable provenance."""
    path.parent.mkdir(parents=True, exist_ok=True)
    _ = path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")



def contact_sheet(paths: tuple[Path, ...], target: Path) -> None:
    """Show native and nearest 2x frames beside the unchanged adopted references."""
    reference_paths = tuple(ROOT / "art/animal/polish" / species / "try2" / facing / f"{pose}.png"
                            for species, pose4 in (("deer", "graze"), ("wolf", "attack"))
                            for facing in ("S", "E")
                            for pose in ("stand", "walk", "run", pose4, "dead"))
    all_paths = paths + reference_paths
    row_count = (len(all_paths) + 4) // 5
    canvas = Image.new("RGB", (1100, row_count * 320), "#d8cfba")
    draw = ImageDraw.Draw(canvas)
    for index, path in enumerate(all_paths):
        x, y = index % 5 * 220, index // 5 * 320
        draw.text((x + 8, y + 5), "/".join(path.parts[-4:]), fill="#292a24")
        with Image.open(path) as opened:
            sprite = opened.convert("RGBA")
        baseline = 56 if sprite.height == 64 else 72
        for scale, ground in ((1, y + 108), (2, y + 300)):
            enlarged = sprite.resize((sprite.width * scale, sprite.height * scale), Image.Resampling.NEAREST)
            canvas.paste(enlarged, (x + 110 - 48 * scale, ground - baseline * scale), enlarged)
    canvas.save(target, optimize=True)


def process(spec_path: Path) -> Path:
    """Split, normalize and record one sheet without changing its drawn pixels."""
    spec = Spec.model_validate_json(spec_path.read_text())
    evidence = spec.raw.parent / "panels" / spec.raw.stem
    out = ROOT / "art/animal/batch_a1" / spec.species / f"try{spec.attempt}"
    references = [{"path": str(p.resolve()), "sha256": digest(p)} for p in spec.references]
    prompt_text, raw_sha = spec.prompt.read_text(), digest(spec.raw)
    generator = ({"path": str(spec.generator_record), "sha256": digest(spec.generator_record)}
                 if spec.generator_record is not None else None)
    with Image.open(spec.raw) as opened:
        if opened.mode != "RGBA":
            message = "Source must be genuine RGBA; no background removal is performed"
            raise ValueError(message)
        source = opened.copy()
    alpha: NDArray[np.uint8] = np.asarray(source)[:, :, 3]
    source_border = sum(int(np.count_nonzero(e >= 128))
                        for e in (alpha[0], alpha[-1], alpha[:, 0], alpha[:, -1]))
    attack_only = spec.poses == ("attack",)
    if attack_only:
        panels, boxes = split_attack(source)
        ownership = {"reconstructed_exactly": np.array_equal(
            np.concatenate([np.asarray(p) for p in panels], axis=1), np.asarray(source)),
            "foreign_major_pixels": 0,
            "max_gutter_alpha": int(np.max(alpha[:, source.width // 2 - 1:source.width // 2 + 1])),
            "method": "unaltered rectangular crops across alpha<16 gutter; every RGBA pixel retained"}
    else:
        split = split_panels(source, rows=2)
        panels, boxes = split.images, split.boxes
        ownership = asdict(split.ownership)
    upright_scale = spec.shoulder_target / (spec.hoof_y - spec.shoulder_y)
    evidence.mkdir(parents=True, exist_ok=True)
    records: list[JsonValue] = []
    paths: list[Path] = []
    passed: list[bool] = []
    for index, panel in enumerate(panels):
        facing, pose = ("S", "E")[index // len(spec.poses)], spec.poses[index % len(spec.poses)]
        dead = pose == "dead"
        bounds = Image.fromarray(np.asarray(panel)[:, :, 3] >= 128).getbbox()
        if bounds is None:
            message = f"No opaque figure in {facing}/{pose}"
            raise ValueError(message)
        scale = spec.dead_length / max(bounds[2] - bounds[0], bounds[3] - bounds[1]) if dead else upright_scale
        product = normalize(panel, scale, dead)
        result = inspect_frame(product.image, dead, spec.dead_length)
        checks = asdict(result)
        checks.update(source_border=source_border == 0,
                      ownership=bool(ownership["reconstructed_exactly"]) and ownership["foreign_major_pixels"] == 0,
                      discarded_alpha=product.transform.discarded_max_alpha < 16)
        structural = all(checks.values())
        target = out / facing / f"{pose}.png"
        target.parent.mkdir(parents=True, exist_ok=True)
        product.image.save(target, optimize=True)
        native = evidence / f"{facing}_{pose}.png"
        panel.save(native)
        anchor, canvas = (48, 56 if dead else 72), product.image.size
        projection = check_projection(product.image, ProjectionSpec(spec.species, anchor, (1, canvas[1] - 8)))
        record = JSON.validate_json(json.dumps({
            "species": spec.species, "facing": facing, "pose": pose, "attempt": spec.attempt,
            "image": str(target.relative_to(ROOT)), "image_sha256": digest(target),
            "canvas_px": canvas, "anchor_px": anchor, "footprint_tiles": [1, 1],
            "source_panel": str(native), "source_panel_sha256": digest(native), "source_box": boxes[index],
            "raw": str(spec.raw), "raw_sha256": raw_sha, "prompt": str(spec.prompt),
            "prompt_sha256": digest(spec.prompt), "references": references,
            "scale": scale, "upright_scale": upright_scale, "normalization": asdict(product.transform),
            "checks": checks, "structural_pass": structural,
            "projection_report_only": asdict(projection),
            "projection_note": "Canvas-extent height band only; gradient cannot prove camera angle or anatomical shoulder height.",
            "status": "CANDIDATE_PENDING_VISUAL_REVIEW",
        }))
        write_json(target.with_suffix(".json"), record)
        records.append(record)
        paths.append(target)
        passed.append(structural)
    sheet = out / ("attack_sheet.png" if attack_only else "contact_sheet.png")
    contact_sheet(tuple(paths), sheet)
    manifest = out / ("ATTACK_MANIFEST.json" if attack_only else "MANIFEST.json")
    write_json(manifest, JSON.validate_json(json.dumps({
        "spec": spec.model_dump(mode="json"), "spec_path": str(spec_path.resolve()), "spec_sha256": digest(spec_path),
        "raw_sha256": raw_sha, "prompt_text": prompt_text, "prompt_sha256": digest(spec.prompt),
        "seed": spec.seed, "seed_note": "null means generator did not expose a seed", "generator_record": generator,
        "references": references, "ownership": ownership, "source_opaque_border_pixels": source_border,
        "upright_scale": upright_scale, "landmark_note": spec.landmark_note,
        "frames": records, "structural_pass": all(passed), "frame_count": len(records),
        "contact_sheet": str(sheet.relative_to(ROOT)), "contact_sheet_sha256": digest(sheet),
        "status": "CANDIDATE_PENDING_VISUAL_REVIEW",
    })))
    print(json.dumps({"manifest": str(manifest), "structural_pass": all(passed), "frames": len(records)}))
    return manifest


def main() -> int:
    """Run one spec; structural rejection produces a nonzero exit and keeps evidence."""
    if len(sys.argv) != 2:
        print("Usage: python3 -m art.animal.batch_a1.process SPEC.json", file=sys.stderr)
        return 2
    manifest = process(Path(sys.argv[1]))
    return 0 if verify(manifest) else 1


if __name__ == "__main__":
    raise SystemExit(main())
