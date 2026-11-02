"""Offline original-body routing; no Cargo/Godot/world execution."""
import os
import subprocess
import sys
import time
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_full_gate as gate
import generic_library_run as library
import generic_m1_run as m1
import test_generic_m1_run as fixtures
from generic_fixture_run import ComponentResult
from generic_gate import GateContractError, document
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json
from test_generic_full_gate import lock_contender, owned_inputs


class M1CoordinatorTests(unittest.TestCase):
    def context(self, root: Path, control: int, fail: int | None = None) -> gate.OwnedOrderedInputs:
        fixture=fixtures.M1RuntimeTests();self.addCleanup(fixture.doCleanups)
        base=root/'fixture';base.mkdir();request=fixture.fixture(base,fail)
        context=owned_inputs(root,control)
        product=replace(context.components.probes.candidate,input_root=request.template.root,input_files=request.template.inputs,
                        executable=request.executables[0].binary,executable_sha256=file_hash(request.executables[0].binary))
        godot=replace(product,executable=request.godot,executable_sha256=file_hash(request.godot))
        bound=gate.M1Context(product.source_sha,replace(request,template=replace(request.template,held_lane_fd=-1)))
        return replace(context,components=replace(context.components,probes=replace(context.components.probes,candidate=product),
                       godot=replace(context.components.godot,candidate=godot)),m1=bound)

    def test_one_whole_body_covers_eight_and_preserves_other_routes(self) -> None:
        options={'play_enabled':True,'prefix28_enabled':True,'prefix36_enabled':True,
                 'library42_enabled':True,'library43_enabled':True,'library49_enabled':True}
        old=gate.ordered_owned_calls(**options)
        rows=gate.ordered_owned_calls(**options,m1_enabled=True)
        self.assertEqual([r for r in rows if r.step!=17],[r for r in old if r.step!=17])
        selected=[r for r in rows if r.step==17]
        self.assertEqual(selected,[gate.m1_call()]);self.assertEqual(len(selected[0].identities),8)
        self.assertLess(rows.index(selected[0]),next(i for i,r in enumerate(rows) if r.step==18))


    def test_actual_body_returns_pending_and_completed_failure_stays_fail(self) -> None:
        for fail in (None,1):
            with self.subTest(fail=fail),TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();read,write=os.pipe()
                try:
                    context=self.context(root,read,fail)
                    with patch.object(gate,'LOCK',root/'lane'),patch.object(m1,'LOCK',root/'lane'):
                        row=gate.invoke_m1(gate.m1_call(),context,root/'out')
                    self.assertEqual(row.status,'PENDING' if fail is None else 'FAIL',row.error)
                    gate.recheck_m1(context,row)
                    (root/'out/late-empty').mkdir()
                    with self.assertRaises(GateContractError):gate.recheck_m1(context,row)
                finally:os.close(read);os.close(write)

    def test_settled_native_failure_continues_step18(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read,1)
                def action(_out: Path) -> ComponentResult:
                    raise AssertionError('mock tail')
                with patch.object(gate,'LOCK',root/'lane'),patch.object(m1,'LOCK',root/'lane'), \
                        patch.object(gate,'ordered_owned_calls',return_value=(gate.m1_call(),gate.OrderedCall(18,'direct',()))), \
                        patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'), \
                        patch.object(gate,'ordered_action',return_value=action), \
                        patch.object(gate,'invoke',return_value=gate.Outcome('direct','INCOMPLETE',None,None,'')), \
                        patch.object(gate,'lane'):
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,m1=context.m1)
                node=document(path.read_text())
                self.assertEqual(node.field('actual_component_sequence').value,['17:m1','18:direct'])
                self.assertEqual(node.field('components').array()[0].field('status').text(),'FAIL')
            finally:os.close(read);os.close(write)

    def test_publication_drift_returns_failure_with_original_anchor(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read)
                def publish(path: Path, data: dict[str,Json]) -> None:
                    atomic_json(path,data)
                    if path.name=='ordered-dispatch.json':(root/'out/step-17/m1/late-empty').mkdir()
                with patch.object(gate,'LOCK',root/'lane'),patch.object(m1,'LOCK',root/'lane'), \
                        patch.object(gate,'ordered_owned_calls',return_value=(gate.m1_call(),)), \
                        patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'), \
                        patch.object(gate,'atomic_json',side_effect=publish):
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,m1=context.m1)
                node=document(path.read_text())
                self.assertEqual(node.field('status').text(),'FAIL');self.assertTrue(node.field('errors').array())
                self.assertNotEqual(path.name,'ordered-dispatch.json')
                self.assertEqual(node.field('supersedes').field('sha256').text(),file_hash(root/'out/ordered-dispatch.json'))
                self.assertNotIn('late-empty',node.field('components').array()[0].field('m1_state').mapping())
            finally:os.close(read);os.close(write)

    def test_publication_source_drift_returns_failure(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read)
                def publish(path: Path, data: dict[str,Json]) -> None:
                    atomic_json(path,data)
                    if path.name=='ordered-dispatch.json':
                        source=root/'fixture/source/tools/v8/test_locale_trf.gd'
                        _=source.write_bytes(source.read_bytes()+b'\n# source drift')
                with patch.object(gate,'LOCK',root/'lane'),patch.object(m1,'LOCK',root/'lane'), \
                        patch.object(gate,'ordered_owned_calls',return_value=(gate.m1_call(),)), \
                        patch.object(gate,'ordered_bindings'),patch.object(gate,'perf_bindings'), \
                        patch.object(gate,'atomic_json',side_effect=publish):
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,m1=context.m1)
                node=document(path.read_text())
                self.assertEqual(node.field('status').text(),'FAIL');self.assertTrue(node.field('errors').array())
                self.assertNotEqual(path.name,'ordered-dispatch.json')
                self.assertEqual(node.field('supersedes').field('sha256').text(),file_hash(root/'out/ordered-dispatch.json'))
                self.assertNotIn('late-empty',node.field('components').array()[0].field('m1_state').mapping())
            finally:os.close(read);os.close(write)

    def test_unknown_retains_actual_live_thread_request_and_lane(self) -> None:
        script=r'''import os,sys,threading
from pathlib import Path
from unittest.mock import patch
import generic_full_gate as g
import generic_m1_run as m
from test_generic_full_gate_m1 import M1CoordinatorTests
root=Path(sys.argv[1]);fd=int(sys.argv[2]);context=M1CoordinatorTests().context(root,fd)
original_hold=g._M1Owner.hold
known=[]
def launch(request):
 live,_=m.prepare(request,m.checked(request));event=threading.Event()
 live.thread=threading.Thread(target=event.wait,daemon=True);live.thread.start();known.append(live)
 raise m.M1OwnershipUnknown(live,RuntimeError('retained source worker'))
def hold(owner,error):
 assert owner.retained_live is known[0]
 assert owner.retained_request is known[0].request
 assert known[0].thread.is_alive()
 (root/'held').write_text(str(owner.descriptor))
 original_hold(owner,error)
with patch.object(g,'LOCK',root/'lane'),patch.object(m,'LOCK',root/'lane'),patch.object(m,'run_owned',side_effect=launch),patch.object(g._M1Owner,'hold',hold):
 g.invoke_m1(g.m1_call(),context,root/'out')
(root/'returned').write_text('bad')
'''
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1','PYTHONPATH':os.pathsep.join((str(Path(__file__).parent),str(Path(fixtures.__file__).parent),str(Path(library.__file__).parent)))}
            with subprocess.Popen([sys.executable,'-c',script,str(root),str(read)],pass_fds=(read,),env=env,
                                  stdout=subprocess.PIPE,stderr=subprocess.PIPE) as child:
                os.close(read)
                try:
                    deadline=time.monotonic()+10
                    while not (root/'held').exists() and child.poll() is None and time.monotonic()<deadline:time.sleep(.01)
                    self.assertTrue((root/'held').exists())
                    self.assertNotEqual(lock_contender(root/'lane'),0)
                    os.close(write);write=-1;time.sleep(.1)
                    self.assertIsNone(child.poll());self.assertFalse((root/'returned').exists())
                    self.assertNotEqual(lock_contender(root/'lane'),0)
                finally:
                    if write>=0:os.close(write)
                    if child.poll() is None:child.kill()
                    _=child.communicate(timeout=5)

if __name__=='__main__':
    _=unittest.main()
