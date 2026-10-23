# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "scipy", "scikit-image", "pydantic"]
# ///
# How to run: python -m pytest art/pipeline_polish/animal_tests.py
"""Verify native sprite placement and lossless display scaling for animal boards."""
from pathlib import Path

from PIL import Image

from art.pipeline_polish.animal_boards import Frame, place, tile_background


def test_standing_foot_anchor_lands_at_declared_baseline(tmp_path: Path) -> None:
    # Given an upright A1 sprite with one opaque foot at its y72 anchor.
    path = tmp_path / "stand.png"
    sprite = Image.new("RGBA", (96, 80))
    sprite.putpixel((48, 72), (240, 12, 20, 255))
    sprite.save(path)
    canvas = Image.new("RGBA", (200, 180))
    # When displaying at native scale with a specified ground baseline.
    place(canvas, Frame("test", "S", "stand", path), 100, 150, 1)
    # Then the foot, without rescaling or drift, is exactly at that baseline.
    assert canvas.getbbox() == (100, 150, 101, 151)
    assert canvas.getpixel((100, 150)) == (240, 12, 20, 255)


def test_dead_uses_its_shorter_canvas_anchor_at_two_times(tmp_path: Path) -> None:
    # Given a dead A1 sprite whose anchor is y56 instead of y72.
    path = tmp_path / "dead.png"
    sprite = Image.new("RGBA", (96, 64))
    sprite.putpixel((48, 56), (9, 217, 19, 255))
    sprite.save(path)
    canvas = Image.new("RGBA", (220, 180))
    # When displaying at exactly2x nearest-neighbor.
    place(canvas, Frame("test", "E", "dead", path), 110, 150, 2)
    # Then one native pixel becomes an exact2x2 block at the correct anchor.
    assert canvas.getbbox() == (110, 150, 112, 152)
    assert all(canvas.getpixel((x, y)) == (9, 217, 19, 255)
               for y in (150, 151) for x in (110, 111))


def test_background_tiles_cover_nonmultiple_packet_extent() -> None:
    # Given a calm-ground tile and packet dimensions not divisible by64.
    tile = Image.new("RGBA", (64, 64), (73, 77, 65, 255))
    # When tiling at2x to the complete packet extent.
    result = tile_background((1060, 752), tile, 2)
    # Then no unpainted edge is left behind the last row or column.
    assert result.size == (1060, 752)
    assert result.getextrema() == ((73, 73), (77, 77), (65, 65), (255, 255))
