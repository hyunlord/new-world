#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Fresh paired performance for an unchanged-Rust UI successor; no waivers."""
from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from typing import Final, Sequence

from core_p7_admission import AdmissionError
from core_p7_ledger import ledger_text
from core_perf_check import (CEILINGS, SCENES, Document, EvidenceError,
    Json, ROOT, frozen_file, git, loads, require, sequence, sha,
    source_hash, verify_file, write)

BASE: Final = '3814828c06d1a7a211b13b54368e4e0a488cc0e3'
DEPENDENCIES: Final = ('core_ui_successor_perf.py', 'core_perf_check.py',
    'core_p7_admission.py', 'core_p7_ledger.py', 'bench_ledger.py',
    'core_ui_successor_perf_validate.py')


def competing_processes(snapshot: str) -> list[str]:
    """Match executable names, never incidental text in shell arguments."""
    result: list[str] = []
    for line in snapshot.splitlines():
        fields = line.strip().split(maxsplit=1)
        if len(fields) < 2:
            continue
        executable = Path(fields[1]).name.lower()
        if executable.startswith('godot') or executable in ('cargo', 'rustc'):
            result.append(line.strip())
    return result


def process_snapshot() -> str:
    """Record host execution exclusivity around each repeat."""
    return subprocess.check_output(['ps', '-axo', 'pid=,comm='], text=True)


def verify_config(config: Document) -> None:
    """Bind the new published BASE, unchanged Rust and explicit build receipts."""
    require(config.text('kind') == 'ui_successor_fresh_performance_v1', 'wrong config kind')
    require(config.text('base_head') == BASE, 'wrong published BASE')
    head = config.text('candidate_head')
    subprocess.run(['git', '-C', str(ROOT), 'merge-base', '--is-ancestor', head, 'HEAD'], check=True)
    require(git('rev-parse', head + ':rust') == git('rev-parse', BASE + ':rust')
            == git('rev-parse', 'HEAD:rust'), 'UI successor changed Rust')
    require(config.text('candidate_source_sha256') == source_hash(), 'live Rust changed')
    require(config.text('cargo_lock_sha256') == sha(ROOT / 'rust/Cargo.lock'), 'lock changed')
    verify_file(config.child('commands'))
    require(config.child('commands').text('sha256') == sha(ROOT / 'tools/v8/bench_script.json'), 'commands differ')
    deps = config.rows('dependencies')
    require({Path(d.text('path')).name for d in deps} == set(DEPENDENCIES) and len(deps) == len(DEPENDENCIES), 'dependency coverage')
    for dependency in deps:
        require(verify_file(dependency) == ROOT / 'tools/v8' / Path(dependency.text('path')).name, 'dependency location')
    require(config.values.get('identical_runtime_binary') is (config.child('base').text('sha256') == config.child('new').text('sha256')), 'identical binary disclosure mismatch')
    for role, revision in (('base', BASE), ('new', head)):
        binary = verify_file(config.child(role))
        proof = Document.read(verify_file(config.child(role + '_build')))
        build_head = proof.text('source_sha')
        subprocess.run(['git', '-C', str(ROOT), 'merge-base', '--is-ancestor', build_head, 'HEAD'], check=True)
        require(git('rev-parse', build_head + ':rust') == git('rev-parse', revision + ':rust'), 'build source tree mismatch')
        require(proof.text('build_profile') == 'release' and proof.number('exit_code') == 0, 'build profile/exit mismatch')
        require(proof.text('binary_sha256') == sha(binary), 'build binary mismatch')
        require(proof.text('rust_source_sha256') == source_hash()
                and proof.text('cargo_lock_sha256') == config.text('cargo_lock_sha256'), 'build source/lock mismatch')


def freeze(base: Path, candidate: Path, base_build: Path, new_build: Path, output: Path) -> None:
    """Pin external build records; compiler provenance remains explicit."""
    config: dict[str, Json] = {'kind': 'ui_successor_fresh_performance_v1', 'base_head': BASE,
        'candidate_head': git('rev-parse', 'HEAD'), 'candidate_source_sha256': source_hash(),
        'cargo_lock_sha256': sha(ROOT / 'rust/Cargo.lock'),
        'base': frozen_file(base), 'new': frozen_file(candidate),
        'identical_runtime_binary': sha(base) == sha(candidate),
        'base_build': frozen_file(base_build), 'new_build': frozen_file(new_build),
        'commands': frozen_file(ROOT / 'tools/v8/bench_script.json'),
        'dependencies': [frozen_file(ROOT / 'tools/v8' / name) for name in DEPENDENCIES]}
    verify_config(Document.parse(config))
    write(output, config, exclusive=True)


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
    before, after = old.number('alloc_per_agent'), new.number('alloc_per_agent')
    require(before > 0 and after > 0, 'nonpositive allocator slope')
    if after / before > 1.1:
        result.append('alloc_per_agent>1.1')
    require(set(new.child('ceilings').values) == set(CEILINGS), 'C3 coverage')
    for name in CEILINGS:
        if new.child('ceilings').text(name) != 'PASS':
            result.append('standing C3 ' + name)
    return result


def check(out: Path, output: Path | None = None) -> int:
    """Delegate raw revalidation without starting a benchmark."""
    from core_ui_successor_perf_validate import check as check_raw
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
        time.sleep(900)
    raise AdmissionError('admission exhausted')


def run(config_path: Path, out: Path) -> int:
    """Capture complete interleaved pairs; preserve every invalid attempt."""
    config = Document.read(config_path)
    verify_config(config)
    deadline = float(os.environ['CORE_UI_DEADLINE_EPOCH'])
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
                    from core_ui_successor_perf_validate import run_monitored
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
        str(out / 'new-summary.json'), str(out / 'comparison-ledger.md'), '--pass', 'M-UX2.1'], check=False)
    return max(result, ledger.returncode)


def main(args: Sequence[str]) -> int:
    """Positional CLI preserves the repository's dependency-free gate tooling."""
    match list(args):
        case ['freeze', base, candidate, base_build, new_build, output]:
            freeze(Path(base).resolve(), Path(candidate).resolve(), Path(base_build).resolve(), Path(new_build).resolve(), Path(output).resolve())
            return 0
        case ['run', config, out]:
            return run(Path(config).resolve(), Path(out).resolve())
        case ['check', out]:
            return check(Path(out).resolve())
        case ['check', out, output]:
            return check(Path(out).resolve(), Path(output).resolve())
        case _:
            raise SystemExit('usage: freeze BASE_EXE NEW_EXE BASE_BUILD NEW_BUILD CONFIG | run CONFIG SESSION | check SESSION [OUTPUT]')


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
