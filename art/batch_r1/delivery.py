# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy"]
# ///
# How to run: python -m art.batch_r1.delivery adopt TRIAL GENERATION REFERENCE VISUAL
"""Promote complete R1 sheets only after fresh numeric, structural and review gates.

Visual JSON: {"verdict":"pass", "score":90, "frame_sha256":{"S/idle":...}}
with exactly all ten facing/frame keys. Measurements live in trial/measurements.json.
"""
from __future__ import annotations

import json
import shutil
import sys
import tempfile
from collections.abc import Callable
from dataclasses import asdict
from hashlib import sha256
from pathlib import Path
from typing import Final

from PIL import Image

from art.batch_r1.metrics import Json, compare_frames, read_regions
from art.batch_r1.process import FACINGS, REPOSITORY, Request, process
from art.pipeline5.attempt_integrity import digest
from art.pipeline5.whole_checks import FRAMES, check_whole

REGIONS: Final = Path(__file__).with_name('regions')
SHIPPED: Final = REPOSITORY / 'art/character/route_i'
ADOPTED: Final = REPOSITORY / 'art/character/batch_r1'
_decode: Callable[[str], Json] = json.loads


def record(path: Path) -> dict[str, Json]:
    """Require a JSON object, rather than accepting absent evidence."""
    value = _decode(path.read_text())
    if not isinstance(value, dict):
        message = f'expected JSON object: {path}'
        raise TypeError(message)
    return value


def text_field(value: dict[str, Json], key: str) -> str:
    result = value.get(key)
    if not isinstance(result, str) or not result:
        message = f'missing string evidence: {key}'
        raise ValueError(message)
    return result


def bound(path: Path) -> dict[str, Json]:
    return {'path': str(path.resolve()), 'sha256': digest(path)}


def check_hash(path: Path, expected: Json) -> None:
    if digest(path) != expected:
        message = f'hash mismatch: {path}'
        raise ValueError(message)


def inspect(trial: Path, generation: Path, reference: Path, visual: Path) -> dict[str, Json]:
    """Recompute acceptance; stored booleans alone never authorize promotion."""
    summary, generated = record(trial / 'summary.json'), record(generation)
    look = text_field(summary, 'look')
    raw = Path(text_field(summary, 'raw'))
    _ = Request(raw, look, trial, reference)
    if reference.resolve() != (SHIPPED / reference.name).resolve():
        message = 'reference must be a directly shipped route_i look'
        raise ValueError(message)
    if Path(text_field(summary, 'reference')).resolve() != reference.resolve():
        message = 'summary reference mismatch'
        raise ValueError(message)
    if generated.get('look') != look or generated.get('reference_look') != reference.name:
        message = 'generation look/reference mismatch'
        raise ValueError(message)
    for expected in (summary.get('raw_sha256'), generated.get('raw_sha256')):
        check_hash(raw, expected)
    if 'raw' in generated and Path(text_field(generated, 'raw')).resolve() != raw.resolve():
        message = 'generation raw path mismatch'
        raise ValueError(message)
    prompt = text_field(generated, 'prompt')
    prompt_file = generation.with_name('prompt.txt')
    if prompt_file.exists() and prompt_file.read_text().strip() != prompt.strip():
        message = 'prompt file differs from generation record'
        raise ValueError(message)
    reference_raw = Path(text_field(generated, 'reference_raw' if 'reference_raw' in generated else 'reference'))
    reference_hash = generated.get('reference_raw_sha256', generated.get('reference_sha256'))
    check_hash(reference_raw, reference_hash)
    shipped_manifest = record(reference / 'MANIFEST.json')
    check_hash(reference_raw, shipped_manifest.get('raw_sha256'))
    shipped_files = shipped_manifest.get('files')
    if not isinstance(shipped_files, dict):
        message = 'shipped manifest lacks frame hashes'
        raise TypeError(message)
    if 'reference_manifest' in generated:
        check_hash(Path(text_field(generated, 'reference_manifest')), generated.get('reference_manifest_sha256'))
    mask = REGIONS / (reference.name + '.json')
    with Image.open(reference / 'S/idle.png') as baseline, Image.open(trial / 'S/idle.png') as candidate:
        comparison = asdict(compare_frames(baseline, candidate, read_regions(mask)))
    if not comparison['passed'] or record(trial / 'measurements.json') != comparison:
        message = 'numeric measurements failed or stored evidence differs from fresh comparison'
        raise ValueError(message)
    frames: dict[str, Json] = {}
    hashes: dict[str, Json] = {}
    with tempfile.TemporaryDirectory(prefix='r1-replay-') as temporary:
        replay = Path(temporary) / 'frames'
        replayed = process(Request(raw, look, replay, reference))
        if not replayed.structural_pass:
            message = 'raw replay failed structural checks'
            raise ValueError(message)
        if summary != record(replay / 'summary.json'):
            message = 'summary differs from fresh raw replay'
            raise ValueError(message)
        for facing in FACINGS:
            for frame in FRAMES:
                key = f'{facing}/{frame}'
                image_path = trial / (key + '.png')
                sidecar = image_path.with_suffix('.json')
                check_hash(image_path, digest(replay / (key + '.png')))
                evidence = record(sidecar)
                if evidence != record(replay / (key + '.json')):
                    message = f'frame sidecar differs from fresh replay: {key}'
                    raise ValueError(message)
                check_hash(image_path, evidence.get('image_sha256'))
                check_hash(reference / (key + '.png'), evidence.get('reference_sha256'))
                check_hash(reference / (key + '.png'), shipped_files.get(key + '.png'))
                if evidence.get('raw_sha256') != digest(raw):
                    message = f'frame raw mismatch: {key}'
                    raise ValueError(message)
                with Image.open(image_path) as image:
                    checks = check_whole(image, look.split('_')[0], frame)
                if not checks.passed:
                    message = f'structural rejection: {key}: {checks.findings}'
                    raise ValueError(message)
                hashes[key] = digest(image_path)
                frames[key] = {'image_sha256': digest(image_path), 'sidecar_sha256': digest(sidecar),
                               'reference_sha256': digest(reference / (key + '.png')),
                               'anchor': [56, 72] if frame == 'sleep' else [40, 104],
                               'canvas': [112, 80] if frame == 'sleep' else [80, 112],
                               'footprint': [1, 1], 'checks': json.loads(json.dumps(asdict(checks)))}
    review = record(visual)
    score = review.get('score')
    if (review.get('verdict') != 'pass' or isinstance(score, bool)
            or not isinstance(score, (int, float)) or not 90 <= score <= 100
            or review.get('frame_sha256') != hashes):
        message = 'visual verdict must pass at score >=90 and bind exactly all ten frame hashes'
        raise ValueError(message)
    return {'look': look, 'trial': str(trial.resolve()), 'generation': bound(generation),
            'source': bound(raw), 'prompt_sha256': sha256(prompt.encode()).hexdigest(),
            'reference_raw': bound(reference_raw), 'reference': str(reference.resolve()),
            'reference_manifest': bound(reference / 'MANIFEST.json'),
            'regions': bound(mask), 'visual_review': bound(visual),
            'measurements': json.loads(json.dumps(comparison)),
            'measurement_record': bound(trial / 'measurements.json'), 'frames': frames}


