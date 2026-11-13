# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "pytest"]
# ///
"""Delivery rejects stale or missing evidence and never promotes partial sheets."""
import json
from dataclasses import asdict
from pathlib import Path
from typing import Literal, assert_never

import pytest
from PIL import Image, ImageDraw

from art.batch_r1 import delivery
from art.batch_r1.metrics import FrameRegions, compare_frames, write_regions
from art.batch_r1.process import FACINGS, Request, process
from art.batch_r1.test_process import fixture
from art.pipeline5.attempt_integrity import digest
from art.pipeline5.whole_checks import FRAMES


def prepared(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> tuple[Path, Path, Path, Path]:
    request = fixture(tmp_path)
    with Image.open(request.raw) as sheet:
        sheet.paste((0, 0, 0, 0), (0, 0, 1536, 1024))
        draw = ImageDraw.Draw(sheet)
        for row in range(2):
            for column in range(5):
                x, y = column * 307 + 110, row * 512 + 100
                draw.rectangle((x, y, x + 29, y + 99), fill=(180, 100, 40, 255),
                               outline=(30, 30, 30, 255), width=4)
        sheet.save(request.raw)
    assert process(request).structural_pass
    reference = tmp_path / 'shipped' / 'adult_M_hide_wrap_short'
    for facing in FACINGS:
        (reference / facing).mkdir(parents=True)
        for frame in FRAMES:
            source = request.output / facing / (frame + '.png')
            _ = (reference / facing / (frame + '.png')).write_bytes(source.read_bytes())
    _ = (reference / 'MANIFEST.json').write_text(json.dumps({
        'raw_sha256': digest(request.raw),
        'files': {f'{f}/{s}.png': digest(reference / f / (s + '.png'))
                  for f in FACINGS for s in FRAMES}}))
    trial = tmp_path / 'trial'
    assert process(Request(request.raw, request.look, trial, reference)).structural_pass
    regions = tmp_path / 'regions'
    regions.mkdir()
    annotation = FrameRegions((70,), (80,), frozenset({(40, 80)}),
                              (tuple((x, 80) for x in range(29, 40)),), 110)
    write_regions(regions / (reference.name + '.json'), annotation)
    monkeypatch.setattr(delivery, 'SHIPPED', reference.parent)
    monkeypatch.setattr(delivery, 'REGIONS', regions)
    with Image.open(reference / 'S/idle.png') as baseline, Image.open(trial / 'S/idle.png') as candidate:
        comparison = asdict(compare_frames(baseline, candidate, annotation))
    _ = (trial / 'measurements.json').write_text(json.dumps(comparison))
    generation = tmp_path / 'generation.json'
    _ = generation.write_text(json.dumps({'look': request.look, 'reference_look': reference.name,
                                     'raw_sha256': digest(request.raw), 'prompt': 'Keep the rig.',
                                     'reference': str(request.raw), 'reference_sha256': digest(request.raw)}))
    visual = tmp_path / 'visual.json'
    _ = visual.write_text(json.dumps({'verdict': 'pass', 'score': 92,
                                 'frame_sha256': {f'{f}/{s}': digest(trial / f / (s + '.png'))
                                                  for f in FACINGS for s in FRAMES}}))
    return trial, generation, reference, visual


def test_adopt_and_verify_complete_sheet(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    inputs = prepared(tmp_path, monkeypatch)
    target = tmp_path / 'delivery'
    manifest = delivery.adopt(*inputs, destination=target)
    frames = manifest['frames']
    assert isinstance(frames, dict)
    assert len(frames) == 10
    assert len(list(target.rglob('*.png'))) == 10
    assert delivery.verify(target) == manifest
    _ = (target / 'S/idle.png').write_bytes(b'tampered')
    with pytest.raises(ValueError, match='hash mismatch'):
        _ = delivery.verify(target)


@pytest.mark.parametrize('tamper', ['metric', 'png', 'raw', 'visual', 'missing', 'partial', 'prompt'])
def test_reject_bad_evidence_before_delivery(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, tamper: Literal['metric', 'png', 'raw', 'visual', 'missing', 'partial', 'prompt']) -> None:
    trial, generation, reference, visual = prepared(tmp_path, monkeypatch)
    match tamper:
        case 'metric':
            _ = (trial / 'measurements.json').write_text('{"passed":true}')
        case 'png':
            _ = (trial / 'S/idle.png').write_bytes((trial / 'E/idle.png').read_bytes() + b'tampered')
        case 'raw':
            _ = (tmp_path / 'raw.png').write_bytes(b'tampered')
        case 'visual':
            _ = visual.write_text('{"verdict":"pass","score":100,"frame_sha256":{}}')
        case 'missing':
            visual.unlink()
        case 'partial':
            (trial / 'E/sleep.png').unlink()
        case 'prompt':
            _ = (tmp_path / 'prompt.txt').write_text('Different prompt.')
        case _:
            assert_never(tamper)
    target = tmp_path / 'delivery'
    with pytest.raises((ValueError, OSError)):
        _ = delivery.adopt(trial, generation, reference, visual, target)
    assert not target.exists()


def test_flat_generation_layout_and_fresh_numeric_rejection(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    trial, generation, reference, visual = prepared(tmp_path, monkeypatch)
    generated = delivery.record(generation)
    generated['raw'] = str(tmp_path / 'raw.png')
    generated['reference_raw'] = generated.pop('reference')
    generated['reference_raw_sha256'] = generated.pop('reference_sha256')
    flat = tmp_path / 'look_try1.generation.json'
    _ = flat.write_text(json.dumps(generated))
    assert delivery.inspect(trial, flat, reference, visual)['look'] == generated['look']
    with Image.open(trial / 'S/idle.png') as image:
        ImageDraw.Draw(image).point((40, 80), fill=(255, 255, 255, 255))
        image.save(trial / 'S/idle.png')
    with pytest.raises(ValueError, match='numeric measurements failed'):
        _ = delivery.inspect(trial, flat, reference, visual)
