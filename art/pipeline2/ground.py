# /// script
# requires-python = ">=3.12"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: uv run art/pipeline2/ground.py /path/to/raw.png /path/to/output
"""Downscale ground candidates and measure preregistered wrap seams."""
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

import numpy as np
from PIL import Image


@dataclass(frozen=True, slots=True)
class Seam:
    """Wrap discontinuity compared with interior adjacent pixels."""
    horizontal_mae: float
    vertical_mae: float
    horizontal_ratio: float
    vertical_ratio: float
    passed: bool


def measure(image: Image.Image) -> Seam:
    """Apply frozen RGB boundary limits to a working-resolution texture."""
    pixels = np.asarray(image.convert("RGB"), dtype=np.float64)
    horizontal = float(np.abs(pixels[:, 0] - pixels[:, -1]).mean())
    vertical = float(np.abs(pixels[0] - pixels[-1]).mean())
    interior_h = float(np.abs(np.diff(pixels, axis=1)).mean())
    interior_v = float(np.abs(np.diff(pixels, axis=0)).mean())
    ratio_h = horizontal / max(interior_h, 1e-9)
    ratio_v = vertical / max(interior_v, 1e-9)
    return Seam(horizontal, vertical, ratio_h, ratio_v,
                horizontal <= 18 and vertical <= 18 and ratio_h <= 2 and ratio_v <= 2)


def mirror_blend(image: Image.Image) -> Image.Image:
    """Blend only 32-pixel edge strips with reflections after two seam failures."""
    pixels = np.asarray(image.convert("RGB"), dtype=np.float64)
    for axis in (1, 0):
        length = pixels.shape[axis]
        distance = np.minimum(np.arange(length), np.arange(length)[::-1])
        weight = 0.25 * (1 + np.cos(np.pi * np.minimum(distance, 32) / 32))
        shape = [1, 1, 1]
        shape[axis] = length
        weight = weight.reshape(shape)
        pixels = pixels * (1 - weight) + np.flip(pixels, axis=axis) * weight
    return Image.fromarray(np.rint(pixels).clip(0, 255).astype(np.uint8))


def process(source: Path, destination: Path, mirror: bool = False) -> Seam:
    """Preserve raw pixels separately and write Lanczos working tile and repeat."""
    destination.mkdir(parents=True, exist_ok=True)
    with Image.open(source) as raw:
        tile = raw.convert("RGB").resize((512, 512), Image.Resampling.LANCZOS)
    if mirror:
        tile = mirror_blend(tile)
    tile.save(destination / "candidate.png")
    repeat = Image.new("RGB", (1536, 1536))
    for y in range(3):
        for x in range(3):
            repeat.paste(tile, (x * 512, y * 512))
    repeat.save(destination / "repeat_3x3.png")
    seam = measure(tile)
    (destination / "seam.json").write_text(json.dumps(asdict(seam), indent=2) + "\n")
    return seam


if __name__ == "__main__":
    print(json.dumps(asdict(process(Path(sys.argv[1]), Path(sys.argv[2]), "--mirror" in sys.argv[3:]))))
