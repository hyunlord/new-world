# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "numpy", "scipy", "pydantic"]
# ///
# How to run: python3 -m pytest art/animal/batch_a1/test_adopt.py
"""Protect the assembler's byte-preservation boundary."""
from pathlib import Path

import pytest

from art.animal.batch_a1 import adopt
from art.pipeline_env1.animal_normalize import digest


def test_copy_keeps_encoded_bytes(tmp_path: Path) -> None:
    # Given arbitrary encoded source bytes; copying must not invoke an image encoder.
    source = tmp_path / "source.png"
    _ = source.write_bytes(b"immutable PNG bytes including original metadata")
    target = tmp_path / "adopted" / "frame.png"
    # When copied through the verified delivery boundary.
    adopt.copy_exact(source, target, digest(source))
    # Then the delivered file is byte-identical.
    assert target.read_bytes() == source.read_bytes()


def test_wrong_source_hash_preserves_existing_delivery(tmp_path: Path) -> None:
    # Given a changed source and a previously delivered target.
    source, target = tmp_path / "source.png", tmp_path / "delivered.png"
    _ = source.write_bytes(b"changed source")
    _ = target.write_bytes(b"previous delivery")
    # When the expected source hash differs, copying must fail before replacement.
    with pytest.raises(ValueError, match="Source hash changed"):
        adopt.copy_exact(source, target, "0" * 64)
    # Then the previous delivery remains byte-identical.
    assert target.read_bytes() == b"previous delivery"
