# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow"]
# ///
# How to run: imported for review previews; runtime uses site_ghost.gd.
"""Read-only preview of the existing runtime blueprint display recipe."""

from typing import Final

from PIL import Image

SOLID: Final = 96


def ghost(source: Image.Image) -> Image.Image:
    """Apply the published25-percent wash and four-neighbour pale outline."""
    rgba = source.convert("RGBA")
    width, height = rgba.size
    data = rgba.tobytes()
    output = bytearray(data)
    for y in range(height):
        for x in range(width):
            i = (y * width + x) * 4
            alpha = data[i + 3]
            output[i + 3] = (alpha + 2) // 4
            if alpha >= SOLID:
                continue
            edge = (
                (x > 0 and data[i - 1] >= SOLID)
                or (x + 1 < width and data[i + 7] >= SOLID)
                or (y > 0 and data[i - width * 4 + 3] >= SOLID)
                or (y + 1 < height and data[i + width * 4 + 3] >= SOLID)
            )
            if edge:
                output[i : i + 4] = bytes((218, 222, 205, 230))
    return Image.frombytes("RGBA", rgba.size, bytes(output))
