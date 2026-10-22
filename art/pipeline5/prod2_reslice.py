# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: python -m art.pipeline5.prod2_reslice SOURCE DATA_ROOT OUTPUT_ROOT REPORT_JSON
"""Re-evaluate frozen PROD1 sources twice without spending generation attempts."""
from __future__ import annotations

import hashlib
import json
import shutil
import sys
from concurrent.futures import ProcessPoolExecutor
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import TypedDict

from art.pipeline5.attempt_integrity import digest
from art.pipeline5.prod1_process import evaluate
from art.pipeline5.prod1_records import ProductionRequest, SheetRecord
from art.pipeline5.prod1_storage import publish


class FrozenIdentity(TypedDict):
    raw_sha256: str
    prompt_sha256: str


@dataclass(frozen=True, slots=True)
class Source:
    raw: Path
    prompt: Path
    look: str
    attempt: int
    raw_sha256: str
    prompt_sha256: str


@dataclass(frozen=True, slots=True)
class RunTask:
    source: Source
    root: Path
    output: Path


def run_one(task: RunTask) -> SheetRecord:
    """Evaluate independently into a new directory; never reserve generation budget."""
    source = task.source
    request = ProductionRequest(source.raw, source.look, source.look.split('_')[0],
                                source.attempt, source.prompt, task.output, task.root,
                                'ART-PROD2 Part 0: frozen-source sleep-length rule correction')
    request.diagnostic.mkdir(parents=True, exist_ok=False)
    result = evaluate(request)
    if (result.raw_sha256, result.prompt_sha256) != (source.raw_sha256, source.prompt_sha256):
        message = f'frozen source identity changed: {source.raw}'
        raise ValueError(message)
    _ = (request.diagnostic / 'sheet_checks.json').write_text(json.dumps(asdict(result), indent=2) + '\n')
    print(f'{task.root.name} {source.look} try{source.attempt}: {result.passing_panels}/10', flush=True)
    return result


def sources(root: Path) -> tuple[Source, ...]:
    """Require the exact preserved source population and original identity records."""
    items: list[Source] = []
    for raw in sorted(root.glob('*/try*/raw.png')):
        prompt = raw.with_name('prompt.txt')
        frozen: FrozenIdentity = json.loads(raw.with_name('sheet_checks.json').read_text())
        item = Source(raw, prompt, raw.parent.parent.name, int(raw.parent.name[3:]), digest(raw), digest(prompt))
        if (item.raw_sha256, item.prompt_sha256) != (frozen['raw_sha256'], frozen['prompt_sha256']):
            message = f'PROD1 raw/prompt hash mismatch: {raw}'
            raise ValueError(message)
        items.append(item)
    if len(items) != 96 or len({item.look for item in items}) != 36:
        message = f'expected 96 frozen sources / 36 looks; found {len(items)}'
        raise ValueError(message)
    return tuple(items)


def output_hashes(root: Path) -> dict[str, str]:   # path-to-digest domain index
    """PNG bytes remain exact; JSON replaces only the independent output-root path."""
    hashes: dict[str, str] = {}
    for path in sorted(root.rglob('*')):
        if path.is_file():
            payload = path.read_bytes()
            if path.suffix == '.json':
                payload = payload.replace(str(root).encode(), b'<RESLICE_ROOT>')
            hashes[str(path.relative_to(root))] = hashlib.sha256(payload).hexdigest()
    return hashes


def best(records: tuple[SheetRecord, ...]) -> SheetRecord:
    """First complete pass wins; otherwise most passing frames with earliest tie."""
    return min(records, key=lambda record: (not record.passed, -record.passing_panels, record.attempt))


