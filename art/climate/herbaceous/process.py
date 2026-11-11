# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: python -m art.climate.herbaceous.process
"""Export explicitly selected wild plants using the existing whole-frame geometry."""

from __future__ import annotations

import hashlib
import json
from dataclasses import asdict
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image, ImageDraw
from pydantic import BaseModel, ConfigDict, Field

from art.compositor.projection_check import ProjectionSpec, check_projection
from art.flora.normalize import ProductionSpec, apply_transform, digest, plan_transform
from art.pipeline2.judge import judge, load_calibration
from art.pipeline3.structure import components

ROOT: Final = Path(__file__).resolve().parents[3]
DATA: Final = Path("/Users/rexxa/work/art-batch/B2/herbaceous")
OUT: Final = ROOT / "art/flora/climate/herbaceous"
META: Final = ROOT / "art/climate/herbaceous"
HEIGHTS: Final = {
    "wild_grain": 48,
    "wild_barley": 48,
    "wild_einkorn": 48,
    "wild_flax": 40,
    "wild_lentil": 28,
    "wild_pea": 32,
    "wild_hemp": 48,
    "wild_rye": 48,
    "wild_millet": 44,
    "wild_root": 24,
    "wild_grape": 64,
}


class Selection(BaseModel):
    """Only named recorded attempts can be exported."""

    model_config = ConfigDict(frozen=True, extra="forbid")
    id: str = Field(pattern=r"^wild_[a-z]+_(full|harvested|winter)$")
    attempt: int = Field(ge=1, le=3)
    visual_score: int = Field(ge=0, le=100)
    visual_notes: str


