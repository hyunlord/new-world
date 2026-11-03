# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_qualification_adversarial
"""Independent-review counterexamples are parser tests, not product evidence."""
import json
import unittest
from copy import deepcopy
from pathlib import Path
from typing import final

import test_generic_whole_gate_qualification_compare as fixtures
from generic_gate import GateContractError, document
from generic_whole_gate_qualification_compare import NAV_TIMING, compare
from mux2_gate_inputs import EvidenceError, Json, Node


@final
class AdversarialTests(unittest.TestCase):
    def rejects(self, kind: str, left: dict[str, Json], right: dict[str, Json]) -> None:
        with self.assertRaises((GateContractError, EvidenceError, ValueError)):
            _ = compare(kind, json.dumps(left), json.dumps(right))

    def test_unknown_slowest_state(self) -> None:
        a = fixtures.ComparatorTests().perf(); b = deepcopy(a)
        b['slowest_ticks'] = [{'tick':1,'display_day':1,'clock':'00:01','total_ms':1,
                              'top_systems':[],'due_periodic':[], 'KnowledgeLost':1}]
        self.rejects('perf',a,b)

    def test_unknown_top_system_state(self) -> None:
        a = fixtures.ComparatorTests().perf(); b = deepcopy(a)
        b['slowest_ticks'] = [{'tick':1,'display_day':1,'clock':'00:01','total_ms':1,
                              'top_systems':[{'name':'S','ms':1,'KnowledgeLost':1}],'due_periodic':[]}]
        self.rejects('perf',a,b)

    def test_scalar_state_object(self) -> None:
        a = fixtures.ComparatorTests().perf(); b = deepcopy(a)
        b['wall_seconds'] = {'KnowledgeLost':1}
        self.rejects('perf',a,b)

    def test_scalar_boolean(self) -> None:
        a = fixtures.ComparatorTests().perf(); b = deepcopy(a)
        b['wall_seconds'] = True
        self.rejects('perf',a,b)

    def test_array_state_object(self) -> None:
        a = fixtures.ComparatorTests().perf(); b = deepcopy(a)
        Node(b).field('per_day').array()[0].mapping()['systems_ms'] = {'KnowledgeLost':1}
        self.rejects('perf',a,b)

    def test_array_element_state_object(self) -> None:
        a = fixtures.ComparatorTests().perf(); b = deepcopy(a)
        Node(b).field('per_day').array()[0].mapping()['systems_ms'] = [{'KnowledgeLost':1}]
        self.rejects('perf',a,b)

    def test_perf_retained_false_is_not_zero(self) -> None:
        a = fixtures.ComparatorTests().perf(); a['future_state'] = {'KnowledgeLost':False}
        b = deepcopy(a); b['future_state'] = {'KnowledgeLost':0}
        self.rejects('perf',a,b)

    def test_nav_retained_true_is_not_one(self) -> None:
        a: dict[str, Json] = {key:1 for key in (*NAV_TIMING,'agents','seed','warmup_ticks','measured_ticks','alive_end','requests','memo_hits','searches','paths_now','nav_version')}
        a.update(within_budget=True, load_before='1.0 1.0 1.0', load_after='1.0 1.0 1.0',future_state={'active':True})
        b = deepcopy(a); b['future_state'] = {'active':1}
        self.rejects('nav',a,b)

    def test_unknown_state_control(self) -> None:
        a = fixtures.ComparatorTests().perf(); b=deepcopy(a); b['future_state']={'KnowledgeLost':1}
        self.rejects('perf',a,b)

    def test_slowest_known_membership_can_change(self) -> None:
        a = fixtures.ComparatorTests().perf(); b=deepcopy(a)
        b['slowest_ticks']=[{'tick':1,'display_day':1,'clock':'00:01','total_ms':1,
                            'top_systems':[{'name':'S','ms':0.5}],'due_periodic':['S']}]
        self.assertIn('/slowest_ticks',compare('perf',json.dumps(a),json.dumps(b)))

    def test_matrix_original_stage11_baseline(self) -> None:
        matrix=document(Path(__file__).with_name('whole_gate_observer_matrix.json').read_text())
        row=next(row for row in matrix.field('rows').array() if row.field('id').text()=='probe_baseline')
        self.assertEqual(row.field('command').text(),'worldsim_probe --scene baseline --seeds 0..4 --days 120 --workers 4 --check-determinism --out {out}')
        self.assertIn('tools/v8/gate.sh',row.field('source_refs').mapping())
        self.assertIn('baseline_s0.jsonl',[item.text() for item in row.field('artifacts').array()])

if __name__ == '__main__':
    _ = unittest.main()
