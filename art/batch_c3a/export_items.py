# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image"]
# ///
# How to run: PYTHONPATH=. uv run art/batch_c3a/export_items.py
"""Export C3A item candidates through the unchanged ENV1 geometry convention."""

from __future__ import annotations

import json
from dataclasses import asdict
from hashlib import sha256
from pathlib import Path
from typing import TYPE_CHECKING, Annotated, ClassVar, Final, Literal

import numpy as np
from PIL import Image
from pydantic import BaseModel, ConfigDict, Field

from art.batch_c3a.items.boards import boards
from art.compositor.projection_check import (
    ALPHA_THRESHOLD,
    ProjectionSpec,
    check_projection,
)
from art.pipeline2.judge import judge, load_calibration
from art.pipeline_env1.items_prepare import normalize
from art.pipeline_env1.items_records import JsonRecord, JsonValue

if TYPE_CHECKING:
    from numpy.typing import NDArray

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path("/Users/rexxa/work/art-batch/C3A")
OUT: Final = ROOT / "art/batch_c3a/items"
CALIBRATION: Final = ROOT / "docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json"
SAFE_EDGE: Final = 63
STACKABLE: Final = frozenset(
    {"wool_cloth", "limestone", "nails", "bog_iron", "cut_stone"}
)
ITEMS: Final = STACKABLE | frozenset(
    {
        "iron_knife",
        "iron_adze",
        "iron_shield_boss",
        "composite_bow",
        "iron_hoe",
        "iron_plough",
    }
)
LAYOUTS: Final = (
    ("", 64, ((0, 0),)),
    ("_count_2_8", 42, ((3, 3), (16, 16), (3, 20))),
    (
        "_count_9_plus",
        29,
        (
            (2, 3),
            (20, 3),
            (33, 7),
            (2, 20),
            (19, 20),
            (33, 23),
            (5, 34),
            (20, 34),
            (33, 34),
        ),
    ),
)


class Generation(BaseModel):
    """Parse relevant generation fields without discarding the source record."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    id: str = Field(pattern=r"^[a-z][a-z0-9_]*$")
    attempt: int = Field(ge=1, le=3)
    era: Literal["bronze", "iron"]
    raw: Path
    prompt: str = Field(min_length=1)
    reference: Path


class Selection(BaseModel):
    """Only explicitly reviewed attempts may enter the proposed import set."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    approved: dict[str, Annotated[int, Field(ge=1, le=3)]]


def digest(path: Path) -> str:
    """Hash literal source or export bytes."""
    return sha256(path.read_bytes()).hexdigest()


def count_tiles(
    sprite: Image.Image, *, counts: bool
) -> tuple[tuple[str, Image.Image], ...]:
    """Apply exactly ENV1's existing 1/3/9 whole-icon copy layouts."""
    exports: list[tuple[str, Image.Image]] = []
    for suffix, size, offsets in LAYOUTS if counts else LAYOUTS[:1]:
        tile = Image.new("RGBA", (64, 64))
        part = sprite.resize((size, size), Image.Resampling.LANCZOS)
        for offset in offsets:
            tile.alpha_composite(part, offset)
        exports.append((suffix, tile))
    return tuple(exports)


