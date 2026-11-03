"""Actual tiny compiler child receipts; no native Cargo/worlds or mocked pair admission."""
import subprocess
import sys
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_build as build
import test_generic_build as fixtures
from generic_process import file_hash

COMPILER = r'''
import json,os,pathlib,sys
if '--version' in sys.argv:
 print('synthetic compiler 1.93.0\nrelease: 1.93.0\nhost: synthetic-test-only');sys.exit(0)
root=pathlib.Path.cwd();target=pathlib.Path(os.environ['CARGO_TARGET_DIR']);packages=[]
for package_name,normal in (('alpha','shared'),('beta','perf')):
 p=root/'crates'/package_name
 packages.append({'id':'path+'+str(p),'name':package_name,'manifest_path':str(p/'Cargo.toml'),
  'targets':[{'name':'shared','kind':['lib'],'crate_types':['lib'],'src_path':str(p/'src/lib.rs'),'test':True,'doctest':True},
             {'name':normal,'kind':['bin'],'crate_types':['bin'],'src_path':str(p/'src/bin'/f'{normal}.rs'),'test':False,'doctest':False}]})
if sys.argv[1]=='metadata':
 print(json.dumps({'version':1,'workspace_root':str(root),'target_directory':str(target),
  'workspace_members':[p['id'] for p in packages],'packages':packages}));sys.exit(0)
assert '--workspace' in sys.argv and '--no-run' in sys.argv
profile='release' if '--release' in sys.argv else 'debug'
for package in packages:
 binary=target/profile/'deps'/(package['name']+'-hash')
 binary.parent.mkdir(parents=True,exist_ok=True);binary.write_bytes(b'synthetic test only');binary.chmod(0o755)
 print(json.dumps({'reason':'compiler-artifact','package_id':package['id'],'manifest_path':package['manifest_path'],
  'target':package['targets'][0],'profile':{'test':True},'fresh':False,'executable':str(binary),'filenames':[str(binary)]}))
 normal=package['targets'][1];out=target/profile/normal['name']
 if FAULT!='missing-output':out.write_bytes(b'synthetic normal only');out.chmod(0o755)
 row={'reason':'compiler-artifact','package_id':package['id'],'manifest_path':package['manifest_path'],
  'target':dict(normal),'profile':{'test':False},'fresh':False,'executable':str(out),'filenames':[str(out)]}
 if FAULT=='wrong-source':row['target']['src_path']=package['targets'][0]['src_path']
 if FAULT=='wrong-manifest':row['manifest_path']=str(root/'Cargo.toml')
 if FAULT=='wrong-package':row['package_id']='foreign-package'
 if FAULT=='wrong-profile':row['executable']=str(target/('debug' if profile=='release' else 'release')/normal['name']);row['filenames']=[row['executable']]
 if FAULT=='wrong-crate':row['target']['crate_types']=['proc-macro']
 if FAULT=='fresh':row['fresh']=True
 if FAULT=='duplicate-filename':row['filenames']*=2
 if FAULT=='macro-only':row['target']['kind']=['proc-macro'];row['target']['crate_types']=['proc-macro']
 if FAULT!='missing-release' or profile!='release':print(json.dumps(row))
 if FAULT=='duplicate-record':print(json.dumps(row))
 macro=dict(row);macro.update(package_id='registry+synthetic-macro',executable=None,filenames=[])
 macro['target']={'name':normal['name'],'kind':['proc-macro'],'crate_types':['proc-macro']}
 print(json.dumps(macro))
print(json.dumps({'reason':'build-finished','success':True}))
'''


