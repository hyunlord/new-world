"""Given masked leaves, preserve the untouched sprite and bounded snow layer."""
from pathlib import Path

import numpy as np
from PIL import Image

from art.flora import seasons
from art.flora.schema import SeasonTint, SnowCap


def test_masked_shift_preserves_ink_trunk_and_alpha() -> None:
    # Given a leaf, brown trunk and transparent pixel, only the leaf is masked.
    pixels = np.array([[[80, 122, 56, 255], [90, 60, 35, 255], [25, 17, 9, 0]]], dtype=np.uint8)
    mask = Image.fromarray(np.array([[255, 0, 0]], dtype=np.uint8))
    # When a spring shift is applied.
    result = np.asarray(seasons.apply_tint(Image.fromarray(pixels), mask, SeasonTint(delta_lab=(6, 0, 6))))
    # Then RGB outside foliage and every alpha byte are exact originals.
    np.testing.assert_array_equal(result[:, 1:], pixels[:, 1:])
    np.testing.assert_array_equal(result[..., 3], pixels[..., 3])
    assert not np.array_equal(result[:, :1, :3], pixels[:, :1, :3])


def test_identity_is_byte_exact() -> None:
    # Given arbitrary RGBA including partially transparent ink.
    pixels = np.arange(100, dtype=np.uint8).reshape(5, 5, 4)
    # When identity summer is applied.
    result = seasons.apply_tint(Image.fromarray(pixels), Image.new('L', (5, 5), 255), SeasonTint())
    # Then no color roundtrip changes even one byte.
    np.testing.assert_array_equal(np.asarray(result), pixels)


def test_snow_only_occupies_top_edge_at_bounded_alpha() -> None:
    # Given a canopy with an interior gap and a lower separate lobe.
    mask = Image.new('L', (9, 12), 0)
    pixels = np.array(mask)
    pixels[2:6, 2:7] = 255
    pixels[8:11, 2:7] = 255
    # When a two-pixel cap is eroded from the top.
    cap = np.asarray(seasons.snow_layer(Image.fromarray(pixels), SnowCap(depth_px=2, alpha=0.7)))
    # Then no lower gap gets snow and alpha never exceeds 70 percent.
    assert cap[2:4, 2:7, 3].min() > 0
    assert cap[4:, :, 3].max() == 0
    assert cap[..., 3].max() <= 178


def test_palette_clusters_exclude_brown_and_ink() -> None:
    # Given foliage prototypes beside brown bark and almost black ink.
    pixels = np.array([[[80, 122, 56, 255], [117, 156, 75, 255], [90, 60, 35, 255], [24, 17, 10, 255]]], dtype=np.uint8)
    # When Lab nearest-palette clustering runs.
    mask = seasons.canopy_mask(Image.fromarray(pixels), Path('.'))
    # Then both foliage colors are covered, while bark and ink are excluded.
    assert np.asarray(mask).tolist() == [[255, 255, 0, 0]]
