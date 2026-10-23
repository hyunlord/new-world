# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "typer"]
# ///
# How to run: python3 -m art.pipeline_polish2.portrait_appearance
"""Import source-observed appearance and preserve exact enlarged sprite references."""
from __future__ import annotations

from hashlib import sha256
from io import BytesIO
from pathlib import Path
from typing import ClassVar, Literal, Self

import typer
from PIL import Image
from pydantic import BaseModel, ConfigDict, Field, model_validator

from art.compositor.face_variants import FacePlan
from art.pipeline_polish2.blind_round import seal, seal_model
from art.pipeline_polish2.portrait_models import (
    ROOT,
    AssetRef,
    BustDetail,
    Evidence,
    SourceAppearance,
    write_evidence,
)


class SourceFrame(BaseModel):
    """Project the observed appearance fields from the larger annotation document."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    look: str
    source: Path
    source_sha256: str
    appearance_brief: str = Field(min_length=1)


class SourceEnvelope(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    frames: tuple[SourceFrame, ...]


class AppearanceManifest(Evidence):
    schema_version: Literal[1] = 1
    envelopes: tuple[AssetRef, ...]
    looks: tuple[SourceAppearance, ...]

    @model_validator(mode="after")
    def unique(self) -> Self:
        if len({row.look for row in self.looks}) != len(self.looks):
            message = "Source appearance must have exactly one observation per look"
            raise ValueError(message)
        return self


def compile_appearance(root: Path, envelopes: tuple[Path, ...]) -> AppearanceManifest:
    """Require one literal, hash-verified original-source observation for every accepted look."""
    rows: list[SourceAppearance] = []
    for envelope in envelopes:
        parsed = SourceEnvelope.model_validate_json(envelope.read_bytes())
        for frame in parsed.frames:
            expected = root / "art/character/route_i" / frame.look / "S/idle.png"
            source = AssetRef(path=(root / frame.source).resolve(), sha256=frame.source_sha256)
            if source.path != expected.resolve():
                message = f"Appearance must describe the original S/idle source: {frame.look}"
                raise ValueError(message)
            source.verify()
            rows.append(SourceAppearance(look=frame.look, source=source, appearance_brief=frame.appearance_brief))
    result = AppearanceManifest(envelopes=tuple(AssetRef.of(path) for path in envelopes),
                                looks=tuple(sorted(rows, key=lambda row: row.look)))
    plan = FacePlan.read(root / "art/character/FACE_VARIANTS.json")
    if {row.look for row in result.looks} != {row.look for row in plan.looks}:
        message = "Source appearance must cover exactly the accepted face-plan looks"
        raise ValueError(message)
    write_evidence(root / "art/character/source_appearance.json", result)
    return result


def verified_appearance(reference: AssetRef) -> AppearanceManifest:
    """Check the compiled bytes and every original review envelope before use."""
    reference.verify()
    manifest = AppearanceManifest.model_validate_json(reference.path.read_bytes())
    for envelope in manifest.envelopes:
        envelope.verify()
    return manifest


def appearance_for(root: Path, look: str) -> tuple[AssetRef, SourceAppearance]:
    reference = AssetRef.of(root / "art/character/source_appearance.json")
    manifest = verified_appearance(reference)
    for row in manifest.looks:
        if row.look == look:
            row.source.verify()
            return reference, row
    message = f"Missing source-observed appearance: {look}"
    raise ValueError(message)


def enlarge_reference(source: AssetRef, directory: Path) -> AssetRef:
    """Replicate every native pixel into a 4x4 block, preserving mode and alpha."""
    source.verify()
    with Image.open(source.path) as native, BytesIO() as encoded:
        native.resize((native.width * 4, native.height * 4), Image.Resampling.NEAREST).save(encoded, format="PNG")
        return seal(directory / f"{source.sha256}.4x.png", encoded.getvalue())


def bust_pixels(native: Image.Image) -> tuple[tuple[int, int, int, int], tuple[int, int, int, int], Image.Image]:
    """Crop the full visible width and ceil(3/4 visible height), then replicate pixels."""
    if native.mode != "RGBA":
        message = "Bust detail requires the native RGBA source without conversion"
        raise ValueError(message)
    visible = native.getchannel("A").getbbox()
    if visible is None:
        message = "Bust detail requires nonempty visible alpha"
        raise ValueError(message)
    left, top, right, bottom = visible
    crop = (left, top, right, top + (3 * (bottom - top) + 3) // 4)
    with native.crop(crop) as cropped:
        expanded = cropped.resize((cropped.width * 12, cropped.height * 12), Image.Resampling.NEAREST)
    return visible, crop, expanded


def verify_bust_detail(binding: AssetRef, source: AssetRef, image: AssetRef) -> BustDetail:
    """Verify provenance and recompute every RGBA pixel, rejecting rehashed retouching."""
    binding.verify()
    detail = BustDetail.model_validate_json(binding.path.read_bytes())
    if detail.source != source or detail.image != image:
        message = "Bust detail must bind native reference1 and displayed reference4 exactly"
        raise ValueError(message)
    source.verify()
    image.verify()
    with Image.open(source.path) as native, Image.open(image.path) as actual:
        visible, crop, expected = bust_pixels(native)
        with expected:
            if (detail.visible_bbox != visible or detail.crop_bbox != crop or actual.mode != "RGBA"
                    or actual.size != expected.size or actual.tobytes() != expected.tobytes()):
                message = "Bust detail differs from its deterministic source crop and exact12x pixels"
                raise ValueError(message)
    return detail


def create_bust_detail(source: AssetRef, directory: Path) -> AssetRef:
    """Seal a source-hash-named crop and sidecar; existing derivations must verify before reuse."""
    source.verify()
    with Image.open(source.path) as native:
        visible, crop, expanded = bust_pixels(native)
        with expanded, BytesIO() as encoded:
            expanded.save(encoded, format="PNG")
            payload = encoded.getvalue()
    stem = f"{source.sha256}.upper3of4.12x"
    image = AssetRef(path=(directory / f"{stem}.png").resolve(), sha256=sha256(payload).hexdigest())
    metadata = directory / f"{stem}.json"
    if metadata.exists():
        binding = AssetRef.of(metadata)
        _ = verify_bust_detail(binding, source, image)
        return binding
    _ = seal(image.path, payload)
    return seal_model(metadata, BustDetail(source=source, image=image, visible_bbox=visible, crop_bbox=crop))


def main(root: Path = ROOT) -> None:
    docs = root / "docs/v8/artpolish2"
    result = compile_appearance(root, (docs / "OWNER_SOURCE_ENVELOPES.json", docs / "IDLE_SOURCE_ENVELOPES.json"))
    print(f"Imported {len(result.looks)} source-observed appearance briefs")


if __name__ == "__main__":
    typer.run(main)
