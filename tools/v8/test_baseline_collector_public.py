"""Public collector disk-fixture regressions, explicitly not a native run."""
import tempfile
import unittest
from pathlib import Path

from baseline_collector_fixture import SyntheticRun, raw_log
from generic_gate import GateContractError
from generic_whole_gate_baseline_observer import CASE, collect, evaluate_family
from mux2_gate_inputs import Json, Node
from test_generic_whole_gate_baseline_observer import fixture


class PublicCollectorTests(unittest.TestCase):
    def test_full_public_pair_debug_and_release(self) -> None:
        for step in (4, 5):
            with self.subTest(step=step), tempfile.TemporaryDirectory() as temporary:
                run = SyntheticRun(Path(temporary), step)
                actual = collect(Node(run.session), Node(run.stages['candidate']))
                self.assertEqual((actual.pid, actual.sequence), (123, 2))
                result = Node(evaluate_family(Node(run.session), Node(run.stages['base']), Node(run.stages['candidate'])))
                result.field('status').equals('PASS')
                result.field('library_complete').equals(False)
                self.assertEqual(len(result.field('safety').array()), 1)
                result.field('safety').array()[0].field('candidate_alive').equals(10)

    def test_wrong_aggregate_scope_and_original_outcome(self) -> None:
        good = raw_log()
        logs = (good.replace('/debug/', '/release/'), good.replace('sim_test-abcd', 'sim_core-abcd'),
                good.replace(CASE, CASE + '_foreign'), good + good,
                good.replace('... ok', '... ignored').replace('1 passed; 0 failed; 0 ignored', '0 passed; 0 failed; 1 ignored'),
                good.replace('... ok', '... FAILED').replace('test result: ok.', 'test result: FAILED.').replace('1 passed; 0 failed', '0 passed; 1 failed'))
        for raw in logs:
            with self.subTest(raw=raw), tempfile.TemporaryDirectory() as temporary:
                run = SyntheticRun(Path(temporary))
                run.write_log('candidate', raw)
                with self.assertRaises(GateContractError):
                    _ = collect(Node(run.session), Node(run.stages['candidate']))

    def test_stale_aggregate_receipt_and_log_hash(self) -> None:
        for filename in ('process.json', 'stdout.log', 'stderr.log'):
            with self.subTest(filename=filename), tempfile.TemporaryDirectory() as temporary:
                run = SyntheticRun(Path(temporary))
                path = run.process('candidate').with_name(filename)
                _ = path.write_text(path.read_text() + ' ')
                with self.assertRaises(GateContractError):
                    _ = collect(Node(run.session), Node(run.stages['candidate']))

    def test_foreign_namespace_and_unbound_mode(self) -> None:
        for foreign in (True, False):
            with self.subTest(foreign=foreign), tempfile.TemporaryDirectory() as temporary:
                run = SyntheticRun(Path(temporary))
                stage = run.stages['candidate']
                if foreign:
                    environment = dict(Node(stage['environment']).mapping())
                    environment['WORLDSIM_GATE_TEST_OUT'] = str(run.observer('base'))
                    stage['environment'] = environment
                    run.receipts['candidate']['environment'] = environment
                    run.seal('candidate')
                else:
                    stage['observer_directories'] = {str(run.observer('candidate')): 0}
                with self.assertRaises(GateContractError):
                    _ = collect(Node(run.session), Node(stage))

    def test_missing_duplicate_unbound_and_stale_mirror(self) -> None:
        for mutation in ('missing', 'duplicate', 'unbound', 'stale'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                run = SyntheticRun(Path(temporary))
                path = run.observer('candidate') / 'probe_pid123_world2.json'
                if mutation == 'missing':
                    path.unlink()
                elif mutation == 'duplicate':
                    node = dict(fixture().mapping())
                    node['world_sequence'] = 3
                    _ = run.write_mirror('candidate', Node(node), 'probe_pid123_world3.json')
                elif mutation == 'unbound':
                    run.stages['candidate']['observer_artifacts'] = {}
                else:
                    _ = path.write_text(path.read_text() + ' ')
                with self.assertRaises(GateContractError):
                    _ = collect(Node(run.session), Node(run.stages['candidate']))

    def test_changed_emitter_thread_horizon_and_source(self) -> None:
        changes: tuple[tuple[str, Json], ...] = (('process_id', 124), ('thread_name', None),
                                               ('terminal_tick', 1), ('source_inputs', {}))
        for key, value in changes:
            with self.subTest(key=key), tempfile.TemporaryDirectory() as temporary:
                run = SyntheticRun(Path(temporary))
                mirror = dict(fixture().mapping())
                if key == 'source_inputs':
                    arms = dict(Node(run.session['arms']).mapping())
                    candidate = dict(Node(arms['candidate']).mapping())
                    candidate['source_inputs'] = value
                    arms['candidate'] = candidate
                    run.session['arms'] = arms
                elif key == 'terminal_tick':
                    observation = dict(Node(mirror['observation']).mapping())
                    observation[key] = value
                    mirror['observation'] = observation
                    _ = run.write_mirror('candidate', Node(mirror))
                else:
                    mirror[key] = value
                    _ = run.write_mirror('candidate', Node(mirror))
                with self.assertRaises(GateContractError):
                    _ = collect(Node(run.session), Node(run.stages['candidate']))

    def test_public_pair_rejects_different_founders(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run = SyntheticRun(Path(temporary))
            mirror = dict(fixture().mapping())
            observation = dict(Node(mirror['observation']).mapping())
            observation.update(initial_population=9, terminal_alive=9)
            mirror['observation'] = observation
            _ = run.write_mirror('candidate', Node(mirror))
            _ = collect(Node(run.session), Node(run.stages['candidate']))
            with self.assertRaisesRegex(GateContractError, 'initial census'):
                _ = evaluate_family(Node(run.session), Node(run.stages['base']), Node(run.stages['candidate']))


if __name__ == '__main__':
    _ = unittest.main()
