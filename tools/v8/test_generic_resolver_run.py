"""Tiny shell/Python fixtures only; no native Godot or simulation evidence."""
import json
import os
import sys
import unittest
from collections.abc import Generator
from contextlib import contextmanager
from dataclasses import dataclass, replace
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import override
from unittest.mock import patch

import generic_resolver_run as resolver
import generic_static_run
from generic_fixtures import ProbeProduct
from generic_gate import document
from generic_process import ProcessResult, ProcessSpec, execute, file_hash
from generic_static_run import ExecutionEnvironment
from mux2_gate_inputs import Json

SOURCE=Path(generic_static_run.__file__).resolve().parent


@dataclass(frozen=True)
class Fixture:
    root: Path
    output: Path
    product: ProbeProduct
    context: ExecutionEnvironment
    external: tuple[tuple[Path,str],...]


@contextmanager
def fixture(*, native_code: int=0, bad_export: bool=False) -> Generator[Fixture]:
    with TemporaryDirectory() as folder:
        root=Path(folder).resolve();product=root/'product';tools=product/'tools/v8';tools.mkdir(parents=True)
        for name in ('run_bounded.sh','resolver_parity.py','resolver_parity_expected.json'):
            _=(tools/name).write_bytes((SOURCE/name).read_bytes())
        python=Path(sys.executable).resolve();bash=Path('/bin/bash').resolve();bin_dir=root/'bin';bin_dir.mkdir()
        (bin_dir/'python3').symlink_to(python);(bin_dir/'bash').symlink_to(bash)
        native=root/'fake-godot';_=native.write_text(f'''#!{python}
import json,sys
from pathlib import Path
if {native_code}:sys.exit({native_code})
x=json.loads(Path('tools/v8/resolver_parity_expected.json').read_text())
x['problems']=['synthetic failure'] if {bad_export!r} else []
x['flora_problems']=[]
Path(sys.argv[-1]).write_text(json.dumps(x))
print('RESOLVER_PARITY_DONE synthetic')
''');native.chmod(0o755)
        files=tuple((str(p.relative_to(product)),file_hash(p)) for p in sorted(tools.iterdir()))
        external=tuple((p,file_hash(p)) for p in (python,bash,native))
        lock=root/'lane';descriptor=os.open(lock,os.O_RDWR|os.O_CREAT,0o600)
        context=ExecutionEnvironment((('PATH',str(bin_dir)+':/usr/bin:/bin'),('HOME',str(root)),('TMPDIR',str(root)),
            ('GODOT_BIN',str(native)),('PYTHONNOUSERSITE','1'),('PYTHONDONTWRITEBYTECODE','1')),
            (('bash',bash,file_hash(bash)),('python3',python,file_hash(python))),external,held_lane_fd=descriptor)
        try:
            with patch.object(resolver,'LOCK',lock):
                yield Fixture(root,root/'output',ProbeProduct('a'*40,native,file_hash(native),product,files),context,external)
        finally:os.close(descriptor)


