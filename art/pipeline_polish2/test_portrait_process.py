# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic", "typer", "numpy", "scipy", "scikit-image"]
# ///
# How to run: uv run --with pytest --with pillow --with pydantic --with typer --with numpy --with scipy --with scikit-image python -m pytest art/pipeline_polish2/test_portrait_process.py -q
"""Protect raw provenance and the boundary between observations and intended geometry."""
from __future__ import annotations

from pathlib import Path

import pytest
from PIL import Image

from art.pipeline_env1.portrait_finalize import Inspection
from art.pipeline_polish2.portrait_appearance import AppearanceManifest
from art.pipeline_polish2.portrait_models import (
    AssetRef,
    Request,
    SourceAppearance,
    write_evidence,
)
from art.pipeline_polish2.portrait_process import Processed, ProcessInput, process


@pytest.fixture
def request_path(tmp_path: Path) -> Path:
    raw = tmp_path / "fixture.png"
    Image.new("RGB", (256, 256), "#a08060").save(raw)
    ref = AssetRef.of(raw)
    request = Request(look="synthetic_look", variant="v1", description="SYNTHETIC fixture", prompt="test only",
                      references=(ref, ref), face_plan=ref, canonical_path=tmp_path / "canonical.png",
                      raw_directory=tmp_path / "generation")
    path = tmp_path / "request.json"
    write_evidence(path, request)
    return path


def test_process_preserves_raw_and_leaves_structure_pending_without_observation(request_path: Path) -> None:
    # Given: a hashed synthetic square input without a geometric observation.
    request = Request.model_validate_json(request_path.read_bytes())
    spec = ProcessInput(request_path=request_path, raw_path=request.references[0].path, attempt=1)
    # When: drive the actual raw preservation, numeric judge and canonicalization path.
    result = process(spec)
    # Then: only a full-frame resize is baked; no structural result is fabricated.
    assert result.raw.path.read_bytes() == request.references[0].path.read_bytes()
    assert result.structure_status == "PENDING_OBSERVATION" and result.structural is None
    assert not result.owner_adopted
    with Image.open(request.references[0].path) as original, Image.open(result.candidate.portrait.path) as output:
        assert output.tobytes() == original.convert("RGB").resize((128, 128), Image.Resampling.LANCZOS).tobytes()


def test_process_rejects_changed_tinted_reference(request_path: Path) -> None:
    # Given: the actual tinted reference was changed after request freeze.
    request = Request.model_validate_json(request_path.read_bytes())
    Image.new("RGB", (256, 256), "#202020").save(request.references[0].path)
    # When/Then: fail before publishing a portrait that would claim stale provenance.
    with pytest.raises(ValueError, match="changed after request"):
        _ = process(ProcessInput(request_path=request_path, raw_path=request.references[0].path, attempt=1))
    assert not request.canonical_path.exists()


def test_process_rejects_replacing_an_existing_attempt(request_path: Path, tmp_path: Path) -> None:
    # Given: an original attempt is preserved and a different output arrives under same attempt number.
    request = Request.model_validate_json(request_path.read_bytes())
    _ = process(ProcessInput(request_path=request_path, raw_path=request.references[0].path, attempt=1))
    replacement = tmp_path / "replacement.png"
    Image.new("RGB", (256, 256), "#202020").save(replacement)
    # When/Then: reject overwrite of the historical raw output.
    with pytest.raises(ValueError, match="different raw image"):
        _ = process(ProcessInput(request_path=request_path, raw_path=replacement, attempt=1))


