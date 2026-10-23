# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "typer"]
# ///
# How to run: python3 -m art.pipeline_polish2.portrait_appearance
"""Import source-observed appearance and preserve exact enlarged sprite references."""
from __future__ import annotations

from io import BytesIO
from pathlib import Path
from typing import ClassVar, Literal, Self

import typer
from PIL import Image
from pydantic import BaseModel, ConfigDict, Field, model_validator

from art.compositor.face_variants import FacePlan
from art.pipeline_polish2.blind_round import seal
from art.pipeline_polish2.portrait_models import (
    ROOT,
    AssetRef,
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


def main(root: Path = ROOT) -> None:
    docs = root / "docs/v8/artpolish2"
    result = compile_appearance(root, (docs / "OWNER_SOURCE_ENVELOPES.json", docs / "IDLE_SOURCE_ENVELOPES.json"))
    print(f"Imported {len(result.looks)} source-observed appearance briefs")


if __name__ == "__main__":
    typer.run(main)