def adopt(trial: Path, generation: Path, reference: Path, visual_verdict: Path,
          destination: Path | None = None) -> dict[str, Json]:
    """Validate everything before creating a complete, non-overwriting delivery."""
    manifest = inspect(trial, generation, reference, visual_verdict)
    target = destination if destination is not None else ADOPTED / text_field(manifest, 'look')
    if target.exists():
        message = f'delivery already exists: {target}'
        raise FileExistsError(message)
    target.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.r1-delivery-', dir=target.parent) as temporary:
        staged = Path(temporary) / 'look'
        for facing in FACINGS:
            (staged / facing).mkdir(parents=True)
            for frame in FRAMES:
                for suffix in ('.png', '.json'):
                    _ = shutil.copyfile(trial / facing / (frame + suffix), staged / facing / (frame + suffix))
        _ = (staged / 'MANIFEST.json').write_text(json.dumps(manifest, indent=2) + '\n')
        _ = staged.rename(target)
    return manifest


def verify(destination: Path) -> dict[str, Json]:
    """Revalidate external provenance and compare every delivered file hash."""
    manifest = record(destination / 'MANIFEST.json')
    generation = record_path(manifest, 'generation')
    visual = record_path(manifest, 'visual_review')
    fresh = inspect(Path(text_field(manifest, 'trial')), generation,
                    Path(text_field(manifest, 'reference')), visual)
    if fresh != manifest:
        message = 'delivery manifest differs from freshly verified provenance'
        raise ValueError(message)
    frames = manifest['frames']
    if not isinstance(frames, dict):
        message = 'missing frames'
        raise TypeError(message)
    for key, frame in frames.items():
        if not isinstance(frame, dict):
            message = 'invalid frame record'
            raise TypeError(message)
        check_hash(destination / (key + '.png'), frame['image_sha256'])
        check_hash(destination / (key + '.json'), frame['sidecar_sha256'])
    return manifest


def record_path(manifest: dict[str, Json], key: str) -> Path:
    value = manifest.get(key)
    if not isinstance(value, dict):
        message = f'missing bound record: {key}'
        raise TypeError(message)
    path = Path(text_field(value, 'path'))
    check_hash(path, value.get('sha256'))
    return path


def main() -> int:
    try:
        arguments = sys.argv[1:]
        if len(arguments) == 5 and arguments[0] == 'adopt':
            result = adopt(*(Path(value) for value in arguments[1:]))
        elif len(arguments) == 2 and arguments[0] == 'verify':
            result = verify(Path(arguments[1]))
        else:
            message = 'usage: adopt TRIAL GENERATION SHIPPED_LOOK VISUAL_VERDICT | verify ADOPTED_LOOK'
            raise ValueError(message)
        print(json.dumps(result))
        return 0
    except (ValueError, TypeError, OSError, KeyError) as failure:
        print(json.dumps({'passed': False, 'error': str(failure)}))
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
