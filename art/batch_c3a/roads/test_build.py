# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest", "pydantic"]
# ///
# How to run: uv run --with pytest --with pillow --with numpy --with pydantic python -m pytest art/batch_c3a/roads/test_build.py -q.
"""Road edge direction and lossless interior contracts."""

from pathlib import Path

import numpy as np
from PIL import Image


def test_edges_preserve_interior_and_follow_missing_north_connection() -> None:
    # Given contrasting fully opaque pavement and grass.
    from art.batch_c3a.roads.build import transition

    pavement = Image.new("RGBA", (64, 64), (200, 180, 140, 255))
    grass = Image.new("RGBA", (64, 64), (20, 80, 30, 255))
    # When north is the only missing connection.
    result = transition(14, pavement, grass)
    # Then only the north12px receives the existing terrain transition.
    assert (
        result.crop((0, 12, 64, 64)).tobytes()
        == pavement.crop((0, 12, 64, 64)).tobytes()
    )
    assert (
        result.crop((0, 0, 64, 12)).tobytes() != pavement.crop((0, 0, 64, 12)).tobytes()
    )
    assert result.getchannel("A").getextrema() == (255, 255)


def test_complete_junction_keeps_every_source_pixel() -> None:
    # Given an arbitrary generated material tile.
    from art.batch_c3a.roads.build import transition

    pixels = np.arange(64 * 64 * 4, dtype=np.uint8).reshape(64, 64, 4)
    pavement = Image.fromarray(pixels)
    # When all four neighbours have the same pavement.
    result = transition(15, pavement, Image.new("RGBA", (64, 64), "green"))
    # Then the junction uses the complete source unchanged.
    assert result.tobytes() == pavement.tobytes()


def test_export_has_all_opaque_directional_tiles(tmp_path: Path) -> None:
    # Given a new destination.
    from art.batch_c3a.roads.build import export

    # When the production exporter runs.
    export(tmp_path / "output")
    # Then both sides of the transition have16 valid tiles.
    for family in ("paved_road", "meadow_edge"):
        for mask in range(16):
            with Image.open(tmp_path / "output" / f"{family}_{mask:02}.png") as tile:
                assert tile.size == (64, 64)
                assert tile.getchannel("A").getextrema() == (255, 255)
