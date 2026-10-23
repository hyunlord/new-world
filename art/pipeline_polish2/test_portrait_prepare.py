# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic", "typer"]
# ///
# How to run: python3 -m pytest art/pipeline_polish2/test_portrait_prepare.py -q
"""Protect source appearance, exact reference scaling and separate retry requests."""
from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

import pytest
from PIL import Image

from art.compositor.face_variants import FacePlan
from art.pipeline_polish2 import portrait_prepare
from art.pipeline_polish2.portrait_appearance import compile_appearance
from art.pipeline_polish2.portrait_models import AssetRef, Request, write_evidence


@dataclass(frozen=True, slots=True)
class Workspace:
    root: Path
    data: Path
    source: Path
    sprite: Path
    envelope: Path
    brief: str


@pytest.fixture
def workspace(tmp_path: Path) -> Workspace:
    plan = FacePlan.read()
    look = plan.looks[0]
    root = tmp_path / "repo"
    plan = plan.model_copy(update={"looks": (look,), "fallbacks": {}, "owner_sample": (look.look,)})
    write_evidence(root / "art/character/FACE_VARIANTS.json", plan)
    source = root / "art/character/route_i" / look.look / "S/idle.png"
    sprite = source.parent.parent / "v1/S/idle.png"
    source.parent.mkdir(parents=True)
    sprite.parent.mkdir(parents=True)
    pixels = Image.new("RGBA", (3, 2), "#68472a")
    pixels.putpixel((0, 0), (12, 34, 56, 0))
    pixels.putpixel((2, 1), (200, 180, 160, 127))
    pixels.save(source)
    pixels.save(sprite)
    anchor = root / "art/style05/anchors/portrait.png"
    anchor.parent.mkdir(parents=True)
    Image.new("RGB", (128, 128), "#a08060").save(anchor)
    brief = "Observed asymmetric folded collar closes high across both shoulders; the gathered hair tail is occluded."
    envelope = tmp_path / "source_envelope.json"
    _ = envelope.write_text(json.dumps({"frames": [{"look": look.look,
        "source": str(source.relative_to(root)), "source_sha256": AssetRef.of(source).sha256,
        "appearance_brief": brief, "unrelated_annotation": "retained in source report"}]}))
    return Workspace(root, tmp_path / "data", source, sprite, envelope, brief)


def test_preparation_carries_observation_and_exact_fourfold_pixels(workspace: Workspace) -> None:
    # Given: an observed source brief with details that cannot be inferred from a look name.
    _ = compile_appearance(workspace.root, (workspace.envelope,))
    original = workspace.sprite.read_bytes()
    # When: prepare the actual available-only generation queue.
    prepared = portrait_prepare.prepare(workspace.root, workspace.data, available_only=True)
    # Then: both original native pixels and exact nearest-neighbor RGBA expansion are preserved.
    request = prepared.requests[0]
    assert workspace.brief in request.prompt and request.source_appearance is not None
    assert request.source_appearance.appearance_brief == workspace.brief
    assert request.references[0].path == workspace.sprite and len(request.references) == 3
    assert workspace.sprite.read_bytes() == original
    with Image.open(workspace.sprite) as native, Image.open(request.references[2].path) as enlarged:
        assert enlarged.mode == native.mode and enlarged.size == (12, 8)
        assert enlarged.tobytes() == native.resize((12, 8), Image.Resampling.NEAREST).tobytes()
    assert request.appearance_manifest is not None
    request.appearance_manifest.verify()


def test_appearance_import_rejects_changed_original_source(workspace: Workspace) -> None:
    # Given: the actual original changed after its independent observation was recorded.
    Image.new("RGBA", (3, 2), "#abcdef").save(workspace.source)
    # When/Then: no brief may be presented as belonging to different source bytes.
    with pytest.raises(ValueError, match="changed after request"):
        _ = compile_appearance(workspace.root, (workspace.envelope,))


