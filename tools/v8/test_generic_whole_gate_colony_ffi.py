"""Full synthetic public joins; these receipts confer no native qualification."""
import hashlib
import re
import tempfile
import unittest
from pathlib import Path

from colony_ffi_fixture import Fixture
from gate_test_summary import SummaryError
from generic_gate import GateContractError
from generic_whole_gate_colony_ffi import TESTS, collect, evaluate_family
from mux2_gate_inputs import Json, Node


class ColonyFFITests(unittest.TestCase):
    def test_two_worlds_and_explicit_intervention_all_original_routes(self) -> None:
        for step in (4, 5, 17):
            with self.subTest(step=step), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary), step)
                result = Node(evaluate_family(Node(run.session), Node(run.stages['base']), Node(run.stages['candidate'])))
                result.field('status').equals('PASS')
                self.assertEqual(len(result.field('safety').array()), 2)
                result.field('coverage').array()[1].field('classification').equals('CONTROLLED_DIRECT_CARRY_INSERT_TICK0')
                self.assertEqual(len(result.field('candidate_commands').array()), 3)
                result.field('candidate_carry').field('count').equals(4)

    def test_source_pin_and_changed_producer_rejected(self) -> None:
        for missing in (True, False):
            with self.subTest(missing=missing), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary))
                if missing:
                    arms = dict(Node(run.session['arms']).mapping())
                    candidate = dict(Node(arms['candidate']).mapping())
                    candidate['source_inputs'] = {}
                    arms['candidate'] = candidate
                    run.session['arms'] = arms
                else:
                    path = run.source / 'rust/crates/sim-test/tests/engine_colony_ffi.rs'
                    _ = path.write_text(path.read_text() + '// changed\n')
                with self.assertRaises((GateContractError, SummaryError)):
                    _ = collect(Node(run.session), Node(run.stages['candidate']))

    def test_wrong_case_profile_missing_oracle_and_duplicate_scope(self) -> None:
        for mutation in ('profile', 'case', 'duplicate'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary))
                path = run.process('candidate').with_name('stdout.log')
                raw = path.read_text()
                if mutation == 'profile':
                    raw = raw.replace('/debug/', '/release/')
                elif mutation == 'case':
                    raw = raw.replace(TESTS[0], 'unknown_case')
                else:
                    raw += raw
                _ = path.write_text(raw)
                run.seal('candidate')
                with self.assertRaises((GateContractError, SummaryError)):
                    _ = collect(Node(run.session), Node(run.stages['candidate']))

    def test_missing_extra_endpoint_and_unbound_bytes_rejected(self) -> None:
        for mutation in ('missing', 'extra', 'stale'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary))
                folder = run.folder('candidate', 1)
                if mutation == 'missing':
                    (folder / 'terminal.json').unlink()
                elif mutation == 'extra':
                    (folder.parent / 'foreign_s0_primary').mkdir()
                    run.seal('candidate')
                else:
                    path = folder / 'terminal.json'
                    _ = path.write_text(path.read_text() + ' ')
                with self.assertRaises((GateContractError, SummaryError)):
                    _ = collect(Node(run.session), Node(run.stages['candidate']))

    def test_wrong_horizon_and_incomplete_history_rejected(self) -> None:
        for mutation in ('tick', 'history'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary))
                path = run.folder('candidate', 0) / 'terminal.json'
                node = run.read(path)
                if mutation == 'tick':
                    terminal = dict(Node(node['terminal']).mapping())
                    terminal['tick'] = 1
                    node['terminal'] = terminal
                    node['completed_ticks'] = 1
                else:
                    node['history_complete'] = False
                run.write(path, node)
                run.seal('candidate')
                with self.assertRaises((GateContractError, SummaryError)):
                    _ = collect(Node(run.session), Node(run.stages['candidate']))

    def test_actual_command_sequence_not_just_count(self) -> None:
        for mutation in ('kind', 'tick', 'rejected', 'count'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary))
                path = run.folder('candidate', 0) / 'terminal.json'
                node = run.read(path)
                terminal = dict(Node(node['terminal']).mapping())
                commands = [row.value for row in Node(terminal['command_log']).array()]
                first = dict(Node(commands[0]).mapping())
                if mutation == 'kind':
                    command = dict(Node(first['command']).mapping())
                    command['cmd'] = 'cancel_area'
                    first['command'] = command
                elif mutation == 'tick':
                    first['tick'] = 1
                elif mutation == 'rejected':
                    first['accepted'] = False
                else:
                    first['count'] = 2
                commands[0] = first
                terminal['command_log'] = commands
                node['terminal'] = terminal
                run.write(path, node)
                run.seal('candidate')
                with self.assertRaises((GateContractError, SummaryError)):
                    _ = collect(Node(run.session), Node(run.stages['candidate']))

    def test_carry_annotation_is_bound_and_exact(self) -> None:
        changes: tuple[tuple[str, Json], ...] = (('selected_row', 1), ('selected_entity_bits', 102),
            ('count', 5), ('item', 'Firewood'), ('jobs', [[0, -1, 0], [0, -1, 0], [0, 0, 5]]))
        for key, value in changes:
            with self.subTest(key=key), tempfile.TemporaryDirectory() as temporary:
                run = Fixture(Path(temporary))
                path = run.folder('candidate', 1) / 'checkpoint_carry_intervention.json'
                node = run.read(path)
                detail = dict(Node(node['detail']).mapping())
                detail[key] = value
                node['detail'] = detail
                run.write(path, node)
                run.seal('candidate')
                with self.assertRaises((GateContractError, SummaryError)):
                    _ = collect(Node(run.session), Node(run.stages['candidate']))

    def test_candidate_recorded_loss_fails_even_at_tick_zero(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Fixture(Path(temporary))
            path = run.folder('candidate', 0) / 'terminal.json'
            node = run.read(path)
            terminal = dict(Node(node['terminal']).mapping())
            terminal['knowledge_lost'] = [{'tick': 0, 'node': 'doorway', 'holder': None}]
            terminal['chronicle_entries'] = 1
            node['terminal'] = terminal
            run.write(path, node)
            run.seal('candidate')
            with self.assertRaisesRegex(GateContractError, 'KnowledgeLost'):
                _ = evaluate_family(Node(run.session), Node(run.stages['base']), Node(run.stages['candidate']))

    def test_original_source_restored_by_removing_only_observer_seams(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Fixture(Path(temporary))
            source = (run.source / 'rust/crates/sim-test/tests/engine_colony_ffi.rs').read_text()
            self.assertEqual(source.count('Watch::begin('), 2)
            self.assertEqual(source.count('watch.finish('), 2)
            source = source.replace('pub mod save_gate_observation;\n', '').replace('use save_gate_observation::Watch;\n', '')
            source = re.sub(r'    let watch = Watch::begin\(.*?\);\n', '', source, flags=re.DOTALL)
            source = re.sub(r'    watch.annotate\(.*?\n    \}\);\n', '', source, flags=re.DOTALL)
            source = source.replace('    watch.finish(&engine, "original_endpoint");\n', '')
            self.assertEqual(hashlib.sha256(re.sub(r'\s+', '', source).encode()).hexdigest(), '0db47ad5f75d219fa684ed73886d36bd9038835167bda73c58178ae906f1dcb9')

    def test_base_loss_is_reference_only_and_missing_intervention_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = Fixture(Path(temporary))
            path = run.folder('base', 0) / 'terminal.json'
            node = run.read(path)
            terminal = dict(Node(node['terminal']).mapping())
            terminal['knowledge_lost'] = [{'tick': 0, 'node': 'doorway', 'holder': None}]
            terminal['chronicle_entries'] = 1
            node['terminal'] = terminal
            run.write(path, node)
            run.seal('base')
            _ = evaluate_family(Node(run.session), Node(run.stages['base']), Node(run.stages['candidate']))
            (run.folder('candidate', 1) / 'checkpoint_carry_intervention.json').unlink()
            run.seal('candidate')
            with self.assertRaisesRegex(GateContractError, 'missing retained intervention'):
                _ = collect(Node(run.session), Node(run.stages['candidate']))


if __name__ == '__main__':
    _ = unittest.main()
