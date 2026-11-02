"""Tiny Git and fake compiler processes, never Cargo/Godot or simulation evidence."""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_stall_build as subject
from generic_build import Artifact, BuildSpec, Overlay, build_product
from generic_gate import document
from generic_process import ProcessResult, ProcessSpec, execute, file_hash
from generic_sources import inventory
from generic_stall_source import DerivedSourceBinding, recheck_derived_source
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Json


class Fixture:
    def __init__(self, home: Path) -> None:
        self.home: Path=home
        self.root: Path=home/'product';self.root.mkdir()
        found=shutil.which('git')
        if found is None:raise RuntimeError('Git required for tiny repository tests')
        self.git: Path=Path(found).resolve()
        _=self.run('init','-q')
        for name in INPUT_DIRS:
            path=self.root/name;path.mkdir(parents=True,exist_ok=True);_=(path/'input').write_text('stable')
        for name in INPUT_FILES:
            path=self.root/name;path.parent.mkdir(parents=True,exist_ok=True);_=path.write_text('input')
        _=(self.root/'rust/rust-toolchain.toml').write_text('[toolchain]\nchannel="1.93"\n')
        _=(self.root/'rust/Cargo.toml').write_text('[workspace]\nmembers=["crates/sim-bridge"]\n[workspace.package]\nversion="0.7.0"\n')
        crate=self.root/'rust/crates/sim-bridge';(crate/'src').mkdir(parents=True)
        _=(crate/'Cargo.toml').write_text('[package]\nname="sim-bridge"\nversion.workspace=true\n[lib]\ncrate-type=["cdylib","rlib"]\n')
        _=(crate/'src/lib.rs').write_text('// tiny source fixture, never compiled\n')
        _=(self.root/'sim_bridge.gdextension').write_text('[libraries]\nmacos.debug = "res://rust/target/debug/libsim_bridge.dylib"\n')
        _=(self.root/'.gitignore').write_text('target/\n')
        self.changed: Path=self.root/'tools/v8/observer.py';_=self.changed.write_text('old\n')
        self.changed.chmod(0o755)
        _=self.run('add','.');_=self.run('-c','user.name=Tiny','-c','user.email=t@example.invalid','commit','-qm','fixture')
        self.sha: str=self.run('rev-parse','HEAD').decode().strip()
        _=self.changed.write_text('new\n')
        self.patch: Path=home/'overlay.patch'
        _=self.patch.write_bytes(self.run('diff','--no-ext-diff')+b'diff --git a/tools/v8/added.py b/tools/v8/added.py\nnew file mode 100644\n--- /dev/null\n+++ b/tools/v8/added.py\n@@ -0,0 +1 @@\n+added\n')
        _=self.run('checkout','--','tools/v8/observer.py')
        self.manifest: Path=home/'overlay.json'
        import hashlib
        rows: list[Json]=[{'path':'tools/v8/observer.py','operation':'modify','base0_sha256':hashlib.sha256(b'old\n').hexdigest(),'observer_sha256':hashlib.sha256(b'new\n').hexdigest()},
            {'path':'tools/v8/added.py','operation':'add','base0_sha256':None,'observer_sha256':hashlib.sha256(b'added\n').hexdigest()}]
        _=self.manifest.write_text(json.dumps({'schema_version':1,'product_base_sha':self.sha,'observer_patch_sha256':file_hash(self.patch),
            'allowed_paths':['tools/v8/observer.py','tools/v8/added.py'],'files':rows}))
        self.overlay: Overlay=Overlay(self.patch,file_hash(self.patch),self.manifest,file_hash(self.manifest))
        self.transform: subject.TransformSpec=subject.TransformSpec(self.root,self.sha,self.overlay,self.git,file_hash(self.git),home/'scratch',home/'transform')
        compiler=home/'compiler'
        _=compiler.write_text('#!'+sys.executable+'''\nimport json,os,pathlib,sys
if '--version' in sys.argv:
 print('synthetic 1.93.0\\nrelease: 1.93.0\\nhost: test-only');sys.exit(0)
p=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'debug'/'libsim_bridge.dylib'
p.parent.mkdir(parents=True);p.write_bytes(b'fake bridge not loadable')
crate=pathlib.Path.cwd()/'crates'/'sim-bridge'
print(json.dumps({'reason':'compiler-artifact','package_id':'path+'+crate.as_uri()+'#0.7.0','manifest_path':str(crate/'Cargo.toml'),
 'target':{'name':'sim_bridge','kind':['cdylib','rlib'],'crate_types':['cdylib','rlib'],'src_path':str(crate/'src/lib.rs')},
 'fresh':False,'filenames':[str(p)]}))
print(json.dumps({'reason':'build-finished','success':True}))
''');compiler.chmod(0o755)
        cargo_home=home/'cargo-home';cargo_home.mkdir()
        self.build: BuildSpec=BuildSpec(self.root,self.sha,compiler,file_hash(compiler),compiler,file_hash(compiler),cargo_home,
            home/'build',('sim-bridge',),(Artifact('sim_bridge','debug/libsim_bridge.dylib'),),profile='debug',overlay=self.overlay)

    def run(self,*args: str) -> bytes:
        return subprocess.run((str(self.git),'-C',str(self.root),*args),check=True,capture_output=True).stdout

    def prepared(self) -> subject.BuildEvidenceSpec:
        result=subject.prove_transformation(self.transform)
        if document(result.read_text()).field('status').value!='PASS':raise AssertionError(result.read_text())
        built=build_product(self.build)
        if built.status!='PASS':raise AssertionError(built.receipt.read_text())
        return self.evidence()

    def evidence(self) -> subject.BuildEvidenceSpec:
        return subject.BuildEvidenceSpec(self.build,self.transform,
            tuple(inventory(self.transform.output,[p.name for p in self.transform.output.iterdir()]).items()),
            tuple(inventory(self.build.output,[p.name for p in self.build.output.iterdir()]).items()))


