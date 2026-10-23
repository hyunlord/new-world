# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_build --diagnostic frames_diagnostic_v2 --review <review-json>
"""Install a reviewed complete diagnostic while preserving frozen idle assets."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image
from pydantic import BaseModel, JsonValue

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_colour import rgb_to_lab, shifted
from art.pipeline_polish2.sprite_freeze import FreezeGuard, ReferenceFreeze
from art.pipeline_polish2.sprite_records import (
    FrameEvidence,
    StoredTransforms,
    field,
    read_mask,
)


class IntegrateArgs(argparse.Namespace):
    diagnostic: str = ""
    review: list[str] | None = None


class BoundReview(BaseModel):
    candidate_directory: str
    summary_sha256: str


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_freeze(plan: FacePlan, freeze: ReferenceFreeze, guard: FreezeGuard) -> None:
    frozen_looks = {entry.look for entry in freeze.looks}
    if len(freeze.looks) != 34 or frozen_looks != {look.look for look in plan.looks}:
        raise ValueError("Complete34-look reference freeze required before integration")
    required: set[Path] = {ROOT / "art/character/variant_transforms.json"}
    for look in freeze.looks:
        if (look.source.path != f"art/character/route_i/{look.look}/S/idle.png"
                or look.skin_mask.path != f"art/character/variant_masks/{look.look}/S/idle_skin.png"
                or look.hair_mask.path != f"art/character/variant_masks/{look.look}/S/idle_hair.png"):
            raise ValueError(f"Frozen source/mask paths disagree with their look: {look.look}")
        expected_variants = {f"art/character/route_i/{look.look}/{variant.id}/S/idle.png" for variant in plan.for_look(look.look).variants}
        if len(look.variants) != 6 or {entry.path for entry in look.variants} != expected_variants:
            raise ValueError(f"Incomplete frozen variant references: {look.look}")
        required.update(ROOT / path for path in expected_variants)
        required.add(ROOT / "art/character/route_i" / look.look / "S/idle.png")
        required.update(ROOT / "art/character/variant_masks" / look.look / "S" / f"idle_{region}.png" for region in ("skin", "hair"))
    if not required.issubset(guard.hashes):
        raise ValueError("Freeze must protect every original, mask, reference and transform record")


def validate_review(path: Path, folder: Path) -> None:
    review = BoundReview.model_validate_json(path.read_bytes())
    if Path(review.candidate_directory).resolve() != folder.resolve() or review.summary_sha256 != digest(folder / "summary.json"):
        raise ValueError(f"Review is bound to a different diagnostic: {path}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument("--diagnostic", required=True)
    _ = parser.add_argument("--review", action="append", required=True)
    args = parser.parse_args(namespace=IntegrateArgs())
    if Path(args.diagnostic).name != args.diagnostic:
        raise ValueError("Diagnostic must be a single external directory name")
    folder = ROOT.parent / "artpolish2-data/masks" / args.diagnostic
    evidence = FrameEvidence.model_validate_json((folder / "summary.json").read_bytes())
    plan = FacePlan.read()
    freeze_path = ROOT / "art/character/variant_reference_freeze.json"
    freeze = ReferenceFreeze.model_validate_json(freeze_path.read_bytes())
    guard = FreezeGuard.read()
    validate_freeze(plan, freeze, guard)
    expected = {(look.look, facing, pose) for look in plan.looks for facing in ("S", "E")
                for pose in ("idle", "walk_a", "work_a", "carry", "sleep")}
    keys = [(field(row, "look"), field(row, "facing"), field(row, "pose")) for row in evidence.frames]
    if len(keys) != 340 or set(keys) != expected:
        raise ValueError("Diagnostic does not contain exactly the340 expected frame keys")
    transforms_path = ROOT / "art/character/variant_transforms.json"
    if transforms_path not in guard.hashes:
        raise ValueError("Transform record must be protected by the reference freeze")
    if digest(transforms_path) != digest(folder / "inputs/variant_transforms.json"):
        raise ValueError("Diagnostic did not use the currently frozen transform record")
    if digest(ROOT / "art/character/FACE_VARIANTS.json") != digest(folder / "inputs/FACE_VARIANTS.json"):
        raise ValueError("Diagnostic variant plan differs")
    for path, expected_hash in evidence.source_reports.items():
        if digest(ROOT / path) != expected_hash:
            raise ValueError(f"Source report changed after diagnostic: {path}")
    reviews: list[dict[str, JsonValue]] = []
    for name in args.review or []:
        path = ROOT / name
        validate_review(path, folder)
        reviews.append({"path": name, "sha256": digest(path)})
    stored = StoredTransforms.model_validate_json(transforms_path.read_bytes())
    transforms = {(entry.look, entry.variant): entry for entry in stored.transforms}
    if len(transforms) != 204:
        raise ValueError("Transform inventory is incomplete")
    copies: list[tuple[Path, Path, str]] = []
    rows: list[dict[str, JsonValue]] = []
    for entry, (look, facing, pose) in zip(evidence.frames, keys, strict=True):
        source_path = ROOT / "art/character/route_i" / look / facing / f"{pose}.png"
        target = folder / look / facing / pose
        if digest(source_path) != field(entry, "source_sha256"):
            raise ValueError(f"Original changed: {look}/{facing}/{pose}")
        with Image.open(source_path) as opened:
            source = opened.convert("RGBA")
        skin, hair = read_mask(target / "skin.png"), read_mask(target / "hair.png")
        original = np.asarray(source)
        outline = (rgb_to_lab(original[:, :, :3].astype(np.float64))[:, :, 0] <= 18) | (original[:, :, 3] < 128)
        if np.any(skin & hair) or np.any((skin | hair) & outline):
            raise ValueError(f"Invalid masks: {look}/{facing}/{pose}")
        for region in ("skin", "hair"):
            copies.append((target / f"{region}.png", ROOT / "art/character/variant_masks" / look / facing / f"{pose}_{region}.png",
                           field(entry, f"{region}_sha256")))
        hashes = entry["candidate_sha256"]
        if not isinstance(hashes, dict):
            raise TypeError("Candidate hash map is required")
        for variant in plan.for_look(look).variants:
            expected_hash = field(hashes, variant.id)
            candidate = target / f"{variant.id}.png"
            with Image.open(candidate) as opened:
                baked = opened.convert("RGBA")
            transform = transforms[(look, variant.id)]
            reproduced = shifted(shifted(source, skin, transform.skin_delta_lab), hair, transform.hair_delta_lab)
            if not np.array_equal(np.asarray(reproduced), np.asarray(baked)):
                raise ValueError(f"Candidate does not reproduce: {look}/{variant.id}/{facing}/{pose}")
            copies.append((candidate, ROOT / "art/character/route_i" / look / variant.id / facing / f"{pose}.png", expected_hash))
        rows.append(entry | {"baked_sha256": hashes, "source": str(source_path.relative_to(ROOT)),
                             "semantic_status": "RECORDED_IN_LINKED_INDEPENDENT_REVIEWS",
                             "hair_formal_status": "NOT_CLAIMED_SEE_LINKED_FIXED40_AUDIT"})
    for source, target, expected_hash in copies:
        if digest(source) != expected_hash:
            raise ValueError(f"Candidate bytes changed: {source}")
        frozen_hash = guard.hashes.get(target)
        if frozen_hash is not None and expected_hash != frozen_hash:
            raise ValueError(f"Candidate would change a frozen reference: {target}")
    manifest = {"schema_version": 2, "source_reports": evidence.source_reports, "semantic_reviews": reviews,
                "diagnostic_summary_sha256": digest(folder / "summary.json"),
                "semantic_status": "SEE_INDEPENDENT_REVIEWS_NO_NUMERIC_WAIVER", "frames": rows}
    manifest_data = (json.dumps(manifest, separators=(",", ":")) + "\n").encode()
    if len(manifest_data) > 1_500_000:
        raise ValueError("Mask manifest exceeds the committed-file size limit")
    for source, target, expected_hash in copies:
        if target in guard.hashes:
            continue
        data = source.read_bytes()
        if hashlib.sha256(data).hexdigest() != expected_hash:
            raise ValueError(f"Candidate changed during install: {source}")
        _ = target.write_bytes(data)
    for _, target, expected_hash in copies:
        if digest(target) != expected_hash:
            raise ValueError(f"Installed file differs: {target}")
    _ = FreezeGuard.read()
    _ = (ROOT / "art/character/variant_masks/MANIFEST.json").write_bytes(manifest_data)
    print("Installed and verified 680 masks and 2040 bakes; all 34 frozen idle references and 204 transforms unchanged")


if __name__ == "__main__":
    main()
