"""Closed setup-prefix selector boundary; no native tools are launched."""
import unittest
from pathlib import Path

import generic_cli_execute as cli
from generic_gate import GateContractError, document, integer
from mux2_gate_inputs import Node


class SelectorTests(unittest.TestCase):
    def test_schema_is_closed_before_any_execution(self) -> None:
        with self.assertRaises(GateContractError):
            _=cli.parse_spec(document('{"schema":"cli_setup_prefix.v1","caller_pass":true}'))

    def test_relative_paths_and_malformed_hashes_reject(self) -> None:
        with self.assertRaises(GateContractError):_=cli.path(Node('relative'))
        with self.assertRaises(GateContractError):_=cli.digest(Node('not-a-hash'))
        self.assertEqual(cli.path(Node('/tmp/spec')),Path('/tmp/spec'))
        self.assertEqual(cli.digest(Node('a'*64)),'a'*64)

    def test_duplicate_environment_entries_reject(self) -> None:
        with self.assertRaises(GateContractError):
            _=cli.pairs(Node([['PATH','/bin'],['PATH','/usr/bin']]))




class SetupContextTests(unittest.TestCase):
    def test_real_setup_chain_accepts_no_future_product_placeholders(self) -> None:
        import os
        import tempfile
        from unittest.mock import patch

        import gate_oracle_child as child
        import generic_full_gate as gate
        import generic_setup_bridge as bridge
        import generic_setup_clippy as clippy
        import generic_setup_import as imports
        import generic_workspace_run as workspace
        from test_generic_full_gate_setup import SetupCoordinatorTests

        fixture=SetupCoordinatorTests()
        self.addCleanup(fixture.doCleanups)
        with tempfile.TemporaryDirectory() as temporary:
            home=Path(temporary).resolve();control,write=os.pipe()
            try:
                old=fixture.fixture(home,control)
                assert old.setup is not None
                prepared=old.setup.prepared
                godot=next(t for t in prepared.inputs.tools if t.name=='godot')
                context=gate.SetupPrefixContext(old.setup,prepared.inputs.root,prepared.inputs.source_sha,
                                               prepared.materialized,godot.path,godot.sha256)
                lock=home/'runtime/lane'
                with patch.object(gate,'LOCK',lock),patch.object(bridge,'LOCK',lock),patch.object(child,'LOCK',lock),patch.object(workspace,'LOCK',lock), \
                     patch.object(bridge,'qualify_wait',return_value=()),patch.object(imports,'qualify_wait',return_value=()),patch.object(clippy,'qualify_clippy',return_value=()):
                    (home/'out/step-02').mkdir(parents=True)
                    owner=gate.SetupOwner(control,home/'out/step-02/setup',old.setup)
                    try:
                        rows: list[gate.Outcome]=[]
                        for call in gate.setup_calls():
                            output=home/'out'/f'step-{call.step:02d}'/'setup';output.parent.mkdir(exist_ok=True)
                            rows.append(gate.invoke_setup(call,context,output,owner))
                        self.assertTrue(owner.closed)
                        self.assertEqual([row.status for row in rows],['PENDING']*5)
                        for row in rows:gate.recheck_setup(context,row)
                    finally:
                        if not owner.in_flight:owner.close()
            finally:os.close(control);os.close(write)


class PublicationTests(unittest.TestCase):
    def test_publication_rejects_extra_file_and_empty_directory(self) -> None:
        import tempfile
        from unittest.mock import patch

        from generic_process import atomic_json
        from mux2_gate_inputs import Json

        for kind in ('file','directory'):
            with self.subTest(kind=kind),tempfile.TemporaryDirectory() as temporary:
                root=Path(temporary).resolve()
                def changed(path: Path, value: dict[str,Json], kind: str=kind, root: Path=root) -> None:
                    atomic_json(path,value)
                    if kind=='file':_= (root/'late').write_text('extra')
                    else:(root/'late-empty').mkdir()
                with patch.object(cli,'atomic_json',changed),self.assertRaises(GateContractError):
                    _=cli.publish(root/'stage.json',{'status':'INCOMPLETE'})




