# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy", "pillow", "pytest"]
# ///
# How to run: python3 -m pytest art/climate_colour/test_colour.py -q
"""Colour transport contracts, including circular red hue and protected alpha."""

from __future__ import annotations

import colorsys

import numpy as np
import pytest
from PIL import Image

from art.climate_colour.colour import distribution, recolour


def gradient(hue: float, saturation: float) -> Image.Image:
    """Make an independently defined warm/green colour ramp."""
    pixels = [
        tuple(
            round(value * 255)
            for value in colorsys.hsv_to_rgb(hue + x / 1000, saturation, 0.3 + x / 200)
        )
        + (255,)
        for x in range(100)
    ]
    image = Image.new("RGBA", (10, 10))
    image.putdata(pixels)
    return image


def test_transport_reaches_reference_quantiles_and_is_deterministic() -> None:
    source, reference = gradient(0.15, 0.15), gradient(0.25, 0.65)
    output = recolour(source, reference)
    # Replacing recolour with identity misses S by 0.5 and V by up to 0.004.
    np.testing.assert_allclose(
        distribution(output), distribution(reference), atol=0.012
    )
    assert output.tobytes() == recolour(source, reference).tobytes()
    assert output.tobytes() != source.tobytes()


def test_identity_preserves_every_byte() -> None:
    source = gradient(0.2, 0.6)
    assert recolour(source, source).tobytes() == source.tobytes()


def test_alpha_hidden_rgb_and_canvas_are_untouched() -> None:
    source = gradient(0.15, 0.15)
    source.putpixel((0, 0), (221, 12, 43, 0))
    source.putpixel((1, 0), (150, 120, 100, 42))
    output = recolour(source, gradient(0.25, 0.65))
    assert output.size == source.size
    assert output.getchannel("A").tobytes() == source.getchannel("A").tobytes()
    assert output.getpixel((0, 0)) == (221, 12, 43, 0)


def test_red_wrap_stays_red() -> None:
    source = gradient(0.96, 0.2)
    output = recolour(source, gradient(0.96, 0.7))
    hue, saturation, _ = distribution(output)
    assert abs(hue[1]) < 0.04
    assert saturation[1] > 0.6
    np.testing.assert_allclose(
        distribution(output), distribution(gradient(0.96, 0.7)), atol=0.012
    )


def test_flat_channel_maps_to_reference_median() -> None:
    output = recolour(
        Image.new("RGBA", (5, 5), (100, 100, 100, 255)), gradient(0.25, 0.65)
    )
    np.testing.assert_allclose(
        np.asarray(distribution(output))[:, 1],
        np.asarray(distribution(gradient(0.25, 0.65)))[:, 1],
        atol=0.012,
    )


@pytest.mark.parametrize("empty_source", [True, False])
def test_empty_distribution_is_rejected(empty_source: bool) -> None:
    empty = Image.new("RGBA", (3, 3))
    filled = gradient(0.2, 0.4)
    with pytest.raises(ValueError, match="opaque pixels"):
        recolour(empty, filled) if empty_source else recolour(filled, empty)
