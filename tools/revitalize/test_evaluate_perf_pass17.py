import hashlib
import json
from datetime import datetime
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

import evaluate_perf_pass17 as perf


class PerformanceEvaluation(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name).resolve()
        self.addCleanup(self.temp.cleanup)
        self.make_fixture()

    def make_fixture(self, stage: str = 'a', old_mean: float = 1.0, new_mean: float = 1.1) -> None:
        old, new = ('20', '21') if stage == 'a' else ('21', '22')
        builds = {}
        for role, commit in (('final', 'a' * 40), ('baseline', 'b' * 40)):
            binary = self.root / f'{role}_binary'
            binary.write_bytes(role.encode())
            builds[role] = {'path': str(binary), 'sha256': perf.digest(binary), 'commit': commit}
        prefix = '2026-10-15 10:00:00 KST HOLD start: fixture; load=1.0; contended=0; waited=0.0s\n'
        quiet = self.root / 'quiet.log'
        quiet.write_text(prefix + '2026-10-15 10:00:10 KST HOLD end: fixture; exit=0; held=10.0s\n')
        commands = []
        batches = [('final', f'a/r{r}', 'stress', version, [500], 1) for r in (1, 2, 3) for version in (old, new)]
        batches += [('baseline', 'quick_base', 'quick', '19', [500], 1), ('final', 'quick_final', 'quick', '19', [500], 1)]
        batches += [('final', f'b_game_v{version}', 'stress', version, [100, 500, 1000], 2) for version in (new, old)]
        for index, (role, directory, mode, version, sizes, days) in enumerate(batches):
            out = self.root / directory
            out.mkdir(parents=True, exist_ok=True)
            argv = [builds[role]['path'], mode]
            if mode == 'stress':
                argv += ['--seeds', '15100..15101', '--agents', ','.join(map(str, sizes)), '--side', '64', '--days', str(days), '--feed', 'fed', '--rules', f'game_v{version}']
            argv += ['--out', str(out)]
            outputs = {}
            entries = json.loads((out / 'runs.json').read_text())['runs'] if (out / 'runs.json').exists() else []
            for size in sizes:
                stem = f'{mode}_stress_v{version}_s15100_n{size}_side64_fed'
                mean = new_mean if version == new else old_mean
                record = {'pass': 15, 'mode': mode, 'scene': f'stress_v{version}', 'seed': 15100, 'agents_requested': size, 'founders': size, 'side': 64, 'feed': 'fed', 'build_profile': 'release', 'ticks_per_day': 1440, 'ticks': days * 1440, 'days': float(days), 'warmup': {'ticks': 60}, 'tick_summary': {'ticks': days * 1440 - 60, 'mean_ms': mean, 'p999_ms': 2.0, 'max_ms': 3.0}, 'mean_alive_0600': float(size), 'systems': [{'name': 'AgentDecisionSystem', 'calls': days * 1440 - 60, 'mean_us_per_call': size * 0.5, 'share_of_tick_time': 0.5}], 'git_sha': builds[role]['commit'], 'machine': {'cpu': 'fixture', 'ncpu': '1'}, 'contended': False, 'fingerprint': 'frozen'}
                path = out / f'{stem}.json'
                path.write_text(json.dumps(record))
                outputs[str(path.relative_to(self.root))] = perf.digest(path)
                entries.append({'stem': stem, 'file': path.name, 'status': 'ok'})
            (out / 'runs.json').write_text(json.dumps({'pass': 15, 'runs': entries}))
            started = datetime.fromisoformat('2026-10-15T01:00:00+00:00').timestamp() + index
            commands.append({'index': index, 'build': role, 'argv': argv, 'started': started, 'finished': started + 0.5, 'exit_code': 0, 'binary_sha256_before': builds[role]['sha256'], 'binary_sha256_after': builds[role]['sha256'], 'outputs': outputs})
        ledger = self.root / 'commands.jsonl'
        ledger.write_text(''.join(json.dumps(command) + '\n' for command in commands))
        runlog = self.root / 'RUNLOG.txt'
        runlog.write_text('owned lock permitted\n' * 10)
        manifest = {'schema_version': 1, 'stage': stage, 'status': 0, 'builds': builds, 'quiet': {'owner': 'pass17-sim (Codex, new-world-pass4)', 'pid': '123', 'started': '2026-10-15T01:00:00Z', 'purpose': 'fixture', 'log': str(quiet), 'prefix_bytes': len(prefix.encode()), 'prefix_sha256': hashlib.sha256(prefix.encode()).hexdigest(), 'contended': False}, 'commands': {'file': 'commands.jsonl', 'sha256': perf.digest(ledger)}, 'runlog': {'file': 'RUNLOG.txt', 'sha256': perf.digest(runlog)}, 'indexes': {str(path.relative_to(self.root)): perf.digest(path) for path in self.root.glob('**/runs.json')}}
        (self.root / 'provenance.json').write_text(json.dumps(manifest))

    def change_run(self, relative: str, edit) -> None:
        path = self.root / relative
        record = json.loads(path.read_text())
        edit(record)
        path.write_text(json.dumps(record))
        ledger = self.root / 'commands.jsonl'
        commands = [json.loads(line) for line in ledger.read_text().splitlines()]
        for command in commands:
            if relative in command['outputs']:
                command['outputs'][relative] = perf.digest(path)
        ledger.write_text(''.join(json.dumps(command) + '\n' for command in commands))
        manifest = json.loads((self.root / 'provenance.json').read_text())
        manifest['commands']['sha256'] = perf.digest(ledger)
        (self.root / 'provenance.json').write_text(json.dumps(manifest))

    def test_complete_stage_a_and_b_pass(self) -> None:
        for stage in ('a', 'b'):
            with self.subTest(stage=stage):
                if stage == 'b':
                    self.temp.cleanup()
                    self.temp = tempfile.TemporaryDirectory()
                    self.addCleanup(self.temp.cleanup)
                    self.root = Path(self.temp.name).resolve()
                    self.make_fixture(stage)
                result = perf.evaluate(self.root, stage)
                self.assertEqual(result['verdict'], 'PASS', result['errors'])
                self.assertAlmostEqual(result['slopes']['new']['b'], 1.0)

    def test_faster_than_minus_fifteen_percent_still_passes(self) -> None:
        for repeat in (1, 2, 3):
            self.change_run(f'a/r{repeat}/stress_stress_v21_s15100_n500_side64_fed.json', lambda run: run['tick_summary'].update(mean_ms=0.5))
        self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'PASS')

    def test_mean_and_tail_use_separate_three_run_medians(self) -> None:
        for repeat, mean, tail in ((1, 1.1, 2.0), (2, 1.1, 2.0), (3, 99.0, 99.0)):
            self.change_run(f'a/r{repeat}/stress_stress_v21_s15100_n500_side64_fed.json', lambda run, m=mean, t=tail: run['tick_summary'].update(mean_ms=m, p999_ms=t))
        result = perf.evaluate(self.root, 'a')
        self.assertEqual(result['verdict'], 'PASS')
        self.assertEqual(result['medians']['new'], {'mean_ms': 1.1, 'p999_ms': 2.0})

    def test_mean_relative_failure(self) -> None:
        for repeat in (1, 2, 3):
            self.change_run(f'a/r{repeat}/stress_stress_v21_s15100_n500_side64_fed.json', lambda run: run['tick_summary'].update(mean_ms=1.151))
        self.assertFalse(perf.evaluate(self.root, 'a')['checks']['mean_relative']['passed'])

    def test_tail_relative_failure(self) -> None:
        for repeat in (1, 2, 3):
            self.change_run(f'a/r{repeat}/stress_stress_v21_s15100_n500_side64_fed.json', lambda run: run['tick_summary'].update(p999_ms=2.701))
        self.assertFalse(perf.evaluate(self.root, 'a')['checks']['p999_relative']['passed'])

    def test_relative_limits_are_inclusive(self) -> None:
        for repeat in (1, 2, 3):
            self.change_run(f'a/r{repeat}/stress_stress_v21_s15100_n500_side64_fed.json', lambda run: run['tick_summary'].update(mean_ms=1.15, p999_ms=2.7))
        self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'PASS')

    def test_absolute_limit_and_failure(self) -> None:
        for mean, verdict in ((4.0, 'PASS'), (4.001, 'FAIL')):
            for repeat in (1, 2, 3):
                self.change_run(f'a/r{repeat}/stress_stress_v20_s15100_n500_side64_fed.json', lambda run: run['tick_summary'].update(mean_ms=4.0))
                self.change_run(f'a/r{repeat}/stress_stress_v21_s15100_n500_side64_fed.json', lambda run, m=mean: run['tick_summary'].update(mean_ms=m))
            self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], verdict)

    def change_ledger(self, edit) -> None:
        ledger = self.root / 'commands.jsonl'
        commands = [json.loads(line) for line in ledger.read_text().splitlines()]
        edit(commands)
        ledger.write_text(''.join(json.dumps(command) + '\n' for command in commands))
        path = self.root / 'provenance.json'
        manifest = json.loads(path.read_text())
        manifest['commands']['sha256'] = perf.digest(ledger)
        path.write_text(json.dumps(manifest))

    def test_failed_command_is_error(self) -> None:
        self.change_ledger(lambda rows: rows[0].update(exit_code=1))
        self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'ERROR')

    def test_wrong_command_and_command_outside_hold_are_errors(self) -> None:
        original = json.loads((self.root / 'commands.jsonl').read_text().splitlines()[0])
        for edit in (lambda row: row.update(argv=['not the recorded workload']), lambda row: row.update(finished=row['started'] + 3601)):
            with self.subTest(edit=edit):
                self.change_ledger(lambda rows: rows[0].update(original))
                self.change_ledger(lambda rows: edit(rows[0]))
                self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'ERROR')

    def test_changed_binary_is_error(self) -> None:
        (self.root / 'final_binary').write_text('changed')
        self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'ERROR')

    def test_changed_run_index_is_error(self) -> None:
        path = self.root / 'a/r1/runs.json'
        path.write_text(path.read_text() + ' ')
        self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'ERROR')

    def test_wrong_stage_is_error(self) -> None:
        self.assertEqual(perf.evaluate(self.root, 'b')['verdict'], 'ERROR')

    def test_degenerate_slope_is_error(self) -> None:
        for size in (100, 500, 1000):
            self.change_run(f'b_game_v21/stress_stress_v21_s15100_n{size}_side64_fed.json', lambda run: run.update(mean_alive_0600=10.0))
        self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'ERROR')

    def test_slope_uses_observed_population_and_rejects_near_quadratic(self) -> None:
        for size in (100, 500, 1000):
            self.change_run(f'b_game_v21/stress_stress_v21_s15100_n{size}_side64_fed.json', lambda run, n=size: run.update(mean_alive_0600=n / 2, systems=[{'name': 'AgentDecisionSystem', 'calls': 2820, 'mean_us_per_call': (n / 2) ** 1.8, 'share_of_tick_time': 0.5}]))
        result = perf.evaluate(self.root, 'a')
        self.assertAlmostEqual(result['slopes']['new']['b'], 1.8)
        self.assertFalse(result['checks']['slope']['passed'])

    def test_missing_run_is_error(self) -> None:
        (self.root / 'quick_base/quick_stress_v19_s15100_n500_side64_fed.json').unlink()
        self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'ERROR')

    def test_tampered_run_is_error(self) -> None:
        path = self.root / 'a/r1/stress_stress_v21_s15100_n500_side64_fed.json'
        path.write_text(path.read_text() + ' ')
        self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'ERROR')

    def test_wrong_build_and_nan_are_errors(self) -> None:
        relative = 'a/r1/stress_stress_v21_s15100_n500_side64_fed.json'
        for field, value in (('git_sha', 'unknown'), ('mean_alive_0600', float('nan'))):
            with self.subTest(field=field):
                self.change_run(relative, lambda run: run.update({'git_sha': 'a' * 40, 'mean_alive_0600': 500.0, field: value}))
                self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'ERROR')

    def test_unclosed_quiet_hold_is_error(self) -> None:
        path = self.root / 'quiet.log'
        path.write_text(path.read_text().splitlines()[0] + '\n')
        self.assertEqual(perf.evaluate(self.root, 'a')['verdict'], 'ERROR')

    def test_cli_writes_machine_and_human_reports(self) -> None:
        out = self.root / 'report'
        self.assertEqual(perf.main(['--stage', 'a', '--input', str(self.root), '--out', str(out)]), 0)
        self.assertEqual(json.loads((out / 'perf.json').read_text())['verdict'], 'PASS')
        self.assertIn('AgentDecisionSystem', (out / 'perf.md').read_text())

    def test_real_runner_with_fake_measurements_records_complete_provenance(self) -> None:
        scripts = self.root / 'repo/tools/revitalize'
        scripts.mkdir(parents=True)
        lock = self.root / 'fake.lock'
        for name in ('perf_p17.sh', 'quiet_lock_p17.sh', 'wait_mac_p17.sh'):
            content = Path(__file__).with_name(name).read_text().replace('/Users/rexxa/work/.mac_quiet.lock', str(lock)).replace('/Users/rexxa/work/p17-data/quiet_lock_last_release', str(self.root / 'last_release')).replace('CLEANUP_SECONDS: Final = 3', 'CLEANUP_SECONDS: Final = 0.01').replace('SPACING_SECONDS: Final = 600', 'SPACING_SECONDS: Final = 0')
            (scripts / name).write_text(content)
        fake_tools = self.root / 'fake_tools'
        fake_tools.mkdir()
        for name, content in (('sysctl', '#!/bin/sh\nprintf "{ 1.0 1.0 1.0 }\\n"\n'), ('pgrep', '#!/bin/sh\nexit 1\n')):
            path = fake_tools / name
            path.write_text(content)
            path.chmod(0o755)
        fake_perf = '''#!/usr/bin/env python3
import json
import os
from pathlib import Path
import sys
if os.environ.get('P17_TEST_FAIL') == '1':
    raise SystemExit(3)
args = sys.argv[1:]
out = Path(args[args.index('--out') + 1])
relative = out.relative_to(os.environ['P17_TEST_RAW'])
source = Path(os.environ['P17_TEST_TEMPLATES']) / relative
out.mkdir(parents=True, exist_ok=True)
mode = args[0]
version = args[args.index('--rules') + 1].removeprefix('game_v') if mode == 'stress' else '19'
sizes = args[args.index('--agents') + 1].split(',') if mode == 'stress' else ['500']
index = out / 'runs.json'
entries = json.loads(index.read_text())['runs'] if index.exists() else []
for size in sizes:
    stem = f'{mode}_stress_v{version}_s15100_n{size}_side64_fed'
    run = json.loads((source / f'{stem}.json').read_text())
    run['git_sha'] = os.environ['WORLDSIM_GIT_SHA']
    run['contended'] = os.environ['PERF_CONTENDED'] == '1'
    (out / f'{stem}.json').write_text(json.dumps(run))
    entries.append({'stem': stem, 'file': f'{stem}.json', 'status': 'ok'})
index.write_text(json.dumps({'pass': 15, 'runs': entries}))
'''
        binaries = []
        for role in ('final', 'baseline'):
            directory = self.root / f'{role}_bin'
            directory.mkdir()
            binary = directory / 'worldsim_perf'
            binary.write_text(fake_perf)
            binary.chmod(0o755)
            binaries.append(str(directory))
        env = os.environ | {'PATH': str(fake_tools) + os.pathsep + os.environ['PATH'], 'P17_TEST_TEMPLATES': str(self.root)}
        for key in ('P17_QUIET_PID', 'P17_QUIET_OWNER', 'P17_QUIET_STARTED'):
            env.pop(key, None)
        for failure in (False, True):
            with self.subTest(failure=failure):
                raw = self.root / f'fake_raw_{failure}'
                quiet_log = raw / 'quiet.log'
                env.update(P17_TEST_RAW=str(raw), P17_TEST_FAIL=str(int(failure)))
                args = ['bash', str(scripts / 'quiet_lock_p17.sh'), str(quiet_log), 'fake runner verification', 'bash', str(scripts / 'perf_p17.sh'), str(raw), *binaries, 'a', 'a' * 40, 'b' * 40, str(quiet_log)]
                completed = subprocess.run(args, env=env, capture_output=True, text=True, timeout=20, check=False)
                self.assertEqual(completed.returncode, int(failure), completed.stderr)
                self.assertFalse(lock.exists())
                result = perf.evaluate(raw, 'a')
                self.assertEqual(result['verdict'], 'ERROR' if failure else 'PASS', result['errors'])
                self.assertEqual(len((raw / 'commands.jsonl').read_text().splitlines()), 10)
                if not failure:
                    self.assertEqual(len(result['repeats']['new']), 3)
                    self.assertEqual(result['provenance']['builds']['final']['commit'], 'a' * 40)
                    self.assertEqual(result['provenance']['builds']['baseline']['commit'], 'b' * 40)
