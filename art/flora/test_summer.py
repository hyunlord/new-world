"""Canonical summer normalization never changes geometry or protected bytes."""
import numpy as np
from PIL import Image

from art.flora import summer


def test_chroma_center_preserves_luminance_alpha_and_unmasked_pixels() -> None:
    # Given an olive leaf and an unmasked brown trunk pixel.
    pixels = np.array([[[120, 122, 50, 255], [110, 84, 42, 128]]], dtype=np.uint8)
    mask = Image.fromarray(np.array([[255, 0]], dtype=np.uint8))
    # When the leaf mean chroma is centered on the green target.
    result, delta = summer.center_chroma(Image.fromarray(pixels), mask, (80, 122, 56))
    # Then only masked RGB changes and the transform never requests a lightness shift.
    output = np.asarray(result)
    assert delta[0] == 0
    np.testing.assert_array_equal(output[..., 3], pixels[..., 3])
    np.testing.assert_array_equal(output[:, 1:], pixels[:, 1:])
    assert output[0, 0, 1] > output[0, 0, 0]


def test_empty_mask_does_not_change_pixels() -> None:
    # Given a bare silhouette without foliage.
    source = Image.new('RGBA', (3, 3), (90, 60, 30, 255))
    # When no pixels are selected.
    result, delta = summer.center_chroma(source, Image.new('L', (3, 3)), (80, 122, 56))
    # Then the transform is identity.
    assert result.tobytes() == source.tobytes()
    assert delta == (0, 0, 0)
