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


class NativeGeometry(BaseModel):
    """The immutable shipped W2 dimensions required by pass2."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    wall_width: int
    cap_height: int
    face_height: int
    anchor: tuple[int, int]


def test_native_cli_exports_thin_wall_masks_caps_and_oriented_doors(
    tmp_path: Path,
) -> None:
    # Given the sharp native stone field and an empty destination.
    target = tmp_path / "native"
    # When the actual wall-piece CLI composes a pass2 delivery.
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "tools.v8.compose_wall_pieces",
            "--native",
            "art/materials/textures/drystone/face.png",
            str(target),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    # Then all topology/cap/door images exist with unchanged W2 dimensions.
    assert result.returncode == 0, result.stderr
    geometry = NativeGeometry.model_validate_json(
        (target / "manifest.json").read_bytes()
    )
    assert (geometry.wall_width, geometry.cap_height, geometry.face_height) == (
        26,
        16,
        22,
    )
    assert geometry.anchor == (32, 59)
    assert len(list(target.glob("mask_*.png"))) == 16
    assert len(list(target.glob("corner_*.png"))) == 4
    assert len(list(target.glob("end_*.png"))) == 4
    assert (target / "corner_ne.png").read_bytes() == (
        target / "mask_03.png"
    ).read_bytes()
    for orientation in ("ew", "ns"):
        for state in ("closed", "open"):
            with Image.open(target / f"door_{orientation}_{state}.png") as image:
                assert image.size == (64, 60)
                assert image.getchannel("A").getbbox() is not None
    with (
        Image.open(target / "rooms_1x.png") as one,
        Image.open(target / "rooms_2x.png") as two,
    ):
        assert two.size == (one.width * 2, one.height * 2)
