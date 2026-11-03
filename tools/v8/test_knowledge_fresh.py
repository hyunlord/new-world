"""Synthetic endpoints only, not native execution or qualification."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError, document
from generic_whole_gate_knowledge_fresh import (
    ALL_TESTS,
    TESTS,
    collect,
    evaluate_family,
    occurrences,
)
from knowledge_fresh_fixture import Fixture
from mux2_gate_inputs import Node
from test_colony_jobs_source import tokens


class KnowledgeFreshTests(unittest.TestCase):
    def test_exact43_worlds_and_remaining14(self) -> None:
        self.assertEqual(len(occurrences()),43)
        self.assertEqual([row.end_tick for row in occurrences()],[0]*8+[1]*3+[0]*32)
        self.assertEqual(len(set(ALL_TESTS)-set(TESTS)),14)
        with tempfile.TemporaryDirectory() as directory:
            f=Fixture(Path(directory));candidate=Node({**f.stage().mapping(),'arm':'candidate'})
            with patch('generic_whole_gate_knowledge_fresh.aggregate_output',return_value=(f.raw(),987)):
                result=Node(evaluate_family(f.session(),f.stage(),candidate))
            result.field('status').equals('INCOMPLETE');result.field('target_gap_retained').equals(True)
            self.assertEqual(len(result.field('safety').array()),43)
            self.assertEqual(len(result.field('uncovered_tests').array()),14)

    def test_release_route_and_other_steps_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            f=Fixture(Path(directory));stage=Node({**f.stage().mapping(),'step':5})
            with patch('generic_whole_gate_knowledge_fresh.aggregate_output',return_value=(f.raw().replace('target/debug/','target/release/'),987)):
                self.assertEqual(len(collect(f.session(),stage)),43)
            with self.assertRaises(GateContractError):
                _=collect(f.session(),Node({**f.stage().mapping(),'step':17}))

    def test_missing_seed_and_unbound_source_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            f=Fixture(Path(directory));(f.parent/(TESTS[2]+'_s31_primary')/'terminal.json').unlink()
            with patch('generic_whole_gate_knowledge_fresh.aggregate_output',return_value=(f.raw(),987)),self.assertRaises(ValueError):
                _=collect(f.session(),f.stage())
        with tempfile.TemporaryDirectory() as directory:
            f=Fixture(Path(directory));_ = (f.source/'rust/crates/sim-test/tests/colony_knowledge.rs').write_text('drift')
            with patch('generic_whole_gate_knowledge_fresh.aggregate_output',return_value=(f.raw(),987)),self.assertRaises(ValueError):
                _=collect(f.session(),f.stage())

    def test_original_case_failure_not_bypassed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            f=Fixture(Path(directory))
            for test in (TESTS[0],next(t for t in ALL_TESTS if t not in TESTS)):
                raw=f.raw().replace(test+' ... ok',test+' ... FAILED')
                with patch('generic_whole_gate_knowledge_fresh.aggregate_output',return_value=(raw,987)),self.assertRaises(ValueError):
                    _=collect(f.session(),f.stage())

    def test_recorded_loss_still_fails_ordinary_pair(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            f=Fixture(Path(directory));path=f.parent/(TESTS[1]+'_s0_primary')/'terminal.json'
            node=document(path.read_text());node.field('terminal').mapping().update(chronicle_entries=1,knowledge_lost=[{'tick':1,'node':'oral_history','holder':1}]);f.write(path,node.value)
            with patch('generic_whole_gate_knowledge_fresh.aggregate_output',return_value=(f.raw(),987)),self.assertRaisesRegex(GateContractError,'KnowledgeLost'):
                _=evaluate_family(f.session(),f.stage(),Node({**f.stage().mapping(),'arm':'candidate'}))

    def test_original_tokens_and_unavailable_matrix(self) -> None:
        base=Path(__file__).resolve().parents[2]
        original=(base/'tools/v8/fixtures/colony_knowledge.original.rs').read_text()
        observed=(base/'rust/crates/sim-test/tests/colony_knowledge.rs').read_text()
        observed=observed.replace('pub mod save_gate_observation;','').replace('use save_gate_observation::Watch;','')
        self.assertEqual(tokens(original),tokens(observed))
        matrix=document((base/'tools/v8/whole_gate_observer_matrix.json').read_text())
        row=next(row for row in matrix.field('rows').array() if row.field('id').text()=='native_knowledge_fresh_subset')
        row.field('comparator').equals('UNAVAILABLE');row.field('evidence').equals('NOT_RUN')
