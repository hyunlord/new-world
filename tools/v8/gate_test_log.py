#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by gate_test_evidence.py.
"""Read final outer libtest result for every Cargo execution segment."""
import re
import math
from typing import TypedDict


class Target(TypedDict):
    target: str
    status: str
    exit_code: int
    passed: int
    failed: int
    ignored: int
    measured: int
    filtered: int
    seconds: float


class Summary(TypedDict):
    targets: list[Target]
    passed: int
    ignored: int
    compile_seconds: float
    run_seconds: float

from mux2_gate_inputs import require

HEADER = re.compile(r'^\s*(?:Running (.+?) \(target/(?:debug|release)/deps/([^()]+)-[0-9a-f]+\)|Doc-tests (\w+))\s*$')
RESULT = re.compile(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out; finished in ([\d.]+)s$')


def parse_log(text: str, expected: list[str] | None = None, debug: bool = True) -> Summary:
    require(text.endswith('\n'), 'partial log: missing final newline')
    lines = text.splitlines()
    compile_times = []
    segments: list[tuple[str, list[str]]] = []
    current: tuple[str, list[str]] | None = None
    headers = set()
    for line in lines:
        finished = re.search(r'Finished `([^`]+)` profile .* target\(s\) in ([\d.]+)([sm])(?: ([\d.]+)s)?$', line)
        if finished:
            require(not debug or finished[1] == 'test', 'not a debug test profile')
            seconds = float(finished[2]) * (60 if finished[3] == 'm' else 1) + float(finished[4] or 0)
            require(math.isfinite(seconds) and seconds >= 0, 'invalid compile duration')
            compile_times.append(seconds)
        header = HEADER.match(line)
        if header:
            require(not debug or 'target/release/' not in line, 'release target in debug suite')
            require(line.strip() not in headers, 'duplicate execution header')
            headers.add(line.strip())
            if current:
                segments.append(current)
            description, binary, doc = header.groups()
            if doc:
                key = f'doc:{doc}'
            elif description.startswith('unittests '):
                source = description.removeprefix('unittests ')
                key = f'unit:{binary}:lib' if source == 'src/lib.rs' else f'unit:{binary}:{source}'
            else:
                require(description.startswith('tests/'), 'unknown execution target')
                key = 'test:' + description.removeprefix('tests/').removesuffix('.rs').removesuffix('/main')
            current = (key, [])
        elif current:
            current[1].append(line)
    if current:
        segments.append(current)
    require(len(compile_times) == 1 and segments, 'missing/duplicate compile banner or targets')
    targets: list[Target] = []
    for key, segment_lines in segments:
        nonempty = [line for line in segment_lines if line.strip()]
        require(bool(nonempty), 'empty target')
        running = next((re.fullmatch(r'running (\d+) tests?', line) for line in nonempty
                        if re.fullmatch(r'running (\d+) tests?', line)), None)
        require(running is not None, 'missing outer test count')
        result = RESULT.fullmatch(nonempty[-1])
        require(result is not None, 'missing final outer test result')
        status, passed, failed, ignored, measured, filtered, seconds = result.groups()
        require(status == 'ok' and int(failed) == 0 and int(filtered) == 0 and int(measured) == 0,
                'failed/filtered/measured suite')
        require(int(running[1]) == int(passed) + int(ignored), 'outer test count mismatch')
        require(math.isfinite(float(seconds)) and float(seconds) >= 0, 'invalid run duration')
        targets.append({'target': key, 'status': 'ok', 'exit_code': 0, 'passed': int(passed), 'failed': int(failed),
                        'ignored': int(ignored), 'measured': int(measured),
                        'filtered': int(filtered), 'seconds': float(seconds)})
    keys = sorted(row['target'] for row in targets)
    if expected is not None:
        require(keys == sorted(expected), 'missing/unexpected target inventory')
    total = sum(row['passed'] for row in targets)
    require(total > 0, 'zero tests')
    return {'targets': targets, 'passed': total, 'ignored': sum(row['ignored'] for row in targets),
            'compile_seconds': compile_times[0], 'run_seconds': sum(row['seconds'] for row in targets)}
