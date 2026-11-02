# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_fixture_run.py
"""Synthetic executable integration, never native simulation evidence."""
import json
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_direct_run as direct
import generic_fixture_registry as registry_module
import generic_fixture_run as component
from generic_declarations import Change, Declarations
from generic_fixture_registry import ExpansionContext, expand, load_registry
from generic_fixture_run import Products, prepare, run_registered
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError
from generic_process import file_hash
from mux2_gate_inputs import Json
from test_generic_observations import receipt


class ComponentTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.inputs = self.root / 'inputs'
        self.inputs.mkdir()
        self.registry = load_registry(Path(registry_module.__file__).with_name('gate_fixtures.json'))
        self.template = self.inputs / 'observation.json'
        self.template.write_text(json.dumps(receipt()))
        (self.inputs / 'tools/v8').mkdir(parents=True)
        (self.inputs / 'tools/v8/u_script.json').write_text('[]')
        self.config = self.inputs / 'config.json'
        self.config.write_text('{}')
        self.binary = self.root / 'probe'
        self.binary.write_text('#!' + str(Path(sys.executable).resolve()) + '\n' + '''
import json,sys,os
from pathlib import Path
args=sys.argv
out=Path(args[args.index('--out')+1]);out.mkdir(exist_ok=True)
cfg=json.loads(Path('config.json').read_text())
arm='candidate' if os.environ['WORLDSIM_GIT_SHA']=='b'*40 else 'base'
if cfg.get('exit')==arm: sys.exit(7)
if cfg.get('fail_arm')==arm and cfg.get('fail_fixture')==out.parent.name: sys.exit(7)
start,end=map(int,args[args.index('--seeds')+1].split('..'))
days=int(args[args.index('--days')+1]);duration=days*1440
rows=[]
folder=out/'gate_observations';folder.mkdir()
for seed in range(start,end):
 rows.append({'seed':seed,'days':days,'value':cfg.get(arm,0),'ledger_violation_ticks':0,'reservation_violation_ticks':0,'m5':{'pressures':[{'kind':'beasts','start_day':12},{'kind':'cold_snap','start_day':20}]}})
 roles=['primary','determinism_twin'] if '--check-determinism' in args else ['primary']
 for role in roles:
  d=json.loads(Path('observation.json').read_text());d.update(seed=seed,occurrence=role,requested_ticks=duration,requested_terminal_tick=duration,terminal_tick=duration)
  (folder/f'colony_s{seed}_{role}.json').write_text(json.dumps(d))
(out/'summary.json').write_text(json.dumps({'scene':'colony','days':days,'seeds':rows}))
if '--commands-out' in args:
 folder=Path(args[args.index('--commands-out')+1]);folder.mkdir()
 (folder/'colony_s5_commands.json').write_text(json.dumps(cfg.get('commands_'+arm,[])))
if '--check-determinism' in args and not cfg.get('no_determinism'): print('DETERMINISM PASS')
if arm=='candidate' and cfg.get('mutate_prior'):
 for path in out.parents[2].glob('base/**/summary.json'):
  path.write_text(path.read_text()+' ')
''')
        self.binary.chmod(0o755)

    def run_component(self, ids=('core.steward_off',), *, change=False, reverse_plans=False, preservation=False):
        if preservation:
            for name in ('soak_script.json', 'm5_u_script.json', 'm55_u_script.json'):
                (self.inputs / 'tools/v8' / name).write_text('[]')
        files=tuple((p.relative_to(self.inputs).as_posix(),file_hash(p)) for p in self.inputs.rglob('*') if p.is_file())
        products=Products(*(ProbeProduct(c*40,self.binary,file_hash(self.binary),self.inputs,files) for c in ('a','b')))
        plans=tuple(p for p in expand(self.registry,ExpansionContext(self.inputs,self.inputs,self.root/'out',self.binary,self.binary)) if p.fixture_id in ids)
        if preservation:
            order=direct.step_fixtures(50)
            plans=tuple(sorted(plans,key=lambda p:(order.index(p.fixture_id),p.arm,p.repeat)))
        if reverse_plans: plans=tuple(reversed(plans))
        declarations=Declarations('b'*40,'a'*40,'publication',tuple(f.id for f in self.registry.fixtures),('b'*40,'c'*40),
            (Change(ids[0],'a'*40,'c'*40,'intentional','docs/pass/CHANGED_FIXTURES.json','b'*40,('behavior.rs',)),) if change else (),(),(),'d'*64)
        runner=component.run_preservation if preservation else run_registered
        result=runner(self.registry,plans,products,declarations,self.root/'result.json')
        return result,json.loads(result.receipt.read_text()),plans,products

    def test_actual_distinct_children_pass_and_never_publish(self):
        result,data,_,_=self.run_component()
        self.assertEqual(result.status,'PASS')
        self.assertEqual(len(data['executions']),2)
        self.assertFalse(data['publication_authority'])
        receipts=[json.loads(Path(r['receipt']).read_text()) for r in data['executions']]
        self.assertNotEqual(receipts[0]['pid'],receipts[1]['pid'])
        self.assertEqual(data['pairs'][0]['worlds'][0]['status'],'PASS')

    def test_matching_native_losses_fail_with_both_observations_retained(self):
        data=receipt();data['knowledge_lost']=[{'tick':100,'node':'doorway','holder':3}]
        self.template.write_text(json.dumps(data))
        result,data,_,_=self.run_component(change=True)
        self.assertEqual(result.status,'FAIL')
        world=data['pairs'][0]['worlds'][0]
        self.assertEqual(world['base']['losses'],world['candidate']['losses'])
        self.assertIn('KnowledgeLost',world['error'])

    def test_semantic_difference_requires_source_bound_declaration(self):
        self.config.write_text('{"candidate":1}')
        result,data,_,_=self.run_component()
        self.assertEqual(result.status,'FAIL')
        self.assertFalse(data['pairs'][0]['semantic_equal'])

    def test_declared_safe_difference_is_admitted(self):
        self.config.write_text('{"candidate":1}')
        result,data,_,_=self.run_component(change=True)
        self.assertEqual(result.status,'PASS')
        self.assertIsNotNone(data['pairs'][0]['difference'])

    def test_replay_binds_successful_same_arm_original_paths(self):
        result,data,_plans,_=self.run_component(('replay.recorded','replay.scripted','replay.generate'))
        self.assertEqual(result.status,'PASS')
        replay=[r for r in data['executions'] if r['generated_input']]
        self.assertEqual(len(replay),2)
        for row in replay:
            dependency=row['generated_input']
            self.assertTrue(dependency['producer'].startswith(row['id'].split('/')[0]+'/'))
            process=json.loads(Path(row['receipt']).read_text())
            self.assertEqual(process['before_external_inputs'][dependency['path']],dependency['sha256'])

    def test_cross_arm_generated_command_difference_requires_declaration(self):
        self.config.write_text('{"commands_candidate":[{"tick":1}]}')
        result, data, _, _ = self.run_component(('replay.generate', 'replay.recorded', 'replay.scripted'))
        self.assertEqual(result.status, 'FAIL')
        pair = next(p for p in data['pairs'] if p['fixture'] == 'replay.recorded')
        self.assertIn('/generated_input', pair['difference'])

    def test_failed_producer_retains_fail_and_no_fake_comparison(self):
        self.config.write_text('{"exit":"candidate"}')
        result,data,_,_=self.run_component()
        self.assertEqual(result.status,'FAIL')
        self.assertEqual(data['executions'][1]['status'],'FAIL')

    def test_determinism_oracle_failure_keeps_all_world_safety(self):
        self.config.write_text('{"no_determinism":true}')
        result,data,_,_=self.run_component(('colony.u10',))
        self.assertEqual(result.status,'FAIL')
        self.assertEqual(len(data['pairs'][0]['worlds']),8)
        self.assertEqual(len(data['pairs'][0]['oracle_errors']),2)

    def test_existing_story_scorer_runs_with_hash_bound_script(self):
        source = Path(registry_module.__file__).resolve().parents[2]
        script = self.inputs / 'docs/v8/m5/m5_score.py'
        script.parent.mkdir(parents=True)
        script.write_bytes((source / 'docs/v8/m5/m5_score.py').read_bytes())
        (self.inputs / 'tools/v8/m5_u_script.json').write_text('[]')
        result, data, _, _ = self.run_component(('story.script48',))
        self.assertEqual(result.status, 'PASS')
        for arm in ('base', 'candidate'):
            oracle = data['pairs'][0][arm + '_oracles'][0]
            process = json.loads(Path(oracle['receipt']).read_text())
            self.assertEqual(process['native_exit'], 0)
            self.assertIn('docs/v8/m5/m5_score.py', process['before_inputs'])

    def test_existing_neighbours_scorer_failure_is_retained(self):
        source = Path(registry_module.__file__).resolve().parents[2]
        script = self.inputs / 'docs/v8/m5_5/m55_score.py'
        script.parent.mkdir(parents=True)
        script.write_bytes((source / 'docs/v8/m5_5/m55_score.py').read_bytes())
        (self.inputs / 'tools/v8/m55_u_script.json').write_text('[]')
        result, data, plans, _ = self.run_component(('neighbours.script48',))
        self.assertEqual(result.status, 'FAIL')
        self.assertEqual(len(data['pairs'][0]['worlds']), 2)
        for plan in plans:
            process = json.loads((plan.out / 'standing-scorer/process.json').read_text())
            self.assertNotEqual(process['native_exit'], 0)

    def test_later_child_cannot_rebind_prior_outputs(self):
        self.config.write_text('{"mutate_prior":true}')
        result, data, _, _ = self.run_component()
        self.assertEqual(result.status, 'FAIL')
        self.assertTrue(any('artifacts changed' in e['error'] for e in data['errors']))

    def test_scorer_cannot_mutate_validated_sidecar(self):
        script = self.inputs / 'docs/v8/m5/m5_score.py'
        script.parent.mkdir(parents=True)
        script.write_text("import sys\nfrom pathlib import Path\np=next((Path(sys.argv[2]).parent/'gate_observations').glob('*.json'))\np.write_text(p.read_text()+' ')\n")
        (self.inputs / 'tools/v8/m5_u_script.json').write_text('[]')
        result, data, _, _ = self.run_component(('story.script48',))
        self.assertEqual(result.status, 'FAIL')
        self.assertTrue(any('artifacts changed' in e['error'] for e in data['errors']))

    def test_terminal_input_and_binary_mutations_retain_failure(self):
        original = component._normalise
        for kind in ('input', 'binary'):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp:
                self.root = Path(tmp).resolve()
                target = self.config if kind == 'input' else self.binary

                def changed(a: Json, b: Json, sha: str | None, target: Path = target) -> tuple[object, object, list[str]]:
                    answer = original(a, b, sha)
                    _ = target.write_text(target.read_text() + ' ')
                    return answer

                with patch.object(component, '_normalise', side_effect=changed):
                    result, data, _, _ = self.run_component()
                self.assertEqual(result.status, 'FAIL')
                self.assertTrue(any('product inputs or executable changed' in e['error'] for e in data['errors']))

    def test_weakened_plan_and_unsupported_fixture_rejected_before_execution(self):
        _,_,plans,products=self.run_component()
        with self.assertRaises(GateContractError): prepare(self.registry,plans[:1],products)
        with self.assertRaises(GateContractError): prepare(self.registry,(replace(plans[0],seeds=(1,)),plans[1]),products)
        unsupported=tuple(p for p in expand(self.registry,ExpansionContext(self.inputs,self.inputs,self.root/'other',self.binary,self.binary)) if p.fixture_id=='identity.baseline_twins')
        with self.assertRaisesRegex(GateContractError,'unsupported'): prepare(self.registry,unsupported,products)


    def test_replay_native_order_is_original_chain_even_with_reversed_input(self):
        result, data, _, _ = self.run_component(component.REPLAY_CHAIN, reverse_plans=True)
        self.assertEqual(result.status, 'PASS')
        for arm in ('base', 'candidate'):
            actual = [row['id'].split('/')[1] for row in data['executions'] if row['id'].startswith(arm + '/')]
            self.assertEqual(actual, list(component.REPLAY_CHAIN))

    def test_failed_generate_blocks_only_its_same_arm_recorded_and_scripted(self):
        self.config.write_text('{"fail_arm":"base","fail_fixture":"replay.generate"}')
        result, data, _, _ = self.run_component(component.REPLAY_CHAIN)
        self.assertEqual(result.status, 'FAIL')
        self.assertEqual([row['id'] for row in data['executions'] if row['id'].startswith('base/')],
                         ['base/replay.generate/1'])
        self.assertEqual([row['id'].split('/')[1] for row in data['executions'] if row['id'].startswith('candidate/')],
                         list(component.REPLAY_CHAIN))
        self.assertTrue(any(row['invocation'] == 'base/replay.recorded/1' for row in data['errors'] if 'invocation' in row))
        self.assertTrue(any(row['invocation'] == 'base/replay.scripted/1' for row in data['errors'] if 'invocation' in row))

    def test_failed_recorded_blocks_only_its_same_arm_scripted(self):
        self.config.write_text('{"fail_arm":"candidate","fail_fixture":"replay.recorded"}')
        result, data, _, _ = self.run_component(component.REPLAY_CHAIN)
        self.assertEqual(result.status, 'FAIL')
        self.assertEqual([row['id'].split('/')[1] for row in data['executions'] if row['id'].startswith('candidate/')],
                         ['replay.generate', 'replay.recorded'])
        self.assertEqual([row['id'].split('/')[1] for row in data['executions'] if row['id'].startswith('base/')],
                         list(component.REPLAY_CHAIN))

    def test_replay_reordering_keeps_unrelated_fixture_slots(self):
        _, _, plans, _ = self.run_component(('replay.generate', 'replay.recorded', 'replay.scripted', 'core.steward_off'))
        shuffled = tuple(reversed(plans))
        ordered = component._execution_order(shuffled)
        for index, plan in enumerate(shuffled):
            if plan.fixture_id not in component.REPLAY_CHAIN:
                self.assertEqual(ordered[index], plan)


    def test_preservation_actual_child_sequence_is_fixture_then_base_candidate(self):
        _, data, plans, _ = self.run_component(direct.step_fixtures(50), preservation=True)
        # Standing story/neighbour source prerequisites are intentionally absent in
        # this synthetic child-order fixture; no whole-component PASS is claimed.
        expected=[f'{arm}/{fixture}/1' for fixture in direct.step_fixtures(50) for arm in ('base','candidate')]
        self.assertEqual([row['id'] for row in data['executions']],expected)
        self.assertEqual([p.id for p in plans],expected)
        self.assertEqual(data['not_run'],[])

    def preservation_failure(self, arm):
        self.config.write_text(json.dumps({'fail_arm':arm,'fail_fixture':'read.determinism'}))
        result,data,plans,_=self.run_component(direct.step_fixtures(50),preservation=True)
        self.assertEqual(result.status,'FAIL')
        count=1 if arm=='base' else 2
        expected=[f'{a}/{fixture}/1' for fixture in direct.step_fixtures(50) for a in ('base','candidate')]
        self.assertEqual([row['id'] for row in data['executions']],expected[:count])
        self.assertEqual(data['executions'][-1]['status'],'FAIL')
        self.assertEqual([row['id'] for row in data['not_run']],expected[count:])
        self.assertTrue(all(row['status']=='NOT_RUN' and row['blocked_by']==expected[count-1] for row in data['not_run']))
        self.assertTrue(all(not p.out.exists() for p in plans[count:]))

    def test_preservation_base_failure_stops_every_later_child(self):
        self.preservation_failure('base')

    def test_preservation_candidate_failure_stops_every_later_child(self):
        self.preservation_failure('candidate')

    def test_development_batch_still_continues_after_first_failed_child(self):
        self.config.write_text('{"fail_arm":"base","fail_fixture":"core.steward_off"}')
        result,data,_,_=self.run_component(('core.steward_off',))
        self.assertEqual(result.status,'FAIL')
        self.assertEqual([row['id'] for row in data['executions']],
                         ['base/core.steward_off/1','candidate/core.steward_off/1'])
        self.assertNotIn('not_run',data)


    def test_preservation_late_failure_never_launches_completed_pair_scorers(self):
        for relative in ('docs/v8/m5/m5_score.py', 'docs/v8/m5_5/m55_score.py'):
            path=self.inputs/relative
            path.parent.mkdir(parents=True,exist_ok=True)
            path.write_text('raise SystemExit(0)\n')
        self.config.write_text('{"fail_arm":"base","fail_fixture":"read.m6a_steward_on72"}')
        with patch.object(component,'execute',wraps=component.execute) as scorer_child:
            result,data,_,_=self.run_component(direct.step_fixtures(50),preservation=True)
        self.assertEqual(result.status,'FAIL')
        self.assertEqual(len(data['executions']),11)
        self.assertEqual(data['not_run'][0]['id'],'candidate/read.m6a_steward_on72/1')
        scorer_child.assert_not_called()
        self.assertEqual(data['scorer_outputs'],{})
        prior=[row for row in data['pairs'] if row['fixture']!='read.m6a_steward_on72']
        self.assertEqual(len(prior),5)
        self.assertTrue(all(row['oracle_status']=='NOT_RUN' and row['worlds'] for row in prior))
        self.assertTrue(data['producer_outputs'])


if __name__ == '__main__':
    unittest.main()
