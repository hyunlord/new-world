# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image", "pytest"]
# ///
# How to run: see README.md for the dependency-complete uv command.
# python -m pytest art/batch_a2/test_process.py
"""A2 candidate provenance and rejection tests."""

from __future__ import annotations

import importlib.util
from pathlib import Path

import numpy as np
import pytest
from PIL import Image
from pydantic import ValidationError

from art.batch_a2.process import Record, preserve, process, raw_alpha


def test_batch_exporter_exists() -> None:
    # Given a candidate workflow that must never run the historical batch writer.
    # When locating the isolated A2 exporter.
    spec = importlib.util.find_spec("art.batch_a2.process")
    # Then a dedicated entry point must exist.
    assert spec is not None


def record_for(path: Path) -> Record:
    return Record(
        id="test_sprite",
        lane="test",
        era="bronze",
        attempt=1,
        raw=path,
        prompt="A test fixture, never a generated asset.",
        references=(
            Path("art/style05/anchors/props.png"),
            Path("art/style05/anchors/scene.png"),
        ),
        target_height=35,
        projection_family="workbench",
    )


def test_raw_identity_conflict_does_not_overwrite(tmp_path: Path) -> None:
    # Given a prior attempt already preserved with different bytes.
    source, target = tmp_path / "source", tmp_path / "target"
    _ = source.write_bytes(b"new generation")
    _ = target.write_bytes(b"prior generation")
    # When trying to reuse its attempt id.
    with pytest.raises(ValueError, match="identity conflict"):
        preserve(source, target)
    # Then the prior raw evidence survives unchanged.
    assert target.read_bytes() == b"prior generation"


def test_opaque_background_is_rejected_with_record(tmp_path: Path) -> None:
    # Given an RGB generation with no generated transparent background.
    raw = tmp_path / "opaque.png"
    Image.new("RGB", (64, 64), (100, 80, 60)).save(raw)
    # When exporting it.
    result = process(record_for(raw), tmp_path / "evidence")
    # Then it remains rejected, with raw bytes and a durable diagnostic sidecar.
    assert result.status == "CANDIDATE_REJECTED"
    assert result.normalized is None
    assert "lacks generated alpha" in result.failures[0]
    assert Path(result.raw_preserved).read_bytes() == raw.read_bytes()
    assert (tmp_path / "evidence/test/normalized/test_sprite_try1.json").exists()


def test_export_preserves_source_and_declares_actual_anchor(tmp_path: Path) -> None:
    # Given a source with a 40x80 opaque body, surrounded by generated alpha.
    raw = tmp_path / "transparent.png"
    image = Image.new("RGBA", (100, 120))
    image.paste((130, 100, 70, 255), (30, 20, 70, 100))
    image.save(raw)
    before = raw.read_bytes()
    # When uniformly exported using the existing STRUCT normalizer.
    result = process(record_for(raw), tmp_path / "evidence")
    # Then the source is unchanged, and the actual opaque base agrees with metadata.
    assert result.normalized is not None
    with Image.open(result.normalized) as exported:
        rows = np.nonzero(np.asarray(exported)[:, :, 3] >= 128)[0]
    assert result.anchor is not None
    assert result.anchor[1] == int(rows.max())
    assert int(rows.max() - rows.min() + 1) == 35
    assert raw.read_bytes() == before
    assert result.review is None
    assert result.record.seed is None


def test_existing_height_band_cannot_be_relaxed(tmp_path: Path) -> None:
    # Given an existing workbench target band of 30..40 pixels.
    data = record_for(tmp_path / "source.png").model_dump()
    data.update(height_band=(1, 100), height_band_reason="A tempting shortcut")
    # When parsing a record that tries to widen the accepted band.
    # Then input is rejected before an image can be processed.
    with pytest.raises(ValidationError, match="cannot be relaxed"):
        _ = Record.model_validate(data)


def test_raw_border_clipping_remains_a_rejection(tmp_path: Path) -> None:
    # Given a transparent image whose subject is cut off at the left border.
    raw = tmp_path / "clipped.png"
    image = Image.new("RGBA", (80, 100))
    image.paste((130, 100, 70, 255), (0, 20, 50, 80))
    image.save(raw)
    # When the cropped candidate would otherwise have clean padded borders.
    result = process(record_for(raw), tmp_path / "evidence")
    # Then the original clipping is not hidden by normalization.
    assert result.clipped_at_raw_border
    assert "raw subject touches image border" in result.failures
    assert not result.numeric_ready


def test_raw_faint_halo_is_reported_without_removal() -> None:
    # Given a faint wash outside an otherwise opaque object's bounds.
    image = Image.new("RGBA", (100, 100))
    image.paste((100, 70, 40, 40), (5, 5, 95, 95))
    image.paste((100, 70, 40, 255), (30, 30, 70, 70))
    before = image.tobytes()
    # When inspecting raw alpha before normalizing.
    _, _, nonzero, opaque, far_ratio, _ = raw_alpha(image)
    # Then separate bboxes and the faint outer area remain observable.
    assert nonzero == (5, 5, 95, 95)
    assert opaque == (30, 30, 70, 70)
    assert far_ratio > 0.5
    assert image.tobytes() == before
