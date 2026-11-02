"""Tiny Clippy producer transitions; historical admission is an explicit fixture mock."""
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_setup_clippy as clippy
from generic_gate import GateContractError
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_setup_prepare import PreparedSetup
from generic_workspace_capture import WorkspaceCapture
from generic_workspace_endpoints import (
    WorkspaceEndpoint,
    WorkspaceRequest,
    WorkspaceResult,
    anchor,
    check_endpoint,
    generated,
)
from generic_workspace_tests import Inventory
from mux2_gate_inputs import Json
from test_generic_setup_bridge import unused_wait
from test_generic_setup_clippy import fixture


class EndpointTests(unittest.TestCase):
    def test_transition_and_boundary_faults(self) -> None:
        for fault in ('none', 'before', 'producer-cache', 'producer-settings', 'authentication', 'publication', 'output-publication', 'history-publication', 'outer-publication', 'receipt-publication', 'predecessor-publication', 'wrong-step', 'unadmitted'):
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                home = Path(temporary).resolve()
                prepared = fixture(home)
                history = home/'history'; history.mkdir()
                old_receipt = history/'result.json'; _ = old_receipt.write_text('{}')
                # Historical admission is mocked, but the retained objects have real types.
                spec = ProcessSpec(('/bin/bash',), prepared.inputs.root, Path('/bin/bash'), file_hash(Path('/bin/bash')),
                                   prepared.inputs.root, (), history/'process', prepared.inputs.environment)
                old_request = WorkspaceRequest(5, 'candidate', 'synthetic', spec, Inventory((), ()), old_receipt,
                                               file_hash(old_receipt), history, -1)
                old_result = WorkspaceResult(old_receipt, 'OBSERVED_CASES_ONLY',
                    WorkspaceCapture((), (), (), (), 'PASS', 0, 'PASS', 'COMPLETE', 'PASS'))
                state = generated(prepared)
                prior = WorkspaceEndpoint(old_request, old_result, prepared, 5, 'candidate', 'synthetic',
                                          old_receipt, file_hash(old_receipt), state, state, (anchor(history),))
                if fault == 'wrong-step': prior = replace(prior, step=4)
                request = clippy.ClippyRequest(prepared, clippy.ClippyWaitEvidence(unused_wait(prepared), ()), home/'step6', -1, prior)
                lock = home/'lock'
                def producer(spec: ProcessSpec, fault: str = fault, prepared: PreparedSetup = prepared) -> ProcessResult:
                    result = execute(spec)
                    if fault.startswith('producer-'):
                        directory = prepared.inputs.root/'.godot' if fault.endswith('cache') else prepared.inputs.settings
                        directory.mkdir(exist_ok=True); (directory/'unexpected-empty').mkdir()
                    return result
                calls = 0
                def qualify(_prepared: PreparedSetup, _wait: clippy.ClippyWaitEvidence, fault: str = fault, prepared: PreparedSetup = prepared) -> tuple[tuple[Path, str], ...]:
                    nonlocal calls
                    calls += 1
                    if fault == 'authentication' and calls == 2:
                        (prepared.inputs.root/'rust/target/late-empty').mkdir()
                    return ()
                def publish(path: Path, row: dict[str, Json], fault: str = fault, prepared: PreparedSetup = prepared, history: Path = history) -> None:
                    atomic_json(path, row)
                    if path.name == 'receipt.json':
                        if fault == 'publication': (prepared.inputs.root/'rust/target/late-empty').mkdir()
                        if fault == 'output-publication': (path.parent/'process/late-empty').mkdir()
                        if fault == 'history-publication': (history/'late-empty').mkdir()
                        if fault == 'outer-publication': (path.parent/'late-empty').mkdir()
                        if fault == 'receipt-publication': _ = path.write_text(path.read_text().replace('OBSERVED_ENDPOINTS_ONLY', 'forged'))
                        if fault == 'predecessor-publication':
                            _ = path.write_text(path.read_text().replace(str(row['predecessor_sha256']), '0'*64))
                if fault == 'before': (prepared.inputs.root/'rust/target/between').mkdir()
                with lock.open('w') as lane, patch('generic_setup_bridge.LOCK', lock), patch.object(clippy, 'check_endpoint', wraps=check_endpoint if fault == 'unadmitted' else None), patch.object(clippy, 'qualify_clippy', qualify), patch.object(clippy, 'execute', producer), patch.object(clippy, 'atomic_json', publish):
                    request = replace(request, descriptor=lane.fileno())
                    if fault != 'none':
                        with self.assertRaises(clippy.ClippyFailure): _ = clippy.run_clippy_request(request)
                    else:
                        result = clippy.run_clippy_request(request)
                        self.assertIsNotNone(result.endpoint)
                        assert result.endpoint is not None
                        self.assertIs(result.endpoint.request, request)
                        self.assertIs(result.endpoint.predecessor, prior)
                        self.assertEqual(result.endpoint.result.receipt, result.receipt)
                        self.assertEqual(result.endpoint.result.pid, result.pid)
                        self.assertFalse(result.build_product_authority)
                        clippy.check_clippy_endpoint(result.endpoint)
                        (prepared.inputs.root/'rust/target/later-producer').mkdir()
                        clippy.check_clippy_endpoint(result.endpoint)
                        (result.receipt.parent/'late-empty').mkdir()
                        with self.assertRaises(GateContractError): clippy.check_clippy_endpoint(result.endpoint)
