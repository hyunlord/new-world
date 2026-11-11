# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: python -m art.climate.trees.process RECORDS.json [OUTPUT_DIR]
"""Export tree candidates with shared registration and unchanged diagnostic gates."""

from __future__ import annotations

import hashlib
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, ClassVar, Final, Literal

import numpy as np
from PIL import Image
from pydantic import BaseModel, ConfigDict, Field, TypeAdapter

from art.compositor.projection_check import ProjectionResult, ProjectionSpec, check_projection
from art.flora.check_bare import SilhouetteMetrics, compare_silhouettes
from art.flora.normalize import Transform, apply_transform, digest, plan_transform, production_spec
from art.flora.schema import SeasonTint
from art.flora.seasons import apply_tint, canopy_mask
from art.pipeline2.judge import Verdict, judge, load_calibration
from art.pipeline3.structure import components

if TYPE_CHECKING:
    from numpy.typing import NDArray

ROOT: Final = Path(__file__).resolve().parents[3]
CALIBRATION: Final = ROOT / "docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json"
DEFAULT_OUTPUT: Final = ROOT / "art/flora/climate/trees"
OPAQUE: Final = 128
VISIBLE: Final = 16
CLI_WITH_OUTPUT: Final = 3
SIZES: Final = ("small", "large")
BANDS: Final = {"small": (64, 96), "large": (128, 192)}


class Record(BaseModel):
    """One explicit generated tree state; no season is synthesized implicitly."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    id: str = Field(pattern=r"^[a-z][a-z0-9_]*$")
    species: Literal["oak", "birch", "pine", "wild_apple", "wild_olive"]
    state: Literal["full", "harvested", "winter"]
    variant: str = Field(pattern=r"^v[1-9][0-9]*$")
    source: Path
    generation_record: Path
    transform_reference: str | None = None
    season_tints: dict[Literal["spring", "summer", "autumn", "winter"], SeasonTint] = Field(default_factory=dict)


class Generation(BaseModel):
    """Read provenance fields while preserving the full original by file hash."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    id: str
    attempt: int = Field(ge=1)
    source: Path
    preserved_source: Path
    prompt: str = Field(min_length=1)
    reference_images: tuple[Path, ...] = Field(min_length=1)
    seed: int | None = None


@dataclass(frozen=True, slots=True)
class FileIdentity:
    """Immutable file-byte identity."""

    path: str
    sha256: str


@dataclass(frozen=True, slots=True)
class Provenance:
    """Recorded generation inputs used for an export."""

    source: FileIdentity
    generation_record: FileIdentity
    prompt_sha256: str
    references: tuple[FileIdentity, ...]
    seed: int | None
    attempt: int


@dataclass(frozen=True, slots=True)
class Checks:
    """Unmodified source and export geometry measurements."""

    alpha_min: int
    alpha_max: int
    opaque_height: int
    opaque_components: int
    visible_border: bool
    raw_transparent: bool
    raw_opaque_border: bool
    source_opaque_outside_crop: int
    canvas_matches: bool
    passed: bool


