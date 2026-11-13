# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy", "pillow", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: uv run --with numpy --with pillow --with pydantic --with scipy --with scikit-image python -m art.climate_colour.export [ASSET_COMMIT]
"""Export R2 colour-only corrections from immutable B2 bytes and shipped controls."""

from __future__ import annotations

import json
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image
from pydantic import JsonValue, TypeAdapter

from art.climate_colour.colour import distribution, recolour
from art.climate_colour.models import Draft, Ground, ShrubFamily
from art.climate_colour.references import GROUND_REFERENCES, ROOT, read, reference
from art.flora.check_assets import mask_metrics, summer_metrics
from art.flora.normalize import digest
from art.flora.seasons import canopy_mask
from art.pipeline2.ground import measure

DATA: Final = Path("/Users/rexxa/work/art-batch/R2")
BASE: Final = DATA / "baseline"
OUT: Final = ROOT / "art/climate_colour"
B2: Final = "403caffd81174d4ecdc00b79d3dd2c465f1ab236"
SHIPPED: Final = "f182769fe06b1cb60105e4d0bf127b6524277838"


def write(path: Path, value: JsonValue) -> None:
    """Write human-reviewable records with actual newlines."""
    path.parent.mkdir(parents=True, exist_ok=True)
    _ = path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")


def serial(value: str) -> JsonValue:
    """Parse typed model JSON at the serialization boundary."""
    adapter: TypeAdapter[JsonValue] = TypeAdapter(JsonValue)
    return adapter.validate_json(value)


def unchanged(source: Image.Image, output: Image.Image) -> bool:
    """Prove alpha, hidden pixels and canvas identity without threshold masks."""
    a, b = np.asarray(source), np.asarray(output)
    return (
        source.size == output.size
        and bool(np.array_equal(a[..., 3], b[..., 3]))
        and bool(np.array_equal(a[a[..., 3] == 0], b[a[..., 3] == 0]))
    )


def quantiles(image: Image.Image) -> list[JsonValue]:
    """Convert measured channel tuples into JSON arrays."""
    return [[float(x) for x in channel] for channel in distribution(image)]


