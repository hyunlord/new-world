# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest"]
# ///
# How to run: uv run --with pytest --with pillow --with numpy python -m pytest art/compositor/test_tone.py
"""Behavioral boundaries for ground readability and hearth value restoration."""
from __future__ import annotations

import numpy as np
from PIL import Image

from art.compositor.tone import GroundLimit, NightLight, cap_ground, night_value


def test_cap_enforces_limits_on_bright_saturated_high_contrast_ground() -> None:
    # Given a bright, highly saturated repeating ground tile.
    pixels = np.zeros((64, 64, 3), dtype=np.uint8)
    pixels[::2] = [255, 240, 0]
    pixels[1::2] = [0, 40, 5]
    source = Image.fromarray(pixels)
    # When the registered transform caps this ground against a person anchor.
    result = cap_ground(source, GroundLimit(.18))
    # Then every independent metric satisfies the frozen limits.
    assert result.passed
    assert result.after.luminance_mean <= .40
    assert result.after.luminance_std <= .18 * .60
    assert result.after.saturation_mean <= .30


def test_night_preserves_hue_and_restores_hearth_value_at_five_tile_radius() -> None:
    # Given a colour and a hearth at the centre of a 12-tile image.
    source = Image.new("RGBA", (768, 768), (200, 100, 40, 255))
    # When the night renderer applies its declared five-tile light radius.
    output = night_value(source, NightLight(((384, 384),), 64))
    # Then hearth centre is unchanged and radius edge has exactly 55% value.
    assert output.getpixel((384, 384)) == (200, 100, 40, 255)
    assert output.getpixel((704, 384)) == (110, 55, 22, 255)
    assert output.getpixel((0, 0)) == (110, 55, 22, 255)