class ClassifierTests(unittest.TestCase):
    def test_exact_profiles_and_foreign_or_stale_records(self) -> None:
        import json
        import tempfile

        from generic_process import file_hash
        from mux2_gate_inputs import Json

        for profile in ('debug','release'):
            with self.subTest(profile=profile),tempfile.TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();product=root/'product';cargo=root/'cargo'
                deps=product/'rust/target'/profile/'deps';deps.mkdir(parents=True)
                rows: list[dict[str,Json]]=[];members: dict[str,str]={}
                for name in (*cli.REGISTRY,'sim_bridge',*cli.BINS):
                    executable: str | None=None
                    if name in cli.REGISTRY:
                        package,version=cli.REGISTRY[name]
                        package_id=f'registry+https://github.com/rust-lang/crates.io-index#{package}@{version}'
                        source=cargo/'registry/src/index.crates.io-1949cf8c6b5b557f'/f'{package}-{version}'/'src/lib.rs'
                        kind=['proc-macro'];output=deps/f'lib{name}-1234567812345678.dylib';member=output
                    elif name=='sim_bridge':
                        package_id=f'path+file://{product}/rust/crates/sim-bridge#0.7.0'
                        source=product/'rust/crates/sim-bridge/src/lib.rs';kind=['cdylib','rlib']
                        output=deps/'libsim_bridge.dylib';member=output
                    else:
                        package_id=f'path+file://{product}/rust/crates/sim-test#0.7.0'
                        source=product/'rust/crates/sim-test/src/bin'/f'{name}.rs';kind=['bin']
                        output=deps.parent/name;member=deps/f'{name}-1234567812345678';executable=str(output)
                    source.parent.mkdir(parents=True,exist_ok=True);_=source.write_text('// fixture')
                    for path in (output,member):_=path.write_text(name);path.chmod(0o755)
                    members[member.name]=file_hash(member)
                    rows.append({'reason':'compiler-artifact','package_id':package_id,'fresh':False,
                        'target':{'name':name,'kind':list(kind),'crate_types':list(kind),'src_path':str(source)},
                        'profile':{'test':False},'filenames':[str(output)],'executable':executable})
                stream=root/'stream.jsonl'
                def write(stream: Path=stream, rows: list[dict[str,Json]]=rows) -> None:
                    _=stream.write_text(''.join(json.dumps(row)+'\n' for row in rows))
                write()
                selected='debug' if profile=='debug' else 'release'
                result=cli.classify_compiled(product,cargo,selected,stream,{},members)
                self.assertEqual(len(result.table),9)
                wrong='release' if selected=='debug' else 'debug'
                with self.assertRaises(ValueError):_=cli.classify_compiled(product,cargo,wrong,stream,{},members)
                rows[0]['fresh']=True;write()
                with self.assertRaises(ValueError):_=cli.classify_compiled(product,cargo,selected,stream,{},members)
                rows[0]['fresh']=False;rows[0]['package_id']='foreign';write()
                with self.assertRaises(ValueError):_=cli.classify_compiled(product,cargo,selected,stream,{},members)




class InitializerOwnershipTests(unittest.TestCase):
    def test_unknown_initializer_keeps_real_lane_until_test_owned_child_is_reaped(self) -> None:
        import fcntl
        import os
        import subprocess
        import sys
        import tempfile
        from unittest.mock import patch

        import generic_full_gate as gate
        from generic_cli_prepare import ArmSelector, CliSelectors
        from generic_process import file_hash
        from generic_setup_prepare import Tool
        from generic_static_run import ExecutionEnvironment

        class HeldForTest(RuntimeError):
            pass

        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe();lane=root/'lane'
            source=root/'selectors.json';_=source.write_text('{}')
            arm=ArmSelector(root/'candidate','a'*40,root/'cache',(),())
            selectors=CliSelectors(root/'harness','a'*40,'a'*40,'self_proof',arm,arm,
                root/'cargo','b'*64,root/'rustc','c'*64,root/'builds')
            env=ExecutionEnvironment((),(),())
            setup=cli.SetupSelectors((),(),root/'lock','d'*64,(),())
            spec=cli.PrefixSpec(root/'output',read,selectors,setup,env,env,Tool('rustdoc',root/'rustdoc','e'*64))
            document=cli.PrefixDocument(source,file_hash(source),spec)
            children: list[subprocess.Popen[bytes]]=[];descriptors: list[int]=[]
            original_flock=fcntl.flock
            def lock(descriptor: int, operation: int) -> None:
                original_flock(descriptor,operation);descriptors.append(descriptor)
            def fail(selected: cli.PrefixSpec, owner: cli.InitializationOwner) -> cli.PrefixPrepared:
                selected.output.mkdir();owner.in_flight=True
                children.append(subprocess.Popen((sys.executable,'-c','import time;time.sleep(30)')))
                raise RuntimeError('injected postlaunch failure')
            try:
                with patch.object(gate,'LOCK',lane),patch.object(cli,'read_spec',return_value=document), \
                     patch.object(cli,'initialize',side_effect=fail),patch.object(fcntl,'flock',side_effect=lock), \
                     patch('generic_full_gate._StrictOwner.hold',side_effect=HeldForTest),self.assertRaises(HeldForTest):
                    _=gate.execute_setup_prefix(source)
                self.assertEqual(len(children),1);self.assertIsNone(children[0].poll())
                self.assertEqual(len(descriptors),1);_=os.fstat(descriptors[0])
                self.assertFalse((spec.output/'setup-prefix-failure.json').exists())
                check=os.open(lane,os.O_RDWR)
                try:
                    with self.assertRaises(BlockingIOError):original_flock(check,fcntl.LOCK_EX|fcntl.LOCK_NB)
                finally:os.close(check)
            finally:
                for child in children:
                    child.terminate();_=child.wait(timeout=5)
                for descriptor in descriptors:os.close(descriptor)
                os.close(read);os.close(write)




