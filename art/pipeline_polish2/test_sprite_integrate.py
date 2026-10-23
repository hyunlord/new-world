"""Integration requires complete frozen references and exact review provenance."""
from __future__ import annotations

import json
from pathlib import Path

import pytest

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_freeze import FreezeGuard, ReferenceFreeze
from art.pipeline_polish2.sprite_integrate import (
    digest,
    validate_freeze,
    validate_review,
)


def test_incomplete_reference_freeze_cannot_bootstrap_integration() -> None:
    plan = FacePlan.read()
    with pytest.raises(ValueError, match="Complete34"):
        validate_freeze(plan, ReferenceFreeze(looks=()), FreezeGuard({}))
    freeze = ReferenceFreeze.model_validate_json((ROOT / "art/character/variant_reference_freeze.json").read_bytes())
    with pytest.raises(ValueError, match="protect every"):
        validate_freeze(plan, freeze, FreezeGuard({}))
    validate_freeze(plan, freeze, FreezeGuard.read())


def test_reviews_bind_exact_diagnostic_without_waiving_failures(tmp_path: Path) -> None:
    folder = tmp_path / "candidate"
    folder.mkdir()
    summary = folder / "summary.json"
    _ = summary.write_text('{"frames":[]}\n')
    review = tmp_path / "review.json"
    record = {"candidate_directory": str(folder), "summary_sha256": digest(summary), "overall_verdict": "FAIL"}
    _ = review.write_text(json.dumps(record))
    validate_review(review, folder)
    _ = summary.write_text('{"frames":["changed"]}\n')
    with pytest.raises(ValueError, match="different diagnostic"):
        validate_review(review, folder)
    record["summary_sha256"] = digest(summary)
    record["candidate_directory"] = str(tmp_path / "wrong")
    _ = review.write_text(json.dumps(record))
    with pytest.raises(ValueError, match="different diagnostic"):
        validate_review(review, folder)
