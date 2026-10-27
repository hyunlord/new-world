#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/v8/mux2_gate_receipts.py --help
"""Capture or verify serialized GPU evidence, bound to source content and binary."""
import argparse
from datetime import datetime, timezone
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import time
from typing import Final, Literal, TypeAlias, assert_never
from mux2_gate_inputs import (BRIDGE, EvidenceError, Json, Node, check_hash, clean_sources,
                             digest, external, git, read_json, require, same_inputs, snapshot,
                             validate_capture_status)
from mux2_gate_stall import validate_stall
from mux2_gate_lock import FD_ENV, lane
from mux2_gate_visual import validate_visibility, validate_why

Kind: TypeAlias = Literal['stall', 'visibility', 'why']
KINDS: Final = ('stall', 'visibility', 'why')


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


def write(path: Path, value: Json) -> None:
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value, indent=2, allow_nan=False) + '\n')
    temporary.replace(path)


def stall_timeout() -> int:
    if 'CORE_P7_RUN' not in os.environ:
        return 960
    deadline = float(os.environ.get('CORE_P7_DEADLINE_EPOCH', 'nan'))
    require(math.isfinite(deadline), 'NOT_RUN: P7 requires finite CORE_P7_DEADLINE_EPOCH')
    remaining = math.floor(deadline - time.time())
    require(remaining >= 650, 'NOT_RUN: P7 deadline cannot fit both stall arms')
    return min(960 + 2 * 8 * 900, remaining)


def validate_p7_admission(report: Node) -> None:
    policy = report.field('p7_admission')
    deadline = policy.field('deadline_epoch').number()
    require(policy.field('wait_seconds').number() == 900, 'P7 wait policy must be 900 seconds')
    policy.field('max_waits').equals(8)
    require(policy.field('load1_limit_exclusive').number() == 8, 'P7 load policy must be below 8')
    for arm in report.field('arms').array():
        admission = arm.field('admission')
        admission.field('status').equals('ADMITTED')
        samples = admission.field('samples').array()
        require(1 <= len(samples) <= 9, 'P7 admission sample count')
        for index, sample in enumerate(samples):
            loads = [value.number() for value in sample.field('loads_1_5_15').array()]
            require(len(loads) == 3 and all(value >= 0 for value in loads), 'P7 load triplet')
            sample.field('waits').equals(index)
            sample.field('sysctl_exit').equals(0)
            final = index == len(samples) - 1
            sample.field('action').equals('ADMITTED' if final else 'WAIT')
            require(loads[0] < 8 if final else loads[0] >= 8, 'P7 admission load threshold')
            if index:
                require(sample.field('epoch').number() - samples[index - 1].field('epoch').number() >= 900,
                        'P7 admission windows shorter than 900 seconds')
        admitted = samples[-1].field('epoch').number()
        started = arm.field('sampling_started_epoch').number()
        require(admitted <= started < deadline, 'P7 admission/start ordering')


def commands(root: Path, output: Path, kind: Kind, godot: Path) -> list[list[str]]:
    base = ['bash', str(root / 'tools/v8/run_bounded.sh')]
    engine = [str(godot), '--path', str(root), '--resolution', '1920x1080']
    match kind:
        case 'stall':
            return [base + [str(stall_timeout())] + engine + ['--always-on-top', '--windowed', '--script', 'res://tools/v8/mux2_stall_test.gd',
                                              '--', str(output / 'run')]]
        case 'visibility':
            return [base + ['1800'] + engine + ['--headless', '--script',
                    'res://tools/v8/mux2_visibility_saves.gd', '--', str(output / 'saves')],
                    base + ['1200'] + engine + ['--script', 'res://tools/v8/mux2_visibility_test.gd',
                    '--', '--manifest', str(output / 'saves/manifest.json'), '--out', str(output / 'run')]]
        case 'why':
            return [base + ['180'] + engine + ['--script', 'res://tools/v8/mux2_why_input_test.gd']]
        case unreachable:
            assert_never(unreachable)


def validate_output(root: Path, out: Path, kind: Kind) -> list[Path]:
    log = out / 'command.log'
    require('SCRIPT ERROR' not in log.read_text(), 'capture script error')
    extra: list[Path] = []
    match kind:
        case 'stall':
            report = read_json(out / 'run/stall-report.json')
            validate_stall(report.value)
            if 'CORE_P7_RUN' in os.environ or 'p7_admission' in report.mapping():
                validate_p7_admission(report)
            report.field('bridge').field('sha256').equals(digest(root / BRIDGE))
            for name, value in report.field('source_sha256').mapping().items():
                check_hash(root / name, Node(value).text())
        case 'visibility':
            validate_visibility(root, out)
        case 'why':
            extra = validate_why(log)
        case unreachable:
            assert_never(unreachable)
    return extra


