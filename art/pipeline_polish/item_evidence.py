# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: PYTHONPATH=. python -m art.pipeline_polish.item_evidence
"""Finalize item provenance without rewriting frozen asset or packet images."""

from __future__ import annotations

from dataclasses import asdict
from pathlib import Path

import numpy as np
from PIL import Image
from scipy.spatial.distance import pdist

from art.pipeline2.judge import ModelScores, judge, load_calibration
from art.pipeline_env1.items_records import JsonRecord, json_sequence, read_json
from art.pipeline_polish.item_build import (
    CALIBRATION,
    DATA,
    DOCS,
    ROOT,
    digest,
    save_json,
)


def length_check(assets: list[JsonRecord]) -> JsonRecord:
    """Measure whole-object spans as upper bounds on spear shaft length."""
    rows: list[JsonRecord] = []
    spans: dict[str, float] = {}
    for row in assets:
        if row.text("id") in {"spear", "stone_axe", "ground_axe"}:
            with Image.open(ROOT / row.text("path")) as image:
                points = np.argwhere(np.asarray(image.convert("RGBA"))[:, :, 3] >= 128)
            spans[row.text("id")] = float(pdist(points).max())
            rows.append(
                JsonRecord(
                    {
                        "id": row.text("id"),
                        "path": row.text("path"),
                        "sha256": row.text("sha256"),
                        "whole_object_span_px": spans[row.text("id")],
                    }
                )
            )
    spear = spans["spear"]
    for row in rows:
        row.fields["spear_total_to_item_ratio"] = spear / spans[row.text("id")]
    return JsonRecord(
        {
            "method": "Alpha>=128 span at64px; spear total/axe ratios below2 prove shaft requirement failure.",
            "results": [r.fields for r in rows],
            "requirement_pass": False,
        }
    )


def verify_packet(packet: Path) -> JsonRecord:
    """Reject changes to dispatched boards, keys or any source sprite."""
    info = JsonRecord.read(packet / "packet_manifest.json")
    verify_hash(packet / "answer_key.json", info.text("answer_key_sha256"))
    for board in info.records("boards"):
        verify_hash(Path(board.text("path")), board.text("sha256"))
    for value in json_sequence(read_json(packet / "answer_key.json")):
        entry = JsonRecord.parse(value)
        verify_hash(ROOT / entry.text("path"), entry.text("sha256"))
    return info


def verify_hash(path: Path, expected: str) -> None:
    """Fail evidence generation on any changed source bytes."""
    if digest(path) != expected:
        message = f"Frozen artifact hash mismatch: {path}"
        raise RuntimeError(message)


def main() -> None:
    """Join exact-hash model evidence, frozen numeric checks and provenance."""
    records = sorted((JsonRecord.read(p) for p in DATA.glob("call_*.json")), key=lambda row: row.integer("call_index"))
    cards = [r for r in records if r.text("kind") == "model_scorecard"]
    transcriptions = {t.text("raw_sha256"): t for c in cards for t in c.records("transcription")}
    calibration = load_calibration(CALIBRATION)
    for row in records:
        raw = Path(row.text("raw"))
        if digest(raw) != row.text("raw_sha256") or digest(Path(row.text("tool_source"))) != row.text("tool_sha256"):
            message = f"Provenance mismatch: {raw}"
            raise RuntimeError(message)
        for ref in row.records("refs"):
            if digest(Path(ref.text("path"))) != ref.text("sha256"):
                message = f"Reference changed: {ref.text('path')}"
                raise RuntimeError(message)
        if row.text("kind") == "asset":
            review = transcriptions.get(row.text("raw_sha256"))
            model = (
                None
                if review is None
                else ModelScores(
                    review.integer("medium"),
                    review.integer("line"),
                    review.integer("light"),
                    review.integer("palette"),
                    review.text("sentence"),
                )
            )
            result = judge(raw, calibration, model, family="items", model_family_verified=review is not None)
            row.fields["numeric"] = JsonRecord.parse(asdict(result)).fields
            target = ROOT / f"art/style05/polish/items/{row.text('id')}_try{row.integer('attempt')}.png"
            row.fields.update(normalized=str(target.relative_to(ROOT)), normalized_sha256=digest(target))
        save_json(DATA / f"call_{row.integer('call_index')}.json", row.fields)
    save_json(DATA / "ledger.json", [r.fields for r in records])
    save_json(DOCS / "items_generation_ledger.json", [r.fields for r in records])
    save_json(DOCS / "items_model_reviews.json", [c.fields for c in cards])

    manifest = JsonRecord.read(DOCS / "items_manifest.json")
    assets = manifest.records("assets")
    baseline = {
        r.text("id"): r for r in JsonRecord.read(ROOT / "docs/v8/artenv1/items_manifest.json").records("assets")
    }
    for asset in assets:
        if digest(ROOT / asset.text("path")) != asset.text("sha256"):
            message = f"Frozen asset changed: {asset.text('id')}"
            raise RuntimeError(message)
        if asset.flag("reused"):
            old = baseline[asset.text("id")]
            asset.fields["style_accepted"] = old.fields.get("style_accepted", asset.flag("numeric_pass"))
            asset.fields["model_status"] = "INHERITED_ENV1_EVIDENCE"
            continue
        attempts = [r for r in records if r.text("kind") == "asset" and r.text("id") == asset.text("id")]
        chosen = next(r for r in attempts if r.integer("attempt") == asset.integer("selected_attempt"))
        verdict = chosen.record("numeric")
        asset.fields.update(
            attempts=[r.fields for r in attempts],
            numeric=verdict.fields,
            model_status=verdict.text("model_status"),
            style_accepted=verdict.flag("passed"),
        )
    manifest.fields.update(
        assets=[a.fields for a in assets],
        generation_calls=len(records),
        asset_calls=sum(r.text("kind") == "asset" for r in records),
        scorecard_calls=len(cards),
        failed_tool_calls=0,
        numeric_pass_count=sum(a.flag("numeric_pass") for a in assets),
        style_pass_count=sum(a.flag("style_accepted") for a in assets),
        final_asset_mutation_allowed=False,
    )
    blind = JsonRecord.read(DOCS / "blind_items_round1.json")
    manifest.fields["blind"] = {"evidence": "docs/v8/artpolish/blind_items_round1.json",
                                "sha256": digest(DOCS / "blind_items_round1.json"),
                                "passed": blind.flag("passed")}
    save_json(DOCS / "items_manifest.json", manifest.fields)
    old_length = JsonRecord.read(DOCS / "items_length_check.json")
    final_length = length_check(assets)
    final_length.fields["round0"] = old_length.fields.get("round0", old_length.fields)
    save_json(DOCS / "items_length_check.json", final_length.fields)

    packets = [verify_packet(DATA / f"round{number}").fields for number in (0, 1)]
    files = [*(ROOT / "art/style05/polish/items").glob("*.png"), ROOT / "art/boards/items_owner.png"]
    oversized = [str(p.relative_to(ROOT)) for p in files if p.stat().st_size > 1_500_000]
    save_json(
        DOCS / "items_integrity.json",
        {
            "raw_tool_pairs": len(records),
            "references_checked": True,
            "frozen_packets": packets,
            "retained_byte_identity": [a.text("id") for a in assets if a.flag("reused")],
            "max_artifact_bytes": max(p.stat().st_size for p in files),
            "oversized": oversized,
            "selected_asset_count": len(assets),
            "owner_board": "art/boards/items_owner.png",
            "owner_board_sha256": digest(ROOT / "art/boards/items_owner.png"),
        },
    )


if __name__ == "__main__":
    main()
