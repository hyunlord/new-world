"""Actual harmless aggregate fixtures; printed native rows are explicitly synthetic."""
import json
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_process
from generic_gate import GateContractError
from generic_process import ProcessSpec, environment_for, execute, file_hash
from generic_test_endpoint import EndpointEvidence, authenticate_endpoint
from generic_test_history import FORMAT_SOURCE, identity
from generic_test_run import JOBS_INPUTS, fixed_plan, save_plan
from generic_test_steward_scope import DEFENCE, EXTRA_SOURCE, HOUSING, IGNORED, REPLAY
from generic_test_steward_scope import TESTS as STEWARD_TESTS
from generic_whole_gate_m1 import SOURCE_PINS as M1_PINS
from generic_whole_gate_m1 import SOURCE_TREES
from generic_whole_gate_native import (
    SOURCE_PINS,
    TESTS,
    _native_segment,
    collect_family,
    evaluate_family,
)
from mux2_gate_inputs import Json, Node
from test_generic_test_history import CHILD, cases, graph
from test_generic_test_observations import ObservationTests
from test_generic_test_run import HUT_CHILD
from test_generic_whole_gate_m1 import PROBE_CHILD
from test_generic_whole_gate_steward import EXTRA
from test_generic_whole_gate_steward import rows as steward_rows
from test_generic_whole_gate_workspace_integration import accounting


class EndpointTests(ObservationTests):
    def test_endpoint_never_invents_native_exit(self) -> None:
        result = authenticate_endpoint(EndpointEvidence(self.output, self.pid, self.hashes()), self.expected)
        self.assertFalse(hasattr(result, 'native_exit'))
        self.assertFalse(hasattr(result, 'native_test_outcome'))
        self.assertEqual(result.terminal.tick, 5)
        with self.assertRaises(GateContractError):
            authenticate_endpoint(EndpointEvidence(self.output, self.pid, self.hashes()), replace(self.expected, end_tick=6))


class WholeNativeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        harness = self.root/'harness'; (harness/'tools/v8').mkdir(parents=True)
        self.output = self.root/'output'; self.output.mkdir()
        python = str(Path(sys.executable).resolve())
        (harness/'tools/v8/gate.sh').write_text(f'step 4 test_debug {python} emitter.py plan.json\n')
        original = Path(generic_process.__file__).resolve().parents[2]
        self.session: dict[str,Json] = {'output':str(self.output), 'qualification':'TEST_FIXTURE_NOT_PRODUCT_QUALIFIED',
            'harness':{'root':str(harness),'inputs':{'tools/v8/gate.sh':file_hash(harness/'tools/v8/gate.sh')}},
            'bash':{'path':'/bin/bash','sha256':file_hash(Path('/bin/bash'))}, 'arms':{}}
        arms: dict[str,Json] = {}; stages: list[Node] = []
        plan: dict[str, Json] = {}
        for target in ('colony_commands', 'colony_world'):
            plan[target] = [{'test':p.occurrence.test,'seed':p.occurrence.seed,
                'branch':p.occurrence.branch,'start':0,'end':p.occurrence.end_tick,'stop':p.stop} for p in fixed_plan(target)]
        rows = cases()
        with graph(rows) as data:
            stops = {identity(p.occurrence): p.stop for p in save_plan(data.binding())}
        for row in rows:
            row['stop'] = stops[f"{row['test']}_s{row['seed']}_{row['branch']}"]
        plan['save_load'] = rows
        plan['colony_jobs'] = [{'test':'hut_sleeps_four','seed':2,'branch':'gifted_hut',
            'start':0,'end':1640,'stop':'actual_hut_completion_plus_1440'}]
        for target in STEWARD_TESTS:
            plan[target] = steward_rows(target) if target in ('colony_steward','steward_defence') else []
        for arm in ('base','candidate'):
            product=self.root/arm; product.mkdir()
            inputs: dict[str,str]={}
            for target in plan:
                name=f'rust/crates/sim-test/tests/{target}.rs'; dest=product/name; dest.parent.mkdir(parents=True,exist_ok=True)
                dest.write_bytes((original/name).read_bytes()); self.assertEqual(file_hash(dest),SOURCE_PINS[target]); inputs[name]=file_hash(dest)
            extra_source = [str(path.relative_to(original)) for prefix in SOURCE_TREES for path in (original/prefix).rglob('*.rs')]
            for name in (*JOBS_INPUTS, FORMAT_SOURCE, EXTRA_SOURCE, *M1_PINS, *extra_source):
                dest=product/name;dest.parent.mkdir(parents=True,exist_ok=True)
                dest.write_bytes((original/name).read_bytes());inputs[name]=file_hash(dest)
            driver = 'import subprocess,sys\n'
            for target, members in plan.items():
                (product/(target+'.json')).write_text(json.dumps(members))
                child=CHILD.replace("'plan.json'", repr(target+'.json')).replace("'synthetic planned boundary'", "r['stop']")
                child=child[:child.index("for test in sorted({r['test'] for r in rows})")]
                if target=='colony_jobs':child+=HUT_CHILD.replace('COMPLETION_TICK','200')
                if target in ('colony_steward','steward_defence'):
                    child+='\nTARGET='+repr(target)+'\nREPLAY='+repr(REPLAY)+'\nHOUSING='+repr(HOUSING)+'\nDEFENCE='+repr(DEFENCE)+'\n'+EXTRA
                child+='\nfor test in '+repr(TESTS[target])+": print('test '+test+' ... '+('ignored' if test=="+repr(IGNORED)+" else 'ok'))\n"
                child+="print('test result: ok. "+str(len(TESTS[target])-int(IGNORED in TESTS[target]))+" passed; 0 failed; "+str(int(IGNORED in TESTS[target]))+" ignored; 0 measured; 0 filtered out; finished in 0.01s')\n"
                (product/(target+'.py')).write_text(child)
                driver+="print('     Running tests/"+target+".rs (synthetic-fixture)',flush=True)\n"
                driver+="subprocess.run([sys.executable,"+repr(target+'.py')+"],check=True)\n"
                inputs[target+'.py']=file_hash(product/(target+'.py'));inputs[target+'.json']=file_hash(product/(target+'.json'))
            (product/'native_probes.py').write_text(PROBE_CHILD);inputs['native_probes.py']=file_hash(product/'native_probes.py')
            driver+="print('Running unittests src/lib.rs (target/debug/deps/sim_test-synthetic)',flush=True)\nsubprocess.run([sys.executable,'native_probes.py'],check=True)\n"
            (product/'emitter.py').write_text(driver);(product/'plan.json').write_text('{}')
            inputs.update({name:file_hash(product/name) for name in ('emitter.py','plan.json')})
            arms[arm]={'root':str(product),'source_inputs':inputs}
            step=self.output/'steps'/arm/'04-test_debug'; observer=step/'native-worlds';observer.mkdir(parents=True)
            argv=(python,'emitter.py','plan.json')
            spec=ProcessSpec(('/bin/bash','-c','exec "$@" 2>&1','whole-step',*argv),product,Path('/bin/bash'),file_hash(Path('/bin/bash')),product,tuple(sorted(inputs.items())),step/'process',(('WORLDSIM_GATE_OBSERVER','1'),('WORLDSIM_GATE_TEST_OUT',str(observer))))
            actual=execute(spec);self.assertEqual(actual.native_exit,0)
            files={str(p):file_hash(p) for p in observer.rglob('*') if p.is_file()}
            dirs={str(p):p.stat().st_mode for p in (observer,*observer.rglob('*')) if p.is_dir()}
            stages.append(Node({'step':4,'arm':arm,'state':'RETURNED','native_exit':actual.native_exit,'pid':actual.pid,
                'argv':list(argv),'cwd':str(product),'environment':environment_for(spec),'process_receipt':str(actual.receipt),
                'process_receipt_sha256':file_hash(actual.receipt),'observer_artifacts':files,'observer_directories':dirs}))
        self.session['arms']=arms
        self.base,self.candidate=stages

    def test_actual_aggregate_subset_never_grants_full_coverage(self) -> None:
        # This fixture owns fixed-family evidence; appearance and ASCII routing have separate tests.
        with patch('generic_whole_gate_workspace.evaluate_family', return_value=accounting()), \
             patch('generic_whole_gate_colony_ffi.evaluate_family', return_value={'status': 'PASS', 'safety': [], 'coverage': []}), \
             patch('generic_whole_gate_baseline_observer.evaluate_family', return_value={'safety': [], 'coverage': []}), \
             patch('generic_whole_gate_knowledge_defaults.evaluate_knowledge_defaults', return_value={'safety': [], 'coverage': []}), \
             patch('generic_whole_gate_appearance.evaluate_appearance',
                   return_value={'status':'INCOMPLETE','safety':[],'coverage':[]}), \
             patch('generic_whole_gate_ascii.evaluate_ascii',
                   return_value={'status':'INCOMPLETE','safety':[],'coverage':[],'unsupported':[]}):
            result=evaluate_family(Node(self.session),self.base,self.candidate)
        self.assertEqual(result['status'],'INCOMPLETE')
        self.assertEqual(len(Node(result['coverage']).array()),106)
        family=collect_family(Node(self.session),self.base,'colony_commands')
        self.assertNotEqual(family.emitted_pid,family.aggregate_pid)

    def test_wrong_source_or_command_rejected(self) -> None:
        row=dict(self.base.mapping());row['argv']=['foreign']
        with self.assertRaises(GateContractError):collect_family(Node(self.session),Node(row),'colony_commands')
        source=Path(Node(self.session['arms']).field('base').field('root').text())/'rust/crates/sim-test/tests/colony_commands.rs'
        source.write_text('foreign')
        with self.assertRaises(GateContractError):collect_family(Node(self.session),self.base,'colony_commands')

    def test_missing_or_mutated_endpoint_rejected(self) -> None:
        files=self.base.field('observer_artifacts').mapping()
        target=next(Path(name) for name in files if 'replay_of_' in name and name.endswith('terminal.json'))
        target.write_text('{}')
        with self.assertRaises(GateContractError):collect_family(Node(self.session),self.base,'colony_commands')

    def test_repeated_native_target_or_failed_outcome_rejected(self) -> None:
        receipt=Path(self.base.field('process_receipt').text())
        log=(receipt.parent/'stdout.log').read_text()
        with self.assertRaises(GateContractError):_native_segment(log+log,'colony_commands')
        with self.assertRaises(GateContractError):_native_segment(log.replace(' ... ok',' ... FAILED',1),'colony_commands')

    def test_candidate_loss_fails_even_with_authentic_new_output_anchor(self) -> None:
        row=dict(self.candidate.mapping());files=dict(self.candidate.field('observer_artifacts').mapping())
        path=next(Path(name) for name in files if 'replay_of_' in name and '_original/' in name and name.endswith('terminal.json'))
        data=json.loads(path.read_text());data['terminal']['knowledge_lost']=[{'tick':2,'node':'Fire','holder':None}];data['terminal']['chronicle_entries']=1
        path.write_text(json.dumps(data));files[str(path)]=file_hash(path);row['observer_artifacts']=files
        with self.assertRaises(GateContractError):evaluate_family(Node(self.session),self.base,Node(row))

    def test_probe_mirror_semantics_reject_foreign_branch_and_unbound_file(self) -> None:
        from generic_whole_gate_native_probes import collect_replays
        row=dict(self.base.mapping());files=dict(self.base.field('observer_artifacts').mapping())
        path=next(Path(name) for name in files if Path(name).name.startswith('probe_pid')
                  and Path(json.loads(Path(name).read_text())['local_output']).name == 'first')
        original=path.read_text();data=json.loads(original)
        data['local_output']=data['local_output'].replace('/first','/foreign')
        path.write_text(json.dumps(data));files[str(path)]=file_hash(path);row['observer_artifacts']=files
        with self.assertRaises(GateContractError):collect_replays(Node(self.session),Node(row))
        path.write_text(original);files[str(path)]=file_hash(path)
        del files[str(path)];row['observer_artifacts']=files
        with self.assertRaises(GateContractError):collect_replays(Node(self.session),Node(row))

    def test_other_source_families_remain_explicitly_incomplete(self) -> None:
        left=dict(self.base.mapping());right=dict(self.candidate.mapping());left['step']=38;right['step']=38
        result=evaluate_family(Node(self.session),Node(left),Node(right))
        self.assertEqual(result['status'],'INCOMPLETE');self.assertEqual(result['coverage'],[])

if __name__=='__main__': unittest.main()
