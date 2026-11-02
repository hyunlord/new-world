"""Actual tiny shell producers; no Cargo, Godot or native workloads executed."""
import fcntl
import os
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_setup_clippy as clippy
from generic_gate import GateContractError, document
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_setup_prepare import PreparedSetup, Tool, prepare
from mux2_gate_inputs import Json
from test_generic_setup_bridge import unused_wait
from test_generic_setup_prepare import Fixture


def fixture(home: Path, code: int = 0) -> PreparedSetup:
    f = Fixture(home)
    (f.bin/'bash').unlink()
    _ = (f.bin/'cargo').write_text('#!/bin/sh\nprintf "%s\\n" "$@"\nprintf clippy > target/clippy-marker\nexit '+str(code)+'\n')
    tools = tuple(Tool(t.name, Path('/bin/bash') if t.name == 'bash' else t.path,
                       file_hash(Path('/bin/bash') if t.name == 'bash' else t.path)) for t in f.inputs.tools)
    env = tuple((k, v+':/bin' if k == 'PATH' else v) for k,v in f.inputs.environment)
    prepared = prepare(replace(f.inputs, tools=tools, environment=env))
    # Existing target is a synthetic endpoint, explicitly not prior workspace authority.
    (f.root/'rust/target').mkdir()
    _ = (f.root/'rust/target/prior-marker').write_text('synthetic after5 bytes')
    return prepared


