"""Pure real-file collector tests; only aggregate lifecycle authentication is a synthetic seam."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from generic_gate import GateContractError, document
from generic_observations import ordinary_pair
from generic_test_jobs_plans import jobs_plan
from generic_whole_gate_native import collect_family
from generic_whole_gate_projection import project_endpoint
from jobs_fixture import Fixture
from mux2_gate_inputs import EvidenceError, Json


class JobsTests(unittest.TestCase):
    def test_public_collector_all_ten_ordinary_worlds_and_histories(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = Fixture(Path(directory))
            with patch('generic_whole_gate_native.aggregate_output', return_value=(fixture.raw(), 987)):
                family = collect_family(fixture.session(), fixture.stage(), 'colony_jobs')
            self.assertEqual(len(family.plans), 10)
            self.assertEqual(len({row.occurrence.test for row in family.plans}), 9)
            self.assertTrue(all(row.occurrence.ordinary for row in family.plans))
            for endpoint, history in zip(family.endpoints, family.histories, strict=True):
                projected = project_endpoint(endpoint, history)
                self.assertEqual(ordinary_pair(projected, projected)['classification'], 'ordinary')
            child = next(row for row in family.plans if row.occurrence.test == 'jobs_children_never_work')
            self.assertEqual(child.occurrence.end_tick, 601)

    def test_missing_repeat_is_not_hidden_by_passing_native_case(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = Fixture(Path(directory))
            (fixture.parent / 'jobs_chop_one_tree_s0_again/terminal.json').unlink()
            with (patch('generic_whole_gate_native.aggregate_output', return_value=(fixture.raw(), 987)),
                  self.assertRaises((GateContractError, EvidenceError))):
                _ = collect_family(fixture.session(), fixture.stage(), 'colony_jobs')

    def test_unregistered_checkpoint_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = Fixture(Path(directory))
            _ = (fixture.parent / 'jobs_stuck_cancel_s0_primary/extra.json').write_text('{}')
            with self.assertRaises(GateContractError):
                _ = jobs_plan(fixture.graph())

    def mutate(self, fixture: Fixture, folder: str, filename: str, changes: dict[str, Json]) -> None:
        path = fixture.parent / folder / filename
        node = document(path.read_text())
        row = node.mapping()
        detail = node.field('detail').mapping()
        detail.update(changes)
        row['detail'] = detail
        Fixture.write(path, row)

    def test_hunger_intervention_and_adult_spawn_are_exact(self) -> None:
        mutations: tuple[tuple[str, str, dict[str, Json]], ...] = (
            ('jobs_cancel_on_hunger_s1_primary', 'checkpoint_after_hunger_intervention.json', {'actual_hunger': 79.0}),
            ('jobs_children_never_work_s0_primary', 'checkpoint_after_adult_spawn.json', {'adult_id': 1}),
            ('jobs_stuck_cancel_s0_primary', 'checkpoint_removed_movement.json', {'movement_rng_absent': False}),
        )
        for folder, filename, changes in mutations:
            with self.subTest(folder=folder), tempfile.TemporaryDirectory() as directory:
                fixture = Fixture(Path(directory))
                self.mutate(fixture, folder, filename, changes)
                with self.assertRaises((GateContractError, EvidenceError)):
                    _ = jobs_plan(fixture.graph())

    def test_source_changed_cannot_use_existing_plan(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = Fixture(Path(directory))
            path = fixture.source / 'rust/crates/sim-test/tests/colony_jobs.rs'
            _ = path.write_text(path.read_text() + '\n// changed\n')
            with (patch('generic_whole_gate_native.aggregate_output', return_value=(fixture.raw(), 987)),
                  self.assertRaises(GateContractError)):
                _ = collect_family(fixture.session(), fixture.stage(), 'colony_jobs')

    def test_failure_outcome_cannot_borrow_complete_mirrors(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = Fixture(Path(directory))
            with (patch('generic_whole_gate_native.aggregate_output', return_value=(fixture.raw().replace('jobs_cancel_on_hunger ... ok', 'jobs_cancel_on_hunger ... FAILED'), 987)),
                  self.assertRaises(GateContractError)):
                _ = collect_family(fixture.session(), fixture.stage(), 'colony_jobs')


if __name__ == '__main__':
    _ = unittest.main()