class Export(BaseModel):
    """Candidate evidence; a successful export is never automatic adoption."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    id: str
    path: str
    sha256: str
    mask_path: str
    mask_sha256: str
    record: Record
    provenance: Provenance
    transform: Transform
    footprint: tuple[int, int] = (1, 1)
    checks: Checks
    projection: ProjectionResult
    judge_v2: Verdict
    calibration_sha256: str
    pair: SilhouetteMetrics | None
    derived: bool
    tint: SeasonTint | None
    failures: tuple[str, ...]
    status: Literal["CANDIDATE_REVIEW_REQUIRED"] = "CANDIDATE_REVIEW_REQUIRED"


@dataclass(frozen=True, slots=True)
class Inputs:
    """Parsed source records with exact provenance and opened RGBA copies."""

    records: tuple[Record, ...]
    images: dict[str, Image.Image]
    provenance: dict[str, Provenance]


def identity(path: Path) -> FileIdentity:
    """Freeze exact bytes, including the immutable generation record."""
    return FileIdentity(str(path), digest(path))


def inspect(image: Image.Image, raw: Image.Image, transform: Transform) -> Checks:
    """Measure original and exported alpha without correcting pixels."""
    alpha: NDArray[np.uint8] = np.asarray(image.getchannel("A"), dtype=np.uint8)
    source: NDArray[np.uint8] = np.asarray(raw.getchannel("A"), dtype=np.uint8)
    ys = np.nonzero(alpha >= OPAQUE)[0]
    border = any(bool(np.any(edge >= VISIBLE)) for edge in (alpha[0], alpha[-1], alpha[:, 0], alpha[:, -1]))
    raw_border = any(bool(np.any(edge >= OPAQUE)) for edge in (source[0], source[-1], source[:, 0], source[:, -1]))
    groups = len(components(alpha >= OPAQUE))
    crop: NDArray[np.uint8] = np.asarray(raw.getchannel("A").crop(transform.crop), dtype=np.uint8)
    outside = int(np.count_nonzero(source >= OPAQUE) - np.count_nonzero(crop >= OPAQUE))
    transparent = int(source.min()) == 0 and int(source.max()) >= OPAQUE
    matches = image.size == transform.output_canvas
    return Checks(
        int(alpha.min()),
        int(alpha.max()),
        int(ys.max() - ys.min() + 1) if ys.size else 0,
        groups,
        border,
        transparent,
        raw_border,
        outside,
        matches,
        groups == 1 and not border and transparent and not raw_border and outside == 0 and matches,
    )


def read_records(records_path: Path) -> Inputs:
    """Parse identities and relationships before writing any candidate."""
    records = TypeAdapter(tuple[Record, ...]).validate_json(records_path.read_bytes())
    lookup = {row.id: row for row in records}
    if not records or len(lookup) != len(records):
        message = "Tree records must be nonempty and have unique ids"
        raise ValueError(message)
    images: dict[str, Image.Image] = {}
    provenance: dict[str, Provenance] = {}
    for row in records:
        generation = Generation.model_validate_json(row.generation_record.read_bytes())
        source = identity(row.source)
        if generation.id != row.id or any(
            digest(path) != source.sha256 for path in (generation.source, generation.preserved_source)
        ):
            message = f"Generation id/source hash mismatch: {row.id}"
            raise ValueError(message)
        provenance[row.id] = Provenance(
            source,
            identity(row.generation_record),
            hashlib.sha256(generation.prompt.encode()).hexdigest(),
            tuple(identity(path) for path in generation.reference_images),
            generation.seed,
            generation.attempt,
        )
        with Image.open(row.source) as opened:
            images[row.id] = opened.convert("RGBA")
        if row.transform_reference is not None:
            reference = lookup.get(row.transform_reference)
            if (
                reference is None
                or reference.transform_reference is not None
                or reference.species != row.species
                or reference.variant != row.variant
            ):
                message = f"Invalid transform reference: {row.id}"
                raise ValueError(message)
            if row.state == "full" or reference.state != "full":
                message = f"Only harvested/winter states can share a full-state transform: {row.id}"
                raise ValueError(message)
        if any(tint.frame != "foliage" for tint in row.season_tints.values()):
            message = f"Season rendering cannot select an unprovided bare source: {row.id}"
            raise ValueError(message)
    if any(tint.frame != "foliage" for row in records for tint in row.season_tints.values()):
        message = "Season rendering cannot select an unprovided bare source"
        raise ValueError(message)
    return Inputs(records, images, provenance)


def run(records_path: Path, output: Path = DEFAULT_OUTPUT) -> tuple[Export, ...]:
    """Export both production sizes without silently accepting failures."""
    inputs = read_records(records_path)
    records, images, provenance = inputs.records, inputs.images, inputs.provenance
    calibration = load_calibration(CALIBRATION)
    results: list[Export] = []
    for row in records:
        raw = images[row.id]
        style = judge(row.source, calibration, family="plants")
        reference_id = row.transform_reference or row.id
        for size in SIZES:
            transform = plan_transform(images[reference_id], production_spec(f"tree_{size}_v1"))
            normalized = apply_transform(raw, transform)
            mask = canopy_mask(normalized, ROOT)
            reference_frame = apply_transform(images[reference_id], transform)
            pair = compare_silhouettes(reference_frame, normalized) if row.transform_reference else None
            images_to_save: list[tuple[str, Image.Image, SeasonTint | None]] = [(f"{row.id}_{size}", normalized, None)]
            images_to_save.extend(
                (f"{row.id}_{size}_{season}", apply_tint(normalized, mask, tint), tint)
                for season, tint in sorted(row.season_tints.items())
            )
            for asset_id, image, tint in images_to_save:
                check = inspect(image, raw, transform)
                projection = check_projection(image, ProjectionSpec("tree", transform.anchor, BANDS[size]))
                failures = tuple(
                    name
                    for name, passed in (
                        ("structure", check.passed),
                        ("projection", projection.passed),
                        ("raw_style_numeric", style.numeric_pass),
                    )
                    if not passed
                )
                output.mkdir(parents=True, exist_ok=True)
                path, mask_path = (
                    output / f"{asset_id}.png",
                    output / f"{asset_id}_mask.png",
                )
                image.save(path, optimize=True)
                mask.save(mask_path, optimize=True)
                result = Export(
                    id=asset_id,
                    path=str(path),
                    sha256=digest(path),
                    mask_path=str(mask_path),
                    mask_sha256=digest(mask_path),
                    record=row,
                    provenance=provenance[row.id],
                    transform=transform,
                    checks=check,
                    projection=projection,
                    judge_v2=style,
                    calibration_sha256=digest(CALIBRATION),
                    pair=pair,
                    derived=tint is not None,
                    tint=tint,
                    failures=failures,
                )
                _ = path.with_suffix(".json").write_text(result.model_dump_json(indent=2) + "\n")
                results.append(result)
    return tuple(results)


if __name__ == "__main__":
    if len(sys.argv) not in (2, CLI_WITH_OUTPUT):
        message = "Usage: python -m art.climate.trees.process RECORDS.json [OUTPUT_DIR]"
        raise SystemExit(message)
    rows = run(Path(sys.argv[1]), Path(sys.argv[2]) if len(sys.argv) == CLI_WITH_OUTPUT else DEFAULT_OUTPUT)
    derived = sum(row.derived for row in rows)
    failed = sum(bool(row.failures) for row in rows)
    _ = sys.stdout.write(
        f"Exported {len(rows)} candidates; {derived} derived; {failed} with failures; review required.\n"
    )
