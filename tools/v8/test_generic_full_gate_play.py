"""Offline coordinator fixtures, never original launcher qualification."""
import os
import signal
import subprocess
import sys
import time
import unittest
from contextlib import nullcontext
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import gate_oracle_child
import generic_full_gate as gate
import generic_play_owner as play_owner
import generic_play_sources as sources
import test_generic_play_owner as fixture_owner
from generic_fixture_run import ComponentResult
from generic_gate import GateContractError, document
from generic_process import file_hash
from test_generic_full_gate import lock_contender, owned_inputs


class PlayCoordinatorTests(unittest.TestCase):
    def test_original_whole_call_is_optional_and_ordered(self) -> None:
        disabled = gate.ordered_owned_calls()
        enabled = gate.ordered_owned_calls(play_enabled=True)
        play = [call for call in enabled if call.step == 21]
        self.assertEqual(len(play), 1)
        self.assertEqual(play[0].component, 'play')
        self.assertEqual(len(play[0].identities), 6)
        self.assertEqual(tuple(call for call in enabled if call.step != 21), disabled)
        self.assertLess(next(i for i,c in enumerate(enabled) if c.step == 20), enabled.index(play[0]))
        self.assertLess(enabled.index(play[0]), next(i for i,c in enumerate(enabled) if c.step == 22))

    def test_actual_whole_outer_and_late_membership(self) -> None:
        for mutation in ('none','owner-empty','native-empty','native-file','native-type','terminal'):
            with self.subTest(mutation=mutation), TemporaryDirectory() as folder, patch.object(sources.SourceGrants,'check'):
                base=Path(folder).resolve(); (base/'context').mkdir(); plan=fixture_owner.OwnerTests().fixture(base)
                lock=base/'lane'; lock.touch(); read,write=os.pipe()
                try:
                    context=owned_inputs(base/'context',read)
                    bound=gate.PlayContext(plan)
                    with patch.object(gate,'LOCK',lock), patch.object(gate_oracle_child,'LOCK',lock), \
                            patch.object(gate,'play_bindings',return_value=bound), \
                            patch.object(gate,'authenticate_play',wraps=gate.authenticate_play) as authentication:
                        # Pure fixture source admission is mocked, original owner/native protocol is real.

                        def wait(live: play_owner.PlayOwner) -> play_owner.PlayOutcome:
                            result=ORIGINAL_WAIT(live)
                            live.source_grants=fixture_grants()
                            return result
                        with patch.object(play_owner.PlayOwner,'wait',wait):
                            row=gate.invoke_play(gate.play_call(),context,base/'actual')
                        self.assertEqual(row.status,'PENDING'); self.assertTrue(authentication.called)
                    self.assertIsNotNone(row.play_evidence)
                    assert row.play_evidence is not None
                    native=row.play_evidence.native_members[0][0]
                    if mutation=='owner-empty': (base/'actual/late').mkdir()
                    if mutation=='native-empty': (native/'late').mkdir()
                    if mutation=='native-file': _=(native/'late').write_text('drift')
                    if mutation=='native-type':
                        (native/'stdout.log').unlink(); (native/'stdout.log').mkdir()
                    if mutation=='terminal': _=(base/'actual/terminal.json').write_text('{}')
                    context=replace(context,play=bound)
                    if mutation=='none':gate.recheck_play(context,row)
                    else:
                        with self.assertRaises((GateContractError,OSError,ValueError)):
                            gate.recheck_play(context,row)
                finally:
                    os.close(read);os.close(write)

    def test_rejected_unexpected_external_file_bytes_remain_frozen(self) -> None:
        with TemporaryDirectory() as folder, patch.object(sources.SourceGrants,'check'):
            base=Path(folder).resolve();(base/'context').mkdir();plan=fixture_owner.OwnerTests().fixture(base)
            lock=base/'lane';lock.touch();read,write=os.pipe()
            try:
                context=owned_inputs(base/'context',read);bound=gate.PlayContext(plan)
                unexpected: list[Path]=[]
                def wait(live: play_owner.PlayOwner) -> play_owner.PlayOutcome:
                    result=ORIGINAL_WAIT(live);live.source_grants=fixture_grants()
                    path=next(iter(live.native_memberships))/'unexpected.txt'
                    _=path.write_text('first rejected bytes');unexpected.append(path)
                    return result
                with patch.object(gate,'LOCK',lock),patch.object(gate_oracle_child,'LOCK',lock), \
                        patch.object(gate,'play_bindings',return_value=bound),patch.object(play_owner.PlayOwner,'wait',wait):
                    row=gate.invoke_play(gate.play_call(),context,base/'actual')
                self.assertEqual(row.status,'FAIL')
                assert row.play_evidence is not None
                self.assertIsNotNone(row.play_evidence.failure)
                context=replace(context,play=bound)
                gate.recheck_play(context,row)
                _=unexpected[0].write_text('second rejected bytes')
                with self.assertRaisesRegex(ValueError,'rejected native file'):
                    gate.recheck_play(context,row)
                self.assertEqual(row.status,'FAIL')
                _=unexpected[0].write_text('first rejected bytes')
                gate.recheck_play(context,row)
                added=unexpected[0].with_name('second-extra.txt');_=added.write_text('new member')
                with self.assertRaisesRegex(ValueError,'membership'):
                    gate.recheck_play(context,row)
                added.unlink()
                unexpected[0].unlink();unexpected[0].mkdir()
                with self.assertRaisesRegex(ValueError,'membership'):
                    gate.recheck_play(context,row)
                unexpected[0].rmdir();unexpected[0].symlink_to(plan.suite)
                with self.assertRaisesRegex(ValueError,'symlink'):
                    gate.recheck_play(context,row)
                unexpected[0].unlink();_=unexpected[0].write_text('first rejected bytes')
                gate.recheck_play(context,row)
                self.assertEqual(row.status,'FAIL')
            finally:os.close(read);os.close(write)

    def test_missing_source_grants_rejected_before_owner_creation(self) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve(); (base/'context').mkdir(); plan=fixture_owner.OwnerTests().fixture(base)
            read,write=os.pipe();lock=base/'lane';lock.touch()
            try:
                context=replace(owned_inputs(base/'context',read),play=gate.PlayContext(plan))
                with patch.object(gate,'LOCK',lock),patch.object(play_owner,'PlayOwner') as launch, self.assertRaises(GateContractError):
                    _=gate.invoke_play(gate.play_call(),context,base/'actual')
                launch.assert_not_called()
                self.assertFalse((base/'actual').exists())
            finally:os.close(read);os.close(write)

    def test_settled_original_oracle_failure_continues22(self) -> None:
        with TemporaryDirectory() as folder, patch.object(sources.SourceGrants,'check'):
            base=Path(folder).resolve();(base/'context').mkdir()
            plan=fixture_owner.OwnerTests().fixture(base)
            script=plan.root/'tools/fixture.py'
            _=script.write_text(script.read_text().replace("else 0)","else 9)"))
            plan=replace(plan,inputs=tuple((p,file_hash(p)) for p,_ in plan.inputs))
            bound=gate.PlayContext(plan);read,write=os.pipe();lock=base/'lane';lock.touch()
            try:
                context=replace(owned_inputs(base/'context',read),play=bound)
                seen: list[str]=[]
                def next_action(call: gate.OrderedCall,_context: gate.OrderedInputs) -> gate.Action:
                    seen.append(call.identity)
                    return lambda out:ComponentResult(out/'receipt.json','FAIL')
                def wait(live: play_owner.PlayOwner) -> play_owner.PlayOutcome:
                    result=ORIGINAL_WAIT(live);live.source_grants=fixture_grants();return result
                calls=(gate.play_call(),gate.OrderedCall(22,'direct',()))
                with patch.object(gate,'LOCK',lock),patch.object(gate_oracle_child,'LOCK',lock), \
                        patch.object(gate,'play_bindings',return_value=bound),patch.object(gate,'play_dependencies',return_value={}), \
                        patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'), \
                        patch.object(gate,'perf_bindings'),patch.object(gate,'ordered_action',side_effect=next_action), \
                        patch.object(gate,'invoke',return_value=gate.Outcome('direct','INCOMPLETE',None,None,'')), \
                        patch.object(gate,'lane',return_value=nullcontext()) as outer,patch.object(play_owner.PlayOwner,'wait',wait):
                    path=gate.run_ordered_owned(context,context.strict,base/'ordered',read,play=bound)
                self.assertEqual(seen,['22:direct']);self.assertEqual(outer.call_count,1)
                node=document(path.read_text());self.assertEqual(node.field('status').text(),'FAIL')
                self.assertEqual(node.field('actual_component_sequence').value,['21:play','22:direct'])
                self.assertEqual(node.field('components').array()[0].field('status').text(),'FAIL')
            finally:os.close(read);os.close(write)

    def test_settled_raw_log_error_continues22_without_admission(self) -> None:
        with TemporaryDirectory() as folder, patch.object(sources.SourceGrants,'check'):
            base=Path(folder).resolve();(base/'context').mkdir()
            plan=fixture_owner.OwnerTests().fixture(base)
            script=plan.root/'tools/fixture.py'
            self.assertTrue(script.exists())
            plan=replace(plan,inputs=tuple((p,file_hash(p)) for p,_ in plan.inputs))
            bound=gate.PlayContext(plan);read,write=os.pipe();lock=base/'lane';lock.touch()
            try:
                context=replace(owned_inputs(base/'context',read),play=bound)
                seen: list[str]=[]
                def next_action(call: gate.OrderedCall,_context: gate.OrderedInputs) -> gate.Action:
                    seen.append(call.identity)
                    return lambda out:ComponentResult(out/'receipt.json','FAIL')
                def wait(live: play_owner.PlayOwner) -> play_owner.PlayOutcome:
                    result=ORIGINAL_WAIT(live);live.source_grants=fixture_grants()
                    _=(live.plan.output/'stdout.log').write_text('post-wait raw drift fixture')
                    return result
                calls=(gate.play_call(),gate.OrderedCall(22,'direct',()))
                with patch.object(gate,'LOCK',lock),patch.object(gate_oracle_child,'LOCK',lock), \
                        patch.object(gate,'play_bindings',return_value=bound),patch.object(gate,'play_dependencies',return_value={}), \
                        patch.object(gate,'ordered_owned_calls',return_value=calls),patch.object(gate,'ordered_bindings'), \
                        patch.object(gate,'perf_bindings'),patch.object(gate,'ordered_action',side_effect=next_action), \
                        patch.object(gate,'invoke',return_value=gate.Outcome('direct','INCOMPLETE',None,None,'')), \
                        patch.object(gate,'lane',return_value=nullcontext()) as outer,patch.object(play_owner.PlayOwner,'wait',wait):
                    path=gate.run_ordered_owned(context,context.strict,base/'ordered',read,play=bound)
                self.assertEqual(seen,['22:direct']);self.assertEqual(outer.call_count,1)
                node=document(path.read_text());self.assertEqual(node.field('status').text(),'FAIL')
                self.assertEqual(node.field('actual_component_sequence').value,['21:play','22:direct'])
                self.assertEqual(node.field('components').array()[0].field('status').text(),'FAIL')
            finally:os.close(read);os.close(write)

    def test_context_requires_main_and_complete_current_candidate_inputs(self) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();(base/'context').mkdir();plan=fixture_owner.OwnerTests().fixture(base)
            read,write=os.pipe()
            try:
                context=owned_inputs(base/'context',read)
                tool=Path('/bin/sh').resolve();digest=file_hash(tool)
                historical=sources.HistoricalBindings((sources.HistoricalSource('main',context.components.probes.base.source_sha,'b'*64,'c'*64),),
                    (tool,digest),(tool,digest),(tool,digest),())
                plan=replace(plan,historical=historical)
                candidate=replace(context.components.probes.candidate,input_root=plan.root,
                    input_files=tuple((str(p.relative_to(plan.root)),h) for p,h in plan.inputs if p.is_relative_to(plan.root)))
                context=replace(context,play=gate.PlayContext(plan),components=replace(context.components,
                    probes=replace(context.components.probes,candidate=candidate)))
                with patch.object(sources,'check_bindings'),patch.object(gate,'freeze_source',return_value={}) as frozen:
                    self.assertEqual(gate.play_bindings(context),gate.PlayContext(plan))
                    frozen.assert_called_once_with(plan.root,candidate.source_sha)
                    badmain=replace(historical,sources=(replace(historical.sources[0],commit='f'*40),))
                    for bad in (replace(plan,historical=badmain),replace(plan,inputs=plan.inputs[1:]),
                                replace(plan,inputs=(*plan.inputs,plan.inputs[0]))):
                        with self.assertRaises(GateContractError):
                            _=gate.play_bindings(replace(context,play=gate.PlayContext(bad)))
                with patch.object(sources,'check_bindings'),patch.object(gate,'freeze_source',side_effect=GateContractError('binding','/source','candidate differs')),self.assertRaises(GateContractError):
                    _=gate.play_bindings(context)
            finally:os.close(read);os.close(write)

    def test_unknown_holds_actual_owner_and_lane_after_diagnostic_fault_and_eof(self) -> None:
        script=r'''import os,signal,subprocess,sys
from pathlib import Path
from unittest.mock import patch
sys.path[:0]=sys.argv[3:]
import generic_full_gate as gate
import generic_play_owner as play
import gate_oracle_child
import test_generic_play_owner as fixture
from test_generic_full_gate import owned_inputs
root=Path(sys.argv[1]);read=int(sys.argv[2]);(root/'context').mkdir()
plan=fixture.OwnerTests().fixture(root);bound=gate.PlayContext(plan);context=owned_inputs(root/'context',read)
children=[]
def cleanup(_signum,_frame):
 for child in children:
  child.terminate();child.wait()
 (root/'reaped').write_text('yes')
signal.signal(signal.SIGUSR1,cleanup)
class Hostile(RuntimeError):
 def __str__(self):raise TypeError('diagnostics unavailable')
def launch(live):
 live.shell=subprocess.Popen([sys.executable,'-c','import time;time.sleep(300)'])
 children.append(live.shell)
 live.request_read,live.request_write=os.pipe();live.control_read,live.control_write=os.pipe()
 (root/'entered').write_text(str(live.shell.pid))
 live.retain_failure(Hostile())
def bad_status(_owner):raise Hostile()
with patch.object(gate,'LOCK',root/'lane'),patch.object(gate_oracle_child,'LOCK',root/'lane'),patch.object(gate,'play_bindings',return_value=bound),patch.object(play.PlayOwner,'launch',launch),patch.object(gate._PlayOwner,'status',bad_status),patch.object(gate,'play_outcome',side_effect=AssertionError('unknown snapshot')):
 gate.invoke_play(gate.play_call(),context,root/'actual')
 (root/'downstream').write_text('bad')
'''
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();read,write=os.pipe()
            paths=[str(Path(gate.__file__).parent),str(Path(fixture_owner.__file__).parent)]
            with subprocess.Popen([sys.executable,'-c',script,str(root),str(read),*paths],pass_fds=(read,),stdout=subprocess.PIPE,stderr=subprocess.PIPE) as child:
                os.close(read)
                try:
                    deadline=time.monotonic()+15
                    while not (root/'entered').exists() and child.poll() is None and time.monotonic()<deadline:time.sleep(.01)
                    if child.poll() is not None:self.fail(repr(child.communicate()))
                    self.assertTrue((root/'entered').exists());self.assertNotEqual(lock_contender(root/'lane'),0)
                    os.close(write);write=-1;time.sleep(.1)
                    self.assertIsNone(child.poll());self.assertNotEqual(lock_contender(root/'lane'),0)
                    self.assertFalse((root/'downstream').exists());self.assertFalse((root/'actual/terminal.json').exists())
                finally:
                    if write>=0:os.close(write)
                    if child.poll() is None and (root/'entered').exists():
                        child.send_signal(signal.SIGUSR1);deadline=time.monotonic()+10
                        while not (root/'reaped').exists() and time.monotonic()<deadline:time.sleep(.01)
                        self.assertTrue((root/'reaped').exists())
                    if child.poll() is None:child.kill()
                    _=child.communicate(timeout=5)


ORIGINAL_WAIT = play_owner.PlayOwner.wait


def fixture_grants() -> sources.SourceGrants:
    tool=Path('/bin/sh').resolve()
    bindings=sources.HistoricalBindings((),(tool,file_hash(tool)),(tool,file_hash(tool)),(tool,file_hash(tool)),())
    with patch.object(sources,'check_bindings'):
        grants=sources.SourceGrants(bindings,tool.parent,tool.parent)
    grants.built={version:sources.BuiltBridge(tool,file_hash(tool),tool,file_hash(tool),1) for version in ('main','m1','v7')}
    return grants
