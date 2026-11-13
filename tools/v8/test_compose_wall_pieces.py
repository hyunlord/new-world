# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "pydantic"]
# ///
# How to run: uv run --with pytest --with pillow --with numpy --with pydantic python -m pytest tools/v8/test_compose_wall_pieces.py -q.
"""Exercise the wall-piece export CLI against the existing material renderer."""

import subprocess
import sys
from pathlib import Path
from typing import ClassVar

from PIL import Image
from pydantic import BaseModel, ConfigDict


class ExportPiece(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    mask: int


class Doors(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    ns: None


class Capability(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    runtime_rows_to_append: tuple[str, ...]
    doors: Doors


class ExportManifest(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    pieces: tuple[ExportPiece, ...]
    anchor: tuple[int, int]
    capability: Capability


ROOT = Path(__file__).resolve().parents[2]


def test_cli_exports_all_masks_and_preserves_caps(tmp_path: Path) -> None:
    # Given an empty export directory and the shipped drystone material kit.
    target = tmp_path / "walls"
    # When the actual CLI exports that kit.
    result = subprocess.run(
        [sys.executable, "-m", "tools.v8.compose_wall_pieces", str(target)],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    # Then every mask is present, anchored, and its cap aliases are exact copies.
    assert result.returncode == 0, result.stderr
    manifest = ExportManifest.model_validate_json(
        (target / "manifest.json").read_bytes()
    )
    assert {row.mask for row in manifest.pieces} == set(range(16))
    assert manifest.anchor == (32, 59)
    for mask in range(16):
        with Image.open(target / f"stone_wall_{mask:02}.png") as image:
            assert image.size == (64, 64)
            assert image.mode == "RGBA"
            assert image.getchannel("A").getbbox() is not None
    assert (target / "corner_cap_ne.png").read_bytes() == (
        target / "stone_wall_03.png"
    ).read_bytes()
    assert manifest.capability.runtime_rows_to_append == ()
    assert manifest.capability.doors.ns is None


def test_cli_refuses_to_overwrite_exports(tmp_path: Path) -> None:
    # Given an occupied destination containing owner data.
    _ = (tmp_path / "owner.txt").write_text("keep")
    # When the CLI is asked to write there.
    result = subprocess.run(
        [sys.executable, "-m", "tools.v8.compose_wall_pieces", str(tmp_path)],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    # Then it fails and preserves that data.
    assert result.returncode != 0
    assert (tmp_path / "owner.txt").read_text() == "keep"