def write_json(path: Path, value: JsonValue) -> None:
    """Persist strict JSON without nonfinite measurement values."""
    path.parent.mkdir(parents=True, exist_ok=True)
    _ = path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def export_record(path: Path, *, selected: bool = False) -> JsonRecord:
    """Keep failed diagnostics external; publish only numeric-ready candidates."""
    row = Generation.model_validate_json(path.read_text())
    raw = row.raw if row.raw.is_absolute() else path.parent / row.raw
    reference = (
        row.reference if row.reference.is_absolute() else path.parent / row.reference
    )
    style = judge(raw, load_calibration(CALIBRATION), family="items")
    with Image.open(raw) as opened:
        alpha: NDArray[np.uint8] = np.asarray(opened.convert("RGBA"))[:, :, 3]
        border: NDArray[np.uint8] = np.concatenate(
            (alpha[0], alpha[-1], alpha[:, 0], alpha[:, -1]),
        )
        source_alpha = {
            "has_alpha": "A" in opened.getbands(),
            "minimum": int(alpha.min()),
            "maximum": int(alpha.max()),
            "border_maximum": int(border.max()),
            "nonzero_bbox": opened.convert("RGBA").getchannel("A").getbbox(),
        }
    alpha_pass = (
        source_alpha["has_alpha"]
        and int(alpha.min()) == 0
        and int(border.max()) < ALPHA_THRESHOLD
        and int(alpha.max()) >= ALPHA_THRESHOLD
    )
    sprite = normalize(raw)
    exports: list[JsonValue] = []
    structural_pass = bool(alpha_pass)
    for suffix, tile in count_tiles(sprite, counts=row.id in STACKABLE):
        bounds = tile.getchannel("A").getbbox()
        safe = (
            bounds is not None and min(bounds[:2]) >= 1 and max(bounds[2:]) <= SAFE_EDGE
        )
        structural_pass = structural_pass and safe
        opaque_rows = np.nonzero(np.asarray(tile)[:, :, 3] >= ALPHA_THRESHOLD)[0]
        base = int(opaque_rows.max()) if opaque_rows.size else 32
        projection = check_projection(tile, ProjectionSpec("item", (32, base), (4, 58)))
        target = DATA / "candidates/items" / f"{row.id}_try{row.attempt}{suffix}.png"
        target.parent.mkdir(parents=True, exist_ok=True)
        tile.save(target, optimize=True)
        exports.append(
            {
                "id": row.id + suffix,
                "candidate": str(target),
                "sha256": digest(target),
                "canvas_px": [64, 64],
                "anchor_px": [32, 32],
                "tile_footprint": [1, 1],
                "safe_bounds": safe,
                "projection": asdict(projection),
            }
        )
    ready = structural_pass and style.numeric_pass
    if selected and not ready:
        message = f"Selected item fails frozen checks: {row.id} try{row.attempt}"
        raise ValueError(message)
    evidence: dict[str, JsonValue] = {
        "id": row.id,
        "era": row.era,
        "attempt": row.attempt,
        "status": (
            "ADOPTED_NUMERIC_AND_LEADER_VISUAL"
            if selected
            else "NUMERIC_READY_VISUAL_REVIEW_PENDING"
        )
        if ready
        else "REJECTED_NUMERIC_OR_STRUCTURE",
        "generation_record": str(path),
        "record_sha256": digest(path),
        "generation": JsonRecord.read(path).fields,
        "raw_sha256": digest(raw),
        "reference": {"path": str(reference), "sha256": digest(reference)},
        "calibration": str(CALIBRATION.relative_to(ROOT)),
        "calibration_sha256": digest(CALIBRATION),
        "source_alpha": source_alpha,
        "structural_pass": structural_pass,
        "style": asdict(style),
        "exports": exports,
        "operations": [
            "ENV1 nonzero-alpha crop/proportional Lanczos fit58/pad64",
            "ENV1 whole-icon 1/3/9 copy layouts",
        ],
        "visual_review": "PASS_LEADER" if selected else "PENDING_LEADER",
        "selection_sha256": digest(DATA / "item_selection.json") if selected else None,
        "seam": "N/A isolated inventory item",
    }
    write_json(DATA / "checks/items" / f"{row.id}_try{row.attempt}.json", evidence)
    if ready and selected:
        for item in exports:
            record = JsonRecord.parse(item)
            source = Path(record.text("candidate"))
            destination = OUT / (record.text("id") + ".png")
            destination.parent.mkdir(parents=True, exist_ok=True)
            _ = destination.write_bytes(source.read_bytes())
        write_json(OUT / (row.id + ".json"), evidence)
    return JsonRecord(evidence)


def main() -> None:
    """Export every available attempt, retaining the latest numeric-ready candidate."""
    selection = Selection.model_validate_json(
        (DATA / "item_selection.json").read_text()
    )
    for item_id, attempt in selection.approved.items():
        if item_id not in ITEMS or not (DATA / f"{item_id}_try{attempt}.json").exists():
            message = f"Unknown or missing selected item: {item_id} try{attempt}"
            raise ValueError(message)
    for item_id in sorted(ITEMS):
        for path in OUT.glob(item_id + "*.png"):
            path.unlink()
        (OUT / (item_id + ".json")).unlink(missing_ok=True)
    results: list[JsonValue] = []
    for item_id in sorted(ITEMS):
        for path in sorted(DATA.glob(f"{item_id}_try[123].json")):
            row = Generation.model_validate_json(path.read_text())
            selected = selection.approved.get(row.id) == row.attempt
            results.append(export_record(path, selected=selected).fields)
    assets: list[JsonValue] = []
    for path in sorted(OUT.glob("*.json")):
        if path.stem not in ITEMS:
            continue
        row = JsonRecord.read(path)
        assets.extend(
            {
                "id": item.text("id"),
                "canvas_px": [64, 64],
                "anchor_px": [32, 32],
                "tile_footprint": [1, 1],
            }
            for item in row.records("exports")
        )
    write_json(
        OUT / "items_manifest.draft.json",
        {
            "schema_version": 1,
            "status": "LEADER_REVIEWED_PROPOSED_NOT_IMPORTED",
            "assets": assets,
        },
    )
    names = [JsonRecord.parse(asset).text("id") for asset in assets]
    boards(names, OUT, DATA)
    write_json(DATA / "checks/items_summary.json", results)
    print(
        f"{len(results)} attempts; {len(assets)} selected PNGs;",
        "not game imported",
    )


if __name__ == "__main__":
    main()
