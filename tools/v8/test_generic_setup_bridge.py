"""Tiny shell processes only; production Cargo wait qualification is explicitly mocked."""
import fcntl
import os
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError, document
from generic_process import ProcessResult, ProcessSpec, execute, file_hash
from generic_setup_bridge import (
    BridgeFailure,
    BridgeOwnershipUnknown,
    CargoWaitEvidence,
    run_bridge_owned,
)
from generic_setup_prepare import PreparedSetup, Tool, prepare
from generic_workspace_cases import ListingProcess
from mux2_gate_inputs import EvidenceError
from test_generic_setup_prepare import Fixture


def unused_wait(prepared: PreparedSetup) -> CargoWaitEvidence:
    """Never admitted: qualification is patched only for shell lifecycle fixture tests."""
    tool = prepared.inputs.tools[0]
    spec = ProcessSpec((str(tool.path),), prepared.inputs.root, tool.path, tool.sha256,
                       prepared.inputs.root, (), prepared.inputs.output/'unused')
    binding = ListingProcess(spec, ProcessResult(spec.output_root/'process.json', 'FAIL', None, None), '0'*64)
    return CargoWaitEvidence((), binding, binding)


class BridgeTests(unittest.TestCase):
    def test_missing_lane_rejected_before_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            prepared = prepare(f.inputs)
            with self.assertRaises(BridgeFailure) as caught:
                _ = run_bridge_owned(prepared, unused_wait(prepared), -1)
            self.assertIsInstance(caught.exception.primary, GateContractError)
            self.assertFalse(f.inputs.output.exists())

    def run_fixture(self, code: int, *, bridge: bool = True) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            (f.bin/'bash').unlink()
            commands = '/bin/mkdir -p target/debug\nprintf bridge > target/debug/libsim_bridge.dylib\n' if bridge else ''
            _ = (f.bin/'cargo').write_text('#!/bin/sh\n'+commands+f'exit {code}\n')
            tools = tuple(Tool(t.name, Path('/bin/bash') if t.name == 'bash' else t.path,
                               file_hash(Path('/bin/bash') if t.name == 'bash' else t.path)) for t in f.inputs.tools)
            env = tuple((k, v+':/bin' if k == 'PATH' else v) for k,v in f.inputs.environment)
            prepared = prepare(replace(f.inputs, tools=tools, environment=env))
            lock = f.home/'lock'
            with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_bridge.qualify_wait', return_value=()):
                fcntl.flock(owner.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                if code or not bridge:
                    expected = BridgeOwnershipUnknown if code else BridgeFailure
                    with self.assertRaises(expected):
                        _ = run_bridge_owned(prepared, unused_wait(prepared), owner.fileno())
                else:
                    result = run_bridge_owned(prepared, unused_wait(prepared), owner.fileno())
                    self.assertEqual(result.status, 'PASS_STEP2_ENDPOINT_ONLY')
                    self.assertFalse(result.build_product_authority)
                self.assertGreater(os.fstat(owner.fileno()).st_ino, 0)
                with lock.open('a') as rival, self.assertRaises(BlockingIOError):
                    fcntl.flock(rival.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                raw = document((f.inputs.output/'receipt.json').read_text())
                self.assertEqual(raw.field('native_exit').value, code)
                native = document((f.inputs.output/'process/process.json').read_text())
                self.assertEqual(native.field('argv').value, ['/bin/bash', '-c', 'cd rust && cargo build -p sim-bridge'])
                self.assertIsNotNone(native.field('pid').value)
                self.assertEqual(raw.field('ownership').value, 'UNKNOWN_KEEP_CALLER_LANE' if code else 'NORMAL_CARGO_RETURN' if bridge else 'NO_INFLIGHT_OWNED_CALL')

    def test_actual_shell_success_retains_original_recipe_and_lane(self) -> None:
        self.run_fixture(0)

    def test_actual_nonzero_retains_raw_failure_and_lane(self) -> None:
        self.run_fixture(17)

    def test_normal_return_missing_bridge_is_failure_not_unknown(self) -> None:
        self.run_fixture(0, bridge=False)

    def test_unreturned_exception_retained_without_lane_release(self) -> None:
        for original in (KeyboardInterrupt(), SystemExit(3), RuntimeError('fixture')):
            with self.subTest(original=type(original)), tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                prepared = prepare(f.inputs)
                lock = f.home/'lock'
                with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_bridge.qualify_wait', return_value=()), patch('generic_setup_bridge.execute', side_effect=original):
                    with self.assertRaises(BridgeOwnershipUnknown) as caught:
                        _ = run_bridge_owned(prepared, unused_wait(prepared), owner.fileno())
                    self.assertIs(caught.exception.primary, original)
                    with lock.open('a') as rival, self.assertRaises(BlockingIOError):
                        fcntl.flock(rival.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                    row = document((f.inputs.output/'receipt.json').read_text())
                    self.assertEqual(row.field('ownership').value, 'UNKNOWN_KEEP_CALLER_LANE')
                    self.assertIsNone(row.field('target_inventory').value)

    def test_corrupted_actual_process_output_remains_unknown(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            prepared = prepare(f.inputs)
            lock = f.home/'lock'

            def corrupt(spec: ProcessSpec) -> ProcessResult:
                result = execute(spec)
                _ = (spec.output_root/'stdout.log').write_text('late mutation')
                return result

            # Tiny fixture bash exits97: force zero to reach receipt authentication instead.
            _ = (f.bin/'bash').write_text('#!/bin/sh\nexit 0\n')
            prepared = prepare(replace(f.inputs, tools=tuple(Tool(t.name,t.path,file_hash(t.path)) for t in f.inputs.tools)))
            with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_bridge.qualify_wait', return_value=()), patch('generic_setup_bridge.execute', side_effect=corrupt):
                with self.assertRaises(BridgeOwnershipUnknown) as caught:
                    _ = run_bridge_owned(prepared, unused_wait(prepared), owner.fileno())
                self.assertIsInstance(caught.exception.primary, EvidenceError)
                self.assertEqual(document((f.inputs.output/'receipt.json').read_text()).field('native_exit').value, 0)

    def test_production_wait_profile_is_not_a_caller_pass_boolean(self) -> None:
        from generic_setup_bridge import qualify_wait
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            prepared = prepare(f.inputs)
            with self.assertRaisesRegex(GateContractError, 'incomplete source anchors'):
                _ = qualify_wait(prepared, unused_wait(prepared))

    def test_diagnostic_failure_preserves_unreturned_exception(self) -> None:
        from generic_process import atomic_json
        from mux2_gate_inputs import Json
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            prepared = prepare(f.inputs)
            lock = f.home/'lock'
            original = KeyboardInterrupt()
            diagnostic = OSError('fixture failure writer')

            def fail_terminal(path: Path, value: dict[str, Json]) -> None:
                if path.name == 'receipt.json':raise diagnostic
                atomic_json(path, value)

            with lock.open('w') as owner, patch('generic_setup_bridge.LOCK', lock), patch('generic_setup_bridge.qualify_wait', return_value=()), patch('generic_setup_bridge.execute', side_effect=original), patch('generic_setup_bridge.atomic_json', side_effect=fail_terminal):
                with self.assertRaises(BridgeOwnershipUnknown) as caught:
                    _ = run_bridge_owned(prepared, unused_wait(prepared), owner.fileno())
                self.assertIs(caught.exception.primary, original)
                self.assertIs(caught.exception.secondary, diagnostic)
                self.assertFalse((f.inputs.output/'receipt.json').exists())
                self.assertEqual(document((f.inputs.output/'started.json').read_text()).field('status').value, 'FAIL')
                with lock.open('a') as rival, self.assertRaises(BlockingIOError):
                    fcntl.flock(rival.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_synthetic_authenticated_version_profile_rejects_late_bytes(self) -> None:
        from generic_setup_bridge import COMMITS, qualify_wait
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            for name, commit in COMMITS.items():
                _ = (f.bin/name).write_text('#!/bin/sh\nprintf "commit-hash: '+commit+'\\nrelease: 1.93.1\\nhost: aarch64-apple-darwin\\n"\n')
            prepared = prepare(replace(f.inputs, tools=tuple(Tool(t.name,t.path,file_hash(t.path)) for t in f.inputs.tools)))
            versions: list[ListingProcess] = []
            for name in ('cargo', 'rustc'):
                tool = next(t for t in prepared.inputs.tools if t.name == name)
                spec = ProcessSpec((str(tool.path),'--version','--verbose'), f.root, tool.path, tool.sha256,
                                   f.root, prepared.materialized, f.home/(name+'-version'))
                result = execute(spec)
                self.assertEqual(result.native_exit, 0)
                versions.append(ListingProcess(spec,result,file_hash(result.receipt)))
            audit = f.home/'synthetic-audit'
            _ = audit.write_text('synthetic source profile, not native qualification')
            wait = CargoWaitEvidence((audit,), versions[0], versions[1])
            digests = {t.name:t.sha256 for t in prepared.inputs.tools if t.name in {'cargo','rustc','bash'}}
            with patch('generic_setup_bridge.WAIT_SOURCES', {audit.name:file_hash(audit)}), patch('generic_setup_bridge.TOOL_DIGESTS', digests):
                bound = qualify_wait(prepared, wait)
                self.assertIn((audit,file_hash(audit)), bound)
                _ = (versions[0].spec.output_root/'stdout.log').write_text('late forged version')
                with self.assertRaises(EvidenceError):
                    _ = qualify_wait(prepared, wait)

    def test_source_bound_cargo_override_is_not_a_normal_wait_profile(self) -> None:
        from generic_setup_bridge import qualify_wait
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            config = f.root/'rust/.cargo/config.toml'
            _ = config.write_text('[build]\nrustc="/unreviewed/compiler"\n')
            _ = f.git('add', '.')
            _ = f.git('-c','user.name=Test','-c','user.email=t@example.invalid','commit','-qm','bound but unsupported config')
            changed = replace(f.inputs, source_sha=f.git('rev-parse','HEAD').strip(),
                              configs=((str(config),file_hash(config)),),
                              effective_inputs=tuple((name,file_hash(f.root/name)) for name,_ in f.inputs.effective_inputs))
            prepared = prepare(changed)
            audit = f.home/'synthetic-audit'
            _ = audit.write_text('synthetic source profile')
            stub = unused_wait(prepared)
            wait = replace(stub, sources=(audit,))
            digests = {t.name:t.sha256 for t in prepared.inputs.tools if t.name in {'cargo','rustc','bash'}}
            with patch('generic_setup_bridge.WAIT_SOURCES', {audit.name:file_hash(audit)}), patch('generic_setup_bridge.TOOL_DIGESTS', digests), self.assertRaisesRegex(GateContractError, 'unreviewed build config'):
                _ = qualify_wait(prepared, wait)
