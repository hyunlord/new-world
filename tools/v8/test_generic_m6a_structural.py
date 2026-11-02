"""Synthetic capture/immutability tests; no Godot or simulation is executed."""
import os
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from collections.abc import Callable
from dataclasses import replace
from pathlib import Path
from typing import final
from unittest.mock import patch

import gate_oracle_child as oracle_child
import generic_m6a_structural as structural
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document
from generic_process import ProcessResult, atomic_json, file_hash
from generic_sources import inventory
from m6a_gate_history import run as original_run


def transcript() -> str:
    labels = ['data-only plant maps admitted', 'fixture content installs',
              'duplicate selected maps rejected', 'species ripe phase wins', 'unbound category retains placeholder',
              'malformed plant art rejected: synthetic test']
    labels += [f'{label} {phase}' for phase in range(4) for label in
               ('one actual batch phase', 'native texture phase', 'native anchor phase',
                'upright UV phase', 'unknown species uses category phase')]
    return ('M6A_PLANT_ART_BACKEND display=macOS method=gl_compatibility\n' +
            ''.join('M6A_PLANT_ART_PASS '+label+'\n' for label in labels) +
            f'M6A_PLANT_ART_DONE checks={len(labels)} failures=0\n')


class StructuralTests(unittest.TestCase):
    def test_owned_api_is_separate_from_legacy_capture(self) -> None:
        self.assertTrue(callable(getattr(structural, "run_owned_entrypoint", None)))

    def fixture(self, parent: Path, payload: bytes | None = None, exit_code: int = 0
                ) -> tuple[Products, structural.Tooling]:
        source = Path(original_run.__code__.co_filename).resolve().parent
        product_source = source.parent.parent
        toolroot = parent/'tools'
        toolroot.mkdir()
        for path in source.glob('*.py'):
            _ = shutil.copyfile(path,toolroot/path.name)
        _ = shutil.copyfile(Path(structural.__file__),toolroot/'generic_m6a_structural.py')
        closure = structural.import_closure(toolroot,'generic_m6a_structural.py')
        for path in toolroot.glob('*.py'):
            if path.name not in closure:
                path.unlink()
        python = Path(sys.executable).resolve()
        bash_name = shutil.which('bash')
        assert bash_name is not None
        bash = Path(bash_name).resolve()
        tools = structural.Tooling(python,file_hash(python),bash,file_hash(bash),toolroot,
                                  tuple(sorted(inventory(toolroot,list(closure)).items())),
                                  str(python.parent)+':'+str(bash.parent)+':/usr/bin:/bin')
        binary = parent/'synthetic-renderer'
        contents = transcript().encode() if payload is None else payload
        _ = binary.write_text('#!'+str(python)+'\nimport os,sys\nos.write(1,'+repr(contents)+')\n'
                              + 'os.write(2,b"synthetic stderr\\n")\nsys.exit('+str(exit_code)+')\n')
        binary.chmod(0o755)
        products: list[ProbeProduct] = []
        for arm in ('base','candidate'):
            root = parent/arm
            root.mkdir()
            for name in structural.PINS:
                target = root/name
                target.parent.mkdir(parents=True,exist_ok=True)
                _ = shutil.copyfile(product_source/name,target)
            for name in ('assets','localization','shaders','.godot/imported'):
                (root/name).mkdir(parents=True,exist_ok=True)
            for name in ('sim_bridge.gdextension','rust/target/debug/libsim_bridge.dylib','rust/target/release/libsim_bridge.dylib',
                         'tools/v8/fixtures/m6a_plant_art.json','.godot/global_script_class_cache.cfg',
                         '.godot/extension_list.cfg','.godot/uid_cache.bin'):
                path = root/name
                path.parent.mkdir(parents=True,exist_ok=True)
                if not path.exists():
                    _ = path.write_text('synthetic fixture only')
            _ = shutil.copyfile(product_source/'.godot/global_script_class_cache.cfg',root/'.godot/global_script_class_cache.cfg')
            for name in ('scripts/ui/person_names.gd','scripts/world/art_hash.gd'):
                target = root/name
                target.parent.mkdir(parents=True,exist_ok=True)
                _ = shutil.copyfile(product_source/name,target)
            _ = (root/'.godot/extension_list.cfg').write_text('res://sim_bridge.gdextension\n')
            _ = (root/'assets/texture.png').write_bytes(b'synthetic source')
            _ = (root/'.godot/imported/texture.ctex').write_bytes(b'synthetic imported texture')
            _ = (root/'assets/texture.png.import').write_text(
                '[remap]\nimporter="texture"\ntype="CompressedTexture2D"\npath="res://.godot/imported/texture.ctex"\n[deps]\n'
                + 'source_file="res://assets/texture.png"\ndest_files=["res://.godot/imported/texture.ctex"]\n')
            uid_path = b'res://assets/texture.png'
            _ = (root/'.godot/uid_cache.bin').write_bytes(
                (1).to_bytes(4,'little')+(7).to_bytes(8,'little')+len(uid_path).to_bytes(4,'little')+uid_path)
            files = inventory(root,sorted(structural.required_resources(root)))
            products.append(ProbeProduct('a'*40,binary,file_hash(binary),root,tuple(sorted(files.items()))))
        return Products(products[0],products[1]),tools

    def test_actual_self_contained_capture_preserves_original_recipe_and_merged_stream(self) -> None:
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ,{},clear=True):
            root = Path(directory).resolve()
            products,tools = self.fixture(root)
            plan = structural.prepare(products.base,tools,root/'capture')
            self.assertIsNone(plan.spec.timeout_seconds)
            self.assertEqual(document(plan.plan_path.read_text()).field('command').value,
                             ['bash','tools/v8/run_bounded.sh','180',str(products.base.executable),
                              '--rendering-driver','opengl3','--path','.',
                              '--script','res://tools/v8/m6a_plant_art_test.gd'])
            result = structural.capture_once(plan)
            self.assertEqual(structural.inspect_output(plan,result),26)
            self.assertEqual((plan.output/'plant-art-structural.log').read_bytes(),
                             transcript().encode()+b'synthetic stderr\n')
            self.assertIsNone(document((plan.output/'capture-completion.json').read_text()).field('godot_pid').value)

    def test_pair_pass_is_only_structural(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            products,tools = self.fixture(root)
            result = structural.run_pair(products,tools,root/'pair')
            self.assertEqual(result.status,'PASS')
            receipt = document(result.receipt.read_text())
            self.assertEqual(receipt.field('visual_status').value,'NOT_EVALUATED')
            self.assertEqual(receipt.field('worlds').value,'NO_SIMULATION_WORLD')
            self.assertIs(receipt.field('publication_authority').value,False)
            self.assertIs(receipt.field('lane_release_claim').value,False)

    def test_bad_utf8_and_nonzero_are_retained_failures(self) -> None:
        for payload,code in ((b'\xff',0),(transcript().encode(),7)):
            with self.subTest(code=code), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                products,tools = self.fixture(root,payload,code)
                result = structural.run_pair(products,tools,root/'pair')
                self.assertEqual(result.status,'FAIL')
                self.assertEqual((root/'pair/base/plant-art-structural.log').read_bytes(),payload+b'synthetic stderr\n')
                self.assertTrue((root/'pair/candidate/process.json').is_file())

    def test_backend_and_original_oracle_fail_closed(self) -> None:
        texts = [transcript().replace('display=macOS','display=headless'),
                 transcript()+ 'M6A_PLANT_ART_BACKEND display=macOS method=gl_compatibility\n',
                 transcript().replace('native anchor phase 2','missing anchor phase 2'),
                 transcript()+'ERROR: broken\n',transcript()+'Vulkan API\n']
        for raw in texts:
            with self.subTest(raw=raw[-50:]), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                products,tools = self.fixture(root,raw.encode())
                plan = structural.prepare(products.base,tools,root/'capture')
                result = structural.capture_once(plan)
                with self.assertRaises(ValueError):
                    _ = structural.inspect_output(plan,result)

    def test_preflight_missing_pin_and_overlapping_roots(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            products,tools = self.fixture(root)
            bad = replace(products.base,input_files=tuple((n,h) for n,h in products.base.input_files if n!='project.godot'))
            with self.assertRaises(GateContractError):
                _ = structural.prepare(bad,tools,root/'missing')
            with self.assertRaises(GateContractError):
                _ = structural.prepare(products.base,tools,products.base.input_root/'capture')
            bad_tools = replace(tools,files=tuple((n,h) for n,h in tools.files if n!='m6a_gate_history.py'))
            with self.assertRaises(GateContractError):
                _ = structural.prepare(products.base,bad_tools,root/'missing-tool')

    def test_later_arm_cannot_mutate_earlier_receipt_or_executable(self) -> None:
        for target in ('output','binary','input'):
            with self.subTest(target=target), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                products,tools = self.fixture(root)
                original = structural.capture_once
                def mutate(plan: structural.StructuralPlan, original: Callable[[structural.StructuralPlan], ProcessResult] = original,
                           root: Path = root, products: Products = products, target: str = target) -> ProcessResult:
                    result = original(plan)
                    if plan.output.name == 'candidate':
                        path = {'output':root/'pair/base/plant-art-structural.log',
                                'binary':products.base.executable,
                                'input':products.base.input_root/'project.godot'}[target]
                        with path.open('ab') as stream:
                            _ = stream.write(b'altered')
                    return result
                with patch.object(structural,'capture_once',side_effect=mutate):
                    result = structural.run_pair(products,tools,root/'pair')
                self.assertEqual(result.status,'FAIL')

    def test_completed_flag_cannot_hide_foreign_adapter_or_unexpected_environment(self) -> None:
        for mutation in ('adapter_pid','environment'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                products,tools = self.fixture(root)
                plan = structural.prepare(products.base,tools,root/'capture')
                result = structural.capture_once(plan)
                name = 'capture-completion.json' if mutation == 'adapter_pid' else 'capture-start.json'
                path = plan.output/name
                row = dict(document(path.read_text()).mapping())
                row[mutation] = -1 if mutation == 'adapter_pid' else {'CORE_P7_ENABLED':'1'}
                path.unlink()
                atomic_json(path,row)
                with self.assertRaises(ValueError):
                    _ = structural.inspect_output(plan,result)

    def test_preflight_failure_retains_not_run_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            products,tools = self.fixture(root)
            bad = replace(products.base,input_files=())
            with patch.object(structural,'capture_once') as capture:
                result = structural.run_pair(Products(bad,products.candidate),tools,root/'pair')
            capture.assert_not_called()
            self.assertEqual(result.status,'FAIL')
            self.assertEqual(document(result.receipt.read_text()).field('phases').value,
                             {'base':'NOT_RUN','candidate':'NOT_RUN'})

    def test_external_extension_class_uid_and_remap_rejected_even_when_rehashed(self) -> None:
        mutations = {
            '.godot/extension_list.cfg': b'res://sim_bridge.gdextension\n/tmp/unbound.gdextension\n',
            '.godot/global_script_class_cache.cfg': b'list=[{"path":"/tmp/foreign.gd"}]',
            'assets/texture.png.import': b'[remap]\nimporter="texture"\ntype="CompressedTexture2D"\npath="/tmp/foreign.ctex"\n',
            '.godot/uid_cache.bin': (1).to_bytes(4,'little')+(7).to_bytes(8,'little')
                + (12).to_bytes(4,'little')+b'/tmp/foreign',
        }
        for name,payload in mutations.items():
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                products,tools = self.fixture(root)
                _ = (products.base.input_root/name).write_bytes(payload)
                new_inputs = tuple((n,file_hash(products.base.input_root/n)) for n,_ in products.base.input_files)
                changed = replace(products.base,input_files=new_inputs)
                with self.assertRaises(ValueError):
                    _ = structural.prepare(changed,tools,root/'capture')

    def test_unbound_local_remap_and_truncated_uid_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            products,tools = self.fixture(root)
            remap = products.base.input_root/'assets/texture.png.import'
            _ = remap.write_text('[remap]\nimporter="texture"\ntype="CompressedTexture2D"\npath="res://.godot/imported/unbound.ctex"\n')
            _ = (products.base.input_root/'.godot/imported/unbound.ctex').write_bytes(b'exists but unbound')
            changed = replace(products.base,input_files=tuple((n,file_hash(products.base.input_root/n))
                                                              for n,_ in products.base.input_files))
            with self.assertRaises(ValueError):
                _ = structural.prepare(changed,tools,root/'capture')
        for raw in (b'',(1).to_bytes(4,'little'),(0).to_bytes(4,'little')+b'extra'):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                _ = structural.uid_paths(raw)

    def test_ambient_shadow_oracle_is_not_the_sealed_oracle(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            source = Path(original_run.__code__.co_filename).resolve().parent
            # Separate interpreter ensures a shadow is imported before the adapter.
            script = r"""
import ast,importlib.util,sys
from pathlib import Path
source,draft,home=map(Path,sys.argv[1:])
sys.path[:0]=[str(draft),str(source)]
original=(source/'m6a_gate_art.py').read_text()
node=next(n for n in ast.parse(original).body if isinstance(n,ast.FunctionDef) and n.name=='validate')
lines=original.splitlines(keepends=True)
lines[node.lineno-1:node.end_lineno]=['def validate(text: str) -> int:\n    return 26\n']
shadow=home/'m6a_gate_art.py';shadow.write_text(''.join(lines))
spec=importlib.util.spec_from_file_location('m6a_gate_art',shadow)
module=importlib.util.module_from_spec(spec);sys.modules['m6a_gate_art']=module;spec.loader.exec_module(module)
import gate_oracle_child as oracle_child
import generic_m6a_structural as structural
from test_generic_m6a_structural import StructuralTests,transcript
products,tools=StructuralTests().fixture(home,transcript().replace('native anchor phase 2','missing anchor phase 2').encode())
result=structural.run_pair(products,tools,home/'pair')
assert result.status=='FAIL',result.status
assert not (home/'pair/base/process.json').exists()
print(result.status)
"""
            result = subprocess.run([sys.executable,'-c',script,str(source),str(Path(structural.__file__).parent),str(root)],
                                    capture_output=True,text=True,check=False)
            self.assertEqual(result.returncode,0,result.stderr)
            self.assertEqual(result.stdout.strip(),'FAIL')

    def test_nested_interruption_records_unknown_without_freeze_or_next_arm(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            products,tools = self.fixture(root)
            # This tiny child is explicitly test-owned. Adapter interruption must not
            # claim it was reaped; the test itself reaps it in finally.
            children: list[subprocess.Popen[bytes]] = []
            def interrupted_wrapper(_root: Path, _command: list[str | Path], _log: Path) -> None:
                children.append(subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']))
                raise KeyboardInterrupt
            try:
                def interrupted(plan: structural.StructuralPlan) -> ProcessResult:
                    plan.output.mkdir()
                    with patch.object(structural,'original_run',side_effect=interrupted_wrapper), \
                         patch.object(Path,'cwd',return_value=plan.product.input_root):
                        self.assertEqual(structural.capture_child(plan.plan_path,plan.plan_sha256),1)
                    atomic_json(plan.output/'process.json',{'pid':os.getpid(),'native_exit':1,
                                'argv':list(plan.spec.argv),'cwd':str(plan.spec.cwd)})
                    return ProcessResult(plan.output/'process.json','FAIL',1,os.getpid())
                with patch.object(structural,'capture_once',side_effect=interrupted) as capture:
                    result = structural.run_pair(products,tools,root/'pair')
                self.assertEqual(capture.call_count,1)
                self.assertEqual(result.status,'INCOMPLETE')
                self.assertEqual(len(children),1)
                self.assertIsNone(children[0].poll())
                receipt = document(result.receipt.read_text())
                self.assertIs(receipt.field('terminal_inventory_complete').value,False)
                self.assertEqual(receipt.field('artifacts').value,{})
                self.assertEqual(receipt.field('phases').field('candidate').value,'NOT_RUN')
            finally:
                for child in children:
                    child.terminate()
                    _ = child.wait(timeout=5)


def owned_fixture(root: Path, code: int=0, timeout: bool=False) -> tuple[Products,structural.Tooling,Path]:
    """Synthetic lock-path overlay; lifecycle tests separately mock source admission."""
    products,tools=StructuralTests().fixture(root,exit_code=code)
    lock=root/'owned.lock';lock.touch()
    if timeout:
        binary=products.base.executable
        _=binary.write_text(binary.read_text().replace('sys.exit(', 'import time; time.sleep(30); sys.exit('))
    prepared: list[ProbeProduct]=[]
    for product in (products.base,products.candidate):
        names=set(dict(product.input_files))
        for name in structural.import_closure(tools.root,'gate_oracle_child.py'):
            target=product.input_root/'tools/v8'/name
            value=(tools.root/name).read_text()
            if name=='mux2_gate_lock.py':value=value.replace('/Users/rexxa/work/mux2-data/.mux2-gpu.lock',str(lock))
            if name=='gate_oracle_child.py' and timeout:value=value.replace('supervise(argv,limit,started,control)','supervise(argv,0,started,control)')
            _=target.write_text(value);names.add('tools/v8/'+name)
        prepared.append(replace(product,executable_sha256=file_hash(product.executable),input_files=tuple(sorted(inventory(product.input_root,sorted(names)).items()))))
    return Products(prepared[0],prepared[1]),tools,lock


def contender(lock: Path) -> int:
    result=subprocess.run([sys.executable,'-c',
        'import fcntl,sys; f=open(sys.argv[1],"a"); fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)',str(lock)],
        capture_output=True,check=False)
    return result.returncode


@final
class OwnedTests(unittest.TestCase):
    def test_loaded_helper_and_product_copy_must_both_match_sealed_tools(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder).resolve();products,tools,_=owned_fixture(root)
            original=tools.root/'mux2_gate_lock.py'
            target=products.base.input_root/'tools/v8/mux2_gate_lock.py'
            _=target.write_bytes(original.read_bytes())
            product=replace(products.base,input_files=tuple((name,file_hash(products.base.input_root/name)) for name,_ in products.base.input_files))
            structural.check_owned_helpers(product,tools)
            _=target.write_text(target.read_text()+'\n')
            changed=replace(product,input_files=tuple((name,file_hash(product.input_root/name)) for name,_ in product.input_files))
            with self.assertRaises(ValueError):structural.check_owned_helpers(changed,tools)

    def test_real_lane_lifetime_normal_and_nonzero(self) -> None:
        for code,timeout in ((0,False),(7,False),(0,True)):
            with self.subTest(code=code,timeout=timeout),tempfile.TemporaryDirectory() as folder:
                root=Path(folder).resolve();products,tools,lock=owned_fixture(root,code,timeout)
                with patch.object(structural,'LOCK',lock),patch.object(oracle_child,'LOCK',lock),patch.object(structural,'check_owned_helpers'):
                    owner=structural.StructuralLaneOwner.acquire(root/'owned',structural.OwnedAdmission(products,tools))
                    self.assertNotEqual(contender(lock),0)
                    result=owner.run(products,tools)
                    self.assertIsInstance(result,structural.SettledStructural)
                    assert isinstance(result,structural.SettledStructural)
                    self.assertNotEqual(contender(lock),0)
                    with self.assertRaises(ValueError):_=owner.finish(structural.SettlementEvidence(owner.attempt,False))
                    self.assertNotEqual(contender(lock),0)
                    final=owner.finish(result)
                    self.assertEqual(final.status,'PASS' if code==0 and not timeout else 'FAIL')
                    if timeout:self.assertEqual(owner.results['base'].wrapper_exit,124)
                    self.assertEqual(contender(lock),0)
                    self.assertEqual(owner.state,'CLOSED')
                    self.assertTrue(owner.results['base'].native_pid)
                    with self.assertRaises(ValueError):_=owner.finish(result)

    def test_no_launch_preflight_failure_closes_only_via_finish(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder).resolve();products,tools,lock=owned_fixture(root)
            bad=Products(replace(products.base,input_files=()),products.candidate)
            with patch.object(structural,'LOCK',lock),patch.object(oracle_child,'capture') as capture:
                owner=structural.StructuralLaneOwner.acquire(root/'owned',structural.OwnedAdmission(bad,tools))
                result=owner.run(bad,tools)
                capture.assert_not_called();self.assertFalse(owner.dispatched)
                self.assertIsInstance(result,structural.SettledStructural)
                assert isinstance(result,structural.SettledStructural)
                self.assertNotEqual(contender(lock),0)
                self.assertEqual(owner.finish(result).status,'FAIL')
                self.assertEqual(contender(lock),0)

    def test_original_completed_receipts_recover_after_reader_fault(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder).resolve();products,tools,lock=owned_fixture(root)
            original=oracle_child.inspect
            reads=0
            def once(path: Path,digest: str,pid: int | None,code: int | None) -> oracle_child.OwnedResult:
                nonlocal reads
                reads+=1
                if reads==1:return oracle_child.OwnedResult('UNKNOWN',pid,None,code,None,'one-time reader fault')
                if reads in (2,3):raise KeyboardInterrupt
                return original(path,digest,pid,code)
            with patch.object(structural,'LOCK',lock),patch.object(oracle_child,'LOCK',lock),patch.object(structural,'check_owned_helpers'),patch.object(oracle_child,'inspect',side_effect=once):
                owner=structural.StructuralLaneOwner.acquire(root/'owned',structural.OwnedAdmission(products,tools))
                pending=owner.run(products,tools)
                self.assertIsInstance(pending,structural.RecoveryRequired)
                assert isinstance(pending,structural.RecoveryRequired)
                self.assertEqual(set(owner.results),{'base'})
                self.assertNotEqual(contender(lock),0)
                with self.assertRaises(ValueError):_=owner.finish(structural.SettlementEvidence(owner.attempt,True))
                forged=structural.RecoveryRequired(pending.attempt,pending.arm,pending.paths)
                with self.assertRaises(ValueError):_=owner.inspect_recovery(forged)
                for _ in range(2):
                    self.assertIsInstance(owner.inspect_recovery(pending),structural.StillUnknown)
                    self.assertNotEqual(contender(lock),0)
                recovered=owner.inspect_recovery(pending)
                self.assertIsInstance(recovered,structural.SettlementEvidence)
                assert isinstance(recovered,structural.SettlementEvidence)
                self.assertEqual(owner.finish(recovered).status,'FAIL')
                self.assertEqual(contender(lock),0)
                self.assertEqual(set(owner.results),{'base'})

    def test_post_settlement_drift_is_failed_but_safe_to_close(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder).resolve();products,tools,lock=owned_fixture(root)
            with patch.object(structural,'LOCK',lock),patch.object(oracle_child,'LOCK',lock),patch.object(structural,'check_owned_helpers'):
                owner=structural.StructuralLaneOwner.acquire(root/'owned',structural.OwnedAdmission(products,tools))
                result=owner.run(products,tools)
                assert isinstance(result,structural.SettledStructural)
                _=(owner.plans['base'].output/'plant-art-structural.log').write_text('late mutation')
                self.assertEqual(owner.finish(result).status,'FAIL')
                self.assertEqual(contender(lock),0)

    def test_live_driver_inspection_and_controller_eof(self) -> None:
        for disconnect,missing,live,boundary in ((False,False,False,''),(True,False,False,''),(False,True,False,''),(True,False,True,''),(True,False,True,'capture'),(True,False,True,'inspect_recovery'),(True,False,True,'record_status'),(True,False,True,'evil_capture'),(True,False,True,'evil_inspect'),(True,False,True,'evil_status')):
            with self.subTest(disconnect=disconnect,missing=missing,live=live,boundary=boundary),tempfile.TemporaryDirectory() as folder:
                root=Path(folder).resolve();read,write=os.pipe()
                script=r"""
import os,sys
from pathlib import Path
from unittest.mock import patch
sys.path[:0]=sys.argv[5:]
import generic_m6a_structural as s
from test_generic_m6a_structural import owned_fixture
root=Path(sys.argv[1]);products,tools,lock=owned_fixture(root)
missing=sys.argv[3]=='missing'
live=sys.argv[3]=='live'
boundary=sys.argv[4]
children=[]
class EvilError(Exception):
 def __str__(self):raise RuntimeError('exception rendering failed')
def capture(path,digest,fd):
 import subprocess
 child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)'])
 children.append(child)
 (root/'test-owned-child.pid').write_text(str(child.pid))
 if boundary=='capture':raise RuntimeError('injected after actual test-owned dispatch')
 if boundary=='evil_capture':raise EvilError()
 return s.oracle_child.OwnedResult('UNKNOWN',None,None,None,None,'test-owned direct child; not production recovery')
def reap():
 for child in children:
  if child.poll() is None:child.terminate()
  child.wait()
def cleanup(signum,frame):
 reap()
 os._exit(0)
import atexit
atexit.register(reap)
if live:
 import signal
 signal.signal(signal.SIGTERM,cleanup)
original=s.oracle_child.inspect
once=True
def inspect(path,digest,pid,code):
 global once
 if once:
  once=False
  if missing:
   original(path,digest,pid,code)
   (root/'owned/base/evidence/native-terminal.json').unlink()
  return s.oracle_child.OwnedResult('UNKNOWN',pid,None,code,None,'one-time reader fault')
 return original(path,digest,pid,code)
def broken_inspect(self,token):
 if boundary=='evil_inspect':raise EvilError()
 raise TypeError('injected recovery inspection')
def broken_status(self):
 if boundary=='evil_status':raise EvilError()
 raise RuntimeError('injected status publication')
if boundary in ('inspect_recovery','evil_inspect'):s.StructuralLaneOwner.inspect_recovery=broken_inspect
if boundary in ('record_status','evil_status'):s.StructuralLaneOwner.record_status=broken_status
with patch.object(s,'LOCK',lock),patch.object(s.oracle_child,'LOCK',lock),patch.object(s,'check_owned_helpers'),patch.object(s.oracle_child,'inspect',side_effect=inspect),patch.object(s.oracle_child,'capture',side_effect=capture if live else s.oracle_child.capture):
 result=s.run_owned_entrypoint(products,tools,root/'owned',int(sys.argv[2]))
 print(result.status,flush=True)
"""
                with subprocess.Popen([sys.executable,'-c',script,str(root),str(read),'live' if live else ('missing' if missing else 'reader'),boundary,str(Path(__file__).resolve().parent),str(Path(structural.__file__).resolve().parent),str(Path(oracle_child.__file__).resolve().parent)],pass_fds=(read,),env={key:value for key,value in os.environ.items() if key!='PYTHONPATH'},stdout=subprocess.PIPE,stderr=subprocess.PIPE) as driver:
                    os.close(read)
                    try:
                        deadline=time.monotonic()+10
                        marker=root/'test-owned-child.pid' if boundary else root/'owned/ownership-event-1.json'
                        while not marker.exists() and driver.poll() is None and time.monotonic()<deadline:time.sleep(0.01)
                        if not marker.exists():
                            if driver.poll() is None:driver.terminate()
                            stdout,stderr=driver.communicate(timeout=5)
                            self.fail(f'child startup failed ({driver.returncode}): stdout={stdout.decode(errors="replace")} stderr={stderr.decode(errors="replace")}')
                        self.assertIsNone(driver.poll());self.assertNotEqual(contender(root/'owned.lock'),0)
                        if live:os.kill(int((root/'test-owned-child.pid').read_text()),0)
                        if boundary in ('inspect_recovery','evil_inspect'):
                            _=os.write(write,b'inspect\n');time.sleep(0.1)
                            self.assertIsNone(driver.poll())
                        if disconnect or missing:
                            if missing:_=os.write(write,b'inspect\n')
                            else:os.close(write);write=-1
                            time.sleep(0.1)
                            self.assertIsNone(driver.poll());self.assertNotEqual(contender(root/'owned.lock'),0)
                            # The live-fixture handler reaps its own actual Popen before exiting;
                            # other modes already completed. This is not production recovery.
                            driver.terminate();_=driver.wait(timeout=5)
                        else:
                            _=os.write(write,b'status\ninspect\n')
                            stdout,stderr=driver.communicate(timeout=10)
                            self.assertEqual(driver.returncode,0,stderr.decode())
                            self.assertEqual(stdout.strip(),b'FAIL');self.assertEqual(contender(root/'owned.lock'),0)
                    finally:
                        if write>=0:os.close(write)
                        if driver.poll() is None:driver.terminate();_=driver.wait(timeout=5)


if __name__ == '__main__':
    _ = unittest.main()
