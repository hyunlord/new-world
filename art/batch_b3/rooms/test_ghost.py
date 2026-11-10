# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pytest"]
# ///
# How to run: uv run --with pillow --with pytest python -m pytest art/batch_b3/rooms/test_ghost.py
"""Mirror the published code-companion ghost, including four-neighbour edges."""

from PIL import Image

from art.batch_b3.rooms.ghost import ghost


def test_ghost_preserves_rgb_and_anchor_canvas() -> None:
    # Given a solid centre pixel and transparent padding.
    source = Image.new("RGBA", (3, 3))
    source.putpixel((1, 1), (90, 60, 30, 255))
    # When the existing site's25-percent ghost recipe is previewed.
    result = ghost(source)
    # Then RGB/canvas persist and only alpha changes on the original silhouette.
    assert result.size == source.size
    assert result.getpixel((1, 1)) == (90, 60, 30, 64)
    assert source.getpixel((1, 1)) == (90, 60, 30, 255)


def test_ghost_uses_four_neighbours_not_diagonals() -> None:
    # Given a lone solid centre pixel.
    source = Image.new("RGBA", (3, 3))
    source.putpixel((1, 1), (90, 60, 30, 255))
    # When the code-companion edge is generated.
    result = ghost(source)
    # Then cardinal neighbours have pale ink and corners stay transparent.
    assert result.getpixel((1, 0)) == (218, 222, 205, 230)
    assert result.getpixel((0, 0)) == (0, 0, 0, 0)
