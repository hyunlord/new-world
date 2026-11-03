"""Private Python/shell fixtures; no native Cargo or simulation qualification."""
# These fixtures exercise real Bash -> private Cargo stand-in and real probe
# processes. Upstream private build/import/workspace/Clippy processes are real; pinned Cargo
# topology qualification is explicitly mocked.
import fcntl
import json
import sys
import tempfile
import unittest
from collections.abc import Callable
from contextlib import ExitStack
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_baseline_run as baseline
import generic_cli_godot_products as godot
import generic_embedded_build as build
import generic_embedded_workload as workload
import generic_fixtures as fixtures
import generic_workspace_run as runner
import test_generic_baseline_run as baseline_tests
import test_generic_cli_godot_products as godot_tests
import test_generic_cli_setup_handoff as handoff_tests
import test_generic_workspace_endpoints as endpoints
from generic_fixture_registry import load_registry
from generic_process import ProcessResult, ProcessSpec, atomic_json
from generic_setup_prepare import PreparedSetup
from generic_workspace_cases import ListingProcess, process_output
from mux2_gate_inputs import Json
from test_generic_cli_products import COMPILER
from test_generic_observations import receipt
from test_generic_setup_bridge import unused_wait


class RetentionTests(unittest.TestCase):
    def test_actual_objects_retained_and_no_return_stops_next_launch(self) -> None:
        fixture = baseline_tests.BaselineTests()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        plans, products = fixture.plans(('core.baseline10',))
        retained: list[tuple[ProcessSpec, ProcessResult]] = []
        def retain(spec: ProcessSpec, result: ProcessResult) -> None:
            retained.append((spec, result))
        with patch.object(fixtures, 'execute', side_effect=OSError('unreturned')) as launch:
            with self.assertRaisesRegex(OSError, 'unreturned'):
                _ = baseline.run_registered(fixture.registry, plans, products, fixture.root/'failed.json', retain=retain)
            self.assertEqual(launch.call_count, 1)
        self.assertEqual(retained, [])
        result = baseline.run_registered(fixture.registry, plans, products, fixture.root/'result.json', retain=retain)
        self.assertEqual(result.status, 'PASS')
        self.assertEqual(len(retained), 2)
        self.assertTrue(all(type(row.pid) is int and row.native_exit == 0 for _, row in retained))
        self.assertTrue(all(row.receipt == spec.output_root/'process.json' for spec, row in retained))



def actual_request(home: Path, stack: ExitStack, *, native_exit: int = 0, build_exit: int = 0) -> workload.Request:
    helper = godot_tests.GodotTests()
    _ = stack.callback(helper.doCleanups)
    original = endpoints.EndpointTests.fixture
    def source_fixture(instance: endpoints.EndpointTests, folder: Path, *, fail: bool = False) -> tuple[runner.WorkspaceRequest, PreparedSetup]:
        req, prepared = original(instance, folder, fail=fail)
        root = prepared.inputs.root
        gate = root/'tools/v8/gate.sh'
        _ = gate.write_text(gate.read_text()+'\n'+build.SOURCES[11][0]+'\n')
        _ = (root/'config.json').write_text('{}')
        _ = (root/'observation.json').write_text(json.dumps(receipt()))
        _ = (root/'tools/v8/identity_removed_keys.txt').write_bytes(Path(baseline_tests.__file__).with_name('identity_removed_keys.txt').read_bytes())
        cargo = req.spec.executable
        source = cargo.read_text()
        source = source.replace('if sys.argv[1] == "build":', 'if sys.argv[1] == "build" and "--release" in sys.argv and '+str(build_exit)+':\n raise SystemExit('+str(build_exit)+')\nif sys.argv[1] == "build":')
        _ = cargo.write_text(source)
        return req, prepared
    body = '#!'+str(Path(sys.executable).resolve())+'\nimport sys\nfrom pathlib import Path\n'
    body += 'if Path.cwd().name != "base" and '+str(native_exit)+': sys.exit('+str(native_exit)+')\n'
    body += baseline_tests.CHILD
    compiler = COMPILER.replace("b'synthetic executable only'", repr(body.encode()))
    with patch.object(endpoints.EndpointTests, 'fixture', source_fixture), patch.object(handoff_tests, 'COMPILER', compiler):
        owner, products, base, candidate, native = helper.fixture(home)
    setup = godot.begin_godot_setup(products,base,candidate)
    lock=home/'runtime/lane'
    for name in ('generic_setup_bridge.LOCK','gate_oracle_child.LOCK','generic_workspace_run.LOCK'):
        _ = stack.enter_context(patch(name,lock))
    for name in ('generic_setup_bridge.qualify_wait','generic_setup_import.qualify_wait','generic_setup_clippy.qualify_clippy'):
        _ = stack.enter_context(patch(name,return_value=()))
    _ = stack.enter_context(patch.object(build,'qualify_wait',return_value=()))
    ready=godot.run_base_setup_owned(setup,unused_wait(base),home/'base-imports',native.held_lane_fd)
    witness=godot.base_probe_witness(ready)
    prior=owner.finish(home,candidate,native)
    output=home/'run'
    return workload.Request(witness,build.EmbeddedRequest(candidate,11,prior,unused_wait(candidate),output/'candidate-build',native.held_lane_fd),
        load_registry(Path(baseline_tests.__file__).with_name('gate_fixtures.json')),Path(baseline_tests.__file__).with_name('gate_fixtures.json'),output)