class NormalProductTests(unittest.TestCase):
    def fixture(self, fault: str = '') -> tuple[build.WorkspaceBuildPair, fixtures.WorkspaceBuildTests]:
        producer=fixtures.WorkspaceBuildTests();producer.setUp();self.addCleanup(producer.doCleanups)
        producer.workspace_spec()
        for package,name in (('alpha','shared'),('beta','perf')):
            source=producer.root/'rust/crates'/package/'src/bin'/f'{name}.rs'
            source.parent.mkdir();_=source.write_text('synthetic normal source')
        _=subprocess.run(('git','-C',str(producer.root),'add','.'),check=True,capture_output=True)
        _=subprocess.run(('git','-C',str(producer.root),'-c','user.name=Test','-c','user.email=t@example.invalid','commit','-qm','normal fixtures'),check=True,capture_output=True)
        sha=subprocess.check_output(('git','-C',str(producer.root),'rev-parse','HEAD'),text=True).strip()
        _=producer.compiler.write_text('#!'+sys.executable+'\nFAULT='+repr(fault)+'\n'+COMPILER)
        spec=replace(producer.spec,source_sha=sha,cargo_sha256=file_hash(producer.compiler),rustc_sha256=file_hash(producer.compiler))
        debug=replace(spec,profile='debug',output=producer.home/'debug')
        release=replace(spec,profile='release',output=producer.home/'release')
        pair=build.build_workspace_pair(debug,release,producer.home/'pair')
        self.assertEqual(pair.status,'BUILD_ANTECEDENTS_ONLY',pair.receipt.read_text())
        return pair,producer

    def test_actual_pair_separates_profiles_tests_macros_and_normal_bins(self) -> None:
        pair,_=self.fixture()
        for profile in ('debug','release'):
            products=build.normal_workspace_products(pair,profile,(('alpha','shared'),('beta','perf')))
            self.assertIs(products.pair,pair)
            self.assertFalse(products.publication_authority)
            self.assertEqual(products.runtime_qualification,'NOT_RUN')
            self.assertEqual(tuple(p.name for p in products.products),('shared','perf'))
            selected=pair.debug if profile=='debug' else pair.release
            assert selected is not None
            self.assertTrue(all(p.binary.parent.name==profile and p.binary not in {path for _,path,_ in selected.artifacts} for p in products.products))
            build.recheck_normal_products(products)
            with self.assertRaises(ValueError):build.recheck_normal_products(replace(products,retained_anchor=None))
            with self.assertRaises(ValueError):build.recheck_normal_products(replace(products,products=tuple(replace(p,sha256='0'*64) for p in products.products)))
        with self.assertRaises(ValueError):_=build.normal_workspace_products(replace(pair,retained_anchor=None),'release',(('alpha','shared'),))

    def test_bad_actual_compiler_records_are_not_normal_product_authority(self) -> None:
        for fault in ('missing-release','missing-output','wrong-source','wrong-manifest','wrong-package','wrong-profile','wrong-crate','fresh','duplicate-filename','macro-only','duplicate-record'):
            with self.subTest(fault=fault):
                pair,_=self.fixture(fault)
                with self.assertRaises((OSError,ValueError)):_=build.normal_workspace_products(pair,'release',(('alpha','shared'),))

    def test_retained_target_stream_source_and_namespace_drift_reject(self) -> None:
        for fault in ('binary','stream','source','empty-directory'):
            with self.subTest(fault=fault):
                pair,producer=self.fixture()
                selected=build.normal_workspace_products(pair,'release',(('alpha','shared'),))
                assert pair.release is not None
                if fault=='binary':_=selected.products[0].binary.write_bytes(b'changed')
                if fault=='stream':_= (pair.release.receipt.parent/'cargo-build/stdout.log').write_text('{}')
                if fault=='source':_=selected.products[0].source.write_text('changed')
                if fault=='empty-directory':(producer.root/'rust/target/unobserved-empty').mkdir()
                with self.assertRaises(ValueError):build.recheck_normal_products(selected)

    def test_late_read_callback_drift_is_not_adopted(self) -> None:
        pair,producer=self.fixture()
        altered=False
        def changed(path: Path) -> str:
            nonlocal altered
            result=file_hash(path)
            if path==producer.root/'rust/target/release/shared' and not altered:
                altered=True
                assert pair.release is not None
                _=(pair.release.receipt.parent/'cargo-build/stdout.log').write_text('{}')
            return result
        with patch.object(build,'file_hash',side_effect=changed),self.assertRaises(ValueError):
            _=build.normal_workspace_products(pair,'release',(('alpha','shared'),))
        self.assertTrue(altered)

    def test_selectors_cannot_choose_unrelated_or_empty_products(self) -> None:
        pair,_=self.fixture()
        for names in ((),(('missing','shared'),),(('alpha','missing'),),(('alpha','shared'),('alpha','shared'))):
            with self.subTest(names=names),self.assertRaises(ValueError):_=build.normal_workspace_products(pair,'release',names)

    def test_final_pair_callback_cannot_change_returned_binary(self) -> None:
        pair,producer=self.fixture()
        original=build.recheck_workspace_pair;calls=0
        def changed(actual: build.WorkspaceBuildPair) -> tuple[build.BuildSpec,build.BuildSpec]:
            nonlocal calls
            specs=original(actual);calls+=1
            if calls==2:_=(producer.root/'rust/target/release/shared').write_bytes(b'late mutation')
            return specs
        with patch.object(build,'recheck_workspace_pair',side_effect=changed),self.assertRaises(ValueError):
            _=build.normal_workspace_products(pair,'release',(('alpha','shared'),))

    def test_final_pair_callback_preserves_full_closure_on_selection_and_recheck(self) -> None:
        for retained in (False, True):
            for fault in ('empty-directory', 'source', 'unselected-binary', 'selected-mode', 'evidence-directory'):
                with self.subTest(retained=retained, fault=fault):
                    self.assert_final_callback_rejected(retained, fault)

    def assert_final_callback_rejected(self, retained: bool, fault: str) -> None:
        pair, producer = self.fixture()
        selected = build.normal_workspace_products(pair, 'release', (('alpha', 'shared'),))
        original = build.recheck_workspace_pair
        calls = 0
        def changed(actual: build.WorkspaceBuildPair) -> tuple[build.BuildSpec, build.BuildSpec]:
            nonlocal calls
            specs = original(actual)
            calls += 1
            if calls == 2:
                if fault == 'empty-directory':
                    (producer.root/'rust/target/final-empty').mkdir()
                if fault == 'source':
                    _ = (producer.root/'rust/crates/alpha/src/bin/shared.rs').write_text('changed')
                if fault == 'unselected-binary':
                    _ = (producer.root/'rust/target/release/perf').write_bytes(b'changed')
                if fault == 'selected-mode':
                    (producer.root/'rust/target/release/shared').chmod(0o644)
                if fault == 'evidence-directory':
                    (pair.receipt.parent/'final-empty').mkdir()
            return specs
        with patch.object(build, 'recheck_workspace_pair', side_effect=changed), self.assertRaises(ValueError):
            if retained:
                build.recheck_normal_products(selected)
            else:
                _ = build.normal_workspace_products(pair, 'release', (('alpha', 'shared'),))
        self.assertEqual(calls, 2)