class InitializerFailureTests(unittest.TestCase):
    def test_settled_failure_records_all_obligations_without_launch_claim(self) -> None:
        import os
        import tempfile
        from unittest.mock import patch

        import generic_full_gate as gate
        from generic_cli_prepare import ArmSelector, CliSelectors
        from generic_process import file_hash
        from generic_setup_prepare import Tool
        from generic_static_run import ExecutionEnvironment

        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe();lane=root/'lane'
            source=root/'selectors.json';_=source.write_text('{}')
            arm=ArmSelector(root/'candidate','a'*40,root/'cache',(),())
            selectors=CliSelectors(root/'harness','a'*40,'a'*40,'self_proof',arm,arm,
                root/'cargo','b'*64,root/'rustc','c'*64,root/'builds')
            env=ExecutionEnvironment((),(),())
            setup=cli.SetupSelectors((),(),root/'lock','d'*64,(),())
            spec=cli.PrefixSpec(root/'output',read,selectors,setup,env,env,Tool('rustdoc',root/'rustdoc','e'*64))
            selected=cli.PrefixDocument(source,file_hash(source),spec)
            def fail(spec: cli.PrefixSpec, _owner: cli.InitializationOwner) -> cli.PrefixPrepared:
                spec.output.mkdir()
                raise ValueError('injected prelaunch validation')
            try:
                with patch.object(gate,'LOCK',lane),patch.object(cli,'read_spec',return_value=selected), \
                     patch.object(cli,'initialize',side_effect=fail):
                    receipt=gate.execute_setup_prefix(source)
                actual=document(receipt.read_text())
                self.assertEqual(actual.field('status').value,'FAIL')
                self.assertEqual(actual.field('actual_steps').value,[])
                self.assertEqual(actual.field('publication_authorized').value,False)
                self.assertEqual({integer(row.field('step')) for row in actual.field('obligations').array()},set(range(1,52)))
                self.assertTrue(all(row.field('status').value=='NOT_RUN' for row in actual.field('obligations').array()))
            finally:os.close(read);os.close(write)




class RecipeTests(unittest.TestCase):
    def test_recipe_emits_closed_profiles_without_running_binaries(self) -> None:
        import os
        import struct
        import tempfile
        from contextlib import chdir
        from unittest.mock import patch

        from fixtures import fake_cargo_recipe as recipe
        from generic_process import file_hash

        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();workspace=root/'product/rust';workspace.mkdir(parents=True)
            for package in recipe.LIBRARIES:
                source=workspace/'crates'/package/'src/lib.rs';source.parent.mkdir(parents=True);_=source.write_text('// fixture')
                if package=='sim-test':
                    for name in recipe.BINS:
                        binary_source=source.parent/'bin'/f'{name}.rs';binary_source.parent.mkdir(exist_ok=True);_=binary_source.write_text('// fixture')
            environment={'CARGO_HOME':str(root/'cache'),'CARGO_TARGET_DIR':str(workspace/'target')}
            for profile in ('debug','release'):
                with chdir(workspace),patch.dict(os.environ,environment):
                    argv=('/fixture/cargo','test','--workspace','--no-run',*(() if profile=='debug' else ('--release',)))
                    raw=recipe.main(argv)
                cursor=0;parts: list[bytes]=[]
                while raw[cursor:cursor+1]!=b'Q':
                    self.assertEqual(raw[cursor:cursor+1],b'O')
                    size=int.from_bytes(raw[cursor+1:cursor+5],'big');cursor+=5
                    parts.append(raw[cursor:cursor+size]);cursor+=size
                self.assertEqual(raw[cursor:],b'Q'+struct.pack('!I',0))
                stream=root/(profile+'.jsonl');_=stream.write_bytes(b''.join(parts))
                deps=workspace/'target'/profile/'deps'
                members={path.name:file_hash(path) for path in deps.iterdir()}
                native={str(path):('native-'+path.name,file_hash(path)) for path in deps.iterdir() if path.name.endswith('-testfixture')}
                result=cli.classify_compiled(workspace.parent,root/'cache','debug' if profile=='debug' else 'release',stream,native,members)
                self.assertEqual(len(result.table),9);self.assertEqual(len(native),5)


