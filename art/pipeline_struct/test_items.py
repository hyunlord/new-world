"""Geometry and count semantics of deterministic generated-item exports."""
import numpy as np
import pytest
from PIL import Image, ImageDraw

from art.pipeline_struct.items import count_tile, tile


def test_item_fit_and_counts() -> None:
    source = Image.new("RGBA", (300, 120))
    ImageDraw.Draw(source).rectangle((30, 30, 270, 90), fill=(120, 90, 60, 255))
    result = tile(source, 24)
    assert result.size == (64, 64)
    ys, xs = np.nonzero(np.asarray(result)[:, :, 3] >= 128)
    assert int(ys.max()) == 43
    assert int(xs.max() - xs.min() + 1) <= 54
    variants = [count_tile(result, count) for count in (1, 3, 6)]
    assert len({image.tobytes() for image in variants}) == 3
    for image in variants:
        alpha = np.asarray(image)[:, :, 3]
        assert int(np.nonzero(alpha >= 128)[0].max()) == 43
        assert not alpha[0].any() and not alpha[-1].any()
        assert not alpha[:, 0].any() and not alpha[:, -1].any()


def test_empty_and_unsupported_count() -> None:
    with pytest.raises(ValueError, match="opaque"):
        tile(Image.new("RGBA", (64, 64)), 12)
    with pytest.raises(ValueError, match="supports"):
        count_tile(Image.new("RGBA", (64, 64)), 2)
