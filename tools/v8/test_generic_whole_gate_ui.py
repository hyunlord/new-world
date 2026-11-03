# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_ui
"""Actual tiny Bash wait with explicitly synthetic UI evidence; never launch Godot."""
import copy
import json
import sys
import unittest
from dataclasses import replace
from pathlib import Path

import test_generic_ui_receipts as fixtures
from generic_gate import GateContractError
from generic_process import ProcessSpec, environment_for, execute, file_hash
from mux2_gate_inputs import Json, Node

from generic_ui_receipts import UIPlan, validate_lifecycle
from generic_whole_gate_ui import WholeUIPlan, collect_ui
from generic_whole_gate_ui_fixed import collect_fixed


class WholeFixture:
    root: Path
    cwd: Path
    path: Path
    data: dict[str, Json]
    plan: UIPlan
    spec: ProcessSpec
    session: Node
    stage: Node
    whole: WholeUIPlan

    def __init__(self, case: unittest.TestCase, child: str = '') -> None:
        fixture=fixtures.UIReceiptTests()
        fixture.setUp()
        case.addCleanup(fixture.doCleanups)
        self.root,self.cwd,self.path=fixture.root,fixture.cwd,fixture.path
        self.data,self.plan,self.spec=fixture.data,fixture.plan,fixture.spec
        step = self.root/'steps'/'candidate'/'34-fixture'
        step.mkdir(parents=True)
        sidecars = step/'ui-worlds'
        sidecars.mkdir()
        selected=sidecars/child if child else sidecars
        if child:selected.mkdir()
        self.path = selected/'fixture-123.json'
        self.data['invocation_id']='whole-fixture'
        # Given synthetic world bytes, emitted by a real harmless Python child.
        emitter = self.cwd/'emitter.py'
        _ = emitter.write_text('from pathlib import Path\nPath('+repr(str(self.path))+').write_text('+repr(json.dumps(self.data))+')\n')
        inputs = tuple((name,file_hash(self.cwd/name)) for name in ('producer.gd','helper.gd','emitter.py'))
        env=(('WORLDSIM_GATE_OBSERVER','1'),('WORLDSIM_GATE_INVOCATION_ID','whole-fixture'),
             ('WORLDSIM_GATE_UI_OUT',str(sidecars)))
        python=Path(sys.executable).resolve(strict=True)
        argv=(str(python),str(emitter))
        bash=Path('/bin/bash')
        spec=ProcessSpec((str(bash),'-c','exec "$@" 2>&1','whole-step',*argv),self.cwd,bash,file_hash(bash),
                         self.cwd,inputs,step/'process',env,((python,file_hash(python)),))
        # When the actual outer stage waits normally; its PID is not synthetic UI PID123.
        result=execute(spec)
        case.assertEqual(result.native_exit,0)
        files={str(p):file_hash(p) for p in step.rglob('*') if p.is_file()}
        directories={str(p):p.stat().st_mode for p in (step,*step.rglob('*')) if p.is_dir()}
        self.session=Node({'output':str(self.root),'arms':{'candidate':{'root':str(self.cwd),'source_inputs':dict(inputs)}},
                           'bash':{'path':str(bash),'sha256':file_hash(bash)},
                           'external_inputs':{str(python):file_hash(python)}})
        stage: dict[str, Json] = {'schema':'whole_gate_step.v1','step':34,'name':'fixture','arm':'candidate','argv':list(argv),
                         'cwd':str(self.cwd),'environment':{k:v for k,v in environment_for(spec).items()},'state':'RETURNED','native_exit':0,
                         'process_status':'PASS','pid':result.pid,'process_receipt':str(result.receipt),
                         'process_receipt_sha256':file_hash(result.receipt),'observer_artifacts':dict(files),
                         'observer_directories':dict(directories)}
        self.stage=Node(stage)
        self.whole=WholeUIPlan(self.path,'fixture',self.spec.argv,self.plan.script,self.plan.helper,
                                tuple((n,d) for n,d in inputs if n!='emitter.py'),self.plan.checkpoints,self.plan.instances)



