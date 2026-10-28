#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Authenticate UI-successor paired raws and recompute all standing/C3 evidence."""
from __future__ import annotations
import json
from pathlib import Path
import statistics
from core_perf_check import (CEILINGS, RSS_SCENES, SCENES, Document, Json, ROOT, frozen_file,
    loads, read_records, require, sequence, sha, validate_dimensions,
    validate_schedule, verify_file, write)


def check(out: Path, output: Path | None = None) -> int:
    """Recheck evidence; an external output leaves the frozen session untouched."""
    out = out.resolve()
    if output is not None:
        output = output.resolve()
        require(not output.is_relative_to(out.resolve()), 'fresh receipt must be outside the frozen session')
        require(not output.exists(), 'fresh receipt output already exists')
    from core_ui_successor_perf import BASE, competing_processes, failures, verify_config
    config = Document.read(out / 'config.json')
    verify_config(config)
    prior_receipt = out / 'receipt.json'
    if prior_receipt.exists():
        prior = Document.read(prior_receipt)
        if 'files' in prior.values:
            require(prior.text('config_sha256') == sha(out / 'config.json'), 'changed session configuration')
            for record in prior.rows('files'):
                path = verify_file(record)
                require(path.is_relative_to(out), 'receipt file escaped session')
    records = read_records(out / 'attempts.json')
    for row in records:
        monitor_path = verify_file(row.child('process_monitor'))
        require(monitor_path.parent == out / 'attempts', 'monitor escaped session')
        monitor = Document.read(monitor_path)
        require(monitor.number('started') == row.number('started') and monitor.number('ended') == row.number('ended'), 'monitor invocation mismatch')
        require(monitor.number('exit_code') == row.number('exit_code'), 'monitor exit mismatch')
        if row.values.get('pair_accepted') is True:
            require(monitor_clean(monitor), 'accepted competing process/gap/deadline')
        for name in ('processes_before', 'processes_after'):
            snapshot = verify_file(row.child(name))
            require(snapshot.parent == out / 'attempts', 'process evidence escaped session')
            if row.values.get('pair_accepted') is True:
                require(not competing_processes(snapshot.read_text()), 'accepted concurrent work')
    validate_schedule(records)
    admissions = [Document.parse(json.loads(line)) for line in (out / 'admission.jsonl').read_text().splitlines()]
    admitted_rows = [row for row in admissions if row.text('status') == 'ADMITTED']
    admitted = {row.text('label'): row for row in admitted_rows}
    require(len(admitted) == len(admitted_rows), 'duplicate external admission label')
    selected: dict[tuple[bool, str, int, str], Document] = {}
    previous_end = 0.0
    for row in records:
        raw_path = Path(row.text('raw'))
        require(raw_path.parent == (out / 'attempts').resolve(), 'raw is outside this session')
        require(row.number('started') >= previous_end, 'overlapping or reordered timed invocations')
        previous_end = row.number('ended')
        require(previous_end >= row.number('started'), 'negative invocation duration')
        if raw_path.exists():
            require(sha(raw_path) == row.text('sha256'), 'changed raw attempt')
        build_only = row.values.get('build_only')
        require(type(build_only) is bool, 'missing build-only discriminator')
        scene, repeat, role = row.text('scene'), int(row.number('repeat')), row.text('role')
        require(role in ('base', 'new'), 'unknown binary role')
        if row.values.get('pair_accepted') is not True:
            continue
        require(row.text('status') == 'VALID' and row.number('exit_code') == 0, 'invalid accepted member')
        raw = Document.read(raw_path)
        validate_dimensions(raw, scene, build_only, repeat)
        external = row.values.get('external_load')
        require(type(external) is list and len(external) == 3, 'missing external 1/5/15 loads')
        sample = loads(' '.join(str(value) for value in external))
        internal = loads(raw.text('load_before'))
        require(sample[0] < 8 and internal[0] < 8, 'contended accepted member')
        require(list(internal) == row.values.get('internal_load'), 'internal admission differs from raw')
        admission = admitted.get(raw_path.stem)
        require(admission is not None and admission.values.get('loads_1_5_15') == external, 'missing matching external admission')
        require(0 <= row.number('started') - admission.number('utc') <= 30, 'admission is not immediately before execution')
        if build_only:
            require(raw.number('repeat') == repeat, 'RSS repeat mismatch')
        else:
            require(raw.number('seed') == 15100 and raw.number('warmup_ticks') == 1440
                    and raw.number('measured_ticks') == 2880 and raw.text('build_profile') == 'release', 'wrong timed protocol')
        slot = (build_only, scene, repeat, role)
        require(slot not in selected, 'duplicate accepted repeat')
        selected[slot] = raw
        accepted_path = out / 'accepted' / role / f'{"rss" if build_only else "timed"}-{scene}-r{repeat}.json'
        require(sha(accepted_path) == sha(raw_path), 'summary input differs from raw pair')
    expected = {(build, scene, repeat, role) for build in (False, True)
                for repeat, scene in sequence(build) for role in ('base', 'new')}
    require(set(selected) == expected, 'missing or extra standing timed/RSS pairs')
    for role in ('base', 'new'):
        require(len(list((out / 'accepted' / role).glob('*.json'))) == 33, 'unexpected summary input file')
    summaries = {role: Document.read(out / f'{role}-summary.json') for role in ('base', 'new')}
    for role, summary in summaries.items():
        head = config.text('base_head' if role == 'base' else 'candidate_head')
        require(summary.text('commit') == head, 'summary commit mismatch')
        rows = summary.rows('scenes')
        require(len(rows) == 6 and {row.text('label') for row in rows} == set(SCENES), 'summary scene coverage')
        require(summary.child('rss_outer').number('repeats') == 5, 'RSS outer bound needs five repeats')
        for scene in rows:
            label = scene.text('label')
            require(scene.number('repeats') == 3, 'summary needs three timed repeats')
            for metric in ('tick_mean_us', 'tick_p99_us'):
                median = statistics.median(selected[(False, label, repeat, role)].number(metric) for repeat in (1, 2, 3))
                require(abs(scene.number(metric) - median) < 1e-8, 'summary differs from paired raws')
        med_alloc = {scene: statistics.median(selected[(False, scene, repeat, role)].child('alloc_bytes').number('after_build') for repeat in (1, 2, 3)) for scene in RSS_SCENES}
        alloc_agent = (med_alloc['n2000'] - med_alloc['n200']) / 1800
        alloc_tile = (med_alloc['n200'] - med_alloc['n200_side64']) / (192**2 - 64**2)
        rss_agent = statistics.median((selected[(True, 'n2000', repeat, role)].child('rss_kib').number('after_build') - selected[(True, 'n200', repeat, role)].child('rss_kib').number('after_build')) * 1024 / 1800 for repeat in range(1, 6))
        rss_tile = statistics.median((selected[(True, 'n200', repeat, role)].child('rss_kib').number('after_build') - selected[(True, 'n200_side64', repeat, role)].child('rss_kib').number('after_build')) * 1024 / (192**2 - 64**2) for repeat in range(1, 6))
        require(summary.text('memory_ceiling_source') == 'alloc', 'standing C3 lacks allocator counter')
        for field, expected_value in [('alloc_per_agent', alloc_agent), ('alloc_per_tile', alloc_tile)]:
            require(abs(summary.number(field) - expected_value) < 1e-8, 'allocator summary differs from raw')
        for field, expected_value in [('per_agent_median', rss_agent), ('per_tile_median', rss_tile)]:
            require(abs(summary.child('rss_outer').number(field) - expected_value) < 1e-8, 'RSS summary differs from raw pairs')
        by_label = {row.text('label'): row for row in rows}
        expected_ceilings = dict(zip(CEILINGS, [by_label['n1000'].number('tick_mean_us') <= 5000,
            by_label['n2000'].number('tick_mean_us') <= 10000, alloc_agent <= 4096,
            alloc_tile <= 96, rss_agent <= 5120, rss_tile <= 128], strict=True))
        for field, passes in expected_ceilings.items():
            require(summary.child('ceilings').text(field) == ('PASS' if passes else 'MISS'), 'C3 flag differs from raw instrument')
    failed = failures(summaries['base'], summaries['new'])
    receipt: dict[str, Json] = {'kind': 'ui_successor_fresh_performance_v1',
        'status': 'FAIL' if failed else 'PASS', 'failures': failed,
        'base_head': BASE, 'candidate_head': config.text('candidate_head'),
        'identical_runtime_binary': config.values['identical_runtime_binary'],
        'config_sha256': sha(out / 'config.json'), 'accepted_pairs': 33,
        'files': [frozen_file(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name != 'receipt.json'],
        'waivers': [], 'comparison': 'fresh-same-session-published-BASE'}
    destination = output if output is not None else out / 'receipt.json'
    write(destination, receipt, exclusive=output is not None)
    print(json.dumps({'status': receipt['status'], 'failures': failed}))
    return int(bool(failed))



from collections.abc import Mapping, Sequence
from dataclasses import dataclass
import subprocess
import time


@dataclass(frozen=True, slots=True)
class MonitoredResult:
    """Completion of the single owned child, with trace-derived admission."""
    returncode: int
    started: float
    ended: float
    clean: bool


def monitor_clean(trace: Document) -> bool:
    """Reject gaps, competing work or deadline overrun across a full invocation."""
    from core_ui_successor_perf import competing_processes
    samples = trace.rows('samples')
    start, end = trace.number('started'), trace.number('ended')
    if not samples or trace.number('pid') <= 0 or end < start:
        return False
    if trace.values.get('deadline_exceeded') is not False or end > trace.number('deadline'):
        return False
    previous = start
    for sample in samples:
        current = sample.number('utc')
        if current < previous or current - previous > 2 or current > end:
            return False
        if competing_processes(sample.text('snapshot')):
            return False
        previous = current
    return end - previous <= 2


def run_monitored(argv: Sequence[str], environment: Mapping[str, str], log: Path,
                  trace_path: Path, deadline: float) -> MonitoredResult:
    """Poll each second and terminate only this Popen child at the hard deadline."""
    from core_ui_successor_perf import process_snapshot
    started = time.time()
    samples: list[Json] = []
    exceeded = started >= deadline
    child_pid, code = 0, 124
    with log.open('w') as stream:
        if not exceeded:
            with subprocess.Popen(argv, cwd=ROOT, env=environment, stdout=stream, stderr=subprocess.STDOUT) as child:
                child_pid = child.pid
                try:
                    while True:
                        snapshot = process_snapshot()
                        now = time.time()
                        samples.append({'utc': now, 'snapshot': snapshot})
                        code_now = child.poll()
                        if code_now is not None:
                            code = code_now
                            break
                        if now >= deadline:
                            exceeded = True
                            break
                        time.sleep(min(1., max(0., deadline - now)))
                finally:
                    if child.poll() is None:
                        child.terminate()
                        try:
                            child.wait(timeout=1.)
                        except subprocess.TimeoutExpired:
                            child.kill()
                            child.wait()
                    code = child.returncode
    ended = time.time()
    trace = Document.parse({'pid': child_pid, 'started': started, 'ended': ended,
        'deadline': deadline, 'deadline_exceeded': exceeded, 'exit_code': code, 'samples': samples})
    write(trace_path, dict(trace.values), exclusive=True)
    return MonitoredResult(code, started, ended, monitor_clean(trace))
