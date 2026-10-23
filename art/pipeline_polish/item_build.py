# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: PYTHONPATH=. python -m art.pipeline_polish.item_build 0
"""Preserve item attempts, measure frozen v2, and create literal-size boards."""

from __future__ import annotations

import hashlib
import json
import random
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw, ImageFont

from art.pipeline2.judge import judge, load_calibration
from art.pipeline_env1.items_prepare import normalize
from art.pipeline_env1.items_records import JsonRecord, JsonValue

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path("/Users/rexxa/work/artpolish-data/items")
DOCS: Final = ROOT / "docs/v8/artpolish"
DEST: Final = ROOT / "art/style05/polish/items"
CALIBRATION: Final = ROOT / "docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json"
COHORT_SIZE: Final = 22
REGENERATE: Final = frozenset(
    (
        "firewood",
        "ground_axe",
        "cured_hide",
        "hide_wrap",
        "fur_cloak",
        "wolf_hide",
        "raw_pot",
        "flint_nodule",
        "flint_pile",
        "log_pile_medium",
        "log_pile_large",
        "flint_blade",
        "clay",
    ),
)


def digest(path: Path) -> str:
    """Bind an artifact to its unchanged bytes."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save_json(path: Path, value: JsonValue) -> None:
    """Write inspectable UTF-8 evidence."""
    _ = path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")


def measure_attempts() -> list[JsonRecord]:
    """Every completed call retains its own provenance and numeric verdict."""
    records = sorted((JsonRecord.read(p) for p in DATA.glob("call_*.json")), key=lambda row: row.integer("call_index"))
    calibration = load_calibration(CALIBRATION)
    for row in records:
        if row.text("status") != "COMPLETED" or row.text("kind") != "asset":
            continue
        raw = Path(row.text("raw"))
        if digest(raw) != row.text("raw_sha256"):
            message = f"Raw identity mismatch: {raw}"
            raise RuntimeError(message)
        verdict = asdict(judge(raw, calibration, family="items"))
        row.fields["numeric"] = JsonRecord.parse(verdict).fields
        target = DEST / f"{row.text('id')}_try{row.integer('attempt')}.png"
        normalize(raw).save(target, optimize=True)
        row.fields["normalized"] = str(target.relative_to(ROOT))
        row.fields["normalized_sha256"] = digest(target)
    save_json(DATA / "ledger.json", [r.fields for r in records])
    return records


def selected_assets(records: list[JsonRecord], round_number: int) -> list[JsonRecord]:
    """Apply preregistered round selection, retaining the fixed ENV1 cohort."""
    override_path = DATA / f"selection_round{round_number}.json"
    overrides = JsonRecord.read(override_path) if override_path.exists() else JsonRecord({})
    assets = JsonRecord.read(ROOT / "docs/v8/artenv1/items_manifest.json").records("assets")
    if len(assets) != COHORT_SIZE or len({a.text("id") for a in assets}) != COHORT_SIZE:
        message = "Expected exactly22 unique ENV1 item images"
        raise RuntimeError(message)
    result: list[JsonRecord] = []
    for old in assets:
        name = old.text("id")
        row = JsonRecord(
            {"id": name, "baseline_path": old.text("path"), "baseline_sha256": digest(ROOT / old.text("path"))},
        )
        if name in REGENERATE or name in overrides.fields:
            attempts = [
                r
                for r in records
                if r.text("kind") == "asset" and r.text("id") == name and r.text("status") == "COMPLETED"
            ]
            if not attempts:
                message = f"Missing required regenerated item: {name}"
                raise RuntimeError(message)
            passing = [r for r in attempts if r.record("numeric").flag("numeric_pass")]
            chosen = passing[0] if passing else attempts[-1]
            if name in overrides.fields:
                chosen = next(r for r in attempts if r.integer("attempt") == overrides.integer(name))
            row.fields.update(
                path=chosen.text("normalized"),
                sha256=chosen.text("normalized_sha256"),
                selected_attempt=chosen.integer("attempt"),
                reused=False,
                numeric_pass=chosen.record("numeric").flag("numeric_pass"),
                raw=chosen.text("raw"),
                raw_sha256=chosen.text("raw_sha256"),
                attempts=[r.fields for r in attempts],
                model_status="PENDING_SCORECARD",
            )
        else:
            row.fields.update(
                path=old.text("path"),
                sha256=digest(ROOT / old.text("path")),
                reused=True,
                numeric_pass=old.fields.get(
                    "prior_numeric_pass",
                    old.record("numeric").fields["numeric_pass"] if "numeric" in old.fields else False,
                ),
                inherited_status=old.text("status"),
                baseline_evidence="docs/v8/artenv1/items_manifest.json",
            )
        result.append(row)
    return result


def sprite_at(row: JsonRecord, size: int) -> Image.Image:
    """Load the selected sprite at its actual review size."""
    with Image.open(ROOT / row.text("path")) as opened:
        image = opened.convert("RGBA")
    return image.resize((size, size), Image.Resampling.LANCZOS)


def ground() -> Image.Image:
    """Use frozen calm meadow terrain, identically tiled for every specimen."""
    with Image.open(ROOT / "art/style05/ground_v2/meadow.png") as opened:
        return opened.convert("RGB").resize((64, 64), Image.Resampling.LANCZOS)


def ground_panel(width: int, height: int) -> Image.Image:
    """Tile the same calm ground without modifying sprite pixels."""
    tile = ground()
    panel = Image.new("RGB", (width, height))
    for y in range(0, height, 64):
        for x in range(0, width, 64):
            panel.paste(tile, (x, y))
    return panel


def boards(assets: list[JsonRecord], round_number: int) -> None:
    """Render the owner sheet and an opaque-ID review packet."""
    owner = Image.new("RGB", (1080, 1056), "#e5dcc8")
    draw = ImageDraw.Draw(owner)
    draw.text((20, 12), "ART-POLISH / 22 ITEMS / 32 px + 64 px / OWNER CHOICE PENDING", fill="#33271d")
    for index, row in enumerate(assets):
        x, y = index % 4 * 270, 44 + index // 4 * 168
        owner.paste(ground_panel(246, 100), (x + 12, y))
        for size, offset in ((32, 30), (64, 134)):
            sprite = sprite_at(row, size)
            owner.paste(sprite, (x + offset, y + (100 - size) // 2), sprite)
        draw.text((x + 14, y + 105), row.text("id"), fill="#33271d")
        label = "ENV1 retained" if row.flag("reused") else f"POLISH try{row.integer('selected_attempt')}"
        draw.text((x + 14, y + 122), label, fill="#33271d")
        draw.text((x + 14, y + 137), "numeric PASS" if row.flag("numeric_pass") else "numeric FAIL", fill="#33271d")
    owner.save(ROOT / "art/boards/items_owner.png", optimize=True)
    packet = DATA / f"round{round_number}"
    packet.mkdir(exist_ok=True)
    keys: list[JsonValue] = []
    for size in (32, 64):
        ordered = list(assets)
        rng = random.Random(2026102400 + round_number * 100 + size)
        rng.shuffle(ordered)
        identifiers = rng.sample(range(1000, 9999), 22)
        board = Image.new("RGB", (768, 456), "#e5dcc8")
        ink = ImageDraw.Draw(board)
        ink.text((12, 9), f"{size}px / first name only / actual-size viewing", fill="#33271d")
        for index, row in enumerate(ordered):
            x, y = index % 6 * 128, 32 + index // 6 * 106
            board.paste(ground_panel(112, 78), (x + 8, y + 20))
            sprite = sprite_at(row, size)
            board.paste(sprite, (x + (128 - size) // 2, y + 20 + (78 - size) // 2), sprite)
            identifier = f"Q{identifiers[index]}"
            ink.text((x + 12, y + 1), identifier, fill="#33271d", font=ImageFont.load_default(size=14))
            keys.append(
                {
                    "id": identifier,
                    "size": size,
                    "asset": row.text("id"),
                    "path": row.text("path"),
                    "sha256": row.text("sha256"),
                },
            )
        board.save(packet / f"items_{size}px.png", optimize=True)
    save_json(packet / "answer_key.json", keys)
    save_json(
        packet / "packet_manifest.json",
        {
            "round": round_number,
            "ground": "art/style05/ground_v2/meadow.png",
            "ground_sha256": digest(ROOT / "art/style05/ground_v2/meadow.png"),
            "boards": [
                {
                    "size": size,
                    "path": str(packet / f"items_{size}px.png"),
                    "sha256": digest(packet / f"items_{size}px.png"),
                }
                for size in (32, 64)
            ],
            "answer_key_sha256": digest(packet / "answer_key.json"),
        },
    )


def main() -> None:
    """Prepare one authorized blind round, refusing frozen packet replacement."""
    round_number = int(sys.argv[1])
    if round_number not in (0, 1):
        message = "Only blind rounds0 and1 are authorized"
        raise ValueError(message)
    if (DATA / f"round{round_number}/packet_manifest.json").exists():
        message = "Blind packet already frozen; do not overwrite a dispatched round"
        raise FileExistsError(message)
    DEST.mkdir(parents=True, exist_ok=True)
    records = measure_attempts()
    assets = selected_assets(records, round_number)
    save_json(
        DOCS / "items_manifest.json",
        {
            "version": 1,
            "owner_approval": "PENDING_OWNER_CHOICE",
            "assets": [a.fields for a in assets],
            "calibration": str(CALIBRATION.relative_to(ROOT)),
            "calibration_sha256": digest(CALIBRATION),
            "selection_rule": ("preregistered per-round overrides, else earliest numeric pass or latest fail; "
                               "D6 retained cohort fixed"),
            "operations": ["generated alpha preserved", "alpha-bounds crop", "uniform Lanczos fit58px", "center64px"],
            "generation_calls": len(records),
            "blind_round": round_number,
        },
    )
    boards(assets, round_number)
    print(
        json.dumps(
            {
                "assets": len(assets),
                "numeric_pass": sum(a.flag("numeric_pass") for a in assets),
                "calls": len(records),
                "packet": str(DATA / f"round{round_number}"),
            },
        ),
    )


if __name__ == "__main__":
    main()
