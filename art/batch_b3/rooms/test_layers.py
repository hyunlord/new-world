# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "pytest"]
# ///
# How to run: uv run --with pillow --with numpy --with pydantic --with pytest python -m pytest art/batch_b3/rooms/test_layers.py
"""Low-course geometry must keep the original material pixels and world base."""

from PIL import Image

from art.batch_b3.rooms.layers import course_image, course_origin


def test_foundation_preserves_cap_and_first_eight_face_rows() -> None:
    # Given a uniquely coloured row at every height in an existing wall tile.
    source = Image.new("RGBA", (64, 60))
    source.putdata([(y, x, 70, 255) for y in range(60) for x in range(64)])
    # When its exposed course is 24px instead of 60px.
    result = course_image(source, 24)
    # Then pixels are an exact crop, never a squeezed facade.
    assert result.size == (64, 24)
    assert result.tobytes() == source.crop((0, 0, 64, 24)).tobytes()


def test_north_south_cap_retains_full_tile_depth() -> None:
    # Given a full-depth north/south top tile with no visible south face.
    source = Image.new("RGBA", (64, 64), (40, 50, 60, 255))
    # When construction lowers the course's elevation.
    result = course_image(source, 24)
    # Then full64px plan depth remains, avoiding48px gaps along the run.
    assert result.size == (64, 64)
    assert result.tobytes() == source.tobytes()


def test_course_origin_keeps_south_face_at_world_base() -> None:
    # Given the cell in the third column and second row.
    # When a24px course is placed with the tile64 and world base43 contract.
    origin = course_origin((2, 1), 24)
    # Then its last facade row stays at y=107 and x remains128.
    assert origin == (128, 84)
    assert origin[1] + 24 - 1 == 107