if __name__=='__main__':_=unittest.main()


class FixtureTransformationTests(unittest.TestCase):
    def test_exact_source_only_transform_and_changed_preimage_rejection(self) -> None:
        import hashlib
        import shutil
        import tempfile

        import generic_setup_bridge
        import generic_setup_clippy
        import mux2_gate_lock
        from fixtures import prepare_harness

        sources: dict[str,Path]={}
        for module in (generic_setup_bridge,generic_setup_clippy,mux2_gate_lock):
            assert module.__file__ is not None
            source=Path(module.__file__).resolve()
            sources[source.name]=source
        for mutate in (False,True):
            with self.subTest(mutate=mutate),tempfile.TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();tools=root/'tools/v8';tools.mkdir(parents=True)
                for name in prepare_harness.PREIMAGES:
                    _=shutil.copy2(sources[name],tools/name)
                binaries: dict[str,Path]={}
                for name in prepare_harness.TOOL_NAMES:
                    path=root/name;_=path.write_bytes(('inert-'+name).encode());binaries[name]=path
                if mutate:
                    with (tools/'mux2_gate_lock.py').open('a') as stream:_=stream.write('\n# changed\n')
                before={name:(tools/name).read_bytes() for name in prepare_harness.PREIMAGES}
                if mutate:
                    with self.assertRaises(ValueError):
                        _=prepare_harness.transform(root,binaries,root/'fixture-lane')
                    self.assertEqual(before,{name:(tools/name).read_bytes() for name in before})
                else:
                    receipt=prepare_harness.transform(root,binaries,root/'fixture-lane')
                    self.assertEqual(receipt['qualification'],'TRANSFORMED_SOURCE_FIXTURE_ONLY')
                    self.assertEqual(len(receipt['files']),3)
                    for name,row in receipt['files'].items():
                        self.assertEqual(row['before'],hashlib.sha256(before[name]).hexdigest())
                        self.assertEqual(row['after'],hashlib.sha256((tools/name).read_bytes()).hexdigest())
                    with self.assertRaises(ValueError):
                        _=prepare_harness.transform(root,binaries,root/'fixture-lane')


class SetupInputProjectionTests(unittest.TestCase):
    def test_retains_all_effective_tracked_sources_and_ignored_build_inputs(self) -> None:
        manifest=Node({'.gitignore':['100644','a'*64],'rust/Cargo.toml':['100644','b'*64]})
        self.assertEqual(cli.setup_input_files(manifest,(('rust/Cargo.toml','b'*64),('rust/Cargo.lock','c'*64))),
            (('.gitignore','a'*64),('rust/Cargo.lock','c'*64),('rust/Cargo.toml','b'*64)))
        with self.assertRaises(GateContractError):
            _=cli.setup_input_files(manifest,(('rust/Cargo.toml','d'*64),))


class AuthenticatedSetupInputTests(unittest.TestCase):
    def test_actual_source_proof_rejects_missing_mutated_or_conflicting_manifest(self) -> None:
        import json
        from dataclasses import replace

        from generic_build import workspace_pair_source
        from test_generic_workspace_policy import actual_tiny_pair

        pair,_runner=actual_tiny_pair(self)
        proof=workspace_pair_source(pair)
        selected=cli.setup_source_inputs(proof)
        manifest=document(proof.effective_json).field('manifest').mapping()
        self.assertTrue(manifest.keys() <= dict(selected).keys())
        self.assertIn('rust/Cargo.lock',dict(selected))
        name=next(iter(manifest))
        for mode in ('missing','mutated','conflicting'):
            with self.subTest(mode=mode):
                changed=dict(manifest)
                if mode=='missing':_=changed.pop(name)
                else:changed[name]=['100644','0'*64]
                candidate=replace(proof,effective_json=json.dumps({'manifest':changed,'sha256':'0'*64}))
                if mode=='conflicting':candidate=replace(candidate,effective_inputs=(*proof.effective_inputs,(name,'1'*64)))
                with self.assertRaises(ValueError):_=cli.setup_source_inputs(candidate)


