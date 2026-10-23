# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic", "typer", "numpy", "scipy", "scikit-image"]
# ///
# How to run: uv run --with pytest --with pillow --with pydantic --with typer --with numpy --with scipy --with scikit-image python -m pytest art/pipeline_polish2/test_portrait_previous.py -q
"""Bind optional edit input to immutable first-attempt evidence, never current selection."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import pytest
from PIL import Image

from art.compositor.face_variants import FacePlan
from art.pipeline_polish2 import portrait_prepare
from art.pipeline_polish2.portrait_appearance import (
    AppearanceManifest,
    create_bust_detail,
    enlarge_reference,
)
from art.pipeline_polish2.portrait_models import (
    AssetRef,
    BustDetail,
    Request,
    SourceAppearance,
    ToolRequest,
    write_evidence,
)
from art.pipeline_polish2.portrait_process import Processed, ProcessInput, process


@dataclass(frozen=True, slots=True)
class FirstAttempt:
    root: Path
    path: Path
    request: Request
    processed: Processed


def make_first_attempt(tmp_path: Path, raw_color: str = "#a08060") -> FirstAttempt:
    root = tmp_path / "repo"
    plan = FacePlan.read()
    look = plan.looks[0]
    plan = plan.model_copy(update={"looks": (look,), "owner_sample": (look.look,), "fallbacks": {}})
    write_evidence(root / "art/character/FACE_VARIANTS.json", plan)
    native = root / "art/character/route_i" / look.look / "v1/S/idle.png"
    native.parent.mkdir(parents=True)
    Image.new("RGBA", (8, 12), (70, 80, 90, 127)).save(native)
    original = native.parent.parent.parent / "S/idle.png"
    original.parent.mkdir(parents=True)
    _ = original.write_bytes(native.read_bytes())
    write_evidence(root / "art/character/source_appearance.json", AppearanceManifest(envelopes=(),
        looks=(SourceAppearance(look=look.look, source=AssetRef.of(original), appearance_brief="SYNTHETIC high collar"),)))
    anchor = root / "art/style05/anchors/portrait.png"
    anchor.parent.mkdir(parents=True)
    Image.new("RGB", (256, 256), raw_color).save(anchor)
    request = portrait_prepare.build_request(root, tmp_path / "data/generation" / look.look / "v1",
        (look, look.variants[0]), include_bust_detail=False)
    path = request.raw_directory / "request.json"
    write_evidence(path, request)
    raw = request.raw_directory / "try1.png"
    _ = raw.write_bytes(anchor.read_bytes())
    record = process(ProcessInput(request_path=path, raw_path=raw, attempt=1))
    return FirstAttempt(root, path, request, record)


@pytest.fixture
def first(tmp_path: Path) -> FirstAttempt:
    return make_first_attempt(tmp_path)


def test_previous_portrait_is_exact_preserved_raw_and_forces_detail(first: FirstAttempt) -> None:
    # Given: mutable selection is unusable, but the immutable first record/raw/request are intact.
    _ = (first.request.raw_directory / "selected.json").write_text("Do not use this mutable pointer")
    before = first.path.read_bytes()
    # When: opt into the actual first portrait as an edit reference.
    retry = portrait_prepare.prepare_retry(first.path, "Observed collar drift.", first.root, use_previous_portrait=True)
    # Then: reference5 is the first preserved raw and reference4 remains the verified source detail.
    assert retry.attempt == 2 and len(retry.references) == 5 and retry.bust_detail is not None
    assert retry.references[4] == first.processed.raw and retry.previous_portrait is not None
    assert retry.previous_portrait.processed == AssetRef.of(first.request.raw_directory / "try1/processed.json")
    assert retry.previous_portrait.request == first.processed.request and retry.previous_portrait.raw == first.processed.raw
    assert retry.references[:3] == first.request.references and first.path.read_bytes() == before
    queue = ToolRequest.model_validate_json((retry.raw_directory / "try2.queue.json").read_bytes())
    assert queue.references[4] == first.processed.raw and queue.previous_portrait == retry.previous_portrait


@pytest.mark.parametrize("tamper", ["raw", "request", "record", "foreign", "replacement", "native"])
def test_prepare_rejects_first_evidence_tamper_before_writes(first: FirstAttempt, tamper: str) -> None:
    # Given: one part of the first-attempt provenance has been changed before opting into an edit input.
    record_path = first.request.raw_directory / "try1/processed.json"
    if tamper == "raw":
        _ = first.processed.raw.path.write_bytes(b"changed raw")
    if tamper == "request":
        write_evidence(first.path, first.request.model_copy(update={"prompt": "changed first prompt"}))
    if tamper == "record":
        write_evidence(record_path, first.processed.model_copy(update={"candidate": first.processed.candidate.model_copy(update={"attempt": 2})}))
    if tamper == "foreign":
        foreign = first.request.raw_directory / "foreign.png"
        _ = foreign.write_bytes(first.processed.raw.path.read_bytes())
        write_evidence(record_path, first.processed.model_copy(update={"raw": AssetRef.of(foreign)}))
    if tamper == "replacement":
        Image.new("RGB", (256, 256), "#223344").save(first.processed.raw.path)
        write_evidence(record_path, first.processed.model_copy(update={"raw": AssetRef.of(first.processed.raw.path)}))
    if tamper == "native":
        Image.new("RGBA", (8, 12), (10, 20, 30, 127)).save(first.request.references[0].path)
    before = {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()}
    # When/Then: fail before creating detail/queue/request files from unauthenticated first evidence.
    with pytest.raises(ValueError):
        _ = portrait_prepare.prepare_retry(first.path, "Observed collar drift.", first.root, use_previous_portrait=True)
    assert {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()} == before


def test_second_processing_keeps_first_snapshot_and_native_identity_reference(first: FirstAttempt) -> None:
    # Given: a five-reference retry prepared from immutable first evidence.
    retry = portrait_prepare.prepare_retry(first.path, "Observed collar drift.", first.root, use_previous_portrait=True)
    before = {p.name: p.read_bytes() for p in (first.request.raw_directory / "try1").iterdir() if p.is_file()}
    # When: process the second result through the actual pipeline.
    result = process(ProcessInput(request_path=retry.raw_directory / "try2.request.json", raw_path=first.processed.raw.path, attempt=2))
    # Then: the native identity source stays ref0 and every first-attempt artifact remains byte-identical.
    assert result.candidate.attempt == 2 and result.candidate.sprite == retry.references[0]
    assert {p.name: p.read_bytes() for p in (first.request.raw_directory / "try1").iterdir() if p.is_file()} == before


@pytest.mark.parametrize("tamper", ["raw", "record", "fifth"])
def test_process_rejects_stale_previous_binding_before_writes(first: FirstAttempt, tamper: str) -> None:
    # Given: the fifth reference was frozen, then a linked artifact or the displayed fifth slot changed.
    retry = portrait_prepare.prepare_retry(first.path, "Observed collar drift.", first.root, use_previous_portrait=True)
    retry_path = retry.raw_directory / "try2.request.json"
    if tamper == "raw":
        _ = first.processed.raw.path.write_bytes(b"changed after retry freeze")
    if tamper == "record":
        _ = (first.request.raw_directory / "try1/processed.json").write_text("changed record")
    if tamper == "fifth":
        write_evidence(retry_path, retry.model_copy(update={"references": (*retry.references[:4], retry.references[1])}))
    before = {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()}
    # When/Then: no candidate/result/history writes occur from a mismatched edit reference.
    with pytest.raises(ValueError):
        _ = process(ProcessInput(request_path=retry_path, raw_path=first.request.raw_directory / "try1.png", attempt=2))
    assert {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()} == before


def test_sealed_retry_mode_change_rejects_without_creating_detail(first: FirstAttempt) -> None:
    # Given: a sealed ordinary retry and no previous-portrait binding.
    ordinary = portrait_prepare.prepare_retry(first.path, "Observed collar drift.", first.root)
    before = {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()}
    # When/Then: changing its mode to five-reference editing rejects without extra reference creation.
    with pytest.raises(ValueError, match="Conflicting frozen evidence"):
        _ = portrait_prepare.prepare_retry(first.path, "Observed collar drift.", first.root, use_previous_portrait=True)
    assert ordinary.previous_portrait is None and "previous_portrait" not in ordinary.model_dump(mode="json")
    assert {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()} == before


def test_fifth_reference_keeps_full_override_exact_on_identical_repeat(first: FirstAttempt) -> None:
    # Given: a complete edit prompt is authoritative, and its eligible fifth-reference retry is sealed.
    override = "  Edit portrait five.\r\n실제 목선을 보존.\n"
    retry = portrait_prepare.prepare_retry(first.path, "Observed collar drift.", first.root,
        use_previous_portrait=True, prompt_override=override)
    before = {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()}
    # When: repeat exactly the same edit mode, correction and complete prompt.
    repeated = portrait_prepare.prepare_retry(first.path, "Observed collar drift.", first.root,
        use_previous_portrait=True, prompt_override=override)
    # Then: no edit preface is added to the full override and all sealed bytes remain unchanged.
    assert retry.prompt == repeated.prompt == override and repeated == retry
    assert {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()} == before


def test_fifth_reference_cannot_label_a_first_or_third_attempt(first: FirstAttempt) -> None:
    # Given: a valid second-attempt edit request.
    retry = portrait_prepare.prepare_retry(first.path, "Observed collar drift.", first.root, use_previous_portrait=True)
    # When/Then: changing the attempt label cannot bypass the second-attempt-only binding.
    for attempt in (1, 3):
        with pytest.raises(ValueError):
            _ = Request.model_validate_json(retry.model_copy(update={"attempt": attempt}).model_dump_json())


def test_complete_foreign_previous_chain_cannot_publish_target_retry(first: FirstAttempt) -> None:
    # Given: two valid immutable first histories, with different directories and portrait bytes.
    foreign = make_first_attempt(first.root.parent / "foreign", "#204060")
    target = portrait_prepare.prepare_retry(first.path, "Observed drift.", first.root, use_previous_portrait=True)
    foreign_retry = portrait_prepare.prepare_retry(foreign.path, "Observed drift.", foreign.root, use_previous_portrait=True)
    injected = target.model_copy(update={"previous_request": AssetRef.of(foreign.path),
        "previous_portrait": foreign_retry.previous_portrait, "references": (*target.references[:4], foreign.processed.raw)})
    path = target.raw_directory / "try2.request.json"
    write_evidence(path, injected)
    before = {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()}
    # When/Then: an internally valid foreign chain cannot authorize writes into the target candidate.
    with pytest.raises(ValueError):
        _ = process(ProcessInput(request_path=path, raw_path=first.request.raw_directory / "try1.png", attempt=2))
    assert {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()} == before


@pytest.mark.parametrize("field", ["look", "variant", "raw_directory", "canonical_path", "face_plan", "native"])
def test_previous_chain_must_match_consuming_retry_context(first: FirstAttempt, field: str) -> None:
    # Given: a valid edit request whose outer identity/output/plan/input context is changed independently.
    retry = portrait_prepare.prepare_retry(first.path, "Observed drift.", first.root, use_previous_portrait=True)
    if field == "look":
        retry = retry.model_copy(update={"look": "another_look"})
    if field == "variant":
        retry = retry.model_copy(update={"variant": "v2"})
    if field == "raw_directory":
        retry = retry.model_copy(update={"raw_directory": retry.raw_directory / "foreign"})
    if field == "canonical_path":
        retry = retry.model_copy(update={"canonical_path": retry.canonical_path.with_name("foreign.png")})
    if field == "face_plan":
        plan = first.root.parent / "changed_plan.json"
        _ = plan.write_bytes(first.request.face_plan.path.read_bytes() + b"\n")
        retry = retry.model_copy(update={"face_plan": AssetRef.of(plan)})
    if field == "native":
        image = first.root.parent / "changed_native.png"
        Image.new("RGBA", (8, 12), (10, 20, 30, 127)).save(image)
        native = AssetRef.of(image)
        directory = first.root.parent / "foreign_references"
        detail = create_bust_detail(native, directory)
        detail_image = BustDetail.model_validate_json(detail.path.read_bytes()).image
        retry = retry.model_copy(update={"bust_detail": detail,
            "references": (native, retry.references[1], enlarge_reference(native, directory), detail_image, retry.references[4])})
    path = first.request.raw_directory / "try2.request.json"
    write_evidence(path, retry)
    before = {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()}
    # When/Then: internal chain validity alone cannot authorize changed consuming context.
    with pytest.raises(ValueError):
        _ = process(ProcessInput(request_path=path, raw_path=first.request.raw_directory / "try1.png", attempt=2))
    assert {str(p): p.read_bytes() for p in first.root.parent.rglob("*") if p.is_file()} == before
