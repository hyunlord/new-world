"""Synthetic parser regressions only; no native execution or qualification evidence."""
import copy
import unittest
from pathlib import Path
from typing import final

from generic_gate import GateContractError
from generic_process import __file__ as process_file
from generic_whole_gate_twins import check_sources, collect, library_scope
from generic_whole_gate_twins_plan import CASES, pair_twins, select_twins
from generic_whole_gate_twins_sources import PINS
from mux2_gate_inputs import Json, Node
from test_generic_observations import receipt


def rows(offset: int = 0) -> tuple[tuple[Path, Node], ...]:
    result: list[tuple[Path, Node]] = []
    for scene, counter, sequences in (('colony', 2, (5, 10)), ('baseline', 4, (2, 15)),
                                       ('colony', 19, (60, 90)), ('baseline', 70, (30, 50))):
        for occurrence, sequence in zip(('primary', 'determinism_twin'), sequences, strict=True):
            pid = 123 + offset
            sequence += offset
            output = f'/private/tmp/gate-observer-{pid}-{counter + offset}'
            observation = receipt()
            observation.update(scene=scene, occurrence=occurrence, requested_ticks=1440,
                               requested_terminal_tick=1440, terminal_tick=1440, terminal_alive=10,
                               knowledge_lost=[] if scene == 'colony' else None,
                               knowledge_capability={'enabled': scene == 'colony',
                                                     'source': 'resources.rules.colony.knowledge'})
            result.append((Path(f'probe_pid{pid}_world{sequence}.json'), Node({
                'schema': 'worldsim_gate_test_observation_v1', 'process_id': pid,
                'world_sequence': sequence, 'thread_name': None, 'local_output': output,
                'local_sidecar': output + f'/gate_observations/{scene}_s0_{occurrence}.json',
                'observation': observation})))
    return tuple(result)


def changed(key: str, value: Json, nested: bool = False) -> tuple[tuple[Path, Node], ...]:
    result = list(rows())
    path, node = result[0]
    data = copy.deepcopy(node.mapping())
    target = Node(data['observation']).mapping() if nested else data
    target[key] = value
    result[0] = path, Node(data)
    return tuple(result)


@final
class TwinTests(unittest.TestCase):
    def test_interleaved_groups_pair_without_counter_or_sequence_equality(self) -> None:
        left = select_twins(tuple(reversed(rows())))
        right = select_twins(rows(1000))
        result = pair_twins(left, right)
        self.assertEqual(len(Node(result).field('coverage').array()), 8)
        self.assertEqual(len(Node(result).field('safety').array()), 8)
        self.assertFalse(result['library_complete'])
        self.assertEqual([row.observation.knowledge_enabled for row in left], [False] * 4 + [True] * 4)

    def test_missing_duplicate_extra_and_foreign_output_groups_reject(self) -> None:
        for value in (rows()[:-1], rows() + (rows()[0],), rows() + rows(9)):
            with self.assertRaises(GateContractError):
                _ = select_twins(value)
        for key, value in (('local_output', '/private/tmp/gate-observer-123-99'),
                           ('local_output', '/private/tmp/gate-observer-123-2/../gate-observer-123-2'),
                           ('local_sidecar', '/foreign/colony_s0_primary.json'), ('process_id', 456),
                           ('thread_name', 'unknown'), ('schema', 'foreign')):
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = select_twins(changed(key, value))

    def test_partial_orders_reject_without_adjacency_rule(self) -> None:
        for first, second in ((0, 1), (0, 4)):
            source = list(rows())
            path_a, node_a = source[first]
            path_b, node_b = source[second]
            a, b = dict(node_a.mapping()), dict(node_b.mapping())
            a['world_sequence'], b['world_sequence'] = b['world_sequence'], a['world_sequence']
            source[first], source[second] = (path_b, Node(a)), (path_a, Node(b))
            with self.assertRaises(GateContractError):
                _ = select_twins(tuple(source))

    def test_wrong_world_or_incomplete_history_rejects(self) -> None:
        defects: tuple[tuple[str, Json], ...] = (
            ('seed', 1), ('scene', 'other'), ('requested_ticks', 2880), ('requested_terminal_tick', 1441),
            ('terminal_tick', 1441), ('initial_tick', 1), ('history_complete', False),
            ('history_source', 'prior_history_unverified'), ('terminal_complete', False),
            ('checkpoints', [{}]), ('occurrence', 'other'),
            ('knowledge_capability', {'enabled': False, 'source': 'resources.rules.colony.knowledge'}))
        for key, value in defects:
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = select_twins(changed(key, value, True))

    def test_colony_full_loss_and_survival_regressions_reject(self) -> None:
        left = select_twins(rows())
        defects: tuple[tuple[str, Json], ...] = (('knowledge_lost', [{'tick': 1, 'node': 'fire', 'holder': None}]),
                                                   ('terminal_alive', 1))
        for key, value in defects:
            with self.subTest(key=key), self.assertRaises(GateContractError):
                _ = pair_twins(left, select_twins(changed(key, value, True)))

    def test_exact_library_outcomes_profile_and_route(self) -> None:
        body = '\n'.join(f'test {name} ... ok' for name in CASES.values())
        raw = ('Running unittests src/lib.rs (/tmp/debug/deps/sim_test-abc)\nrunning 2 tests\n'
               + body + '\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1s\n')
        library_scope(raw, 'debug')
        for invalid in (raw.replace(next(iter(CASES.values())), 'absent'), raw + raw,
                        raw.replace(' ... ok', ' ... FAILED', 1), raw.replace('/debug/', '/release/')):
            with self.assertRaises(ValueError):
                library_scope(invalid, 'debug')
        with self.assertRaises(GateContractError):
            _ = collect(Node({}), Node({'step': 17}))

    def test_source_closure_is_actual_and_missing_pin_rejects(self) -> None:
        root = Path(process_file).resolve().parents[2]
        from generic_process import file_hash
        inputs: dict[str, Json] = dict(PINS)
        inputs.update({p.relative_to(root).as_posix(): file_hash(p)
                       for p in (root / 'rust/crates/sim-test/src').rglob('*.rs')})
        check_sources(Node({'root': str(root), 'source_inputs': inputs}))
        for name in PINS:
            missing = dict(inputs)
            del missing[name]
            with self.subTest(name=name), self.assertRaises(GateContractError):
                check_sources(Node({'root': str(root), 'source_inputs': missing}))
        inputs['rust/crates/sim-test/src/foreign.rs'] = '0' * 64
        with self.assertRaises(GateContractError):
            check_sources(Node({'root': str(root), 'source_inputs': inputs}))

    def test_baseline_null_capability_and_unknown_fields_reject(self) -> None:
        values = list(rows())
        path, node = values[2]
        data = copy.deepcopy(node.mapping())
        Node(data['observation']).mapping()['knowledge_lost'] = []
        values[2] = path, Node(data)
        with self.assertRaises(GateContractError):
            _ = select_twins(tuple(values))
        for nested in (False, True):
            with self.assertRaises(GateContractError):
                _ = select_twins(changed('unrecognized', 1, nested))

    def test_complete_matching_loss_still_rejects(self) -> None:
        value = changed('knowledge_lost', [{'tick': 1, 'node': 'fire', 'holder': None}], True)
        selected = select_twins(value)
        with self.assertRaises(GateContractError):
            _ = pair_twins(selected, selected)
