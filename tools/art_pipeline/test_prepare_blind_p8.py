# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 -m pytest tools/art_pipeline/test_prepare_blind_p8.py
"""Frozen blind evidence must preserve bytes and reject key replacement."""
import hashlib
import json
from pathlib import Path

import pytest

from prepare_blind_p8 import freeze, source


def test_source_copy_retains_exact_pixel_file_bytes(tmp_path: Path) -> None:
    # Given: an opaque source fixture.
    original = tmp_path / 'original.png'
    original.write_bytes(b'test fixture original image bytes')
    destination = tmp_path / 'packet' / 'O001.png'
    # When: the neutral packet is prepared.
    binding = source(original, destination)
    # Then: original and shown bytes have the same recorded SHA.
    assert destination.read_bytes() == original.read_bytes()
    assert binding['sha256'] == hashlib.sha256(destination.read_bytes()).hexdigest()


def test_freeze_rejects_overwriting_preexisting_answer_key(tmp_path: Path) -> None:
    # Given: a frozen answer written before observers.
    path = tmp_path / 'key.json'
    freeze(path, {'answer': 'N'})
    # When: a later operation attempts to replace the answer.
    with pytest.raises(FileExistsError):
        freeze(path, {'answer': 'S'})
    # Then: the original answer and matching digest survive unchanged.
    assert json.loads(path.read_text()) == {'answer': 'N'}
    assert path.with_suffix('.sha256').read_text().strip() == hashlib.sha256(path.read_bytes()).hexdigest()
