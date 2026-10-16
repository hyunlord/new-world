# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow"]
# ///
# Run: python3 tools/revitalize/contact_sheet.py [--baseline DIR --final DIR]
"""Pair baseline and final captures only at matching ticks and framings."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image, ImageDraw


def capture_keys(directory: Path) -> set[tuple[int, str]]:
    """Read the completed run's manifest so stale PNGs cannot enter the sheet."""
    manifest = json.loads((directory / "manifest.json").read_text(encoding="utf-8"))
    if manifest["schema_version"] != 1:
        raise RuntimeError("Unsupported capture manifest schema")
    return {
        (row["tick"], row["framing"])
        for row in manifest["captures"]
        if row["framing"] in ("F1", "F2")
    }


def contact_sheet(baseline: Path, final: Path, output: Path) -> None:
    """Build four honest side-by-side comparisons, never substitute another tick."""
    keys = sorted(capture_keys(baseline) & capture_keys(final))
    expected = [(0, "F1"), (0, "F2"), (1440, "F1"), (1440, "F2")]
    if keys != expected:
        raise RuntimeError("Missing required baseline/final pairs at ticks 0 and 1440")
    sheet = Image.new("RGB", (960, len(keys) * 294), "#202020")
    draw = ImageDraw.Draw(sheet)
    for row, (tick, framing) in enumerate(keys):
        for column, (plan, directory) in enumerate((("baseline", baseline), ("final", final))):
            name = f"{plan}_t{tick}_{framing}.png"
            with Image.open(directory / name) as image:
                if image.size != (960, 540):
                    raise RuntimeError(f"Expected 960x540 review capture: {name}")
                preview = image.convert("RGB").resize((480, 270), Image.Resampling.NEAREST)
                sheet.paste(preview, (column * 480, row * 294 + 24))
            draw.text((column * 480 + 8, row * 294 + 5), f"{plan}: tick {tick}, {framing}", fill="white")
    output.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(output)


def main() -> None:
    """Use the committed review directories by default."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, default=Path("docs/revitalize/shots/baseline"))
    parser.add_argument("--final", type=Path, default=Path("docs/revitalize/shots/final"))
    parser.add_argument("--out", type=Path, default=Path("docs/revitalize/shots/final/contact_sheet.png"))
    args = parser.parse_args()
    contact_sheet(args.baseline, args.final, args.out)


if __name__ == "__main__":
    main()
