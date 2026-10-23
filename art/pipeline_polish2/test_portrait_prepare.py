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
from art.pipeline_polish2.portrait_models import (
    AssetRef,
    BustDetail,
    Request,
    ToolRequest,
    write_evidence,
)


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
    assert request.references[0].path == workspace.sprite and len(request.references) == 4
    assert request.bust_detail is not None
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
    first = first.model_copy(update={"references": first.references[:2], "bust_detail": None, "prompt": "Original immutable prompt"})
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


def test_unstarted_upgrade_preserves_first_three_references(workspace: Workspace) -> None:
    # Given: an unstarted three-reference request already has frozen native/style/exact4x references.
    _ = compile_appearance(workspace.root, (workspace.envelope,))
    first = portrait_prepare.prepare(workspace.root, workspace.data, available_only=True).requests[0]
    old = first.model_copy(update={"references": first.references[:3], "bust_detail": None})
    write_evidence(first.raw_directory / "request.json", old)
    before = tuple(ref.path.read_bytes() for ref in old.references)
    # When: upgrade only the unstarted queue to the four-reference format.
    current = portrait_prepare.prepare(workspace.root, workspace.data, available_only=True, unstarted_only=True).requests[0]
    # Then: the original three references are unchanged, with one separately bound detail appended.
    assert current.references[:3] == old.references and len(current.references) == 4
    assert tuple(ref.path.read_bytes() for ref in current.references[:3]) == before
    assert current.bust_detail is not None


def test_bad_detail_blocks_preparation_before_reference_writes(workspace: Workspace) -> None:
    # Given: a stored detail has forged crop metadata and an unrelated reference file needs rebuilding.
    _ = compile_appearance(workspace.root, (workspace.envelope,))
    first = portrait_prepare.prepare(workspace.root, workspace.data, available_only=True).requests[0]
    assert first.bust_detail is not None
    detail = BustDetail.model_validate_json(first.bust_detail.path.read_bytes())
    write_evidence(first.bust_detail.path, detail.model_copy(update={"crop_bbox": (0, 0, 1, 1)}))
    first.references[2].path.unlink()
    path = first.raw_directory / "request.json"
    before = path.read_bytes()
    # When/Then: reject the invalid detail before recreating any reference or replacing the request.
    with pytest.raises(ValueError):
        _ = portrait_prepare.prepare(workspace.root, workspace.data, available_only=True, unstarted_only=True)
    assert not first.references[2].path.exists() and path.read_bytes() == before


@pytest.mark.parametrize("legacy_count", [2, 3])
def test_retry_detail_requires_explicit_opt_in_and_keeps_history(workspace: Workspace, legacy_count: int) -> None:
    # Given: a completed legacy request and raw evidence remain unchanged at their original paths.
    _ = compile_appearance(workspace.root, (workspace.envelope,))
    first = portrait_prepare.prepare(workspace.root, workspace.data, available_only=True).requests[0]
    legacy = first.model_copy(update={"references": first.references[:legacy_count], "bust_detail": None})
    path = first.raw_directory / "request.json"
    write_evidence(path, legacy)
    raw = first.raw_directory / "try1.png"
    _ = raw.write_bytes(b"SYNTHETIC original first-attempt evidence")
    before, raw_before = path.read_bytes(), raw.read_bytes()
    # When: explicitly request additional source detail for the one eligible second attempt.
    retry = portrait_prepare.prepare_retry(path, "Preserve the observed high collar.", workspace.root, include_bust_detail=True)
    # Then: the separate second request is bound to four references without modifying the first history.
    assert retry.attempt == 2 and len(retry.references) == 4 and retry.bust_detail is not None
    assert retry.references[0] == legacy.references[0] and retry.face_plan == legacy.face_plan
    assert path.read_bytes() == before and raw.read_bytes() == raw_before
    retry_path = first.raw_directory / "try2.request.json"
    retry_bytes = retry_path.read_bytes()
    with pytest.raises(ValueError, match="Conflicting frozen evidence"):
        _ = portrait_prepare.prepare_retry(path, "Preserve the observed high collar.", workspace.root)
    assert retry_path.read_bytes() == retry_bytes