def test_changed_inspection_preserves_original_failed_observation(request_path: Path, tmp_path: Path) -> None:
    # Given: a real processing path with an explicitly synthetic failed observation.
    request = Request.model_validate_json(request_path.read_bytes())
    observation = Inspection(look="synthetic_look/v1", attempt=1, head_bbox=(0, 0, 128, 128),
                  face_bbox=(0, 0, 128, 128), head_uncropped=False, hair_match=False, top_match=False,
                  extra_accessory=True, identity_note="SYNTHETIC FAIL", inspection_method="Synthetic regression fixture")
    path = tmp_path / "inspection.json"
    write_evidence(path, observation)
    spec = ProcessInput(request_path=request_path, raw_path=request.references[0].path, attempt=1, inspection_path=path)
    failed = process(spec)
    passed = observation.model_copy(update={"head_bbox": (20, 10, 108, 110), "face_bbox": (40, 32, 88, 88),
                            "head_uncropped": True, "hair_match": True, "top_match": True,
                            "extra_accessory": False, "identity_note": "SYNTHETIC RECHECK"})
    write_evidence(path, passed)
    # When: a changed observation for the same unchanged raw is processed.
    recheck = process(spec)
    # Then: the first measured FAIL and full observation remain immutable; the recheck is separately recorded.
    first = Processed.model_validate_json((request.raw_directory / "try1/processed.json").read_bytes())
    assert failed.structure_status == first.structure_status == "FAIL"
    assert recheck.structure_status == "PASS"
    assert first.inspection is not None and recheck.inspection is not None
    first.inspection.verify()
    assert first.inspection.sha256 != recheck.inspection.sha256
    assert first.observation is not None and first.observation.extra_accessory and not first.observation.hair_match
    assert (request.raw_directory / "try1/processed.revision002.json").is_file()


@pytest.mark.parametrize("changed", ["manifest", "source"])
def test_processing_rejects_changed_appearance_evidence(request_path: Path, tmp_path: Path, changed: str) -> None:
    # Given: a request binds both the literal observation file and its actual original source.
    request = Request.model_validate_json(request_path.read_bytes())
    source = tmp_path / "original_source.png"
    _ = source.write_bytes(request.references[0].path.read_bytes())
    manifest = tmp_path / "appearance.json"
    appearance = SourceAppearance(look=request.look, source=AssetRef.of(source), appearance_brief="SYNTHETIC closed collar")
    write_evidence(manifest, AppearanceManifest(envelopes=(), looks=(appearance,)))
    bound = request.model_copy(update={"appearance_manifest": AssetRef.of(manifest),
        "source_appearance": appearance})
    write_evidence(request_path, bound)
    modified = manifest if changed == "manifest" else source
    _ = modified.write_bytes(b"changed evidence after preparation")
    # When/Then: reject the stale appearance binding before publishing a portrait.
    with pytest.raises(ValueError, match="changed after request"):
        _ = process(ProcessInput(request_path=request_path, raw_path=request.references[0].path, attempt=1))
    assert not request.canonical_path.exists()


def test_processing_rejects_attempt_number_different_from_request(request_path: Path) -> None:
    # Given: a request is specifically for attempt1 (including legacy records with no attempt field).
    request = Request.model_validate_json(request_path.read_bytes())
    # When/Then: an explicit attempt2 request is required to consume the second attempt.
    with pytest.raises(ValueError, match="Attempt must match"):
        _ = process(ProcessInput(request_path=request_path, raw_path=request.references[0].path, attempt=2))
    assert not request.canonical_path.exists()


@pytest.mark.parametrize("changed_index", [0, 1])
def test_process_rejects_changed_linked_envelope_before_writes(request_path: Path, tmp_path: Path, changed_index: int) -> None:
    # Given: a valid request/compiled manifest and two unchanged source-review envelopes.
    request = Request.model_validate_json(request_path.read_bytes())
    envelopes = (tmp_path / "owner_envelope.json", tmp_path / "idle_envelope.json")
    for path in envelopes:
        _ = path.write_text('{"appearance_brief": "original synthetic closed collar"}')
    appearance = SourceAppearance(look=request.look, source=request.references[0], appearance_brief="original synthetic closed collar")
    manifest = tmp_path / "appearance.json"
    write_evidence(manifest, AppearanceManifest(envelopes=tuple(AssetRef.of(path) for path in envelopes), looks=(appearance,)))
    write_evidence(request_path, request.model_copy(update={"appearance_manifest": AssetRef.of(manifest), "source_appearance": appearance}))
    _ = envelopes[changed_index].write_text('{"appearance_brief": "revised synthetic high crossed collar"}')
    # When/Then: all linked envelope hashes are rechecked before raw/canonical/selected writes.
    with pytest.raises(ValueError, match="changed after request"):
        _ = process(ProcessInput(request_path=request_path, raw_path=request.references[0].path, attempt=1))
    assert not request.canonical_path.exists() and not request.raw_directory.exists()
