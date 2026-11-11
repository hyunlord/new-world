# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: python3 -m art.ui_b4.check ROOT EXTERNAL_REPORT.json
"""Audit B4 artifact identity and coverage, never visual adoption."""

from __future__ import annotations

import hashlib
import sys
import tempfile
from collections import Counter
from collections.abc import Mapping
from dataclasses import replace
from pathlib import Path
from typing import Final

from PIL import Image
from pydantic import TypeAdapter, ValidationError

from art.techmap.assets import (
    AssetRejected,
    Expectation,
    Family,
    Normalization,
    measure,
    normalize,
)
from art.ui_b4.model import (
    Asset,
    AuditResult,
    Catalogue,
    Coverage,
    Generation,
    Lane,
    LaneResult,
    Manifest,
)

LANES: Final[tuple[Lane, ...]] = ("jobs", "cards", "alerts", "needs", "governance")


def digest(path: Path) -> str:
    """Measure actual file identity without reading producer claims."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def audit_asset(root: Path, asset: Asset, generation: Generation) -> tuple[str, ...]:
    """Check a canonical file, its sidecar, and a reproducible raw normalization."""
    errors: list[str] = []
    expected_path = Path(f"art/style05/ui_b4/{asset.lane}/{asset.id}.png")
    family = Family(asset.family)
    if (asset.lane == "cards") != (family == Family.CHAPTER):
        errors.append("lane family differs")
    if asset.path != expected_path:
        return ("canonical path differs from lane/id",)
    path = root / asset.path
    sidecar = Asset.model_validate_json(path.with_suffix(".json").read_bytes())
    if sidecar != asset:
        errors.append("sidecar differs from handoff")
    center = tuple(n // 2 for n in family.canvas)
    if asset.canvas != family.canvas or asset.anchor != center:
        errors.append("canvas or center anchor differs from UI family")
    measurement = measure(path, Expectation(canvas=family.canvas, sha256=asset.sha256))
    with Image.open(path) as source:
        if source.mode != "RGBA":
            errors.append("canonical image is not RGBA")
    if not measurement.passed:
        errors.append("canonical alpha/canvas/hash check failed")
    if digest(asset.attempt_record) != asset.attempt_record_sha256:
        errors.append("attempt record hash differs")
    if (generation.id, generation.family) != (asset.id, asset.family):
        errors.append("selected generation identity differs")
    saved = TypeAdapter(Normalization).validate_json(asset.normalization.read_bytes())
    if saved.raw.sha256 != generation.raw_sha256 or saved.output.sha256 != asset.sha256:
        errors.append("normalization source/output hash differs")
    with tempfile.TemporaryDirectory(prefix="worldsim-b4-audit-") as directory:
        replay_path = Path(directory) / "replay.png"
        replay = normalize(generation.raw_path, replay_path, family)
        if digest(replay_path) != asset.sha256:
            errors.append("raw normalization does not reproduce canonical PNG")
        if replace(saved, raw=replay.raw, output=replay.output) != replay:
            errors.append("normalization geometry or operations differ from replay")
    return tuple(errors)


def audit_generation(
    root: Path, record: Path, generation: Generation
) -> tuple[str, ...]:
    """Compare a frozen attempt's raw source and delivered reference bytes."""
    anchors = root.resolve() / "art/style05/anchors"
    references = {reference.path.resolve() for reference in generation.references}
    subjects = {
        anchors / f"{name}.png"
        for name in ("props", "person", "portrait", "hut", "ground")
    }
    checks = (
        (
            (anchors / "scene.png") in references and bool(subjects & references),
            "missing locked scene and subject references",
        ),
        (digest(generation.raw_path) == generation.raw_sha256, "raw hash differs"),
        (
            digest(generation.original_tool_path) == generation.raw_sha256,
            "original tool output differs",
        ),
        *(
            (
                digest(reference.path) == reference.sha256,
                f"reference hash differs: {reference.path}",
            )
            for reference in generation.references
        ),
    )
    return tuple(f"{message}: {record}" for passed, message in checks if not passed)


def audit_review(
    manifest: Manifest, records: Mapping[Path, Generation]
) -> tuple[str, ...]:
    """Require a measured family fraction and distinct before/after generations."""
    errors: list[str] = []
    review = manifest.self_review
    ids = {asset.id for asset in manifest.assets}
    regenerated = set(review.regenerated_ids)
    compared = tuple(comparison.id for comparison in review.before_after)
    if len(regenerated) != len(review.regenerated_ids):
        errors.append("regenerated IDs contain duplicates")
    if not (regenerated <= ids and regenerated <= set(review.weakest_ids)):
        errors.append("regenerated IDs are not selected weakest-family members")
    if not 10 * len(ids) <= 100 * len(regenerated) <= 20 * len(ids):
        errors.append("regeneration count is outside 10-20 percent")
    if len(set(compared)) != len(compared) or set(compared) != regenerated:
        errors.append("before/after comparisons do not match regenerated IDs")
    for comparison in review.before_after:
        before = records.get(comparison.before_record)
        after = records.get(comparison.after_record)
        if before is None or after is None:
            errors.append(
                f"{comparison.id}: before/after record absent from attempt ledger"
            )
            continue
        subject = (before.id, before.family)
        if subject != (after.id, after.family) or before.id != comparison.id:
            errors.append(f"{comparison.id}: before/after subject differs")
        if after.attempt <= before.attempt or before.raw_sha256 == after.raw_sha256:
            errors.append(f"{comparison.id}: not a distinct later raw image")
    return tuple(errors)