class TransformationTests(unittest.TestCase):
    def test_actual_mutable_scratch_preserves_pristine_product_and_modes(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=Fixture(Path(temp).resolve());before=(f.run('status','--porcelain'),f.run('ls-files','--stage','-z'))
            receipt=subject.prove_transformation(f.transform)
            node=document(receipt.read_text())
            self.assertEqual(node.field('status').value,'PASS')
            self.assertEqual(node.field('historical_product_application').value,'NOT_VERIFIED')
            self.assertEqual((f.run('status','--porcelain'),f.run('ls-files','--stage','-z')),before)
            self.assertEqual((f.transform.scratch/'tools/v8/observer.py').read_text(),'new\n')
            self.assertTrue((f.transform.scratch/'tools/v8/observer.py').stat().st_mode & 0o111)

    def test_enclosing_repo_and_inherited_git_traps_cannot_change_product(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home=Path(temp).resolve();f=Fixture(home)
            enclosing=home/'enclosing';enclosing.mkdir()
            _=subprocess.run((str(f.git),'init','-q',str(enclosing)),check=True,capture_output=True)
            spec=replace(f.transform,scratch=enclosing/'scratch')
            before=f.run('ls-files','--stage','-z')
            with patch.dict(os.environ,{'GIT_DIR':str(f.root/'.git'),'GIT_WORK_TREE':str(f.root),'GIT_INDEX_FILE':str(f.root/'.git/index'),
                                        'GIT_CONFIG_COUNT':'1','GIT_CONFIG_KEY_0':'apply.whitespace','GIT_CONFIG_VALUE_0':'error'}):
                receipt=subject.prove_transformation(spec)
            self.assertEqual(document(receipt.read_text()).field('status').value,'PASS')
            self.assertEqual(f.run('ls-files','--stage','-z'),before)
            native=document((spec.output/'discovery/process.json').read_text())
            self.assertEqual(native.field('native_exit').value,128)
            self.assertNotIn('GIT_INDEX_FILE',native.field('environment').mapping())

    def test_same_path_unrelated_hunk_fails_actual_transformation(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=Fixture(Path(temp).resolve())
            _=f.patch.write_bytes(f.patch.read_bytes().replace(b'+new\n',b'+unrelated\n'))
            data=document(f.manifest.read_text()).mapping();data['observer_patch_sha256']=file_hash(f.patch)
            _=f.manifest.write_text(json.dumps(data))
            spec=replace(f.transform,overlay=Overlay(f.patch,file_hash(f.patch),f.manifest,file_hash(f.manifest)))
            result=subject.prove_transformation(spec)
            self.assertEqual(document(result.read_text()).field('status').value,'FAIL')
            self.assertIn('postimages',result.read_text())

    def test_scratch_overlap_and_stale_outputs_reject(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=Fixture(Path(temp).resolve())
            with self.assertRaises(ValueError):_=subject.prove_transformation(replace(f.transform,scratch=f.root/'scratch'))
            f.transform.scratch.mkdir()
            with self.assertRaises(ValueError):_=subject.prove_transformation(f.transform)


class BuildTests(unittest.TestCase):
    def test_real_fake_compiler_receipts_join_without_execution_authority(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=Fixture(Path(temp).resolve());spec=f.prepared()
            result=subject.authenticate_build(spec)
            self.assertEqual(result.status,'CONSISTENT_SUCCESS')
            self.assertEqual(result.execution_provenance,'CALLER_DRIVER_EVIDENCE_REQUIRED')
            self.assertEqual(result.runtime_admission,'PENDING');self.assertFalse(result.publication_authority)

    def test_changed_earlier_version_log_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=Fixture(Path(temp).resolve());_=f.prepared()
            _=(f.build.output/'cargo-version/stdout.log').write_text('changed')
            # Even a newly supplied output inventory cannot repair the recorded process hash join.
            with self.assertRaises(ValueError):_=subject.authenticate_build(f.evidence())

    def test_wrong_recipe_or_artifact_is_rejected(self) -> None:
        for fault in ('profile','features','source','artifact','compiler','pid'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory() as temp:
                f=Fixture(Path(temp).resolve());spec=f.prepared()
                if fault=='profile':spec=replace(spec,build=replace(spec.build,profile='release'))
                elif fault=='features':spec=replace(spec,build=replace(spec.build,features=('foreign',)))
                elif fault=='source':spec=replace(spec,build=replace(spec.build,source_sha='f'*40))
                elif fault=='artifact':_=(f.root/'rust/target/debug/libsim_bridge.dylib').write_bytes(b'changed')
                elif fault=='compiler':_=(f.build.output/'cargo-build/stdout.log').write_text('{"reason":"build-finished","success":true}\n')
                else:
                    path=f.build.output/'cargo-build/process.json';data=document(path.read_text()).mapping();data['pid']=0
                    _=path.write_text(json.dumps(data))
                with self.assertRaises(ValueError):_=subject.authenticate_build(spec)

    def test_sparse_failed_build_retains_primary_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=Fixture(Path(temp).resolve());f.build.output.mkdir()
            _=(f.build.output/'build.json').write_text(json.dumps({'root':str(f.root),'source_sha':f.sha,'profile':'debug',
                'source_kind':'DERIVED_OBSERVER','packages':['sim-bridge'],'features':[],'status':'FAIL','error':'compiler failed before artifact'}))
            spec=subject.BuildEvidenceSpec(f.build,f.transform,(),(('build.json',file_hash(f.build.output/'build.json')),))
            result=subject.authenticate_build(spec)
            self.assertEqual(result.status,'BUILD_FAILED');self.assertEqual(result.errors,('compiler failed before artifact',))
            self.assertIsNone(result.artifact)

    def test_forged_process_completion_is_rejected_even_with_resealed_outer_inventory(self) -> None:
        for fault in ('pid','spawn','argv','compiler'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory() as temp:
                f=Fixture(Path(temp).resolve());_=f.prepared()
                process=f.build.output/'cargo-build/process.json';native=document(process.read_text()).mapping()
                if fault=='pid':native['pid']=0
                elif fault=='spawn':
                    spawn=process.parent/'process-spawn.json';data=document(spawn.read_text()).mapping();data['pid']=999
                    _=spawn.write_text(json.dumps(data))
                elif fault=='argv':native['argv']=['foreign']
                else:
                    log=process.parent/'stdout.log';_=log.write_text(log.read_text().replace('sim_bridge','foreign'))
                    native['stdout_sha256']=file_hash(log)
                _=process.write_text(json.dumps(native))
                receipt=f.build.output/'build.json';data=document(receipt.read_text()).mapping();data['process_receipt_sha256']=file_hash(process)
                _=receipt.write_text(json.dumps(data))
                side=f.build.output/'artifact-sim_bridge.json';data=document(side.read_text()).mapping();data['build_receipt_sha256']=file_hash(receipt)
                _=side.write_text(json.dumps(data))
                with self.assertRaises(ValueError):_=subject.authenticate_build(f.evidence())

    def test_patch_or_input_drift_after_build_is_rejected(self) -> None:
        for fault in ('patch','lock','descriptor','extra_evidence'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory() as temp:
                f=Fixture(Path(temp).resolve());spec=f.prepared()
                path={'patch':f.patch,'lock':f.root/'rust/Cargo.lock','descriptor':f.root/'sim_bridge.gdextension',
                      'extra_evidence':f.build.output/'extra'}[fault]
                _=path.write_text('changed')
                with self.assertRaises(ValueError):_=subject.authenticate_build(spec)

    def test_mutation_during_final_build_validation_rejects(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=Fixture(Path(temp).resolve());spec=f.prepared();original=recheck_derived_source
            def changed(binding: DerivedSourceBinding) -> DerivedSourceBinding:
                result=original(binding)
                _=(f.build.output/'cargo-version/stdout.log').write_text('late mutation')
                return result
            with patch.object(subject,'recheck_derived_source',side_effect=changed),self.assertRaises(ValueError):
                _=subject.authenticate_build(spec)


class FailureTests(unittest.TestCase):
    def test_discovery_integrity_error_is_retained_and_rejects_transform(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=Fixture(Path(temp).resolve())
            def inject(spec: ProcessSpec) -> ProcessResult:
                result=execute(spec)
                if spec.output_root.name=='discovery':
                    raw=document(result.receipt.read_text()).mapping()
                    raw['errors']=['native exit 128','postflight: injected integrity error']
                    _=result.receipt.write_text(json.dumps(raw))
                return result
            with patch.object(subject,'execute',side_effect=inject):
                receipt=subject.prove_transformation(f.transform)
            node=document(receipt.read_text())
            self.assertEqual(node.field('status').value,'FAIL')
            self.assertIn('/.errors:',str(node.field('errors').value))
            raw=document((f.transform.output/'discovery/process.json').read_text())
            self.assertEqual(raw.field('errors').value,['native exit 128','postflight: injected integrity error'])
            self.assertEqual(raw.field('native_exit').value,128)
            self.assertFalse((f.transform.output/'check').exists())

    def test_partial_transform_failure_retains_output_and_does_not_edit_product(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=Fixture(Path(temp).resolve());original=execute
            def mutate(spec: ProcessSpec) -> ProcessResult:
                result=original(spec)
                if spec.output_root.name=='check':_=f.patch.write_text('changed')
                return result
            before=f.run('ls-files','--stage','-z')
            with patch.object(subject,'execute',side_effect=mutate):receipt=subject.prove_transformation(f.transform)
            self.assertEqual(document(receipt.read_text()).field('status').value,'FAIL')
            self.assertTrue((f.transform.output/'apply/process.json').is_file())
            self.assertEqual(f.run('ls-files','--stage','-z'),before)

    def test_invalid_preimage_and_path_escape_fail_closed(self) -> None:
        for fault in ('preimage','escape','binary'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory() as temp:
                f=Fixture(Path(temp).resolve());data=document(f.manifest.read_text()).mapping()
                rows=document(f.manifest.read_text()).field('files').array()
                if fault=='preimage':rows[0].mapping()['base0_sha256']='0'*64
                elif fault=='escape':rows[0].mapping()['path']='../escape'
                else:_=f.patch.write_text(f.patch.read_text()+'GIT binary patch\n')
                data['files']=[row.value for row in rows];data['observer_patch_sha256']=file_hash(f.patch)
                _=f.manifest.write_text(json.dumps(data))
                spec=replace(f.transform,overlay=Overlay(f.patch,file_hash(f.patch),f.manifest,file_hash(f.manifest)))
                receipt=subject.prove_transformation(spec)
                self.assertEqual(document(receipt.read_text()).field('status').value,'FAIL')
                self.assertFalse((f.home/'escape').exists())

    def test_actual_fake_compiler_failure_retains_sparse_evidence_without_sidecar(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f=Fixture(Path(temp).resolve());_=subject.prove_transformation(f.transform)
            compiler=f.build.cargo
            _=compiler.write_text(compiler.read_text().replace("p=pathlib.Path(os.environ", "sys.exit(7)\np=pathlib.Path(os.environ"))
            f.build=replace(f.build,cargo_sha256=file_hash(compiler),rustc_sha256=file_hash(compiler))
            result=build_product(f.build)
            self.assertEqual(result.status,'FAIL')
            self.assertFalse((f.build.output/'artifact-sim_bridge.json').exists())
            bound=subject.authenticate_build(f.evidence())
            self.assertEqual(bound.status,'BUILD_FAILED');self.assertTrue(bound.errors)
            self.assertTrue((f.build.output/'cargo-build/stderr.log').is_file())


class ReviewRegressionTests(unittest.TestCase):
    def test_zero_elapsed_and_foreign_library_identity_reject_resealed_evidence(self) -> None:
        for fault in ('duration','package','manifest','kind','crate_types','source'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory() as temp:
                f=Fixture(Path(temp).resolve());_=f.prepared()
                process=f.build.output/'cargo-build/process.json';native=document(process.read_text()).mapping()
                if fault=='duration':native['duration_ns']=0
                else:
                    log=process.parent/'stdout.log';rows=[document(line).mapping() for line in log.read_text().splitlines()]
                    artifact=rows[0]
                    if fault=='package':artifact['package_id']='path+file:///foreign#other@0.0.0'
                    elif fault=='manifest':artifact['manifest_path']='/foreign/Cargo.toml'
                    else:
                        target=document(json.dumps(artifact['target'])).mapping()
                        target[{'kind':'kind','crate_types':'crate_types','source':'src_path'}[fault]]='/foreign/main.rs' if fault=='source' else ['bin']
                        artifact['target']=target
                    _=log.write_text(''.join(json.dumps(row)+'\n' for row in rows));native['stdout_sha256']=file_hash(log)
                _=process.write_text(json.dumps(native))
                receipt=f.build.output/'build.json';data=document(receipt.read_text()).mapping();data['process_receipt_sha256']=file_hash(process)
                _=receipt.write_text(json.dumps(data))
                side=f.build.output/'artifact-sim_bridge.json';data=document(side.read_text()).mapping();data['build_receipt_sha256']=file_hash(receipt)
                _=side.write_text(json.dumps(data))
                with self.assertRaises(ValueError):_=subject.authenticate_build(f.evidence())
