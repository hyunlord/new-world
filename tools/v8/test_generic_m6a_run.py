"""Synthetic child processes verify orchestration; no native world or perf runs."""
import json
import shutil
import sys
import tempfile
import unittest
from collections.abc import Generator
from contextlib import contextmanager
from dataclasses import dataclass, replace
from pathlib import Path

from generic_declarations import Change, Declarations
from generic_fixture_registry import (
    ExpansionContext,
    Invocation,
    Registry,
    expand,
    load_registry,
)
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document, integer
from generic_m6a_run import FIXED, PACK, SCORERS, SELECTED, prepare, run_registered
from generic_process import file_hash
from mux2_gate_inputs import Json, Node
from test_generic_m6a_observations import chain_report, fixed_report, sidecar

PRODUCER = '''
import json, os, sys
from pathlib import Path
args=sys.argv
cfg=json.loads(Path('config.json').read_text())
arm='candidate' if os.environ['WORLDSIM_GIT_SHA']=='b'*40 else 'base'
fixed=args[1]=='m6a'
companion='--matched-horizon-ticks' in args
if cfg.get('exit') == arm: sys.exit(7)
out=Path(args[args.index('--out')+1]);out.mkdir()
folder=out/'gate_observations';folder.mkdir()
seed=int(args[args.index('--seed')+1]) if fixed else 0
end=103680 if fixed else (int(args[args.index('--matched-horizon-ticks')+1]) if companion else (121 if arm=='candidate' else 120))
if cfg.get('zero') and not fixed and arm=='candidate':end=0
scene='m6a' if fixed else ('m6a-data-only-companion' if companion else 'm6a-data-only')
obs=json.loads(Path('sidecar.json').read_text())
obs.update(scene=scene,seed=seed,requested_ticks=end,requested_terminal_tick=end,terminal_tick=end)
if not fixed and not companion:
 obs.update(schema_version=2,requested_ticks=8640,requested_terminal_tick=8640,event_horizon={'condition':'m6a_food_chain_complete','condition_met':True,'reached_limit':False})
if cfg.get('loss') and arm=='candidate':obs['knowledge_lost']=[{'tick':100,'node':'doorway','holder':None}]
if cfg.get('survival') and arm=='candidate':obs['terminal_alive']=7
if cfg.get('companion_drift') and companion:obs['terminal_tick']+=1
(folder/f'{scene}_s{seed}_primary.json').write_text(json.dumps(obs))
if fixed:
 report=json.loads(Path('fixed.json').read_text());report.update(seed=seed,days=72,end_tick=end,observed_ticks=end,invariant_checks=2*end)
 if cfg.get('difference') and arm=='candidate':report['fingerprint']=99
 if cfg.get('violations') and arm=='candidate':report['violation_count']=1
 (out/'events.jsonl').write_text('')
 (out/'chronicle.json').write_text('{}')
 filename='summary.json'
else:
 report=json.loads(Path('chain.json').read_text());report.update(end_tick=end,eaten_from_store_tick=end)
 report['intake_evidence'][0]['tick']=end;report['intake_evidence'][0]['meal']['tick']=end
 filename='data-only.json'
 if cfg.get('chain_fail') and arm=='base' and not companion:report.update(passed=False,harvested_pieces=0)
 if companion:
  del report['passed']
  report.update(kind='m6a_data_only_companion_v1',workload='fixed_horizon_companion_primary_chain_oracle_required_separately',requested_ticks=end,maximum_ticks=8640,chain_condition_met=True,primary_oracle_satisfied=False,horizon_complete=True)
  filename='data-only-companion.json'
(out/filename).write_text(json.dumps(report))
print('synthetic child only')
'''
SCORER = '''
import hashlib,json,sys
from pathlib import Path
args=sys.argv
receipt=json.loads(Path(args[args.index('--receipt')+1]).read_text())
assert receipt['arm']=='candidate' and receipt['seeds']==[0,1] and receipt['days']==72
assert receipt['product_arm'] in ('base','candidate')
for world in receipt['worlds']:
 for ref in world['files']+[world['log']]:
  assert hashlib.sha256(Path(ref['path']).read_bytes()).hexdigest()==ref['sha256']
cfg=json.loads(Path('config.json').read_text())
status='FAIL' if cfg.get('score_fail') else 'PASS'
Path(args[args.index('--out')+1]).write_text(json.dumps({'status':status}))
sys.exit(status!='PASS')
'''


