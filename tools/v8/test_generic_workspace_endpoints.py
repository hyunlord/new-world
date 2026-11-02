"""Tiny actual import/workspace chains; Cargo wait qualification explicitly mocked."""
import shutil
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import gate_oracle_child as child
import generic_workspace_endpoints as endpoints
import generic_workspace_run as runner
from generic_gate import GateContractError, document
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_setup_bridge import run_bridge_owned
from generic_setup_import import ImportOwner, PriorBridge, start_import_pair
from generic_setup_prepare import PreparedSetup, prepare
from generic_workspace_capture import CapturePlan, WorkspaceCapture, reconcile_capture
from mux2_gate_inputs import Json
from test_generic_setup_bridge import unused_wait
from test_generic_setup_import import make_fixture
from test_generic_workspace_run import RuntimeTests


class EndpointTests(unittest.TestCase):
    def fixture(self, home: Path, *, fail: bool = False) -> tuple[runner.WorkspaceRequest, PreparedSetup]:
        runtime = home/'runtime'; runtime.mkdir()
        fixture = RuntimeTests()
        self.addCleanup(fixture.doCleanups)
        request = fixture.fixture(runtime, fail=fail)
        setup = home/'setup'; setup.mkdir()
        f = make_fixture(setup, 0, 0)
        child_source=f.root/'tools/v8/gate_oracle_child.py'
        _=child_source.write_text(child_source.read_text().replace(str(setup/'lock'),str(runtime/'lane')))
        _ = shutil.copytree(request.spec.input_root/'rust/crate', f.root/'rust/crate')
        cargo = f.bin/'cargo'
        body = request.spec.executable.read_text().replace(str(request.spec.input_root), str(f.root))
        lines = body.splitlines()
        body = lines[0]+'\nimport sys,pathlib\nif sys.argv[1] == "build":\n pathlib.Path("target/debug").mkdir(parents=True)\n pathlib.Path("target/debug/libsim_bridge.dylib").write_text("bridge")\n raise SystemExit(0)\n'+'\n'.join(lines[1:])+'\n'
        _ = cargo.write_text(body)
        _ = f.git('add', '.')
        _ = f.git('-c', 'user.name=Test', '-c', 'user.email=t@example.invalid', 'commit', '-qm', 'tiny workspace')
        f.inputs = replace(f.inputs, source_sha=f.git('rev-parse', 'HEAD').strip(),
            tools=tuple(replace(t, sha256=file_hash(cargo)) if t.name == 'cargo' else t for t in f.inputs.tools),
            environment=tuple((k, '8' if k == 'CARGO_BUILD_JOBS' else v) for k,v in f.inputs.environment
                              if k not in {'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'})+(('RUSTC_WRAPPER',''),('RUSTC_WORKSPACE_WRAPPER','')),
            effective_inputs=tuple(sorted((name,file_hash(f.root/name)) for name in (*f.git('ls-files').splitlines(),'rust/Cargo.lock'))))
        prepared = prepare(f.inputs)
        manifest_text = request.manifest.read_text().replace(str(request.spec.input_root), str(f.root))
        _ = request.manifest.write_text(manifest_text)
        artifacts = tuple(replace(item, target=replace(item.target, source=f.root/'rust/crate/lib.rs', manifest=f.root/'rust/crate/Cargo.toml')) for item in request.inventory.executables)
        env = dict(prepared.inputs.environment); env.update(dict(request.spec.environment)); env['CARGO_CAPTURE_MANIFEST_SHA256'] = file_hash(request.manifest)
        external = dict(request.spec.external_inputs); _ = external.pop(request.spec.executable)
        external.update(dict(prepared.external)); external[request.manifest] = file_hash(request.manifest)
        spec = replace(request.spec, executable=cargo, executable_sha256=file_hash(cargo),
            argv=runner.workspace_argv(str(cargo),4), input_root=f.root, cwd=f.root/'rust',
            inputs=prepared.materialized, external_inputs=tuple(sorted(external.items())), environment=tuple(sorted(env.items())))
        request = replace(request, spec=spec, manifest_sha256=file_hash(request.manifest), inventory=replace(request.inventory,executables=artifacts))
        return request, prepared

    def test_actual_chain_and_late_generated_drift(self) -> None:
        for mutation in ('none', 'target', 'cache', 'settings', 'history', 'between', 'failure', 'environment', 'protected', 'publication', 'native-fail', 'validation', 'publication-evidence'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as tmp:
                home=Path(tmp).resolve(); request,prepared=self.fixture(home, fail=mutation=='native-fail')
                lock=home/'runtime/lane'
                with patch('generic_setup_bridge.LOCK',lock), patch.object(child,'LOCK',lock), patch.object(runner,'LOCK',lock), \
                     patch('generic_setup_bridge.qualify_wait',return_value=()), patch('generic_setup_import.qualify_wait',return_value=()):
                    bridge=run_bridge_owned(prepared,unused_wait(prepared),request.held_lane_fd)
                    owner=start_import_pair(prepared,PriorBridge(bridge,file_hash(bridge.receipt),unused_wait(prepared)),home/'imports',request.held_lane_fd)
                    imported=owner.finish()
                    prior=endpoints.ImportPredecessor(owner,imported,file_hash(imported.receipt))
                    request=replace(request,chain=endpoints.WorkspaceChain(prepared,prior))
                    if mutation == 'between': (prepared.inputs.root/'.godot/between').mkdir()
                    if mutation == 'failure': _=(owner.output/'failure.json').write_text('{}')
                    if mutation == 'environment':
                        request=replace(request,spec=replace(request.spec,environment=tuple((k,'9' if k=='CARGO_BUILD_JOBS' else v) for k,v in request.spec.environment)))
                    if mutation in {'between','failure','environment'}:
                        with self.assertRaises(GateContractError): _=runner.run_workspace_owned(request)
                        self.assertFalse(request.output.exists())
                        continue
                    def changed(spec: ProcessSpec, prepared: PreparedSetup = prepared, mutation: str = mutation, owner: ImportOwner = owner, request: runner.WorkspaceRequest = request) -> ProcessResult:
                        result=execute(spec)
                        if spec.output_root.name=='parent':
                            _=(prepared.inputs.root/'rust/target/producer-new').write_text('permitted producer output')
                        if spec.output_root.name=='parent' and mutation=='protected':
                            _=request.inventory.executables[0].binary.write_text('changed protected binary')
                        if spec.output_root.name=='listing-0-ignored' and mutation in {'target','cache','settings','history'}:
                            root={'target':prepared.inputs.root/'rust/target','cache':prepared.inputs.root/'.godot',
                                  'settings':prepared.inputs.settings,'history':owner.output}[mutation]
                            root.mkdir(exist_ok=True); (root/'late-empty').mkdir()
                        return result
                    def published(path: Path, value: dict[str, Json], mutation: str = mutation, prepared: PreparedSetup = prepared) -> None:
                        atomic_json(path,value)
                        if mutation=='publication' and path.name=='result.json':
                            prepared.inputs.settings.mkdir(exist_ok=True)
                            (prepared.inputs.settings/'late').mkdir()
                        if mutation=='publication-evidence' and path.name=='result.json':
                            (path.parent/'listing-0-all/late').mkdir()
                    def validated(plan: CapturePlan, parent: ProcessResult, mutation: str = mutation, prepared: PreparedSetup = prepared) -> WorkspaceCapture:
                        captured=reconcile_capture(plan,parent)
                        if mutation=='validation': (prepared.inputs.root/'.godot/validator-added').mkdir()
                        return captured
                    with patch.object(runner,'execute',changed), patch.object(runner,'atomic_json',published), patch.object(runner,'reconcile_capture',validated):
                        if mutation in {'none','native-fail'}:
                            result=runner.run_workspace_owned(request)
                            self.assertEqual(result.status, 'FAIL' if mutation=='native-fail' else 'OBSERVED_CASES_ONLY')
                            self.assertIsNotNone(result.endpoint)
                            assert result.endpoint is not None
                            endpoints.check_endpoint(result.endpoint)
                            with self.assertRaises(GateContractError): endpoints.check_endpoint(replace(result.endpoint,arm='base'))
                            self.assertNotEqual(result.endpoint.before,result.endpoint.after)
                            self.assertFalse(result.publication_authority)
                            manifest_data=document(request.manifest.read_text()).mapping()
                            next_output=home/'release'; next_manifest=home/'release-manifest.json'
                            manifest_data.update(step=5,profile='release',output_root=str(next_output/'capture'))
                            atomic_json(next_manifest,manifest_data)
                            external=dict(request.spec.external_inputs); _=external.pop(request.manifest)
                            external[next_manifest]=file_hash(next_manifest)
                            environment=dict(request.spec.environment)
                            environment.update(CARGO_CAPTURE_MANIFEST=str(next_manifest),CARGO_CAPTURE_MANIFEST_SHA256=file_hash(next_manifest))
                            next_request=replace(request,step=5,output=next_output,manifest=next_manifest,
                                manifest_sha256=file_hash(next_manifest),
                                chain=endpoints.WorkspaceChain(prepared,result.endpoint),
                                inventory=replace(request.inventory,executables=tuple(replace(item,step=5,profile='release') for item in request.inventory.executables)),
                                spec=replace(request.spec,argv=runner.workspace_argv(str(request.spec.executable),5),
                                    output_root=next_output/'parent',external_inputs=tuple(sorted(external.items())),environment=tuple(sorted(environment.items()))))
                            released=runner.run_workspace_owned(next_request)
                            self.assertIsNotNone(released.endpoint)
                            # Historical validation must not re-run old import target checks.
                            _=(prepared.inputs.root/'rust/target/next-producer').write_text('later')
                            endpoints.check_endpoint(result.endpoint)
                            with self.assertRaises(GateContractError): _=endpoints.begin_workspace(next_request)
                        else:
                            with self.assertRaises((runner.WorkspaceSettledFailure,runner.WorkspaceOwnershipUnknown)):
                                _=runner.run_workspace_owned(request)
                            self.assertEqual((request.output/'result.json').exists(), mutation in {'publication','publication-evidence'})
                            if mutation in {'publication','publication-evidence'}: self.assertTrue((request.output/'endpoint-failure.json').exists())

    def test_unbound_checkpoint_is_not_an_import_owner(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            request,prepared=self.fixture(Path(tmp).resolve())
            self.assertIsNone(request.chain)
            with patch.object(runner,'LOCK',Path(tmp).resolve()/'runtime/lane'), self.assertRaises(GateContractError):
                _=runner.checked(request)
            with self.assertRaises(GateContractError): _=endpoints.begin_workspace(request)
            self.assertFalse(prepared.inputs.output.exists())

    def test_publication_predecessor_and_chronology_corruption_reject(self) -> None:
        for step, field, replacement in ((step, field, replacement) for step in (4, 5)
                for field, replacement in (('predecessor_sha256', '0'*64), ('chronology', 'UNQUALIFIED'))):
            with self.subTest(step=step, field=field), tempfile.TemporaryDirectory() as tmp:
                home = Path(tmp).resolve(); request, prepared = self.fixture(home)
                lock = home/'runtime/lane'
                with patch('generic_setup_bridge.LOCK', lock), patch.object(child, 'LOCK', lock), patch.object(runner, 'LOCK', lock), \
                        patch('generic_setup_bridge.qualify_wait', return_value=()), patch('generic_setup_import.qualify_wait', return_value=()):
                    bridge = run_bridge_owned(prepared, unused_wait(prepared), request.held_lane_fd)
                    owner = start_import_pair(prepared, PriorBridge(bridge, file_hash(bridge.receipt), unused_wait(prepared)), home/'imports', request.held_lane_fd)
                    imported = owner.finish()
                    prior = endpoints.ImportPredecessor(owner, imported, file_hash(imported.receipt))
                    request = replace(request, chain=endpoints.WorkspaceChain(prepared, prior))
                    if step == 5:
                        debug = runner.run_workspace_owned(request)
                        assert debug.endpoint is not None
                        output = home/'release'; manifest = home/'release-manifest.json'
                        data = document(request.manifest.read_text()).mapping()
                        data.update(step=5, profile='release', output_root=str(output/'capture'))
                        atomic_json(manifest, data)
                        external = dict(request.spec.external_inputs); _ = external.pop(request.manifest)
                        external[manifest] = file_hash(manifest)
                        environment = dict(request.spec.environment)
                        environment.update(CARGO_CAPTURE_MANIFEST=str(manifest), CARGO_CAPTURE_MANIFEST_SHA256=file_hash(manifest))
                        request = replace(request, step=5, output=output, manifest=manifest,
                            manifest_sha256=file_hash(manifest), chain=endpoints.WorkspaceChain(prepared, debug.endpoint),
                            inventory=replace(request.inventory, executables=tuple(replace(item, step=5, profile='release') for item in request.inventory.executables)),
                            spec=replace(request.spec, argv=runner.workspace_argv(str(request.spec.executable), 5),
                                output_root=output/'parent', external_inputs=tuple(sorted(external.items())), environment=tuple(sorted(environment.items()))))
                    def publish(path: Path, value: dict[str, Json], field: str = field, replacement: str = replacement) -> None:
                        if path.name == 'result.json': value = {**value, field: replacement}
                        atomic_json(path, value)
                    with patch.object(runner, 'atomic_json', publish), self.assertRaises(runner.WorkspaceSettledFailure):
                        _ = runner.run_workspace_owned(request)
                    self.assertTrue((request.output/'endpoint-failure.json').is_file())
                    self.assertEqual(document((request.output/'result.json').read_text()).field(field).value, replacement)
                    self.assertEqual(document((request.output/'endpoint-failure.json').read_text()).field('chronology').value, 'ENDPOINT_NOT_RETURNED')
