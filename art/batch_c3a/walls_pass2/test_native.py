"""Native-course topology must preserve phase, geometry and alpha borders."""

from pathlib import Path

import numpy as np
import pytest
from PIL import Image

from art.batch_c3a.walls_pass2 import native


def test_native_piece_preserves_source_pixels_and_thin_geometry() -> None:
    # Given a unique opaque64px stone field, with no resampling opportunity.
    pixels = np.zeros((64, 64, 4), dtype=np.uint8)
    pixels[..., 0] = np.arange(64, dtype=np.uint8)
    pixels[..., 1] = np.arange(64, dtype=np.uint8)[:, None]
    pixels[..., 3] = 255
    source = Image.fromarray(pixels)
    # When a straight east-west piece is composed.
    result = native.compose_piece(10, source)
    # Then the top and front preserve1:1 source coordinates, at W2 geometry.
    assert result.size == (64, 64)
    assert result.getpixel((30, 8)) == source.getpixel((30, 8))
    assert result.getpixel((30, 22)) == source.getpixel((30, 6))
    assert result.getchannel("A").tobytes()[36 * 64 + 30] == 204
    assert result.getchannel("A").tobytes()[41 * 64 + 30] == 0


def test_ns_piece_is_26px_wide_and_all_masks_nonempty() -> None:
    # Given an opaque native texture.
    source = Image.new("RGBA", (64, 64), (150, 130, 100, 255))
    # When every neighbour mask is composed.
    pieces = [native.compose_piece(mask, source) for mask in range(16)]
    # Then the north-south strip matches W2's26px thickness and64px run.
    assert pieces[5].getchannel("A").getbbox() == (19, 0, 45, 64)
    assert all(p.getchannel("A").getbbox() is not None for p in pieces)
    assert len({p.tobytes() for p in pieces}) >= 11


def test_source_crop_rejects_resampling_and_records_native_origin() -> None:
    # Given the real sharp stone face from the baseline art kit.
    source = Path("art/materials/textures/drystone/face.png")
    # When the deterministic crop selector builds a repeating field.
    field, selection = native.course_field(source)
    # Then every source pixel is copied, with no scale/filter operation.
    assert field.size == (64, 64)
    assert selection.width == 64 and selection.height == 32
    with Image.open(source) as original:
        crop = original.convert("RGBA").crop(
            (selection.x, selection.y, selection.x + 64, selection.y + 32)
        )
    assert field.crop((0, 0, 64, 32)).tobytes() == crop.tobytes()
    assert field.crop((0, 32, 64, 64)).tobytes() == crop.tobytes()


@pytest.mark.parametrize("mask", [3, 6, 9, 12, 7, 11, 13, 14, 15])
def test_corner_and_junction_horizontal_faces_have_no_transparent_notches(
    mask: int,
) -> None:
    # Given every corner/T/X, connected side quadrants must not leave hollow joins.
    source = Image.new("RGBA", (64, 64), (150, 130, 100, 255))
    result = native.compose_piece(mask, source)
    # Then S-bearing junctions stay solid to the next tile, not just to row38.
    spans: list[tuple[int, int]] = []
    if mask & 8:
        spans.append((0, 19))
    if mask & 2:
        spans.append((45, 64))
    alpha = result.getchannel("A")
    for left, right in spans:
        bottom = 64 if mask & 4 else 38
        assert min(alpha.crop((left, 26, right, bottom)).tobytes()) >= 96, mask


def test_south_junction_fill_copies_native_face_pixels_without_filling_floor() -> None:
    # Given identifiable pixels, repair adds faces rather than a full-tile patch.
    pixels = np.zeros((64, 64, 4), dtype=np.uint8)
    pixels[..., 0] = np.arange(64, dtype=np.uint8)
    pixels[..., 1] = np.arange(64, dtype=np.uint8)[:, None]
    pixels[..., 3] = 255
    source = Image.fromarray(pixels)
    # When an X junction has both horizontal arms and a continuing south shaft.
    result = native.compose_piece(15, source)
    # Then face pixels retain1:1 sampling and the vertical shaft keeps its phase.
    assert result.getpixel((8, 30)) == source.getpixel((8, 30))
    assert result.getpixel((55, 30)) == source.getpixel((55, 30))
    assert result.getpixel((30, 30)) == source.getpixel((30, 30))
    # Only quadrants outside the connected corner footprint remain floor.
    southeast = native.compose_piece(6, source).getchannel("A").tobytes()
    southwest = native.compose_piece(12, source).getchannel("A").tobytes()
    assert southeast[45 * 64 + 8] == 0
    assert southwest[45 * 64 + 55] == 0


def test_dense_l_wall_and_tx_blocks_have_no_hollow_lower_quadrants() -> None:
    # Given dense L/T/X wall joins, these formerly empty regions are wall, not floor.
    source = Image.new("RGBA", (64, 64), (150, 130, 100, 255))
    for mask, bounds in ((14, (0, 0, 64, 64)), (15, (0, 0, 64, 64))):
        alpha = native.compose_piece(mask, source).getchannel("A")
        assert min(alpha.crop(bounds).tobytes()) >= 96, mask
