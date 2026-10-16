#!/usr/bin/env python3
"""Explicitly launched final gate; never claims or removes a quiet lock."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

GATE_OUTPUT = Path('docs/art/production10/gates/final-attempt2')
QUIET_LOCK = Path('/Users/rexxa/work/.mac_quiet.lock')
SOURCE_DIRS = ('scripts', 'scenes', 'shaders', 'assets', 'localization',
               'rust/crates/sim-test/tests', 'rust/crates/sim-bridge/src',
               'tools/art_pipeline', 'tools/harness')
SOURCE_FILES = ('project.godot', 'rust/Cargo.toml', 'rust/Cargo.lock')
BASE_ROWS = {'cargo_test', 'clippy', 'strict', 'bridge', 'smoke', 'smoke_errors', 'python'}


def digest(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def save(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def snapshot(root: Path) -> dict:
    files: set[Path] = set()
    for directory in SOURCE_DIRS:
        files.update(p for p in (root / directory).rglob('*') if p.is_file()
                     and '__pycache__' not in p.parts and p.suffix != '.pyc')
    files.update(root / name for name in SOURCE_FILES if (root / name).is_file())
    files.update((root / 'rust').glob('crates/*/Cargo.toml'))
    dylibs = sorted((root / 'rust/target/debug').glob('*sim_bridge*.dylib'))
    return {
        'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
        'source_sha256': {str(p.relative_to(root)): digest(p) for p in sorted(files)},
        'dylib_sha256': {str(p.relative_to(root)): digest(p) for p in dylibs},
        'gate_sha256': digest(root / 'tools/art_pipeline/gate.sh'),
    }


def validate_results(path: Path, expected: set[str], terminal: int) -> dict:
    errors: list[str] = []
    rows: dict[str, dict] = {}
    if not path.is_file():
        errors.append('Missing results.tsv')
    else:
        for number, line in enumerate(path.read_text().splitlines(), 1):
            parts = line.split('\t')
            if len(parts) != 3:
                errors.append(f'Malformed row {number}')
                continue
            name, code, seconds = parts
            try:
                status, elapsed = int(code), int(seconds)
            except ValueError:
                errors.append(f'Noninteger row {number}')
                continue
            if name in rows:
                errors.append(f'Duplicate row: {name}')
            rows[name] = {'exit_code': status, 'seconds': elapsed}
            if status != 0 or elapsed < 0:
                errors.append(f'Failed or invalid row: {name}')
    if set(rows) != expected:
        errors.append(f'Missing rows: {sorted(expected - set(rows))}; unexpected rows: {sorted(set(rows) - expected)}')
    if terminal != 0:
        errors.append(f'Gate terminal exit: {terminal}')
    return {'valid': not errors, 'errors': errors, 'rows': rows, 'terminal_exit': terminal}


def source_stable(before: dict, after: dict) -> bool:
    return all(before[key] == after[key] for key in ('head', 'source_sha256', 'gate_sha256'))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('checkout', type=Path)
    parser.add_argument('evidence', type=Path, help='NEW external supervisor output directory')
    parser.add_argument('--run', action='store_true', required=True, help='Explicitly execute the full gate')
    args = parser.parse_args()
    root, out = args.checkout.resolve(), args.evidence.resolve()
    gate_out = root / GATE_OUTPUT
    if out.exists() or gate_out.exists():
        parser.error('Refusing existing supervisor or docs final gate output')
    out.mkdir(parents=True, exist_ok=False)
    command = ['bash', 'tools/art_pipeline/gate.sh', 'final']
    env = os.environ.copy()
    env.update(CARGO_BUILD_JOBS='6', CARGO_PROFILE_TEST_OPT_LEVEL='2', GATE_OUTPUT_DIR=str(GATE_OUTPUT))
    save(out / 'invocation.json', {'checkout': str(root), 'command': command,
         'environment': {key: env[key] for key in ('CARGO_BUILD_JOBS', 'CARGO_PROFILE_TEST_OPT_LEVEL', 'GATE_OUTPUT_DIR')},
         'supervisor_sha256': digest(Path(__file__)), 'quiet_lock': str(QUIET_LOCK),
         'quiet_poll_seconds': 300, 'deadline': None})
    terminal = None
    try:
        while QUIET_LOCK.exists():
            with (out / 'quiet-wait.jsonl').open('a') as stream:
                stream.write(json.dumps({'epoch': time.time(), 'status': 'WAIT_FOREIGN_QUIET'}) + '\n')
            time.sleep(300)
        # Parent must serialize launches: absence of a lock is not a lock acquisition.
        if gate_out.exists():
            raise FileExistsError(f'Refusing existing final gate output: {gate_out}')
        before = snapshot(root)
        if not before['dylib_sha256']:
            raise RuntimeError('Missing pre-gate debug sim_bridge dylib')
        save(out / 'before.json', before)
        expected = BASE_ROWS | {p.stem for p in (root / 'scripts/test/art').glob('test_*.gd')}
        save(out / 'expected-rows.json', sorted(expected))
        if QUIET_LOCK.exists():
            raise RuntimeError('Quiet lock appeared during provenance capture; no gate launched')
        gate_out.mkdir(parents=True, exist_ok=False)
        with (out / 'gate.log').open('wb') as log:
            terminal = subprocess.run(command, cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT, check=False).returncode
        (out / 'gate.exit').write_text(f'{terminal}\n')
        after = snapshot(root)
        save(out / 'after.json', after)
        result = validate_results(gate_out / 'results.tsv', expected, terminal)
        missing_logs = sorted(name for name in expected if not (gate_out / f'{name}.log').is_file())
        if missing_logs:
            result['errors'].append(f'Missing raw gate logs: {missing_logs}')
            result['valid'] = False
        result.update(source_stable=source_stable(before, after),
                      dylib_changed=before['dylib_sha256'] != after['dylib_sha256'],
                      dylib_before=before['dylib_sha256'], dylib_after=after['dylib_sha256'])
        save(out / 'gate-artifacts-sha256.json', {str(p.relative_to(root)): digest(p) for p in sorted(gate_out.rglob('*')) if p.is_file()})
        result['changed_source_paths'] = sorted(key for key in before['source_sha256'].keys() | after['source_sha256'].keys() if before['source_sha256'].get(key) != after['source_sha256'].get(key))
        result['valid'] = result['valid'] and result['source_stable'] and bool(after['dylib_sha256'])
        result['status'] = 'PASS' if result['valid'] else 'NOT_VERIFIED_OR_FAILED'
        save(out / 'result.json', result)
        return 0 if result['valid'] else 1
    except (Exception, KeyboardInterrupt) as error:
        save(out / 'supervisor-error.json', {'error': str(error), 'terminal_exit': terminal, 'status': 'NOT_VERIFIED'})
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
