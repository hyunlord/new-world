# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: imported by python -m art.pipeline5.prod1_process
"""Immutable attempt storage and atomic paired-facing delivery."""
from __future__ import annotations

import fcntl
import json
import shutil
import tempfile
from collections.abc import Generator
from contextlib import contextmanager
from dataclasses import asdict
from pathlib import Path

from art.pipeline5.attempt_integrity import digest
from art.pipeline5.judge_panels import CALIBRATION_SHA256
from art.pipeline5.prod1_exchange import exchange
from art.pipeline5.prod1_records import (
    FACINGS,
    AttemptIdentity,
    ProductionRequest,
    SheetRecord,
    SidecarIdentity,
)
from art.pipeline5.whole_checks import FRAMES, check_whole_path


@contextmanager
def production_lock(root: Path) -> Generator[None]:
    """Serialize cap checks and publication across CLI processes."""
    root.mkdir(parents=True, exist_ok=True)
    with (root / '.production.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(lock, fcntl.LOCK_UN)


def reserve(request: ProductionRequest) -> None:
    """Refuse skipped/reused tries, >108 sheets, and post-acceptance retries."""
    target = request.output_root / request.look
    if target.exists() and request.review_reason is None:
        message = f'first_pass_stop: accepted look exists: {target}'
        raise ValueError(message)
    previous = sorted((request.diagnostics_root / request.look).glob('try*'))
    expected = [f'try{number}' for number in range(1, request.attempt)]
    if [path.name for path in previous] != expected:
        message = f'attempt_sequence: expected previous {expected}'
        raise ValueError(message)
    for path in previous:
        record: AttemptIdentity = json.loads((path / 'sheet_checks.json').read_text())
        if record.get('review_reason') is not None and request.review_reason is not None:
            message = 'review_round_limit: one targeted post-review attempt per look'
            raise ValueError(message)
        if record['passed'] and request.review_reason is None:
            message = f'first_pass_stop: successful earlier attempt {path}'
            raise ValueError(message)
    if len(list(request.diagnostics_root.glob('*/try*'))) >= 108:
        message = 'generation_limit: already recorded 108 sheets'
        raise ValueError(message)
    request.diagnostic.mkdir(parents=True, exist_ok=False)


def record_attempt(record: SheetRecord, root: Path) -> None:
    """Keep every result and rebuild the generation ledger without dropping rejects."""
    path = Path(record.diagnostics) / 'sheet_checks.json'
    with path.open('x') as stream:
        _ = stream.write(json.dumps(asdict(record), indent=2) + '\n')
    records: list[AttemptIdentity] = [json.loads(item.read_text()) for item in sorted(root.glob('*/try*/sheet_checks.json'))]
    best: dict[str, int] = {}
    for entry in records:
        if not entry['passed']:
            look = entry['look']
            prior = next((item for item in records if item['look'] == look
                          and item['attempt'] == best.get(look)), None)
            if prior is None or entry['passing_panels'] > prior['passing_panels']:
                best[look] = entry['attempt']
    payload = {'attempts': records, 'best_rejected_attempt': best,
               'processed_attempt_count': len(records), 'processed_attempt_limit': 108,
               'actual_generation_calls': 'tracked separately by the generation driver'}
    temporary = root / '.PROD1_GENERATION_LOG.tmp'
    _ = temporary.write_text(json.dumps(payload, indent=2) + '\n')
    _ = temporary.replace(root / 'PROD1_GENERATION_LOG.json')


def publish(request: ProductionRequest, record: SheetRecord) -> None:
    """Validate copied bytes, then expose both facings in one directory rename."""
    target = request.output_root / request.look
    target.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix='.' + request.look + '-', dir=target.parent))
    try:
        hashes: dict[str, str] = {}
        for facing in FACINGS:
            (stage / facing).mkdir()
            for frame in FRAMES:
                for suffix in ('.png', '.json'):
                    relative = Path(facing) / (frame + suffix)
                    _ = shutil.copyfile(request.diagnostic / relative, stage / relative)
                    hashes[str(relative)] = digest(stage / relative)
                image = stage / facing / (frame + '.png')
                data: SidecarIdentity = json.loads(image.with_suffix('.json').read_text())
                if (data['image_sha256'] != digest(image)
                        or not data['checks']['passed'] or not data['judge']['numeric_pass']
                        or not check_whole_path(image, request.age, frame).passed):
                    message = f'delivery_identity: rejected staged frame {facing}/{frame}'
                    raise ValueError(message)
        manifest = {'look': request.look, 'age': request.age, 'sex': request.look.split('_')[1],
                    'top': next(top for top in ('loincloth', 'hide_wrap', 'fur_cloak')
                                if ('_' + top + '_') in request.look),
                    'hair': next(hair for hair in ('short', 'long_tied', 'bun')
                                 if request.look.endswith('_' + hair)),
                    'raw': str(request.diagnostic / 'raw.png'),
                    'diagnostics': str(request.diagnostic), 'sheet_try': request.attempt,
                    'calibration_sha256': CALIBRATION_SHA256,
                    'attempt': request.attempt, 'scale': record.scale,
                    'standing_scale': record.scale,
                    'sleep_scales': {facing: json.loads(
                        (stage / facing / 'sleep.json').read_text()).get('sleep_scale', record.scale)
                        for facing in FACINGS},
                    'raw_sha256': record.raw_sha256, 'prompt_sha256': record.prompt_sha256,
                    'rotations': {f'{facing}/{frame}': json.loads(
                        (stage / facing / (frame + '.json')).read_text())['rotation']
                        for facing in FACINGS for frame in FRAMES},
                    'files': hashes, 'frames': [asdict(frame) for frame in record.frames]}
        _ = (stage / 'MANIFEST.json').write_text(json.dumps(manifest, indent=2) + '\n')
        if target.exists():
            if request.review_reason is None:
                message = f'delivery exists; preserve it: {target}'
                raise ValueError(message)
            _ = shutil.copytree(target, request.diagnostic / 'previous_accepted')
            exchange(stage, target)
        else:
            _ = stage.rename(target)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
