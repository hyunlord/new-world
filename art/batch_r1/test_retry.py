# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest"]
# ///
# How to run: python3 -m pytest art/batch_r1/test_retry.py
"""Raw corrections preserve geometry and apply one colour rule everywhere."""

from pathlib import Path

import numpy as np
import pytest
from numpy.typing import NDArray
from PIL import Image

from art.batch_r1.retry import transform_raw


def test_uniform_transform_preserves_raw_geometry(tmp_path: Path) -> None:
    source = tmp_path / "source.png"
    original: NDArray[np.uint8] = np.zeros((1024, 1536, 4), dtype=np.uint8)
    colours = [
        (200, 100, 50, 255),
        (200, 100, 50, 80),
        (20, 10, 5, 255),
        (250, 248, 247, 255),
        (200, 100, 50, 0),
    ]
    for index, colour in enumerate(colours):
        original[index, index] = colour
        original[100 + index, 800 + index] = colour
    Image.fromarray(original).save(source)
    source_bytes = source.read_bytes()
    first, second = tmp_path / "first.png", tmp_path / "second.png"
    for destination in (first, second):
        transform_raw(
            source, destination, saturation_gain=0.5, value_gain=0.8, ink_gain=0.5
        )
    assert first.read_bytes() == second.read_bytes()
    assert source.read_bytes() == source_bytes
    with Image.open(first) as image:
        result: NDArray[np.uint8] = np.array(image)
        assert image.size == (1536, 1024)
        assert image.mode == "RGBA"
    assert np.array_equal(result[:, :, 3], original[:, :, 3])
    assert np.array_equal(result[0:1, 0:1], [[[160, 120, 100, 255]]])
    assert np.array_equal(result[1:2, 1:2], [[[160, 120, 100, 80]]])
    assert result[2, 2, 0] < 16  # Dark ink scales more than ordinary V gain.
    for index in range(len(colours)):
        assert np.array_equal(
            result[index : index + 1, index : index + 1],
            result[100 + index : 101 + index, 800 + index : 801 + index],
        )
    assert np.array_equal(result[3:4, 3:4], original[3:4, 3:4])
    assert np.array_equal(result[4:5, 4:5], original[4:5, 4:5])
    with pytest.raises(FileExistsError):
        transform_raw(source, first)
    with pytest.raises(ValueError, match="separate destination"):
        transform_raw(source, source)


def test_neutral_gains_preserve_every_pixel(tmp_path: Path) -> None:
    source, destination = tmp_path / "source.png", tmp_path / "neutral.png"
    original: NDArray[np.uint8] = np.random.default_rng(42).integers(
        0, 256, (1024, 1536, 4), dtype=np.uint8
    )
    Image.fromarray(original).save(source)
    transform_raw(source, destination)
    with Image.open(destination) as image:
        assert np.array_equal(np.array(image), original)


@pytest.mark.parametrize("gain", [0, -1, float("nan"), float("inf")])
@pytest.mark.parametrize("parameter", ["saturation_gain", "value_gain", "ink_gain"])
def test_invalid_gain_rejected(tmp_path: Path, gain: float, parameter: str) -> None:
    with pytest.raises(ValueError, match="finite and positive"):
        transform_raw(
            tmp_path / "absent.png", tmp_path / "output.png", **{parameter: gain}
        )
    assert not (tmp_path / "output.png").exists()


@pytest.mark.parametrize("mode,size", [("RGB", (1536, 1024)), ("RGBA", (10, 10))])
def test_invalid_sheet_rejected(
    tmp_path: Path, mode: str, size: tuple[int, int]
) -> None:
    source = tmp_path / "source.png"
    Image.new(mode, size).save(source)
    with pytest.raises(ValueError, match="1536x1024 RGBA"):
        transform_raw(source, tmp_path / "output.png")


def test_optional_ink_cutoff_retains_default_and_reaches_midtones(
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.png"
    Image.new("RGBA", (1536, 1024), (128, 64, 32, 255)).save(source)
    default, explicit, broad = (
        tmp_path / name for name in ("default.png", "explicit.png", "broad.png")
    )
    transform_raw(source, default, ink_gain=2.0)
    transform_raw(source, explicit, ink_gain=2.0, ink_cutoff=0.25)
    transform_raw(source, broad, ink_gain=2.0, ink_cutoff=0.75)
    assert default.read_bytes() == explicit.read_bytes()
    with Image.open(default) as first, Image.open(broad) as second:
        assert first.getpixel((0, 0)) == (128, 64, 32, 255)
        assert second.getpixel((0, 0)) == (142, 71, 36, 255)
        assert second.getchannel("A").tobytes() == first.getchannel("A").tobytes()


@pytest.mark.parametrize("cutoff", [0, -1, 1.01, float("nan"), float("inf")])
def test_invalid_ink_cutoff_rejected(tmp_path: Path, cutoff: float) -> None:
    with pytest.raises(ValueError, match="ink cutoff"):
        transform_raw(
            tmp_path / "absent.png", tmp_path / "output.png", ink_cutoff=cutoff
        )
    assert not (tmp_path / "output.png").exists()


def test_value_offset_default_and_black_lift(tmp_path: Path) -> None:
    source = tmp_path / "source.png"
    Image.new("RGBA", (1536, 1024), (0, 0, 0, 255)).save(source)
    default, explicit, lifted = (
        tmp_path / name for name in ("default.png", "explicit.png", "lifted.png")
    )
    transform_raw(source, default, saturation_gain=0.8, value_gain=1.1)
    transform_raw(source, explicit, saturation_gain=0.8, value_gain=1.1, value_offset=0)
    transform_raw(source, lifted, value_offset=0.2)
    assert default.read_bytes() == explicit.read_bytes()
    with Image.open(lifted) as image:
        assert image.getpixel((0, 0)) == (51, 51, 51, 255)


def test_negative_value_offset_clamps_to_black(tmp_path: Path) -> None:
    source, destination = tmp_path / "source.png", tmp_path / "dark.png"
    Image.new("RGBA", (1536, 1024), (100, 50, 25, 100)).save(source)
    transform_raw(source, destination, value_offset=-0.5)
    with Image.open(destination) as image:
        assert image.getpixel((0, 0)) == (0, 0, 0, 100)


@pytest.mark.parametrize("offset", [-1.01, 1.01, float("nan"), float("inf")])
def test_invalid_value_offset_rejected(tmp_path: Path, offset: float) -> None:
    with pytest.raises(ValueError, match="value offset"):
        transform_raw(
            tmp_path / "absent.png", tmp_path / "output.png", value_offset=offset
        )
    assert not (tmp_path / "output.png").exists()
