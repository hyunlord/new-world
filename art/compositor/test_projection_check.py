# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pytest"]
# ///
# How to run: uv run --with pytest --with pillow --with numpy python -m pytest art/compositor/test_projection_check.py
"""Reject incorrect bases, sizes and lighting without altering the artwork."""
from __future__ import annotations

import importlib
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

import numpy as np
import pytest
from PIL import Image


def sprite(*, reverse: bool = False, height: int = 34) -> Image.Image:
    """An opaque body with a controlled diagonal luminance gradient."""
    pixels = np.zeros((64, 64, 4), dtype=np.uint8)
    for y in range(44 - height, 44):
        for x in range(20, 44):
            value = 40 + x + y if reverse else 230 - x - y
            pixels[y, x] = (value, value, value, 255)
    return Image.fromarray(pixels)


def test_checker_exists() -> None:
    # Given the standalone projection instrument contract.
    # When resolving its module.
    found = importlib.util.find_spec("art.compositor.projection_check")
    # Then the instrument is available independently of the scene renderer.
    assert found is not None


def test_upper_left_gradient_and_declared_base_pass() -> None:
    # Given a 34px workbench with its last opaque row at 43.
    checker = importlib.import_module("art.compositor.projection_check")
    source = sprite()
    before = source.tobytes()
    # When checked against the declared base.
    result = checker.check_projection(source, checker.ProjectionSpec("workbench", (32, 43)))
    # Then every check passes and image bytes are preserved.
    assert result.passed
    assert result.base_row == 43
    assert result.height == 34
    assert result.light_angle_degrees == pytest.approx(225.0)
    assert source.tobytes() == before


@pytest.mark.parametrize("anchor_y, expected", [(41, True), (45, True), (40, False), (46, False)])
def test_base_tolerance_is_two_pixels(anchor_y: int, expected: bool) -> None:
    # Given a known base row and an independently declared anchor.
    checker = importlib.import_module("art.compositor.projection_check")
    # When compared without normalization.
    result = checker.check_projection(sprite(), checker.ProjectionSpec("workbench", (32, anchor_y)))
    # Then the frozen two-pixel tolerance applies inclusively.
    assert result.base_passed is expected


def test_opposite_light_fails_even_when_geometry_passes() -> None:
    # Given a correctly sized body lit from bottom right.
    checker = importlib.import_module("art.compositor.projection_check")
    # When checked.
    result = checker.check_projection(sprite(reverse=True), checker.ProjectionSpec("workbench", (32, 43)))
    # Then lighting rejects it independently of geometry.
    assert result.base_passed and result.height_passed
    assert not result.light_passed
    assert not result.passed
    assert result.light_angle_degrees == pytest.approx(45.0)


def test_wrong_height_fails() -> None:
    # Given an oversized bench.
    checker = importlib.import_module("art.compositor.projection_check")
    # When compared with the workbench band.
    result = checker.check_projection(sprite(height=42), checker.ProjectionSpec("workbench", (32, 43)))
    # Then its correct anchor and light cannot conceal the height failure.
    assert not result.height_passed
    assert not result.passed


@pytest.mark.parametrize("colour", [(0, 0, 0, 0), (150, 150, 150, 255)])
def test_no_measurable_light_is_inconclusive(colour: tuple[int, int, int, int]) -> None:
    # Given transparent or flat artwork.
    checker = importlib.import_module("art.compositor.projection_check")
    # When checked.
    result = checker.check_projection(Image.new("RGBA", (64, 64), colour), checker.ProjectionSpec("tree", (32, 63)))
    # Then a missing light estimate never passes.
    assert result.light_angle_degrees is None
    assert not result.light_passed
    assert not result.passed


def test_alpha_threshold_excludes_faint_shadow() -> None:
    # Given faint shadow below the solid object.
    checker = importlib.import_module("art.compositor.projection_check")
    source = sprite()
    source.putpixel((32, 62), (0, 0, 0, 127))
    # When checking the alpha>=128 silhouette.
    result = checker.check_projection(source, checker.ProjectionSpec("workbench", (32, 43)))
    # Then the shadow cannot move its measured base.
    assert result.base_row == 43
    assert result.height == 34


@pytest.mark.parametrize("reverse, code", [(False, 0), (True, 1)])
def test_cli_reports_measurements_and_rejection(tmp_path: Path, reverse: bool, code: int) -> None:
    # Given an on-disk candidate with known lighting.
    path = tmp_path / "candidate.png"
    sprite(reverse=reverse).save(path)
    # When running the real instrument CLI.
    process = subprocess.run(
        [sys.executable, "-m", "art.compositor.projection_check", str(path), "workbench", "32", "43"],
        check=False, capture_output=True, text=True,
    )
    # Then the command emits numeric evidence and a meaningful status code.
    result = json.loads(process.stdout)
    assert process.returncode == code
    assert result["passed"] is (code == 0)
    assert result["height"] == 34
    assert result["base_row"] == 43
