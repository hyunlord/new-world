"""Create geometry-only portrait deliverables and raw-resolution judge evidence."""
from __future__ import annotations

import hashlib
import json
from dataclasses import asdict
from pathlib import Path
from typing import ClassVar, Final, Literal

from PIL import Image, ImageDraw
from pydantic import BaseModel, ConfigDict, JsonValue, TypeAdapter

from art.pipeline2.judge import judge, load_calibration

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = ROOT.parent / "artenv1-data" / "portraits"
DOCS: Final = ROOT / "docs/v8/artenv1"
OUT: Final = ROOT / "art/portrait/env1"


class Generation(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    look: str
    attempt: int
    source: Path
    raw: Path
    kind: Literal["art"]


def main() -> None:
    """Keep attempts distinct and emit ID-labelled model-review contact sheets."""
    rows = TypeAdapter(list[Generation]).validate_json((DATA / "ledger.json").read_text())
    calibration = load_calibration(DOCS / "portrait_calibration.json")
    OUT.mkdir(parents=True, exist_ok=True)
    verdicts: list[JsonValue] = []
    numeric_pass = 0
    for row in sorted(rows, key=lambda item: (item.look, item.attempt)):
        target = OUT / f"{row.look}_try{row.attempt}.png"
        with Image.open(row.raw) as source:
            source.convert("RGB").resize((128, 128), Image.Resampling.LANCZOS).save(target)
        result = judge(row.raw, calibration, family="portrait")
        numeric_pass += result.numeric_pass
        verdicts.append(TypeAdapter[JsonValue](JsonValue).validate_python({"look": row.look, "attempt": row.attempt, "raw": str(row.raw),
                         "raw_sha256": hashlib.sha256(row.raw.read_bytes()).hexdigest(),
                         "canonical": str(target.relative_to(ROOT)),
                         "canonical_sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
                         "operations": ["full-frame uniform LANCZOS resize to128x128"],
                         "numeric": asdict(result)}))
    _ = (DOCS / "portrait_numeric.json").write_text(json.dumps(verdicts, indent=2) + "\n")
    for start in range(0, len(rows), 8):
        board_path = DATA / f"model_batch_{start // 8 + 1}.png"
        if board_path.exists():
            continue
        group = sorted(rows, key=lambda item: (item.look, item.attempt))[start:start + 8]
        board = Image.new("RGB", (4 * 320, 2 * 350), "#e5dcc8")
        draw = ImageDraw.Draw(board)
        for index, row in enumerate(group):
            x, y = index % 4 * 320, index // 4 * 350
            with Image.open(row.raw) as raw:
                board.paste(raw.resize((288, 288), Image.Resampling.LANCZOS), (x + 16, y + 15))
            draw.text((x + 10, y + 310), f"P{start + index + 1:02d} {row.look}", fill="#33271d")
            draw.text((x + 10, y + 326), f"try{row.attempt}", fill="#33271d")
        board.save(board_path)
    print(json.dumps({"portraits": len(rows), "numeric_pass": numeric_pass}))


if __name__ == "__main__":
    main()
