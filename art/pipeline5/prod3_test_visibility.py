# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "pytest"]
# ///
# How to run: python -m pytest art/pipeline5/prod3_test_visibility.py
"""PROD3 alpha16 geometry preserves in-canvas pixels and logs discarded halos."""
from dataclasses import asdict

import numpy as np
import pytest
from PIL import Image, ImageDraw

from art.pipeline5.sheet_geometry import normalize_panel
from art.pipeline5.whole_checks import check_whole


@pytest.mark.parametrize('sleep', [False, True])
@pytest.mark.parametrize('halo_alpha', [3, 12, 15])
def test_six_pixel_halo_can_leave_canvas_without_changing_retained_pixels(
    sleep: bool, halo_alpha: int,
) -> None:
    width, height = (108, 24) if sleep else (76, 56)
    source = Image.new('RGBA', (width + 12, height + 12), (10, 20, 30, halo_alpha))
    ImageDraw.Draw(source).rectangle((6, 6, width + 5, height + 5), fill=(80, 90, 100, 255))
    result = normalize_panel(source, 1, sleep)
    dx, dy = result.transform.resized_to_canvas_offset
    discarded = 0
    for sy in range(source.height):
        for sx in range(source.width):
            tx, ty = sx + dx, sy + dy
            if 0 <= tx < result.image.width and 0 <= ty < result.image.height:
                assert source.getpixel((sx, sy)) == result.image.getpixel((tx, ty))
            else:
                discarded += 1
    record = asdict(result.transform)
    assert record['discarded_pixels'] == discarded > 0
    assert record['discarded_max_alpha'] == halo_alpha
    visible_y = np.nonzero(np.asarray(result.image)[:, :, 3] >= 16)[0]
    assert visible_y.max() == (72 if sleep else 104)
    assert not any('alpha_edge' in finding for finding in check_whole(result.image, 'adult', 'carry' if not sleep else 'sleep').findings)


@pytest.mark.parametrize('sleep', [False, True])
@pytest.mark.parametrize('alpha', [16, 128, 255])
def test_visible_body_that_overflows_is_rejected(sleep: bool, alpha: int) -> None:
    source = Image.new('RGBA', (120, 20) if sleep else (90, 56), (10, 20, 30, alpha))
    with pytest.raises(ValueError, match='visible extent'):
        _ = normalize_panel(source, 1, sleep)


def test_bands_and_contact_use_alpha16_not_opaque_body() -> None:
    source = Image.new('RGBA', (80, 112))
    ImageDraw.Draw(source).rectangle((28, 49, 51, 104), fill=(80, 90, 100, 255))
    ImageDraw.Draw(source).rectangle((28, 105, 51, 108), fill=(80, 90, 100, 16))
    checks = check_whole(source, 'adult', 'idle')
    assert not checks.passed
    assert not checks.contact_ok
    assert any('height 60' in finding for finding in checks.findings)


def test_no_discard_is_explicit_and_alpha15_inside_is_preserved() -> None:
    source = Image.new('RGBA', (30, 56), (80, 90, 100, 255))
    source.putpixel((0, 0), (10, 20, 30, 15))
    result = normalize_panel(source, 1, False)
    assert result.transform.discarded_pixels == 0
    assert result.transform.discarded_max_alpha == 0
    dx, dy = result.transform.resized_to_canvas_offset
    assert result.image.getpixel((dx, dy)) == source.getpixel((0, 0))


@pytest.mark.parametrize(('sleep', 'width', 'height'), [(False, 24, 56), (True, 56, 24)])
def test_halo_and_faint_tail_pass_complete_structure_check(sleep: bool, width: int, height: int) -> None:
    source = Image.new('RGBA', (width + 12, height + 20))
    draw = ImageDraw.Draw(source)
    draw.rectangle((0, 0, width + 11, height + 11), fill=(10, 20, 30, 12))
    draw.rectangle((6, 6, width + 5, height + 5), fill=(80, 90, 100, 255))
    draw.line((width // 2, height + 12, width // 2, height + 19), fill=(10, 20, 30, 3))
    result = normalize_panel(source, 1, sleep)
    checked = check_whole(result.image, 'adult', 'sleep' if sleep else 'idle')
    assert checked.passed, checked.findings
    assert result.transform.discarded_pixels == 7
    assert result.transform.discarded_max_alpha == 3
