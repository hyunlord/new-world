# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "pytest"]
# ///
# How to run: uv run --with pillow --with pydantic --with pytest python -m pytest art/techmap/test_assets.py
"""Reject invalid exports and preserve the source during mechanical resizing."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path
from typing import Literal, assert_never

import pytest
from PIL import Image

from art.techmap import assets


@pytest.mark.parametrize("defect", ["blank", "opaque", "clipped", "wrong_size"])
def test_check_rejects_invalid_export(
    tmp_path: Path,
    defect: Literal["blank", "opaque", "clipped", "wrong_size"],
) -> None:
    # Given one concrete defect in a nominal icon export.
    source = Image.new("RGBA", (64, 64))
    source.paste((90, 70, 40, 255), (6, 6, 58, 58))
    match defect:
        case "blank":
            source = Image.new("RGBA", (64, 64))
        case "opaque":
            source = Image.new("RGBA", (64, 64), (90, 70, 40, 255))
        case "clipped":
            source.paste((90, 70, 40, 255), (0, 20, 10, 40))
        case "wrong_size":
            source = source.resize((48, 48))
        case unreachable:
            assert_never(unreachable)
    path = tmp_path / "bad.png"
    source.save(path)
    # When checking against the declared 64px contract.
    result = assets.measure(path, assets.Expectation(canvas=(64, 64)))
    # Then the defective image cannot pass.
    assert not result.passed


def test_normalization_preserves_content_alpha_and_proportion(tmp_path: Path) -> None:
    # Given a 40x80 solid subject with an enclosed alpha hole.
    source = Image.new("RGBA", (100, 120))
    source.paste((100, 150, 200, 255), (30, 20, 70, 100))
    source.paste((0, 0, 0, 0), (40, 40, 60, 80))
    raw = tmp_path / "raw.png"
    source.save(raw)
    before = raw.read_bytes()
    output = tmp_path / "normalized.png"
    # When mechanically fitting the icon canvas.
    result = assets.normalize(raw, output, assets.Family.ICON)
    # Then original bytes and transparent hole survive; aspect stays 1:2.
    assert raw.read_bytes() == before
    assert result.output.passed
    assert result.output.canvas == (64, 64)
    assert result.anchor == (32, 32)
    assert result.output.bbox128 is not None
    left, top, right, bottom = result.output.bbox128
    assert abs((right - left) * 2 - (bottom - top)) <= 1
    with Image.open(output) as image:
        assert image.getchannel("A").getpixel((32, 32)) == 0


def test_faint_canvas_residue_does_not_shrink_subject(tmp_path: Path) -> None:
    # Given near-invisible generated alpha noise that extends to the raw canvas edge.
    source = Image.new("RGBA", (100, 100), (0, 0, 0, 1))
    source.paste((0, 0, 0, 0), (1, 1, 99, 99))
    source.paste((100, 150, 200, 255), (40, 30, 60, 70))
    raw = tmp_path / "faint_residue.png"
    source.save(raw)
    # When normalized using the existing solid-alpha crop with a two-pixel fringe.
    result = assets.normalize(raw, tmp_path / "icon.png", assets.Family.ICON)
    # Then the actual subject determines scale and discarded noise is disclosed.
    assert result.crop == (38, 28, 62, 72)
    assert result.resized == (32, 58)
    assert result.discarded_alpha_pixels == 396
    assert result.discarded_alpha_max == 1


def test_hash_mismatch_fails_even_when_pixels_are_valid(tmp_path: Path) -> None:
    # Given a legitimate transparent icon with an incorrect expected byte identity.
    source = Image.new("RGBA", (64, 64))
    source.paste((90, 70, 40, 255), (6, 6, 58, 58))
    path = tmp_path / "icon.png"
    source.save(path)
    # When the declared SHA differs.
    result = assets.measure(path, assets.Expectation(canvas=(64, 64), sha256="0" * 64))
    # Then structural success cannot hide a stale or replaced artifact.
    assert result.hash_matches is False
    assert not result.passed


def test_cli_rejects_clipped_raw_without_creating_export(tmp_path: Path) -> None:
    # Given a generated image whose subject was clipped at its source boundary.
    source = Image.new("RGBA", (64, 64))
    source.paste((90, 70, 40, 255), (0, 5, 50, 55))
    raw = tmp_path / "clipped.png"
    source.save(raw)
    output = tmp_path / "exports"
    evidence = tmp_path / "checks"
    # When using the actual CLI rather than calling the checker in isolation.
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "art.techmap.assets",
            "icon",
            str(output),
            str(evidence),
            str(raw),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    # Then failure is observable, with a record but no normalized image.
    assert result.returncode == 1
    assert list(evidence.glob("*.json"))
    assert not list(output.glob("*.png"))