def audit_lane(root: Path, lane: Lane, expected: Coverage) -> LaneResult:
    """Audit one complete family, collecting actionable artifact errors."""
    errors: list[str] = []
    manifest = Manifest.model_validate_json(
        (root / f"art/ui_b4/{lane}.json").read_bytes()
    )
    ids = tuple(asset.id for asset in manifest.assets)
    expected_ids = (*expected.required_ids, *expected.variant_ids)
    if not expected_ids or len(set(expected_ids)) != len(expected_ids):
        errors.append("catalogue coverage is empty or duplicated")
    if manifest.lane != lane or any(asset.lane != lane for asset in manifest.assets):
        errors.append("lane identity differs")
    if len(set(ids)) != len(ids) or set(ids) != set(expected_ids):
        errors.append("asset coverage differs from catalogue or contains duplicates")
    if manifest.coverage != expected:
        errors.append("declared required/variant coverage differs from catalogue")
    rejected = {decision.record for decision in manifest.rejected}
    records: dict[Path, Generation] = {}
    for record_path in manifest.attempts:
        try:
            generation = Generation.model_validate_json(record_path.read_bytes())
            if record_path in records:
                errors.append(f"duplicate attempt record: {record_path}")
            records[record_path] = generation
            if generation.id not in expected_ids and record_path not in rejected:
                errors.append(f"unexpected attempt id: {generation.id}")
            errors.extend(audit_generation(root, record_path, generation))
        except (OSError, ValidationError) as error:
            errors.append(f"generation record {record_path}: {error}")
    for asset in manifest.assets:
        generation = records.get(asset.attempt_record)
        if generation is None:
            errors.append(f"{asset.id}: selected attempt is absent from ledger")
            continue
        try:
            errors.extend(
                f"{asset.id}: {error}" for error in audit_asset(root, asset, generation)
            )
        except (OSError, ValidationError, AssetRejected) as error:
            errors.append(f"{asset.id}: {error}")
    dispositions = Counter(asset.attempt_record for asset in manifest.assets)
    dispositions.update(
        decision.record for decision in (*manifest.rejected, *manifest.superseded)
    )
    if dispositions != Counter(dict.fromkeys(manifest.attempts, 1)):
        errors.append("every attempt must have exactly one disposition")
    counts = Counter(record.id for record in records.values())
    if any(count > 3 for count in counts.values()):
        errors.append("asset has more than three completed attempts")
    errors.extend(audit_review(manifest, records))
    return LaneResult(
        lane=lane,
        assets=len(ids),
        attempts=len(manifest.attempts),
        rejected=len(manifest.rejected),
        superseded=len(manifest.superseded),
        regenerated=len(set(manifest.self_review.regenerated_ids)),
        errors=tuple(errors),
    )


def audit(root: Path) -> AuditResult:
    """Require every declared lane and retain failures in a JSON-reportable result."""
    errors: list[str] = []
    lanes: list[LaneResult] = []
    source: str | None = None
    try:
        catalogue = Catalogue.model_validate_json(
            (root / "art/ui_b4/catalogue.json").read_bytes()
        )
        source = catalogue.runtime_source_commit
        if set(catalogue.lanes) != set(LANES):
            errors.append("catalogue must contain exactly the five B4 lanes")
        for lane in LANES:
            expected = catalogue.lanes.get(lane)
            if expected is None:
                errors.append(f"missing catalogue lane: {lane}")
                continue
            try:
                result = audit_lane(root, lane, expected)
                lanes.append(result)
                errors.extend(f"{lane}: {error}" for error in result.errors)
            except (OSError, ValidationError) as error:
                errors.append(f"{lane}: {error}")
    except (OSError, ValidationError) as error:
        errors.append(f"catalogue: {error}")
    return AuditResult(
        root=root,
        runtime_source_commit=source,
        passed=not errors,
        lanes=tuple(lanes),
        errors=tuple(errors),
    )


def main(arguments: list[str]) -> int:
    """Write external evidence and exit nonzero for missing or invalid artifacts."""
    if len(arguments) != 2:
        _ = sys.stderr.write(
            "Usage: python3 -m art.ui_b4.check ROOT EXTERNAL_REPORT.json\n"
        )
        return 2
    root, destination = (Path(value).resolve() for value in arguments)
    if destination.is_relative_to(root):
        _ = sys.stderr.write("Audit reports must live outside the repository.\n")
        return 2
    result = audit(root)
    destination.parent.mkdir(parents=True, exist_ok=True)
    _ = destination.write_text(
        result.model_dump_json(indent=2) + "\n", encoding="utf-8"
    )
    _ = sys.stdout.write(
        f"B4 file audit: {len(result.lanes)} lanes, {len(result.errors)} errors.\n"
    )
    return int(not result.passed)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
