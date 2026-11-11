# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: python -m art.climate.grounds.process
"""Measure B2 ground attempts using existing tone, seam and style instruments."""

from __future__ import annotations

import json
from dataclasses import asdict
from pathlib import Path
from typing import ClassVar, Final

from PIL import Image, ImageDraw
from pydantic import BaseModel, ConfigDict, Field

from art.flora.normalize import digest
from art.pipeline2.ground import measure
from art.pipeline2.ground_v2 import cap_tone, luminance, tone
from art.pipeline2.judge import foreground, judge, load_calibration

ROOT: Final = Path(__file__).resolve().parents[3]
DATA: Final = Path("/Users/rexxa/work/art-batch/B2/grounds")


class Generation(BaseModel):
    """Parse immutable existing generation records at the evidence boundary."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    id: str
    attempt: int = Field(ge=1, le=3)
    prompt: str
    refs: tuple[Path, ...]
    source: Path
    raw: Path
    seed: int | None


def main() -> None:
    """Write diagnostics and native-scale repeat boards, never auto-adopt."""
    rgb, mask = foreground(ROOT / "art/style05/anchors/person.png", 512)
    cap = float(luminance(rgb)[mask].std()) * 0.6
    calibration = load_calibration(ROOT / "docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json")
    output = DATA / "working"
    output.mkdir(exist_ok=True)
    records = sorted((DATA / "records").glob("*_try*.json"))
    summary = []
    for path in records:
        row = Generation.model_validate_json(path.read_bytes())
        if digest(row.source) != digest(row.raw):
            message = f"Original mismatch: {row.id}"
            raise ValueError(message)
        with Image.open(row.raw) as opened:
            raw = opened.convert("RGB").resize((64, 64), Image.Resampling.LANCZOS)
            alpha = opened.convert("RGBA").getchannel("A").getextrema()
        tile, contrast, chroma = cap_tone(raw, cap)
        seam = measure(tile)
        after = tone(tile)
        name = f"{row.id}_try{row.attempt}"
        tile.save(output / f"{name}.png")
        board = Image.new("RGB", (384, 212), "#ddd8ca")
        for y in range(3):
            for x in range(3):
                board.paste(tile, (x * 64, y * 64 + 20))
        board.paste(tile.resize((128, 128), Image.Resampling.NEAREST), (224, 20))
        ImageDraw.Draw(board).text((4, 3), name, fill="black")
        board.save(output / f"{name}_repeat.png")
        result = {
            "id": row.id,
            "attempt": row.attempt,
            "raw": str(row.raw),
            "raw_sha256": digest(row.raw),
            "record_sha256": digest(path),
            "refs": [{"path": str(ref), "sha256": digest(ref)} for ref in row.refs],
            "tile": str(output / f"{name}.png"),
            "tile_sha256": digest(output / f"{name}.png"),
            "alpha_extrema": alpha,
            "seam": asdict(seam),
            "before": asdict(tone(raw)),
            "after": asdict(after),
            "contrast_factor": contrast,
            "chroma_factor": chroma,
            "tone_pass": after.luminance_std <= cap and after.saturation_mean <= 0.35,
            "style": asdict(judge(row.raw, calibration, family="ground")),
            "adoption": "REVIEW_REQUIRED",
        }
        summary.append(result)
    (DATA / "diagnostics.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(f"Measured {len(summary)} attempts; no automatic adoption")


if __name__ == "__main__":
    main()
