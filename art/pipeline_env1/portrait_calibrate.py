"""Freeze portrait-family crop evidence using the existing judge v2 formula."""
from __future__ import annotations

import hashlib
import json
from dataclasses import asdict
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw
from pydantic import BaseModel

from art.pipeline2.judge import (
    Anchor,
    Calibration,
    Fold,
    Verdict,
    bounds,
    compare,
    judge,
    measure,
)

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = ROOT.parent / "artenv1-data" / "portraits"
DOCS: Final = ROOT / "docs/v8/artenv1"
BOXES: Final = (
    ("portrait_full", (0, 0, 1254, 1254)),
    ("portrait_bust", (150, 20, 1254, 1254)),
    ("portrait_head_shoulders", (250, 20, 1050, 1150)),
)


class Look(BaseModel):
    look: str


class SourceManifest(BaseModel):
    accepted_looks: list[Look]


def main() -> None:
    """Write calibration, held-out checks and retained historical controls."""
    DATA.mkdir(parents=True, exist_ok=True)
    references = DATA / "references"
    references.mkdir(exist_ok=True)
    anchors: list[Anchor] = []
    with Image.open(ROOT / "art/style05/anchors/portrait.png") as source:
        for name, box in BOXES:
            target = references / f"{name}.png"
            source.crop(box).save(target)
            anchors.append(Anchor(name, "portrait", str(target), hashlib.sha256(target.read_bytes()).hexdigest(), measure(target)))
    folds: list[Fold] = []
    for held in anchors:
        checks = compare(held.metrics, bounds("portrait", [row for row in anchors if row.name != held.name]))
        folds.append(Fold(held.name, "portrait", all(checks.values()), checks))
    calibration = Calibration(anchors, [bounds("portrait", anchors)], folds)
    _ = (DOCS / "portrait_calibration.json").write_text(json.dumps(asdict(calibration), indent=2) + "\n")
    negatives: list[Verdict] = []
    for name in ("negative_painterly.png", "negative_woodcut.png", "negative_photo_try1.png"):
        path = ROOT.parent / "arttrial2-data" / name
        negatives.append(judge(path, calibration, family="portrait"))
    _ = (DOCS / "portrait_negative_controls.json").write_text(json.dumps([asdict(row) for row in negatives], indent=2) + "\n")
    manifest = SourceManifest.model_validate_json((ROOT / "art/character/route_i/MANIFEST.json").read_text())
    looks = [entry.look for entry in manifest.accepted_looks]
    board = Image.new("RGB", (8 * 260, 5 * 480), "#e5dcc8")
    draw = ImageDraw.Draw(board)
    for index, look in enumerate(looks):
        x, y = index % 8 * 260, index // 8 * 480
        with Image.open(ROOT / "art/character/route_i" / look / "S/idle.png") as sprite:
            shown = sprite.resize((240, 336), Image.Resampling.NEAREST)
            board.paste(shown, (x + 10, y + 25), shown)
        draw.text((x + 5, y + 365), f"{index + 1:02d} {look}", fill="#33271d")
    board.save(references / "adopted34_identity_board.png")
    print(json.dumps({"loo": [row.passed for row in folds], "negative_rejections": [not row.numeric_pass for row in negatives]}))


if __name__ == "__main__":
    main()
