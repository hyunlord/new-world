#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Fresh unequal-source M6a paired performance; no inherited waivers."""
from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from typing import Sequence

from core_p7_admission import AdmissionError
from core_p7_ledger import ledger_text
from core_perf_check import (CEILINGS, SCENES, EvidenceError,
    Json, ROOT, loads, require, sequence, sha, write)
from m6a_evidence import reference_json as frozen_file

from m6a_perf_config import (BASE, StrictDocument as Document, freeze, reference as verify_file, verify_config)


def competing_processes(snapshot: str, owned_pid: int | None = None) -> list[str]:
    """Match executable names, never incidental text in shell arguments."""
    result: list[str] = []
    for line in snapshot.splitlines():
        fields = line.strip().split(maxsplit=1)
        if len(fields) < 2:
            continue
        if owned_pid is not None and fields[0] == str(owned_pid):
            continue
        executable = Path(fields[1]).name.lower()
        if executable.startswith('godot') or executable in ('cargo', 'rustc', 'worldsim_perf', 'worldsim_probe'):
            result.append(line.strip())
    return result


def process_snapshot() -> str:
    """Record host execution exclusivity around each repeat."""
    return subprocess.check_output(['ps', '-axo', 'pid=,comm='], text=True)


def failures(old: Document, new: Document) -> list[str]:
    """Use all standing1.10 and C3 bars without historical exceptions."""
    old_rows = {row.text('label'): row for row in old.rows('scenes')}
    new_rows = {row.text('label'): row for row in new.rows('scenes')}
    require(set(old_rows) == set(new_rows) == set(SCENES), 'scene coverage')
    result: list[str] = []
    for scene in SCENES:
        for metric in ('tick_mean_us', 'tick_p99_us'):
            before, after = old_rows[scene].number(metric), new_rows[scene].number(metric)
            require(before > 0 and after > 0, 'nonpositive timing')
            if after / before > 1.1:
                result.append(f'{scene}/{metric}>1.1')
    for metric in ('alloc_per_agent', 'alloc_per_tile'):
        before, after = old.number(metric), new.number(metric)
        require(before > 0 and after > 0, 'nonpositive allocator slope')
        if after / before > 1.1:
            result.append(metric + '>1.1')
    require(set(new.child('ceilings').values) == set(CEILINGS), 'C3 coverage')
    for name in CEILINGS:
        if new.child('ceilings').text(name) != 'PASS':
            result.append('standing C3 ' + name)
    return result


def check(out: Path, output: Path | None = None) -> int:
    """Delegate raw revalidation without starting a benchmark."""
    from m6a_perf_validate import check as check_raw
    return check_raw(out, output)


def admission(out: Path, label: str, deadline: float) -> tuple[str, tuple[float, float, float]]:
    """Wait at most eight15-minute windows for an exclusive quiet host."""
    for window in range(9):
        snapshot = process_snapshot()
        sample = loads(' '.join(str(value) for value in os.getloadavg()))
        busy = competing_processes(snapshot)
        now = time.time()
        allowed = not busy and sample[0] < 8 and now < deadline
        with (out / 'admission.jsonl').open('a') as stream:
            stream.write(json.dumps({'utc': now, 'label': label, 'window': window,
                'loads_1_5_15': list(sample), 'processes': busy,
                'status': 'ADMITTED' if allowed else 'NOT_RUN'}) + '\n')
        if allowed:
            return snapshot, sample
        if window == 8 or now + 900 >= deadline:
            raise AdmissionError('exclusive host not available within deadline/windows')
        until = now + 900
        while time.time() < until:
            print(json.dumps({'status': 'WAITING', 'label': label, 'window': window,
                'remaining_seconds': max(0, int(until - time.time()))}), flush=True)
            time.sleep(min(60, max(0, until - time.time())))
    raise AdmissionError('admission exhausted')


