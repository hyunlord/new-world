"""Real tiny compiler children and retained pairs; no Cargo/game execution."""
import subprocess
import sys
import unittest
from collections.abc import Callable
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_cli_prepare as cli
import generic_cli_products as products
import test_generic_cli_prepare as fixtures
from generic_bench import AuditError, Product, validate_products
from generic_gate import document
from generic_process import atomic_json, file_hash
from generic_sources import inventory
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Json

COMPILER = r'''
import json,os,pathlib,sys
if '--version' in sys.argv:
 print('synthetic compiler 1.93.0\nrelease: 1.93.0\nhost: synthetic-test-only');sys.exit(0)
root=pathlib.Path.cwd();target=pathlib.Path(os.environ['CARGO_TARGET_DIR']);p=root/'crates/sim-test'
targets=[{'name':'sim_test','kind':['lib'],'crate_types':['lib'],'src_path':str(p/'src/lib.rs'),'test':True,'doctest':True}]
for name in ('worldsim_probe','worldsim_perf'):
 targets.append({'name':name,'kind':['bin'],'crate_types':['bin'],'src_path':str(p/'src/bin'/f'{name}.rs'),'test':False,'doctest':False})
package={'id':'path+'+str(p),'name':'sim-test','manifest_path':str(p/'Cargo.toml'),'targets':targets}
if sys.argv[1]=='metadata':
 print(json.dumps({'version':1,'workspace_root':str(root),'target_directory':str(target),'workspace_members':[package['id']],'packages':[package]}));sys.exit(0)
assert '--workspace' in sys.argv and '--no-run' in sys.argv
profile='release' if '--release' in sys.argv else 'debug'
for t in targets:
 test=t['kind']==['lib'];out=target/profile/('deps/sim_test-hash' if test else t['name'])
 out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(b'synthetic executable only');out.chmod(0o755)
 if OMIT and t['name']=='worldsim_perf' and profile=='release':continue
 print(json.dumps({'reason':'compiler-artifact','package_id':package['id'],'manifest_path':package['manifest_path'],
 'target':t,'profile':{'test':test},'fresh':False,'executable':str(out),'filenames':[str(out)]}))
print(json.dumps({'reason':'build-finished','success':True}))
'''


