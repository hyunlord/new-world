# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy"]
# ///
# How to run: python -m art.batch_r1.portraits process GENERATION OUTPUT
"""Extract six complete portrait cells; promote only with hash-bound visual review."""
from __future__ import annotations

import json
import re
import shutil
import sys
import tempfile
from pathlib import Path
from typing import Final

from PIL import Image

from art.batch_r1.delivery import bound, check_hash, record, record_path, text_field
from art.batch_r1.metrics import Json
from art.batch_r1.process import REPOSITORY
from art.pipeline5.attempt_integrity import digest

FACE_PLAN: Final = REPOSITORY / 'art/character/FACE_VARIANTS.json'
PORTRAITS: Final = REPOSITORY / 'art/portrait/polish2'
ADOPTED: Final = REPOSITORY / 'art/portrait/batch_r1'
VARIANTS: Final = tuple(f'v{i}' for i in range(1, 7))


def provenance(generation: Path) -> dict[str, Json]:
    """Require literal prompt, body, shipped portrait and identity-plan provenance."""
    result = record(generation)
    look = text_field(result, 'look')
    if re.fullmatch(r'(adult|child)_[MF]_[a-z0-9_]+', look) is None:
        message = 'invalid portrait look name'
        raise ValueError(message)
    reference_look = text_field(result, 'reference_look')
    _ = text_field(result, 'prompt')
    attempt = result.get('attempt')
    if not isinstance(attempt, int) or isinstance(attempt, bool) or not 1 <= attempt <= 3:
        message = 'portrait attempts must be integers 1..3'
        raise ValueError(message)
    if generation.with_name('prompt.txt').read_text() != result['prompt']:
        message = 'portrait prompt differs from generation record'
        raise ValueError(message)
    _ = record_path(result, 'body_reference')
    plan_path = record_path(result, 'face_plan')
    if plan_path.resolve() != FACE_PLAN.resolve():
        message = 'identity plan must be the shipped FACE_VARIANTS.json'
        raise ValueError(message)
    plan = record(plan_path).get('looks')
    if not isinstance(plan, list):
        message = 'missing identity look list'
        raise TypeError(message)
    matching = [look for look in plan if isinstance(look, dict) and look.get('look') == reference_look]
    if len(matching) != 1 or result.get('variants') != matching[0].get('variants'):
        message = 'portrait identities differ from shipped face plan'
        raise ValueError(message)
    references = result.get('portrait_references')
    if not isinstance(references, list) or len(references) != 6:
        message = 'all six shipped portrait references are required'
        raise ValueError(message)
    for variant, reference in zip(VARIANTS, references, strict=True):
        if not isinstance(reference, dict):
            message = 'invalid portrait reference'
            raise TypeError(message)
        path = Path(text_field(reference, 'path'))
        if path.resolve() != (PORTRAITS / reference_look / (variant + '.png')).resolve():
            message = 'portrait reference order/path mismatch'
            raise ValueError(message)
        check_hash(path, reference.get('sha256'))
    check_hash(Path(text_field(result, 'raw')), result.get('raw_sha256'))
    return result


def process(generation: Path, output: Path) -> dict[str, Json]:
    """Resize each full 512px cell to128px, retaining opaque parchment pixels."""
    if output.resolve().is_relative_to(REPOSITORY):
        message = 'portrait evidence output must be outside repository'
        raise ValueError(message)
    source_record = provenance(generation)
    raw = Path(text_field(source_record, 'raw'))
    with Image.open(raw) as source:
        if source.size != (1536, 1024) or source.mode not in ('RGB', 'RGBA'):
            message = 'portrait sheet must be 1536x1024 RGB or opaque RGBA'
            raise ValueError(message)
        if source.mode == 'RGBA' and source.getchannel('A').getextrema() != (255, 255):
            message = 'portrait sheet must be fully opaque'
            raise ValueError(message)
        sheet = source.convert('RGB')
    output.mkdir(parents=True, exist_ok=False)
    frames: dict[str, Json] = {}
    for index, variant in enumerate(VARIANTS):
        x, y = index % 3 * 512, index // 3 * 512
        box = (x, y, x + 512, y + 512)
        path = output / (variant + '.png')
        sheet.crop(box).resize((128, 128), Image.Resampling.LANCZOS).save(path)
        sidecar: dict[str, Json] = {
            'variant': variant, 'image_sha256': digest(path), 'source': bound(raw),
            'source_box': list(box), 'anchor': [64, 64], 'footprint': None,
            'operations': ['full-cell RGB conversion', 'full-cell Lanczos resize to128x128'],
        }
        sidecar_path = path.with_suffix('.json')
        _ = sidecar_path.write_text(json.dumps(sidecar, indent=2) + '\n')
        frames[variant] = {'image_sha256': digest(path), 'sidecar_sha256': digest(sidecar_path)}
    summary: dict[str, Json] = {'look': source_record['look'], 'generation': bound(generation),
                              'raw': bound(raw), 'frames': frames}
    _ = (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    return summary


def adopt(trial: Path, visual: Path, destination: Path | None = None) -> dict[str, Json]:
    """Replay extraction and require an actual six-portrait review before delivery."""
    summary = record(trial / 'summary.json')
    generation = record_path(summary, 'generation')
    with tempfile.TemporaryDirectory(prefix='r1-portrait-replay-') as temporary:
        replay = Path(temporary) / 'portraits'
        if process(generation, replay) != summary:
            message = 'portrait summary differs from fresh extraction'
            raise ValueError(message)
        for variant in VARIANTS:
            for suffix in ('.png', '.json'):
                check_hash(trial / (variant + suffix), digest(replay / (variant + suffix)))
    review = record(visual)
    score = review.get('score')
    hashes: dict[str, Json] = {v: digest(trial / (v + '.png')) for v in VARIANTS}
    if (review.get('verdict') != 'pass' or isinstance(score, bool)
            or not isinstance(score, (int, float)) or not 90 <= score <= 100
            or review.get('portrait_sha256') != hashes):
        message = 'portrait visual review must pass at >=90 and bind all six image hashes'
        raise ValueError(message)
    manifest = dict(summary)
    manifest['visual_review'] = bound(visual)
    target = destination if destination is not None else ADOPTED / text_field(summary, 'look')
    if target.exists():
        message = f'portrait delivery exists: {target}'
        raise FileExistsError(message)
    target.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.r1-portrait-', dir=target.parent) as temporary:
        staged = Path(temporary) / 'portraits'
        staged.mkdir()
        for variant in VARIANTS:
            for suffix in ('.png', '.json'):
                _ = shutil.copyfile(trial / (variant + suffix), staged / (variant + suffix))
        _ = (staged / 'MANIFEST.json').write_text(json.dumps(manifest, indent=2) + '\n')
        _ = staged.rename(target)
    return manifest


def main() -> int:
    try:
        arguments = sys.argv[1:]
        if len(arguments) == 3 and arguments[0] == 'process':
            result = process(Path(arguments[1]), Path(arguments[2]))
        elif len(arguments) == 3 and arguments[0] == 'adopt':
            result = adopt(Path(arguments[1]), Path(arguments[2]))
        else:
            message = 'usage: process GENERATION OUTPUT | adopt TRIAL VISUAL_VERDICT'
            raise ValueError(message)
        print(json.dumps(result))
        return 0
    except (ValueError, TypeError, OSError, KeyError) as failure:
        print(json.dumps({'passed': False, 'error': str(failure)}))
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
