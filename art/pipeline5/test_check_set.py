# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image", "pytest"]
# ///
# How to run: python -m pytest art/pipeline5/test_check_set.py
"""Non-vacuous population and deterministic sampling contract."""
from pathlib import Path

from art.pipeline5 import check_set


def test_empty_delivery_fails(tmp_path: Path) -> None:
    result = check_set.check_set(tmp_path)
    assert not result.passed
    assert result.sampled == ()
    assert result.accepted_looks == 0
    assert 'need 48 distinct look/facing identities; found 0' in result.findings


def test_sample_is_balanced_repeatable_and_unique() -> None:
    identities = tuple(f'{age}_{sex}_hide_wrap_hair{index}/{facing}'
                       for age in ('adult', 'child') for sex in ('F', 'M')
                       for index in range(9) for facing in ('S', 'E'))
    result = check_set.sample_identities(identities)
    assert len(result) == len(set(result)) == 48
    assert result == check_set.sample_identities(tuple(reversed(identities)))
    assert all(sum(value.startswith(f'{age}_{sex}_') for value in result) == 12
               for age in ('adult', 'child') for sex in ('F', 'M'))


def test_missing_facing_is_reported(tmp_path: Path) -> None:
    (tmp_path / 'adult_M_hide_wrap_short' / 'S').mkdir(parents=True)
    result = check_set.check_set(tmp_path)
    assert not result.passed
    assert any('missing' in reason for reason in result.findings)


def test_fresh_frame_verification_rejects_metadata_and_pixel_tampering(tmp_path: Path) -> None:
    import json
    from dataclasses import asdict

    import pytest
    from PIL import Image, ImageDraw

    from art.pipeline5.attempt_integrity import digest
    from art.pipeline5.judge_panels import CALIBRATION_SHA256, ROOT, judge_panels
    from art.pipeline5.whole_checks import check_whole

    source = ROOT / 'art/style05/anchors/crops/person_full.png'
    with Image.open(source) as original:
        fresh = judge_panels(source, (original.convert('RGBA'),), tmp_path / 'judge', expected_count=1)
    image = Image.new('RGBA', (80, 112))
    ImageDraw.Draw(image).rectangle((25, 49, 54, 104), fill=(70, 40, 20, 255))
    path = tmp_path / 'idle.png'
    image.save(path)
    identity = ('adult_M_hide_wrap_short', 'S', 'idle', 0)
    data = {'look': identity[0], 'age': 'adult', 'facing': 'S', 'frame': 'idle',
            'panel_index': 0, 'raw': str(source), 'raw_sha256': fresh.source_sha256,
            'image_sha256': digest(path), 'calibration_sha256': CALIBRATION_SHA256,
            'checks': asdict(check_whole(image, 'adult', 'idle')),
            'judge': asdict(fresh.panels[0].verdict)}
    _ = path.with_suffix('.json').write_text(json.dumps(data))
    assert check_set.audit_frame(path, identity, fresh)['image_sha256'] == digest(path)
    for key, value in [('panel_index', 1), ('raw_sha256', 'f' * 64), ('image_sha256', '0' * 64)]:
        _ = path.with_suffix('.json').write_text(json.dumps(data | {key: value}))
        with pytest.raises(ValueError, match='mismatch'):
            _ = check_set.audit_frame(path, identity, fresh)
    _ = path.with_suffix('.json').write_text(json.dumps(data))
    image.putpixel((1, 1), (0, 0, 0, 255))
    image.save(path)
    with pytest.raises(ValueError, match='hash'):
        _ = check_set.audit_frame(path, identity, fresh)


def test_missing_required_manifest_fails(tmp_path: Path) -> None:
    import pytest

    metadata: check_set.Metadata = {
        'look': 'adult_M_hide_wrap_short', 'age': 'adult', 'facing': 'S',
        'frame': 'idle', 'attempt': 1, 'raw': 'unused.png', 'raw_sha256': '',
        'prompt_sha256': '', 'image_sha256': '', 'scale': 1.0,
        'panel_index': 0, 'rotation': 0, 'source_box': [0, 0, 1, 1],
        'calibration_sha256': '', 'checks': {},
        'judge': {'candidate': '', 'sha256': '', 'family': 'person',
                  'metrics': {}, 'numeric_checks': {}, 'numeric_pass': False},
    }
    with pytest.raises(ValueError, match='missing required MANIFEST'):
        check_set.audit_manifest(tmp_path, metadata)
