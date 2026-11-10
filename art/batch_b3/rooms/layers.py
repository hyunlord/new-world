# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: imported by art.batch_b3.rooms.render.
"""Stage-course geometry over the existing material wall compositor."""

from PIL import Image

from art.compositor.walls import ANCHOR, CAP, FACE, TILE


def course_image(source: Image.Image, height: int) -> Image.Image:
    """Retain plan depth and crop exposed face rows without repainting."""
    return (
        source.copy()
        if source.height > CAP + FACE
        else source.crop((0, 0, source.width, height))
    )


def course_origin(cell: tuple[int, int], height: int) -> tuple[int, int]:
    """Place a low course against the unchanged world base."""
    return cell[0] * TILE, cell[1] * TILE + ANCHOR - height + 1