def test_retry_preserves_original_request_and_attempt(workspace: Workspace) -> None:
    # Given: a completed legacy two-reference request and preserved first raw.
    _ = compile_appearance(workspace.root, (workspace.envelope,))
    first = portrait_prepare.prepare(workspace.root, workspace.data, available_only=True).requests[0]
    first = first.model_copy(update={"references": first.references[:2], "prompt": "Original immutable prompt"})
    path = first.raw_directory / "request.json"
    write_evidence(path, first)
    raw = first.raw_directory / "try1.png"
    _ = raw.write_bytes(b"synthetic first raw evidence")
    before, raw_before = path.read_bytes(), raw.read_bytes()
    Image.new("RGBA", (3, 2), "#978654").save(workspace.sprite)
    # When: prepare one explicit correction with the current tinted source.
    retry = portrait_prepare.prepare_retry(path, "Restore the observed closed collar and gathered hairstyle.", workspace.root)
    # Then: the old request/raw remain intact and the retry binds current references and the same plan.
    saved = Request.model_validate_json((first.raw_directory / "try2.request.json").read_bytes())
    assert saved == retry and saved.attempt == 2 and len(saved.references) == 3
    assert saved.references[0] == AssetRef.of(workspace.sprite) and saved.face_plan == first.face_plan
    assert path.read_bytes() == before and raw.read_bytes() == raw_before
    assert saved.previous_request == AssetRef.of(path)
    assert portrait_prepare.prepare_retry(path, saved.correction or "", workspace.root) == retry
    with pytest.raises(ValueError, match="Conflicting frozen evidence"):
        _ = portrait_prepare.prepare_retry(path, "Different correction must not overwrite the retry.", workspace.root)
    with pytest.raises(ValueError, match="first-attempt request"):
        _ = portrait_prepare.prepare_retry(first.raw_directory / "try2.request.json", "No third attempt.", workspace.root)


def test_unstarted_queue_preserves_completed_request(workspace: Workspace) -> None:
    # Given: one completed variant and another newly available variant.
    _ = compile_appearance(workspace.root, (workspace.envelope,))
    first = portrait_prepare.prepare(workspace.root, workspace.data, available_only=True).requests[0]
    request_path = first.raw_directory / "request.json"
    before = request_path.read_bytes()
    _ = (first.raw_directory / "try1.png").write_bytes(b"synthetic generated evidence")
    second = workspace.sprite.parent.parent.parent / "v2/S/idle.png"
    second.parent.mkdir(parents=True)
    _ = second.write_bytes(workspace.sprite.read_bytes())
    # When: resume preparation for only unstarted variants.
    result = portrait_prepare.prepare(workspace.root, workspace.data, available_only=True, unstarted_only=True)
    # Then: queue counts remain explicitly partial and the completed request is byte-identical.
    assert tuple(row.variant for row in result.requests) == ("v2",)
    assert result.expected_count == 6 and result.available_count == 1 and not result.complete
    assert result.skipped_started_count == 1 and request_path.read_bytes() == before


def test_legacy_two_reference_request_defaults_to_first_attempt(workspace: Workspace) -> None:
    # Given: the old JSON shape has two references and no appearance or attempt fields.
    source = AssetRef.of(workspace.sprite)
    legacy = {"look": "synthetic", "variant": "v1", "description": "original", "prompt": "original",
        "references": [source.model_dump(mode="json"), source.model_dump(mode="json")],
        "face_plan": source.model_dump(mode="json"), "canonical_path": str(workspace.data / "portrait.png"),
        "raw_directory": str(workspace.data)}
    # When: parse historical request bytes with the updated request model.
    request = Request.model_validate_json(json.dumps(legacy))
    # Then: legacy inputs remain explicitly first-attempt records without fabricated source observations.
    assert request.attempt == 1 and len(request.references) == 2
    assert request.appearance_manifest is None and request.source_appearance is None


@pytest.mark.parametrize("changed_index", [0, 1])
def test_prepare_rejects_changed_envelope_before_writes(workspace: Workspace, changed_index: int) -> None:
    # Given: both source-review envelopes were hash-bound when the manifest was compiled.
    other = workspace.envelope.with_name("other_envelope.json")
    _ = other.write_text('{"frames": [], "review": "original synthetic review"}')
    envelopes = (workspace.envelope, other)
    _ = compile_appearance(workspace.root, envelopes)
    changed = envelopes[changed_index]
    text = changed.read_text().replace(workspace.brief, "Revised observed collar").replace("original synthetic", "revised synthetic")
    _ = changed.write_text(text)
    # When/Then: a stale envelope, even one containing another cohort, blocks all request/reference writes.
    with pytest.raises(ValueError, match="changed after request"):
        _ = portrait_prepare.prepare(workspace.root, workspace.data, available_only=True)
    assert not workspace.data.exists()
