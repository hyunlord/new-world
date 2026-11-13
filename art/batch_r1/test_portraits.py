# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "pytest"]
# ///
"""Full-cell portrait extraction and fail-closed promotion evidence."""
import json
from pathlib import Path
from typing import Literal, assert_never

import pytest
from PIL import Image, ImageDraw

from art.batch_r1 import portraits
from art.batch_r1.delivery import bound, record
from art.pipeline5.attempt_integrity import digest


def prepared(tmp_path: Path) -> tuple[Path, Path, Path]:
    raw = tmp_path / 'raw.png'
    sheet = Image.new('RGB', (1536, 1024))
    draw = ImageDraw.Draw(sheet)
    for index in range(6):
        x, y = index % 3 * 512, index // 3 * 512
        draw.rectangle((x, y, x + 511, y + 511), fill=(40 + index * 20, 80, 120))
    sheet.save(raw)
    body = tmp_path / 'body.png'
    Image.new('RGBA', (80, 112)).save(body)
    plan = record(portraits.FACE_PLAN)['looks']
    assert isinstance(plan, list)
    matching = next(look for look in plan if isinstance(look, dict)
                    and look['look'] == 'adult_M_hide_wrap_short')
    assert isinstance(matching, dict)
    prompt = 'Preserve all six exact identities; change only the plant fibre wrap.'
    generation = tmp_path / 'generation.json'
    _ = generation.with_name('prompt.txt').write_text(prompt)
    _ = generation.write_text(json.dumps({
        'look': 'adult_M_neo_plant_short', 'reference_look': matching['look'], 'attempt': 1,
        'seed': None, 'prompt': prompt, 'body_reference': bound(body),
        'face_plan': bound(portraits.FACE_PLAN), 'variants': matching['variants'],
        'portrait_references': [bound(portraits.PORTRAITS / 'adult_M_hide_wrap_short' / (v + '.png'))
                                for v in portraits.VARIANTS],
        'raw': str(raw), 'raw_sha256': digest(raw)}))
    trial = tmp_path / 'processed'
    _ = portraits.process(generation, trial)
    visual = tmp_path / 'visual.json'
    _ = visual.write_text(json.dumps({'verdict': 'pass', 'score': 95,
                                     'portrait_sha256': {v: digest(trial / (v + '.png'))
                                                        for v in portraits.VARIANTS}}))
    return generation, trial, visual


def test_full_cells_and_complete_adoption(tmp_path: Path) -> None:
    _, trial, visual = prepared(tmp_path)
    for index, variant in enumerate(portraits.VARIANTS):
        with Image.open(trial / (variant + '.png')) as image:
            assert image.mode == 'RGB'
            assert image.size == (128, 128)
            assert image.getpixel((0, 0)) == (40 + index * 20, 80, 120)
            assert image.getpixel((127, 127)) == (40 + index * 20, 80, 120)
        sidecar = record(trial / (variant + '.json'))
        assert sidecar['anchor'] == [64, 64]
        assert sidecar['footprint'] is None
    target = tmp_path / 'delivery'
    result = portraits.adopt(trial, visual, target)
    assert result['look'] == 'adult_M_neo_plant_short'
    assert len(list(target.glob('*.png'))) == 6


@pytest.mark.parametrize('tamper', ['raw', 'body', 'prompt', 'identity', 'portrait', 'missing', 'review'])
def test_bad_provenance_or_review_never_promotes(
    tmp_path: Path, tamper: Literal['raw', 'body', 'prompt', 'identity', 'portrait', 'missing', 'review'],
) -> None:
    generation, trial, visual = prepared(tmp_path)
    match tamper:
        case 'raw':
            _ = (tmp_path / 'raw.png').write_bytes(b'tampered')
        case 'body':
            _ = (tmp_path / 'body.png').write_bytes(b'tampered')
        case 'prompt':
            _ = (tmp_path / 'prompt.txt').write_text('different prompt')
        case 'identity':
            generated = record(generation)
            generated['variants'] = []
            _ = generation.write_text(json.dumps(generated))
        case 'portrait':
            _ = (trial / 'v6.png').write_bytes(b'tampered')
        case 'missing':
            visual.unlink()
        case 'review':
            _ = visual.write_text('{"verdict":"pass","score":100,"portrait_sha256":{}}')
        case _:
            assert_never(tamper)
    target = tmp_path / 'delivery'
    with pytest.raises((ValueError, OSError)):
        _ = portraits.adopt(trial, visual, target)
    assert not target.exists()


def test_transparent_portrait_sheet_rejected(tmp_path: Path) -> None:
    generation, _, _ = prepared(tmp_path)
    raw = tmp_path / 'raw.png'
    Image.new('RGBA', (1536, 1024), (90, 80, 70, 254)).save(raw)
    generated = record(generation)
    generated['raw_sha256'] = digest(raw)
    _ = generation.write_text(json.dumps(generated))
    with pytest.raises(ValueError, match='fully opaque'):
        _ = portraits.process(generation, tmp_path / 'other')
