"""Offline step37 ownership/routing; original canonical lock is not acquired."""
import unittest

import generic_full_gate as gate


class Mux2CoordinatorTests(unittest.TestCase):
    def test_whole_original_body_replaces_partial37_at_exact_position(self) -> None:
        options={'library42_enabled':True,'library43_enabled':True,'oracles_enabled':True}
        old=gate.ordered_owned_calls(**options)
        rows=gate.ordered_owned_calls(**options,mux2_enabled=True)
        self.assertEqual([r for r in rows if r.step!=37],[r for r in old if r.step!=37])
        selected=[r for r in rows if r.step==37]
        self.assertEqual(selected,[gate.mux2_call()])
        self.assertEqual(len(selected[0].identities),11)
        i=rows.index(selected[0])
        self.assertEqual(rows[i-1].step,43)
        self.assertEqual(rows[i+1].step,38)

import ast
import fcntl
import os
import signal
import subprocess
import sys
import time
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_mux2_run as mux
import test_generic_mux2_run as fixtures
from generic_gate import GateContractError, document
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json
from test_generic_full_gate import lock_contender, owned_inputs


class RuntimeCoordinatorTests(unittest.TestCase):
    def fixture_lock(self, context: gate.OwnedOrderedInputs) -> Path:
        assert context.mux2 is not None
        path=context.mux2.request.template.root/'tools/v8/mux2_gate_lock.py'
        for node in ast.walk(ast.parse(path.read_text())):
            if isinstance(node,ast.AnnAssign) and isinstance(node.target,ast.Name) and node.target.id=='LOCK':
                assert isinstance(node.value,ast.Call) and len(node.value.args)==1
                value=node.value.args[0]
                assert isinstance(value,ast.Constant) and isinstance(value.value,str)
                return Path(value.value)
        raise AssertionError('missing fixture lock literal')

    def context(self, root: Path, control: int, fd: int, fail: int | None = None) -> gate.OwnedOrderedInputs:
        base=root/'fixture';base.mkdir();request=fixtures.fixture(base,fd,fail=fail)
        context=owned_inputs(root,control)
        product=replace(context.components.probes.candidate,input_root=request.template.root,input_files=request.template.inputs,
                        executable=request.executables[0].binary,executable_sha256=file_hash(request.executables[0].binary))
        bound=gate.Mux2Context(product.source_sha,replace(request,template=replace(request.template,held_lane_fd=-1)))
        return replace(context,components=replace(context.components,probes=replace(context.components.probes,candidate=product)),mux2=bound)

    def test_actual_isolated_body_pending_and_failure_remains_failed(self) -> None:
        for fail in (None,1):
            with self.subTest(fail=fail),fixtures.IsolatedMachineryTests().lane() as fd,TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();read,write=os.pipe()
                try:
                    context=self.context(root,read,fd,fail)
                    fcntl.flock(fd,fcntl.LOCK_UN)
                    with patch.object(gate,'LOCK',self.fixture_lock(context)):row=gate.invoke_mux2(gate.mux2_call(),context,root/'out')
                    self.assertEqual(row.status,'PENDING' if fail is None else 'FAIL',row.error)
                    gate.recheck_mux2(context,row)
                    (root/'out/late-empty').mkdir()
                    with self.assertRaises(GateContractError):gate.recheck_mux2(context,row)
                finally:os.close(read);os.close(write)

    def test_settled_failure_continues38(self) -> None:
        with fixtures.IsolatedMachineryTests().lane() as fd,TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read,fd,1);fcntl.flock(fd,fcntl.LOCK_UN)
                tail=gate.OrderedCall(38,'oracles',())
                with patch.object(gate,'LOCK',self.fixture_lock(context)),patch.object(gate,'ordered_owned_calls',return_value=(gate.mux2_call(),tail)), \
                     patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'), \
                     patch.object(gate,'invoke_oracles',return_value=gate.Outcome('oracles','INCOMPLETE',None,None,'')):
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,mux2=context.mux2)
                node=document(path.read_text())
                self.assertEqual(node.field('actual_component_sequence').value,['37:mux2','38:oracles'])
                self.assertEqual(node.field('components').array()[0].field('status').text(),'FAIL')
            finally:os.close(read);os.close(write)

    def test_first_auth_cannot_adopt_empty_directory(self) -> None:
        with fixtures.IsolatedMachineryTests().lane() as fd,TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read,fd,1);fcntl.flock(fd,fcntl.LOCK_UN)
                original=gate.authenticate_mux2
                def authenticate(bound: gate.Mux2Context, output: Path, result: mux.DefenceResult) -> str:
                    value=original(bound,output,result);(output/'injected').mkdir();return value
                with patch.object(gate,'LOCK',self.fixture_lock(context)),patch.object(gate,'authenticate_mux2',side_effect=authenticate):
                    row=gate.invoke_mux2(gate.mux2_call(),context,root/'out')
                self.assertEqual(row.status,'FAIL');self.assertIn('first authentication',row.error)
                self.assertNotIn('injected',dict(row.mux2_state))
            finally:os.close(read);os.close(write)

    def test_final_publication_rechecks_members_and_sources(self) -> None:
        for kind in ('member','source'):
            with self.subTest(kind=kind),fixtures.IsolatedMachineryTests().lane() as fd,TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();read,write=os.pipe()
                try:
                    context=self.context(root,read,fd,1);fcntl.flock(fd,fcntl.LOCK_UN)
                    def publish(path: Path, data: dict[str,Json], kind: str = kind, root: Path = root) -> None:
                        atomic_json(path,data)
                        if path.name=='ordered-dispatch.json':
                            if kind=='member':(root/'out/step-37/mux2/injected').mkdir()
                            else:
                                source=root/'fixture/source/tools/v8/mux2_gate_checks.sh'
                                _=source.write_bytes(source.read_bytes()+b'\n# changed\n')
                    with patch.object(gate,'LOCK',self.fixture_lock(context)),patch.object(gate,'ordered_owned_calls',return_value=(gate.mux2_call(),)), \
                         patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'),patch.object(gate,'atomic_json',side_effect=publish):
                        path=gate.run_ordered_owned(context,context.strict,root/'out',read,mux2=context.mux2)
                    node=document(path.read_text());self.assertEqual(node.field('status').text(),'FAIL')
                    self.assertNotEqual(path.name,'ordered-dispatch.json');self.assertTrue(node.field('errors').array())
                    self.assertNotIn('injected',node.field('components').array()[0].field('mux2_state').mapping())
                finally:os.close(read);os.close(write)

    def test_invalid_template_never_launches_and_releases_lane(self) -> None:
        with fixtures.IsolatedMachineryTests().lane() as fd,TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read,fd);assert context.mux2 is not None
                invalid=replace(context.mux2,request=replace(context.mux2.request,
                    template=replace(context.mux2.request.template,arm='base')))
                lock=self.fixture_lock(context);fcntl.flock(fd,fcntl.LOCK_UN)
                with patch.object(gate,'LOCK',lock),patch.object(mux,'run_defence_owned') as run:
                    with self.assertRaises(GateContractError):
                        _=gate.invoke_mux2(gate.mux2_call(),replace(context,mux2=invalid),root/'out')
                    run.assert_not_called()
                self.assertEqual(lock_contender(lock),0)
            finally:os.close(read);os.close(write)

    def test_failure_publication_cannot_mutate_rejected_evidence_again(self) -> None:
        with fixtures.IsolatedMachineryTests().lane() as fd,TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read,fd,1);fcntl.flock(fd,fcntl.LOCK_UN)
                def publish(path: Path, data: dict[str,Json]) -> None:
                    atomic_json(path,data)
                    if path.name in ('ordered-dispatch.json','ordered-dispatch-prefix-failure.json'):
                        (root/'out/step-37/mux2'/path.stem).mkdir()
                with patch.object(gate,'LOCK',self.fixture_lock(context)),patch.object(gate,'ordered_owned_calls',return_value=(gate.mux2_call(),)), \
                     patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'),patch.object(gate,'atomic_json',side_effect=publish), \
                     self.assertRaises(GateContractError) as caught:
                    _=gate.run_ordered_owned(context,context.strict,root/'out',read,mux2=context.mux2)
                self.assertIn('failure publication',str(caught.exception))
            finally:os.close(read);os.close(write)

    def test_unknown_retains_actual_popen_request_and_lane_after_eof(self) -> None:
        script=r'''import fcntl,os,signal,subprocess,sys
from pathlib import Path
from unittest.mock import patch
import generic_full_gate as g
import generic_mux2_run as m
import test_generic_mux2_run as fixtures
from test_generic_full_gate_mux2 import RuntimeCoordinatorTests
root=Path(sys.argv[1]);control=int(sys.argv[2]);known=[]
with fixtures.IsolatedMachineryTests().lane() as fd:
 context=RuntimeCoordinatorTests().context(root,control,fd);fcntl.flock(fd,fcntl.LOCK_UN)
 original=g._Mux2Owner.hold
 def launch(request):
  live=m.prepare(request,m.checked(request));live.launch_intent=True
  live.shell=subprocess.Popen([sys.executable,'-c','import time;time.sleep(60)'])
  known.append(live);(root/'child-pid').write_text(str(live.shell.pid))
  def cleanup(_signal,_frame):
   live.shell.terminate();code=live.shell.wait(timeout=5)
   (root/'cleanup').write_text(str(code));os._exit(0)
  signal.signal(signal.SIGTERM,cleanup)
  raise m.DefenceOwnershipUnknown(live,RuntimeError('actual live child'))
 def hold(owner,error):
  assert owner.retained_live is known[0]
  assert owner.retained_request is known[0].request
  assert known[0].shell.poll() is None
  assert os.fstat(owner.descriptor).st_ino==os.fstat(known[0].request.template.held_lane_fd).st_ino
  (root/'held').write_text(str(m.LOCK))
  try:original(owner,error)
  finally:
   known[0].shell.terminate();known[0].shell.wait(timeout=5)
 with patch.object(g,'LOCK',m.LOCK),patch.object(m,'run_defence_owned',side_effect=launch),patch.object(g._Mux2Owner,'hold',hold):
  g.invoke_mux2(g.mux2_call(),context,root/'out')
'''
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1','PYTHONPATH':os.pathsep.join((str(Path(__file__).parent),str(Path(fixtures.__file__).parent)))}
            with subprocess.Popen([sys.executable,'-c',script,str(root),str(read)],pass_fds=(read,),env=env,
                                  stdout=subprocess.PIPE,stderr=subprocess.PIPE) as child:
                os.close(read)
                try:
                    deadline=time.monotonic()+10
                    while not (root/'held').exists() and child.poll() is None and time.monotonic()<deadline:time.sleep(.01)
                    self.assertTrue((root/'held').exists())
                    lock=Path((root/'held').read_text());self.assertNotEqual(lock_contender(lock),0)
                    os.close(write);write=-1;time.sleep(.1)
                    self.assertIsNone(child.poll());self.assertNotEqual(lock_contender(lock),0)
                finally:
                    if write>=0:os.close(write)
                    # Test-owned signal reaches Python finally; only this child owns its tiny descendant.
                    if child.poll() is None:child.send_signal(signal.SIGTERM)
                    stdout,stderr=child.communicate(timeout=10)
                    self.assertEqual(child.returncode,0,(stdout,stderr))
                    self.assertTrue((root/'cleanup').exists())
