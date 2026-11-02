# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: PYTHONPATH=.:ROOT/tools/v8 python3 -B -m unittest test_generic_core_evaluation
"""Actual tiny Python fixture producers; no native build/world or held-out run."""
import json
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from typing import final
from unittest.mock import patch

import generic_core_evaluation as evaluation
import generic_direct_run as direct
import generic_fixture_registry as registry_module
from generic_declarations import Change, Declarations
from generic_fixture_registry import load_registry
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import document
from generic_process import file_hash
from generic_sources import inventory
from mux2_gate_inputs import Json
from test_generic_baseline_run import CHILD
from test_generic_observations import receipt

COLONY = r'''
if scene=='colony':
 (out/'summary.json').write_text(json.dumps({'scene':scene,'days':days,'seeds':[{'seed':s,'days':days,'value':cfg.get(arm,0)} for s in range(lo,hi)]}))
 for p in (out/'gate_observations').glob('*.json'):
  d=json.loads(p.read_text());d.update(knowledge_capability={'enabled':True,'source':'resources.rules.colony.knowledge'},knowledge_lost=[])
  if cfg.get('lost')==arm and 'core.steward_off' in str(out):d['knowledge_lost']=[{'tick':2,'node':'fire','holder':None}]
  p.write_text(json.dumps(d))
'''


