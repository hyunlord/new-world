"""Paired silhouette measurements distinguish identity from base alignment."""
from PIL import Image, ImageDraw

from art.flora.check_bare import compare_silhouettes


def test_identical_pair_has_exact_alpha_and_silhouette() -> None:
    # Given identical lower trunks.
    image = Image.new('RGBA', (20, 20))
    ImageDraw.Draw(image).rectangle((8, 10, 11, 18), fill=(60, 40, 20, 255))
    # When comparing the same raster.
    result = compare_silhouettes(image, image.copy())
    # Then exact identity is measured rather than inferred from anchors.
    assert result.lower_alpha128_iou == 1
    assert result.lower_alpha_bytes_exact_on_visible_union_fraction == 1
    assert result.exact_lower_silhouette


def test_equal_base_does_not_imply_same_trunk() -> None:
    # Given two trunks at the same base but different x.
    summer = Image.new('RGBA', (20, 20))
    bare = Image.new('RGBA', (20, 20))
    ImageDraw.Draw(summer).rectangle((8, 10, 11, 18), fill=(60, 40, 20, 255))
    ImageDraw.Draw(bare).rectangle((10, 10, 13, 18), fill=(60, 40, 20, 255))
    # When comparing aligned bases.
    result = compare_silhouettes(summer, bare)
    # Then the measured overlap exposes the shifted silhouette.
    assert result.base_delta_px == 0
    assert result.lower_alpha128_iou == 1/3
    assert not result.exact_lower_silhouette