def main() -> int:
    """Record complete repeated results, then publish each passing look atomically."""
    if len(sys.argv) != 5:
        print(__doc__)
        return 2
    source_root, data_root, output_root, report = map(Path, sys.argv[1:])
    frozen = sources(source_root)
    data_root.mkdir(parents=True, exist_ok=True)
    _ = (data_root / 'FROZEN_SOURCES.json').write_text(json.dumps([
        {**asdict(item), 'raw': str(item.raw), 'prompt': str(item.prompt)} for item in frozen], indent=2) + '\n')
    runs: list[tuple[SheetRecord, ...]] = []
    for number in (1, 2):
        root = data_root / f'reslice-run{number}'
        with ProcessPoolExecutor(max_workers=4) as executor:
            runs.append(tuple(executor.map(run_one, (RunTask(item, root, output_root) for item in frozen))))
    hashes = [output_hashes(data_root / f'reslice-run{number}') for number in (1, 2)]
    mismatches = sorted(key for key in hashes[0].keys() | hashes[1].keys() if hashes[0].get(key) != hashes[1].get(key))
    _ = (data_root / 'DETERMINISM.json').write_text(json.dumps({
        'passed': not mismatches, 'method': 'PNG exact bytes; JSON replaces only reslice-run root paths',
        'run1': hashes[0], 'run2': hashes[1], 'mismatches': mismatches}, indent=2) + '\n')
    if mismatches or sources(source_root) != frozen:
        message = 'determinism or frozen source preservation failed; see external evidence'
        raise ValueError(message)
    outcomes = []
    for look in sorted({item.look for item in frozen}):
        selected = best(tuple(record for record in runs[0] if record.look == look))
        before = [json.loads((source_root / look / f'try{record.attempt}' / 'sheet_checks.json').read_text())
                  for record in runs[0] if record.look == look]
        if selected.passed:
            source = next(item for item in frozen if item.look == look and item.attempt == selected.attempt)
            request = ProductionRequest(source.raw, look, selected.age, selected.attempt, source.prompt,
                                        output_root, data_root / 'reslice-run1', selected.review_reason)
            publish(request, selected)
        elif (output_root / look).exists():
            withdrawn = data_root / 'prior-accepted-withdrawn' / look
            withdrawn.parent.mkdir(parents=True, exist_ok=True)
            if withdrawn.exists():
                message = f'withdrawal evidence already exists: {withdrawn}'
                raise ValueError(message)
            _ = shutil.move(str(output_root / look), withdrawn)
        outcomes.append({'look': look, 'best_try': selected.attempt, 'accepted': selected.passed,
                         'before_attempts': before, 'after_best': asdict(selected),
                         'after_attempts': [asdict(record) for record in runs[0] if record.look == look]})
    payload = {'source_count': len(frozen), 'new_generations': 0, 'determinism_passed': not mismatches,
               'determinism_evidence': str(data_root / 'DETERMINISM.json'),
               'accepted_looks': sum(item['accepted'] for item in outcomes),
               'accepted_adults': sum(item['accepted'] for item in outcomes if item['look'].startswith('adult_')),
               'missing': [item['look'] for item in outcomes if not item['accepted']], 'looks': outcomes}
    report.parent.mkdir(parents=True, exist_ok=True)
    _ = (data_root / 'PART0_RESULTS_FULL.json').write_text(json.dumps(payload, indent=2) + '\n')
    for outcome in outcomes:
        for key in ('before_attempts', 'after_attempts'):
            outcome[key] = [{'attempt': record['attempt'], 'passed': record['passed'],
                             'passing_panels': record['passing_panels'],
                             'structural_passes': sum(frame['checks']['passed'] for frame in record['frames']),
                             'panel_judge_passes': sum(frame['judge'] is not None and frame['judge']['numeric_pass'] for frame in record['frames']),
                             'failures': [{'facing': frame['facing'], 'frame': frame['frame'],
                                           'structural': frame['checks']['findings'],
                                           'panel_numeric_pass': frame['judge']['numeric_pass'] if frame['judge'] else None,
                                           'error': frame['error']} for frame in record['frames'] if not frame['passed']]}
                            for record in outcome[key]]
        del outcome['after_best']
    payload['full_evidence'] = str(data_root / 'PART0_RESULTS_FULL.json')
    _ = report.write_text(json.dumps(payload, indent=2) + '\n')
    print(json.dumps({key: value for key, value in payload.items() if key != 'looks'}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