@pytest.fixture
def completed_request(workspace: Workspace) -> Request:
    _ = compile_appearance(workspace.root, (workspace.envelope,))
    look = FacePlan.read(workspace.root / "art/character/FACE_VARIANTS.json").looks[0]
    request = portrait_prepare.build_request(workspace.root, workspace.data / "generation" / look.look / "v1",
        (look, look.variants[0]), include_bust_detail=False)
    write_evidence(request.raw_directory / "request.json", request)
    _ = (request.raw_directory / "try1.png").write_bytes(b"SYNTHETIC preserved first raw")
    return request


@pytest.mark.parametrize("include_detail", [False, True])
def test_retry_full_prompt_override_is_exact(completed_request: Request, workspace: Workspace, include_detail: bool) -> None:
    # Given: an eligible first attempt and a complete prompt with intentional Unicode/newlines/spacing.
    first = completed_request
    path = first.raw_directory / "request.json"
    before = path.read_bytes()
    raw_before = (first.raw_directory / "try1.png").read_bytes()
    override = "  Complete concise prompt.\r\n목선과 어깨를 보존.\n "
    # When: first-create the second request using the explicit full prompt.
    retry = portrait_prepare.prepare_retry(path, "Observed neckline was too low.", workspace.root,
        include_bust_detail=include_detail, prompt_override=override)
    # Then: the request and queue carry exactly that text while original provenance remains unchanged.
    queue = ToolRequest.model_validate_json((first.raw_directory / "try2.queue.json").read_bytes())
    assert retry.prompt == queue.prompt == override and retry.correction == "Observed neckline was too low."
    assert retry.references[:3] == first.references and retry.face_plan == first.face_plan
    assert path.read_bytes() == before and (first.raw_directory / "try1.png").read_bytes() == raw_before


@pytest.mark.parametrize("override", ["", " \t\r\n"])
def test_retry_rejects_blank_full_prompt(completed_request: Request, workspace: Workspace, override: str) -> None:
    # Given: eligible inputs but an explicitly empty replacement prompt.
    path = completed_request.raw_directory / "request.json"
    # When/Then: reject before creating retry evidence.
    with pytest.raises(ValueError, match="nonblank full prompt"):
        _ = portrait_prepare.prepare_retry(path, "Actual observed reason.", workspace.root, prompt_override=override)
    assert not (completed_request.raw_directory / "try2.request.json").exists()


def test_retry_none_override_keeps_default_bytes(completed_request: Request, workspace: Workspace) -> None:
    # Given: the normal generated correction prefix and unchanged full prompt.
    path = completed_request.raw_directory / "request.json"
    retry = portrait_prepare.prepare_retry(path, "Observed reason.", workspace.root)
    retry_path = completed_request.raw_directory / "try2.request.json"
    before = retry_path.read_bytes()
    # When: repeat with the explicit default None.
    repeated = portrait_prepare.prepare_retry(path, "Observed reason.", workspace.root, prompt_override=None)
    # Then: the established default prompt and sealed bytes are unchanged.
    assert repeated == retry and retry_path.read_bytes() == before
    assert retry.prompt == f"Second and final attempt. Explicit correction: Observed reason.\n{completed_request.prompt}"


@pytest.mark.parametrize("reference_state", ["present", "missing", "changed", "orphan_queue"])
def test_override_conflict_leaves_entire_workspace_unchanged(completed_request: Request, workspace: Workspace, reference_state: str) -> None:
    # Given: a sealed default three-reference retry, optionally with a missing/changed old reference.
    path = completed_request.raw_directory / "request.json"
    retry = portrait_prepare.prepare_retry(path, "Observed reason.", workspace.root)
    if reference_state == "missing":
        retry.references[2].path.unlink()
    if reference_state == "changed":
        _ = retry.references[2].path.write_bytes(b"changed synthetic reference")
    if reference_state == "orphan_queue":
        (retry.raw_directory / "try2.request.json").unlink()
    before = {str(p): p.read_bytes() for p in workspace.root.parent.rglob("*") if p.is_file()}
    # When/Then: a different prompt plus newly requested detail rejects before any file recovery/creation.
    with pytest.raises(ValueError):
        _ = portrait_prepare.prepare_retry(path, "Observed reason.", workspace.root,
            include_bust_detail=True, prompt_override="Different complete prompt.")
    assert {str(p): p.read_bytes() for p in workspace.root.parent.rglob("*") if p.is_file()} == before