@final
class EvaluationTests(unittest.TestCase):
    def __init__(self, methodName: str = "runTest") -> None:
        super().__init__(methodName)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.inputs = self.root/'input';self.inputs.mkdir()
        self.registry = load_registry(Path(registry_module.__file__).with_name('gate_fixtures.json'))
        _ = (self.inputs/'config.json').write_text('{}')
        _ = (self.inputs/'observation.json').write_text(json.dumps(receipt()))
        scripts = self.inputs/'tools/v8';scripts.mkdir(parents=True)
        for name in ('u_script.json','soak_script.json'):_ = (scripts/name).write_text('[]')
        _ = (scripts/'identity_removed_keys.txt').write_bytes(Path(registry_module.__file__).with_name('identity_removed_keys.txt').read_bytes())
        self.binary = self.root/'probe';_ = self.binary.write_text('#!'+str(Path(sys.executable).resolve())+'\n'+CHILD+COLONY);self.binary.chmod(0o755)
        self.declarations = Declarations('b'*40,'a'*40,'publication',tuple(sorted(registry_module.DIRECT_IDS)),('b'*40,'c'*40),(),(),(),'d'*64)

    def produce(self, config: dict[str,Json] | None = None) -> tuple[Products,evaluation.PriorArtifacts]:
        _ = (self.inputs/'config.json').write_text(json.dumps(config or {}))
        files=tuple(inventory(self.inputs,[p.name for p in self.inputs.iterdir()]).items())
        products=Products(*(ProbeProduct(c*40,self.binary,file_hash(self.binary),self.inputs,files) for c in ('a','b')))
        result=direct.run_step(self.registry,products,self.declarations,44,self.root/'prior')
        prior=evaluation.PriorArtifacts(result.receipt,file_hash(result.receipt),tuple(inventory(result.receipt.parent,[p.name for p in result.receipt.parent.iterdir()]).items()))
        return products,prior

    def evaluate(self, products: Products, prior: evaluation.PriorArtifacts) -> evaluation.EvaluationResult:
        with patch('subprocess.Popen',side_effect=AssertionError('evaluation must not launch children')):
            return evaluation.evaluate(self.registry,products,self.declarations,prior,self.root/'evaluation')

    def test_actual_matrix_rechecked_without_execution_or_authority(self) -> None:
        products,prior=self.produce();result=self.evaluate(products,prior);node=document(result.receipt.read_text())
        self.assertEqual(result.status,'PENDING',result.receipt.read_text());node.field('evaluated_scope').equals('PASS')
        self.assertEqual(len(node.field('pairs').array()),9)
        node.field('ordinary_safety').equals('PENDING_SOURCE_BUILD_QUALIFICATION')
        node.field('publication_authority').equals(False)
        node.field('global_coverage').equals('PENDING')
        self.assertEqual(file_hash(result.receipt),result.sha256)

    def test_original_failed_component_never_upgrades(self) -> None:
        products,prior=self.produce({'candidate':1});result=self.evaluate(products,prior)
        self.assertEqual(result.status,'FAIL')
        self.assertIn('original',result.receipt.read_text())

    def test_declared_semantic_change_still_requires_zero_loss(self) -> None:
        changes=tuple(Change(name,'a'*40,'c'*40,'intentional synthetic change','docs/test/CHANGED_FIXTURES.json','b'*40,('behavior.rs',)) for name in direct.step_fixtures(44) if name!='core.baseline10')
        self.declarations=replace(self.declarations,changes=changes)
        products,prior=self.produce({'candidate':1,'lost':'candidate'})
        self.assertEqual(self.evaluate(products,prior).status,'FAIL')

    def test_changed_receipt_or_raw_artifact_rejected(self) -> None:
        products,prior=self.produce();path=prior.receipt.parent/'runs/base/core.steward_off/1/summary.json'
        _ = path.write_text(path.read_text()+' ')
        self.assertEqual(self.evaluate(products,prior).status,'FAIL')

    def test_missing_original_digest_cannot_be_refreshed(self) -> None:
        products,prior=self.produce()
        self.assertEqual(self.evaluate(products,replace(prior,sha256='0'*64)).status,'FAIL')

    def test_caller_supplied_pass_cannot_hide_missing_world(self) -> None:
        products,prior=self.produce();path=prior.receipt.parent/'runs/base/core.steward_off/1/gate_observations/colony_s0_primary.json';path.unlink()
        refreshed=replace(prior,files=tuple(inventory(prior.receipt.parent,[p.name for p in prior.receipt.parent.iterdir()]).items()))
        self.assertEqual(self.evaluate(products,refreshed).status,'FAIL')

    def test_wrong_candidate_binding_rejected(self) -> None:
        products,prior=self.produce();products=replace(products,candidate=replace(products.candidate,source_sha='c'*40))
        self.assertEqual(self.evaluate(products,prior).status,'FAIL')

    def test_declared_changes_pass_semantics_but_still_remain_pending(self) -> None:
        self.declarations=replace(self.declarations,changes=tuple(Change(name,'a'*40,'c'*40,'intentional synthetic change','docs/test/CHANGED_FIXTURES.json','b'*40,('behavior.rs',)) for name in direct.step_fixtures(44) if name!='core.baseline10'))
        products,prior=self.produce({'candidate':1})
        self.assertEqual(self.evaluate(products,prior).status,'PENDING')

    def test_recomputed_safety_rejects_loss_even_if_prior_claims_pass(self) -> None:
        products,prior=self.produce()
        plans=evaluation.plans_for(self.registry,products,prior.receipt.parent)
        left=next(p for p in plans if p.arm=='base' and p.fixture_id=='core.steward_off')
        right=next(p for p in plans if p.arm=='candidate' and p.fixture_id=='core.steward_off')
        path=right.out/'gate_observations/colony_s0_primary.json'
        node=document(path.read_text()).mapping();node['knowledge_lost']=[{'tick':2,'node':'fire','holder':None}]
        _=path.write_text(json.dumps(node))
        # Native process receipt remains original. Even a claimed replacement raw
        # inventory cannot manufacture semantic safety from the mutated sidecar.
        refreshed=replace(prior,files=tuple(inventory(prior.receipt.parent,[p.name for p in prior.receipt.parent.iterdir()]).items()))
        with self.assertRaisesRegex(ValueError,'KnowledgeLost|knowledge|loss|lost'):
            _=evaluation.evaluate_pair(self.registry,products,self.declarations,refreshed,left,right)

    def test_native_argv_is_authenticated_not_just_status(self) -> None:
        products,prior=self.produce();plans=evaluation.plans_for(self.registry,products,prior.receipt.parent)
        plan=plans[0];path=plan.out/'process.json';node=document(path.read_text()).mapping();node['argv']=['invented']
        _=path.write_text(json.dumps(node))
        refreshed=replace(prior,files=tuple(inventory(prior.receipt.parent,[p.name for p in prior.receipt.parent.iterdir()]).items()))
        with self.assertRaisesRegex(ValueError,'argv'):_=evaluation.process(plan,products,refreshed)