class RetainedCompilerOutputTests(unittest.TestCase):
    def test_closed_normal_output_uses_retained_mode_and_hash(self) -> None:
        import tempfile

        from generic_process import file_hash

        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();path=root/'rust/target/debug/worldsim_probe';path.parent.mkdir(parents=True)
            _=path.write_bytes(b'inert normal compiler output');path.chmod(0o755)
            expected=file_hash(path);state=Node({'debug/worldsim_probe':f'file:{path.stat().st_mode}:{expected}'})
            self.assertEqual(cli.retained_normal_output(root,'debug',path,state),expected)
            for bad in (Node({}),Node({'debug/worldsim_probe':f'file:{path.stat().st_mode}:'+('0'*64)}),
                        Node({'debug/worldsim_probe':f'file:{0o100644}:{expected}'})):
                with self.assertRaises(ValueError):_=cli.retained_normal_output(root,'debug',path,bad)
            with self.assertRaises(ValueError):_=cli.retained_normal_output(root,'release',path,state)
            foreign=path.with_name('foreign');_=foreign.write_bytes(path.read_bytes());foreign.chmod(0o755)
            with self.assertRaises(ValueError):_=cli.retained_normal_output(root,'debug',foreign,state)
            path.chmod(0o644)
            with self.assertRaises(ValueError):_=cli.retained_normal_output(root,'debug',path,state)
            path.chmod(0o755);_=path.write_bytes(b'changed actual normal output')
            with self.assertRaises(ValueError):_=cli.retained_normal_output(root,'debug',path,state)


class ProcessPartitionTests(unittest.TestCase):
    def test_generated_inputs_stay_materialized_and_only_outside_paths_are_external(self) -> None:
        root=Path('/fixture/candidate')
        values={str(root/'tools/entry.py'):'a'*64,str(root/'rust/target/debug/deps/native'):'b'*64,
                '/fixture/harness/helper.py':'c'*64}
        self.assertEqual(cli.process_bindings(root,values),
            ((('rust/target/debug/deps/native','b'*64),('tools/entry.py','a'*64)),
             ((Path('/fixture/harness/helper.py'),'c'*64),)))

    def test_root_and_path_aliases_reject_before_partition(self) -> None:
        with self.assertRaises(GateContractError):
            _=cli.process_bindings(Path('/fixture/candidate/..'),{'/fixture/input':'a'*64})
        for path in ('relative','/fixture/candidate/../outside','/fixture/candidate//input'):
            with self.subTest(path=path),self.assertRaises(GateContractError):
                _=cli.process_bindings(Path('/fixture/candidate'),{path:'a'*64})


class PrefixNamespaceTests(unittest.TestCase):
    def test_closed_step_container_rejects_initial_and_later_membership_or_mode_changes(self) -> None:
        import tempfile

        import generic_full_gate as gate
        for change in ('initial-file','later-directory','later-mode'):
            with self.subTest(change=change),tempfile.TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();(root/'static').mkdir()
                if change=='initial-file':
                    _=(root/'late').write_text('extra')
                    with self.assertRaises(GateContractError):_=gate.prefix_container(1,root)
                else:
                    retained=gate.prefix_container(1,root)
                    if change=='later-directory':(root/'late-empty').mkdir()
                    else:root.chmod(0o700 if root.stat().st_mode&0o777 !=0o700 else 0o755)
                    with self.assertRaises(GateContractError):retained.recheck()

    def test_prefix_only_allows_original_scheduled_paths(self) -> None:
        import tempfile

        import generic_full_gate as gate
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();_= (root/'plan.json').write_text('{}');(root/'step-01').mkdir()
            mode=root.lstat().st_mode
            gate.prefix_membership(root,(1,),False,False,mode)
            (root/'step-02').mkdir()
            with self.assertRaises(GateContractError):gate.prefix_membership(root,(1,),False,False,mode)
            gate.prefix_membership(root,(1,2),False,False,mode)
            _=(root/'setup-prefix.json').write_text('{}')
            with self.assertRaises(GateContractError):gate.prefix_membership(root,(1,2),False,False,mode)
            gate.prefix_membership(root,(1,2),False,True,mode)