def run(config_path: Path, out: Path) -> int:
    """Capture complete interleaved pairs; preserve every invalid attempt."""
    config = Document.read(config_path)
    verify_config(config)
    deadline = config.number('deadline_epoch')
    out.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(config_path, out / 'config.json')
    for role in ('base', 'new'):
        (out / 'accepted' / role).mkdir(parents=True)
    (out / 'attempts').mkdir()
    records: list[Json] = []
    for build_only in (False, True):
        for repeat, scene in sequence(build_only):
            accepted = False
            for attempt in range(1, 4):
                pair: list[Json] = []
                for role in (('base', 'new') if repeat % 2 else ('new', 'base')):
                    label = f'{"rss" if build_only else "timed"}-{scene}-r{repeat}-a{attempt}-{role}'
                    verify_config(config)
                    try:
                        before, external = admission(out, label, deadline)
                    except AdmissionError as error:
                        write(out / 'attempts.json', records + pair)
                        write(out / 'receipt.json', {'status': 'PENDING', 'reason': str(error)})
                        return 75
                    verify_config(config)
                    raw = out / 'attempts' / (label + '.json')
                    before_path, after_path = (out / 'attempts' / (label + suffix) for suffix in ('-before.txt', '-after.txt'))
                    before_path.write_text(before)
                    argv = [str(verify_file(config.child(role))), 'bench-one', '--scene-label', scene,
                            '--out', str(raw), '--commands', str(verify_file(config.child('commands')))]
                    if build_only:
                        argv += ['--build-only', str(repeat)]
                    environment = os.environ.copy()
                    environment.pop('WORLDSIM_MIND_HISTORY_SATURATE', None)
                    from m6a_perf_validate import run_monitored
                    monitor_path = out / 'attempts' / (label + '-monitor.json')
                    result = run_monitored(argv, environment, out / 'attempts' / (label + '.log'), monitor_path, deadline)
                    started, ended = result.started, result.ended
                    after_path.write_text(process_snapshot())
                    verify_config(config)
                    internal = None
                    if result.returncode == 0 and raw.exists():
                        try:
                            internal = loads(Document.read(raw).text('load_before'))
                        except (EvidenceError, ValueError):
                            internal = None
                    valid = result.clean and internal is not None and internal[0] < 8 and external[0] < 8 and not competing_processes(after_path.read_text())
                    row: dict[str, Json] = {'role': role, 'scene': scene, 'repeat': repeat, 'attempt': attempt,
                        'build_only': build_only, 'started': started, 'ended': ended, 'external_load': list(external),
                        'internal_load': list(internal) if internal else None, 'exit_code': result.returncode,
                        'raw': str(raw), 'sha256': sha(raw) if raw.exists() else None,
                        'status': 'VALID' if valid else 'INVALID', 'pair_accepted': False,
                        'process_monitor': frozen_file(monitor_path),
                        'processes_before': frozen_file(before_path), 'processes_after': frozen_file(after_path)}
                    pair.append(row)
                accepted = all(Document.parse(row).text('status') == 'VALID' for row in pair)
                for value in pair:
                    row = dict(Document.parse(value).values)
                    row['pair_accepted'] = accepted
                    records.append(row)
                    if accepted:
                        role = Document.parse(row).text('role')
                        shutil.copyfile(Document.parse(row).text('raw'), out / 'accepted' / role / f'{"rss" if build_only else "timed"}-{scene}-r{repeat}.json')
                write(out / 'attempts.json', records)
                if accepted:
                    break
            if not accepted:
                write(out / 'receipt.json', {'status': 'PENDING', 'reason': 'three invalid pair attempts retained'})
                return 75
    for role in ('base', 'new'):
        subprocess.run([str(verify_file(config.child(role))), 'bench-summarise', '--runs', str(out / 'accepted' / role),
                        '--out', str(out / f'{role}-summary.json'), '--commit', config.text('base_head' if role == 'base' else 'candidate_head')], cwd=ROOT, check=True)
    (out / 'comparison-ledger.md').write_text(ledger_text(out / 'base-summary.json'))
    result = check(out)
    ledger = subprocess.run([sys.executable, str(ROOT / 'tools/v8/bench_ledger.py'), 'check',
        str(out / 'new-summary.json'), str(out / 'comparison-ledger.md'), '--pass', 'M6a'], check=False)
    return max(result, ledger.returncode)


def check_config(config: Path, out: Path) -> int:
    """Require the caller-pinned config and a complete existing session receipt."""
    require(sha(config) == sha(out / 'config.json'), 'session uses different config')
    from m6a_perf_validate import check as validate
    return validate(out, require_receipt=True)


def main(args: Sequence[str]) -> int:
    match list(args):
        case ['freeze', base_build, new_build, deadline, output]:
            freeze(Path(base_build).resolve(), Path(new_build).resolve(), int(deadline), Path(output).resolve())
            return 0
        case ['run', config, out]:
            return run(Path(config).resolve(), Path(out).resolve())
        case ['check', config, out]:
            return check_config(Path(config).resolve(), Path(out).resolve())
        case _:
            raise SystemExit('usage: freeze BASE_BUILD NEW_BUILD DEADLINE CONFIG | run CONFIG OUT | check CONFIG OUT')


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
