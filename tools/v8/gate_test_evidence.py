#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/v8/gate_test_evidence.py --help
"""Complete frozen debug-suite receipts, created only after successful pregate step 4."""
import argparse
from datetime import datetime, timezone
import json
import math
import os
from pathlib import Path
import sys
import subprocess
import tempfile

from gate_test_inputs import capture
from gate_test_log import parse_log
from mux2_gate_inputs import Json, digest, external, require


def atomic_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    handle, temporary = tempfile.mkstemp(prefix='.' + path.name, dir=path.parent)
    try:
        with os.fdopen(handle, 'w') as stream:
            json.dump(value, stream, sort_keys=True, indent=2, allow_nan=False)
            stream.write('\n')
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def read(path: Path) -> dict[str, Json]:
    def unique(pairs: list[tuple[str, Json]]) -> dict[str, Json]:
        result = {}
        for key, value in pairs:
            require(key not in result, 'duplicate JSON key')
            result[key] = value
        return result
    value = json.loads(path.read_text(), object_pairs_hook=unique)
    require(isinstance(value, dict), 'expected JSON object')
    return value


def create(root: Path, before: Path, log: Path, receipt: Path, exit_code: int,
           wall_seconds: float) -> dict[str, object]:
    receipt = external(receipt, root)
    log = external(log, root)
    require(exit_code == 0, 'debug command failed')
    require(math.isfinite(wall_seconds) and wall_seconds > 0, 'invalid wall time')
    inputs = read(before)
    require(inputs == capture(root), 'inputs changed during debug suite')
    sha = digest(log)
    summary = parse_log(log.read_text(), inputs['inventory'])
    require(digest(log) == sha, 'log changed during parse')
    require(wall_seconds + 2 >= summary['compile_seconds'] + summary['run_seconds'],
            'wall time shorter than complete suite')
    require(inputs == capture(root), 'inputs changed during receipt creation')
    value = {'schema': 1, 'kind': 'complete-debug-suite', 'complete': True, 'exit_code': 0,
             'created_utc': datetime.now(timezone.utc).isoformat(), 'inputs': inputs,
             'log': str(log), 'log_sha256': sha, 'wall_seconds': wall_seconds, 'summary': summary}
    atomic_json(receipt, value)
    return value


def verify(root: Path, receipt: Path) -> str:
    receipt = external(receipt, root)
    sha = digest(receipt)
    value = read(receipt)
    require(type(value.get('schema')) is int and value['schema'] == 1
            and value.get('kind') == 'complete-debug-suite'
            and value.get('complete') is True and type(value.get('exit_code')) is int
            and value['exit_code'] == 0, 'incomplete/unsupported receipt')
    wall = value['wall_seconds']
    require(type(wall) in (int, float) and math.isfinite(wall) and wall > 0, 'invalid wall time')
    require(value['inputs'] == capture(root), 'receipt input mismatch')
    log = external(Path(value['log']), root)
    require(digest(log) == value['log_sha256'], 'missing/changed suite log')
    summary = parse_log(log.read_text(), value['inputs']['inventory'])
    require(summary == value['summary'], 'receipt summary mismatch')
    require(wall + 2 >= summary['compile_seconds'] + summary['run_seconds'], 'invalid suite wall time')
    require(digest(log) == value['log_sha256'] and digest(receipt) == sha,
            'evidence changed during verification')
    require(value['inputs'] == capture(root), 'inputs changed during verification')
    return (f'PASS_REUSED {receipt} {sha} {wall:g}\n'
            f'test_debug passed {summary["passed"]} failed 0 ignored {summary["ignored"]}')


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    for name in ('snapshot', 'create', 'verify'):
        command = commands.add_parser(name)
        command.add_argument('--root', type=Path, required=True)
        if name == 'snapshot':
            command.add_argument('--out', type=Path, required=True)
        else:
            command.add_argument('--receipt', type=Path, required=True)
        if name == 'create':
            command.add_argument('--before', type=Path, required=True)
            command.add_argument('--log', type=Path, required=True)
            command.add_argument('--exit-code', type=int, required=True)
            command.add_argument('--wall-seconds', type=float, required=True)
    summary = commands.add_parser('summary')
    summary.add_argument('--log', type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == 'snapshot':
            atomic_json(external(args.out, args.root), capture(args.root))
        elif args.command == 'create':
            create(args.root, args.before, args.log, args.receipt, args.exit_code, args.wall_seconds)
        elif args.command == 'verify':
            print(verify(args.root, args.receipt))
        else:
            result = parse_log(args.log.read_text())
            print(f'test_debug passed {result["passed"]} failed 0 ignored {result["ignored"]}')
        return 0
    except (ValueError, OSError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(f'debug evidence rejected: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
