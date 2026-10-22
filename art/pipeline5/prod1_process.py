# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: python -m art.pipeline5.prod1_process DISPATCH.json --output-root OUTPUT --diagnostics-root EXTERNAL
"""Evaluate ten source panels and publish a complete, first-passing S/E look."""
from __future__ import annotations

import getopt
import json
import shutil
import sys
from dataclasses import asdict, replace
from pathlib import Path

from PIL import Image

from art.pipeline5.attempt_integrity import digest
from art.pipeline5.judge_panels import judge_panels
from art.pipeline5.prod1_records import (
    FACINGS,
    FrameRecord,
    ProductionRequest,
    SheetRecord,
    Sidecar,
    parse_dispatch,
)
from art.pipeline5.prod1_storage import (
    production_lock,
    publish,
    record_attempt,
    reserve,
)
from art.pipeline5.sheet_geometry import normalize_panel, split_panels
from art.pipeline5.whole_checks import FRAMES, WholeResult, check_whole


def failed_frames(reason: str) -> tuple[FrameRecord, ...]:
    """A sheet-level failure explicitly accounts for all ten undeliverable frames."""
    checks = WholeResult(False, (reason,), 0, 0, 0, False)
    return tuple(FrameRecord(facing, frame, False, checks, None, reason)
                 for facing in FACINGS for frame in FRAMES)


def evaluate(request: ProductionRequest) -> SheetRecord:
    """Keep normalization failures alongside numeric failures at panel granularity."""
    raw_hash = digest(request.raw)
    prompt_hash = digest(request.prompt_path)
    _ = shutil.copyfile(request.raw, request.diagnostic / 'raw.png')
    _ = shutil.copyfile(request.prompt_path, request.diagnostic / 'prompt.txt')
    if (digest(request.diagnostic / 'raw.png') != raw_hash
            or digest(request.diagnostic / 'prompt.txt') != prompt_hash):
        message = 'source_identity: copied raw or prompt differs from input digest'
        raise OSError(message)
    result = SheetRecord(request.look, request.age, request.attempt, str(request.raw),
                         raw_hash, prompt_hash, str(request.diagnostic), False, 0, (),
                         review_reason=request.review_reason)
    try:
        with Image.open(request.diagnostic / 'raw.png') as source:
            if source.mode != 'RGBA' or source.size != (1536, 1024):
                message = f'raw_sheet: expected 1536x1024 RGBA, got {source.size} {source.mode}'
                raise ValueError(message)
            panels = split_panels(source, rows=2)
        scale = (40 if request.age == 'child' else 56) / panels.idle_height
        judges = judge_panels(request.diagnostic / 'raw.png', panels.images, request.diagnostic / 'judge', expected_count=10)
        ownership = {'idle_source_height': panels.idle_height, 'scale': scale,
                     'source_boxes': panels.boxes, 'ownership': asdict(panels.ownership)}
        _ = (request.diagnostic / 'ownership.json').write_text(json.dumps(ownership, indent=2) + '\n')
        records: list[FrameRecord] = []
        for index, (panel, box) in enumerate(zip(panels.images, panels.boxes, strict=True)):
            facing, frame = FACINGS[index // 5], FRAMES[index % 5]
            directory = request.diagnostic / facing
            directory.mkdir(exist_ok=True)
            verdict = judges.panels[index].verdict
            try:
                normalized = normalize_panel(panel, scale, frame == 'sleep')
                checks = check_whole(normalized.image, request.age, frame)
                path = directory / (frame + '.png')
                normalized.image.save(path)
                metadata = Sidecar(request.look, request.age, facing, frame, request.attempt,
                                   raw_hash, prompt_hash, digest(path), str(request.diagnostic / 'raw.png'),
                                   str(request.diagnostic), index, judges.calibration_sha256, scale,
                                   normalized.transform.rotation_degrees, box,
                                   normalized.transform, checks, verdict)
                _ = path.with_suffix('.json').write_text(json.dumps(asdict(metadata), indent=2) + '\n')
                records.append(FrameRecord(facing, frame, checks.passed and verdict.numeric_pass,
                                           checks, verdict))
            except ValueError as error:
                reason = str(error)
                panel.save(directory / (frame + '_rejected_source.png'))
                checks = WholeResult(False, (reason,), 0, 0, 0, False)
                records.append(FrameRecord(facing, frame, False, checks, verdict, reason))
        passing = sum(frame.passed for frame in records)
        result = replace(result, passed=passing == 10 and judges.numeric_pass,
                         passing_panels=passing, frames=tuple(records), scale=scale)
    except (ValueError, OSError) as error:
        result = replace(result, error=str(error), frames=failed_frames(str(error)))
    return result


def process(request: ProductionRequest) -> bool:
    """Serialize attempts so generation caps and immutable first-pass stopping hold."""
    with production_lock(request.diagnostics_root):
        reserve(request)
        try:
            result = evaluate(request)
        except OSError as error:
            result = SheetRecord(request.look, request.age, request.attempt, str(request.raw),
                                 '', '', str(request.diagnostic), False, 0,
                                 failed_frames(str(error)), review_reason=request.review_reason, error=str(error))
        if result.passed:
            try:
                publish(request, result)
            except (ValueError, OSError) as error:
                result = replace(result, passed=False, error=f'publication: {error}')
        record_attempt(result, request.diagnostics_root)
        print(json.dumps({'passed': result.passed, 'passing_panels': result.passing_panels,
                          'diagnostics': result.diagnostics, 'error': result.error}))
        return result.passed


def main() -> int:
    """Consume a dispatch JSON plus explicit public and external evidence roots."""
    try:
        if len(sys.argv) < 2:
            message = 'usage: DISPATCH.json --output-root OUTPUT --diagnostics-root EXTERNAL'
            raise ValueError(message)
        options, rest = getopt.getopt(sys.argv[2:], '', ['output-root=', 'diagnostics-root=', 'review-retry'])
        values = dict(options)
        review_retry = values.pop('--review-retry', None) is not None
        if rest or set(values) != {'--output-root', '--diagnostics-root'}:
            message = 'require --output-root and --diagnostics-root'
            raise ValueError(message)
        roots = (Path(values['--output-root']), Path(values['--diagnostics-root']))
        request = parse_dispatch(Path(sys.argv[1]), roots, review_retry)
        return 0 if process(request) else 1
    except (ValueError, OSError, getopt.GetoptError) as error:
        print(json.dumps({'passed': False, 'error': str(error)}))
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