class WholeUITests(unittest.TestCase):
    def test_real_outer_wait_keeps_child_wait_unclaimed(self) -> None:
        fixture = WholeFixture(self)
        actual=collect_ui(fixture.session,fixture.stage,fixture.whole)
        self.assertEqual(actual.emitted_pid,123)
        self.assertNotEqual(actual.aggregate_pid,123)
        self.assertFalse(actual.independent_child_wait)
        self.assertEqual(actual.qualification,'SOURCE_BOUND_AGGREGATE_ONLY')
        self.assertEqual(actual.validated.fresh_observations()[0].terminal_tick,30)

    def test_extracted_lifecycle_rejects_omitted_release(self) -> None:
        fixture = WholeFixture(self)
        with self.assertRaises(GateContractError):
            _=validate_lifecycle(Node(fixture.data),tuple(replace(c,releases_node=False) for c in fixture.plan.checkpoints),fixture.plan.instances)

    def test_added_empty_directory_is_not_accepted_as_new_anchor(self) -> None:
        fixture = WholeFixture(self)
        (fixture.path.parent/'foreign').mkdir()
        with self.assertRaisesRegex(GateContractError,'namespace'):
            _=collect_ui(fixture.session,fixture.stage,fixture.whole)

    def test_changed_sidecar_and_source_are_rejected(self) -> None:
        fixture = WholeFixture(self)
        original=fixture.path.read_bytes()
        _=fixture.path.write_text('{}')
        with self.assertRaisesRegex(GateContractError,'bytes'):
            _=collect_ui(fixture.session,fixture.stage,fixture.whole)
        _=fixture.path.write_bytes(original)
        _=(fixture.cwd/'producer.gd').write_text('foreign')
        with self.assertRaisesRegex(GateContractError,'source'):
            _=collect_ui(fixture.session,fixture.stage,fixture.whole)

    def test_foreign_invocation_and_plan_arguments_rejected(self) -> None:
        fixture = WholeFixture(self)
        with self.assertRaises(GateContractError):
            _=collect_ui(fixture.session,fixture.stage,replace(fixture.whole,argv=('godot','--script','res://foreign.gd')))
        with self.assertRaisesRegex(GateContractError,'source semantics'):
            _=collect_ui(fixture.session,fixture.stage,replace(fixture.whole,source_pins=(('producer.gd','0'*64),)))

    def test_parent_failure_cannot_be_replaced_by_complete_sidecar(self) -> None:
        fixture = WholeFixture(self)
        altered=copy.deepcopy(fixture.stage.mapping());altered['native_exit']=1
        with self.assertRaises(GateContractError):
            _=collect_ui(fixture.session,Node(altered),fixture.whole)

    def test_source_owned_fixed_selector_rejects_arbitrary_script(self) -> None:
        fixture = WholeFixture(self)
        altered=copy.deepcopy(fixture.stage.mapping());altered['name']='mart2_render'
        with self.assertRaisesRegex(GateContractError,'source command'):
            _=collect_fixed(fixture.session,Node(altered))

    def test_pure_lifecycle_keeps_exact_checkpoint_plan(self) -> None:
        fixture = WholeFixture(self)
        # The pure entry keeps the same exact native checkpoint semantics.
        rows=fixtures.checkpoints(fixture.data)
        self.assertEqual(len(rows),2)
        self.assertEqual(validate_lifecycle(Node(fixture.data),fixture.plan.checkpoints,fixture.plan.instances).loaded_unqualified,())


class WholeUIRouteTests(unittest.TestCase):
    def test_source_owned_nested_sidecar_keeps_original_namespace(self) -> None:
        fixture=WholeFixture(self,'why-en')
        result=collect_ui(fixture.session,fixture.stage,fixture.whole)
        self.assertFalse(result.independent_child_wait)
        _=(fixture.path.parent/'late.png').write_bytes(b'foreign')
        with self.assertRaisesRegex(GateContractError,'namespace'):
            _=collect_ui(fixture.session,fixture.stage,fixture.whole)

    def test_foreign_subtree_is_rejected(self) -> None:
        fixture=WholeFixture(self)
        with self.assertRaises(GateContractError):
            _=collect_ui(fixture.session,fixture.stage,replace(fixture.whole,sidecar=fixture.cwd/'fixture-123.json'))

    def test_why_source_owned_command_rejects_stale_envelope(self) -> None:
        from generic_whole_gate_ui_regressions import collect_regressions
        fixture=WholeFixture(self)
        value=dict(fixture.stage.mapping());value.update(step=46,name='mux21_regressions')
        with self.assertRaisesRegex(GateContractError,'source command'):
            _=collect_regressions(fixture.session,Node(value))

    def test_selected_ui_routes_reject_foreign_command(self) -> None:
        from generic_whole_gate_ui_selected import collect_pack, collect_renderer
        fixture=WholeFixture(self)
        for step,name,collect in ((23,'renderer_agreement',collect_renderer),(36,'data_only_ext',collect_pack)):
            value=dict(fixture.stage.mapping());value.update(step=step,name=name)
            with self.assertRaisesRegex(GateContractError,'original UI command'):
                _=collect(fixture.session,Node(value))

    def test_smoke_source_route_rejects_foreign_command(self) -> None:
        from generic_whole_gate_ui_smoke import collect_smoke
        fixture=WholeFixture(self)
        value=dict(fixture.stage.mapping());value.update(step=9,name='smoke')
        with self.assertRaisesRegex(GateContractError,'original smoke command'):
            _=collect_smoke(fixture.session,Node(value))


    def test_visibility_source_route_rejects_foreign_command(self) -> None:
        from generic_whole_gate_ui_visibility import collect_visibility
        fixture=WholeFixture(self)
        value=dict(fixture.stage.mapping());value.update(step=39,name='mux2_visibility')
        with self.assertRaisesRegex(GateContractError,'original visibility command'):
            _=collect_visibility(fixture.session,Node(value))


    def test_original_why_source_route_rejects_locale_envelope(self) -> None:
        from generic_whole_gate_ui_why import collect_why
        fixture=WholeFixture(self)
        value=dict(fixture.stage.mapping());value.update(step=40,name='mux2_why')
        with self.assertRaisesRegex(GateContractError,'original Why capture command'):
            _=collect_why(fixture.session,Node(value))


    def test_actual_returned_process_pid_is_not_capped_at_one(self) -> None:
        from generic_whole_gate_ui_smoke_join import read_returned_process
        fixture=WholeFixture(self)
        path=Path(fixture.stage.field('process_receipt').text())
        actual=read_returned_process(path,fixture.stage.field('process_receipt_sha256').text())
        self.assertEqual(actual.pid,fixture.stage.field('pid').value)
        self.assertGreater(actual.pid or 0,1)
