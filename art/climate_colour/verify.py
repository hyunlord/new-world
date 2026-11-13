# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy", "pillow", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: uv run --with numpy --with pillow --with pydantic --with scipy --with scikit-image python -m art.climate_colour.verify
"""Recompute geometry, provenance and colour evidence from saved R2 outputs."""

from __future__ import annotations

import json
import subprocess
from typing import ClassVar

import numpy as np
from numpy.typing import NDArray
from pydantic import BaseModel, ConfigDict, TypeAdapter

from art.climate_colour.colour import distribution
from art.climate_colour.export import B2, BASE, DATA, OUT, unchanged
from art.climate_colour.models import Draft, ShrubFamily
from art.climate_colour.references import ROOT, read, reference
from art.flora.check_assets import mask_metrics, summer_metrics
from art.flora.normalize import digest
from art.flora.seasons import canopy_mask
from art.pipeline2.ground import measure


class Handoff(Draft):
    """An importer must resolve a real immutable asset commit."""

    source_commit: str | None


class GroundOutput(BaseModel):
    """Geometry and source identity for rejected ground review tiles."""

    model_config: ClassVar[ConfigDict] = ConfigDict(extra="ignore", frozen=True)
    id: str
    source: str
    source_sha256: str
    path: str
    sha256: str
    status: str


def main() -> None:
    """Fail on absent rows, unknown hashes, changed geometry or leaked proposals."""
    draft = Handoff.model_validate_json((OUT / "art_import.draft.json").read_bytes())
    baseline = Draft.model_validate_json(
        (BASE / "art/climate/art_import.draft.json").read_bytes()
    )
    assert (
        len(draft.files),
        len(draft.proposed_files),
        len(draft.existing_references),
    ) == (67, 12, 19)
    assert len({r.id for r in (*draft.files, *draft.proposed_files)}) == 79
    assert {r.id for r in draft.files} == {r.id for r in baseline.files}
    assert {r.id for r in draft.proposed_files} == {
        r.id for r in baseline.proposed_files
    }
    originals = {r.id: r for r in (*baseline.files, *baseline.proposed_files)}
    shrubs = TypeAdapter(list[ShrubFamily]).validate_json(
        (BASE / "art/climate/shrubs/catalog.json").read_bytes()
    )
    changed = 0
    olive_failures = 0
    worst_error = 0.0
    for row in (*draft.files, *draft.proposed_files):
        original = originals[row.id]
        assert (
            row.canvas == original.canvas
            and row.anchor == original.anchor
            and row.footprint == original.footprint
        )
        source, output = read(BASE / row.path), read(ROOT / row.path)
        assert source.size == output.size == row.canvas and unchanged(source, output), (
            row.id
        )
        assert (
            digest(BASE / row.path) == original.sha256
            and digest(ROOT / row.path) == row.sha256
        )
        changed += source.tobytes() != output.tobytes()
        if original.lane == "shrubs":
            family, state = row.id.rsplit("_", 1)
            climates = next(r.climate for r in shrubs if r.id == family)
            original = original.model_copy(
                update={"state": state, "species": family, "climate": climates}
            )
        control, _, _ = reference(original)
        residual: NDArray[np.float64] = np.abs(
            np.array(distribution(output), dtype=np.float64)
            - np.array(distribution(control), dtype=np.float64)
        )
        error = float(np.max(residual))
        worst_error = max(worst_error, error)
        assert error <= 0.025, (row.id, error)
        source_blob = subprocess.check_output(
            ["git", "show", f"{B2}:{row.path}"], cwd=ROOT
        )
        assert source_blob == (BASE / row.path).read_bytes(), row.id
        if row.species == "wild_olive":
            mask = read(BASE / row.mask)
            assert (ROOT / row.mask).read_bytes() == (BASE / row.mask).read_bytes()
            summer = summer_metrics(output, mask, ROOT)
            classified = summer_metrics(output, canopy_mask(output, ROOT), ROOT)
            anatomy = mask_metrics(output, mask, ROOT)
            olive_failures += not (
                summer.passed and classified.passed and anatomy.passed
            )
            assert row.id not in {r.id for r in draft.files}
        if draft.source_commit:
            pinned = subprocess.check_output(
                ["git", "show", f"{draft.source_commit}:{row.path}"], cwd=ROOT
            )
            assert pinned == (ROOT / row.path).read_bytes(), row.id
    assert changed == 53 and olive_failures == 12, (changed, olive_failures)
    grounds = TypeAdapter(list[GroundOutput]).validate_json(
        (OUT / "ground_review.json").read_bytes()
    )
    assert len(grounds) == 8
    for row in grounds:
        source, output = read(DATA / row.source), read(DATA / row.path)
        assert source.size == output.size == (64, 64) and unchanged(source, output)
        assert (
            digest(DATA / row.source) == row.source_sha256
            and digest(DATA / row.path) == row.sha256
        )
        assert not measure(output).passed and row.status.startswith("BLOCKED")
    for row in draft.existing_references:
        assert digest(ROOT / row.path) == row.sha256
    result = {
        "flora": 79,
        "changed": changed,
        "identity_seasons": 26,
        "geometry_alpha_hidden": "PASS",
        "source_blob_identity": "PASS",
        "worst_HSV_quantile_error": worst_error,
        "tolerance": 0.025,
        "olive_blocked": olive_failures,
        "ground_blocked": 8,
        "source_commit": draft.source_commit,
    }
    _ = (DATA / "verification.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