class ProductTests(unittest.TestCase):
    def fixture(self, *, omit: bool = False) -> tuple[cli.CliBuilds, Path]:
        helper=fixtures.CliPrepareTests();self.addCleanup(helper.doCleanups);helper.setUp()
        selected=helper.selected();root=selected.base.root
        for name in ('Cargo.toml','src/lib.rs','src/bin/worldsim_probe.rs','src/bin/worldsim_perf.rs'):
            path=root/'rust/crates/sim-test'/name;path.parent.mkdir(parents=True,exist_ok=True)
            _=path.write_text('synthetic normal '+name)
        _=subprocess.run(('git','-C',str(root),'add','.'),check=True,capture_output=True)
        _=subprocess.run(('git','-C',str(root),'-c','user.name=Test','-c','user.email=t@example.invalid','commit','-qm','normal product fixture'),check=True,capture_output=True)
        sha=subprocess.check_output(('git','-C',str(root),'rev-parse','HEAD'),text=True).strip()
        for other in (selected.candidate.root,selected.harness_root):
            _=subprocess.run(('git','-C',str(other),'fetch','-q',str(root),sha),check=True,capture_output=True)
            _=subprocess.run(('git','-C',str(other),'merge','--ff-only','-q','FETCH_HEAD'),check=True,capture_output=True)
        _=subprocess.run(('git','-C',str(selected.harness_root),'update-ref','refs/remotes/origin/v8/main',sha),check=True,capture_output=True)
        _=selected.cargo.write_text('#!'+sys.executable+'\nOMIT='+repr(omit)+'\n'+COMPILER)
        selected=replace(selected,harness_sha=sha,published_sha=sha,
            base=replace(selected.base,source_sha=sha),candidate=replace(selected.candidate,source_sha=sha),
            cargo_sha256=file_hash(selected.cargo),rustc_sha256=file_hash(selected.rustc))
        actual=cli.build_cli(cli.prepare_cli(selected))
        self.assertEqual(actual.status,'BUILD_ANTECEDENTS_ONLY')
        return actual,helper.fixture.home/'products'

    def test_real_pairs_supply_normal_products_and_effective_inventory(self) -> None:
        actual,out=self.fixture();result=products.construct_products(actual,out)
        self.assertIs(result.builds,actual);self.assertIs(result.normal[0].pair,actual.base)
        self.assertIs(result.normal[1].pair,actual.candidate)
        self.assertEqual(result.runtime_qualification,'NOT_RUN');self.assertFalse(result.publication_authority)
        for arm,probe,perf,source,runtime in zip(('base','candidate'),(result.probes.base,result.probes.candidate),result.perf,result.sources,result.runtime_inputs,strict=True):
            self.assertEqual(probe.executable.name,'worldsim_probe');self.assertEqual(perf.binary.name,'worldsim_perf')
            self.assertEqual(dict(probe.input_files),dict(perf.input_files))
            self.assertEqual(source.effective_inputs,probe.input_files)
            self.assertEqual(inventory(perf.root,[*INPUT_DIRS,*INPUT_FILES]),dict(perf.input_files))
            self.assertIn('rust/target/release/worldsim_probe',dict(runtime))
            self.assertNotIn('rust/target/release/worldsim_probe',dict(probe.input_files))
            row=document((out/f'{arm}-perf.json').read_text())
            row.field('kind').equals('normal_product_adapter_projection')
            row.field('normal_selection').field('name').equals('worldsim_perf')
            self.assertEqual(row.field('exit_code').value,0)
        validate_products(*result.perf,'self_proof');products.recheck_products(result)
        for changed in (replace(result,retained_anchor=None),replace(result,perf=(result.perf[1],result.perf[0])),
                        replace(result,builds=replace(actual)),replace(result,runtime_inputs=((),())),replace(result,runtime_qualification='PASS')):
            with self.assertRaises(ValueError):products.recheck_products(changed)

    def test_wrong_actual_arms_and_missing_release_record_reject(self) -> None:
        actual,out=self.fixture()
        for changed in (replace(actual,base=actual.candidate),replace(actual,candidate=actual.base),replace(actual,status='FAIL')):
            with self.assertRaises(ValueError):_=products.construct_products(changed,out)
        absent,other=self.fixture(omit=True)
        with self.assertRaises(ValueError):_=products.construct_products(absent,other)
        self.assertFalse(other.exists())

    def test_setup_target_changes_require_new_endpoint_not_refresh(self) -> None:
        actual,out=self.fixture();result=products.construct_products(actual,out)
        (actual.prepared.selectors.candidate.root/'rust/target/setup-later').mkdir()
        with self.assertRaises(ValueError):products.recheck_products(result)
        with self.assertRaises(ValueError):_=products.construct_products(actual,out.with_name('refreshed'))
        self.assertFalse(out.with_name('refreshed').exists())

    def test_late_sidecar_and_prior_source_mutations_preserve_failure(self) -> None:
        for fault in ('receipt','empty-directory','source'):
            with self.subTest(fault=fault):
                actual,out=self.fixture()
                def publish(path: Path,value: dict[str,Json], *, fault: str=fault, out: Path=out, actual: cli.CliBuilds=actual) -> None:
                    atomic_json(path,value)
                    if path.name=='candidate-perf.json':
                        if fault=='receipt':_= (out/'base-perf.json').write_bytes(b'changed')
                        elif fault=='empty-directory':(out/'late-empty').mkdir()
                        else:_=(actual.prepared.selectors.base.root/'tools/v8/input.txt').write_bytes(b'changed')
                with patch.object(products,'atomic_json',side_effect=publish),self.assertRaises((ValueError,AuditError)):
                    _=products.construct_products(actual,out)
                self.assertTrue(out.exists());self.assertTrue((out/'candidate-perf.json').exists())

    def test_final_validator_cannot_mutate_published_namespace(self) -> None:
        actual,out=self.fixture();result=products.construct_products(actual,out)
        def verify(base: Product, candidate: Product, purpose: str) -> None:
            validate_products(base,candidate,purpose)
            (out/'after-validator').mkdir()
        with patch.object(products,'validate_products',side_effect=verify),self.assertRaises(ValueError):
            products.recheck_products(result)
        self.assertTrue((out/'after-validator').is_dir())

    def test_first_publication_cannot_define_modes(self) -> None:
        for directory in (False,True):
            with self.subTest(directory=directory):
                actual,out=self.fixture()
                def publish(path: Path,value: dict[str,Json], *, out: Path=out, directory: bool=directory) -> None:
                    atomic_json(path,value)
                    if path.name=='base-perf.json':
                        (out if directory else path).chmod(0o777)
                with patch.object(products,'atomic_json',side_effect=publish),self.assertRaises(ValueError):
                    _=products.construct_products(actual,out)
                self.assertEqual((out if directory else out/'base-perf.json').stat().st_mode & 0o777,0o777)

    def assert_terminal_closure(self, construction: bool) -> None:
        for fault in ('source','target','executable','sidecar','output'):
            with self.subTest(construction=construction,fault=fault):
                actual,out=self.fixture()
                result=None if construction else products.construct_products(actual,out)
                original=cli.recheck_cli_builds
                calls=0
                def check(value: cli.CliBuilds, *, actual: cli.CliBuilds=actual, out: Path=out, fault: str=fault, original: Callable[[cli.CliBuilds],None]=original) -> None:
                    nonlocal calls
                    original(value);calls+=1
                    if calls==(4 if construction else 2):
                        root=actual.prepared.selectors.base.root
                        if fault=='source':_=(root/'tools/v8/input.txt').write_bytes(b'late source')
                        elif fault=='target':(root/'rust/target/late-empty').mkdir()
                        elif fault=='executable':(root/'rust/target/release/worldsim_perf').chmod(0o644)
                        elif fault=='sidecar':(out/'base-perf.json').chmod(0o777)
                        else:out.chmod(0o777)
                with patch.object(products,'recheck_cli_builds',side_effect=check),self.assertRaises(ValueError):
                    if construction:_=products.construct_products(actual,out)
                    else:
                        assert result is not None
                        products.recheck_products(result)
                self.assertEqual(calls,4 if construction else 2)

    def test_construct_terminal_original_closure_and_intended_modes(self) -> None:
        self.assert_terminal_closure(True)

    def test_retained_terminal_original_closure_and_intended_modes(self) -> None:
        self.assert_terminal_closure(False)

    def test_arbitrary_digest_only_products_are_not_a_handoff(self) -> None:
        # No arbitrary path/hash constructor exists: actual pair admission is first.
        actual,out=self.fixture()
        assert actual.candidate is not None
        with self.assertRaises(ValueError):
            _=products.construct_products(replace(actual,candidate=replace(actual.candidate,retained_anchor=None)),out)
        self.assertFalse(out.exists())
