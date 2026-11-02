"""Synthetic exact-horizon admission tests; no Godot runtime."""
import json
import unittest
from dataclasses import replace
from typing import override

import test_generic_ui_receipts as fixtures
from generic_gate import GateContractError
from generic_observations import Observation
from generic_process import environment_for, file_hash
from generic_ui_companion import validate_smoke_companion
from generic_ui_history import UIEvidence
from mux2_gate_inputs import Json


class CompanionTests(unittest.TestCase):
    actual: fixtures.UIReceiptTests
    companion: fixtures.UIReceiptTests

    @override
    def setUp(self) -> None:
        self.actual = fixtures.UIReceiptTests()
        self.companion = fixtures.UIReceiptTests()
        for fixture,producer,script in ((self.actual,'smoke.labels','smoke.gd'),
                                       (self.companion,'smoke.companion','gate_ui_companion.gd')):
            fixture.setUp();self.addCleanup(fixture.doCleanups)
            names = ('tools/v8/smoke.gd','tools/v8/gate_ui_companion.gd','bridge.dylib','scripts/core/main_bootstrap.gd')
            for name in names:
                path = fixture.cwd/name;path.parent.mkdir(parents=True,exist_ok=True);_ = path.write_text('synthetic source')
            inputs = fixture.spec.inputs + tuple((name,file_hash(fixture.cwd/name)) for name in names)
            environment = tuple((k,producer if k=='WORLDSIM_GATE_INVOCATION_ID' else v) for k,v in fixture.spec.environment)
            environment += (('WORLDSIM_TEST_SETTINGS_DIR',str(fixture.output/'settings')),)
            argv=list(fixture.spec.argv);argv[argv.index('--script')+1]='res://tools/v8/'+script
            fixture.data['argv']=[v for v in argv[4:argv.index('--')]]
            fixture.process['argv']=[v for v in argv]
            fixture.spec = replace(fixture.spec,argv=tuple(argv),inputs=inputs,environment=environment,fresh_directories=('settings',))
            fixture.plan = replace(fixture.plan,producer=producer,invocation_id=producer,script='res://tools/v8/'+script,process=fixture.spec)
            fixture.path=fixture.sidecars/(producer+'-123.json')
            fixture.data.update(producer=producer,invocation_id=producer,script=fixture.plan.script,
                                script_sha256=dict(inputs)['tools/v8/'+script])
            fixture.process['fresh_directories']=['settings']
            fixture.process.update(environment={k:v for k,v in environment_for(fixture.spec).items()},expected_inputs=dict(inputs),
                                   before_inputs=dict(inputs),after_inputs=dict(inputs))
            for cp in fixtures.checkpoints(fixture.data):
                cp['commands']={'total':1,'ticks':[0],'kinds':['SetSteward'],'accepted':[1]}
        fixtures.obj(self.companion.data, 'checkpoints', 1)['label']='companion_terminal'

    def evidence(self, fixture: fixtures.UIReceiptTests) -> UIEvidence:
        _ = fixture.path.write_text(json.dumps(fixture.data));_ = fixture.result.receipt.write_text(json.dumps(fixture.process))
        return UIEvidence(fixture.path,file_hash(fixture.path),fixture.plan,fixture.result,file_hash(fixture.result.receipt))

    def validate(self) -> Observation:
        actual = self.evidence(self.actual)
        c=self.companion
        args=c.spec.argv[:c.spec.argv.index('--')+1]+('--ticks','30','--original-producer',self.actual.plan.producer,
                                                   '--original-sidecar-sha256',actual.sidecar_sha256)
        c.spec=replace(c.spec,argv=args)
        context: dict[str, Json] = {'requested_tick':30,'original_producer':self.actual.plan.producer,'original_sidecar_sha256':actual.sidecar_sha256}
        c.plan=replace(c.plan,process=c.spec,checkpoints=(c.plan.checkpoints[0],replace(c.plan.checkpoints[1],
                       label='companion_terminal',context_json=json.dumps(context))))
        c.process['argv']=list(args)
        c.data['argv']=list(args[4:args.index('--')]);c.data['user_args']=list(args[args.index('--')+1:])
        fixtures.obj(c.data, 'checkpoints', 1)['context']=context
        return validate_smoke_companion(actual,self.evidence(c))

    def test_exact_native_comparison_is_not_ui_oracle(self) -> None:
        self.assertEqual(self.validate().terminal_tick,30)

    def test_plain_boot_native_exit_seam_is_supported(self) -> None:
        a=self.actual
        script='res://scripts/core/main_bootstrap.gd'
        args=list(a.spec.argv);index=args.index('--script');del args[index:index+2]
        environment=tuple((k,'smoke.boot' if k=='WORLDSIM_GATE_INVOCATION_ID' else v) for k,v in a.spec.environment)
        a.spec=replace(a.spec,argv=tuple(args),environment=environment+(('WORLDSIM_GATE_UI_BOOT','1'),))
        a.plan=replace(a.plan,producer='smoke.boot',invocation_id='smoke.boot',script=script,
                       completion='native_tree_exit',process=a.spec)
        a.path=a.sidecars/'smoke.boot-123.json'
        a.data.update(producer='smoke.boot',invocation_id='smoke.boot',script=script,
                      script_sha256=dict(a.spec.inputs)[script[6:]],completion='native_tree_exit',native_exit_code=-1,
                      argv=[v for v in args[4:args.index('--')]])
        a.process.update(argv=[v for v in args],environment={k:v for k,v in environment_for(a.spec).items()})
        self.assertEqual(self.validate().terminal_tick,30)

    def test_existing_user_settings_not_admitted(self) -> None:
        a=self.actual
        a.spec=replace(a.spec,environment=tuple((k,v) for k,v in a.spec.environment if k!='WORLDSIM_TEST_SETTINGS_DIR'))
        a.plan=replace(a.plan,process=a.spec)
        with self.assertRaisesRegex(GateContractError,'fresh process output'):_ = self.validate()

    def test_native_steward_commands_are_preserved_and_matched(self) -> None:
        for fixture in (self.actual, self.companion):
            fixtures.obj(fixture.data, 'checkpoints', 1)['commands'] = {
                'total': 2, 'ticks': [0, 20], 'kinds': ['SetSteward', 'Steward'], 'accepted': [1, 1]}
        self.assertEqual(self.validate().terminal_tick, 30)
        fixtures.obj(self.companion.data, 'checkpoints', 1)['commands'] = {
            'total': 2, 'ticks': [0, 21], 'kinds': ['SetSteward', 'Steward'], 'accepted': [1, 1]}
        with self.assertRaisesRegex(GateContractError, 'command logs differ'):
            _ = self.validate()

    def test_player_command_cannot_masquerade_as_native_steward(self) -> None:
        fixtures.obj(self.actual.data, 'checkpoints', 1)['commands'] = {
            'total': 2, 'ticks': [0, 20], 'kinds': ['SetSteward', 'SetSteward'], 'accepted': [1, 1]}
        with self.assertRaisesRegex(GateContractError, 'unexpected UI command'):
            _ = self.validate()

    def test_original_commands_rejected(self) -> None:
        fixtures.obj(self.actual.data, 'checkpoints', 1, 'commands')['total']=2
        with self.assertRaises(GateContractError):_ = self.validate()

    def test_native_census_mismatch_rejected(self) -> None:
        fixtures.obj(self.companion.data, 'checkpoints', 1, 'observation', 'instances', 0)['terminal_agent_ids']=['1']
        with self.assertRaisesRegex(GateContractError,'native state/history'):_ = self.validate()

    def test_non_smoke_original_rejected(self) -> None:
        self.actual.plan=replace(self.actual.plan,producer='renderer.agreement')
        with self.assertRaisesRegex(GateContractError,'unsupported original'):_ = self.validate()


if __name__=='__main__':_ = unittest.main()
