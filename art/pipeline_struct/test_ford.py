# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "pydantic", "scikit-image"]
# ///
# How to run: uv run --with pytest --with pillow --with numpy --with pydantic --with scikit-image python -m pytest art/pipeline_struct/test_ford.py
"""Four-cell extraction preserves ordering and refuses missing sheet content."""
import pytest
from PIL import Image, ImageDraw

from art.pipeline_struct.ford import split_stones


def test_four_stones_preserve_cells_and_uniform_height() -> None:
    # Given four uniquely coloured stones in equal source cells.
    source = Image.new("RGBA", (400, 100))
    colours = ((160, 30, 40, 255), (20, 170, 40, 255), (20, 30, 180, 255), (150, 140, 30, 255))
    for index, colour in enumerate(colours):
        ImageDraw.Draw(source).rectangle((index * 100 + 20, 30, index * 100 + 80, 69), fill=colour)
    # When split and normalized without repainting pixels.
    stones = split_stones(source)
    # Then order, twelve-pixel opaque height and declared base anchors survive.
    assert len(stones) == 4
    for stone, colour in zip(stones, colours, strict=True):
        alpha = stone.image.getchannel("A").point([255 if value >= 128 else 0 for value in range(256)])
        bbox = alpha.getbbox()
        assert bbox is not None
        assert 11 <= bbox[3] - bbox[1] <= 13
        assert stone.anchor[1] == bbox[3] - 1
        assert stone.image.getpixel((stone.image.width // 2, stone.image.height // 2)) == colour


def test_missing_stone_fails_instead_of_reusing_another_cell() -> None:
    # Given an empty sheet rather than four generated subjects.
    source = Image.new("RGBA", (400, 100))
    # When extraction is attempted, then no substitute stones are emitted.
    with pytest.raises(ValueError, match="opaque"):
        _ = split_stones(source)
