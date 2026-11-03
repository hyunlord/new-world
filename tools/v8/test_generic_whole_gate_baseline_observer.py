"""Pure named mirror joins; fixtures are not native execution evidence."""
import tempfile
import unittest
from pathlib import Path

from generic_gate import GateContractError
from generic_whole_gate_baseline_observer import (
    CASE,
    PINS,
    check_sources,
    parse_named,
    select_named,
)
from generic_whole_gate_workspace_plan import __file__ as plan_file
from mux2_gate_inputs import Json, Node
from test_generic_observations import receipt


def fixture() -> Node:
    observation = receipt()
    observation.update(scene='baseline', requested_ticks=0, requested_terminal_tick=0,
                       terminal_tick=0, terminal_alive=10, knowledge_lost=None,
                       knowledge_capability={'enabled': False, 'source': 'resources.rules.colony.knowledge'})
    return Node({'schema': 'worldsim_gate_test_observation_v1', 'process_id': 123,
                 'world_sequence': 2, 'thread_name': CASE, 'local_output': '/tmp/gate-observer-123-9',
                 'local_sidecar': '/tmp/gate-observer-123-9/gate_observations/baseline_s0_primary.json',
                 'observation': observation})


class NamedTests(unittest.TestCase):
    def test_actual_named_shape_accepts_pid_above_one(self) -> None:
        row = parse_named(Path('probe_pid123_world2.json'), fixture())
        self.assertEqual((row.pid, row.sequence, row.observation.terminal_tick), (123, 2, 0))

    def test_missing_duplicate_and_wrong_case_fail(self) -> None:
        for rows in ((), (fixture(), fixture())):
            with self.assertRaises(GateContractError):
                _ = select_named(tuple((Path('probe_pid123_world2.json'), row) for row in rows))
        data: dict[str, Json] = dict(fixture().mapping())
        data['thread_name'] = CASE + '_other'
        with self.assertRaises(GateContractError):
            _ = parse_named(Path('probe_pid123_world2.json'), Node(data))

    def test_foreign_identity_and_path_fail(self) -> None:
        changes: tuple[tuple[str, Json], ...] = (('process_id', 1), ('world_sequence', 7),
            ('local_output', '/tmp/gate-observer-123-9/../foreign'),
            ('local_sidecar', '/tmp/other/gate_observations/baseline_s0_primary.json'))
        for key, value in changes:
            data: dict[str, Json] = dict(fixture().mapping())
            data[key] = value
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = parse_named(Path('probe_pid123_world2.json'), Node(data))

    def test_wrong_horizon_or_incomplete_rejected(self) -> None:
        for key, value in (('terminal_tick', 1), ('terminal_complete', False), ('history_complete', False)):
            data = dict(fixture().mapping())
            observation = dict(Node(data['observation']).mapping())
            observation[key] = value
            data['observation'] = observation
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = parse_named(Path('probe_pid123_world2.json'), Node(data))

    def test_exact_zero_schema_and_capability(self) -> None:
        for key in fixture().field('observation').mapping():
            data: dict[str, Json] = dict(fixture().mapping())
            observation = dict(Node(data['observation']).mapping())
            del observation[key]
            data['observation'] = observation
            with self.subTest(missing=key), self.assertRaises(GateContractError):
                _ = parse_named(Path('probe_pid123_world2.json'), Node(data))
        changes: tuple[tuple[str, Json], ...] = (('seed', True), ('requested_ticks', 1),
            ('scene', 'colony'), ('knowledge_lost', []), ('checkpoints', [{}]),
            ('terminal_alive', 9), ('extra', 0))
        for key, value in changes:
            data = dict(fixture().mapping())
            observation = dict(Node(data['observation']).mapping())
            observation[key] = value
            data['observation'] = observation
            with self.subTest(changed=key), self.assertRaises(GateContractError):
                _ = parse_named(Path('probe_pid123_world2.json'), Node(data))

    def test_source_authority_missing_pin_and_changed_bytes(self) -> None:
        source = Path(plan_file).resolve().parents[2]
        inputs: dict[str, Json] = dict(PINS)
        check_sources(Node({'root': str(source), 'source_inputs': inputs}))
        for relative in PINS:
            missing = dict(inputs)
            del missing[relative]
            with self.subTest(missing=relative), self.assertRaises(GateContractError):
                check_sources(Node({'root': str(source), 'source_inputs': missing}))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in PINS:
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                _ = path.write_bytes((source / relative).read_bytes())
            path = root / 'rust/crates/sim-test/src/probe/gate_observation_tests.rs'
            _ = path.write_bytes(path.read_bytes() + b'// changed source\n')
            with self.assertRaises(GateContractError):
                check_sources(Node({'root': str(root), 'source_inputs': inputs}))


if __name__ == '__main__':
    _ = unittest.main()