class ResolverTests(unittest.TestCase):
    def test_exact_original_recipe(self) -> None:
        with fixture() as f,patch.object(resolver,'bindings',return_value=f.external):
            plan=resolver.prepare(f.product,f.context,f.output)
            self.assertEqual(plan.process.argv,(str(Path('/bin/bash').resolve()),'-c',resolver.COMMAND,str(f.output)))
            self.assertIn('run_bounded.sh 300 ',resolver.COMMAND)
            self.assertIn(' && python3 ',resolver.COMMAND)
            self.assertNotIn('--generate',resolver.COMMAND)
            self.assertIsNone(plan.process.timeout_seconds)
            self.assertEqual(plan.process.cwd,f.product.input_root)

    def test_actual_original_checker_and_historical_authentication(self) -> None:
        with fixture() as f,patch.object(resolver,'bindings',return_value=f.external):
            result=resolver.run_owned(f.product,f.context,f.output)
            self.assertEqual(result.status,'PASS')
            row=document(result.receipt.read_text())
            self.assertEqual(row.field('source_build_admission').value,'PENDING')
            self.assertIsNone(row.field('native_pid').value)
            self.assertEqual(row.field('native_exit').value,'NOT_RECORDED')
            self.assertEqual(resolver.authenticate_result(f.product,f.context,f.output,file_hash(result.receipt)),result)
            _=(f.output/'resolver_godot.json').write_text('{}')
            with self.assertRaises(ValueError):_=resolver.authenticate_result(f.product,f.context,f.output,file_hash(result.receipt))

    def test_native_failure_preserves_and_checker_does_not_run(self) -> None:
        with fixture(native_code=3) as f,patch.object(resolver,'bindings',return_value=f.external):
            with self.assertRaises(resolver.ResolverOwnershipUnknown) as caught:
                _=resolver.run_owned(f.product,f.context,f.output)
            self.assertIsNotNone(caught.exception.original)
            self.assertFalse((f.output/'resolver_godot.json').exists())
            self.assertNotIn('resolver_parity:',(f.output/'process/stdout.log').read_text())
            self.assertFalse((f.output/'receipt.json').exists())
            row=document((f.output/'process/process.json').read_text())
            self.assertEqual(row.field('native_exit').value,3)

    def test_original_checker_loader_failure_remains_unknown(self) -> None:
        with fixture(bad_export=True) as f,patch.object(resolver,'bindings',return_value=f.external):
            with self.assertRaises(resolver.ResolverOwnershipUnknown):_=resolver.run_owned(f.product,f.context,f.output)
            self.assertIn('FAIL loader problems',(f.output/'process/stdout.log').read_text())
            self.assertTrue((f.output/'resolver_godot.json').exists())

    def test_unreturned_execution_never_snapshots(self) -> None:
        original=RuntimeError('actual execute failed')
        with fixture() as f,patch.object(resolver,'bindings',return_value=f.external), \
             patch.object(resolver,'execute',side_effect=original),patch.object(resolver,'raw_inventory') as snapshots:
            with self.assertRaises(resolver.ResolverOwnershipUnknown) as caught:
                _=resolver.run_owned(f.product,f.context,f.output)
            self.assertIs(caught.exception.original,original);snapshots.assert_not_called()

    def test_postcompletion_error_is_settled_and_retains_diagnostic_failure(self) -> None:
        original=RuntimeError('postcompletion error');secondary=SystemExit('diagnostic failed')
        with fixture() as f,patch.object(resolver,'bindings',return_value=f.external), \
             patch.object(resolver,'checker_terminal',side_effect=original),patch.object(resolver,'atomic_json',side_effect=secondary):
            with self.assertRaises(resolver.ResolverSettledFailure) as caught:
                _=resolver.run_owned(f.product,f.context,f.output)
            self.assertIs(caught.exception.original,original)
            self.assertEqual(caught.exception.secondary,(secondary,));self.assertIsNone(caught.exception.receipt)

    def test_no_exception_stringification_even_without_terminal(self) -> None:
        class Unprintable(RuntimeError):
            @override
            def __str__(self) -> str:raise AssertionError('must not render')
        original=Unprintable()
        with fixture() as f,patch.object(resolver,'bindings',return_value=f.external), \
             patch.object(resolver,'execute',side_effect=original),patch.object(resolver,'atomic_json',side_effect=OSError('disk')):
            with self.assertRaises(resolver.ResolverOwnershipUnknown) as caught:
                _=resolver.run_owned(f.product,f.context,f.output)
            self.assertIs(caught.exception.original,original);self.assertEqual(len(caught.exception.secondary),1)

    def test_late_output_mutation_is_settled_failure(self) -> None:
        with fixture() as f,patch.object(resolver,'bindings',return_value=f.external):
            original=resolver.checker_terminal
            def mutate(plan: resolver.ResolverPlan,stdout: bytes) -> dict[str,Json]:
                result=original(plan,stdout)
                _=(f.output/'resolver_godot.json').write_text('{}')
                return result
            with patch.object(resolver,'checker_terminal',side_effect=mutate),self.assertRaises(resolver.ResolverSettledFailure):
                _=resolver.run_owned(f.product,f.context,f.output)

    def test_bool_pid_cannot_settle(self) -> None:
        with fixture() as f,patch.object(resolver,'bindings',return_value=f.external):
            def wrong(spec: ProcessSpec) -> ProcessResult:
                actual=execute(spec)
                return replace(actual,pid=True)
            with patch.object(resolver,'execute',side_effect=wrong),patch.object(resolver,'raw_inventory') as snapshots, \
                 self.assertRaises(resolver.ResolverOwnershipUnknown):
                _=resolver.run_owned(f.product,f.context,f.output)
            snapshots.assert_not_called()

    def test_unreviewed_sources_reject_before_child(self) -> None:
        with fixture() as f,patch.object(resolver,'execute') as child:
            with self.assertRaises(resolver.ResolverSettledFailure):_=resolver.run_owned(f.product,f.context,f.output)
            child.assert_not_called();self.assertFalse(f.output.exists())

    def test_unknown_keeps_actual_temporary_lock(self) -> None:
        import fcntl
        with fixture(native_code=2) as f,patch.object(resolver,'bindings',return_value=f.external):
            with self.assertRaises(resolver.ResolverOwnershipUnknown):_=resolver.run_owned(f.product,f.context,f.output)
            with (f.root/'lane').open('a') as other,self.assertRaises(BlockingIOError):
                fcntl.flock(other.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)

    def test_historical_status_only_and_extra_output_rejected(self) -> None:
        with fixture() as f,patch.object(resolver,'bindings',return_value=f.external):
            result=resolver.run_owned(f.product,f.context,f.output)
            _=(f.output/'foreign').write_text('unexpected')
            with self.assertRaises(ValueError):_=resolver.authenticate_result(f.product,f.context,f.output,file_hash(result.receipt))
            (f.output/'foreign').unlink();_=result.receipt.write_text(json.dumps({'status':'PASS'}))
            with self.assertRaises(ValueError):_=resolver.authenticate_result(f.product,f.context,f.output,file_hash(result.receipt))

    def test_binding_policy_rejects_runtime_source_and_tool_changes(self) -> None:
        pins: dict[str,str]={}
        with fixture() as f,patch.object(resolver,'PINS',pins),patch.object(resolver,'resource_names',return_value=['tools']):
            _=resolver.bindings(f.product,f.context)
            for context in (replace(f.context,values=(*f.context.values,('PYTHONPATH','/foreign'))),
                            replace(f.context,values=(*f.context.values,('WORLDSIM_ORACLE_SUITE_PLAN','/foreign'))),
                            replace(f.context,tools=()),replace(f.context,external_inputs=())):
                with self.assertRaises(ValueError):_=resolver.bindings(f.product,context)
            file=f.product.input_root/'tools/v8/resolver_parity_expected.json'
            _=file.write_text('{}')
            with self.assertRaises(ValueError):_=resolver.bindings(f.product,f.context)

    def test_flora_fixture_cannot_escape_product(self) -> None:
        with fixture() as f,patch.object(resolver,'resolution_paths',return_value=set()):
            file=f.product.input_root/'tools/v8/flora_parity_fixture.json'
            for key in ('../outside','/absolute','with space','a/b',2):
                _=file.write_text(json.dumps({'cases':[{'fixture':key}]}))
                with self.assertRaises(ValueError):_=resolver.resource_names(f.product.input_root)

    def test_negative_none_and_zero_pid_never_settle(self) -> None:
        with fixture() as f,patch.object(resolver,'bindings',return_value=f.external):
            plan=resolver.prepare(f.product,f.context,f.output)
            for result in (ProcessResult(f.output/'process/process.json','FAIL',-9,123),
                           ProcessResult(f.output/'process/process.json','FAIL',None,None),
                           ProcessResult(f.output/'process/process.json','PASS',0,0)):
                with self.assertRaises(ValueError):_=resolver.completed(plan,result,'0'*64)

    def test_changed_log_and_forged_pid_are_not_historical_proof(self) -> None:
        with fixture() as f,patch.object(resolver,'bindings',return_value=f.external):
            result=resolver.run_owned(f.product,f.context,f.output)
            raw=result.receipt.read_bytes();row=document(raw.decode()).mapping();row['parent_pid']=0
            _=result.receipt.write_text(json.dumps(row))
            with self.assertRaises(ValueError):_=resolver.authenticate_result(f.product,f.context,f.output,file_hash(result.receipt))
            _=result.receipt.write_bytes(raw)
            _=(f.output/'process/stdout.log').write_text('forged PASS')
            with self.assertRaises(ValueError):_=resolver.authenticate_result(f.product,f.context,f.output,file_hash(result.receipt))

    def test_postcompletion_source_drift_preserves_actual_exception(self) -> None:
        error=RuntimeError('source changed')
        with fixture() as f,patch.object(resolver,'bindings',side_effect=[f.external,error]):
            with self.assertRaises(resolver.ResolverSettledFailure) as caught:
                _=resolver.run_owned(f.product,f.context,f.output)
            self.assertIs(caught.exception.original,error)
            self.assertTrue((f.output/'settled-failure.json').exists())


    def test_real_resource_closure_and_bindings_have_no_overlapping_scopes(self) -> None:
        from generic_sources import inventory
        from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES

        with fixture() as f:
            product=f.product.input_root
            for name in INPUT_DIRS:
                (product/name).mkdir(parents=True,exist_ok=True)
            for name in (*INPUT_FILES,*resolver.PINS):
                target=product/name;target.parent.mkdir(parents=True,exist_ok=True)
                source=SOURCE.parents[1]/name
                _=target.write_bytes(source.read_bytes() if name in resolver.PINS else b'synthetic source only')
            for name in ('scripts/core/sim_reads.gd','scripts/ui/person_names.gd','scripts/world/art_hash.gd'):
                target=product/name;target.parent.mkdir(parents=True,exist_ok=True)
                _=target.write_bytes((SOURCE.parents[1]/name).read_bytes())
            for name in ('resolver_parity_fixture.json','flora_parity_fixture.json'):
                _=(product/'tools/v8'/name).write_bytes((SOURCE/name).read_bytes())
            for name in ('manifest.json','bindings.json','render_params.json','materials/construction_styles.json',
                         'materials/material_looks.json','materials/category_defaults.json',
                         'flora/species.json','flora/habit_defaults.json','flora/tests/test_alder.json'):
                target=product/'assets/v8art'/name;target.parent.mkdir(parents=True,exist_ok=True);_=target.write_text('{}')
            _=(product/'localization/manifest.json').write_text('{}')
            (product/'.godot/imported').mkdir(parents=True)
            _=(product/'.godot/global_script_class_cache.cfg').write_text('list=[{\n"base": &"Node",\n"class": &"SimReads",\n"icon": "",\n"is_abstract": false,\n"is_tool": false,\n"language": &"GDScript",\n"path": "res://scripts/core/sim_reads.gd"\n}, {\n"base": &"RefCounted",\n"class": &"PersonNames",\n"icon": "",\n"is_abstract": false,\n"is_tool": false,\n"language": &"GDScript",\n"path": "res://scripts/ui/person_names.gd"\n}, {\n"base": &"RefCounted",\n"class": &"ArtHash",\n"icon": "",\n"is_abstract": false,\n"is_tool": false,\n"language": &"GDScript",\n"path": "res://scripts/world/art_hash.gd"\n}]\n')
            _=(product/'.godot/extension_list.cfg').write_text('res://sim_bridge.gdextension\n')
            uri=b'res://assets/fixture.png'
            _=(product/'.godot/uid_cache.bin').write_bytes((1).to_bytes(4,'little')+(7).to_bytes(8,'little')+len(uri).to_bytes(4,'little')+uri)
            _=(product/'assets/fixture.png').write_bytes(b'synthetic image')
            _=(product/'.godot/imported/fixture.ctex').write_bytes(b'synthetic import')
            _=(product/'assets/fixture.png.import').write_text(
                '[remap]\nimporter="texture"\ntype="CompressedTexture2D"\npath="res://.godot/imported/fixture.ctex"\n'
                + '[deps]\nsource_file="res://assets/fixture.png"\ndest_files=["res://.godot/imported/fixture.ctex"]\n')
            for profile in ('debug','release'):
                target=product/'rust/target'/profile/'libsim_bridge.dylib';target.parent.mkdir(parents=True,exist_ok=True)
                _=target.write_bytes(b'synthetic library never loaded')
            expected=inventory(product,['rust','scripts','scenes','shaders','assets','localization','tools','.godot','project.godot','sim_bridge.gdextension'])
            admitted=replace(f.product,input_files=tuple(sorted(expected.items())))
            scopes=resolver.resource_names(product)
            self.assertEqual(inventory(product,scopes),expected)
            _=resolver.bindings(admitted,f.context)
            _=resolver.prepare(admitted,f.context,f.output)
            extra=product/'assets/new.json';_=extra.write_text('{}')
            with self.assertRaises(ValueError):_=resolver.bindings(admitted,f.context)
            extra.unlink()
            mandatory=product/'assets/v8art/materials/material_looks.json';mandatory.unlink()
            reduced=replace(admitted,input_files=tuple((name,digest) for name,digest in admitted.input_files if name!=str(mandatory.relative_to(product))))
            with self.assertRaises((ValueError,OSError)):_=resolver.bindings(reduced,f.context)
            mandatory.symlink_to(product/'assets/v8art/materials/category_defaults.json')
            with self.assertRaises((ValueError,OSError)):_=resolver.bindings(reduced,f.context)
