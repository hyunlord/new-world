"""Actual tiny shell producer; reviewed Cargo qualification is fixture-only."""
import fcntl
import os
import shlex
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_setup_bridge as bridge
from generic_process import atomic_json, file_hash
from generic_setup_prepare import PreparedSetup, Tool, prepare
from generic_workspace_cases import ListingProcess, process_output
from mux2_gate_inputs import Json
from test_generic_setup_bridge import unused_wait
from test_generic_setup_prepare import Fixture


class EndpointIntegrityTests(unittest.TestCase):
    def exercise(self, stage: str, mutation: str) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            (f.bin/'bash').unlink()
            _ = (f.bin/'cargo').write_text('#!/bin/sh\n/bin/mkdir -p target/debug\nprintf bridge > target/debug/libsim_bridge.dylib\n')
            if stage == 'producer-effects':
                with (f.bin/'cargo').open('a') as script:
                    _ = script.write('/bin/mkdir -p target/empty ../.godot/empty '+shlex.quote(str(f.inputs.settings/'empty'))+'\n')
            tools = tuple(Tool(t.name, Path('/bin/bash') if t.name == 'bash' else t.path,
                               file_hash(Path('/bin/bash') if t.name == 'bash' else t.path)) for t in f.inputs.tools)
            env = tuple((k, v+':/bin' if k == 'PATH' else v) for k, v in f.inputs.environment)
            prepared = prepare(replace(f.inputs, tools=tools, environment=env))
            calls = 0
            injected = False
            output_calls = 0

            def mutate() -> None:
                nonlocal injected
                injected = True
                paths = {'target': f.root/'rust/target/debug/libsim_bridge.dylib',
                         'raw': f.inputs.output/'process/extra.txt',
                         'receipt': f.inputs.output/'receipt.json',
                         'directory': f.inputs.output/'empty',
                         'cache': f.root/'.godot', 'settings': f.inputs.settings}
                path = paths[mutation]
                if mutation in {'directory', 'cache', 'settings'}:
                    path.mkdir(exist_ok=True)
                    (path/'empty-added').mkdir()
                else:
                    _ = path.write_text('mutated')

            def qualify(_: PreparedSetup, _evidence: bridge.CargoWaitEvidence) -> tuple[tuple[Path, str], ...]:
                nonlocal calls
                calls += 1
                if (stage == 'authentication' and calls == 3) or (stage == 'final-authentication' and calls == 5):
                    mutate()
                return ()

            def publish(path: Path, value: dict[str, Json]) -> None:
                atomic_json(path, value)
                if stage == 'publication' and path.name == 'receipt.json':
                    mutate()

            def authenticate(binding: ListingProcess, argv: tuple[str, ...], cwd: Path,
                             environment: dict[str, str], required: dict[str, str]) -> bytes:
                nonlocal output_calls
                raw = process_output(binding, argv, cwd, environment, required)
                output_calls += 1
                if (stage == 'first-output' and output_calls == 1) or (stage == 'last-output' and output_calls == 2):
                    mutate()
                return raw

            lock = f.home/'lock'
            with lock.open('w') as owner, patch.object(bridge, 'LOCK', lock), patch.object(bridge, 'qualify_wait', side_effect=qualify), patch.object(bridge, 'atomic_json', side_effect=publish), patch.object(bridge, 'process_output', side_effect=authenticate):
                fcntl.flock(owner.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                if stage == 'producer-effects':
                    result = bridge.run_bridge_owned(prepared, unused_wait(prepared), owner.fileno())
                    self.assertEqual(result.status, 'PASS_STEP2_ENDPOINT_ONLY')
                    self.assertTrue((f.root/'.godot/empty').is_dir())
                    self.assertTrue((f.inputs.settings/'empty').is_dir())
                    return
                with self.assertRaises(bridge.BridgeFailure) as caught:
                    _ = bridge.run_bridge_owned(prepared, unused_wait(prepared), owner.fileno())
                self.assertTrue(injected)
                self.assertIn('endpoint', str(caught.exception.primary))
                self.assertEqual(isinstance(caught.exception, bridge.BridgeOwnershipUnknown), stage in {'authentication', 'first-output'})
                self.assertGreater(os.fstat(owner.fileno()).st_ino, 0)
                with lock.open('a') as rival, self.assertRaises(BlockingIOError):
                    fcntl.flock(rival.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_first_authentication_mutations(self) -> None:
        for mutation in ('target', 'raw', 'directory', 'cache', 'settings'):
            with self.subTest(mutation=mutation):
                self.exercise('authentication', mutation)

    def test_terminal_publication_mutations(self) -> None:
        for mutation in ('target', 'raw', 'receipt', 'directory', 'cache', 'settings'):
            with self.subTest(mutation=mutation):
                self.exercise('publication', mutation)

    def test_first_output_callback_mutations(self) -> None:
        for mutation in ('target', 'raw', 'directory', 'cache', 'settings'):
            with self.subTest(mutation=mutation):
                self.exercise('first-output', mutation)

    def test_last_output_callback_mutations(self) -> None:
        for mutation in ('target', 'raw', 'directory', 'cache', 'settings'):
            with self.subTest(mutation=mutation):
                self.exercise('last-output', mutation)

    def test_final_authentication_mutations(self) -> None:
        for mutation in ('target', 'raw', 'receipt', 'directory', 'cache', 'settings'):
            with self.subTest(mutation=mutation):
                self.exercise('final-authentication', mutation)

    def test_actual_producer_generated_effects_are_admitted(self) -> None:
        self.exercise('producer-effects', '')


class PrelaunchIntegrityTests(unittest.TestCase):
    def exercise(self, warm: bool, mutation: str) -> None:
        from generic_process import ProcessResult, ProcessSpec, execute
        from generic_setup_prepare import prepare_warm
        from test_generic_setup_warm import WarmTests

        with tempfile.TemporaryDirectory() as temp:
            pair = None
            if warm:
                fixture = WarmTests()
                self.addCleanup(fixture.doCleanups)
                inputs, pair = fixture.fixture()
            else:
                inputs = Fixture(Path(temp).resolve()).inputs
            bash = next(t for t in inputs.tools if t.name == 'bash')
            _ = bash.path.write_text('#!/bin/sh\n/bin/mkdir -p rust/target/debug\nprintf bridge > rust/target/debug/libsim_bridge.dylib\n')
            inputs = replace(inputs, tools=tuple(replace(t, sha256=file_hash(t.path)) for t in inputs.tools))
            prepared = prepare_warm(inputs, pair) if pair is not None else prepare(inputs)
            called = False
            injected = False

            def publish(path: Path, value: dict[str, Json]) -> None:
                nonlocal injected
                atomic_json(path, value)
                if path.name != 'started.json' or mutation == 'none':
                    return
                injected = True
                if mutation == 'pair':
                    assert pair is not None
                    _ = pair.receipt.write_text('{}')
                elif mutation == 'source':
                    _ = (inputs.root/'project.godot').write_text('changed')
                else:
                    directories = {'target': inputs.root/'rust/target', 'cache': inputs.root/'.godot',
                                   'settings': inputs.settings, 'output': inputs.output}
                    directory = directories[mutation]
                    directory.mkdir(parents=True, exist_ok=True)
                    (directory/'prelaunch-added').mkdir()

            def launch(spec: ProcessSpec) -> ProcessResult:
                nonlocal called
                called = True
                if mutation != 'none':
                    raise RuntimeError('EXECUTE_REACHED_AFTER_PRELAUNCH_DRIFT')
                return execute(spec)

            lock = inputs.root.parent/'private-prelaunch-lock'
            with lock.open('w') as owner, patch.object(bridge, 'LOCK', lock), patch.object(bridge, 'qualify_wait', return_value=()), patch.object(bridge, 'atomic_json', side_effect=publish), patch.object(bridge, 'execute', side_effect=launch):
                fcntl.flock(owner.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                if mutation == 'none':
                    result = bridge.run_bridge_owned(prepared, unused_wait(prepared), owner.fileno())
                    self.assertEqual(result.status, 'PASS_STEP2_ENDPOINT_ONLY')
                    self.assertTrue(called)
                else:
                    with self.assertRaises(bridge.BridgeFailure) as caught:
                        _ = bridge.run_bridge_owned(prepared, unused_wait(prepared), owner.fileno())
                    self.assertTrue(injected)
                    self.assertFalse(called)
                    self.assertNotIsInstance(caught.exception, bridge.BridgeOwnershipUnknown)
                with lock.open('a') as rival, self.assertRaises(BlockingIOError):
                    fcntl.flock(rival.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_fresh_prelaunch_mutations_do_not_launch(self) -> None:
        for mutation in ('target', 'cache', 'settings', 'output', 'source'):
            with self.subTest(mutation=mutation):
                self.exercise(False, mutation)

    def test_warm_prelaunch_mutations_do_not_launch(self) -> None:
        for mutation in ('target', 'cache', 'settings', 'output', 'source', 'pair'):
            with self.subTest(mutation=mutation):
                self.exercise(True, mutation)

    def test_actual_fresh_and_warm_producers(self) -> None:
        for warm in (False, True):
            with self.subTest(warm=warm):
                self.exercise(warm, 'none')
