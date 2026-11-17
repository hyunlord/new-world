# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest tools/v8/test_gate_test_evidence.py
"""Lightweight fail-closed receipt checks; never invokes Cargo or Godot."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gate_test_evidence as evidence
import gate_test_inputs as inputs
from gate_test_log import parse_log
from mux2_gate_inputs import EvidenceError

FIXTURES = Path(__file__).parent / 'fixtures/gate_test_evidence'
LOG = ('    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s\n'
       '     Running unittests src/lib.rs (target/debug/deps/sample-123abc)\n'
       '\nrunning 1 test\ntest works ... ok\n'
       'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s\n')


class LogTests(unittest.TestCase):
    def test_real_debug_excerpts_both_sources(self) -> None:
        for name in ('switch2-debug.log', 'c3a-debug.log'):
            with self.subTest(name=name):
                result = parse_log((FIXTURES / name).read_text(), ['unit:sim_bridge:lib'])
                self.assertGreater(result['passed'], 60)

    def test_real_release_interleaving_uses_outer_result(self) -> None:
        result = parse_log((FIXTURES / 'switch2-release-interleaved.log').read_text(), debug=False)
        self.assertEqual(result['passed'], 546)
        self.assertEqual(len(result['targets']), 1)

    def test_failed_closed_logs(self) -> None:
        cases = [LOG.rstrip(), LOG.rsplit('test result:', 1)[0], '',
                 LOG.replace('1 passed', '0 passed'), LOG.replace('0 failed', '1 failed'),
                 LOG.replace('0 filtered out', '1 filtered out'),
                 LOG.replace('`test`', '`release`'), LOG + 'unexplained tail\n',
                 LOG.replace('running 1 test', 'running 2 tests'), LOG + LOG]
        for text in cases:
            with self.subTest(text=text[-80:]), self.assertRaises(EvidenceError):
                parse_log(text, ['unit:sample:lib'])

    def test_missing_target_and_empty_suite(self) -> None:
        with self.assertRaises(EvidenceError):
            parse_log(LOG, ['unit:sample:lib', 'doc:sample'])
        with self.assertRaises(EvidenceError):
            parse_log(LOG.replace('running 1 test', 'running 0 tests').replace('1 passed', '0 passed'))


class ReceiptTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.root = self.directory / 'root'
        self.root.mkdir()
        self.before = self.directory / 'before.json'
        self.log = self.directory / 'test.log'
        self.receipt = self.directory / 'receipt.json'
        self.binding = {'commit': 'a' * 40, 'tree': 'b' * 40, 'files': {'rust/Cargo.lock': 'lock',
                        'data/value': 'data'}, 'rustc': 'version', 'profile': 'test',
                        'features': 'default', 'environment': {'RUSTFLAGS': 'hash'},
                        'inventory': ['unit:sample:lib']}
        self.before.write_text(json.dumps(self.binding))
        self.log.write_text(LOG)
        self.mock = patch.object(evidence, 'capture', return_value=self.binding)
        self.mock.start()
        self.addCleanup(self.mock.stop)

    def create(self, exit_code: int = 0, wall: float = 1) -> dict[str, object]:
        return evidence.create(self.root, self.before, self.log, self.receipt, exit_code, wall)

    def test_roundtrip_atomic_and_summary(self) -> None:
        value = self.create()
        self.assertEqual(value['summary']['passed'], 1)
        self.assertTrue(evidence.verify(self.root, self.receipt).startswith('PASS_REUSED '))
        self.assertEqual(list(self.directory.glob('.receipt.json*')), [])

    def test_failed_creation_never_issues_receipt(self) -> None:
        for exit_code, wall in [(1, 1), (0, 0), (0, float('nan')), (0, float('inf'))]:
            with self.subTest(exit_code=exit_code, wall=wall), self.assertRaises(EvidenceError):
                self.create(exit_code, wall)
            self.assertFalse(self.receipt.exists())

    def test_before_after_mismatch(self) -> None:
        with patch.object(evidence, 'capture', return_value={'different': True}):
            with self.assertRaises(EvidenceError):
                self.create()
        self.assertFalse(self.receipt.exists())

    def test_changes_while_creating_fail(self) -> None:
        with patch.object(evidence, 'capture', side_effect=[self.binding, {'changed': True}]):
            with self.assertRaises(EvidenceError):
                self.create()

    def test_receipt_and_log_tampering_fail(self) -> None:
        self.create()
        original = self.receipt.read_text()
        for change in ({'schema': True}, {'complete': False}, {'exit_code': 1},
                       {'summary': {}}, {'wall_seconds': -1}, {'inputs': {}}):
            value = json.loads(original)
            value.update(change)
            self.receipt.write_text(json.dumps(value))
            with self.subTest(change=change), self.assertRaises(EvidenceError):
                evidence.verify(self.root, self.receipt)
        self.receipt.write_text(original)
        self.log.write_text(LOG + '\n')
        with self.assertRaises(EvidenceError):
            evidence.verify(self.root, self.receipt)
        self.log.unlink()
        with self.assertRaises(OSError):
            evidence.verify(self.root, self.receipt)

    def test_every_frozen_binding_mismatch_rejects(self) -> None:
        self.create()
        for field in ('commit', 'tree', 'files', 'rustc', 'profile', 'features', 'environment'):
            changed = dict(self.binding)
            changed[field] = 'different'
            with self.subTest(field=field), patch.object(evidence, 'capture', return_value=changed):
                with self.assertRaises(EvidenceError):
                    evidence.verify(self.root, self.receipt)

    def test_missing_and_partial_receipt(self) -> None:
        with self.assertRaises(OSError):
            evidence.verify(self.root, self.receipt)
        self.receipt.write_text('[]')
        with self.assertRaises(EvidenceError):
            evidence.verify(self.root, self.receipt)
        self.receipt.write_text('{"schema":1,"schema":1}')
        with self.assertRaises(EvidenceError):
            evidence.verify(self.root, self.receipt)
        self.receipt.write_text('{')
        with self.assertRaises(ValueError):
            evidence.verify(self.root, self.receipt)


class InputTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'rust/crates/sample/src').mkdir(parents=True)
        (self.root / 'rust/Cargo.toml').write_text('[workspace]\nmembers=["crates/sample"]\n')
        (self.root / 'rust/Cargo.lock').write_text('lock')
        (self.root / 'rust/crates/sample/Cargo.toml').write_text('[package]\nname="sample"\n')
        (self.root / 'rust/crates/sample/src/lib.rs').write_text('// lib')
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True)
        subprocess.run(['git', '-C', str(self.root), 'add', 'rust'], check=True)
        subprocess.run(['git', '-C', str(self.root), '-c', 'user.name=Test', '-c',
                        'user.email=test@example.com', 'commit', '-qm', 'Fixture'], check=True)

    def capture(self) -> dict[str, inputs.Json]:
        with patch.object(inputs.subprocess, 'run', wraps=subprocess.run) as runner:
            result = inputs.capture(self.root)
            self.assertFalse(any(call.args[0][0] == 'cargo' for call in runner.call_args_list))
            return result

    def test_cli_snapshot_create_verify_and_reject(self) -> None:
        script = Path(evidence.__file__).resolve()
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            before, log, receipt = (output / name for name in ('before.json', 'test.log', 'receipt.json'))
            log.write_text(LOG + '\n   Doc-tests sample\n\nrunning 0 tests\n'
                           'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; '
                           '0 filtered out; finished in 0.00s\n')
            commands = [(['snapshot', '--out', str(before)], 0),
                        (['create', '--before', str(before), '--log', str(log), '--receipt', str(receipt),
                          '--exit-code', '0', '--wall-seconds', '1'], 0),
                        (['verify', '--receipt', str(receipt)], 0)]
            for command, expected in commands:
                result = subprocess.run([sys.executable, str(script), *command, '--root', str(self.root)],
                                        capture_output=True, text=True)
                self.assertEqual(result.returncode, expected, result.stderr)
            self.assertIn('PASS_REUSED', result.stdout)
            self.assertIn('test_debug passed 1 failed 0 ignored 0', result.stdout)
            log.write_text('partial')
            result = subprocess.run([sys.executable, str(script), 'verify', '--root', str(self.root),
                                     '--receipt', str(receipt)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertNotIn('PASS_REUSED', result.stdout)

    def test_tracked_internal_directory_and_file_symlinks(self) -> None:
        (self.root / '.agents/skills').mkdir(parents=True)
        (self.root / '.claude/skills/testing').mkdir(parents=True)
        target = self.root / '.claude/skills/testing/SKILL.md'
        target.write_text('first')
        (self.root / '.agents/skills/testing').symlink_to('../../.claude/skills/testing', target_is_directory=True)
        (self.root / 'linked-file').symlink_to('.claude/skills/testing/SKILL.md')
        subprocess.run(['git', '-C', str(self.root), 'add', '.agents', '.claude', 'linked-file'], check=True)
        subprocess.run(['git', '-C', str(self.root), '-c', 'user.name=Test', '-c',
                        'user.email=test@example.com', 'commit', '-qm', 'Links'], check=True)
        before = self.capture()
        self.assertIn('.agents/skills/testing/SKILL.md', before['files'])
        hashes = inputs.tracked_hashes(self.root, {'.agents/skills/testing', 'linked-file'})
        target.write_text('second')
        self.assertNotEqual(hashes, inputs.tracked_hashes(self.root, {'.agents/skills/testing', 'linked-file'}))

    def test_external_directory_symlink_rejects(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            (self.root / 'external').symlink_to(directory, target_is_directory=True)
            with self.assertRaises(EvidenceError):
                inputs.tracked_hashes(self.root, {'external'})

    def test_inventory_and_clean_binding(self) -> None:
        self.assertEqual(inputs.inventory(self.root), ['doc:sample', 'unit:sample:lib'])
        before = self.capture()
        (self.root / 'rust/crates/sample/src/lib.rs').write_text('// dirty')
        with self.assertRaises(EvidenceError):
            self.capture()
        self.assertIn('rust/Cargo.lock', before['files'])

    def test_ignored_lock_data_fixture_and_config_are_bound(self) -> None:
        info = self.root / '.git/info/exclude'
        info.write_text('data/\nfixtures/\nrust/.cargo/\nrust/Cargo.lock\n')
        for folder in ('data', 'fixtures', 'rust/.cargo'):
            (self.root / folder).mkdir(parents=True, exist_ok=True)
        paths = ['data/value.ron', 'fixtures/save.json', 'rust/.cargo/config.toml']
        for name in paths:
            (self.root / name).write_text('one')
        before = self.capture()
        for name in paths:
            (self.root / name).write_text('two')
            self.assertNotEqual(before, self.capture())
            (self.root / name).write_text('one')
        # Lock is tracked in this fixture, so content change is rejected before hashing.
        (self.root / 'rust/Cargo.lock').write_text('changed')
        with self.assertRaises(EvidenceError):
            self.capture()

    def test_external_fixture_bytes_and_scalar_presence(self) -> None:
        external = self.root.parent / (self.root.name + '-fixture')
        external.write_text('first')
        self.addCleanup(external.unlink)
        with patch.dict(os.environ, {'C3_C2B_SAVE_PATH': str(external)}):
            before = self.capture()
            external.write_text('second')
            self.assertNotEqual(before, self.capture())
        before = self.capture()
        with patch.dict(os.environ, {'C3_OUTCROP_KIND': 'iron'}):
            self.assertNotEqual(before, self.capture())

    def test_relative_external_fixture_paths_reject(self) -> None:
        for name in ('C3_C2B_SAVE_PATH', 'MUX2_V10_FIXTURES'):
            with self.subTest(name=name), patch.dict(os.environ, {name: 'relative/fixture'}):
                with self.assertRaisesRegex(EvidenceError, 'must be absolute'):
                    self.capture()

    def test_mux_lane_fd_is_normalized(self) -> None:
        with patch.dict(os.environ, {'WORLDSIM_MUX2_LANE_FD': '7'}):
            before = self.capture()
        with patch.dict(os.environ, {'WORLDSIM_MUX2_LANE_FD': '12'}):
            self.assertEqual(before, self.capture())

    def test_environment_hashes_and_settings_content_normalization(self) -> None:
        one, two = self.root / 'settings-one', self.root / 'settings-two'
        one.mkdir()
        two.mkdir()
        # Settings are gate output outside source control.
        (self.root / '.git/info/exclude').write_text('settings-*/\n')
        with patch.dict(os.environ, {'WORLDSIM_TEST_SETTINGS_DIR': str(one), 'RUSTFLAGS': 'secret'}):
            before = self.capture()
            self.assertNotIn('secret', json.dumps(before))
        with patch.dict(os.environ, {'WORLDSIM_TEST_SETTINGS_DIR': str(two), 'RUSTFLAGS': 'secret'}):
            self.assertEqual(before, self.capture())
            (two / 'settings.json').write_text('{}')
            self.assertNotEqual(before, self.capture())
        with patch.dict(os.environ, {'WORLDSIM_TEST_SETTINGS_DIR': str(one), 'RUSTFLAGS': 'changed'}):
            self.assertNotEqual(before, self.capture())


if __name__ == '__main__':
    unittest.main()
