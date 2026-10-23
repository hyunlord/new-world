# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image", "pydantic"]
# ///
# How to run: uv run --with pillow --with numpy --with scikit-image python -m art.pipeline_env1.built_reviews
"""Bind observed scorecard transcriptions to frozen candidates and retain failures."""
from __future__ import annotations

import json
from collections import Counter
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw
from pydantic import TypeAdapter

from art.pipeline2.ground import Seam
from art.pipeline_env1.built_process import DOC, OUT, RAW, digest
from art.pipeline_env1.items_records import JsonRecord, JsonValue

SEAM_INPUT: Final = TypeAdapter(Seam)

READINGS: Final = {
    "wall_log_straight": "Continuous lashed bark spine reads as a low log wall; generated end texture does not exactly match.",
    "wall_log_corner": "Broad continuous bark elbow matches the straight band width more closely; curved join and texture changes remain visible.",
    "wall_stone_straight": "Two courses of irregular dry stone read as a wall; yellow lichen differs from the corner material.",
    "wall_stone_corner": "Mottled stone elbow now shares the straight strip's material; front-face depth and baked shadow still differ.",
    "door_log_closed": "Thin horizontal gate bar reads closed, but it is too slight beside16px walls and fails numeric wash.",
    "door_log_open": "Two post tops and a north-swung panel leave a clear passage; panel is narrow at half scale.",
    "door_stone_closed": "Timber panel between stone jambs reads shut; front-face emphasis exceeds the pure wall plan.",
    "door_stone_open": "Stone jambs and north-swung narrow panel leave an unambiguous opening.",
    "floor_packed_earth": "Quiet ochre cracked-earth repeat is distinct from rock, but raw style metrics fail after three attempts.",
    "floor_dug_rock": "Broad grey excavation slabs remain visible under the tone cap; raw wash metric fails after three attempts.",
    "bedroll": "Rolled fur pillow and long hide mat read as a sleeping place at1x and half scale.",
    "drying_rack_empty": "Four poles and thin tension ropes surround an empty center, distinct from the filled rack.",
    "drying_rack_hide": "A pale stretched hide fills the lashed frame and distinguishes the active rack.",
    "pottery_bench": "Two clay pots, clay coil and bone scraper identify a pottery work surface.",
    "grinding_stone": "Oval quern basin and oblong hand-stone read as a two-stone grinding work spot.",
    "hearth_pots": "Orange fire and three clay pots read as a firing hearth; individual pots soften at half scale.",
    "hearth_unlit": "Cold ash and crossed dark logs read as extinguished, with no flame or warm glow.",
    "claybank_full": "Broad tawny clay exposure reads fuller than the scraped hollow; muted palette reduces state contrast at half scale.",
    "claybank_depleted": "Scraped grey-brown hollow and small clay rim read depleted.",
    "ford": "Blue shallow water and central gravel crossing read as a ford; repeat check passes along the river.",
    "hill_edge": "A low broken stone ledge separates high and low ground; horizontal repeat passes.",
    "dug_face": "Cut rock face and darker excavated floor are distinct; horizontal boundary exceeds the seam limit.",
    "snow_overlay": "Snow patches contain baked soil and vegetation between them instead of clean transparent gaps; overlay semantics fail.",
}


def write_json(path: Path, value: JsonValue) -> None:
    _ = path.write_text(json.dumps(value, indent=2) + "\n")


