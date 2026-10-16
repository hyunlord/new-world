# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 /Users/rexxa/work/p17-data/operations-tests/run_checks.py
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest

SOURCE = Path('/Users/rexxa/work/new-world-pass4/tools/revitalize')
DATA = Path('/Users/rexxa/work/p17-data/operations-tests')


class OperationChecks(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=DATA)
        self.root = Path(self.temporary.name)
        self.scripts = self.root / 'repo/tools/revitalize'
        self.scripts.mkdir(parents=True)
        self.lock = self.root / 'lock'
        self.log = self.root / 'RUNLOG.txt'
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        for name in ('wait_mac_p17.sh', 'quiet_lock_p17.sh', 'identity_p17.sh', 'perf_p17.sh'):
            text = (SOURCE / name).read_text().replace('/Users/rexxa/work/.mac_quiet.lock', str(self.lock))
            text = text.replace('/Users/rexxa/work/p17-data/quiet_lock_last_release', str(self.root / 'stamp'))
            text = text.replace('POLL_SECONDS: Final = 300', 'POLL_SECONDS: Final = 0.03')
            text = text.replace('SPACING_SECONDS: Final = 600', 'SPACING_SECONDS: Final = 0.15')
            text = text.replace('QUIET_WAIT_SECONDS: Final = 3600', 'QUIET_WAIT_SECONDS: Final = 0.09')
            text = text.replace('HOLD_SECONDS: Final = 3600', 'HOLD_SECONDS: Final = 0.5')
            text = text.replace('CLEANUP_SECONDS: Final = 3', 'CLEANUP_SECONDS: Final = 0.02')
            (self.scripts / name).write_text(text)
        self.write_bin('pgrep', '#!/bin/sh\nexit 1\n')
        self.write_bin('sysctl', '#!/bin/sh\necho "{ 0.0 0.0 0.0 }"\n')
        self.env = os.environ | {'PATH': str(self.bin) + ':' + os.environ['PATH']}
        for key in ('P17_QUIET_PID', 'P17_QUIET_OWNER', 'P17_QUIET_STARTED'):
            self.env.pop(key, None)

    def tearDown(self):
        self.temporary.cleanup()

    def write_bin(self, name, text):
        path = self.bin / name
        path.write_text(text)
        path.chmod(0o755)

    def run_quiet(self, *command):
        return subprocess.run(['bash', str(self.scripts / 'quiet_lock_p17.sh'), str(self.log), 'test hold', *command], env=self.env, capture_output=True, text=True, timeout=10)

    def pid_gone(self, pid):
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            status = subprocess.run(['ps', '-o', 'stat=', '-p', str(pid)], capture_output=True, text=True, check=False)
            if status.returncode or status.stdout.strip().startswith('Z'):
                return True
            time.sleep(0.01)
        return False

    def test_normal_failure_kills_background_child_and_releases(self):
        pidfile = self.root / 'child.pid'
        result = self.run_quiet('bash', '-c', f'sleep 60 & echo $! > "{pidfile}"; exit 7')
        self.assertEqual(result.returncode, 7, result.stderr)
        self.assertFalse(self.lock.exists())
        self.assertTrue(self.pid_gone(int(pidfile.read_text())))
        self.assertIn('exit=7', self.log.read_text())

    def test_timeout_stops_process_group_and_releases(self):
        pidfile = self.root / 'child.pid'
        result = self.run_quiet('bash', '-c', f'sleep 60 & echo $! > "{pidfile}"; wait')
        self.assertEqual(result.returncode, 124, result.stderr)
        self.assertFalse(self.lock.exists())
        self.assertTrue(self.pid_gone(int(pidfile.read_text())))
        self.assertIn('hard hold limit', self.log.read_text())

    def test_signal_stops_process_group_and_releases(self):
        pidfile = self.root / 'child.pid'
        process = subprocess.Popen(['bash', str(self.scripts / 'quiet_lock_p17.sh'), str(self.log), 'test signal', 'bash', '-c', f'sleep 60 & echo $! > "{pidfile}"; wait'], env=self.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        deadline = time.monotonic() + 3
        while not pidfile.exists() and time.monotonic() < deadline:
            time.sleep(0.005)
        self.assertTrue(pidfile.exists())
        process.send_signal(signal.SIGTERM)
        process.communicate(timeout=5)
        self.assertEqual(process.returncode, 143)
        self.assertFalse(self.lock.exists())
        self.assertTrue(self.pid_gone(int(pidfile.read_text())))

    def test_foreign_lock_waits_without_starting_command(self):
        self.lock.mkdir()
        (self.lock / 'info.txt').write_text('owner=foreign\npid=123\nstarted=now\n')
        marker = self.root / 'started'
        process = subprocess.Popen(['bash', str(self.scripts / 'quiet_lock_p17.sh'), str(self.log), 'foreign wait', 'touch', str(marker)], env=self.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        deadline = time.monotonic() + 3
        while (not self.log.exists() or 'foreign lock:' not in self.log.read_text()) and time.monotonic() < deadline:
            time.sleep(0.005)
        process.send_signal(signal.SIGTERM)
        process.communicate(timeout=5)
        self.assertEqual(process.returncode, 143)
        self.assertTrue(self.lock.exists())
        self.assertFalse(marker.exists())
        self.assertIn('owner=foreign', (self.lock / 'info.txt').read_text())

    def test_stale_lock_removed_and_recorded(self):
        self.lock.mkdir()
        (self.lock / 'info.txt').write_text('owner=stale\n')
        os.utime(self.lock, (time.time() - 7201, time.time() - 7201))
        result = subprocess.run(['bash', str(self.scripts / 'wait_mac_p17.sh'), str(self.log), 'stale check'], env=self.env, capture_output=True, text=True, timeout=3)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(self.lock.exists())
        self.assertIn('stale lock removed', self.log.read_text())

    def test_owned_lock_passes_to_wait_helper(self):
        result = self.run_quiet('bash', str(self.scripts / 'wait_mac_p17.sh'), str(self.log), 'owned work')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('owned lock permitted: owned work', self.log.read_text())

    def test_contention_and_spacing_are_recorded(self):
        self.write_bin('sysctl', '#!/bin/sh\necho "{ 6.0 6.0 6.0 }"\n')
        first = self.run_quiet('bash', '-c', 'test "$PERF_CONTENDED" = 1')
        second = self.run_quiet('true')
        self.assertEqual((first.returncode, second.returncode), (0, 0), first.stderr + second.stderr)
        log = self.log.read_text()
        self.assertIn('contended=1', log)
        self.assertIn('WAIT spacing start', log)

    def test_identity_commands_are_sequential_and_raw_stays_external(self):
        recorder = '''#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
with open(os.environ['COMMAND_LOG'], 'a') as stream:
    stream.write(json.dumps([pathlib.Path(sys.argv[0]).name, args]) + '\\n')
out = pathlib.Path(args[args.index('--out') + 1]); out.mkdir(parents=True)
if pathlib.Path(sys.argv[0]).name == 'worldsim_probe':
    (out / 'summary.json').write_text('{"alive": 1}')
    (out / 'raw.jsonl').write_text('raw')
else:
    (out / 'array.npy').write_bytes(b'array')
    (out / 'interventions.json').write_text('[]')
'''
        self.write_bin('worldsim_probe', recorder)
        self.write_bin('worldsim_wm', recorder)
        summary = self.root / 'repo/docs/revitalize/probe'
        raw = self.root / 'data'
        commands = self.root / 'commands.jsonl'
        result = subprocess.run(['bash', str(self.scripts / 'identity_p17.sh'), str(self.bin), str(summary), str(raw), 'sha'], env=self.env | {'COMMAND_LOG': str(commands)}, capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        import json
        calls = [json.loads(line) for line in commands.read_text().splitlines()]
        self.assertEqual([args[args.index('--scene') + 1] for _, args in calls], ['game_v19', 'game_v16', 'game_v20', 'game_v19'])
        self.assertTrue(all(args[args.index('--workers') + 1] == '4' for _, args in calls))
        self.assertEqual(sorted(path.name for path in summary.rglob('*') if path.is_file()), ['i2_sha256.txt', 'summary.json', 'summary.json', 'summary.json'])
        self.assertEqual(len(list(raw.rglob('raw.jsonl'))), 3)

    def test_perf_stage_a_order_and_stale_owner_rejected(self):
        perf = '''#!/usr/bin/env python3
import json, os, sys
with open(os.environ['COMMAND_LOG'], 'a') as stream:
    stream.write(json.dumps(sys.argv[1:]) + '\\n')
'''
        self.write_bin('worldsim_perf', perf)
        commands = self.root / 'commands.jsonl'
        self.env['COMMAND_LOG'] = str(commands)
        copied = self.scripts / 'quiet_lock_p17.sh'
        copied.write_text(copied.read_text().replace('HOLD_SECONDS: Final = 0.5', 'HOLD_SECONDS: Final = 5'))
        result = self.run_quiet('bash', str(self.scripts / 'perf_p17.sh'), str(self.root / 'perf'), str(self.bin), str(self.bin), 'a')
        self.assertEqual(result.returncode, 0, result.stderr)
        import json
        calls = [json.loads(line) for line in commands.read_text().splitlines()]
        self.assertEqual([args[args.index('--rules') + 1] for args in calls[:6]], ['game_v20', 'game_v21'] * 3)
        self.assertEqual([args[0] for args in calls[6:8]], ['quick', 'quick'])
        self.assertEqual([args[args.index('--agents') + 1] for args in calls[8:]], ['100,500,1000'] * 2)
        stale = self.env | {'P17_QUIET_PID': str(os.getpid()), 'P17_QUIET_OWNER': 'pass17-sim (Codex, new-world-pass4)', 'P17_QUIET_STARTED': 'stale'}
        denied = subprocess.run(['bash', str(self.scripts / 'perf_p17.sh'), str(self.root / 'perf-stale'), str(self.bin), str(self.bin), 'a'], env=stale, capture_output=True, text=True, timeout=3)
        self.assertEqual(denied.returncode, 1)
        self.assertIn('owned quiet lock is no longer held', denied.stderr)


if __name__ == '__main__':
    unittest.main(verbosity=2)
