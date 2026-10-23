# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image", "pytest"]
# ///
# How to run: python -m pytest art/pipeline5/prod2_test_sleep.py
"""PROD2 body-length normalization and age-specific acceptance regressions."""
import json
from dataclasses import replace
from pathlib import Path

import numpy as np
import pytest
from PIL import Image, ImageDraw

from art.pipeline5.prod1_process import process
from art.pipeline5.prod1_test_process import fixture_request, passing_judge
from art.pipeline5.sheet_geometry import normalize_panel
from art.pipeline5.whole_checks import check_whole


@pytest.mark.parametrize(('age', 'target'), [('adult', 56), ('child', 40)])
def test_oversized_lying_capsule_normalizes_to_body_length(age: str, target: int) -> None:
    # Given a lying capsule 1.6 times the 200px standing source height.
    source = Image.new('RGBA', (340, 100))
    ImageDraw.Draw(source).rounded_rectangle((10, 10, 329, 89), radius=39, fill=(100, 70, 40, 254))
    before = source.tobytes()
    # When the sleep-only length calibration is applied.
    result = normalize_panel(source, target / 200, True, sleep_length=target)
    # Then the requested age band and exact baseline hold without changing source alpha.
    checked = check_whole(result.image, age, 'sleep')
    assert checked.passed, checked.findings
    assert abs(checked.opaque_width - target) <= 1
    assert np.nonzero(np.asarray(result.image)[:, :, 3] >= 16)[0].max() == 72
    assert result.transform.scale == target / 320
    assert source.tobytes() == before


def test_standing_figure_remains_byte_identical() -> None:
    # Given the same standing panel and S-idle scale.
    source = Image.new('RGBA', (70, 200), (100, 70, 40, 254))
    original = normalize_panel(source, 56 / 200, False)
    # When a sleep length is supplied for a standing frame.
    current = normalize_panel(source, 56 / 200, False, sleep_length=56)
    # Then no standing transform or pixel changes.
    assert current.image.tobytes() == original.image.tobytes()
    assert current.transform == original.transform


@pytest.mark.parametrize(('age', 'width', 'height', 'passed'), [
    ('adult', 51, 20, False), ('adult', 52, 36, True), ('adult', 60, 36, True),
    ('adult', 61, 20, False), ('adult', 56, 37, False),
    ('child', 36, 20, False), ('child', 37, 28, True), ('child', 43, 28, True),
    ('child', 44, 20, False), ('child', 40, 29, False),
])
def test_age_specific_sleep_bounds(age: str, width: int, height: int, passed: bool) -> None:
    # Given a centred, connected sleep frame on its contact line.
    source = Image.new('RGBA', (112, 80))
    ImageDraw.Draw(source).rectangle((56-width//2, 73-height, 55-width//2+width, 72), fill=(1, 2, 3, 255))
    # When structural rules are checked.
    result = check_whole(source, age, 'sleep')
    # Then both length and height limits apply to the correct age.
    assert result.passed == passed, result.findings


@pytest.mark.parametrize(('age', 'target'), [('adult', 56), ('child', 40)])
def test_production_sidecars_record_both_scales(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, age: str, target: int) -> None:
    # Given a real ten-panel source with oversized sleep figures.
    request = fixture_request(tmp_path)
    request = replace(request, age=age, look=f'{age}_M_hide_wrap_long_tied')
    with Image.open(request.raw) as source:
        draw = ImageDraw.Draw(source)
        for row in range(2):
            draw.rectangle((1240, row*512+100, 1439, row*512+179), fill=(0, 0, 0, 0))
            draw.rectangle((1230, row*512+100, 1529, row*512+179), fill=(100, 70, 40, 255))
        source.save(request.raw)
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', passing_judge)
    # When normalization and publication run through the production entrypoint.
    accepted = process(request)
    # Then every delivered frame records standing and facing-specific sleep scales.
    assert accepted
    for path in (request.output_root / request.look).glob('*/*.json'):
        sidecar = json.loads(path.read_text())
        assert sidecar['standing_scale'] == target / 200
        assert sidecar['sleep_scale'] == target / 300
        assert sidecar['scale'] == (target / 300 if sidecar['frame'] == 'sleep' else target / 200)