def main() -> None:
    """A single recorded transform pass; inherited gaps never become active."""
    draft = Draft.model_validate_json(
        (BASE / "art/climate/art_import.draft.json").read_bytes()
    )
    active: list[JsonValue] = []
    proposals: list[JsonValue] = []
    measurements: list[JsonValue] = []
    targets: dict[str, JsonValue] = {}
    shrubs = TypeAdapter(list[ShrubFamily]).validate_json(
        (BASE / "art/climate/shrubs/catalog.json").read_bytes()
    )
    for row in (*draft.files, *draft.proposed_files):
        if row.lane == "shrubs":
            family, state = row.id.rsplit("_", 1)
            membership = next(r.climate for r in shrubs if r.id == family)
            row = row.model_copy(
                update={"species": family, "state": state, "climate": membership}
            )
        source_path = BASE / row.path
        assert digest(source_path) == row.sha256, row.id
        original = read(source_path)
        control, paths, rationale = reference(row)
        target_key = "|".join(paths) + "|" + (row.season or "none") + "|" + row.state
        targets[target_key] = {
            "paths": list(paths),
            "hashes": [digest(ROOT / p) for p in paths],
            "quantiles": quantiles(control),
            "rationale": rationale,
        }
        olive = row.species == "wild_olive"
        output = original.copy() if row.lane == "trees" else recolour(original, control)
        olive_checks: dict[str, JsonValue] = {}
        if olive:
            mask_path = BASE / (row.mask or row.path.replace(".png", "_mask.png"))
            mask = read(mask_path)
            output = recolour(original, control)
            olive_checks = {
                "pass": 3,
                "prior_pass": "R2/measurements-pass1.json and measurements-pass2.json; incorrect shrub proxy superseded by same-size tree control",
                "original_mask_summer": asdict(summer_metrics(output, mask, ROOT)),
                "reclassified_mask_summer": asdict(
                    summer_metrics(output, canopy_mask(output, ROOT), ROOT)
                ),
                "mask": asdict(mask_metrics(output, mask, ROOT)),
                "status": "PROPOSED_SPECIES_NOT_ACTIVE",
            }
            destination_mask = ROOT / str(mask_path.relative_to(BASE))
            destination_mask.parent.mkdir(parents=True, exist_ok=True)
            _ = destination_mask.write_bytes(mask_path.read_bytes())
        assert unchanged(original, output), row.id
        destination = ROOT / row.path
        destination.parent.mkdir(parents=True, exist_ok=True)
        output.save(destination, optimize=True)
        metadata: dict[str, JsonValue] = {
            "id": row.id,
            "path": row.path,
            "sha256": digest(destination),
            "canvas": list(row.canvas),
            "anchor": list(row.anchor),
            "footprint": list(row.footprint),
            "lane": row.lane,
            "role": "tree" if row.lane == "trees" else "flora",
            "type": "flora",
            "size": row.size,
            "mask": row.mask
            or (row.path.replace(".png", "_mask.png") if olive else ""),
            "species": row.species,
            "state": row.state,
            "season": row.season,
            "climate": list(row.climate) if row.climate else None,
            "dest": row.dest,
            "source": row.path,
            "source_sha256": digest(destination),
            "b2_commit": B2,
            "b2_sha256": row.sha256,
            "sidecar": str(destination.with_suffix(".json").relative_to(ROOT)),
            "status": "PROPOSAL_NOT_ACTIVE" if olive else "COLOUR_CORRECTED_CANDIDATE",
            "reference": target_key,
            "olive": olive_checks,
        }
        write(
            destination.with_suffix(".json"),
            {"r2": metadata, "b2_metadata": serial(row.model_dump_json())},
        )
        (proposals if olive else active).append(metadata)
        measurements.append(
            {
                "id": row.id,
                "before": quantiles(original),
                "after": quantiles(output),
                "reference": target_key,
                "alpha_canvas_hidden_unchanged": True,
                "transform": "monotone_HSV_quantile_transport_pass3"
                if olive
                else "identity_shipped_season"
                if row.lane == "trees"
                else "monotone_HSV_quantile_transport",
                "olive": olive_checks,
            }
        )
    grounds = TypeAdapter(list[Ground]).validate_json(
        Path("/Users/rexxa/work/art-batch/B2/grounds/diagnostics.json").read_bytes()
    )
    ground_rows: list[JsonValue] = []
    for species, ref in GROUND_REFERENCES.items():
        row = max(
            (r for r in grounds if r.id == f"{species}_summer"), key=lambda r: r.attempt
        )
        assert digest(row.tile) == row.tile_sha256 and digest(row.raw) == row.raw_sha256
        original = read(row.tile)
        ref_path = f"art/style05/env/ground/{ref}.png"
        control = read(ROOT / ref_path)
        output = recolour(original, control)
        assert unchanged(original, output)
        destination = DATA / f"grounds/{species}_summer.png"
        destination.parent.mkdir(exist_ok=True)
        output.save(destination)
        targets[species] = {
            "paths": [ref_path],
            "hashes": [digest(ROOT / ref_path)],
            "quantiles": quantiles(control),
        }
        ground_rows.append(
            {
                "id": species,
                "source": str(row.tile),
                "source_sha256": row.tile_sha256,
                "path": str(destination),
                "sha256": digest(destination),
                "attempt": row.attempt,
                "canvas": [64, 64],
                "anchor": [32, 43],
                "footprint": [1, 1],
                "before": quantiles(original),
                "after": quantiles(output),
                "reference": species,
                "seam_before": asdict(measure(original)),
                "seam_after": asdict(measure(output)),
                "inherited_style": row.style,
                "status": "BLOCKED_PRIOR_NONCOLOUR_GATES_NOT_WAIVED",
                "alpha_canvas_hidden_unchanged": True,
            }
        )
    source_commit = sys.argv[1] if len(sys.argv) > 1 else None
    write(
        OUT / "targets.json",
        {
            "reference_commit": SHIPPED,
            "channels": ["circular_hue_turns", "saturation", "value"],
            "quantiles": [0.1, 0.5, 0.9],
            "targets": targets,
        },
    )
    write(DATA / "measurements.json", {"flora": measurements, "grounds": ground_rows})
    write(OUT / "ground_review.json", ground_rows)
    write(
        OUT / "art_import.draft.json",
        {
            "schema_version": 1,
            "status": "DRAFT_NOT_APPLIED",
            "mode": "additive_fragment_not_replacement",
            "out": "assets/v8art",
            "source_branch": "v8/art-batch-r2",
            "source_commit": source_commit,
            "b2_commit": B2,
            "files": active,
            "proposed_files": proposals,
            "existing_references": [
                serial(r.model_dump_json()) for r in draft.existing_references
            ],
            "blocked": draft.model_extra["blocked"] if draft.model_extra else {},
            "ground_proposals": "art/climate_colour/ground_review.json",
            "notes": [
                "Keep prior noncolour and missing-model gates; colour correction is not game import.",
                "Olive remains proposed pending all canopy checks and species integration.",
                "Read sidecars for untouched B2 provenance and geometry. No seasonal tint should be applied twice.",
            ],
        },
    )
    bindings = (BASE / "art/climate/bindings.draft.json").read_text()
    for lane in ("herbaceous", "shrubs", "trees"):
        bindings = bindings.replace(
            f"art/climate/{lane}/manifest.json",
            "art/climate_colour/art_import.draft.json",
        )
    bindings = bindings.replace(
        "art/climate/grounds/manifest.json", "art/climate_colour/ground_review.json"
    )
    _ = (OUT / "bindings.draft.json").write_text(bindings)
    print(
        f"R2: {len(active)} active candidates, {len(proposals)} olive proposals, {len(ground_rows)} blocked grounds"
    )


if __name__ == "__main__":
    main()
