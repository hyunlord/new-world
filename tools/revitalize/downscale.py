# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow"]
# ///
# Run: python3 tools/revitalize/downscale.py RAW_DIRECTORY OUTPUT_DIRECTORY
"""Validate the capture manifest and write nearest-neighbour review images."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image

PLANS: dict[str, list[tuple[int, str]]] = {
    "baseline": [(0, "F1"), (0, "F2"), (1440, "F1"), (1440, "F2")],
    "final": [
        (0, "F1"), (0, "F2"), (1440, "F1"), (1440, "F2"), (1440, "F3"),
        (1440, "F4"), (2760, "F2N"), (7200, "F1"), (7200, "F2"),
    ],
}


def downscale(source: Path, output: Path) -> None:
    """Reject incomplete capture sets before creating the review directory."""
    manifest = json.loads((source / "manifest.json").read_text(encoding="utf-8"))
    plan = manifest["plan"]
    expected = PLANS[plan]
    rows = manifest["captures"]
    actual = [(row["tick"], row["framing"]) for row in rows]
    if manifest["schema_version"] != 1 or actual != expected:
        raise RuntimeError("Capture manifest is incomplete or uses an unsupported schema")
    for row in rows:
        name = f"{plan}_t{row['tick']}_{row['framing']}.png"
        if row["file"] != name:
            raise RuntimeError(f"Unexpected capture filename: {row['file']}")
        with Image.open(source / name) as image:
            if image.size != (1920, 1080):
                raise RuntimeError(f"Capture has incorrect dimensions: {name}")
            image.verify()
    output.mkdir(parents=True, exist_ok=True)
    for row in rows:
        with Image.open(source / row["file"]) as image:
            image.resize((960, 540), Image.Resampling.NEAREST).save(output / row["file"])
    manifest["review_width"] = 960
    manifest["review_height"] = 540
    (output / "manifest.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def main() -> None:
    """Read explicit input/output paths; never guess a capture succeeded."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    downscale(args.source, args.output)


if __name__ == "__main__":
    main()
