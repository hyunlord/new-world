# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image", "pytest"]
# ///
# How to run: python -m pytest art/pipeline5/prod1_test_process.py
"""Production publication and attempt-boundary regressions."""
import importlib.util


def test_production_processor_exists() -> None:
    # Given a production run needs an independent paired-facing entrypoint.
    # When its module is resolved.
    module = importlib.util.find_spec('art.pipeline5.prod1_process')
    # Then the entrypoint exists.
    assert module is not None

import json
import subprocess
import sys
from dataclasses import replace
from pathlib import Path

import pytest
from PIL import Image, ImageDraw

from art.pipeline2.judge import Metrics, Verdict
from art.pipeline5.attempt_integrity import digest
from art.pipeline5.judge_panels import CALIBRATION_SHA256, PanelVerdict, SheetVerdict
from art.pipeline5.prod1_process import process
from art.pipeline5.prod1_records import ProductionRequest


def fixture_request(root: Path) -> ProductionRequest:
    image = Image.new('RGBA', (1536, 1024))
    draw = ImageDraw.Draw(image)
    for row in range(2):
        for col in range(5):
            x, y = col * 300 + 40, row * 512 + 100
            width, height = (200, 80) if col == 4 else (70, 200)
            draw.rectangle((x, y, x + width - 1, y + height - 1), fill=(100, 70, 40, 255))
    raw = root / 'raw.png'
    image.save(raw)
    prompt = root / 'prompt.txt'
    prompt.write_text('Synthetic fixture only')
    return ProductionRequest(raw, 'adult_M_hide_wrap_long_tied', 'adult', 1, prompt,
                             root / 'out', root / 'diagnostics')


def passing_judge(source: Path, panels: tuple[Image.Image, ...], output: Path,
                  *, expected_count: int = 10) -> SheetVerdict:
    """Only numeric boundary is substituted; slicing, checking and delivery stay real."""
    metrics = Metrics(0, 0, 0, 0, 0, 0, 0, 0, 0)
    records = tuple(PanelVerdict(index, panel.size, '', (0, 0, *panel.size), (0, 0), panel.size,
                                Verdict(str(source), digest(source), 'person', metrics,
                                        {'synthetic': True}, True, None, 'MISSING', False))
                    for index, panel in enumerate(panels))
    return SheetVerdict(str(source), digest(source), CALIBRATION_SHA256,
                        expected_count, records, True)


