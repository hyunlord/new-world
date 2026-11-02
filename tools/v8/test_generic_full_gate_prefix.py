"""Bounded prefix routing with real Python child fixtures, never Rust/Godot."""
import os
import subprocess
import sys
import time
import unittest
from contextlib import nullcontext
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_full_gate as gate
import generic_integration_prefix_run as prefix
import test_generic_integration_prefix_run as fixtures
from generic_fixture_run import ComponentResult
from generic_gate import GateContractError, document
from generic_library_run import Library49Request, LibrarySettledFailure
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json
from test_generic_full_gate import lock_contender, owned_inputs


class PrefixCoordinatorTests(unittest.TestCase):
    def context(self, root: Path, control: int, step: int, fail: bool = False) -> gate.OwnedOrderedInputs:
        fixture=fixtures.RuntimeTests();self.addCleanup(fixture.doCleanups)
        request=fixture.fixture(root/'fixture',step,fail=fail)
        context=owned_inputs(root,control)
        product=replace(context.components.probes.candidate,input_root=request.root,input_files=request.inputs,
                        executable=request.executable.binary,executable_sha256=file_hash(request.executable.binary))
        bound=gate.Library49Context(product.source_sha,replace(request,held_lane_fd=-1))
        return replace(context,components=replace(context.components,probes=replace(context.components.probes,candidate=product)),
                       prefix28=bound if step==28 else None,prefix36=bound if step==36 else None)

    def test_optional_order_and_default_preserved(self) -> None:
        original=gate.ordered_owned_calls()
        actual=gate.ordered_owned_calls(prefix28_enabled=True,prefix36_enabled=True)
        self.assertEqual(tuple(call for call in actual if call.component!='native_prefix'),original)
        for step in (28,36):
            index=actual.index(gate.prefix_call(step))
            self.assertEqual(actual[index+1].step,step)
            self.assertNotEqual(actual[index+1].component,'native_prefix')

    def test_real_prefix_returns_pending_and_full_membership_is_frozen(self) -> None:
        for step,mutation in ((28,'none'),(36,'none'),(28,'file'),(28,'empty'),(36,'kind'),(28,'bytes')):
            with self.subTest(step=step,mutation=mutation),TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();read,write=os.pipe()
                try:
                    context=self.context(root,read,step)
                    with patch.object(gate,'LOCK',root/'lane'),patch.object(prefix,'LOCK',root/'lane'):
                        row=gate.invoke_prefix(gate.prefix_call(step),context,root/'out')
                    self.assertEqual(row.status,'PENDING',row.error)
                    self.assertEqual(row.reported_status,'PASS_PREFIX_ONLY')
                    gate.recheck_prefix(context,row)
                    if mutation=='none':continue
                    if mutation=='file':_=(root/'out/late').write_text('new')
                    if mutation=='empty':(root/'out/late').mkdir()
                    if mutation=='kind':(root/'out/result.json').unlink();(root/'out/result.json').mkdir()
                    if mutation=='bytes':_=(root/'out/result.json').write_text('{}')
                    with self.assertRaises((GateContractError,OSError,ValueError)):
                        gate.recheck_prefix(context,row)
                finally:os.close(read);os.close(write)

    def test_settled_failure_blocks_tail_but_continues_outer(self) -> None:
        for step in (28,36):
            with self.subTest(step=step),TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();read,write=os.pipe()
                try:
                    context=self.context(root,read,step)
                    calls=(gate.prefix_call(step),gate.OrderedCall(step,'direct' if step==28 else 'pack',()),gate.OrderedCall(step+1,'direct',()))
                    def fail(request: Library49Request, _step: int) -> prefix.NativePrefixResult:
                        request.output.mkdir();_=(request.output/'partial').write_text('failure')
                        raise LibrarySettledFailure(request,ValueError('primary marker'))
                    def action(_out: Path) -> ComponentResult:
                        raise AssertionError('mocked invoke must not execute action')
                    with patch.object(gate,'LOCK',root/'lane'),patch.object(prefix,'LOCK',root/'lane'), \
                            patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'), \
                            patch.object(gate,'perf_bindings'),patch.object(prefix,'run_owned',side_effect=fail), \
                            patch.object(gate,'ordered_action',return_value=action), \
                            patch.object(gate,'invoke',return_value=gate.Outcome('direct','INCOMPLETE',None,None,'')), \
                            patch.object(gate,'lane',return_value=nullcontext()):
                        path=gate.run_ordered_owned(context,context.strict,root/'out',read,prefix28=context.prefix28,prefix36=context.prefix36)
                    node=document(path.read_text());rows=node.field('components').array()
                    self.assertEqual([row.field('status').text() for row in rows],['FAIL','NOT_RUN','INCOMPLETE'])
                    self.assertEqual(node.field('actual_component_sequence').value,[f'{step}:native_prefix',f'{step+1}:direct'])
                    self.assertIn('primary marker',rows[0].field('error').text())
                finally:os.close(read);os.close(write)

    def test_native_failure_is_not_promoted(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read,28,True)
                with patch.object(gate,'LOCK',root/'lane'),patch.object(prefix,'LOCK',root/'lane'):
                    row=gate.invoke_prefix(gate.prefix_call(28),context,root/'out')
                self.assertEqual(row.status,'FAIL');self.assertEqual(row.reported_status,'FAIL')
            finally:os.close(read);os.close(write)


    def test_success_enables_tail_without_full_compound_pass(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read,28)
                calls=(gate.prefix_call(28),gate.OrderedCall(28,'direct',()))
                def action(_out: Path) -> ComponentResult:
                    raise AssertionError('mocked tail')
                with patch.object(gate,'LOCK',root/'lane'),patch.object(prefix,'LOCK',root/'lane'), \
                        patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'), \
                        patch.object(gate,'perf_bindings'),patch.object(gate,'ordered_action',return_value=action), \
                        patch.object(gate,'invoke',return_value=gate.Outcome('direct','INCOMPLETE',None,None,'')), \
                        patch.object(gate,'lane',return_value=nullcontext()):
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,prefix28=context.prefix28)
                node=document(path.read_text())
                self.assertEqual(node.field('actual_component_sequence').value,['28:native_prefix','28:direct'])
                self.assertEqual(node.field('status').text(),'INCOMPLETE')
                self.assertEqual(node.field('publication_authorized').value,False)
            finally:os.close(read);os.close(write)

    def test_later_callback_empty_directory_mutation_blocks_next_callback(self) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read,28)
                calls=(gate.prefix_call(28),gate.OrderedCall(28,'direct',()),gate.OrderedCall(29,'direct',()))
                def action(_out: Path) -> ComponentResult:
                    raise AssertionError('mocked tail')
                def select(_call: gate.OrderedCall, _context: gate.OrderedInputs) -> gate.Action:
                    (root/'out/step-28/native_prefix/late-empty').mkdir()
                    return action
                with patch.object(gate,'LOCK',root/'lane'),patch.object(prefix,'LOCK',root/'lane'), \
                        patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'), \
                        patch.object(gate,'perf_bindings'),patch.object(gate,'ordered_action',side_effect=select), \
                        patch.object(gate,'invoke',return_value=gate.Outcome('direct','INCOMPLETE',None,None,'')) as invoked,patch.object(gate,'lane',return_value=nullcontext()):
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,prefix28=context.prefix28)
                node=document(path.read_text())
                self.assertEqual(invoked.call_count,1)
                self.assertEqual(node.field('status').text(),'FAIL')
                self.assertEqual(node.field('components').array()[2].field('status').text(),'NOT_RUN')
            finally:os.close(read);os.close(write)

    def test_final_callbacks_and_publication_recheck_original_anchor(self) -> None:
        for phase in ('bindings','publication'):
            for failed in (False,True):
                with self.subTest(phase=phase,failed=failed):self.check_final_mutation(phase,failed)

    def check_final_mutation(self, phase: str, failed: bool) -> None:
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            try:
                context=self.context(root,read,28)
                original=atomic_json
                native=root/'out/step-28/native_prefix'
                def mutate() -> None:
                    (native/'late-empty').mkdir(exist_ok=True)
                def bindings(_context: gate.OrderedInputs) -> None:
                    if phase=='bindings' and native.exists():mutate()
                def publish(path: Path, data: dict[str,Json]) -> None:
                    original(path,data)
                    if phase=='publication' and path.name=='ordered-dispatch.json':mutate()
                actual_launch=prefix.run_owned
                def launch(request: Library49Request, step: int) -> prefix.NativePrefixResult:
                    if failed:
                        request.output.mkdir();_=(request.output/'partial').write_text('primary evidence')
                        raise LibrarySettledFailure(request,ValueError('primary marker'))
                    return actual_launch(request,step)
                with patch.object(gate,'LOCK',root/'lane'),patch.object(prefix,'LOCK',root/'lane'), \
                        patch.object(gate,'ordered_owned_calls',return_value=(gate.prefix_call(28),)), \
                        patch.object(gate,'ordered_bindings',side_effect=bindings),patch.object(gate,'perf_bindings'), \
                        patch.object(gate,'atomic_json',side_effect=publish),patch.object(prefix,'run_owned',side_effect=launch):
                    path=gate.run_ordered_owned(context,context.strict,root/'out',read,prefix28=context.prefix28)
                node=document(path.read_text())
                self.assertEqual(path.name,'ordered-dispatch-prefix-failure.json')
                prior=root/'out/ordered-dispatch.json'
                self.assertTrue(prior.is_file())
                self.assertEqual(node.field('supersedes').field('path').text(),str(prior))
                self.assertEqual(node.field('supersedes').field('sha256').text(),file_hash(prior))
                self.assertEqual(node.field('status').text(),'FAIL')
                self.assertTrue(node.field('errors').array())
                row=node.field('components').array()[0]
                self.assertNotIn('late-empty',row.field('native_prefix_state').mapping())
                if failed:self.assertIn('primary marker',row.field('error').text())
            finally:os.close(read);os.close(write)

    def test_unknown_retains_live_fd_request_and_stops_on_eof(self) -> None:
        script=r'''import os,sys
from pathlib import Path
from unittest.mock import patch
import generic_full_gate as g
import generic_integration_prefix_run as prefix
from generic_library_run import LibraryOwnershipUnknown
from test_generic_full_gate_prefix import PrefixCoordinatorTests
root=Path(sys.argv[1]);fd=int(sys.argv[2]);context=PrefixCoordinatorTests().context(root,fd,28)
def launch(request,step):
 (root/'entered').write_text(str(request.held_lane_fd))
 raise LibraryOwnershipUnknown(request,RuntimeError('actual retained request'))
with patch.object(g,'LOCK',root/'lane'),patch.object(prefix,'LOCK',root/'lane'),patch.object(prefix,'run_owned',side_effect=launch),patch.object(g,'prefix_outcome',side_effect=AssertionError('UNKNOWN may not snapshot')):
 g.invoke_prefix(g.prefix_call(28),context,root/'out')
(root/'returned').write_text('bad')
'''
        with TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();read,write=os.pipe()
            env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1','PYTHONPATH':os.pathsep.join((str(Path(__file__).parent),str(Path(fixtures.__file__).parent)))}
            with subprocess.Popen([sys.executable,'-c',script,str(root),str(read)],pass_fds=(read,),env=env,
                                  stdout=subprocess.PIPE,stderr=subprocess.PIPE) as child:
                os.close(read)
                try:
                    deadline=time.monotonic()+10
                    while not (root/'entered').exists() and child.poll() is None and time.monotonic()<deadline:time.sleep(.01)
                    self.assertTrue((root/'entered').exists())
                    self.assertNotEqual(lock_contender(root/'lane'),0)
                    os.close(write);write=-1;time.sleep(.1)
                    self.assertIsNone(child.poll());self.assertNotEqual(lock_contender(root/'lane'),0)
                    self.assertFalse((root/'returned').exists())
                finally:
                    if write>=0:os.close(write)
                    if child.poll() is None:child.kill()
                    _=child.communicate(timeout=5)

if __name__=='__main__':
    _=unittest.main()