def run() -> None:
    """Preserve diagnostics and export only visually selected candidates."""
    selections = [
        Selection.model_validate(row)
        for row in json.loads((DATA / "selection.json").read_text())
    ]
    assert selections and len({row.id for row in selections}) == len(selections)
    calibration = load_calibration(ROOT / "docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json")
    manifest = []
    OUT.mkdir(parents=True, exist_ok=True)
    for row in selections:
        species, state = row.id.rsplit("_", 1)
        record_path = DATA / "records" / f"{row.id}_try{row.attempt}.json"
        record = json.loads(record_path.read_text())
        source = DATA / "raw" / f"{row.id}_try{row.attempt}.png"
        tool_source = Path(record.get("source_tool", record.get("source", "")))
        assert tool_source.is_file() and digest(tool_source) == digest(source), row.id
        # Geometry is preregistered from the original full source even after a retry.
        full_path = DATA / "raw" / f"{species}_full_try1.png"
        with Image.open(source) as opened, Image.open(full_path) as full_opened:
            raw, full = opened.convert("RGBA"), full_opened.convert("RGBA")
        grape = species == "wild_grape"
        spec = ProductionSpec(
            canvas=(96, 96) if grape else (64, 64),
            anchor=(48, 80) if grape else (32, 56),
            height=HEIGHTS[species],
        )
        transform = plan_transform(full, spec)
        size_match = raw.size == full.size
        # shortcut: changed imagegen canvases stay rejected, never independently refit stubble.
        if not size_match:
            manifest.append(
                {
                    "id": row.id,
                    "status": "REJECTED_CANVAS_MISMATCH",
                    "source": str(source),
                    "source_size": raw.size,
                    "full_size": full.size,
                }
            )
            continue
        image = apply_transform(raw, transform)
        alpha = np.asarray(image.getchannel("A"))
        visible = alpha >= 128
        raw_alpha = np.asarray(raw.getchannel("A"))
        raw_border = any(
            bool(np.any(edge >= 128))
            for edge in (raw_alpha[0], raw_alpha[-1], raw_alpha[:, 0], raw_alpha[:, -1])
        )
        groups = components(visible)
        component_count = len(groups)
        component_sizes = sorted((len(group) for group in groups), reverse=True)
        rows = np.nonzero(visible)[0]
        border = any(
            bool(np.any(edge >= 16))
            for edge in (alpha[0], alpha[-1], alpha[:, 0], alpha[:, -1])
        )
        outside = int(
            np.count_nonzero(np.asarray(raw.getchannel("A")) >= 128)
            - np.count_nonzero(
                np.asarray(raw.getchannel("A").crop(transform.crop)) >= 128
            )
        )
        projection = check_projection(
            image, ProjectionSpec("item", spec.anchor, (1, HEIGHTS[species] + 2))
        )
        style = judge(source, calibration, family="plants")
        structural = bool(
            rows.size
            and int(alpha.min()) == 0
            and not border
            and not raw_border
            and outside == 0
        )
        accepted = (
            structural
            and projection.passed
            and style.numeric_pass
            and row.visual_score >= 90
        )
        output = OUT / (row.id + ".png")
        evidence = {
            "id": row.id,
            "state": state,
            "species": species,
            "status": "ADOPTED_CANDIDATE" if accepted else "REJECTED",
            "source": str(source),
            "source_sha256": digest(source),
            "generation_record": str(record_path),
            "generation_record_sha256": digest(record_path),
            "prompt_sha256": hashlib.sha256(record["prompt"].encode()).hexdigest(),
            "reference_sha256": {
                ref: digest(Path(ref))
                for ref in record.get("references", record.get("refs", []))
            },
            "attempt": row.attempt,
            "seed": None,
            "geometry_source": str(full_path),
            "geometry_source_sha256": digest(full_path),
            "transform": transform.model_dump(),
            "footprint": [1, 1],
            "checks": {
                "structure": structural,
                "visible_border": border,
                "raw_border": raw_border,
                "opaque_components": component_count,
                "component_sizes": component_sizes,
                "tree_connectivity": "NOT_APPLICABLE_MULTISTEM_HERB",
                "source_opaque_outside_crop": outside,
                "size_match": size_match,
                "tool_source_sha256": digest(tool_source),
                "actual_raw": str(source),
            },
            "projection": asdict(projection),
            "style": asdict(style),
            "review": row.model_dump(),
        }
        trial_dir = DATA / "normalized"
        trial_dir.mkdir(exist_ok=True)
        image.save(trial_dir / (row.id + ".png"))
        if accepted:
            image.save(output, optimize=True)
            evidence["path"] = str(output.relative_to(ROOT))
            evidence["sha256"] = digest(output)
            output.with_suffix(".json").write_text(
                json.dumps(evidence, indent=2) + "\n"
            )
        else:
            output.unlink(missing_ok=True)
            output.with_suffix(".json").unlink(missing_ok=True)
        manifest.append(evidence)
    (META / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    import_draft(manifest)
    boards(manifest)


def import_draft(manifest: list) -> None:
    """Describe symbolic bindings without assigning nonexistent runtime codes."""
    active = {
        "wild_grain",
        "wild_barley",
        "wild_flax",
        "wild_lentil",
        "wild_pea",
        "wild_rye",
        "wild_millet",
        "wild_root",
    }
    files = []
    for row in manifest:
        if row["status"] != "ADOPTED_CANDIDATE":
            continue
        grape = row["species"] == "wild_grape"
        files.append(
            {
                "id": row["id"],
                "species": row["species"],
                "state": row["state"],
                "path": row["path"],
                "sha256": row["sha256"],
                "canvas": [96, 96] if grape else [64, 64],
                "anchor": [48, 80] if grape else [32, 56],
                "footprint": [1, 1],
                "climate": None
                if row["species"] in {"wild_einkorn", "wild_hemp"}
                else ["mild"]
                if row["species"] in {"wild_grape", "wild_millet"}
                else ["neutral", "harsh"]
                if row["species"] == "wild_rye"
                else ["mild", "neutral"]
                if row["species"] in {"wild_lentil", "wild_pea"}
                else ["mild", "neutral", "harsh"],
                "runtime_binding": "VERIFIED_SYMBOLIC_ID"
                if row["species"] in active
                else "PROPOSED_ID",
                "runtime_code": None,
                "placement_status": "VERIFIED_ORIGIN"
                if row["species"] in active
                else "PROPOSED_NOT_APPLIED",
            }
        )
    draft = {
        "status": "DRAFT_NOT_RUNTIME_IMPORT",
        "files": files,
        "active_ids": sorted(active),
        "proposed_ids": ["wild_einkorn", "wild_grape", "wild_hemp"],
        "notes": [
            "wild_grain is emmer wheat; einkorn is a distinct proposed art ID.",
            "Oats share the bible wild_rye source; no new oat wild-source ID.",
            "Grape is mild only. Symbolic IDs and state mapping require owner integration.",
        ],
        "gaps": [row["id"] for row in manifest if row["status"] != "ADOPTED_CANDIDATE"],
    }
    (META / "art_import.draft.json").write_text(json.dumps(draft, indent=2) + "\n")


def boards(manifest: list) -> None:
    """Show every available output beside unchanged existing flora at two scales."""
    paths = [
        (
            row["id"] + " [" + row["status"].replace("_CANDIDATE", "") + "]",
            DATA / "normalized" / (row["id"] + ".png"),
        )
        for row in manifest
    ]
    paths = [(name, path) for name, path in paths if path.exists()]
    paths[:0] = [
        ("REF berry", ROOT / "art/flora/sprites/berry_bush_full.png"),
        ("REF oak", ROOT / "art/flora/sprites/oak_small_v1.png"),
    ]
    for scale in (1, 2):
        sheet = Image.new(
            "RGB",
            (7 * 150 * scale, ((len(paths) + 6) // 7) * 125 * scale),
            (183, 184, 165),
        )
        draw = ImageDraw.Draw(sheet)
        for index, (name, path) in enumerate(paths):
            with Image.open(path) as opened:
                image = opened.convert("RGBA")
            image = image.resize(
                (image.width * scale, image.height * scale), Image.Resampling.NEAREST
            )
            x, y = index % 7 * 150 * scale, index // 7 * 125 * scale
            sheet.paste(image, (x + (150 * scale - image.width) // 2, y), image)
            label, _, status = name.partition(" [")
            draw.text((x + 2, y + 104 * scale), label, fill=(25, 25, 22))
            draw.text(
                (x + 2, y + 114 * scale),
                status.rstrip("]"),
                fill=(140, 30, 20) if "REJECTED" in status else (25, 65, 25),
            )
        directory = META / "boards"
        directory.mkdir(exist_ok=True)
        sheet.save(directory / f"herbaceous_{scale}x.png")
        sheet.save(DATA / f"herbaceous_{scale}x.png")


if __name__ == "__main__":
    run()
