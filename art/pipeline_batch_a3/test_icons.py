# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image", "pytest"]
# ///
# How to run: PYTHONPATH=. python -m pytest art/pipeline_batch_a3/test_icons.py
"""Regression checks for the adopted centered icon convention and count bounds."""

from __future__ import annotations

from typing import TYPE_CHECKING

from PIL import Image

from art.pipeline_batch_a3.icons import icon_tiles, inspect_icon
from art.pipeline_env1.items_prepare import normalize

if TYPE_CHECKING:
    from pathlib import Path


def test_env1_icon_is_centered_and_fits_58_pixels(tmp_path: Path) -> None:
    # Given a wide transparent source that requires proportional reduction.
    image = Image.new("RGBA", (240, 140))
    image.paste((140, 120, 100, 255), (20, 20, 220, 120))
    path = tmp_path / "raw.png"
    image.save(path)
    # When the unchanged ENV1 normalizer creates the item icon.
    result = normalize(path)
    # Then the full icon is centered, padded, and checked as an icon.
    assert result.size == (64, 64)
    assert result.getbbox() == (3, 17, 61, 46)
    check = inspect_icon(result, "1", result)
    assert check.structural_pass
    assert check.projection.height_band_override
    assert check.projection.height_band == (4, 58)


def test_shifted_icon_fails_center_check() -> None:
    # Given a legal-size icon shifted six pixels left.
    image = Image.new("RGBA", (64, 64))
    image.paste((140, 120, 100, 255), (6, 16, 46, 48))
    expected = Image.new("RGBA", (64, 64))
    expected.paste((140, 120, 100, 255), (12, 16, 52, 48))
    # When its single-item geometry is inspected without repair.
    result = inspect_icon(image, "1", expected)
    # Then the misplaced icon is rejected independently of its lighting.
    assert not result.structural_pass


def test_faint_fringe_does_not_require_recentering(tmp_path: Path) -> None:
    # Given distant alpha=1 fringe that contributes to the ENV1 source crop.
    image = Image.new("RGBA", (240, 140))
    image.putpixel((1, 20), (100, 80, 60, 1))
    image.paste((140, 120, 100, 255), (120, 20, 220, 120))
    path = tmp_path / "fringe.png"
    image.save(path)
    # When unchanged ENV1 scaling makes that faint pixel disappear.
    expected = normalize(path)
    check = inspect_icon(expected, "1", expected)
    # Then placement passes although surviving alpha is visibly off-center.
    assert check.center_offset_px is not None
    assert max(map(abs, check.center_offset_px)) > 1
    assert check.structural_pass
    assert Image.open(path).getpixel((1, 20)) == (100, 80, 60, 1)


def test_env1_count_layouts_keep_every_copy_inside_canvas() -> None:
    # Given a maximally filled, correctly padded ENV1 item icon.
    icon = Image.new("RGBA", (64, 64))
    icon.paste((140, 120, 100, 255), (3, 3, 61, 61))
    # When all three original ENV1 count layouts are rendered.
    exports = icon_tiles(icon, counts=True)
    # Then all buckets retain transparency at the canvas boundary.
    assert tuple(label for label, _ in exports) == ("1", "2_8", "9_plus")
    assert all(
        inspect_icon(sprite, label, sprite).structural_pass for label, sprite in exports
    )
    assert exports[0][1].tobytes() == icon.tobytes()
