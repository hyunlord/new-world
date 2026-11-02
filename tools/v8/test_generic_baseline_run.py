"""Tiny Python producers qualify orchestration only, never native baseline evidence."""
import json
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_baseline_run as component
from generic_baseline_run import prepare, run_registered
from generic_fixture_registry import ExpansionContext, Invocation, expand, load_registry
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document
from generic_process import file_hash
from mux2_gate_inputs import Json
from test_generic_observations import receipt

CHILD = r'''import json,os,sys
from pathlib import Path
a=sys.argv;out=Path(a[a.index('--out')+1]);scene=a[a.index('--scene')+1]
lo,hi=map(int,a[a.index('--seeds')+1].split('..'));days=int(a[a.index('--days')+1]);ticks=days*1440
cfg=json.loads(Path('config.json').read_text());arm='candidate' if os.environ['WORLDSIM_GIT_SHA']=='b'*40 else 'base'
if cfg.get('exit')==arm: sys.exit(9)
twin='--check-determinism' in a
(out/'gate_observations').mkdir()
for seed in range(lo,hi):
 for role in (['primary','determinism_twin'] if twin else ['primary']):
  d=json.loads(Path('observation.json').read_text());d.update(scene=scene,seed=seed,occurrence=role,requested_ticks=ticks,requested_terminal_tick=ticks,terminal_tick=ticks,knowledge_capability={'enabled':False,'source':'resources.rules.colony.knowledge'},knowledge_lost=None)
  if cfg.get('enabled'):d.update(knowledge_capability={'enabled':True,'source':'resources.rules.colony.knowledge'},knowledge_lost=[])
  if cfg.get('loss')==arm:d.update(knowledge_capability={'enabled':True,'source':'resources.rules.colony.knowledge'},knowledge_lost=[{'tick':2,'node':'fire','holder':None}])
  if cfg.get('low_alive')==arm:d['terminal_alive']=1
  if cfg.get('short')==arm:d['terminal_tick']-=1
  if cfg.get('missing_twin')==arm and role=='determinism_twin':continue
  (out/'gate_observations'/f'{scene}_s{seed}_{role}.json').write_text(json.dumps(d))
 (out/f'{scene}_s{seed}.jsonl').write_text(json.dumps({'git_sha':os.environ['WORLDSIM_GIT_SHA'],'seed':seed,'tick':0,'value':cfg.get('raw_'+arm,0)})+'\n')
 (out/f'{scene}_s{seed}_deaths.jsonl').write_text('')
 if '--chronicle-out' in a:
  c=Path(a[a.index('--chronicle-out')+1]);c.mkdir(exist_ok=True)
  for kind in ['chronicle','relations']:(c/f'{kind}_{seed}.json').write_text(json.dumps({'seed':seed,'kinds':[],'entries':[],'value':cfg.get('chronicle_'+arm,0)}))
timing={k:{s:1 for s in ['median','p10','p90','min','max']} for k in ['wall_seconds','ms_per_agent_tick']}
summary={'schema_version':13,'scene':scene,'seeds':{'start':lo,'end_exclusive':hi},'days':{str(d):{} for d in range(1,days+1)},'runs':[{'seed':s,'wall_seconds':1,'ms_per_agent_tick':1} for s in range(lo,hi)],'timing':timing,'determinism_checked':twin,'value':cfg.get('summary_'+arm,0)}
(out/'summary.json').write_text(json.dumps(summary))
if twin and not cfg.get('missing_determinism'):print('DETERMINISM PASS')
if cfg.get('missing_raw')==arm:(out/f'{scene}_s{lo}.jsonl').unlink()
if cfg.get('empty_raw')==arm:(out/f'{scene}_s{lo}.jsonl').write_text('')
if cfg.get('nonzero_retired')==arm:(out/f'{scene}_s{lo}.jsonl').write_text('{"wet_burn_wood":1}\n')
if cfg.get('extra_raw')==arm:(out/'extra.jsonl').write_text('{}\n')
if cfg.get('malformed_chronicle') and '--chronicle-out' in a:
 for p in Path(a[a.index('--chronicle-out')+1]).glob('*.json'):p.write_text('{"entries":null}')
if arm=='candidate' and cfg.get('mutate_prior'):
 for p in out.parents[2].glob('base/**/summary.json'):p.write_text(p.read_text()+' ')
'''


class BaselineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.inputs = self.root / 'inputs'
        self.inputs.mkdir()
        self.config = self.inputs / 'config.json'
        self.config.write_text('{}')
        (self.inputs / 'observation.json').write_text(json.dumps(receipt()))
        removed = Path(__file__).with_name('identity_removed_keys.txt')
        (self.inputs / 'tools/v8').mkdir(parents=True)
        (self.inputs / 'tools/v8/identity_removed_keys.txt').write_bytes(removed.read_bytes())
        self.binary = self.root / 'probe'
        self.binary.write_text('#!' + str(Path(sys.executable).resolve()) + '\n' + CHILD)
        self.binary.chmod(0o755)
        self.registry = load_registry(Path(__file__).with_name('gate_fixtures.json'))

    def plans(self, ids: tuple[str, ...]):
        files = tuple((p.relative_to(self.inputs).as_posix(), file_hash(p)) for p in self.inputs.rglob('*') if p.is_file())
        products = Products(*(ProbeProduct(c * 40, self.binary, file_hash(self.binary), self.inputs, files) for c in ('a', 'b')))
        plans = tuple(p for p in expand(self.registry, ExpansionContext(self.inputs, self.inputs, self.root / 'runs', self.binary, self.binary)) if p.fixture_id in ids)
        return plans, products

    def run_component(self, ids: tuple[str, ...] = ('core.baseline10',)):
        plans, products = self.plans(ids)
        result = run_registered(self.registry, plans, products, self.root / 'result.json')
        return result, document(result.receipt.read_text())

    def test_distinct_real_children_cover_all_baseline_worlds_without_publication(self) -> None:
        result, data = self.run_component()
        self.assertEqual(result.status, 'PASS')
        data.field('publication_authority').equals(False)
        self.assertEqual(len(data.field('pairs').array()[0].field('worlds').array()), 4)

    def test_enabled_knowledge_and_immutable_receipt(self) -> None:
        _ = self.config.write_text('{"enabled":true}')
        result, data = self.run_component()
        self.assertEqual(result.status, 'PASS')
        for world in data.field('pairs').array()[0].field('worlds').array():
            world.field('knowledge_enabled').equals(True)
        with self.assertRaises(GateContractError):
            self.run_component()

    def test_identity_keeps_samples_chronicles_and_twins(self) -> None:
        result, data = self.run_component(('identity.baseline_samples', 'identity.baseline_twins'))
        self.assertEqual(result.status, 'PASS')
        self.assertEqual(sum(len(row.field('worlds').array()) for row in data.field('pairs').array()), 40)

    def test_protected_semantic_and_raw_differences_fail(self) -> None:
        for key in ['summary_candidate', 'raw_candidate']:
            with self.subTest(key=key), tempfile.TemporaryDirectory() as tmp:
                self.root = Path(tmp).resolve()
                self.config.write_text(json.dumps({key: 1}))
                result, _ = self.run_component()
                self.assertEqual(result.status, 'FAIL')

    def test_chronicle_difference_fails_identity(self) -> None:
        self.config.write_text('{"chronicle_candidate":1}')
        result, _ = self.run_component(('identity.baseline_samples', 'identity.baseline_twins'))
        self.assertEqual(result.status, 'FAIL')

    def test_later_child_cannot_rebind_prior_outputs(self) -> None:
        self.config.write_text('{"mutate_prior":true}')
        result, data = self.run_component()
        self.assertEqual(result.status, 'FAIL')
        self.assertTrue(any('artifacts changed' in e.field('error').text() for e in data.field('errors').array()))

    def test_malformed_chronicle_retains_component_failure(self) -> None:
        self.config.write_text('{"malformed_chronicle":true}')
        result, data = self.run_component(('identity.baseline_samples', 'identity.baseline_twins'))
        self.assertEqual(result.status, 'FAIL')
        self.assertTrue(data.field('errors').array())

    def test_terminal_input_and_binary_mutations_retain_failure(self) -> None:
        original = component.identity
        for kind in ('input', 'binary'):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp:
                self.root = Path(tmp).resolve()
                target = self.config if kind == 'input' else self.binary

                def changed(base: Invocation, candidate: Invocation, sha: str, target: Path = target) -> dict[str, Json]:
                    answer = original(base, candidate, sha)
                    _ = target.write_text(target.read_text() + ' ')
                    return answer

                with patch.object(component, 'identity', side_effect=changed):
                    result, data = self.run_component()
                self.assertEqual(result.status, 'FAIL')
                self.assertTrue(any('product inputs or executable changed' in e.field('error').text() for e in data.field('errors').array()))

    def test_short_or_missing_or_low_population_or_failed_producer_rejected(self) -> None:
        for key in ['short', 'low_alive', 'exit', 'extra_raw', 'missing_raw', 'empty_raw', 'nonzero_retired', 'loss']:
            with self.subTest(key=key), tempfile.TemporaryDirectory() as tmp:
                self.root = Path(tmp).resolve()
                self.config.write_text(json.dumps({key: 'candidate', 'enabled': key == 'loss'}))
                result, _ = self.run_component()
                self.assertEqual(result.status, 'FAIL')

    def test_twin_sidecar_and_native_determinism_required(self) -> None:
        for config in [{'missing_twin': 'candidate'}, {'missing_determinism': True}]:
            with self.subTest(config=config), tempfile.TemporaryDirectory() as tmp:
                self.root = Path(tmp).resolve()
                self.config.write_text(json.dumps(config))
                result, _ = self.run_component(('determinism.baseline',))
                self.assertEqual(result.status, 'FAIL')

    def test_weakened_plan_or_incomplete_identity_rejected_before_launch(self) -> None:
        plans, products = self.plans(('core.baseline10',))
        argv = plans[0].argv
        assert argv is not None
        for changed in [plans[:1], (replace(plans[0], seeds=(0,)), plans[1]), (replace(plans[0], argv=(*argv[:-1], '--bad')), plans[1])]:
            with self.subTest(changed=changed), self.assertRaises(GateContractError):
                prepare(self.registry, changed, products)
        plans, products = self.plans(('identity.baseline_samples',))
        with self.assertRaises(GateContractError):
            prepare(self.registry, plans, products)


if __name__ == '__main__':
    unittest.main()