def test_all_ten_pass_publishes_complete_look(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given a geometry-valid pair of facings and an isolated passing numeric boundary.
    request = fixture_request(tmp_path)
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', passing_judge)
    # When the sheet is processed.
    accepted = process(request)
    # Then both facings are delivered with verified hashes and one shared scale.
    assert accepted
    target = request.output_root / request.look
    assert len(list(target.glob('*/*.png'))) == 10
    manifest = json.loads((target / 'MANIFEST.json').read_text())
    assert manifest['hair'] == 'long_tied'
    assert manifest['top'] == 'hide_wrap'
    for sidecar in target.glob('*/*.json'):
        data = json.loads(sidecar.read_text())
        assert data['checks']['passed'] and data['judge']['numeric_pass']
        assert data['image_sha256'] == digest(sidecar.with_suffix('.png'))
        assert data['scale'] == 56 / 200


def test_one_failing_frame_prevents_both_facing_publication(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given a wide carry pose that cannot fit without forbidden per-pose scaling.
    request = fixture_request(tmp_path)
    with Image.open(request.raw) as raw:
        image = raw.copy()
    ImageDraw.Draw(image).rectangle((940, 100, 1230, 299), fill=(100, 70, 40, 255))
    image.save(request.raw)
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', passing_judge)
    # When the sheet is evaluated.
    accepted = process(request)
    # Then the look is missing and every frame has retained evidence.
    assert not accepted
    assert not (request.output_root / request.look).exists()
    record = json.loads((request.diagnostic / 'sheet_checks.json').read_text())
    assert len(record['frames']) == 10
    assert any(not frame['passed'] for frame in record['frames'])


def test_accepted_look_stops_later_attempt(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given an accepted first attempt.
    request = fixture_request(tmp_path)
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', passing_judge)
    assert process(request)
    manifest = request.output_root / request.look / 'MANIFEST.json'
    original = manifest.read_bytes()
    # When another try is submitted, then it is refused without changing acceptance.
    with pytest.raises(ValueError, match='first_pass_stop'):
        process(replace(request, attempt=2))
    assert manifest.read_bytes() == original


def test_attempt_four_is_refused(tmp_path: Path) -> None:
    # Given the three-attempt production contract.
    request = fixture_request(tmp_path)
    # When a fourth attempt is requested, then boundary parsing rejects it.
    with pytest.raises(ValueError, match='attempt 1..3'):
        replace(request, attempt=4)


def test_cli_rejects_bad_grid_with_ten_frame_records(tmp_path: Path) -> None:
    # Given an actual CLI dispatch whose sheet has no components.
    request = fixture_request(tmp_path)
    Image.new('RGBA', (1536, 1024)).save(request.raw)
    dispatch = tmp_path / 'dispatch.json'
    dispatch.write_text(json.dumps({'raw': str(request.raw), 'look': request.look,
        'age': request.age, 'attempt': 1, 'prompt_path': str(request.prompt_path)}))
    # When the real CLI processes it.
    result = subprocess.run([sys.executable, '-m', 'art.pipeline5.prod1_process', str(dispatch),
        '--output-root', str(request.output_root), '--diagnostics-root', str(request.diagnostics_root)],
        capture_output=True, text=True, check=False)
    # Then it reports rejection, retains ten explicit failures, and publishes nothing.
    assert result.returncode == 1, result.stderr
    record = json.loads((request.diagnostic / 'sheet_checks.json').read_text())
    assert len(record['frames']) == 10
    assert all(not frame['passed'] for frame in record['frames'])
    assert not request.output_root.exists()


def rejecting_judge(source: Path, panels: tuple[Image.Image, ...], output: Path,
                    *, expected_count: int = 10) -> SheetVerdict:
    result = passing_judge(source, panels, output, expected_count=expected_count)
    failed = replace(result.panels[7], verdict=replace(result.panels[7].verdict,
                     numeric_pass=False, numeric_checks={'synthetic': False}))
    return replace(result, panels=(*result.panels[:7], failed, *result.panels[8:]), numeric_pass=False)


def test_numeric_failure_rejects_even_with_all_geometry_passed(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given a sheet with just one failed panel numeric gate.
    request = fixture_request(tmp_path)
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', rejecting_judge)
    # When the entire sheet is evaluated.
    accepted = process(request)
    # Then no partial acceptance can leak and the numeric failure is retained.
    assert not accepted
    record = json.loads((request.diagnostic / 'sheet_checks.json').read_text())
    assert record['passing_panels'] == 9
    assert all(frame['checks']['passed'] for frame in record['frames'])
    assert not record['frames'][7]['judge']['numeric_pass']
    assert not request.output_root.exists()


def test_skipped_attempt_rejected(tmp_path: Path) -> None:
    # Given no preceding attempt.
    request = replace(fixture_request(tmp_path), attempt=2)
    # When try2 is processed, then the attempt sequence is refused.
    with pytest.raises(ValueError, match='attempt_sequence'):
        process(request)


def test_failed_review_preserves_original_acceptance(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given an accepted look selected for a targeted readability retry.
    request = fixture_request(tmp_path)
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', passing_judge)
    assert process(request)
    manifest = request.output_root / request.look / 'MANIFEST.json'
    original = manifest.read_bytes()
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', rejecting_judge)
    # When the targeted retry fails its numeric gate.
    accepted = process(replace(request, attempt=2, review_reason='hair misread'))
    # Then the prior accepted set is preserved while rejection remains visible.
    assert not accepted
    assert manifest.read_bytes() == original
    assert (request.diagnostics_root / request.look / 'try2/sheet_checks.json').exists()


def test_successful_review_atomically_replaces_and_archives(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given an accepted look with remaining generation budget.
    request = fixture_request(tmp_path)
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', passing_judge)
    assert process(request)
    manifest = request.output_root / request.look / 'MANIFEST.json'
    original = manifest.read_bytes()
    retry = replace(request, attempt=2, review_reason='hair misread')
    # When a passing targeted retry replaces it.
    accepted = process(retry)
    # Then both facings come from try2 and all original delivery bytes survive.
    assert accepted
    assert json.loads(manifest.read_text())['attempt'] == 2
    assert (retry.diagnostic / 'previous_accepted/MANIFEST.json').read_bytes() == original
    assert len(list(manifest.parent.glob('*/*.png'))) == 10


def test_second_review_round_refused(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given one completed targeted review retry.
    request = fixture_request(tmp_path)
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', passing_judge)
    assert process(request)
    assert process(replace(request, attempt=2, review_reason='hair misread'))
    # When a second review round is requested, then the preregistered cap holds.
    with pytest.raises(ValueError, match='review_round_limit'):
        process(replace(request, attempt=3, review_reason='still misread'))


def test_normalized_pngs_identical_across_two_fresh_runs(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Given identical raw bytes and the same transforms in isolated outputs.
    first_root, second_root = tmp_path / 'first', tmp_path / 'second'
    first_root.mkdir()
    second_root.mkdir()
    first, second = fixture_request(first_root), fixture_request(second_root)
    monkeypatch.setattr('art.pipeline5.prod1_process.judge_panels', passing_judge)
    # When both runs normalize and deliver the sheet.
    accepted = (process(first), process(second))
    # Then all ten PNG bytes agree, independent of output location.
    assert accepted == (True, True)
    first_output, second_output = first.output_root / first.look, second.output_root / second.look
    assert {str(p.relative_to(first_output)): digest(p) for p in first_output.glob('*/*.png')} == {
        str(p.relative_to(second_output)): digest(p) for p in second_output.glob('*/*.png')}


def test_real_numeric_judge_runs_on_all_ten_source_panels(tmp_path: Path) -> None:
    # Given a deliberately flat-color synthetic sheet with a valid grid.
    request = fixture_request(tmp_path)
    # When the complete real pipeline measures it without numeric substitution.
    accepted = process(request)
    # Then all ten source measurements exist and frozen style gates reject the fixture.
    record = json.loads((request.diagnostic / 'sheet_checks.json').read_text())
    assert not accepted
    assert len(record['frames']) == 10
    assert len(list((request.diagnostic / 'judge').glob('panel_*.png'))) == 10
    assert any(not frame['judge']['numeric_pass'] for frame in record['frames'])
    assert digest(request.raw) == digest(request.diagnostic / 'raw.png')
    assert digest(request.prompt_path) == digest(request.diagnostic / 'prompt.txt')


def test_global_processed_attempt_cap_enforced(tmp_path: Path) -> None:
    # Given the full 108-sheet processed budget has been reserved already.
    request = fixture_request(tmp_path)
    for index in range(108):
        (request.diagnostics_root / f'historical{index}' / 'try1').mkdir(parents=True)
    # When another sheet is submitted, then processing refuses the extra attempt.
    with pytest.raises(ValueError, match='generation_limit'):
        process(request)
