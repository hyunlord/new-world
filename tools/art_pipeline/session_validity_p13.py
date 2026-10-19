#!/usr/bin/env python3
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
# How to run: P13_REFERENCE=reference.json P13_VALIDITY_OUT=unique.json python3 session_validity_p13.py COMMAND ...
"""Measure admission on both sides; retain rejected and incomplete evidence."""
from __future__ import annotations

import fcntl
import json
import math
import os
import re
import subprocess
import sys
import time
from dataclasses import asdict
from pathlib import Path

from cpu_probe_p13 import KERNEL_ID, Probe, reference_seconds, snapshot, write_json


def valid_snapshot(value: Probe | dict, reference: float | None = None) -> bool:
    """Foreign CPU percentages are evidence only; load and probe decide validity."""
    data = asdict(value) if isinstance(value, Probe) else value
    reference = reference if reference is not None else data.get('reference_seconds')
    numbers = [data.get('wall_seconds'), data.get('load_1m'), reference]
    if any(isinstance(item, bool) or not isinstance(item, (int, float)) or not math.isfinite(item) for item in numbers):
        return False
    return 0 < data['wall_seconds'] <= 1.05 * reference and reference > 0 and 0 <= data['load_1m'] < 8.0


def load_reference(path: Path) -> float:
    """Reject partial calibrations, changed work, or altered reference calculation."""
    data = json.loads(path.read_text())
    if not data.get('complete') or data.get('kernel_id') != KERNEL_ID:
        raise ValueError(f'incompatible calibration: {path}')
    runs = data['runs']
    reference = reference_seconds([run['wall_seconds'] for run in runs])
    if reference != data['reference_seconds'] or any(b['epoch'] - a['epoch'] < 20 for a, b in zip(runs, runs[1:])):
        raise ValueError(f'invalid calibration reference or spacing: {path}')
    return reference


def require_valid(path: Path) -> None:
    data = json.loads(path.read_text())
    reference = data.get('reference_seconds')
    if data.get('kernel_id') != KERNEL_ID or data.get('exit_code') != 0 or not all(valid_snapshot(data.get(key, {}), reference) for key in ('start', 'end')):
        raise ValueError(f'NOT_VERIFIED (contended or unsuccessful): {path}')


def update_ledger(output: Path | None, wait: tuple[float, float] | None = None) -> bool:
    """Serialize ledger mutations; count only the union of explicit waiting intervals."""
    ledger = Path(os.environ.get('P13_ATTEMPT_LEDGER', 'docs/art/production13/perf/attempts.json'))
    ledger.parent.mkdir(parents=True, exist_ok=True)
    with ledger.with_suffix('.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        data = json.loads(ledger.read_text()) if ledger.exists() else {'items': {}, 'waiting_seconds': 0, 'wait_intervals': [], 'events': []}
        if wait is not None:
            start, end = wait
            if not 0 <= start <= end < float('inf'):
                raise ValueError(f'invalid wait interval: {wait}')
            intervals = sorted(data.get('wait_intervals', []) + [[start, end]])
            merged: list[list[float]] = []
            for start, end in intervals:
                if merged and start <= merged[-1][1]:
                    merged[-1][1] = max(merged[-1][1], end)
                else:
                    merged.append([start, end])
            data['wait_intervals'] = merged
            data['waiting_seconds'] = sum(end - start for start, end in merged)
        admitted = data['waiting_seconds'] <= 10800
        if output is not None:
            item = os.environ.get('P13_ITEM', re.sub(r'-attempt\d+', '', os.environ.get('P13_PURPOSE', str(output))))
            attempts = data['items'].setdefault(item, [])
            now = time.time()
            admitted = admitted and len(attempts) < 6 and (not attempts or now - attempts[-1] >= 1800)
            data['events'].append({'epoch': now, 'item': item, 'admitted': admitted, 'output': str(output)})
            if admitted:
                attempts.append(now)
        write_json(ledger, data)
        return admitted


def record_attempt(output: Path) -> bool:
    return update_ledger(output)


def main() -> int:
    if len(sys.argv) == 4 and sys.argv[1] == '--record-wait':
        return 0 if update_ledger(None, (float(sys.argv[2]), float(sys.argv[3]))) else 75
    output = Path(os.environ['P13_VALIDITY_OUT'])
    if output.exists():
        raise FileExistsError(output)
    reference_path = Path(os.environ['P13_REFERENCE'])
    reference = load_reference(reference_path)
    if not record_attempt(output):
        write_json(output, {'valid': False, 'verdict': 'RETRY_NOT_ADMITTED', 'epoch': time.time()})
        return 75
    holds = Path(os.environ.get('P13_HOLDS', 'docs/art/production13/perf/holds.md'))
    start = snapshot()
    data = {'kernel_id': KERNEL_ID, 'reference_seconds': reference, 'reference_path': str(reference_path),
            'start': asdict(start), 'valid': False, 'verdict': 'INCOMPLETE'}
    write_json(output, data)
    with holds.open('a') as handle:
        handle.write(f'- PROBE_BEFORE {output} P={start.wall_seconds:.9f} R={reference:.9f} load={start.load_1m}\n')
    code = subprocess.run(sys.argv[1:], check=False).returncode if valid_snapshot(start, reference) else 75
    end = snapshot()
    valid = valid_snapshot(start, reference) and valid_snapshot(end, reference) and code == 0
    data.update(end=asdict(end), exit_code=code, valid=valid, verdict='VALID' if valid else 'NOT_VERIFIED (contended or unsuccessful)')
    write_json(output, data)
    with holds.open('a') as handle:
        handle.write(f'- PROBE_AFTER {output} P={end.wall_seconds:.9f} R={reference:.9f} load={end.load_1m} valid={valid} exit={code}\n')
    return code if code else (0 if valid else 75)


if __name__ == '__main__':
    raise SystemExit(main())
