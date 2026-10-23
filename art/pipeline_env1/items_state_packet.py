# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow"]
# ///
# How to run: PYTHONPATH=. uv run art/pipeline_env1/items_state_packet.py
"""Create a blind quantity/state packet without publishing its source answer key."""

from __future__ import annotations

import hashlib
import json
import random
import secrets
from collections import Counter
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.pipeline_env1.items_records import JsonRecord

ROOT: Final = Path(__file__).resolve().parents[2]
PUBLIC: Final = ROOT / "docs/v8/artenv1/reviews/items_states"
PRIVATE: Final = Path("/Users/rexxa/work/artenv1-data/reviews/items_states")
MANIFEST: Final = ROOT / "docs/v8/artenv1/items_manifest.json"
LIMIT: Final = 1_500_000
BUCKETS: Final = ("1", "2–8", "9+")
STATES: Final = {
    "meat": "fresh", "meat_spoiled": "spoiled", "raw_pot": "unfired",
    "clay_pot_empty": "fired_empty", "clay_pot_filled": "fired_filled",
    "cured_hide": "cured", "hide": "untreated",
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_rows(manifest: JsonRecord) -> list[JsonRecord]:
    kinds = {row.text("asset_id"): row.text("sim_kind") for row in manifest.records("item_kinds")}
    kinds.update({"log_pile_medium": "Log", "log_pile_large": "Log", "flint_pile": "Flint",
                  "clay_pot_filled": "ClayPot", "berry_basket": "LegacyFood", "meat_spoiled": "Meat"})
    assets = manifest.records("assets")
    if len(assets) != 22:
        message = "The frozen packet requires exactly 22 existing asset entries"
        raise ValueError(message)
    rows: list[JsonRecord] = []
    for asset in assets:
        name = asset.text("id")
        for bucket, source in zip(BUCKETS, asset.strings("count_buckets"), strict=True):
            rows.append(JsonRecord({"asset_id": name, "expected_kind": kinds[name],
                                    "expected_state": STATES.get(name, "not_applicable"),
                                    "expected_bucket": bucket, "source_path": source,
                                    "entry_role": "existing_count_variant"}))
    spoiled = manifest.record("spoiled_meat")
    rows.append(JsonRecord({"asset_id": "meat_spoiled", "expected_kind": "Meat",
                            "expected_state": "spoiled", "expected_bucket": "1",
                            "source_path": spoiled.text("path"), "entry_role": "derived_state"}))
    if sha256(ROOT / spoiled.text("path")) != spoiled.text("sha256"):
        message = "The derived spoiled-meat source differs from its manifest"
        raise ValueError(message)
    return rows


def fingerprint(row: JsonRecord) -> None:
    path = ROOT / row.text("source_path")
    size = path.stat().st_size
    if size > LIMIT:
        message = f"Source exceeds the decimal file cap: {path}"
        raise ValueError(message)
    with Image.open(path) as opened:
        rgba = opened.convert("RGBA")
    if min(rgba.size) < 32:
        message = f"Source would require enlargement: {path}"
        raise ValueError(message)
    row.fields.update({"source_sha256": sha256(path), "source_bytes": size,
                       "source_canvas": list(rgba.size),
                       "source_rgba_sha256": hashlib.sha256(rgba.tobytes()).hexdigest()})


def render(rows: list[JsonRecord]) -> JsonRecord:
    columns, cell_width, cell_height, heading = 8, 100, 88, 28
    canvas = Image.new("RGB", (columns * cell_width, heading + ((len(rows) + columns - 1) // columns) * cell_height), "#4a4b3d")
    draw = ImageDraw.Draw(canvas)
    draw.text((14, 8), "Native 32px item observations - no enlargement", fill="#f4ead5")
    cells: list[JsonRecord] = []
    for index, row in enumerate(rows):
        column, grid_row = index % columns, index // columns
        x, y = column * cell_width, heading + grid_row * cell_height
        source = ROOT / row.text("source_path")
        with Image.open(source) as opened:
            sprite = opened.convert("RGBA")
        sprite.thumbnail((32, 32), Image.Resampling.LANCZOS)
        left, top = x + (cell_width - sprite.width) // 2, y + 12
        canvas.paste(sprite, (left, top), sprite)
        draw.text((x + 25, y + 56), row.text("opaque_id"), fill="#f4ead5")
        cell = JsonRecord({"opaque_id": row.text("opaque_id"), "row": grid_row, "column": column,
                           "cell_box": [x, y, x + cell_width, y + cell_height],
                           "sprite_box": [left, top, left + sprite.width, top + sprite.height]})
        cells.append(cell)
        row.fields["display_geometry"] = cell.fields
        row.fields["display_rgba_sha256"] = hashlib.sha256(sprite.tobytes()).hexdigest()
    path = PUBLIC / "blind_states_32px.png"
    canvas.save(path, optimize=True)
    if path.stat().st_size > LIMIT:
        message = "Blind board exceeds the decimal file cap"
        raise ValueError(message)
    return JsonRecord({"schema_version": 1, "board": str(path.relative_to(ROOT)),
                       "sha256": sha256(path), "bytes": path.stat().st_size,
                       "canvas_px": list(canvas.size), "entries": len(rows),
                       "sprite_max_px": 32, "cell_px": [cell_width, cell_height],
                       "resize": "proportional Lanczos reduction only; no alpha crop or enlargement",
                       "cells": [cell.fields for cell in cells], "answer_key": "external_only"})


def main() -> None:
    PUBLIC.mkdir(parents=True, exist_ok=True)
    PRIVATE.mkdir(parents=True, exist_ok=True)
    manifest_hash = sha256(MANIFEST)
    rows = source_rows(JsonRecord.read(MANIFEST))
    for row in rows:
        fingerprint(row)
    seed_path = PRIVATE / "randomization.json"
    if seed_path.exists():
        seed = JsonRecord.read(seed_path).text("seed")
    else:
        seed = secrets.token_hex(24)
        _ = seed_path.write_text(json.dumps({"seed": seed}, indent=2) + "\n")
    rng = random.Random(seed)
    rng.shuffle(rows)
    used_ids: set[str] = set()
    for row in rows:
        opaque_id = f"Q{rng.getrandbits(24):06X}"
        while opaque_id in used_ids:
            opaque_id = f"Q{rng.getrandbits(24):06X}"
        used_ids.add(opaque_id)
        row.fields["opaque_id"] = opaque_id
    packet = render(rows)
    duplicate_groups: dict[str, list[str]] = {}
    for row in rows:
        duplicate_groups.setdefault(row.text("source_rgba_sha256"), []).append(row.text("opaque_id"))
        if sha256(ROOT / row.text("source_path")) != row.text("source_sha256"):
            message = "An item source changed while the packet was being built"
            raise RuntimeError(message)
    if sha256(MANIFEST) != manifest_hash:
        message = "Item manifest changed while the packet was being built"
        raise RuntimeError(message)
    key = JsonRecord({"schema_version": 1, "manifest_sha256": manifest_hash,
                      "board_sha256": packet.text("sha256"), "entries": [row.fields for row in rows],
                      "count_by_bucket": dict(Counter(row.text("expected_bucket") for row in rows)),
                      "duplicate_pixel_groups": {key: ids for key, ids in duplicate_groups.items() if len(ids) > 1},
                      "duplicate_policy": "All 66 existing entry/bucket combinations remain, including reused identical log buckets; one derived spoiled-meat state is additional.",
                      "scoring": "Record kind, quantity bucket and applicable visible state separately; no invented acceptance threshold or asset-verdict mutation."})
    _ = (PRIVATE / "answer_key.json").write_text(json.dumps(key.fields, indent=2, ensure_ascii=False) + "\n")
    _ = (PUBLIC / "packet_manifest.json").write_text(json.dumps(packet.fields, indent=2) + "\n")
    print(json.dumps({"entries": len(rows), "board_sha256": packet.text("sha256"),
                      "duplicate_pixel_groups": len(key.record("duplicate_pixel_groups").fields)}))


if __name__ == "__main__":
    main()