@dataclass(frozen=True, slots=True)
class Scenario:
    root: Path
    inputs: Path
    binary: Path
    registry: Registry

    def bindings(self, config: dict[str, Json] | None = None) -> tuple[tuple[Invocation, ...], Products]:
        _ = (self.inputs / 'config.json').write_text(json.dumps(config or {}))
        files = tuple((p.relative_to(self.inputs).as_posix(), file_hash(p)) for p in self.inputs.rglob('*') if p.is_file())
        products = Products(*(ProbeProduct(c * 40, self.binary, file_hash(self.binary), self.inputs, files) for c in ('a', 'b')))
        plans = tuple(p for p in expand(self.registry, ExpansionContext(self.inputs, self.inputs, self.root / 'out', self.binary, self.binary))
                      if p.fixture_id in {FIXED, SELECTED})
        return plans, products

    def run(self, config: dict[str, Json] | None = None, *, declared: bool = False) -> Node:
        plans, products = self.bindings(config)
        changes = (Change(FIXED, 'a' * 40, 'c' * 40, 'intentional behavior', 'docs/pass/CHANGED_FIXTURES.json', 'b' * 40, ('behavior.rs',)),) if declared else ()
        declarations = Declarations('b' * 40, 'a' * 40, 'publication', (FIXED, SELECTED), ('b' * 40, 'c' * 40), changes, (), (), 'd' * 64)
        result = run_registered(self.registry, plans, products, declarations, self.root / 'component.json')
        return document(result.receipt.read_text())


@contextmanager
def scenario() -> Generator[Scenario]:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary).resolve()
        inputs = root / 'inputs'
        inputs.mkdir()
        for name, value in (('sidecar.json', sidecar()), ('fixed.json', fixed_report()), ('chain.json', chain_report())):
            _ = (inputs / name).write_text(json.dumps(value))
        for name in SCORERS:
            path = inputs / name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_text(SCORER if name == SCORERS[0] else '# synthetic dependency\n')
        for name in ('pack.ron', 'plants.ron', 'items.ron'):
            path = inputs / PACK / name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_text('synthetic pack input')
        binary = root / 'probe'
        _ = binary.write_text('#!' + str(Path(sys.executable).resolve()) + '\n' + PRODUCER)
        binary.chmod(0o755)
        yield Scenario(root, inputs, binary, load_registry(Path(__file__).with_name('gate_fixtures.json')))


