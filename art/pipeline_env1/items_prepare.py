"""Preserve item originals, normalize geometry, and measure frozen judge v2.

Run with the existing arttrial2-data virtual environment and PYTHONPATH=.
Only alpha-bound cropping, proportional resize, centering and duplication occur.
"""

from __future__ import annotations

import hashlib
import json
import re
import shutil
from collections.abc import Mapping
from dataclasses import asdict
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.pipeline2.judge import judge, load_calibration
from art.pipeline_env1.items_records import (
    JsonRecord,
    JsonValue,
    json_sequence,
    read_json,
)

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path("/Users/rexxa/work/artenv1-data/items")
DEST: Final = ROOT / "art/style05/env1/items"
DOCS: Final = ROOT / "docs/v8/artenv1"
CALIBRATION: Final = ROOT / "docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json"
REUSE: Final = (
    "log_pile_small", "log_pile_medium", "log_pile_large", "stone_pile",
    "flint_pile", "flint_blade", "stone_axe", "hide", "berry_basket",
)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def normalize(source: Path) -> Image.Image:
    with Image.open(source) as opened:
        image = opened.convert("RGBA")
    bounds = image.getbbox()
    if bounds is None:
        message = f"Empty alpha in {source}"
        raise RuntimeError(message)
    image = image.crop(bounds)
    image.thumbnail((58, 58), Image.Resampling.LANCZOS)
    tile = Image.new("RGBA", (64, 64))
    tile.alpha_composite(image, ((64-image.width)//2, (64-image.height)//2))
    return tile


def buckets(sprite: Image.Image, name: str) -> list[str]:
    paths: list[str] = []
    layouts = (
        ("1", 64, ((0, 0),)),
        ("2_8", 42, ((3, 3), (16, 16), (3, 20))),
        ("9_plus", 29, ((2, 3), (20, 3), (33, 7), (2, 20), (19, 20),
                         (33, 23), (5, 34), (20, 34), (33, 34))),
    )
    for label, size, offsets in layouts:
        tile = Image.new("RGBA", (64, 64))
        part = sprite.resize((size, size), Image.Resampling.LANCZOS)
        for offset in offsets:
            tile.alpha_composite(part, offset)
        path = DEST / f"{name}_count_{label}.png"
        tile.save(path, optimize=True)
        paths.append(str(path.relative_to(ROOT)))
    return paths


def board(names: list[str], size: int) -> None:
    for blind in (False, True):
        canvas = Image.new("RGB", (720, 80 + 130*((len(names)+5)//6)), "#4a4b3d")
        draw = ImageDraw.Draw(canvas)
        draw.text((20, 15), f"ENV1 items at {size}px: native display size", fill="#f4ead5")
        for index, name in enumerate(names):
            x, y = 20+(index%6)*120, 55+(index//6)*130
            path = DEST / f"{name}.png"
            if not path.exists():
                path = ROOT / "art/style05/env/items" / f"{name}.png"
            with Image.open(path) as opened:
                sprite = opened.convert("RGBA")
            sprite.thumbnail((size, size), Image.Resampling.LANCZOS)
            canvas.paste(sprite, (x+(80-sprite.width)//2, y), sprite)
            draw.text((x, y+70), str(index+1) if blind else name, fill="#f4ead5")
        suffix = "blind" if blind else "keyed"
        canvas.save(DEST / f"items_{size}px_{suffix}.png", optimize=True)


def main() -> None:
    records = [JsonRecord.parse(value) for value in json_sequence(read_json(DATA / "generations.json"))]
    calibration = load_calibration(CALIBRATION)
    by_name: dict[str, JsonRecord] = {}
    attempts: dict[str, list[Mapping[str, JsonValue]]] = {}
    for record in records:
        name = record.text("id")
        match = re.search(r" as (.*?) by default", record.text("output_hint"))
        if match is None:
            message = f"Missing tool output path for {name}"
            raise RuntimeError(message)
        source = Path(match.group(1))
        raw = DATA / f"{name}_try{record.integer('attempt')}_raw.png"
        _ = shutil.copyfile(source, raw)
        record.fields["tool"] = "image_gen__imagegen"
        record.fields["timestamp_semantics"] = "started_at is recorded prompt preparation/pre-call bound; completed_at is observed tool return; internal generation scheduling time is not exposed"
        record.fields["raw"] = str(raw)
        record.fields["raw_sha256"] = digest(raw)
        record.fields["references_sha256"] = {ref: digest(Path(ref)) for ref in record.strings("refs")}
        sprite = normalize(raw)
        path = DEST / f"{name}.png"
        sprite.save(path, optimize=True)
        verdict = judge(raw, calibration, family="items")
        numeric: Mapping[str, JsonValue] = asdict(verdict)
        record.fields["numeric"] = numeric
        _ = (DATA / f"{name}_try{record.integer('attempt')}_numeric.json").write_text(json.dumps(numeric, indent=2)+"\n")
        attempts.setdefault(name, []).append(record.fields)
        row = JsonRecord({
            "id": name, "path": str(path.relative_to(ROOT)), "family": "items",
            "tile_footprint": [1, 1], "canvas_px": [64, 64], "anchor_px": [32, 32],
            "source": str(raw), "source_sha256": digest(raw), "sha256": digest(path),
            "attempts": attempts[name], "numeric": numeric,
            "status": "PENDING_MODEL_REVIEW" if verdict.numeric_pass else "NUMERIC_FAIL",
            "operations": ["generated alpha preserved", "alpha-bounds crop", "proportional Lanczos fit to58px", "center in64px tile"],
            "count_buckets": buckets(sprite, name),
        })
        by_name[name] = row
        _ = (DEST / f"{name}.json").write_text(json.dumps(row.fields, indent=2)+"\n")
    assets = list(by_name.values())
    for name in REUSE:
        path = ROOT / "art/style05/env/items" / f"{name}.png"
        previous = JsonRecord.read(path.with_suffix(".json"))
        with Image.open(path) as source:
            dimensions = list(source.size)
        count_paths = (
            [f"art/style05/env/items/log_pile_{size}.png" for size in ("small", "medium", "large")]
            if name.startswith("log_pile_") else buckets(normalize(path), name)
        )
        assets.append(JsonRecord({
            "id": name, "path": str(path.relative_to(ROOT)), "family": "items",
            "tile_footprint": [1, 1], "canvas_px": dimensions,
            "anchor_px": [dimensions[0]/2, dimensions[1]/2],
            "sha256": digest(path), "attempts": [],
            "status": "REUSED_PRIOR_ACCEPTANCE", "prior_status": previous.text("status"),
            "prior_numeric_pass": previous.record("numeric").flag("numeric_pass"),
            "prior_model": previous.record("numeric").fields["model_scores"],
            "source_metadata": str(path.with_suffix(".json").relative_to(ROOT)),
            "count_buckets": count_paths,
        }))
    blind_order = [row.text("id") for row in assets]
    manifest = {
        "version": 1, "family": "items", "generation_calls": len(records),
        "calibration": str(CALIBRATION.relative_to(ROOT)), "calibration_sha256": digest(CALIBRATION),
        "assets": [row.fields for row in assets], "blind_order": blind_order,
        "stack_note": "1 / 2-8 /9+ use1/3/9 complete-sprite duplicates; logs reuse original3sizes. Buckets are ordinal quantity icons, not literal unit counts inside old pile artwork.",
    }
    _ = (DOCS / "items_manifest.json").write_text(json.dumps(manifest, indent=2)+"\n")
    _ = (DATA / "generations_preserved.json").write_text(json.dumps([record.fields for record in records], indent=2)+"\n")
    board(blind_order, 64)
    board(blind_order, 32)
    print(json.dumps([{row.text("id"): row.text("status")} for row in assets]))


if __name__ == "__main__":
    main()
