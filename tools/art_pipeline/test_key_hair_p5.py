# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/test_key_hair_p5.py
"""Boundary regressions for exact key protection, anatomy and fidelity ordering."""
from __future__ import annotations

import numpy as np
from PIL import Image

from key_hair_p5 import KEYS, convert_key, outlined_mask, protected_despeckle
from pixel_normalize import outline

BROWNS = ((101, 75, 48), (69, 42, 31), (35, 22, 17))


def fixture() -> Image.Image:
    """An opaque standing body with six connected keyed hair texels."""
    pixels = np.zeros((20, 10, 4), dtype=np.uint8)
    pixels[:, 2:8] = (153, 98, 55, 255)
    pixels[0:2, 3:6, :3] = KEYS[0]
    return Image.fromarray(pixels)


def test_protect_keys() -> None:
    source = np.array(fixture())
    source[0, 3, :3] = KEYS[2]
    cleaned = np.asarray(protected_despeckle(Image.fromarray(source)).image)
    assert np.array_equal(source[:2, 3:6], cleaned[:2, 3:6])


def test_pre_substitution_error_and_mask() -> None:
    source = fixture()
    result = convert_key(source, source, BROWNS)
    assert result.delta_before == 0
    assert result.delta_after > 0
    assert result.key_count == 6
    assert not result.errors
    assert (np.asarray(result.mask)[..., 0] == 3).sum() == 6
    assert np.array_equal(np.asarray(result.natural)[0, 3, :3], BROWNS[0])
    sprite = outline(result.natural, (3, 0, 0))
    mask = np.asarray(outlined_mask(result.mask, sprite))
    assert (mask[..., 0] == 3).sum() == 6
    assert np.array_equal(mask[..., 3], np.asarray(sprite)[..., 3])


def test_three_exact_error_boundaries() -> None:
    source = np.array(fixture())
    source[0, 3:6, :3] = (153, 98, 55)
    result = convert_key(Image.fromarray(source), Image.fromarray(source), BROWNS)
    assert 'key_hair_missing' in result.errors
    source = np.array(fixture())
    source[8:11, 7, :3] = KEYS[1]
    result = convert_key(Image.fromarray(source), Image.fromarray(source), BROWNS)
    assert 'key_hair_outside_head' in result.errors
    source = np.array(fixture())
    source[19, 3, :3] = KEYS[1]
    result = convert_key(Image.fromarray(source), Image.fromarray(source), BROWNS)
    assert 'key_hair_outside_head' in result.errors
    source = np.array(fixture())
    source[2, 3:6, :3] = (32, 160, 130)
    result = convert_key(Image.fromarray(source), Image.fromarray(source), BROWNS)
    assert result.fringe_count == 3
    assert 'key_fringe' in result.errors


if __name__ == '__main__':
    test_protect_keys()
    test_pre_substitution_error_and_mask()
    test_three_exact_error_boundaries()
    print('3 key-hair regression scenarios passed')
