# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: python -m art.pipeline5.prod2_verify_new DATA_ROOT ROUND EXPECTED_COUNT REPORT_JSON
"""Repeat every completed generation independently without publishing or reserving."""
from __future__ import annotations

import json
import sys
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path
from typing import TypedDict

from art.pipeline5.attempt_integrity import digest
from art.pipeline5.prod1_records import SheetRecord
from art.pipeline5.prod2_reslice import RunTask, Source, output_hashes, run_one


class Intent(TypedDict):
    look: str
    attempt: int
    status: str
    raw: str
    raw_sha256: str
    prompt_path: str
    prompt_sha256: str
    diagnostics: str
    passing_panels: int


class OriginalRecord(TypedDict):
    raw_sha256: str
    prompt_sha256: str
    passed: bool
    passing_panels: int


class Outcome(TypedDict):
    look: str
    attempt: int
    passed: bool
    passing_panels: int
    original_outcome_matches: bool


class PriorD1(TypedDict):
    passed: bool


def generation_sources(data: Path, expected: int) -> tuple[Source, ...]:
    """Bind each terminal intent to its immutable raw, prompt and first evaluation."""
    paths = sorted((data / 'generation').glob('*/try*/intent.json'))
    records = sorted((data / 'new-production').glob('*/try*/sheet_checks.json'))
    if len(paths) != expected or len(records) != expected or expected < 1:
        message = f'expected {expected} intents and records; found {len(paths)} and {len(records)}'
        raise ValueError(message)
    result: list[Source] = []
    for path in paths:
        intent: Intent = json.loads(path.read_text())
        raw, prompt = Path(intent['raw']), Path(intent['prompt_path'])
        record: OriginalRecord = json.loads((Path(intent['diagnostics']) / 'sheet_checks.json').read_text())
        source = Source(raw, prompt, intent['look'], intent['attempt'], digest(raw), digest(prompt))
        identity = (source.raw_sha256, source.prompt_sha256)
        if (intent['status'] not in ('accepted', 'rejected')
                or raw.parent != path.parent or prompt.parent != path.parent
                or identity != (intent['raw_sha256'], intent['prompt_sha256'])
                or identity != (record['raw_sha256'], record['prompt_sha256'])
                or intent['passing_panels'] != record['passing_panels']
                or (intent['status'] == 'accepted') != record['passed']):
            message = f'terminal intent/source/first-evaluation mismatch: {path}'
            raise ValueError(message)
        result.append(source)
    return tuple(result)


def main() -> int:
    """Write full external hash evidence and a compact combined D1 verdict."""
    if len(sys.argv) != 5:
        print('DATA_ROOT ROUND EXPECTED_COUNT REPORT_JSON')
        return 2
    data, round_name, count, report = Path(sys.argv[1]), sys.argv[2], int(sys.argv[3]), Path(sys.argv[4])
    if not round_name.isalnum():
        message = 'round must contain only letters and digits'
        raise ValueError(message)
    sources = generation_sources(data, count)
    preserved = {name: output_hashes(data / name) for name in ('generation', 'new-production')}
    root = data / 'd1-new' / round_name
    root.mkdir(parents=True, exist_ok=False)
    _ = (root / 'FROZEN_INPUTS.json').write_text(json.dumps(preserved, indent=2) + '\n')
    runs: list[tuple[SheetRecord, ...]] = []
    for number in (1, 2):
        with ProcessPoolExecutor(max_workers=4) as executor:
            records = tuple(executor.map(run_one, (RunTask(source, root / f'run{number}', root / 'never-published') for source in sources)))
        runs.append(records)
    hashes = [output_hashes(root / f'run{number}') for number in (1, 2)]
    mismatches = sorted(key for key in hashes[0].keys() | hashes[1].keys() if hashes[0].get(key) != hashes[1].get(key))
    unchanged = sources == generation_sources(data, count) and preserved == {
        name: output_hashes(data / name) for name in preserved}
    outcomes: list[Outcome] = []
    original_matches = True
    for record in runs[0]:
        old: OriginalRecord = json.loads((data / 'new-production' / record.look / f'try{record.attempt}' / 'sheet_checks.json').read_text())
        same_result = (record.passed, record.passing_panels) == (old['passed'], old['passing_panels'])
        original_matches &= same_result
        outcomes.append({'look': record.look, 'attempt': record.attempt, 'passed': record.passed,
                         'passing_panels': record.passing_panels, 'original_outcome_matches': same_result})
    passed = not mismatches and unchanged and original_matches
    evidence = root / 'DETERMINISM.json'
    _ = evidence.write_text(json.dumps({'passed': passed, 'sources': len(sources),
        'input_and_first_evidence_unchanged': unchanged, 'original_outcomes_match': original_matches,
        'method': 'PNG exact bytes; JSON replaces only independent run root paths',
        'run1': hashes[0], 'run2': hashes[1], 'mismatches': mismatches, 'outcomes': outcomes}, indent=2) + '\n')
    original: PriorD1 = json.loads((data / 'DETERMINISM.json').read_text())
    summary = {'passed': original['passed'] and passed, 'original_source_count': 96,
               'new_source_count': len(sources), 'total_source_count': 96 + len(sources),
               'original_evidence': str(data / 'DETERMINISM.json'), 'new_evidence': str(evidence),
               'new_passing_sources': sum(record.passed for record in runs[0]),
               'new_failing_sources': sum(not record.passed for record in runs[0]),
               'new_runs_each': len(sources), 'input_and_first_evidence_unchanged': unchanged,
               'method': 'PNG exact bytes; JSON root paths canonicalized only', 'outcomes': outcomes}
    _ = report.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({key: value for key, value in summary.items() if key != 'outcomes'}))
    return 0 if summary['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
