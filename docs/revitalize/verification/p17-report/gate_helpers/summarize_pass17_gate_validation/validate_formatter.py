#!/usr/bin/env python3
from __future__ import annotations

import ast
import copy
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
FORMATTER = Path('/Users/rexxa/work/p17-data/summarize_pass17_gate.py')
EXPECTED_SHA = 'da035c56e5b052fe6d9a5d26ec95d3f14b53ae4303bd18d66d4413b88d0e1874'
CHECKER = Path('/Users/rexxa/.codex/plugins/cache/sisyphuslabs/omo/4.13.0/skills/programming/scripts/python/check-no-excuse-rules.py')


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def utc() -> str:
    return datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z')


def dump(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + '\n')


def main() -> int:
    if len(sys.argv) != 2:
        print('usage: validate_formatter.py NEW_EVIDENCE_DIRECTORY', file=sys.stderr)
        return 2
    target = Path(sys.argv[1]).resolve()
    target.mkdir(parents=False, exist_ok=False)
    started = utc()
    source = FORMATTER.read_bytes()
    if sha(source) != EXPECTED_SHA:
        raise RuntimeError('formatter differs from source that passed the original transient checks')
    (target / 'formatter_snapshot.py').write_bytes(source)
    spec = importlib.util.spec_from_file_location('formatter_under_test', FORMATTER)
    if spec is None or spec.loader is None:
        raise RuntimeError('cannot load formatter')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    root = target / 'fixture'
    repo, out = root / 'repo', root / 'gate'
    repo.mkdir(parents=True)
    out.mkdir()
    receipt = root / 'receipt.json'
    manifest = root / 'receipt.json.sha256'
    head = 'a' * 40
    lines = ['head ' + head, 'started 2026-10-15 09:00:01 KST']
    for name in module.STEPS:
        lines.append(name + ' exit 0')
        if name in ('test_debug', 'test_release'):
            lines.append(name + ' passed 2200 failed 0 ignored 0')
        if name == 'smoke':
            lines.append('smoke_errors none')
        if name == 'python_unittest':
            lines.append('python_unittest ran 240 skipped 0 result OK')
        if name == 'python_worldmodel':
            lines.append('python_worldmodel ran 49 skipped 24 result OK (skipped=24)')
        if name == 'perf_quick':
            lines.append('quick mean_ms 0.5 p999_ms 2.5 max_ms 8.0')
        (out / (name + '.log')).write_text('synthetic log\n')
    lines.append('finished 2026-10-15 09:00:10 KST status 0')
    (out / 'smoke_errors.log').write_bytes(b'')
    (out / 'launcher.log').write_text('synthetic launcher\n')
    epoch = datetime(2026, 10, 15, tzinfo=timezone.utc).timestamp()
    snapshot = {'head': head, 'staged': [], 'unstaged': [], 'untracked': [], 'ignored_source': []}
    script = {'path': str(repo / 'tools/revitalize/gate_pass17.sh'), 'sha256': 'b' * 64, 'bytes': 5000}
    base = {
        'schema_version': 1, 'status': 'PASS', 'passed': True, 'expected_head': head,
        'repo': str(repo), 'output_directory': str(out), 'command': ['bash', script['path'], str(out)],
        'started_utc': '2026-10-15T00:00:00Z', 'started_epoch': epoch,
        'finished_utc': '2026-10-15T00:00:11Z', 'finished_epoch': epoch + 11, 'elapsed_seconds': 11,
        'gate_started_utc': '2026-10-15T00:00:00.500000Z', 'gate_started_epoch': epoch + .5,
        'gate_finished_utc': '2026-10-15T00:00:10.500000Z', 'gate_finished_epoch': epoch + 10.5,
        'gate_exit_code': 0, 'wrapper_signal': None, 'gate_launched': True,
        'before': snapshot, 'after': copy.deepcopy(snapshot),
        'gate_script_before': script, 'gate_script_after': copy.deepcopy(script),
        'expected_steps': list(module.STEPS), 'errors': [], 'artifacts': [],
        'sha_manifest': str(manifest), 'receipt': str(receipt), 'perf_quick_is_timing_evidence': False,
    }
    results: list[dict[str, object]] = []
    inputs: list[dict[str, object]] = []
    messages: list[str] = []

    def write_fixture(receipt_change=lambda value: None, lines_change=lambda value: None) -> None:
        current, gate_lines = copy.deepcopy(base), list(lines)
        receipt_change(current)
        lines_change(gate_lines)
        (out / 'gate.txt').write_text('\n'.join(gate_lines) + '\n')
        (out / 'smoke_errors.log').write_bytes(b'')
        (out / 'clippy.log').write_text('synthetic log\n')
        current['artifacts'] = [
            {'path': str(path), 'bytes': len(path.read_bytes()), 'sha256': sha(path.read_bytes())}
            for path in sorted(out.iterdir()) if path.is_file()
        ]
        dump(receipt, current)
        manifest.write_text(''.join(
            item['sha256'] + '  ' + item['path'] + '\n' for item in current['artifacts']
        ) + sha(receipt.read_bytes()) + '  ' + str(receipt) + '\n')

    def capture(name: str) -> dict[str, str]:
        observed = {'receipt': receipt, 'manifest': manifest, 'gate': out / 'gate.txt',
                    'smoke_errors': out / 'smoke_errors.log', 'clippy': out / 'clippy.log'}
        payload = {key: path.read_text() for key, path in observed.items()}
        hashes = {key + '_sha256': sha(path.read_bytes()) for key, path in observed.items()}
        inputs.append({'case': name, 'sha256': hashes, 'content': payload})
        return hashes

    def record(name: str, passed: bool, expected: str, actual: dict[str, object], hashes: dict[str, str]) -> None:
        item = {'number': len(results) + 1, 'name': name, 'status': 'PASS' if passed else 'FAIL',
                'expected': expected, 'actual': actual, 'input_sha256': hashes}
        results.append(item)
        messages.append(f"{item['number']:02d} {item['status']} {name}: {json.dumps(actual, ensure_ascii=False)}")

    def reject(name: str, receipt_change=lambda value: None, lines_change=lambda value: None,
               after=lambda: None) -> None:
        write_fixture(receipt_change, lines_change)
        after()
        hashes = capture(name)
        try:
            module.summarize(receipt)
        except (module.GateSummaryError, ValueError, KeyError, TypeError, OverflowError) as error:
            record(name, True, 'reject', {'exception': type(error).__name__, 'message': str(error)}, hashes)
        else:
            record(name, False, 'reject', {'returned': 'PASS'}, hashes)

    def replace(value: list[str], old: str, new: str) -> None:
        value[value.index(old)] = new

    def cli(json_path: Path, md_path: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run([sys.executable, str(FORMATTER), str(receipt), str(json_path), str(md_path)],
                              text=True, capture_output=True, check=False)

    def cli_actual(result: subprocess.CompletedProcess[str]) -> dict[str, object]:
        return {'command': result.args, 'exit_code': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr}

    write_fixture()
    summary = module.summarize(receipt)
    valid = (summary['tests']['test_debug']['passed'] == 2200
             and summary['tests']['python_worldmodel']['executed'] == 25
             and summary['log_interval']['started_utc'] == '2026-10-15T00:00:01Z'
             and summary['invocation_interval']['elapsed_seconds'] == 10.0)
    record('valid_summary', valid, 'accept with exact counts and UTC conversion',
           {'status': summary['status'], 'tests': summary['tests'], 'log_interval': summary['log_interval'],
            'invocation_interval': summary['invocation_interval']}, capture('valid_summary'))
    reject('empty_receipt', after=lambda: receipt.write_bytes(b''))
    reject('pending_receipt', lambda value: value.update(status='PENDING'))
    reject('failed_receipt', lambda value: value.update(status='FAIL', passed=False))
    reject('pass_with_false_passed', lambda value: value.update(passed=False))
    reject('missing_finished_utc', lambda value: value.pop('finished_utc'))
    reject('before_head_mismatch', lambda value: value['before'].update(head='c' * 40))
    reject('dirty_after_source', lambda value: value['after']['unstaged'].append('rust/a.rs'))
    reject('gate_epoch_mismatch', lambda value: value.update(gate_started_epoch=epoch + 20))
    reject('elapsed_mismatch', lambda value: value.update(elapsed_seconds=12))
    reject('changed_script_sha', lambda value: value['gate_script_after'].update(sha256='c' * 64))
    reject('different_gate_command', lambda value: value.update(command=['bash', 'other.sh', str(out)]))
    reject('boolean_gate_exit', lambda value: value.update(gate_exit_code=False))
    reject('quick_claims_timing', lambda value: value.update(perf_quick_is_timing_evidence=True))
    reject('gate_head_mismatch', lines_change=lambda value: replace(value, 'head ' + head, 'head ' + 'c' * 40))
    reject('missing_step', lines_change=lambda value: value.remove('clippy exit 0'))
    reject('duplicate_step', lines_change=lambda value: value.insert(3, 'clippy exit 0'))
    reject('failed_step', lines_change=lambda value: replace(value, 'clippy exit 0', 'clippy exit 1'))
    reject('reordered_step', lines_change=lambda value: (value.remove('clippy exit 0'), value.insert(3, 'clippy exit 0')))
    reject('malformed_step', lines_change=lambda value: value.insert(3, 'clippy exit bogus'))
    reject('smoke_error_marker', lines_change=lambda value: replace(value, 'smoke_errors none', 'smoke_errors found 1'))
    reject('nonzero_finished_status', lines_change=lambda value: replace(value, lines[-1], 'finished 2026-10-15 09:00:10 KST status 1'))
    reject('duplicate_finished_marker', lines_change=lambda value: value.append(value[-1]))
    reject('trailing_after_finished', lines_change=lambda value: value.append('unexpected'))
    reject('failed_debug_totals', lines_change=lambda value: replace(value, 'test_debug passed 2200 failed 0 ignored 0', 'test_debug passed 2199 failed 1 ignored 0'))
    reject('missing_release_totals', lines_change=lambda value: value.remove('test_release passed 2200 failed 0 ignored 0'))
    reject('malformed_extra_debug_totals', lines_change=lambda value: value.insert(3, 'test_debug passed invalid'))
    reject('missing_python_counts', lines_change=lambda value: value.remove('python_unittest ran 240 skipped 0 result OK'))
    reject('inconsistent_python_skips', lines_change=lambda value: replace(value, 'python_worldmodel ran 49 skipped 24 result OK (skipped=24)', 'python_worldmodel ran 49 skipped 24 result OK (skipped=23)'))
    reject('nonfinite_quick_sample', lines_change=lambda value: replace(value, 'quick mean_ms 0.5 p999_ms 2.5 max_ms 8.0', 'quick mean_ms NaN p999_ms 2.5 max_ms 8.0'))
    reject('mutated_smoke_file', after=lambda: (out / 'smoke_errors.log').write_text('ERROR'))
    reject('mutated_artifact', after=lambda: (out / 'clippy.log').write_text('modified'))
    reject('manifest_missing_receipt', after=lambda: manifest.write_text('\n'.join(manifest.read_text().splitlines()[:-1]) + '\n'))
    reject('duplicate_manifest_entry', after=lambda: manifest.write_text(manifest.read_text() + manifest.read_text().splitlines()[0] + '\n'))
    write_fixture()
    json_output, md_output = target / 'summary.json', target / 'summary.md'
    hashes = capture('cli_valid_outputs')
    result = cli(json_output, md_output)
    valid = (result.returncode == 0 and json_output.is_file() and md_output.is_file()
             and 'R5/F8' in md_output.read_text() and sha(json_output.read_bytes()) in md_output.read_text())
    record('cli_valid_outputs', valid, 'exit0; JSON SHA and quick disclaimer in Markdown', cli_actual(result), hashes)
    saved_json = json_output.read_bytes() if json_output.exists() else b''
    hashes = capture('cli_output_collision')
    result = cli(json_output, md_output)
    record('cli_output_collision', result.returncode != 0 and json_output.read_bytes() == saved_json,
           'nonzero; preserve existing JSON', cli_actual(result), hashes)
    forbidden_json = out / 'summary.json'
    hashes = capture('cli_gate_output_forbidden')
    result = cli(forbidden_json, target / 'forbidden_summary.md')
    record('cli_gate_output_forbidden', result.returncode != 0 and not forbidden_json.exists(),
           'nonzero; no output in gate artifacts', cli_actual(result), hashes)
    write_fixture(lambda value: value.update(status='PENDING'))
    pending_json = target / 'pending.json'
    hashes = capture('cli_pending_no_outputs')
    result = cli(pending_json, target / 'pending.md')
    record('cli_pending_no_outputs', result.returncode != 0 and not pending_json.exists(),
           'nonzero; no output from pending evidence', cli_actual(result), hashes)
    write_fixture()
    final_fixture = module.summarize(receipt)
    ast.parse(source, filename=str(FORMATTER))
    static = subprocess.run([sys.executable, str(CHECKER), str(FORMATTER)], text=True, capture_output=True, check=False)
    passed = len(results) == 38 and all(item['status'] == 'PASS' for item in results) and static.returncode == 0
    finished = utc()
    messages += ['AST parsing PASS', 'Static checker: ' + json.dumps(cli_actual(static), ensure_ascii=False),
                 f'{sum(item["status"] == "PASS" for item in results)}/{len(results)} synthetic checks PASS',
                 'No actual gate, Cargo, Godot, build, probe, or cohort was launched or consumed.']
    (target / 'validation.log').write_text('\n'.join(messages) + '\n')
    dump(target / 'input_snapshots.json', inputs)
    readme = (
        '# Pass17 gate formatter synthetic proof\n\n'
        'This is a persisted rerun of the 38 previously transient synthetic checks. '
        'It does not establish a real gate verdict. All fixture HEADs, counts, times, script hashes and logs are fabricated. '
        'The fake gate script path is never executed.\n\n'
        f'- Formatter SHA256: `{sha(source)}`\n'
        f'- Execution UTC: {started} → {finished}\n'
        f'- Outcome: {sum(item["status"] == "PASS" for item in results)}/{len(results)} checks PASS.\n'
        '- `validation.json` contains case outcomes, actual exceptions/CLI output, source hash and artifact hashes.\n'
        '- `input_snapshots.json` preserves receipt, gate, sidecar and mutated-file bytes for every case.\n'
        '- `fixture/` is restored to the valid input used for the successful CLI summary.\n'
        '- `summary.json` and `summary.md` are actual formatter CLI outputs from case 35.\n'
        '- `checksums.sha256` binds all proof files and the validation receipt, excluding itself.\n\n'
        'The original temporary run required resolving the macOS /var alias to /private/var in its fixture path. '
        'The path guard was preserved; this rerun resolves the fixture root before construction.\n\n'
        'Reproduce to a fresh external directory:\n\n'
        f'```sh\npython3 {Path(__file__).resolve()} /Users/rexxa/work/p17-data/formatter_validation_NEW\n```\n'
    )
    (target / 'README.md').write_text(readme)
    artifact_hashes = [
        {'path': str(path), 'bytes': len(path.read_bytes()), 'sha256': sha(path.read_bytes())}
        for path in sorted(target.rglob('*')) if path.is_file()
    ]
    report = {
        'schema_version': 1, 'kind': 'synthetic_formatter_validation', 'status': 'PASS' if passed else 'FAIL',
        'started_utc': started, 'finished_utc': finished, 'python': sys.version,
        'formatter': {'path': str(FORMATTER), 'sha256': sha(source)},
        'harness': {'path': str(Path(__file__).resolve()), 'sha256': sha(Path(__file__).read_bytes())},
        'command': [sys.executable, str(Path(__file__).resolve()), str(target)],
        'check_count': len(results), 'passed': sum(item['status'] == 'PASS' for item in results),
        'failed': sum(item['status'] != 'PASS' for item in results), 'checks': results,
        'ast_parse': 'PASS', 'static_checker': {'sha256': sha(CHECKER.read_bytes()), **cli_actual(static)},
        'final_fixture_status': final_fixture['status'], 'actual_gate_consumed': False, 'heavy_workloads_launched': False,
        'artifacts': artifact_hashes,
    }
    dump(target / 'validation.json', report)
    checksummed = [Path(__file__).resolve()] + [path for path in sorted(target.rglob('*')) if path.is_file()]
    (target / 'checksums.sha256').write_text(''.join(sha(path.read_bytes()) + '  ' + str(path) + '\n' for path in checksummed))
    print(f'{report["status"]}: {report["passed"]}/{report["check_count"]}; {target / "validation.json"}')
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main())