def main() -> None:
    manifest_path = DOC / "built_assets.json"
    manifest = JsonRecord.read(manifest_path)
    rows = {row.text("id"): row for row in manifest.records("assets")}
    reviews: list[dict[str, JsonValue]] = []
    for path in sorted(RAW.glob("judge_*.json")):
        record = JsonRecord.read(path)
        if record.fields.get("type") != "image_model_scorecard":
            continue
        if "transcription" not in record.fields:
            raise RuntimeError(f"Unobserved scorecard: {path}")
        frozen_path = RAW / f"{record.text('id')}_candidates.json"
        frozen = JsonRecord.read(frozen_path)
        if digest(Path(frozen.text("board"))) != frozen.text("sha256"):
            raise RuntimeError(f"Judge board changed: {frozen_path}")
        for candidate in frozen.records("candidates"):
            if digest(Path(candidate.text("raw"))) != candidate.text("sha256"):
                raise RuntimeError(f"Judge candidate changed: {candidate.text('id')}")
        record.fields["frozen_candidates"] = frozen.fields
        scorecard = RAW / f"{record.text('id')}_scorecard_raw.png"
        preview = Image.open(scorecard).convert("RGB")
        preview.thumbnail((1500, 1500), Image.Resampling.LANCZOS)
        destination = OUT / f"{record.text('id')}_scorecard.png"
        preview.save(destination, optimize=True)
        record.fields.update({"scorecard": str(scorecard), "scorecard_sha256": digest(scorecard),
                       "preview": str(destination), "preview_sha256": digest(destination),
                       "input_sha256": {ref: digest(Path(ref)) for ref in record.strings("refs")},
                       "mechanism": "builtin image_gen independent scorecard; actual image manually read"})
        for name in record.strings("candidate_ids"):
            row = rows[name]
            candidate = next(value for value in frozen.records("candidates") if value.text("id") == name)
            if row.text("raw_sha256") != candidate.text("sha256"):
                continue
            row.fields["model"] = {"scores": record.record("transcription").record(name).fields, "review_id": record.text("id"),
                            "candidate_raw_sha256": row.text("raw_sha256"), "scorecard_sha256": digest(scorecard)}
        reviews.append(record.fields)
    for row in rows.values():
        row.fields["physical_footprint"] = [1, 1] if row.text("id") in {"bedroll", "pottery_bench", "claybank_full", "claybank_depleted"} else row.fields["footprint"]
        if "numeric" not in row.fields:
            row.fields["status"] = "SEAM_FAIL_CANDIDATE"
            continue
        name = row.text("id")
        model = JsonRecord.parse(row.fields.get("model", {}))
        scores = JsonRecord.parse(model.fields.get("scores", {}))
        model_pass = bool(scores.fields) and all(scores.integer(key) >= 4 for key in ("medium", "line", "light", "palette"))
        row.fields["style_pass"] = row.record("numeric").flag("numeric_pass") and model_pass
        row.fields["reading"] = READINGS[name]
        row.fields["semantic_pass"] = name != "snow_overlay"
        seam_value = row.fields["seam"]
        if seam_value is not None:
            seam = SEAM_INPUT.validate_python(seam_value)
            metrics = {"horizontal": (seam.horizontal_mae, seam.horizontal_ratio),
                       "vertical": (seam.vertical_mae, seam.vertical_ratio)}
            axes = ["horizontal"] if name in {"hill_edge", "dug_face"} else ["vertical"] if name == "ford" else ["horizontal", "vertical"]
            row.fields["required_wrap_axes"] = axes
            row.fields["required_wrap_pass"] = all(metrics[axis][0] <= 18 and metrics[axis][1] <= 2 for axis in axes)
        else:
            row.fields["required_wrap_pass"] = True
        failures: list[str] = []
        if not row.record("numeric").flag("numeric_pass"):
            failures.append("NUMERIC")
        if not model_pass:
            failures.append("MODEL")
        if not row.fields["semantic_pass"]:
            failures.append("SEMANTIC")
        if not row.fields["required_wrap_pass"] or name.startswith("wall_"):
            failures.append("SEAM")
        row.fields["status"] = "ACCEPTED_ASSET" if not failures else "_".join(failures) + "_FAIL_CANDIDATE"
        row.fields["acceptance_scope"] = "Local asset gates only; no scene readability or owner approval claim."
    for row in rows.values():
        write_json(OUT / f"{row.text('id')}.json", row.fields)
    board = Image.new("RGB", (1152, 960), (92, 88, 74))
    draw = ImageDraw.Draw(board)
    source_rows = [row for row in rows.values() if "numeric" in row.fields]
    for index, row in enumerate(source_rows):
        x, y = index % 6 * 192, index // 6 * 240
        draw.text((x + 8, y + 8), row.text("id"), fill="white")
        label = row.text("status").replace("_FAIL_CANDIDATE", " FAIL")
        if row.text("id") == "snow_overlay":
            label = "MULTIPLE GATES FAIL"
        draw.text((x + 8, y + 24), label, fill="white")
        tile = Image.open(row.text("path")).convert("RGBA")
        preview = tile.copy()
        preview.thumbnail((176, 150), Image.Resampling.NEAREST)
        board.paste(preview, (x + (192 - preview.width) // 2, y + 48), preview)
        half = tile.resize((row.integer("width") // 2, row.integer("height") // 2), Image.Resampling.LANCZOS)
        board.paste(half, (x + 8, y + 174), half)
    board.save(OUT / "built_review_board.png", optimize=True)
    ledger: list[dict[str, JsonValue]] = []
    for path in sorted(RAW.glob("*_try[1-3].json")):
        record = JsonRecord.read(path)
        raw = path.with_name(path.stem + "_raw.png")
        record.fields.update({"raw": str(raw), "raw_sha256": digest(raw),
                       "anchor_sha256": {ref: digest(Path(ref)) for ref in record.strings("anchors")}})
        ledger.append(record.fields)
    manifest.fields["freeze"] = {"raster_paths_final": True, "owner_approval": False,
                           "asset_generations": len(ledger), "judge_generations": len(reviews),
                           "generation_calls_total": len(ledger) + len(reviews)}
    manifest.fields["assets"] = [row.fields for row in rows.values()]
    write_json(manifest_path, manifest.fields)
    write_json(DOC / "built_generation_ledger.json", {"asset_generations": ledger, "model_reviews": reviews})
    write_json(DOC / "built_reviews.json", {"mechanism": "actual builtin image-model scorecards", "reviews": reviews,
                                         "status_counts": dict(Counter(row.text("status") for row in rows.values()))})
    print(json.dumps(manifest.fields["freeze"]))


if __name__ == "__main__":
    main()
