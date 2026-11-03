"""Actual tiny shell/import/workspace chain; no Cargo/Godot/native workloads."""
import fcntl
import os
import tempfile
import unittest
from collections.abc import Callable
from dataclasses import replace
from pathlib import Path
from typing import Literal, NoReturn
from unittest.mock import patch

import gate_oracle_child as child
import generic_full_gate as gate
import generic_setup_bridge as bridge
import generic_setup_clippy as clippy
import generic_setup_import as imports
import generic_setup_prepare as preparation
import generic_static_run as static
import generic_workspace_endpoints as endpoints
import generic_workspace_run as workspace
import test_generic_workspace_endpoints as fixtures
from generic_fixture_run import ComponentResult
from generic_gate import GateContractError, document, require
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json
from test_generic_full_gate import owned_inputs
from test_generic_setup_bridge import unused_wait
from test_generic_setup_import import make_fixture
from test_generic_setup_prepare import Fixture


class SetupCoordinatorTests(unittest.TestCase):
    def test_combined_setup_and_defence_preserve_original_order(self) -> None:
        calls = gate.ordered_owned_calls(setup_enabled=True, mux2_enabled=True, oracles_enabled=True,
                                        m1_enabled=True, library42_enabled=True, library43_enabled=True)
        steps = [call.step for call in calls]
        first = steps.index(1)
        self.assertEqual(steps[first:first+6], [1, 2, 3, 4, 5, 6])
        defence = steps.index(37)
        self.assertEqual(steps[defence-2:defence+2], [42, 43, 37, 38])
        self.assertEqual(steps.count(37), 1)
        self.assertEqual(steps.count(17), 1)
        identities = [identity for call in calls for identity in call.identities]
        self.assertEqual(len(identities), len(set(identities)))

    def fixture(self, home: Path, control: int, *, failed: bool = False, first_failure: bool = False) -> gate.OwnedOrderedInputs:
        helper=fixtures.EndpointTests()
        self.addCleanup(helper.doCleanups)
        def owned_file(path: Path) -> None:
            require(path.is_relative_to(home) and path.resolve().is_relative_to(home)
                    and not path.is_symlink(), '/fixture', 'refuse nonlocal/symlink mutation')
        original_unlink=Path.unlink
        def unlink(path: Path, missing_ok: bool=False) -> None:
            owned_file(path);original_unlink(path,missing_ok=missing_ok)
        def prepared(inputs: preparation.SetupInputs) -> preparation.PreparedSetup:
            cargo=next(t.path for t in inputs.tools if t.name=='cargo')
            owned_file(cargo)
            text=cargo.read_text(); lines=text.splitlines(True)
            _=cargo.write_text(lines[0]+"import sys,pathlib\nif sys.argv[1]=='clippy':\n pathlib.Path('target/clippy-marker').write_text('tiny')\n raise SystemExit(0)\n"+''.join(lines[1:]))
            tools=tuple(replace(t,path=Path('/bin/bash'),sha256=file_hash(Path('/bin/bash'))) if t.name=='bash'
                        else replace(t,sha256=file_hash(t.path)) for t in inputs.tools)
            env=tuple((k,v+':/bin' if k=='PATH' else v) for k,v in inputs.environment)
            return preparation.prepare(replace(inputs,tools=tools,environment=env))
        original_fixture=make_fixture
        def imports_fixture(directory: Path, first: int, second: int) -> Fixture:
            return original_fixture(directory,7 if first_failure else first,second)
        with patch.object(fixtures,'prepare',prepared),patch.object(Path,'unlink',unlink),patch.object(fixtures,'make_fixture',imports_fixture):
            template,bound=helper.fixture(home,fail=failed)
        helper.doCleanups()  # release the fixture's own lane; coordinator acquires its own actual FD
        context=owned_inputs(home,control)
        product=replace(context.components.probes.candidate,input_root=bound.inputs.root,
                        source_sha=bound.inputs.source_sha,input_files=bound.materialized)
        def factory(step: int, chain: endpoints.WorkspaceChain, output: Path, prep: Path, descriptor: int) -> endpoints.WorkspaceRequest:
            data=document(template.manifest.read_text()).mapping()
            data.update(step=step,profile='debug' if step==4 else 'release',output_root=str(output/'capture'))
            manifest=prep/'manifest.json';atomic_json(manifest,data)
            external=dict(template.spec.external_inputs);_=external.pop(template.manifest)
            external[manifest]=file_hash(manifest)
            env=dict(template.spec.environment);env.update(CARGO_CAPTURE_MANIFEST=str(manifest),CARGO_CAPTURE_MANIFEST_SHA256=file_hash(manifest))
            spec=replace(template.spec,argv=workspace.workspace_argv(str(template.spec.executable),step),
                         output_root=output/'parent',external_inputs=tuple(sorted(external.items())),environment=tuple(sorted(env.items())))
            inventory=replace(template.inventory,executables=tuple(replace(item,step=4 if step==4 else 5,
                                profile='debug' if step==4 else 'release') for item in template.inventory.executables))
            return replace(template,step=4 if step==4 else 5,chain=chain,output=output,held_lane_fd=descriptor,
                           manifest=manifest,manifest_sha256=file_hash(manifest),inventory=inventory,spec=spec)
        setup=gate.SetupContext(bound,unused_wait(bound),clippy.ClippyWaitEvidence(unused_wait(bound),()),factory)
        godot=next(t for t in bound.inputs.tools if t.name=='godot')
        godot_product=replace(product,executable=godot.path,executable_sha256=godot.sha256)
        return replace(context,components=replace(context.components,probes=replace(context.components.probes,candidate=product),
            godot=replace(context.components.godot,candidate=godot_product),
            declarations=replace(context.components.declarations,candidate=product.source_sha)),setup=setup)

    def test_route_preserves_step1_and_existing_routes(self) -> None:
        old=gate.ordered_owned_calls(m1_enabled=True,play_enabled=True,prefix28_enabled=True,prefix36_enabled=True)
        new=gate.ordered_owned_calls(m1_enabled=True,play_enabled=True,prefix28_enabled=True,prefix36_enabled=True,setup_enabled=True)
        self.assertEqual([r for r in new if r.component!='setup'],list(old))
        self.assertEqual(tuple(r for r in new if r.component=='setup'),gate.setup_calls())
        self.assertEqual([r.step for r in new[:6]],[1,2,3,4,5,6])

    def test_actual_whole_chain_and_failed_native_endpoint(self) -> None:
        for failed,first_failure in ((False,False),(True,False),(False,True)):
            with self.subTest(failed=failed,first_failure=first_failure),tempfile.TemporaryDirectory() as temporary:
                home=Path(temporary).resolve();control,write=os.pipe()
                try:
                    context=self.fixture(home,control,failed=failed,first_failure=first_failure);lock=home/'runtime/lane'
                    assert context.setup is not None
                    owner: gate.SetupOwner | None=None
                    with patch.object(gate,'LOCK',lock),patch.object(bridge,'LOCK',lock),patch.object(child,'LOCK',lock),patch.object(workspace,'LOCK',lock), \
                         patch.object(bridge,'qualify_wait',return_value=()),patch.object(imports,'qualify_wait',return_value=()),patch.object(clippy,'qualify_clippy',return_value=()):
                        (home/'out/step-02').mkdir(parents=True)
                        owner=gate.SetupOwner(control,home/'out/step-02/setup',context.setup)
                        rows: list[gate.Outcome]=[]
                        try:
                            for call in gate.setup_calls():
                                output=home/'out'/f'step-{call.step:02d}'/'setup';output.parent.mkdir(exist_ok=True)
                                rows.append(gate.invoke_setup(call,context,output,owner))
                            self.assertTrue(owner.closed)
                            assert owner.imported is not None
                            self.assertEqual(owner.imported.first_exit,7 if first_failure else 0)
                            self.assertEqual(owner.imported.second_exit,0)
                            self.assertEqual([r.status for r in rows],['PENDING','PENDING','FAIL' if failed else 'PENDING','FAIL' if failed else 'PENDING','PENDING'])
                            self.assertIsNotNone(rows[-1].setup_evidence)
                            for row in rows:gate.recheck_setup(context,row)
                            (context.setup.prepared.inputs.output/'late-empty').mkdir()
                            with self.assertRaises(GateContractError):gate.recheck_setup(context,rows[0])
                        finally:
                            if not owner.in_flight:owner.close()
                finally:os.close(control);os.close(write)

    def test_dispatch_order_factory_failure_and_final_publication(self) -> None:
        for fault in ('none','factory','factory-drift','publication'):
            with self.subTest(fault=fault):self.dispatch_case(fault)

    def test_generated_target_publication(self) -> None:
        self.dispatch_case('target-publication')

    def test_generated_cache_settings_and_final_callbacks(self) -> None:
        for fault in ('cache-publication','settings-publication','target-final','cache-final','settings-final'):
            with self.subTest(fault=fault):self.dispatch_case(fault)

    def dispatch_case(self, fault: str) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            home=Path(temporary).resolve();control,write=os.pipe()
            try:
                context=self.fixture(home,control);lock=home/'runtime/lane'
                assert context.setup is not None
                original=context.setup.workspace_factory
                def factory(step: int, chain: endpoints.WorkspaceChain, output: Path, prep: Path, descriptor: int,
                            fault: str=fault, original: Callable[[Literal[4,5],endpoints.WorkspaceChain,Path,Path,int],endpoints.WorkspaceRequest]=original) -> endpoints.WorkspaceRequest:
                    if fault=='factory':
                        (chain.prepared.inputs.root/'rust/target/factory-empty').mkdir()
                        raise ValueError('construction failure, no producer entered')
                    request=original(4 if step==4 else 5,chain,output,prep,descriptor)
                    if fault=='factory-drift':(chain.prepared.inputs.root/'rust/target/factory-empty').mkdir()
                    return request
                context=replace(context,setup=replace(context.setup,workspace_factory=factory))
                assert context.setup is not None
                prepared=context.setup.prepared
                changes={'target':prepared.inputs.root/'rust/target',
                         'cache':prepared.inputs.root/'.godot','settings':prepared.inputs.settings}
                later_finished=False
                def mutate(name: str, marker: str) -> None:
                    directory=changes[name];directory.mkdir(exist_ok=True)
                    (directory/marker).mkdir()
                def final_bindings(_context: gate.OrderedInputs) -> None:
                    if later_finished and fault.endswith('-final'):
                        mutate(fault.removesuffix('-final'),'late-empty')
                def static_return(_name: str, _action: gate.Action, _output: Path, _inputs: gate.Inputs, *,
                                  step: int | None=None, static_product: static.StaticProduct | None=None,
                                  static_environment: static.ExecutionEnvironment | None=None,
                                  save_products: gate.SaveRoundtripProducts | None=None,
                                  perf_context: gate.PerfContext | None=None) -> gate.Outcome:
                    nonlocal later_finished
                    self.assertIsNotNone(static_product)
                    self.assertIsNotNone(static_environment)
                    self.assertIs(save_products,context.save_roundtrip)
                    self.assertIs(perf_context,context.perf)
                    if step==7:
                        for name in changes:mutate(name,'authorized-later-producer')
                        later_finished=True
                    return gate.Outcome('static','PENDING',None,None,'')
                def published(path: Path, value: dict[str,Json], fault: str=fault, prepared: preparation.PreparedSetup=prepared) -> None:
                    atomic_json(path,value)
                    if fault=='publication' and path.name=='ordered-dispatch.json':
                        (prepared.inputs.output/'late-empty').mkdir()
                    if fault.endswith('-publication') and path.name=='ordered-dispatch.json':
                        mutate(fault.removesuffix('-publication'),'late-empty')
                def unused_action(_output: Path) -> ComponentResult:
                    raise AssertionError('mocked static producer')
                calls=(gate.OrderedCall(1,'static',('1:fmt',)),*gate.setup_calls(),gate.OrderedCall(7,'static',('7:forbidden_math',)))
                with patch.object(gate,'LOCK',lock),patch.object(bridge,'LOCK',lock),patch.object(child,'LOCK',lock),patch.object(workspace,'LOCK',lock), \
                     patch.object(bridge,'qualify_wait',return_value=()),patch.object(imports,'qualify_wait',return_value=()),patch.object(clippy,'qualify_clippy',return_value=()), \
                     patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings',side_effect=final_bindings),patch.object(gate,'perf_bindings'), \
                     patch.object(gate,'ordered_action',return_value=unused_action),patch.object(gate,'invoke',side_effect=static_return),patch.object(gate,'atomic_json',side_effect=published):
                    returned=gate.run_ordered_owned(context,context.strict,home/'out',control,setup=context.setup)
                row=document(returned.read_text())
                expected=['1:static','2:setup','3:setup','4:setup']+([] if fault.startswith('factory') else ['5:setup','6:setup'])+['7:static']
                self.assertEqual(row.field('actual_component_sequence').value,expected,row.field('errors').value)
                self.assertEqual(row.field('status').text(),'INCOMPLETE' if fault=='none' else 'FAIL')
                if fault.endswith(('-publication','-final')):
                    self.assertIn('current generated state changed after producers settled',str(row.field('errors').value))
                    frozen=str(row.field('setup_terminal_generated').value)
                    self.assertIn('authorized-later-producer',frozen)
                    self.assertNotIn('late-empty',frozen)
                if fault=='publication' or fault.endswith('-publication'):
                    self.assertNotEqual(returned.name,'ordered-dispatch.json')
                    self.assertEqual(row.field('supersedes').field('sha256').text(),file_hash(home/'out/ordered-dispatch.json'))
                if fault.startswith('factory'):
                    components=row.field('components').array()
                    self.assertEqual([components[i].field('status').text() for i in (4,5)],['NOT_RUN','NOT_RUN'])
            finally:os.close(control);os.close(write)

    def test_unreturned_producer_keeps_actual_lane_and_original_error(self) -> None:
        class Held(BaseException):
            pass
        with tempfile.TemporaryDirectory() as temporary:
            home=Path(temporary).resolve();control,write=os.pipe()
            try:
                context=self.fixture(home,control);lock=home/'runtime/lane'
                assert context.setup is not None
                (home/'out/step-02').mkdir(parents=True)
                primary=KeyboardInterrupt('before synthetic dispatch')
                unknown=bridge.BridgeOwnershipUnknown(primary,None,None)
                retained: list[BaseException]=[]
                def held(error: BaseException) -> NoReturn:
                    retained.append(error)
                    raise Held()
                with patch.object(gate,'LOCK',lock),patch.object(bridge,'run_bridge_owned',side_effect=unknown),patch('generic_full_gate._StrictOwner.hold',side_effect=held):
                    owner=gate.SetupOwner(control,home/'out/step-02/setup',context.setup)
                    try:
                        with self.assertRaises(Held): _=gate.invoke_setup(gate.setup_calls()[0],context,home/'out/step-02/setup',owner)
                        self.assertTrue(owner.in_flight);self.assertFalse(owner.closed)
                        self.assertEqual(retained,[unknown]);self.assertIs(unknown.primary,primary)
                        self.assertIs(owner.retained_failure,unknown)
                        with lock.open('a') as rival,self.assertRaises(BlockingIOError):
                            fcntl.flock(rival.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
                        self.assertGreater(os.fstat(owner.descriptor).st_ino,0)
                        with self.assertRaises(GateContractError):owner.close()
                    finally:
                        # Fixture teardown only: mocked entry never spawned a child.
                        os.close(owner.descriptor)
            finally:os.close(control);os.close(write)
