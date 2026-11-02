# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -B -m unittest discover -s tools/v8 -p test_generic_f10_run.py
"""Tiny real children exercise orchestration, never native simulation behavior."""

import json
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from typing import final

import generic_f10_run as runner
from generic_fixture_registry import ExpansionContext, expand, load_registry
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document
from generic_process import file_hash
from test_generic_observations import receipt


@final
class F10RunTests(unittest.TestCase):
    def __init__(self, methodName: str = "runTest") -> None:
        super().__init__(methodName)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.inputs = self.root / "inputs"
        self.inputs.mkdir()
        source = Path(__file__).resolve().parents[2]
        for name in (
            "tools/v8/m4_f10_script.json",
            "tools/v8/gate_f10_no_kill.json",
            "docs/v8/m4/m4_score.py",
        ):
            path = self.inputs / name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_bytes((source / name).read_bytes())
        self.config = self.inputs / "config.json"
        _ = self.config.write_text("{}")
        _ = (self.inputs / "template.json").write_text(json.dumps(receipt()))
        self.binary = self.root / "probe"
        _ = self.binary.write_text(
            "#!"
            + str(Path(sys.executable).resolve())
            + "\n"
            + """
import json,os,sys
from pathlib import Path
from typing import final
args=sys.argv; out=Path(args[args.index('--out')+1]);out.mkdir(exist_ok=True)
script=json.loads(Path(args[args.index('--commands')+1]).read_text())
kills=[r for r in script if r['cmd']=='debug_kill']; injected=bool(kills)
config=json.loads(Path('config.json').read_text())
arm='candidate' if os.environ['WORLDSIM_GIT_SHA']=='b'*40 else 'base'
if config.get('exit')==arm:sys.exit(7)
folder=out/'gate_observations';folder.mkdir()
if injected:(out/'commands').mkdir()
rows=[]
for seed in range(8 if not config.get('missing_seed') else 7):
 d=json.loads(Path('template.json').read_text());d.update(seed=seed,requested_ticks=5760,requested_terminal_tick=5760,terminal_tick=5760)
 d['terminal_alive']=9 if injected else 10
 losses=[[1800,'doorway',9]] if injected else []
 if not injected and config.get('companion_loss'):losses=[[2000,'doorway',9]]
 d['knowledge_lost']=[dict(tick=t,node=n,holder=h) for t,n,h in losses]
 if injected:
  ordered=sorted(script,key=lambda r:r['tick']);index=ordered.index(kills[0])
  c=dict(tick=1800,alive=10,script_index=index,script_spec=kills[0],resolved_command_index=None,debug_kill_agent=None,history_complete=True,knowledge_lost=[])
  if config.get('pre_loss'):c['knowledge_lost']=[dict(tick=100,node='clay_digging',holder=8)];d['knowledge_lost'].insert(0,c['knowledge_lost'][0])
  d['checkpoints']=[dict(c,phase='before_tick_commands'),dict(c,phase='before_debug_kill',resolved_command_index=0,debug_kill_agent=9)]
  if config.get('missing_checkpoint'):d['checkpoints']=[]
  commands=[dict(tick=1800,cmd='debug_kill',agent=9,result=dict(accepted=not config.get('rejected')))]
  (out/'commands'/f'colony_s{seed}_commands.json').write_text(json.dumps(commands))
 m4=dict(lost=losses,node_names=['doorway'],status_end=[4 if injected else 0],waits_end=[[1,0,0,'doorway']] if injected else [])
 if config.get('oracle_fail') and injected:m4['waits_end']=[]
 rows.append(dict(seed=seed,days=4,m4=m4,value=1 if config.get('difference') and arm=='candidate' else 0))
 (folder/f'colony_s{seed}_primary.json').write_text(json.dumps(d))
(out/'summary.json').write_text(json.dumps(dict(scene='colony',days=4,seeds=rows)))
"""
        )
        self.binary.chmod(0o755)
        self.registry = load_registry(source / "tools/v8/gate_fixtures.json")

    def plans_products(self):
        files = tuple(
            sorted(
                (p.relative_to(self.inputs).as_posix(), file_hash(p))
                for p in self.inputs.rglob("*")
                if p.is_file()
            )
        )
        products = Products(
            *(
                ProbeProduct(
                    c * 40, self.binary, file_hash(self.binary), self.inputs, files
                )
                for c in ("a", "b")
            )
        )
        context = ExpansionContext(
            self.inputs,
            self.inputs,
            self.root / "out",
            self.binary,
            self.binary,
            self.inputs / "tools/v8/gate_f10_no_kill.json",
        )
        plans = tuple(
            p
            for p in expand(self.registry, context)
            if p.fixture_id.startswith("knowledge.f10_")
        )
        return plans, products

    def run_component(self):
        plans, products = self.plans_products()
        result = runner.run_registered(
            self.registry, plans, products, self.root / "result.json"
        )
        return result, document(result.receipt.read_text())

    def test_four_children_two_original_oracles_and_no_publication(self):
        result, data = self.run_component()
        self.assertEqual(result.status, "PASS", data)
        self.assertEqual(len(data.field("executions").array()), 4)
        self.assertEqual(len(data.field("worlds").array()), 16)
        self.assertEqual(len(data.field("oracles").array()), 2)
        self.assertFalse(data.field("publication_authority").value)
        self.assertEqual(data.field("registry_status").value, "INCOMPLETE_ADAPTERS")
        pids = [
            document(Path(row.field("receipt").text()).read_text())
            .field("pid")
            .number()
            for row in data.field("executions").array()
        ]
        self.assertEqual(len(set(pids)), 4)
        for row in data.field("oracles").array():
            self.assertEqual(row.field("terminal").value, "F10: 8/8 PASS")

    def test_preflight_rejects_subset_changed_workload_and_roles(self):
        plans, products = self.plans_products()
        assert plans[0].argv is not None
        for changed in (
            plans[:3],
            (replace(plans[0], seeds=(0,)), *plans[1:]),
            (replace(plans[0], roles=("determinism_twin",)), *plans[1:]),
            (replace(plans[0], argv=(*plans[0].argv, "--steward", "on")), *plans[1:]),
        ):
            with self.assertRaises(GateContractError):
                runner.prepare(self.registry, changed, products)
        self.assertFalse((self.root / "out").exists())

    def test_companion_delta_drift_rejected_before_children(self):
        _ = (self.inputs / "tools/v8/gate_f10_no_kill.json").write_text("[]")
        plans, products = self.plans_products()
        with self.assertRaises(GateContractError):
            runner.prepare(self.registry, plans, products)
        self.assertFalse((self.root / "out").exists())

    def test_matching_companion_loss_rejected_with_evidence(self):
        _ = self.config.write_text('{"companion_loss":true}')
        result, data = self.run_component()
        self.assertEqual(result.status, "FAIL")
        self.assertTrue(
            any("KnowledgeLost" in str(row) for row in data.field("worlds").array())
        )
        self.assertEqual(len(data.field("executions").array()), 4)

    def test_missing_checkpoint_and_rejected_kill_rejected(self):
        _ = self.config.write_text('{"missing_checkpoint":true}')
        result, data = self.run_component()
        self.assertEqual(result.status, "FAIL")
        self.assertTrue(data.field("errors").array())

    def test_native_child_failure_retained(self):
        _ = self.config.write_text('{"exit":"candidate"}')
        result, data = self.run_component()
        self.assertEqual(result.status, "FAIL")
        failures = [
            r
            for r in data.field("executions").array()
            if r.field("status").value == "FAIL"
        ]
        self.assertEqual(len(failures), 2)
        self.assertTrue(
            all(
                document(Path(r.field("receipt").text()).read_text())
                .field("native_exit")
                .value
                == 7
                for r in failures
            )
        )

    def test_existing_f10_scorer_zero_exit_fail_text_rejected(self):
        _ = self.config.write_text('{"oracle_fail":true}')
        result, data = self.run_component()
        self.assertEqual(result.status, "FAIL")
        self.assertEqual(len(data.field("oracles").array()), 2)
        self.assertTrue(
            all(
                row.field("status").value == "FAIL"
                for row in data.field("oracles").array()
            )
        )

    def test_undeclared_summary_difference_rejected(self):
        _ = self.config.write_text('{"difference":true}')
        result, data = self.run_component()
        self.assertEqual(result.status, "FAIL")
        self.assertTrue(
            any(
                not row.field("semantic_equal").value
                for row in data.field("comparisons").array()
            )
        )

    def test_missing_world_rejected(self):
        _ = self.config.write_text('{"missing_seed":true}')
        result, _ = self.run_component()
        self.assertEqual(result.status, "FAIL")

    def test_rejected_native_kill_cannot_pass_original_fault_oracle(self):
        _ = self.config.write_text('{"rejected":true}')
        result, data = self.run_component()
        self.assertEqual(result.status, "FAIL")
        self.assertTrue(
            all(
                row.field("status").value == "PASS"
                for row in data.field("oracles").array()
            )
        )
        self.assertTrue(
            any("accepted" in str(row) for row in data.field("worlds").array())
        )

    def test_pre_injection_loss_never_exempted(self):
        _ = self.config.write_text('{"pre_loss":true}')
        result, data = self.run_component()
        self.assertEqual(result.status, "FAIL")
        self.assertTrue(
            any(
                row.field("status").value == "FAIL"
                and row.field("fixture").value == runner.INJECTED
                for row in data.field("worlds").array()
            )
        )

    def test_missing_scorer_inventory_rejected_before_execution(self):
        plans, products = self.plans_products()
        changed = replace(
            products.candidate,
            input_files=tuple(
                row for row in products.candidate.input_files if row[0] != runner.SCORER
            ),
        )
        with self.assertRaises(GateContractError):
            runner.prepare(self.registry, plans, replace(products, candidate=changed))
        self.assertFalse((self.root / "out").exists())

    def test_scorer_mutating_native_command_bytes_fails_postflight(self):
        scorer = self.inputs / runner.SCORER
        original = scorer.read_text()
        mutation = "import sys\nfrom pathlib import Path\np = Path(sys.argv[2]).parent / 'commands/colony_s0_commands.json'\np.write_text(p.read_text() + ' ')\n"
        _ = scorer.write_text(mutation + original)
        result, data = self.run_component()
        self.assertEqual(result.status, "FAIL")
        self.assertTrue(
            all(
                row.field("status").value == "PASS"
                for row in data.field("oracles").array()
            )
        )
        self.assertTrue(
            any("artifacts changed" in str(row) for row in data.field("errors").array())
        )
