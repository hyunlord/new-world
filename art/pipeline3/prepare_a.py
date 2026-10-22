# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline3.prepare_a RAW.png OUTPUT.json
"""Preserve full-canvas coordinates while keying a generated route-A candidate."""
import hashlib
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image


def prepare(raw: Path, sidecar: Path) -> None:
    """Process the supplied sidecar recipe; never fit the subject to its box."""
    metadata = json.loads(sidecar.read_text())
    scale = metadata["scale"]
    size = (96 * scale, 64 * scale) if metadata["frame"] == "sleep" else (64 * scale, 96 * scale)
    with Image.open(raw) as source:
        if source.width * size[1] != source.height * size[0]:
            message = "Generated full-canvas aspect ratio differs from template; reject rather than warp"
            raise ValueError(message)
        pixels = np.array(source.convert("RGBA"))
    rgb = pixels[:, :, :3].astype(np.int16)
    magenta = (rgb[:, :, 0] - rgb[:, :, 1] > 60) & (rgb[:, :, 2] - rgb[:, :, 1] > 60)
    pixels[magenta, 3] = 0
    pixels[pixels[:, :, 3] == 0] = 0
    with Image.fromarray(pixels).resize(size, Image.Resampling.LANCZOS) as resized:
        working = np.array(resized)
    working[working[:, :, 3] < 16] = 0
    target = sidecar.with_suffix(".png")
    Image.fromarray(working).save(target)
    region = {"body": 1, "hair": 2, "top": 3, "item": 5, "carry": 5}[metadata["kind"]]
    mask = np.where(working[:, :, 3] > 0, region, 0).astype(np.uint8)
    Image.fromarray(mask).save(sidecar.parent / metadata["mask"])
    metadata["raw"] = str(raw.resolve())
    metadata["raw_sha256"] = hashlib.sha256(raw.read_bytes()).hexdigest()
    metadata["image_sha256"] = hashlib.sha256(target.read_bytes()).hexdigest()
    metadata["operations"] = [
        "Preserve entire source canvas; reject aspect mismatch; no alpha-bbox fitting",
        "Chroma key R-G>60 and B-G>60; zero RGB where alpha=0",
        f"Lanczos downscale entire canvas to {size[0]}x{size[1]}",
        "Alpha cleanup: alpha<16 to transparent black; no other colour modification",
        f"Code mask: every visible pixel region{region}; no colour quantization",
    ]
    sidecar.write_text(json.dumps(metadata, indent=2) + "\n")


if __name__ == "__main__":
    prepare(Path(sys.argv[1]), Path(sys.argv[2]))
