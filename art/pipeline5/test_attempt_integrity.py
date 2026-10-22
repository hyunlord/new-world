# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image", "pytest"]
# ///
# How to run: uv run --with pytest --with pillow --with numpy --with scipy --with scikit-image python -m pytest art/pipeline5/test_attempt_integrity.py
"""Regression checks for forged verdicts, stale cache identities and partial copies."""
import json
from dataclasses import asdict
from pathlib import Path

import pytest

from art.pipeline2.judge import judge, load_calibration
from art.pipeline5.attempt_integrity import CALIBRATION, digest, verify_judge
from art.pipeline5.process_sheets import process_result
from art.pipeline5.sheet_slice import SheetRequest, process
from art.pipeline5.test_whole_figures import fixture_binding, fixture_sheet


def test_judge_rejects_wrong_raw_family_and_modified_numeric(tmp_path: Path) -> None:
    raw = tmp_path/'raw.png'
    fixture_sheet().save(raw)
    verdict = asdict(judge(raw, load_calibration(CALIBRATION), family='person'))
    path = tmp_path/'judge.json'
    path.write_text(json.dumps(verdict))
    assert verify_judge(path, raw).raw_sha256 == digest(raw)
    for key, value in [('sha256', 'other'), ('family', 'environment'), ('numeric_pass', not verdict['numeric_pass'])]:
        altered = {**verdict, key: value}
        path.write_text(json.dumps(altered))
        with pytest.raises(ValueError, match='judge_identity'):
            verify_judge(path, raw)
    path.write_text(json.dumps({'numeric_pass': True}))
    with pytest.raises(ValueError, match='judge_identity'):
        verify_judge(path, raw)


def test_copy_failure_never_exposes_partial_delivery(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    raw = tmp_path/'raw.png'
    fixture_sheet().save(raw)
    prompt = tmp_path/'prompt.txt'
    prompt.write_text('synthetic fixture')
    verdict = tmp_path/'judge.json'
    verdict.write_text(json.dumps({'numeric_pass': True}))
    request = SheetRequest(raw, 'adult_M_hide_wrap_short', 'adult', 'S', 1, prompt, verdict,
                           tmp_path/'out', tmp_path/'diagnostics')
    monkeypatch.setattr('art.pipeline5.sheet_slice.verify_judge', fixture_binding)
    import shutil
    copy = shutil.copyfile
    copied: list[str] = []

    def interrupt_copy(source: Path, destination: Path) -> str:
        copied.append(str(source))
        if len(copied) == 3:
            message = 'simulated disk error after two staged files'
            raise OSError(message)
        return str(copy(source, destination))

    monkeypatch.setattr('art.pipeline5.atomic_delivery.shutil.copyfile', interrupt_copy)
    assert not process(request)
    assert len(copied) == 3
    assert not (request.output_root/request.look/'S').exists()
    assert not list(request.output_root.rglob('*staging*'))
    diagnostic = request.diagnostics_root/request.look/'S/try1'
    assert len(list(diagnostic.glob('*.png'))) == 5
    assert not json.loads((diagnostic/'sheet_checks.json').read_text())['passed']


def test_cached_identity_mismatch_is_a_new_audit_not_history_rewrite(tmp_path: Path) -> None:
    attempts = tmp_path/'attempts'
    attempts.mkdir()
    base = 'adult_M_hide_wrap_short_S_try1'
    raw = tmp_path/'original.png'
    fixture_sheet().save(raw)
    dispatch = attempts/(base+'.json')
    dispatch.write_text(json.dumps({'look': 'adult_M_hide_wrap_short', 'age': 'adult',
                                    'facing': 'S', 'attempt': 1, 'prompt': 'synthetic fixture'}))
    result = attempts/(base+'_result.json')
    result.write_text(json.dumps({'output_hint': f'output as {raw} by default'}))
    assert process_result(result, tmp_path)
    record = tmp_path/'processed-v4'/(base+'.json')
    prior_hash = digest(record)
    assert process_result(result, tmp_path)
    data = json.loads(record.read_text())
    targets = [result, dispatch, Path(data['diagnostics'])/'idle.json', Path(data['judge'])]
    for target in targets:
        original = target.read_bytes()
        content = json.loads(original)
        content['tampered'] = True
        target.write_text(json.dumps(content))
        # Unknown JSON fields on judge do not change its numeric meaning; change a bound field.
        if target == Path(data['judge']):
            content['family'] = 'environment'
            target.write_text(json.dumps(content))
        if target.name == 'idle.json':
            content['age'] = 'child'
            target.write_text(json.dumps(content))
        assert not process_result(result, tmp_path)
        assert digest(record) == prior_hash
        audits = sorted((tmp_path/'cache-audits-v4'/base).glob('*.json'))
        assert not json.loads(audits[-1].read_text())['passed']
        target.write_bytes(original)


def test_frozen_calibration_hash_is_verified(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    raw = tmp_path/'raw.png'
    fixture_sheet().save(raw)
    path = tmp_path/'judge.json'
    path.write_text(json.dumps(asdict(judge(raw, load_calibration(CALIBRATION), family='person'))))
    replacement = tmp_path/'calibration.json'
    replacement.write_bytes(CALIBRATION.read_bytes()+b'\n')
    monkeypatch.setattr('art.pipeline5.attempt_integrity.CALIBRATION', replacement)
    with pytest.raises(ValueError, match='calibration hash changed'):
        verify_judge(path, raw)


def test_complete_delivery_png_tampering_is_detected(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    from art.pipeline5.attempt_integrity import validate_bundle
    raw = tmp_path/'raw.png'
    fixture_sheet().save(raw)
    prompt = tmp_path/'prompt.txt'
    prompt.write_text('synthetic fixture')
    verdict = tmp_path/'judge.json'
    verdict.write_text(json.dumps({'numeric_pass': True}))
    request = SheetRequest(raw, 'adult_M_hide_wrap_short', 'adult', 'S', 1, prompt, verdict,
                           tmp_path/'out', tmp_path/'diagnostics')
    monkeypatch.setattr('art.pipeline5.sheet_slice.verify_judge', fixture_binding)
    assert process(request)
    target = request.output_root/request.look/'S'
    checks = request.diagnostics_root/request.look/'S/try1/sheet_checks.json'
    validate_bundle(target, checks)
    png = target/'idle.png'
    png.write_bytes(png.read_bytes()+b'tampered')
    with pytest.raises(ValueError, match='hash mismatch'):
        validate_bundle(target, checks)
