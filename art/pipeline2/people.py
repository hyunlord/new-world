# /// script
# requires-python = ">=3.12"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: uv run art/pipeline2/people.py SOURCE DESTINATION
"""Extract authored magenta-backed cut-outs without redrawing their pixels."""
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image


@dataclass(frozen=True, slots=True)
class Part:
    """Authored crop boundary and normalized rig attachment."""
    name: str
    box: tuple[int, int, int, int]
    pivot: tuple[float, float]


PARTS: Final = (
    Part("head", (20, 0, 325, 331), (0.5, 0.65)),
    Part("torso", (340, 0, 710, 333), (0.5, 0.15)),
    Part("upper_arm_l", (750, 0, 940, 330), (0.35, 0.12)),
    Part("upper_arm_r", (1010, 0, 1200, 330), (0.65, 0.12)),
    Part("forearm_l", (100, 365, 235, 591), (0.4, 0.09)),
    Part("forearm_r", (400, 365, 540, 591), (0.6, 0.09)),
    Part("hand_l", (710, 380, 875, 591), (0.43, 0.1)),
    Part("hand_r", (1010, 380, 1165, 591), (0.57, 0.1)),
    Part("upper_leg_l", (90, 625, 240, 870), (0.4, 0.09)),
    Part("upper_leg_r", (405, 625, 550, 870), (0.6, 0.09)),
    Part("lower_leg_l", (720, 625, 840, 870), (0.5, 0.09)),
    Part("lower_leg_r", (1030, 625, 1155, 870), (0.5, 0.09)),
    Part("foot_l", (85, 940, 225, 1185), (0.5, 0.09)),
    Part("foot_r", (400, 940, 540, 1185), (0.5, 0.09)),
)


def cutout(source: Image.Image) -> Image.Image:
    """Chroma key magenta, crop alpha bounds, Lanczos to maximum 256 pixels."""
    rgba = np.asarray(source.convert("RGBA")).copy()
    rgb = rgba[:, :, :3].astype(np.int16)
    key = (rgb[:, :, 0] > 100) & (rgb[:, :, 2] > 100)
    key &= (rgb[:, :, 0] - rgb[:, :, 1] > 65)
    key &= (rgb[:, :, 2] - rgb[:, :, 1] > 65)
    rgba[key, 3] = 0
    image = Image.fromarray(rgba)
    bounds = image.getbbox()
    if bounds is None:
        message = "Chroma extraction produced an empty image"
        raise ValueError(message)
    image = image.crop(bounds)
    image.thumbnail((256, 256), Image.Resampling.LANCZOS)
    return image


def extract(source: Path, destination: Path) -> None:
    """Write fourteen separately cropped parts from the labelled source sheet."""
    destination.mkdir(parents=True, exist_ok=True)
    with Image.open(source) as sheet:
        for part in PARTS:
            cutout(sheet.crop(part.box)).save(destination / f"{part.name}.png")


if __name__ == "__main__":
    extract(Path(sys.argv[1]), Path(sys.argv[2]))
