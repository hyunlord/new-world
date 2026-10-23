# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "numpy", "pillow", "pydantic"]
# ///
# How to run: python3 -m pytest art/materials/test_material_tone.py -q.
"""Colour-transform roundtrip, alpha and deterministic behavior."""
import numpy as np
from PIL import Image

from art.materials.schema import LabTint, Top
from art.materials.tone import lab_to_rgb, rgb_to_lab, shift_image, top_tone


def test_lab_roundtrip_and_reference_white() -> None:
    # Given the D65 primaries, neutrals and a warm colour.
    values = np.array([[0, 0, 0], [1, 1, 1], [1, 0, 0], [.4, .2, .1]])
    # When converting to Lab and back.
    lab = rgb_to_lab(values)
    result = lab_to_rgb(lab)
    # Then the reference white and full roundtrip agree numerically.
    assert np.allclose(lab[1:2, :], [[100, 0, 0]], atol=.0001)
    assert np.allclose(result, values, atol=.000001)


def test_shift_preserves_alpha_and_zero_is_byte_identical() -> None:
    # Given colour and alpha gradients.
    source = Image.fromarray(np.arange(64, dtype=np.uint8).reshape((4, 4, 4)))
    # When shifting nonzero Lab values and also requesting zero shift.
    shifted = shift_image(source, LabTint(dL=5, da=2, db=-3))
    zero = shift_image(source, LabTint())
    # Then source bytes are unchanged, and alpha never changes.
    assert shifted.getchannel("A").tobytes() == source.getchannel("A").tobytes()
    assert zero.tobytes() == source.tobytes()
    assert shifted.tobytes() == shift_image(source, LabTint(dL=5, da=2, db=-3)).tobytes()


def test_top_parameters_have_measurable_value_effect() -> None:
    # Given a uniform achromatic top.
    source = Image.new("RGBA", (256, 64), (200, 200, 200, 255))
    # When requesting material tops at 0.75 value.
    result = top_tone(source, Top())
    # Then intensity matches the declared multiplier exactly.
    assert result.getpixel((0, 0)) == (150, 150, 150, 255)