class WorkloadTests(unittest.TestCase):
    def test_real_pair_and_historical_checks_do_not_refresh_target(self) -> None:
        with tempfile.TemporaryDirectory() as tmp,ExitStack() as stack:
            home=Path(tmp).resolve();req=actual_request(home,stack)
            result=workload.run_owned(req)
            self.assertEqual(result.status,'PASS_REGISTERED_STEP11_ONLY')
            with self.assertRaises(ValueError):workload.check_endpoint(replace(result))
            self.assertEqual(len(result.builds),1)
            self.assertEqual(len(result.children),2)
            self.assertTrue(all(child.binding.result.native_exit == 0 for child in result.children))
            self.assertTrue(all('--days' in child.binding.spec.argv and '120' in child.binding.spec.argv for child in result.children))
            self.assertEqual(result.original_shell_envelope,'NOT_QUALIFIED')
            probe=req.candidate.prepared.inputs.root/'rust/target/release/worldsim_probe'
            _=probe.write_text('later separately owned producer')
            workload.check_endpoint(result)
            with self.assertRaises(ValueError):workload.check_endpoint(replace(result,children=()))
            _=(result.children[0].output.root/'unexpected').mkdir()
            with self.assertRaises(ValueError):workload.check_endpoint(result)

    def test_returned_native_nonzero_is_settled_failure(self) -> None:
        with tempfile.TemporaryDirectory() as tmp,ExitStack() as stack:
            home=Path(tmp).resolve();req=actual_request(home,stack,native_exit=9);lock=home/'runtime/lane'
            result=workload.run_owned(req)
            self.assertEqual(result.status,'FAIL')
            self.assertEqual(result.children[-1].binding.result.native_exit,9)
            workload.check_endpoint(result)
            with lock.open('a') as rival,self.assertRaises(BlockingIOError):fcntl.flock(rival.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)

    def test_build_failure_suppresses_entire_registered_tail(self) -> None:
        with tempfile.TemporaryDirectory() as tmp,ExitStack() as stack:
            home=Path(tmp).resolve();req=actual_request(home,stack,build_exit=7)
            with patch.object(baseline,'run_registered') as tail,self.assertRaises(workload.OwnershipUnknown):_=workload.run_owned(req)
            tail.assert_not_called()
            self.assertFalse((req.output/'runs').exists())
            self.assertIn('"native_exit": 7',(req.output/'candidate-build/process/process.json').read_text())

    def test_unreturned_tail_preserves_owner_and_stops_next_child(self) -> None:
        with tempfile.TemporaryDirectory() as tmp,ExitStack() as stack:
            home=Path(tmp).resolve();req=actual_request(home,stack);lock=home/'runtime/lane';primary=OSError('unreturned real boundary')
            with patch.object(fixtures,'execute',side_effect=primary) as launch,self.assertRaises(workload.OwnershipUnknown) as caught:_=workload.run_owned(req)
            self.assertIs(caught.exception.primary,primary)
            self.assertEqual(launch.call_count,1)
            with lock.open('a') as rival,self.assertRaises(BlockingIOError):fcntl.flock(rival.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)

    def test_first_auth_and_publication_cannot_adopt_new_outputs(self) -> None:
        for fault in ('first-auth-file','first-auth-directory','receipt','publication-directory','final-callback'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory() as tmp,ExitStack() as stack:
                home=Path(tmp).resolve();req=actual_request(home,stack)
                def auth(binding: ListingProcess,argv: tuple[str, ...],cwd: Path,environment: dict[str,str],required: dict[str,str],fault: str = fault) -> bytes:
                    result=process_output(binding,argv,cwd,environment,required)
                    if fault=='first-auth-file':_=(binding.spec.output_root/'extra').write_text('late')
                    if fault=='first-auth-directory':(binding.spec.output_root/'extra').mkdir()
                    return result
                def publish(path: Path,row: dict[str,Json],fault: str = fault,req: workload.Request = req) -> None:
                    atomic_json(path,row)
                    if path==req.output/'receipt.json':
                        if fault=='receipt':_=path.write_text('{}')
                        if fault=='publication-directory':(req.output/'extra').mkdir()
                original_check=workload.check_endpoint
                def final_check(endpoint: workload.Endpoint,fault: str = fault,original_check: Callable[[workload.Endpoint],None] = original_check) -> None:
                    original_check(endpoint)
                    if fault=='final-callback':(endpoint.children[0].output.root/'extra').mkdir()
                with patch.object(workload,'process_output',side_effect=auth),patch.object(workload,'atomic_json',side_effect=publish),patch.object(workload,'check_endpoint',side_effect=final_check),self.assertRaises(workload.WorkloadFailure):_=workload.run_owned(req)
                self.assertTrue((req.output/'failure.json').is_file())

    def test_wrong_step_and_unadmitted_generated_change_reject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp,ExitStack() as stack:
            home=Path(tmp).resolve();req=actual_request(home,stack)
            with self.assertRaises(workload.WorkloadFailure):_=workload.run_owned(replace(req,candidate=replace(req.candidate,step=10)))
            (req.candidate.prepared.inputs.root/'rust/target/unadmitted').mkdir()
            with patch.object(build,'execute') as launch,self.assertRaises(workload.WorkloadFailure):_=workload.run_owned(req)
            launch.assert_not_called()


    def test_final_lane_callback_cannot_adopt_target_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as tmp, ExitStack() as stack:
            home=Path(tmp).resolve();req=actual_request(home,stack)
            from generic_setup_bridge import held_lane
            def changed(descriptor: int) -> None:
                held_lane(descriptor)
                if (req.output/'receipt.json').exists():
                    (req.candidate.prepared.inputs.root/'rust/target/late').mkdir()
            with patch.object(workload,'held_lane',side_effect=changed),self.assertRaises(workload.WorkloadFailure):
                _=workload.run_owned(req)

    def test_final_source_callback_cannot_adopt_untracked_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp, ExitStack() as stack:
            home=Path(tmp).resolve();req=actual_request(home,stack)
            from generic_setup_bridge import immutable_inputs
            calls=0
            victim=req.candidate.prepared.inputs.root/'late-source-empty'
            def changed(prepared: PreparedSetup) -> None:
                nonlocal calls
                immutable_inputs(prepared)
                if (req.output/'receipt.json').exists():
                    calls+=1
                    if calls==2:victim.mkdir()
            with patch.object(workload,'immutable_inputs',side_effect=changed), self.assertRaises(workload.WorkloadFailure):
                _=workload.run_owned(req)
            self.assertTrue(victim.is_dir())
            self.assertTrue((req.output/'failure.json').is_file())

    def test_final_source_callback_cannot_adopt_external_tool_mode(self) -> None:
        with tempfile.TemporaryDirectory() as tmp, ExitStack() as stack:
            home=Path(tmp).resolve();req=actual_request(home,stack)
            from generic_setup_bridge import immutable_inputs
            calls=0
            victim=next(tool.path for tool in req.candidate.prepared.inputs.tools if tool.name=='godot')
            self.assertTrue(victim.is_relative_to(home) and victim.resolve().is_relative_to(home))
            self.assertFalse(victim.is_symlink())
            original_mode=victim.stat().st_mode
            def changed(prepared: PreparedSetup) -> None:
                nonlocal calls
                immutable_inputs(prepared)
                if (req.output/'receipt.json').exists():
                    calls+=1
                    if calls==2:victim.chmod(0o700 if original_mode & 0o777 != 0o700 else 0o755)
            try:
                with patch.object(workload,'immutable_inputs',side_effect=changed), self.assertRaises(workload.WorkloadFailure):
                    _=workload.run_owned(req)
                self.assertNotEqual(victim.stat().st_mode,original_mode)
                self.assertTrue((req.output/'failure.json').is_file())
            finally:
                victim.chmod(original_mode)
