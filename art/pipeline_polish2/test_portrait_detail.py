# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic", "typer", "numpy", "scipy", "scikit-image"]
# ///
# How to run: uv run --with pytest --with pillow --with pydantic --with typer --with numpy --with scipy --with scikit-image python -m pytest art/pipeline_polish2/test_portrait_detail.py -q
"""Verify deterministic bust details, provenance rejection and retained failed attempts."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import pytest
from PIL import Image
from pydantic import ValidationError

from art.pipeline_env1.portrait_finalize import Inspection
from art.pipeline_polish2 import portrait_appearance
from art.pipeline_polish2.portrait_models import (
    AssetRef,
    BustDetail,
    Request,
    write_evidence,
)
from art.pipeline_polish2.portrait_process import ProcessInput, process


@pytest.fixture
def native(tmp_path: Path) -> AssetRef:
    image = Image.new("RGBA", (7, 9), (99, 88, 77, 0))
    for y in range(2, 9):
        for x in range(1, 6):
            image.putpixel((x, y), (x * 31, y * 23, x + y, 127 if x == 3 else 255))
    image.putpixel((2, 3), (23, 45, 67, 0))
    path = tmp_path / "native.png"
    image.save(path)
    return AssetRef.of(path)


def test_detail_preserves_every_rgba_pixel_in_explicit_upper_crop(native: AssetRef, tmp_path: Path) -> None:
    # Given: a non-square visible box, partial alpha and hidden RGB inside the intended crop.
    original = native.path.read_bytes()
    # When: derive and verify the new reference from the actual native source.
    binding = portrait_appearance.create_bust_detail(native, tmp_path / "references")
    detail = BustDetail.model_validate_json(binding.path.read_bytes())
    verified = portrait_appearance.verify_bust_detail(binding, native, detail.image)
    # Then: ceil(7*3/4)=6 source rows become exact12x12 RGBA blocks, including transparent RGB.
    assert verified.visible_bbox == (1, 2, 6, 9) and verified.crop_bbox == (1, 2, 6, 8)
    assert verified.scale == 12 and verified.source == native
    assert native.sha256 in binding.path.name and native.sha256 in detail.image.path.name
    assert native.path.read_bytes() == original
    with Image.open(native.path) as source, Image.open(detail.image.path) as output:
        assert output.mode == "RGBA" and output.size == (60, 72)
        assert all(output.getpixel((x, y)) == source.getpixel((1 + x // 12, 2 + y // 12))
                   for y in range(72) for x in range(60))


@pytest.mark.parametrize("corruption", ["source", "painted", "crop", "visible", "scale"])
def test_detail_rejects_stale_or_rehashed_forged_derivation(native: AssetRef, tmp_path: Path, corruption: str) -> None:
    # Given: a valid detail, then a source change or a forged output/metadata with refreshed hashes.
    binding = portrait_appearance.create_bust_detail(native, tmp_path / "references")
    detail = BustDetail.model_validate_json(binding.path.read_bytes())
    if corruption == "source":
        _ = native.path.write_bytes(b"changed native bytes")
    if corruption == "painted":
        with Image.open(detail.image.path) as image:
            image.putpixel((0, 0), (0, 0, 0, 255))
            image.save(detail.image.path)
        detail = detail.model_copy(update={"image": AssetRef.of(detail.image.path)})
    if corruption == "crop":
        detail = detail.model_copy(update={"crop_bbox": (1, 2, 6, 7)})
    if corruption == "visible":
        detail = detail.model_copy(update={"visible_bbox": (0, 0, 7, 9)})
    if corruption == "scale":
        detail = detail.model_copy(update={"scale": 11})
    write_evidence(binding.path, detail)
    rebound = AssetRef.of(binding.path)
    # When/Then: checking derivation rejects even a newly hash-bound forged sidecar/image.
    with pytest.raises((ValueError, ValidationError)):
        _ = portrait_appearance.verify_bust_detail(rebound, native, detail.image)


def test_existing_painted_detail_rejected_before_preparation_writes(native: AssetRef, tmp_path: Path) -> None:
    # Given: a previously prepared detail has been painted, with all sidecar hashes refreshed.
    directory = tmp_path / "references"
    binding = portrait_appearance.create_bust_detail(native, directory)
    detail = BustDetail.model_validate_json(binding.path.read_bytes())
    with Image.open(detail.image.path) as image:
        image.putpixel((0, 0), (1, 2, 3, 4))
        image.save(detail.image.path)
    write_evidence(binding.path, detail.model_copy(update={"image": AssetRef.of(detail.image.path)}))
    before = {path.name: path.read_bytes() for path in directory.iterdir()}
    # When/Then: preparation rejects the existing corruption without rewriting either artifact.
    with pytest.raises(ValueError):
        _ = portrait_appearance.create_bust_detail(native, directory)
    assert {path.name: path.read_bytes() for path in directory.iterdir()} == before


@dataclass(frozen=True, slots=True)
class DetailRequest:
    path: Path
    request: Request
    raw: Path


@pytest.fixture
def detail_request(native: AssetRef, tmp_path: Path) -> DetailRequest:
    binding = portrait_appearance.create_bust_detail(native, tmp_path / "references")
    detail = BustDetail.model_validate_json(binding.path.read_bytes())
    raw = tmp_path / "synthetic_raw.png"
    Image.new("RGB", (256, 256), "#a08060").save(raw)
    anchor = AssetRef.of(raw)
    request = Request(look="synthetic_look", variant="v1", description="SYNTHETIC", prompt="synthetic",
        references=(native, anchor, portrait_appearance.enlarge_reference(native, tmp_path / "references"), detail.image),
        face_plan=anchor, bust_detail=binding, canonical_path=tmp_path / "canonical.png", raw_directory=tmp_path / "generation")
    path = tmp_path / "request.json"
    write_evidence(path, request)
    return DetailRequest(path, request, raw)


def test_processing_keeps_native_sprite_binding(detail_request: DetailRequest) -> None:
    # Given: a valid four-reference request.
    # When: process through the actual normalization/numeric-report path.
    result = process(ProcessInput(request_path=detail_request.path, raw_path=detail_request.raw, attempt=1))
    # Then: the new display detail never replaces the native sprite used by blind identity packets.
    assert result.candidate.sprite == detail_request.request.references[0]


def test_invalid_detail_preserves_original_failed_processing(detail_request: DetailRequest, tmp_path: Path) -> None:
    # Given: a real prior processed FAIL and immutable raw/observation evidence.
    request = detail_request.request
    observation = Inspection(look="synthetic_look/v1", attempt=1, head_bbox=(0, 0, 128, 128),
        face_bbox=(0, 0, 128, 128), head_uncropped=False, hair_match=False, top_match=False,
        extra_accessory=False, identity_note="SYNTHETIC FAIL", inspection_method="Synthetic regression fixture")
    inspection = tmp_path / "inspection.json"
    write_evidence(inspection, observation)
    spec = ProcessInput(request_path=detail_request.path, raw_path=detail_request.raw, attempt=1, inspection_path=inspection)
    failed = process(spec)
    before = {str(path.relative_to(request.raw_directory)): path.read_bytes()
              for path in request.raw_directory.rglob("*") if path.is_file()}
    assert request.bust_detail is not None
    detail = BustDetail.model_validate_json(request.bust_detail.path.read_bytes())
    write_evidence(request.bust_detail.path, detail.model_copy(update={"crop_bbox": (1, 2, 6, 7)}))
    write_evidence(detail_request.path, request.model_copy(update={"bust_detail": AssetRef.of(request.bust_detail.path)}))
    # When/Then: a recheck rejects forged derivation before replacing the original failed processing history.
    with pytest.raises(ValueError):
        _ = process(spec)
    assert failed.structure_status == "FAIL"
    assert {str(path.relative_to(request.raw_directory)): path.read_bytes()
            for path in request.raw_directory.rglob("*") if path.is_file()} == before


@pytest.mark.parametrize("reference_count", [2, 3])
def test_legacy_reference_counts_need_no_detail(native: AssetRef, tmp_path: Path, reference_count: Literal[2, 3]) -> None:
    # Given: a historical request carrying two or three references and no new sidecar.
    request = Request(look="synthetic", variant="v1", description="old", prompt="old",
        references=(native,) * reference_count, face_plan=native,
        canonical_path=tmp_path / "old.png", raw_directory=tmp_path / "old")
    # When: round-trip historical records through the updated parser.
    parsed = Request.model_validate_json(request.model_dump_json())
    # Then: historical requests remain valid without fabricating detail metadata.
    assert parsed.bust_detail is None and len(parsed.references) == reference_count
