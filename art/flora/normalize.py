# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: python3 -m art.flora.normalize /Users/rexxa/work/arttrees-data/completed.json
"""Registered D2 whole-object production reduction; originals remain immutable."""
from __future__ import annotations

import hashlib
import json
import shutil
import sys
from pathlib import Path
from typing import ClassVar, Final, Literal

from PIL import Image
from pydantic import BaseModel, ConfigDict, Field, JsonValue, TypeAdapter

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path("/Users/rexxa/work/arttrees-data")


class GenerationInput(BaseModel):
    """Additional generation-tool provenance is retained in the evidence record."""
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="allow")
    id: str = Field(pattern=r"^[a-z][a-z0-9_]*$")
    source: Path
    attempt: int = Field(default=1, ge=1, le=3)


class ProductionSpec(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    canvas: tuple[int, int]
    anchor: tuple[int, int]
    height: int


class Transform(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    source_canvas: tuple[int, int]
    crop: tuple[int, int, int, int]
    scale: float
    scaled_canvas: tuple[int, int]
    offset: tuple[int, int]
    output_canvas: tuple[int, int]
    anchor: tuple[int, int]
    target_height: int


class NormalizedRecord(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    id: str
    attempt: int
    source: str
    source_sha256: str
    preserved_original: str
    output: str
    output_sha256: str
    transform_reference: str
    transform: Transform
    source_opaque_pixels_outside_crop: int
    output_opaque_bbox: tuple[int, int, int, int] | None
    operations: tuple[str, ...]
    provenance: dict[str, JsonValue]
    acceptance: Literal["NOT_CLAIMED"] = "NOT_CLAIMED"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def production_spec(asset_id: str) -> ProductionSpec:
    """Classify geometry by declared size suffix, never by species name."""
    if "_large_" in asset_id:
        return ProductionSpec(canvas=(192, 192), anchor=(96, 180), height=160)
    if "_small_" in asset_id:
        return ProductionSpec(canvas=(128, 112), anchor=(64, 100), height=80)
    if asset_id.endswith("_stump"):
        return ProductionSpec(canvas=(64, 48), anchor=(32, 40), height=20)
    message = f"No registered D2 production size for {asset_id!r}"
    raise ValueError(message)


def opaque_bbox(image: Image.Image) -> tuple[int, int, int, int] | None:
    return image.getchannel("A").point([0] * 128 + [255] * 128).getbbox()


def plan_transform(image: Image.Image, spec: ProductionSpec) -> Transform:
    """Register once from the summer source; never refit a bare counterpart."""
    bbox = opaque_bbox(image)
    if bbox is None:
        message = "Cannot normalize an image with no alpha>=128 foreground"
        raise ValueError(message)
    crop = (max(0, bbox[0] - 2), max(0, bbox[1] - 2),
            min(image.width, bbox[2] + 2), min(image.height, bbox[3] + 2))
    scale = spec.height / (bbox[3] - bbox[1])
    size = (max(1, round((crop[2] - crop[0]) * scale)),
            max(1, round((crop[3] - crop[1]) * scale)))
    scaled = image.crop(crop).resize(size, Image.Resampling.LANCZOS)
    scaled_bbox = opaque_bbox(scaled)
    if scaled_bbox is None:
        message = "Whole-object reduction lost the opaque silhouette"
        raise ValueError(message)
    offset = (spec.anchor[0] - (scaled_bbox[0] + scaled_bbox[2] - 1) // 2,
              spec.anchor[1] - scaled_bbox[3] + 1)
    return Transform(source_canvas=image.size, crop=crop, scale=scale,
                     scaled_canvas=size, offset=offset, output_canvas=spec.canvas,
                     anchor=spec.anchor, target_height=spec.height)


def apply_transform(image: Image.Image, transform: Transform) -> Image.Image:
    """Apply one identical whole-frame crop/scale/translation to an RGBA image."""
    if image.size != transform.source_canvas:
        message = "Paired bare and summer source canvases differ; registration is unproven"
        raise ValueError(message)
    scaled = image.crop(transform.crop).resize(transform.scaled_canvas, Image.Resampling.LANCZOS)
    result = Image.new("RGBA", transform.output_canvas)
    result.alpha_composite(scaled, transform.offset)
    return result


def normalize_all(requests_path: Path, root: Path = ROOT) -> tuple[NormalizedRecord, ...]:
    """Normalize chosen records, preserve bytes externally and write provenance."""
    requests = TypeAdapter(list[GenerationInput]).validate_json(requests_path.read_bytes())
    selected: dict[str, GenerationInput] = {}
    for request in requests:
        if request.id in selected:
            message = f"Choose one source per id before normalization: {request.id}"
            raise ValueError(message)
        selected[request.id] = request
    transforms: dict[str, Transform] = {}
    for request in requests:
        if request.id.endswith("_bare"):
            continue
        with Image.open(request.source) as source:
            transforms[request.id] = plan_transform(source.convert("RGBA"), production_spec(request.id))
    originals = requests_path.parent / "originals"
    originals.mkdir(parents=True, exist_ok=True)
    output_directory = root / "art/flora/sprites"
    output_directory.mkdir(parents=True, exist_ok=True)
    records: list[NormalizedRecord] = []
    for request in requests:
        reference = request.id.removesuffix("_bare") + "_v1" if request.id.endswith("_bare") else request.id
        transform = transforms[reference]
        source_hash = digest(request.source)
        original = originals / f"{request.id}_try{request.attempt}_{source_hash[:12]}.png"
        if original.exists() and digest(original) != source_hash:
            message = f"Preserved original hash conflict: {original}"
            raise ValueError(message)
        if not original.exists():
            shutil.copyfile(request.source, original)
        with Image.open(request.source) as source:
            image = source.convert("RGBA")
        opaque = image.getchannel("A").point([0] * 128 + [255] * 128)
        outside = opaque.histogram()[255] - opaque.crop(transform.crop).histogram()[255]
        result = apply_transform(image, transform)
        output = output_directory / f"{request.id}.png"
        result.save(output, optimize=True)
        records.append(NormalizedRecord(
            id=request.id, attempt=request.attempt, source=str(request.source),
            source_sha256=source_hash, preserved_original=str(original),
            output=str(output.relative_to(root)), output_sha256=digest(output),
            transform_reference=reference, transform=transform,
            source_opaque_pixels_outside_crop=outside, output_opaque_bbox=opaque_bbox(result),
            operations=("alpha>=128 bbox with 2px fringe from reference summer v1 or self",
                        "uniform whole-object LANCZOS reduction",
                        "transparent canvas padding at registered D2 anchor",
                        "bare uses exactly the paired summer crop, scale and translation"),
            provenance=request.model_dump(mode="json"),
        ))
    report = root / "docs/v8/arttrees/GENERATION.json"
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps({"schema_version": 1, "assets": [record.model_dump(mode="json")
                      for record in records]}, indent=2) + "\n")
    return tuple(records)


if __name__ == "__main__":
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else DATA / "completed.json"
    result = normalize_all(path)
    print(json.dumps({"normalized": len(result), "acceptance": "NOT_CLAIMED"}))