def capture(root: Path, output: Path, kind: Kind, godot: Path) -> None:
    out = external(output, root)
    require(not out.exists(), 'capture output must be new')
    clean_sources(root)
    require(godot.is_file(), 'Godot executable missing')
    out.mkdir(parents=True)
    with lane() as descriptor:
        before = snapshot(root)
        receipt: dict[str, Json] = {'schema': 1, 'kind': kind, 'complete': False,
            'exit_code': -1, 'started_utc': now(), 'source_head': git(root, 'rev-parse', 'HEAD'),
            'root': str(root), 'inputs': before, 'godot': str(godot),
            'godot_sha256': digest(godot), 'artifacts': {}}
        receipt_path = out / 'receipt.json'
        write(receipt_path, receipt)
        env = dict(os.environ, WORLDSIM_TEST_SETTINGS_DIR=str(out / 'settings'),
                   WORLDSIM_MUX2_PROFILE='1', **{FD_ENV: str(descriptor)})
        (out / 'settings').mkdir()
        code = -1
        try:
            with (out / 'command.log').open('w') as log:
                for command in commands(root, out, kind, godot):
                    log.write(json.dumps(command) + '\n')
                    log.flush()
                    code = subprocess.run(command, cwd=root, env=env, stdout=log,
                                          stderr=subprocess.STDOUT, pass_fds=(descriptor,), check=False).returncode
                    require(code == 0, f'{kind} capture exited {code}')
            same_inputs(before, snapshot(root))
            clean_sources(root)
            require(git(root, 'rev-parse', 'HEAD') == receipt['source_head'], 'commit changed during capture')
            check_hash(godot, Node(receipt['godot_sha256']).text())
            extra = validate_output(root, out, kind)
            artifacts = [p for p in out.rglob('*') if p.is_file() and p != receipt_path]
            receipt['artifacts'] = {str(p): digest(p) for p in sorted(set(artifacts + extra))}
            receipt['complete'] = True
        finally:
            receipt['exit_code'] = code
            receipt['finished_utc'] = now()
            receipt['inputs_end'] = snapshot(root)
            write(receipt_path, receipt)
    print(f'MUX2_CAPTURE_PASS {receipt_path}')


def verify(root: Path, receipt_path: Path, kind: Kind) -> None:
    path = external(receipt_path, root)
    receipt = read_json(path)
    validate_capture_status(receipt.value, kind)
    receipt.field('root').equals(str(root))
    clean_sources(root)
    before = receipt.field('inputs').mapping()
    after = receipt.field('inputs_end').mapping()
    same_inputs(before, after)
    same_inputs(before, snapshot(root))
    head = receipt.field('source_head').text()
    subprocess.run(['git', '-C', str(root), 'merge-base', '--is-ancestor', head, 'HEAD'], check=True)
    check_hash(Path(receipt.field('godot').text()), receipt.field('godot_sha256').text())
    artifacts = receipt.field('artifacts').mapping()
    require(bool(artifacts), 'missing raw artifact hashes')
    for name, value in artifacts.items():
        check_hash(external(Path(name), root), Node(value).text())
    extra = validate_output(root, path.parent, kind)
    actual = [p for p in path.parent.rglob('*') if p.is_file() and p != path]
    same_inputs(artifacts, {str(p): digest(p) for p in sorted(set(actual + extra))})
    print(f'MUX2_RECEIPT_PASS kind={kind} capture_head={head} gate_head={git(root, "rev-parse", "HEAD")}')


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=('capture', 'verify'))
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument('--kind', choices=KINDS, required=True)
    parser.add_argument('--out', type=Path)
    parser.add_argument('--receipt', type=Path)
    parser.add_argument('--godot', type=Path,
                        default=Path(os.environ.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot')))
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        match args.action:
            case 'capture':
                require(args.out is not None, '--out required')
                capture(root, args.out, args.kind, args.godot.resolve())
            case 'verify':
                require(args.receipt is not None, '--receipt required')
                verify(root, args.receipt, args.kind)
            case unreachable:
                assert_never(unreachable)
        return 0
    except (EvidenceError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'MUX2_RECEIPT_FAIL {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
