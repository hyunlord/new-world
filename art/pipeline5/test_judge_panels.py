# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image", "pytest"]
# ///
# How to run: python -m pytest art/pipeline5/test_judge_panels.py
"""Source pixel preservation and non-vacuous panel acceptance boundaries."""
from pathlib import Path

import pytest
from PIL import Image

from art.pipeline5 import judge_panels


def test_crop_preserves_translucent_pixels_and_adds_eight_percent() -> None:
    source = Image.new('RGBA', (80, 90))
    figure = Image.new('RGBA', (25, 50), (70, 40, 20, 128))
    figure.putpixel((0, 0), (5, 6, 7, 1))
    source.paste(figure, (12, 15))
    result = judge_panels.prepare_panel(source)
    assert result.crop_box == (12, 15, 37, 65)
    assert result.padding == (2, 4)
    assert result.image.size == (29, 58)
    assert result.image.crop((2, 4, 27, 54)).tobytes() == figure.tobytes()


def test_empty_panel_is_rejected() -> None:
    with pytest.raises(ValueError, match='empty'):
        _ = judge_panels.prepare_panel(Image.new('RGBA', (10, 10)))


def test_sheet_requires_exact_panel_count_before_writing(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match='exactly 10'):
        _ = judge_panels.judge_panels(tmp_path / 'missing.png', (), tmp_path)
    assert list(tmp_path.iterdir()) == []


def test_person_anchor_uses_frozen_judge_and_writes_bound_evidence(tmp_path: Path) -> None:
    from art.pipeline2.judge import judge, load_calibration

    source = judge_panels.ROOT / 'art/style05/anchors/crops/person_full.png'
    with Image.open(source) as image:
        panel = image.convert('RGBA')
    direct = judge(source, load_calibration(judge_panels.CALIBRATION), family='person')
    result = judge_panels.judge_panels(source, (panel,), tmp_path, expected_count=1)
    measured = result.panels[0].verdict
    assert direct.numeric_pass
    assert result.numeric_pass
    assert measured.metrics == direct.metrics
    assert measured.numeric_checks == direct.numeric_checks
    assert result.calibration_sha256 == judge_panels.CALIBRATION_SHA256
    assert measured.model_status == 'MISSING'
    assert not measured.passed
    assert not result.legacy_whole_sheet
    assert (tmp_path / 'panel_judge.json').is_file()


def test_one_bad_panel_rejects_entire_sheet(tmp_path: Path) -> None:
    source = judge_panels.ROOT / 'art/style05/anchors/crops/person_full.png'
    with Image.open(source) as image:
        panel = image.convert('RGBA')
    bad = Image.new('RGBA', (32, 64), (255, 255, 255, 255))
    result = judge_panels.judge_panels(source, (panel, panel, bad, panel, panel),
                                     tmp_path, expected_count=5)
    assert not result.numeric_pass
    assert len(result.panels) == 5
    assert sum(record.verdict.numeric_pass for record in result.panels) == 4
