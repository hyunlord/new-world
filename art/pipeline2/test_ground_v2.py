# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image", "pytest"]
# ///
# How to run: uv run --with pytest --with pillow --with numpy --with scikit-image python -m pytest art/pipeline2/test_ground_v2.py
"""Guard the ground cap against quantization and background contamination."""
from pathlib import Path

import numpy as np
import pytest
from PIL import Image

from art.pipeline2.ground_v2 import cap_tone, process, tone


def test_contrast_cap_survives_final_eight_bit_rounding() -> None:
    image = Image.fromarray(np.tile(np.array([[255, 0, 0], [0, 255, 255]], dtype=np.uint8), (512, 256, 1)))
    result, _, _ = cap_tone(image, 0.05)
    assert tone(result).luminance_std <= 0.05
    assert tone(result).saturation_mean <= 0.35


def test_person_background_does_not_inflate_cap(tmp_path: Path) -> None:
    person = Image.new("RGBA", (100, 100), (255, 0, 255, 255))
    for x in range(20, 80):
        for y in range(20, 80):
            person.putpixel((x, y), (100 if x < 50 else 150,) * 3 + (255,))
    anchor = tmp_path / "person.png"
    person.save(anchor)
    source = tmp_path / "raw.png"
    Image.new("RGB", (512, 512), (130, 100, 90)).save(source)
    result = process(source, anchor, tmp_path / "out")
    assert result.person_luminance_std == pytest.approx(25 / 255, abs=0.005)
    assert result.tone_pass
    assert (tmp_path / "out" / "repeat_3x3.png").is_file()


def test_saturation_cap_survives_rounding_of_flat_colors() -> None:
    image = Image.new("RGB", (16, 16), (120, 125, 80))
    result, _, _ = cap_tone(image, 0.16)
    assert tone(result).saturation_mean <= 0.35
