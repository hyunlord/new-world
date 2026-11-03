"""Actual tiny build/setup children; only real Cargo topology qualification is mocked."""
import shutil
import subprocess
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import gate_oracle_child as child
import generic_build as build
import generic_cli_prepare as cli
import generic_cli_products as products
import generic_cli_setup_handoff as handoff
import generic_setup_clippy as clippy
import generic_workspace_endpoints as endpoints
import generic_workspace_run as runner
import test_generic_workspace_endpoints as fixtures
from generic_gate import document
from generic_process import atomic_json, file_hash
from generic_setup_bridge import immutable_inputs, run_bridge_owned
from generic_setup_import import PriorBridge, start_import_pair
from generic_setup_prepare import PreparedSetup, prepare_warm
from test_generic_cli_products import COMPILER
from test_generic_setup_bridge import unused_wait


class HandoffTests(unittest.TestCase):
    def fixture(self, home: Path, *, changed_normal: bool = False, changed_bridge: bool = False) -> tuple[products.CliProducts, PreparedSetup, runner.WorkspaceRequest]:
        helper = fixtures.EndpointTests(); self.addCleanup(helper.doCleanups)
        request, old = helper.fixture(home)
        root = old.inputs.root
        for directory in ('scripts','scenes','shaders','assets','localization'):
            folder = root/directory; folder.mkdir(exist_ok=True); _ = (folder/'input.txt').write_text('fixture')
        for name in ('Cargo.toml', 'src/lib.rs', 'src/bin/worldsim_probe.rs', 'src/bin/worldsim_perf.rs'):
            path = root/'rust/crates/sim-test'/name
            path.parent.mkdir(parents=True, exist_ok=True); _ = path.write_text('synthetic normal '+name)
        for name, text in (('rust/Cargo.toml','[workspace]\n'), ('tools/play.sh','#!/bin/sh\n')):
            _ = (root/name).write_text(text)
        _ = shutil.copy2(Path(cli.__file__).with_name('gate_fixtures.json'), root/'tools/v8/gate_fixtures.json')
        config = root/'rust/.cargo/config.toml'
        _ = config.write_text('[build]\njobs=8\n[net]\noffline=true\n')
        cargo = request.spec.executable
        original = cargo.read_text().split('\n', 1)[1].replace('mkdir(parents=True)', 'mkdir(parents=True,exist_ok=True)')
        extra = "\nif len(sys.argv)>1 and sys.argv[1]=='clippy':\n pathlib.Path('target/clippy-new').write_text('producer')\n raise SystemExit(0)\n"
        if changed_normal:
            extra = extra.replace(" raise SystemExit(0)", " pathlib.Path('target/release/worldsim_probe').write_bytes(b'producer-changed')\n raise SystemExit(0)")
        if changed_bridge:
            extra = extra.replace(" raise SystemExit(0)", " pathlib.Path('target/debug/libsim_bridge.dylib').write_bytes(b'producer-changed')\n raise SystemExit(0)")
        body = '#!'+sys.executable+'\nimport sys,pathlib\nOMIT=False\n'
        body += "if '--version' in sys.argv or sys.argv[1]=='metadata' or '--no-run' in sys.argv:\n exec("+repr(COMPILER)+")\n raise SystemExit(0)\n"
        _ = cargo.write_text(body+extra+original)
        rustc = next(t.path for t in old.inputs.tools if t.name == 'rustc')
        _ = rustc.write_text('#!'+sys.executable+'\nprint("synthetic rustc\\nrelease: 1.93.0\\nhost: synthetic-test-only")\n')
        _ = subprocess.run(('git','-C',str(root),'add','.'),check=True,capture_output=True)
        _ = subprocess.run(('git','-C',str(root),'-c','user.name=Test','-c','user.email=t@example.invalid','commit','-qm','combined fixture'),check=True,capture_output=True)
        sha = subprocess.check_output(('git','-C',str(root),'rev-parse','HEAD'),text=True).strip()
        base, harness = home/'base', home/'harness'
        for other in (base,harness):
            _ = subprocess.run(('git','clone','-q',str(root),str(other)),check=True,capture_output=True)
            _ = shutil.copy2(root/'rust/Cargo.lock',other/'rust/Cargo.lock')
        _ = subprocess.run(('git','-C',str(harness),'update-ref','refs/remotes/origin/v8/main',sha),check=True,capture_output=True)
        base_home = home/'base-cache'; base_home.mkdir()
        def arm(path: Path, cache: Path) -> cli.ArmSelector:
            env = {'PATH':f'{rustc.parent}:/usr/bin:/bin:/usr/sbin:/sbin','LANG':'C','LC_ALL':'C',
                   'CARGO_HOME':str(cache),'CARGO_TARGET_DIR':str(path/'rust/target'),'RUSTC':str(rustc),
                   'RUSTC_WRAPPER':'','RUSTC_WORKSPACE_WRAPPER':''}
            cfg = path/'rust/.cargo/config.toml'
            return cli.ArmSelector(path,sha,cache,((str(cfg),file_hash(cfg)),),tuple(sorted(env.items())))
        selection = cli.CliSelectors(harness,sha,sha,'self_proof',arm(base,base_home),arm(root,old.inputs.cargo_home),
                                     cargo,file_hash(cargo),rustc,file_hash(rustc),home/'builds')
        builds = cli.build_cli(cli.prepare_cli(selection))
        self.assertEqual(builds.status,'BUILD_ANTECEDENTS_ONLY')
        views = products.construct_products(builds,home/'products')
        assert builds.candidate is not None
        tracked = subprocess.check_output(('git','-C',str(root),'ls-files'),text=True).splitlines()
        inputs = replace(old.inputs,source_sha=sha,
            tools=tuple(replace(t,sha256=file_hash(t.path)) for t in old.inputs.tools),
            effective_inputs=tuple(sorted((name,file_hash(root/name)) for name in (*tracked,'rust/Cargo.lock'))),
            configs=((str(config),file_hash(config)),))
        prepared = prepare_warm(inputs,builds.candidate)
        spec = replace(request.spec,executable_sha256=file_hash(cargo),inputs=prepared.materialized)
        external = dict(spec.external_inputs); external.update(dict(prepared.external))
        spec = replace(spec,external_inputs=tuple(sorted(external.items())))
        return views,prepared,replace(request,spec=spec)

    def finish(self, home: Path, prepared: PreparedSetup, request: runner.WorkspaceRequest) -> clippy.ClippyEndpoint:
        bridge = run_bridge_owned(prepared,unused_wait(prepared),request.held_lane_fd)
        owner = start_import_pair(prepared,PriorBridge(bridge,file_hash(bridge.receipt),unused_wait(prepared)),home/'imports',request.held_lane_fd)
        imported = owner.finish()
        request = replace(request,chain=endpoints.WorkspaceChain(prepared,endpoints.ImportPredecessor(owner,imported,file_hash(imported.receipt))))
        debug = runner.run_workspace_owned(request)
        assert debug.endpoint is not None
        output = home/'release'; manifest = home/'release-manifest.json'
        data = document(request.manifest.read_text()).mapping()
        data.update(step=5,profile='release',output_root=str(output/'capture'))
        atomic_json(manifest,data)
        external = dict(request.spec.external_inputs); _ = external.pop(request.manifest); external[manifest] = file_hash(manifest)
        env = dict(request.spec.environment); env.update(CARGO_CAPTURE_MANIFEST=str(manifest),CARGO_CAPTURE_MANIFEST_SHA256=file_hash(manifest))
        release_request = replace(request,step=5,output=output,manifest=manifest,manifest_sha256=file_hash(manifest),
            chain=endpoints.WorkspaceChain(prepared,debug.endpoint),
            inventory=replace(request.inventory,executables=tuple(replace(item,step=5,profile='release') for item in request.inventory.executables)),
            spec=replace(request.spec,argv=runner.workspace_argv(str(request.spec.executable),5),output_root=output/'parent',
                         external_inputs=tuple(sorted(external.items())),environment=tuple(sorted(env.items()))))
        release = runner.run_workspace_owned(release_request)
        assert release.endpoint is not None
        result = clippy.run_clippy_request(clippy.ClippyRequest(prepared,clippy.ClippyWaitEvidence(unused_wait(prepared),()),home/'clippy',request.held_lane_fd,release.endpoint))
        assert result.endpoint is not None
        return result.endpoint

    def test_actual_chain_changes_target_without_changing_normal_identity(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp).resolve(); views, prepared, request = self.fixture(home)
            begun = handoff.begin_setup_handoff(views,prepared)
            lock = home/'runtime/lane'
            with patch('generic_setup_bridge.LOCK',lock), patch.object(child,'LOCK',lock), patch.object(runner,'LOCK',lock), \
                 patch('generic_setup_bridge.qualify_wait',return_value=()), patch('generic_setup_import.qualify_wait',return_value=()), \
                 patch.object(clippy,'qualify_clippy',return_value=()):
                endpoint = self.finish(home,prepared,request)
                result = handoff.finish_setup_handoff(begun,endpoint)
                handoff.check_runtime_history(result)
                handoff.check_runtime_current(result)
                self.assertIn('rust/target/clippy-new',dict(result.runtime_inputs[1]))
                self.assertFalse(result.publication_authority)
                with self.assertRaises(ValueError):
                    _ = handoff.finish_setup_handoff(replace(begun,retained_anchor=None),endpoint)
                with self.assertRaises(ValueError):
                    _ = handoff.finish_setup_handoff(begun,replace(endpoint,predecessor=replace(endpoint.predecessor,step=4)))
                with self.assertRaises(ValueError):
                    handoff.check_runtime_current(replace(result,runtime_inputs=((),())))
                with self.assertRaises(ValueError):
                    _ = handoff.finish_setup_handoff(begun,replace(endpoint,request=replace(endpoint.request,prepared=replace(prepared))))
                target = views.normal[1].products[0].binary
                data = target.read_bytes(); mode = target.stat().st_mode
                for fault in ('bytes','mode'):
                    with self.subTest(fault=fault):
                        if fault == 'bytes': _ = target.write_bytes(b'changed normal')
                        else: target.chmod(0o644)
                        with self.assertRaises(ValueError): _ = handoff.finish_setup_handoff(begun,endpoint)
                        with self.assertRaises(ValueError): handoff.check_runtime_current(result)
                        _ = target.write_bytes(data); target.chmod(mode)
                for scope in (prepared.inputs.root/'rust/target',prepared.inputs.root/'.godot',prepared.inputs.settings):
                    with self.subTest(scope=scope):
                        existed = scope.exists()
                        scope.mkdir(exist_ok=True)
                        added = scope/'unexpected'; added.mkdir()
                        with self.assertRaises(ValueError): _ = handoff.finish_setup_handoff(begun,endpoint)
                        added.rmdir()
                        if not existed: scope.rmdir()
                source = views.normal[1].products[0].source
                source_bytes = source.read_bytes()
                original_pair_check = build.recheck_workspace_pair
                def mutate_after_base(pair: build.WorkspaceBuildPair) -> tuple[build.BuildSpec, build.BuildSpec]:
                    specs = original_pair_check(pair)
                    _ = source.write_bytes(b'late candidate source mutation')
                    return specs
                with patch.object(build,'recheck_workspace_pair',side_effect=mutate_after_base), self.assertRaises(ValueError):
                    handoff.check_runtime_current(result)
                _ = source.write_bytes(source_bytes)
                # Exact historical receipts remain protected after all generated transitions.
                history = endpoint.history[0].root/'receipt.json'
                original = history.read_bytes(); _ = history.write_bytes(b'{}')
                with self.assertRaises(ValueError): handoff.check_runtime_history(result)
                _ = history.write_bytes(original)
                handoff.check_runtime_current(result)
                with self.assertRaises(ValueError): products.recheck_products(views)
                (prepared.inputs.root/'rust/target/later-unowned').mkdir()
                handoff.check_runtime_history(result)
                with self.assertRaises(ValueError): handoff.check_runtime_current(result)

    def test_actual_successful_producer_cannot_reauthorize_changed_products(self) -> None:
        for kind in ('normal','bridge'):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp:
                home = Path(tmp).resolve()
                views, prepared, request = self.fixture(home,changed_normal=kind=='normal',changed_bridge=kind=='bridge')
                begun = handoff.begin_setup_handoff(views,prepared)
                lock = home/'runtime/lane'
                with patch('generic_setup_bridge.LOCK',lock), patch.object(child,'LOCK',lock), patch.object(runner,'LOCK',lock), \
                     patch('generic_setup_bridge.qualify_wait',return_value=()), patch('generic_setup_import.qualify_wait',return_value=()), \
                     patch.object(clippy,'qualify_clippy',return_value=()):
                    endpoint = self.finish(home,prepared,request)
                    self.assertEqual(endpoints.generated(prepared),endpoint.after)
                    clippy.check_clippy_endpoint(endpoint)
                    expected = 'selected original binary changed' if kind=='normal' else 'successor changed original bridge binary'
                    with self.assertRaisesRegex(ValueError,expected):
                        _ = handoff.finish_setup_handoff(begun,endpoint)

    def test_original_closure_rejects_final_callbacks_and_harness_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp).resolve(); views, prepared, request = self.fixture(home)
            closure = cli.setup_source_closure(views.builds)
            forged = replace(closure, closure=(), original=replace(closure.original, closure=()))
            with self.assertRaises(ValueError): forged.check()
            with patch.object(handoff, 'setup_source_closure', return_value=forged), self.assertRaises(ValueError):
                _ = handoff.begin_setup_handoff(views, prepared)
            with self.assertRaises(ValueError): closure.check(replace(views.builds))
            replaced_issuer = replace(views.builds, retained_anchor=forged.original)
            with self.assertRaises(ValueError): replace(forged, builds=replaced_issuer).check()
            with patch.object(handoff, 'setup_source_closure', return_value=replace(closure, builds=replace(views.builds))), self.assertRaises(ValueError):
                _ = handoff.begin_setup_handoff(views, prepared)
            candidate = views.builds.prepared.selectors.candidate.root
            for directory in (candidate/'rust/target/allowed-empty', candidate/'.godot'):
                directory.mkdir(); closure.check(); directory.rmdir()
            for directory in (candidate/'rust/target-other', candidate/'.godot-other',
                              views.builds.prepared.selectors.base.root/'unexpected',
                              views.builds.prepared.selectors.output/'unexpected'):
                directory.mkdir()
                with self.assertRaises(ValueError): closure.check()
                directory.rmdir()
            for member in (views.builds.prepared.selectors.cargo, candidate/'rust/.cargo/config.toml'):
                self.assertTrue(member.is_relative_to(home) and member.resolve() == member)
                self.assertTrue(all(not parent.is_symlink() for parent in member.parents if parent.is_relative_to(home)))
                mode = member.stat().st_mode
                member.chmod(0o600)
                with self.assertRaises(ValueError): closure.check()
                member.chmod(mode)
            with self.assertRaises(ValueError): _ = cli.setup_source_closure(replace(views.builds,retained_anchor=None))
            with self.assertRaises(ValueError): replace(closure,candidate=views.builds.prepared.selectors.base.root).check()
            original_anchor = views.builds.retained_anchor
            assert original_anchor is not None
            with self.assertRaises(ValueError):
                _ = cli.setup_source_closure(replace(views.builds,retained_anchor=replace(original_anchor,closure=original_anchor.closure[:-1])))
            genuine_cli = cli.recheck_cli_builds
            harness = views.builds.prepared.selectors.harness_root
            late = harness/'late-issuer-empty'
            def mutate_issuer(value: cli.CliBuilds) -> None:
                genuine_cli(value)
                late.mkdir()
            with patch.object(cli,'recheck_cli_builds',side_effect=mutate_issuer), self.assertRaises(ValueError):
                _ = cli.setup_source_closure(views.builds)
            late.rmdir()
            source = views.normal[1].products[0].source
            raw = source.read_bytes()
            genuine = immutable_inputs
            def change_source(value: PreparedSetup) -> None:
                genuine(value)
                _ = source.write_bytes(b'callback drift')
            with patch.object(handoff,'immutable_inputs',side_effect=change_source), self.assertRaises(ValueError):
                _ = handoff.begin_setup_handoff(views,prepared)
            _ = source.write_bytes(raw)
            begun = handoff.begin_setup_handoff(views,prepared)
            lock = home/'runtime/lane'
            with patch('generic_setup_bridge.LOCK',lock), patch.object(child,'LOCK',lock), patch.object(runner,'LOCK',lock), \
                 patch('generic_setup_bridge.qualify_wait',return_value=()), patch('generic_setup_import.qualify_wait',return_value=()), \
                 patch.object(clippy,'qualify_clippy',return_value=()):
                endpoint = self.finish(home,prepared,request)
                runtime = handoff.finish_setup_handoff(begun,endpoint)
                calls = 0
                def change_last(value: PreparedSetup) -> None:
                    nonlocal calls
                    genuine(value); calls += 1
                    if calls == 3: _ = source.write_bytes(b'callback drift')
                with patch.object(handoff,'immutable_inputs',side_effect=change_last), self.assertRaises(ValueError):
                    handoff.check_runtime_current(runtime)
                _ = source.write_bytes(raw)
                harness = views.builds.prepared.selectors.harness_root
                member = harness/'tools/v8/gate_fixtures.json'; mode = member.stat().st_mode
                member.chmod(0o666)
                with self.assertRaises(ValueError): _ = handoff.finish_setup_handoff(begun,endpoint)
                with self.assertRaises(ValueError): handoff.check_runtime_current(runtime)
                member.chmod(mode)
                empty = harness/'unowned-empty'; empty.mkdir()
                with self.assertRaises(ValueError): _ = handoff.finish_setup_handoff(begun,endpoint)
                with self.assertRaises(ValueError): handoff.check_runtime_current(runtime)
                empty.rmdir()
                handoff.check_runtime_current(runtime)