class ClippyTests(unittest.TestCase):
    def test_callable_adapter_exists(self) -> None:
        self.assertTrue(callable(getattr(clippy, 'run_clippy_owned', None)))

    def test_original_command_existing_target_and_real_pid(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp).resolve()
            prepared = fixture(home)
            lock = home/'lock'
            with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_clippy.qualify_clippy', return_value=()):
                result = clippy.run_clippy_owned(prepared, clippy.ClippyWaitEvidence(unused_wait(prepared), ()), home/'step6', owner.fileno())
                self.assertEqual(result.status, 'PASS_STEP6_ENDPOINT_ONLY')
                self.assertFalse(result.chronology_authority)
                self.assertFalse(result.build_product_authority)
                row = document(result.receipt.read_text())
                self.assertEqual(row.field('original_argv').value, ['/bin/bash', '-c', 'cd rust && cargo clippy --workspace --all-targets -- -D warnings'])
                self.assertGreater(result.pid, 0)
                self.assertEqual(row.field('chronology').value, 'AFTER4_AFTER5_PRODUCER_AUTHENTICATION_REQUIRED')
                native = document((home/'step6/process/process.json').read_text())
                self.assertEqual(native.field('cwd').value, str(prepared.inputs.root))
                self.assertEqual((home/'step6/process/stdout.log').read_text().splitlines(), ['clippy','--workspace','--all-targets','--','-D','warnings'])
                self.assertEqual((prepared.inputs.root/'rust/target/prior-marker').read_text(), 'synthetic after5 bytes')
                self.assertIn('rust/target/prior-marker', row.field('target_before').mapping())
                self.assertIn('rust/target/clippy-marker', row.field('target_after').mapping())
                with lock.open('a') as rival, self.assertRaises(BlockingIOError):
                    fcntl.flock(rival.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_native_nonzero_retains_unknown_and_actual_raw_outcome(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp).resolve()
            prepared = fixture(home, 17)
            lock = home/'lock'
            with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_clippy.qualify_clippy', return_value=()):
                with self.assertRaises(clippy.ClippyOwnershipUnknown):
                    _ = clippy.run_clippy_owned(prepared, clippy.ClippyWaitEvidence(unused_wait(prepared), ()), home/'step6', owner.fileno())
                row = document((home/'step6/receipt.json').read_text())
                self.assertEqual(row.field('native_exit').value, 17)
                self.assertEqual(row.field('ownership').value, 'UNKNOWN_KEEP_CALLER_LANE')
                self.assertIsNone(row.field('target_after').value)
                with lock.open('a') as rival, self.assertRaises(BlockingIOError):
                    fcntl.flock(rival.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_unreturned_and_secondary_failure_preserve_exception_objects(self) -> None:
        for primary in (KeyboardInterrupt(), SystemExit(4), TypeError('primary')):
            with self.subTest(kind=type(primary)), tempfile.TemporaryDirectory() as temp:
                home = Path(temp).resolve()
                prepared = fixture(home)
                lock = home/'lock'
                secondary = OSError('terminal diagnostic')
                def write(path: Path, row: dict[str, Json], error: OSError = secondary) -> None:
                    if path.name == 'receipt.json':raise error
                    atomic_json(path, row)
                with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_clippy.qualify_clippy', return_value=()), patch('generic_setup_clippy.execute', side_effect=primary), patch('generic_setup_clippy.atomic_json', side_effect=write):
                    with self.assertRaises(clippy.ClippyOwnershipUnknown) as caught:
                        _ = clippy.run_clippy_owned(prepared, clippy.ClippyWaitEvidence(unused_wait(prepared), ()), home/'step6', owner.fileno())
                    self.assertIs(caught.exception.primary, primary)
                    self.assertIs(caught.exception.secondary, secondary)
                    self.assertTrue((home/'step6/started.json').is_file())
                    self.assertFalse((home/'step6/receipt.json').exists())
                    self.assertGreater(os.fstat(owner.fileno()).st_ino, 0)

    def test_corrupt_returned_log_is_unknown(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp).resolve()
            prepared = fixture(home)
            lock = home/'lock'
            def corrupt(spec: ProcessSpec) -> ProcessResult:
                result = execute(spec)
                _ = (spec.output_root/'stdout.log').write_text('forged')
                return result
            with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_clippy.qualify_clippy', return_value=()), patch('generic_setup_clippy.execute', side_effect=corrupt), self.assertRaises(clippy.ClippyOwnershipUnknown):
                _ = clippy.run_clippy_owned(prepared, clippy.ClippyWaitEvidence(unused_wait(prepared), ()), home/'step6', owner.fileno())

    def test_missing_lane_fails_before_launch(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp).resolve()
            prepared = fixture(home)
            with patch('generic_setup_clippy.execute') as launch, self.assertRaises(clippy.ClippyFailure):
                _ = clippy.run_clippy_owned(prepared, clippy.ClippyWaitEvidence(unused_wait(prepared), ()), home/'step6', -1)
            launch.assert_not_called()
            self.assertFalse((home/'step6').exists())

    def test_absent_source_qualification_rejects_before_launch(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp).resolve()
            prepared = fixture(home)
            lock = home/'lock'
            with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_clippy.execute') as launch, self.assertRaises(clippy.ClippyFailure):
                _ = clippy.run_clippy_owned(prepared, clippy.ClippyWaitEvidence(unused_wait(prepared), ()), home/'step6', owner.fileno())
            launch.assert_not_called()
            self.assertFalse((home/'step6').exists())

    def test_plugin_resolution_uses_cargo_home_before_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp).resolve()
            prepared = fixture(home)
            source = home/'synthetic-source.rs'
            _ = source.write_text('synthetic source profile, not native qualification')
            tools = {t.name:t.sha256 for t in prepared.inputs.tools if t.name in {'cargo-clippy','clippy-driver'}}
            evidence = clippy.ClippyWaitEvidence(unused_wait(prepared), (source,))
            with patch('generic_setup_clippy.qualify_wait', return_value=()), patch('generic_setup_clippy.CLIPPY_SOURCES', {source.name:file_hash(source)}), patch('generic_setup_clippy.CLIPPY_TOOLS', tools):
                result = clippy.qualify_clippy(prepared, evidence)
                self.assertIn((source, file_hash(source)), result)
                alternate = prepared.inputs.cargo_home/'bin/cargo-clippy'
                alternate.parent.mkdir()
                _ = alternate.write_text('#!/bin/sh\nexit 0\n')
                alternate.chmod(0o755)
                with self.assertRaisesRegex(GateContractError, 'plugin resolution mismatch'):
                    _ = clippy.qualify_clippy(prepared, evidence)

    def test_source_or_driver_bytes_drift_rejected(self) -> None:
        for changed in ('source', 'driver'):
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as temp:
                home = Path(temp).resolve()
                prepared = fixture(home)
                source = home/'synthetic-source.rs'
                _ = source.write_text('synthetic')
                tools = {t.name:t.sha256 for t in prepared.inputs.tools if t.name in {'cargo-clippy','clippy-driver'}}
                evidence = clippy.ClippyWaitEvidence(unused_wait(prepared), (source,))
                pins = {source.name:file_hash(source)}
                with patch('generic_setup_clippy.qualify_wait', return_value=()), patch('generic_setup_clippy.CLIPPY_SOURCES', pins), patch('generic_setup_clippy.CLIPPY_TOOLS', tools):
                    _ = (source if changed == 'source' else home/'bin/clippy-driver').write_text('changed')
                    with self.assertRaises(GateContractError):
                        _ = clippy.qualify_clippy(prepared, evidence)

    def test_settled_terminal_failure_retains_primary_and_secondary(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp).resolve()
            prepared = fixture(home)
            lock = home/'lock'
            first = OSError('first publication')
            second = RuntimeError('second publication')
            attempts = 0
            def write(path: Path, row: dict[str, Json]) -> None:
                nonlocal attempts
                if path.name == 'receipt.json':
                    attempts += 1
                    raise first if attempts == 1 else second
                atomic_json(path, row)
            with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_clippy.qualify_clippy', return_value=()), patch('generic_setup_clippy.atomic_json', side_effect=write):
                with self.assertRaises(clippy.ClippyFailure) as caught:
                    _ = clippy.run_clippy_owned(prepared, clippy.ClippyWaitEvidence(unused_wait(prepared), ()), home/'step6', owner.fileno())
                self.assertNotIsInstance(caught.exception, clippy.ClippyOwnershipUnknown)
                self.assertIs(caught.exception.primary, first)
                self.assertIs(caught.exception.secondary, second)
                self.assertEqual(document((home/'step6/process/process.json').read_text()).field('native_exit').value, 0)

    def test_environment_injection_and_missing_target_do_not_launch(self) -> None:
        for changed in ('environment', 'target'):
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as temp:
                home = Path(temp).resolve()
                prepared = fixture(home)
                if changed == 'environment':
                    prepared = replace(prepared, inputs=replace(prepared.inputs, environment=(*prepared.inputs.environment, ('RUSTFLAGS','--cfg injected'))))
                else:
                    (prepared.inputs.root/'rust/target/prior-marker').unlink()
                    (prepared.inputs.root/'rust/target').rmdir()
                lock = home/'lock'
                with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_clippy.qualify_clippy', return_value=()), patch('generic_setup_clippy.execute') as launch, self.assertRaises(clippy.ClippyFailure) as caught:
                    _ = clippy.run_clippy_owned(prepared, clippy.ClippyWaitEvidence(unused_wait(prepared), ()), home/'step6', owner.fileno())
                self.assertNotIsInstance(caught.exception, clippy.ClippyOwnershipUnknown)
                launch.assert_not_called()

    def test_explicit_import_environment_preserved_without_ambient_defaults(self) -> None:
        for explicit in ((), (('__CF_USER_TEXT_ENCODING', f'0x{os.getuid():X}:0x3:0x33'), ('PYTHONCOERCECLOCALE', '0'))):
            with self.subTest(explicit=bool(explicit)), tempfile.TemporaryDirectory() as temp:
                home = Path(temp).resolve()
                prepared = fixture(home)
                prepared = replace(prepared, inputs=replace(prepared.inputs, environment=(*prepared.inputs.environment, *explicit)))
                lock = home/'lock'
                ambient = {'__CF_USER_TEXT_ENCODING':'ambient-not-admitted', 'PYTHONCOERCECLOCALE':'ambient-not-admitted'}
                with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_clippy.qualify_clippy', return_value=()), patch.dict(os.environ, ambient):
                    result = clippy.run_clippy_owned(prepared, clippy.ClippyWaitEvidence(unused_wait(prepared), ()), home/'step6', owner.fileno())
                expected = dict(prepared.inputs.environment)
                for path in (result.receipt, home/'step6/process/process.json'):
                    actual = document(path.read_text()).field('environment').mapping()
                    self.assertEqual(actual, expected)
                    for name, value in explicit:self.assertEqual(actual[name], value)
                    if not explicit:self.assertTrue(ambient.keys().isdisjoint(actual))
