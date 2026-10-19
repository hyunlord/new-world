#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Aggregate existing raw captures only; never launch or admit a measurement.

Usage: python3 aggregate_after.py INPUT_FOLDER OUTPUT_FOLDER after|historical-smoke
Each attempt remains separate. Calibrations are neither loaded nor substituted.
"""
from __future__ import annotations

import hashlib
import json
import math
import statistics
import sys
from pathlib import Path
from typing import Final

type Json = None | bool | int | float | str | list[Json] | dict[str, Json]
type Record = dict[str, Json]
METRICS: Final = ('primary_cpu_ms', 'process_ms', 'render_cpu_ms', 'setup_cpu_ms', 'pacing_ms', 'ticks_run')
WORLDS: Final = ('W0', 'W1', 'W200', 'W500', 'W-multi')


class EvidenceError(Exception):
    """Malformed raw input, reported without inventing a result."""


def record(value: Json) -> Record:
    if not isinstance(value, dict):
        raise EvidenceError('Expected JSON object')
    return value


def rows(value: Json) -> list[Record]:
    if not isinstance(value, list):
        raise EvidenceError('Expected JSON row array')
    return [record(item) for item in value]


def number(value: Json) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise EvidenceError('Expected finite numeric metric')
    return float(value)


def distribution(values: list[float]) -> Record:
    """Nearest-rank percentiles; an empty set has null statistics, never zero."""
    ordered = sorted(values)
    size = len(ordered)
    return {'count': size, 'median': statistics.median(ordered) if size else None,
            'p90': ordered[math.ceil(.90 * size) - 1] if size else None,
            'p99': ordered[math.ceil(.99 * size) - 1] if size else None,
            'max': ordered[-1] if size else None, 'mean': statistics.mean(ordered) if size else None}


def summarize(samples: list[Record]) -> Record:
    metrics = {name: distribution([number(row[name]) for row in samples]) for name in METRICS}
    for row in samples:
        if any(number(row[key]) < 0 for key in METRICS):
            raise EvidenceError('Negative metric')
        if not math.isclose(number(row['primary_cpu_ms']), number(row['process_ms']) + number(row['render_cpu_ms']), abs_tol=1e-9, rel_tol=0):
            raise EvidenceError('Primary does not equal process plus render')
    ticks = [number(row['ticks_run']) for row in samples]
    carrying = [row for row in samples if number(row['ticks_run']) > 0]
    return {'frames': len(samples), 'ticks_total': sum(ticks), 'tick_carrying_frames': len(carrying),
            'metrics': metrics, 'tick_carrying_metrics': {key: distribution([number(row[key]) for row in carrying]) for key in METRICS},
            'counted_first_state': samples[0]['state'] if samples else None,
            'counted_last_state': samples[-1]['state'] if samples else None,
            'gpu_statuses': sorted({str(row.get('gpu_status', 'NOT_RECORDED')) for row in samples})}


def sidecar(path: Path) -> Record:
    """Retain stated admission only; no import of environment-bound validators."""
    target = Path(str(path) + '.validity.json')
    if not target.is_file():
        return {'status': 'NOT_FOUND', 'path': str(target)}
    raw = target.read_bytes()
    data = record(json.loads(raw))
    result: Record = {key: data.get(key) for key in ('valid', 'verdict', 'exit_code', 'reference_path', 'reference_sha256', 'reference_seconds')}
    result.update(path=str(target), sha256=hashlib.sha256(raw).hexdigest(), status='RECORDED_NOT_REVALIDATED')
    for key in ('start', 'end'):
        result[key] = {k: v for k, v in record(data[key]).items() if k != 'ps'}
    return result


def matrix(data: Record) -> Record:
    pooled: dict[str, list[Record]] = {}
    blocks: list[Json] = []
    labels: list[str] = []
    for block in rows(data['blocks']):
        if block['condition'] not in ('frozen', 'running'):
            continue
        samples = rows(block['samples'])
        label, condition = str(block['label']), str(block['condition'])
        labels.append(label)
        flagged = condition == 'running' and number(block['population_end']) < .9 * number(block['population_start'])
        if block['population_flagged'] != flagged:
            raise EvidenceError('Population flag differs from raw endpoints')
        group = f"{condition}_Z{block['zoom']}_{block['speed']}x"
        pooled.setdefault(group, [])
        if not flagged:
            pooled[group].extend(samples)
        blocks.append({'label': label, 'condition': condition, 'population_flagged': flagged,
                       'population_start': block['population_start'], 'population_end': block['population_end'],
                       'start_state': block['start_state'], 'end_state': block['end_state'],
                       'discarded_frames': len(rows(block['discarded'])), 'statistics': summarize(samples)})
    expected = [f'frozen_Z4_{i}' for i in range(3)] + [f'running_2x_{i}' for i in range(3)]
    complete = labels == expected and all(record(record(b)['statistics'])['frames'] == 300 and record(b)['discarded_frames'] == 30 for b in blocks)
    return {'kind': 'matrix', 'coverage': 'COMPLETE_RAW_SHAPE' if complete else 'INCOMPLETE_RAW_SHAPE',
            'blocks': blocks, 'eligible_pooled': {key: summarize(value) for key, value in pooled.items()},
            'split_fragments': data.get('split_fragments', [])}


def sections(data: Record) -> Record:
    samples = rows(data['samples'])
    timer_rows = [record(row['agent_sections']) for row in samples]
    names = sorted(record(timer_rows[0]['exclusive_usec'])) if timer_rows else []
    for timer, row in zip(timer_rows, samples, strict=True):
        values = record(timer['exclusive_usec'])
        if sorted(values) != names or sum(number(v) for v in values.values()) != number(timer['total_usec']) or timer['remainder_usec'] != 0 or timer['process_frame'] != row['process_frame'] or timer['valid'] is not True or timer['complete'] is not True:
            raise EvidenceError('Incomplete, misaligned, or inconsistent exclusive timers')
    exclusive = {key: distribution([number(record(t['exclusive_usec'])[key]) / 1000 for t in timer_rows]) for key in names}
    getter_names = sorted({key for row in samples for key in record(row['ffi_calls'])} | {key for row in samples for key in record(row['ffi_call_usec'])})
    getters: Record = {}
    for name in getter_names:
        getters[name] = {'calls_per_frame': distribution([number(record(row['ffi_calls']).get(name, 0)) for row in samples]),
                         'ms_per_frame': distribution([number(record(row['ffi_call_usec']).get(name, 0)) / 1000 for row in samples])}
    return {'kind': 'sections', 'coverage': 'COMPLETE_RAW_SHAPE' if len(samples) == 300 and len(rows(data['discarded'])) == 30 else 'INCOMPLETE_RAW_SHAPE',
            'statistics': summarize(samples), 'discarded_frames': len(rows(data['discarded'])), 'initial': data['initial'], 'final': data['final'],
            'exclusive_ms': exclusive, 'agent_renderer_total_ms': distribution([number(t['total_usec']) / 1000 for t in timer_rows]),
            'nested_getters_total_ms': distribution([sum(number(v) for v in record(row['ffi_call_usec']).values()) / 1000 for row in samples]),
            'getters': getters, 'conditions': data.get('conditions'), 'founding_groups_status': data.get('founding_groups_status')}


def aggregate(folder: Path, label: str) -> Record:
    sessions: list[Json] = []
    errors: list[Json] = []
    candidates: list[tuple[Path, Record, str]] = []
    fragments: set[Path] = set()
    for path in sorted(folder.rglob('*.json')) if folder.exists() else []:
        if path.name == 'after-detail.json':
            continue
        try:
            raw = path.read_bytes()
            value: Json = json.loads(raw)
            if not isinstance(value, dict) or 'world' not in value:
                continue
            is_matrix = 'blocks' in value and value.get('phase') in ('baseline', 'after') and not value.get('ablation_only')
            is_sections = value.get('diagnostic_only') is True and 'samples' in value and 'instrumented' in value
            if not (is_matrix or is_sections):
                continue
            for item in rows(value.get('split_fragments', [])):
                fragments.add(Path(str(item['source'])).resolve())
            candidates.append((path, value, hashlib.sha256(raw).hexdigest()))
        except (OSError, ValueError, EvidenceError) as exc:
            errors.append({'source': str(path), 'status': 'UNREADABLE', 'error': str(exc)})
    for path, data, digest in candidates:
        if path.resolve() in fragments:
            continue
        try:
            if label == 'after' and data.get('phase') == 'baseline':
                raise EvidenceError('Baseline raw file refused in after mode; use historical-smoke')
            detail = matrix(data) if 'blocks' in data else sections(data)
            detail.update(source=str(path.resolve()), sha256=digest, world=data['world'],
                          capture_valid=data.get('valid'), capture_errors=data.get('errors'),
                          build_commit=data.get('build_commit'), build_sha256=data.get('build_sha256'),
                          raw_metadata={key: value for key, value in data.items() if key not in ('blocks', 'samples', 'discarded')},
                          admission=sidecar(path), acceptance='NOT_ASSESSED')
            sessions.append(detail)
        except (KeyError, ValueError, EvidenceError) as exc:
            errors.append({'source': str(path), 'status': 'MALFORMED_OR_WRONG_PHASE', 'error': str(exc)})
    pending: list[Json] = []
    for kind, worlds in (('matrix', WORLDS), ('sections', ('W200', 'W-multi'))):
        for world in worlds:
            present = any(record(s)['world'] == world and record(s)['kind'] == kind and record(s)['coverage'] == 'COMPLETE_RAW_SHAPE' for s in sessions)
            if not present:
                pending.append({'kind': kind, 'world': world, 'status': 'NOT_MEASURED_OR_INCOMPLETE'})
    return {'label': label, 'input': str(folder.resolve()), 'method': 'Raw counted rows; nearest-rank percentiles; zero-call frames included; attempts separate; population-loss blocks excluded only from eligible pools.',
            'acceptance': 'NOT_ASSESSED; use official validators with each source-specific calibration',
            'sessions': sessions, 'pending': pending, 'errors': errors}


def markdown(report: Record) -> str:
    lines = [f"# {report['label']} — offline raw detail", '', str(report['acceptance']), '', str(report['method']), '', 'Pending: ' + json.dumps(report['pending']), '', 'Errors: ' + json.dumps(report['errors'])]
    for session in rows(report['sessions']):
        lines += ['', f"## {session['world']} {session['kind']}", '', f"Source: `{session['source']}`; SHA256 `{session['sha256']}`; {session['coverage']}; capture valid={session['capture_valid']}."]
        groups = record(session.get('eligible_pooled', {'capture': session.get('statistics')}))
        for name, group in groups.items():
            g = record(group)
            lines += ['', f"### {name}: {g['frames']} frames, {g['ticks_total']} ticks, {g['tick_carrying_frames']} tick-carrying frames", '', '| Metric | Median | p90 | p99 | Max | Mean |', '|---|---:|---:|---:|---:|---:|']
            for metric, dist in record(g['metrics']).items():
                v = record(dist); lines.append('| ' + metric + ' | ' + ' | '.join(str(v[k]) for k in ('median', 'p90', 'p99', 'max', 'mean')) + ' |')
            lines += ['', 'Counted population/tick endpoints: ' + json.dumps([g['counted_first_state'], g['counted_last_state']])]
        for field in ('exclusive_ms', 'getters'):
            if field in session:
                lines += ['', f'### {field}', '', '```json', json.dumps(session[field], indent=2), '```']
    return '\n'.join(lines) + '\n'


def main() -> None:
    if len(sys.argv) != 4 or sys.argv[3] not in ('after', 'historical-smoke'):
        raise SystemExit(__doc__)
    folder, output = Path(sys.argv[1]), Path(sys.argv[2])
    report = aggregate(folder, sys.argv[3])
    output.mkdir(parents=True, exist_ok=True)
    (output / 'after-detail.json').write_text(json.dumps(report, indent=2, allow_nan=False) + '\n')
    (output / 'after-detail.md').write_text(markdown(report))
    print(json.dumps({'label': report['label'], 'sessions': len(rows(report['sessions'])), 'pending': report['pending'], 'errors': report['errors']}))


if __name__ == '__main__':
    main()