class M6aRunTests(unittest.TestCase):
    def test_real_children_keep_full_primaries_scorers_and_matched_base(self) -> None:
        with scenario() as state:
            result = state.run()
            self.assertEqual(result.field('status').value, 'PASS', result.value)
            self.assertFalse(result.field('publication_authority').value)
            executions = result.field('executions').array()
            self.assertEqual(len(executions), 7)
            self.assertEqual(len(result.field('scorers').array()), 2)
            processes = [document(Path(row.field('receipt').field('path').text()).read_text()) for row in executions]
            self.assertEqual(len({integer(p.field('pid')) for p in processes}), 7)
            for process in processes[:6]:
                self.assertNotIn('--matched-horizon-ticks', [n.text() for n in process.field('argv').array()])
            argv = [n.text() for n in processes[-1].field('argv').array()]
            self.assertEqual(argv[argv.index('--matched-horizon-ticks') + 1], '121')
            self.assertEqual(processes[-1].field('environment').field('WORLDSIM_GIT_SHA').value, 'a' * 40)

    def test_missing_seed_or_shortened_primary_rejected_before_execution(self) -> None:
        with scenario() as state:
            plans, products = state.bindings()
            with self.assertRaises(GateContractError):
                prepare(state.registry, plans[:-1], products)
            assert plans[0].argv is not None
            short = replace(plans[0], argv=tuple('1' if arg == '72' else arg for arg in plans[0].argv))
            with self.assertRaises(GateContractError):
                prepare(state.registry, (short, *plans[1:]), products)
            self.assertFalse((state.root / 'out').exists())

    def test_primary_failure_retains_other_children_and_cannot_use_companion(self) -> None:
        with scenario() as state:
            result = state.run({'chain_fail': True})
            self.assertEqual(result.field('status').value, 'FAIL')
            self.assertEqual(len(result.field('executions').array()), 6)
            self.assertTrue((state.root / 'out/base/m6a.food_data_only/1/observed/data-only.json').is_file())

    def test_declared_difference_never_waives_safety(self) -> None:
        for failure in ('loss', 'survival', 'violations', 'companion_drift', 'zero'):
            with self.subTest(failure=failure), scenario() as state:
                result = state.run({failure: True, 'difference': True}, declared=True)
                self.assertEqual(result.field('status').value, 'FAIL')

    def test_fixed_difference_requires_declaration(self) -> None:
        for declared in (False, True):
            with self.subTest(declared=declared), scenario() as state:
                result = state.run({'difference': True}, declared=declared)
                self.assertEqual(result.field('status').value, 'PASS' if declared else 'FAIL', result.value)

    def test_scorer_failure_is_retained_and_not_waived(self) -> None:
        with scenario() as state:
            result = state.run({'score_fail': True}, declared=True)
            self.assertEqual(result.field('status').value, 'FAIL')
            self.assertTrue((state.root / 'out/base/m6a.food72/1/standing-scorer/process/process.json').is_file())

    def test_real_original_scorer_rejects_incomplete_synthetic_ledger(self) -> None:
        with scenario() as state:
            for name in SCORERS:
                _ = shutil.copyfile(Path(__file__).parent / Path(name).name, state.inputs / name)
            result = state.run()
            self.assertEqual(result.field('status').value, 'FAIL')
            log = state.root / 'out/base/m6a.food72/1/standing-scorer/process/stderr.log'
            self.assertIn('missing daily snapshots', log.read_text())

    def test_tampered_input_fails_child_and_retains_process_receipt(self) -> None:
        with scenario() as state:
            plans, products = state.bindings()
            _ = (state.inputs / PACK / 'items.ron').write_text('changed')
            declarations = Declarations('b'*40, 'a'*40, 'publication', (FIXED, SELECTED), (), (), (), (), 'd'*64)
            result = run_registered(state.registry, plans, products, declarations, state.root / 'component.json')
            self.assertEqual(result.status, 'FAIL')
            self.assertTrue((plans[0].out / 'process.json').is_file())

    def test_pack_or_scorer_bindings_cannot_be_omitted_or_differ(self) -> None:
        with scenario() as state:
            plans, products = state.bindings()
            for name in (SCORERS[0], PACK + 'pack.ron'):
                altered = replace(products.candidate, input_files=tuple((p, '0'*64 if p == name else h) for p, h in products.candidate.input_files))
                with self.subTest(name=name), self.assertRaises(GateContractError):
                    prepare(state.registry, plans, replace(products, candidate=altered))

    def test_nonzero_primary_keeps_all_six_attempt_receipts(self) -> None:
        with scenario() as state:
            result = state.run({'exit': 'candidate'})
            self.assertEqual(result.field('status').value, 'FAIL')
            self.assertEqual(len(result.field('executions').array()), 6)
            native = document((state.root / 'out/candidate/m6a.food72/1/seed-0/process.json').read_text())
            self.assertEqual(native.field('native_exit').value, 7)
            self.assertEqual(native.field('status').value, 'FAIL')

    def test_existing_derived_output_is_never_reused(self) -> None:
        with scenario() as state:
            target = state.root / 'out/base/m6a.food_data_only/matched-companion'
            target.mkdir(parents=True)
            with self.assertRaises(GateContractError):
                _ = state.run()
            self.assertFalse((state.root / 'component.json').exists())
            self.assertFalse((state.root / 'out/base/m6a.food72').exists())
