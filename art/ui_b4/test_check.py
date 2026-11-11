# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "pytest"]
# ///
# How to run: python3 -m pytest art/ui_b4/test_check.py
"""Reject missing or fabricated provenance using real temporary PNG exports."""
# noqa: SIZE_OK - Owner keeps fixture, CLI, and rule tests in one bounded test file.

from __future__ import annotations

import hashlib
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Literal, assert_never

import pytest
from PIL import Image
from pydantic import TypeAdapter

from art.techmap.assets import Family, Normalization, normalize
from art.ui_b4.check import LANES, audit_asset, audit_lane
from art.ui_b4.model import (
    Asset,
    AuditResult,
    Catalogue,
    Comparison,
    Coverage,
    Decision,
    Generation,
    Lane,
    Manifest,
    Reference,
    SelfReview,
)


@dataclass(frozen=True, slots=True)
class Bundle:
    """A complete temporary family and its independent coverage contract."""

    root: Path
    expected: Coverage
    manifest: Manifest


def digest(path: Path) -> str:
    """Hash a fixture's real bytes."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save_manifest(bundle: Bundle, manifest: Manifest) -> None:
    """Publish a fixture handoff with matching sidecars."""
    for asset in manifest.assets:
        _ = (
            (bundle.root / asset.path)
            .with_suffix(".json")
            .write_text(asset.model_dump_json(indent=2) + "\n", encoding="utf-8")
        )
    _ = (bundle.root / "art/ui_b4/jobs.json").write_text(
        manifest.model_dump_json(indent=2) + "\n", encoding="utf-8"
    )


@pytest.fixture
def bundle(tmp_path: Path) -> Bundle:
    """Five selected icons and a genuine sixth generation form a 20% review."""
    (tmp_path / "art/ui_b4").mkdir(parents=True)
    evidence = tmp_path / "evidence"
    evidence.mkdir()
    anchors = tmp_path / "art/style05/anchors"
    anchors.mkdir(parents=True)
    references: list[Reference] = []
    for name in ("scene", "props"):
        reference = anchors / f"{name}.png"
        Image.new("RGBA", (8, 8), (80, 60, 40, 255)).save(reference)
        references.append(Reference(path=reference, sha256=digest(reference)))
    records: list[Path] = []
    selected: list[Asset] = []
    for identifier in ("a", "b", "c", "d", "e", "a"):
        attempt = 2 if selected and identifier == "a" else 1
        raw = evidence / f"{identifier}-{attempt}.png"
        image = Image.new("RGBA", (96, 96))
        image.paste((80 + attempt, 60, 40, 255), (20, 12, 76, 84))
        image.save(raw)
        generation = Generation(
            id=identifier,
            family="icon",
            attempt=attempt,
            prompt="Test fixture",
            raw_path=raw,
            original_tool_path=raw,
            raw_sha256=digest(raw),
            references=tuple(references),
            tool="fixture",
            seed=None,
        )
        record = raw.with_suffix(".json")
        _ = record.write_text(generation.model_dump_json(), encoding="utf-8")
        records.append(record)
        if attempt == 2:
            continue
        relative = Path(f"art/style05/ui_b4/jobs/{identifier}.png")
        result = normalize(raw, tmp_path / relative, Family.ICON)
        normalization = evidence / f"{identifier}-normalization.json"
        _ = normalization.write_bytes(TypeAdapter(Normalization).dump_json(result))
        selected.append(
            Asset(
                id=identifier,
                lane="jobs",
                family="icon",
                era="paleolithic",
                canvas=(64, 64),
                anchor=(32, 32),
                footprint=None,
                path=relative,
                sha256=digest(tmp_path / relative),
                attempt_record=record,
                attempt_record_sha256=digest(record),
                normalization=normalization,
                status="candidate_selected",
                semantic_note="Fixture icon",
                source_runtime_ids=[],
            )
        )
    expected = Coverage(required_ids=("a", "b", "c", "d", "e"), variant_ids=())
    manifest = Manifest(
        schema_version=1,
        lane="jobs",
        assets=tuple(selected),
        attempts=tuple(records),
        rejected=(),
        superseded=(Decision(record=records[-1], reason="Weaker after review"),),
        coverage=expected,
        self_review=SelfReview(
            weakest_ids=("a",),
            regenerated_ids=("a",),
            before_after=(
                Comparison(
                    id="a",
                    before_record=records[0],
                    after_record=records[-1],
                    decision="retain_before",
                    reason="Original silhouette reads better",
                ),
            ),
        ),
    )
    result = Bundle(tmp_path, expected, manifest)
    save_manifest(result, manifest)
    return result


@pytest.mark.parametrize("review_attempt", (2, 3))
def test_valid_provenance_passes_even_when_original_is_retained(
    bundle: Bundle, review_attempt: int
) -> None:
    # Given genuine regeneration whose original remains selected.
    # An interrupted call may leave a gap in the completed attempt numbers.
    record = bundle.manifest.attempts[-1]
    generation = Generation.model_validate_json(record.read_bytes())
    changed = generation.model_copy(update={"attempt": review_attempt})
    _ = record.write_text(changed.model_dump_json(), encoding="utf-8")
    # When auditing the complete candidate family.
    result = audit_lane(bundle.root, "jobs", bundle.expected)
    # Then mechanical provenance passes without judging aesthetic preference.
    assert result.errors == ()
    assert (result.assets, result.attempts, result.regenerated) == (5, 6, 1)


Defect = Literal[
    "missing",
    "duplicate",
    "hash",
    "anchor",
    "opaque",
    "reference",
    "sidecar",
    "normalization",
    "fake_review",
    "review_count",
    "scene_only",
    "subject_only",
    "unaccounted",
    "duplicate_disposition",
    "overlap",
    "fourth_attempt",
    "too_many_attempts",
]


@pytest.mark.parametrize(
    ("defect", "fragment"),
    [
        ("missing", "coverage differs"),
        ("duplicate", "coverage differs"),
        ("hash", "alpha/canvas/hash check failed"),
        ("anchor", "center anchor differs"),
        ("opaque", "alpha/canvas/hash check failed"),
        ("reference", "reference hash differs"),
        ("sidecar", "sidecar differs"),
        ("normalization", "a-normalization.json"),
        ("fake_review", "distinct later raw image"),
        ("review_count", "outside 10-20 percent"),
        ("scene_only", "locked scene and subject"),
        ("subject_only", "locked scene and subject"),
        ("unaccounted", "exactly one disposition"),
        ("duplicate_disposition", "exactly one disposition"),
        ("overlap", "exactly one disposition"),
        ("fourth_attempt", "less than or equal to 3"),
        ("too_many_attempts", "more than three completed attempts"),
    ],
)
def test_concrete_provenance_defects_fail(
    bundle: Bundle, defect: Defect, fragment: str
) -> None:
    # Given one concrete defect in a complete family.
    manifest = bundle.manifest
    first = manifest.assets[0]
    match defect:
        case "missing":
            manifest = manifest.model_copy(update={"assets": manifest.assets[:-1]})
        case "duplicate":
            manifest = manifest.model_copy(update={"assets": (*manifest.assets, first)})
        case "hash":
            first = first.model_copy(update={"sha256": "0" * 64})
        case "anchor":
            first = first.model_copy(update={"anchor": (32, 63)})
        case "opaque":
            path = bundle.root / first.path
            Image.new("RGBA", (64, 64), (80, 60, 40, 255)).save(path)
            first = first.model_copy(update={"sha256": digest(path)})
        case "reference":
            _ = (bundle.root / "art/style05/anchors/props.png").write_bytes(
                b"changed reference"
            )
        case "sidecar":
            first = first.model_copy(update={"semantic_note": "Mismatch"})
        case "normalization":
            first.normalization.unlink()
        case "fake_review":
            before = Generation.model_validate_json(manifest.attempts[0].read_bytes())
            after = before.model_copy(update={"attempt": 2})
            _ = manifest.attempts[-1].write_text(
                after.model_dump_json(), encoding="utf-8"
            )
        case "review_count":
            review = manifest.self_review.model_copy(update={"regenerated_ids": ()})
            manifest = manifest.model_copy(update={"self_review": review})
        case "scene_only" | "subject_only":
            record = first.attempt_record
            generation = Generation.model_validate_json(record.read_bytes())
            index = ("scene_only", "subject_only").index(defect)
            changed = generation.model_copy(
                update={"references": (generation.references[index],)}
            )
            _ = record.write_text(changed.model_dump_json(), encoding="utf-8")
        case "fourth_attempt":
            record = first.attempt_record
            generation = Generation.model_validate_json(record.read_bytes())
            changed = generation.model_copy(update={"attempt": 4})
            _ = first.attempt_record.write_text(
                changed.model_dump_json(), encoding="utf-8"
            )
        case "unaccounted" | "duplicate_disposition" | "overlap":
            dispositions = (
                (),
                manifest.superseded * 2,
                (
                    Decision(record=first.attempt_record, reason="Duplicate selection"),
                    *manifest.superseded,
                ),
            )
            index = ("unaccounted", "duplicate_disposition", "overlap").index(defect)
            manifest = manifest.model_copy(update={"superseded": dispositions[index]})
        case "too_many_attempts":
            for number in (2, 3):
                extra = first.attempt_record.with_name(f"extra-{number}.json")
                generation = Generation.model_validate_json(
                    first.attempt_record.read_bytes()
                ).model_copy(update={"attempt": number})
                _ = extra.write_text(generation.model_dump_json(), encoding="utf-8")
                manifest = manifest.model_copy(
                    update={
                        "attempts": (*manifest.attempts, extra),
                        "rejected": (
                            *manifest.rejected,
                            Decision(record=extra, reason="Fixture extra attempt"),
                        ),
                    }
                )
        case unreachable:
            assert_never(unreachable)
    if first != bundle.manifest.assets[0]:
        manifest = manifest.model_copy(update={"assets": (first, *manifest.assets[1:])})
    save_manifest(bundle, manifest)
    if defect == "sidecar":
        _ = (
            (bundle.root / first.path)
            .with_suffix(".json")
            .write_text(bundle.manifest.assets[0].model_dump_json(), encoding="utf-8")
        )
    # When the audit reads actual files rather than trusting the handoff.
    result = audit_lane(bundle.root, "jobs", bundle.expected)
    # Then the defective provenance cannot be accepted.
    assert any(fragment in error for error in result.errors), result.errors


def test_cli_reports_missing_lanes_instead_of_accepting_partial_work(
    bundle: Bundle,
) -> None:
    # Given a catalogue for five families with only the jobs handoff available.
    catalogue = Catalogue(
        runtime_source_commit="1" * 40, lanes={lane: bundle.expected for lane in LANES}
    )
    _ = (bundle.root / "art/ui_b4/catalogue.json").write_text(
        catalogue.model_dump_json(), encoding="utf-8"
    )
    report = bundle.root.parent / f"{bundle.root.name}-audit.json"
    # When invoking the real command surface.
    result = subprocess.run(
        [sys.executable, "-m", "art.ui_b4.check", str(bundle.root), str(report)],
        capture_output=True,
        text=True,
        check=False,
    )
    # Then a structured external failure names the four absent families.
    assert result.returncode == 1, result.stderr
    audited = AuditResult.model_validate_json(report.read_bytes())
    assert not audited.passed
    assert len(audited.errors) == 4
    assert tuple(lane.lane for lane in audited.lanes) == ("jobs",)


@pytest.mark.parametrize("lane", LANES)
def test_lane_requires_its_canvas_family(bundle: Bundle, lane: Lane) -> None:
    # Given an internally consistent chapter in an icon lane, or icon in cards.
    first = bundle.manifest.assets[0]
    family = {"cards": Family.ICON}.get(lane, Family.CHAPTER)
    path = Path(f"art/style05/ui_b4/{lane}/a.png")
    generation = Generation.model_validate_json(
        first.attempt_record.read_bytes()
    ).model_copy(update={"family": family.value})
    _ = first.attempt_record.write_text(generation.model_dump_json(), encoding="utf-8")
    result = normalize(generation.raw_path, bundle.root / path, family)
    _ = first.normalization.write_bytes(TypeAdapter(Normalization).dump_json(result))
    asset = first.model_copy(
        update={
            "lane": lane,
            "family": family.value,
            "path": path,
            "canvas": family.canvas,
            "anchor": result.anchor,
            "sha256": digest(bundle.root / path),
            "attempt_record_sha256": digest(first.attempt_record),
        }
    )
    _ = (
        (bundle.root / path)
        .with_suffix(".json")
        .write_text(asset.model_dump_json(), encoding="utf-8")
    )
    # When auditing the actual normalized artifact, then reject the wrong family.
    assert "lane family differs" in audit_asset(bundle.root, asset, generation)


@pytest.mark.parametrize("rejected", (True, False))
def test_omitted_variant_can_remain_as_rejected_evidence(
    bundle: Bundle, rejected: bool
) -> None:
    # Given completed art omitted from the final catalogue after a scope change.
    source = bundle.manifest.attempts[0]
    generation = Generation.model_validate_json(source.read_bytes()).model_copy(
        update={"id": "omitted_variant"}
    )
    record = source.with_name("omitted_variant.json")
    _ = record.write_text(generation.model_dump_json(), encoding="utf-8")
    decision = Decision(record=record, reason="Owner omitted optional variants")
    manifest = bundle.manifest.model_copy(
        update={"attempts": (*bundle.manifest.attempts, record)}
    )
    dispositions = (
        (*manifest.rejected, decision) if rejected else (*manifest.superseded, decision)
    )
    manifest = manifest.model_copy(
        update={"rejected" if rejected else "superseded": dispositions}
    )
    save_manifest(bundle, manifest)
    # When checking the complete ledger, then only explicit rejection permits it.
    result = audit_lane(bundle.root, "jobs", bundle.expected)
    assert bool(result.errors) is not rejected, result.errors
