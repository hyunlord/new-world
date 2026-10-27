"""Synthetic raw-proof rejection controls; no runtime binary is launched."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import core_p3d_host_check as h


def decision(retain=False):
    return {'decision': 'continue' if retain else 'evaluate', 'node': 2 if retain else None,
            'reason': None if retain else 'Cadence', 'reads': []}


def row(tick=1, retained=False, evaluation=0, finalized=True):
    return {'schema_version': 1, 'mode': 'off', 'seed': 0, 'tick': tick, 'agent': 1,
            'evaluation': evaluation, 'finalized': finalized, 'port': True,
            'input': {'synthetic': True}, 'actual': decision(retained),
            'dispatch': 'retained' if retained else 'full',
            'full_terminal': None if retained else {'selection': 'success', 'settlement': 'success'}}


def observation(evaluations=2, people=2, ports=2):
    return {'prediction_evaluations': evaluations, 'error_count': 0, 'unmatched_predictions': 0,
            'counts': {'evaluations': evaluations, 'person_ticks': people, 'port_person_ticks': ports,
                       'missing_prediction': 0, 'pending_settlement': 0, 'uncovered': 0}}


class RawParityTests(unittest.TestCase):
    def check_live(self, rows, replay=None, counts=None):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            a, b = root / 'raw.jsonl', root / 'replay.jsonl'
            a.write_text(''.join(json.dumps(value) + '\n' for value in rows))
            b.write_text(''.join(json.dumps(value) + '\n' for value in (rows if replay is None else replay)))
            return h.live_rows(a, b, 'off', 0, observation() if counts is None else counts)

    def test_all_finalized_port_ticks_include_retained(self):
        self.assertEqual(self.check_live([row(1, True), row(2)]), (1, 1))

    def test_replay_evaluations_only_count_last_person_tick(self):
        rows = [row(1, evaluation=0, finalized=False), row(1, True, evaluation=1), row(2)]
        self.assertEqual(self.check_live(rows, counts=observation(3)), (1, 1))

    def test_recorded_prediction_does_not_prove_executed_retention(self):
        rows = [row(1, True), row(2)]
        rows[0]['dispatch'] = 'full'
        with self.assertRaises(h.freeze.FreezeError):
            self.check_live(rows)

    def test_missing_duplicate_out_of_order_and_nonfinal_tick_rejected(self):
        variants = [[row(1)], [row(1), row(1)], [row(2), row(1)],
                    [row(1, finalized=False), row(2)], [row(1), row(2, evaluation=1)]]
        for rows in variants:
            with self.subTest(rows=rows), self.assertRaises(h.freeze.FreezeError):
                self.check_live(rows)

    def test_guard_replay_outcome_mutation_rejected(self):
        rows = [row(1, True), row(2)]
        replay = copy.deepcopy(rows)
        replay[0]['actual']['node'] = 3
        with self.assertRaises(h.freeze.FreezeError):
            self.check_live(rows, replay)

    def test_replay_missing_record_rejected(self):
        with self.assertRaises(h.freeze.FreezeError):
            self.check_live([row(1, True), row(2)], [row(1, True)])

    def test_dropped_port_tick_and_wrong_world_rejected(self):
        for change in ({'port': False}, {'seed': 1}, {'mode': 'on'}, {'tick': 28800}, {'evaluation': False}):
            rows = [row(1, True), row(2)]
            rows[0].update(change)
            with self.subTest(change=change), self.assertRaises(h.freeze.FreezeError):
                self.check_live(rows)

    def test_missing_prediction_and_pending_settlement_rejected(self):
        for key in ('missing_prediction', 'pending_settlement', 'uncovered'):
            counts = observation()
            counts['counts'][key] = 1
            with self.assertRaises(h.freeze.FreezeError):
                self.check_live([row(1, True), row(2)], counts=counts)

    def fixtures(self):
        return [{'schema_version': 1, 'case_id': case, 'input': {'synthetic_primitive': case},
                 'production': decision(case in h.RETAIN_CASES), 'host': decision(case in h.RETAIN_CASES)}
                for case in sorted(h.CASES)]

    def check_fixtures(self, rows, replay=None):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            a, b = root / 'fixtures.jsonl', root / 'replayed.jsonl'
            a.write_text(''.join(json.dumps(value) + '\n' for value in rows))
            b.write_text(''.join(json.dumps(value) + '\n' for value in (rows if replay is None else replay)))
            h.adapter_rows(a, b)

    def test_all_required_primitive_fixture_shapes(self):
        self.check_fixtures(self.fixtures())

    def test_missing_duplicate_and_unrecognized_fixture_rejected(self):
        for rows in (self.fixtures()[:-1], self.fixtures() + self.fixtures()[:1],
                     [{**self.fixtures()[0], 'case_id': 'arbitrary'}] + self.fixtures()[1:]):
            with self.assertRaises(h.freeze.FreezeError):
                self.check_fixtures(rows)

    def test_actual_adapter_difference_and_replay_difference_rejected(self):
        rows = self.fixtures()
        rows[0]['host']['reason'] = 'Different'
        with self.assertRaises(h.freeze.FreezeError):
            self.check_fixtures(rows)
        replay = self.fixtures()
        replay[0]['input'] = {'changed': True}
        with self.assertRaises(h.freeze.FreezeError):
            self.check_fixtures(self.fixtures(), replay)

    def test_duplicate_json_and_nonfinite_rejected(self):
        for source in ('{"a":1,"a":2}', '{"input":{"bad":NaN}}'):
            with self.assertRaises((h.freeze.FreezeError, ValueError)):
                h.json_line(source)

    def test_production_scope_cannot_be_adapter(self):
        contract = {'schema_version': 1, 'candidate_source_sha256': 'source', 'review': 'review', 'parity_tests': ['test'],
                    'shared_files': {'rust/crates/sim-core/src/colony/mind/continuation.rs': 'sha'},
                    'adapters': [{'path': 'rust/crates/sim-core/src/colony/mind/selection.rs',
                                  'before_sha256': None, 'after_sha256': 'sha', 'replacement': '/external', 'reason': 'fake'}]}
        with mock.patch.object(h.freeze, 'digest', return_value='sha'):
            with self.assertRaises(h.freeze.FreezeError):
                h.apply_adapters(Path('/repo'), {'rust/crates/sim-core/src/colony/mind/continuation.rs': 'sha'}, contract)

    def test_capture_nonzero_or_boolean_exit_rejected(self):
        for code in (1, False):
            receipt = {'argv': ['probe'], 'exit_code': code, 'started_utc': '2026-10-27T01:00:00+00:00',
                       'finished_utc': '2026-10-27T01:01:00+00:00', 'log': {}}
            with self.assertRaises(h.freeze.FreezeError):
                h.observed_command(Path('/repo'), receipt, ['probe'])

    def test_replay_process_failure_propagates(self):
        with tempfile.TemporaryDirectory() as temporary:
            log = Path(temporary).resolve() / 'replay.log'
            with mock.patch.object(h.subprocess, 'run', side_effect=h.subprocess.CalledProcessError(1, ['replay'])):
                with self.assertRaises(h.subprocess.CalledProcessError):
                    h.replay_command(Path('/trusted'), [], log)

    def test_wrong_frozen_archive_hash_rejected_before_reading_tar(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary).resolve() / 'wrong.tar.gz'
            path.write_bytes(b'not original archive')
            with self.assertRaises(h.freeze.FreezeError):
                h.archive_inventory(path)

    def test_overlay_does_not_replace_old_host_or_baseline_seam(self):
        for name in ('rust/crates/sim-systems/src/runtime/decision/system.rs',
                     'rust/crates/sim-systems/src/runtime/baseline/system.rs',
                     'rust/crates/sim-systems/src/runtime/jobs/seam.rs',
                     'rust/crates/sim-test/src/bin/worldsim_probe.rs'):
            self.assertFalse(h.overlay(name))
        self.assertTrue(h.overlay('rust/crates/sim-core/src/colony/mind/continuation.rs'))
        self.assertTrue(h.overlay('rust/crates/sim-systems/src/runtime/mind/continuation.rs'))

    def test_shared_gate_current_bytes_must_match(self):
        name = 'rust/crates/sim-core/src/colony/mind/continuation.rs'
        contract = {'schema_version': 1, 'candidate_source_sha256': 'source', 'review': 'review',
                    'parity_tests': ['test'], 'shared_files': {name: 'expected'}, 'adapters': []}
        with mock.patch.object(h.freeze, 'digest', return_value='changed'):
            with self.assertRaises(h.freeze.FreezeError):
                h.apply_adapters(Path('/repo'), {name: 'expected'}, contract)

    def test_boolean_zero_error_counter_rejected(self):
        counts = observation()
        counts['error_count'] = False
        with self.assertRaises(h.freeze.FreezeError):
            self.check_live([row(1, True), row(2)], counts=counts)

    def test_explicit_provider_error_full_fallback_preserved(self):
        rows = [row(1, True), row(2)]
        rows[1]['actual'].update(decision='error', reason='Provider exact failure')
        self.assertEqual(self.check_live(rows), (1, 1))

    def test_error_cannot_be_retained_or_have_missing_terminal(self):
        for terminal in (None, {'selection': 'pending', 'settlement': 'success'},
                         {'selection': 'success', 'settlement': 'failed'}):
            rows = [row(1, True), row(2)]
            rows[1]['actual'].update(decision='error', reason='Provider exact failure')
            rows[1]['full_terminal'] = terminal
            with self.assertRaises(h.freeze.FreezeError):
                self.check_live(rows)
        rows = [row(1, True), row(2)]
        rows[0]['actual'] = {'decision': 'error', 'node': None, 'reason': 'Provider failure', 'reads': []}
        with self.assertRaises(h.freeze.FreezeError):
            self.check_live(rows)

    def test_error_reason_must_replay_exactly(self):
        rows = [row(1, True), row(2)]
        rows[1]['actual'].update(decision='error', reason='Original failure')
        replay = copy.deepcopy(rows)
        replay[1]['actual']['reason'] = 'Different failure'
        with self.assertRaises(h.freeze.FreezeError):
            self.check_live(rows, replay)



class ExternalInputTests(unittest.TestCase):
    def fixture(self, root):
        names = ('assets/ui/content_presentation.json', 'docs/v8/core/SYSTEM_WORLDS.md')
        for name in names:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('synthetic ' + name)
        return {name: h.freeze.digest(root / name) for name in names}

    def test_host_inventory_includes_both_external_inputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            expected = self.fixture(root)
            self.assertEqual(h.live_inventory(root), expected)

    def test_missing_external_input_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            self.fixture(root)
            (root / 'assets/ui/content_presentation.json').unlink()
            with self.assertRaises((h.freeze.FreezeError, OSError)):
                h.live_inventory(root)

    def test_external_inputs_bound_to_commit(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            expected = self.fixture(root)
            with mock.patch.object(h.prior, 'git', side_effect=lambda repo, op, ref: (root / ref.split(':', 1)[1]).read_bytes()):
                self.assertEqual(h.external_inventory(root, 'commit'), expected)

    def test_changed_asset_or_document_rejected(self):
        for name in ('assets/ui/content_presentation.json', 'docs/v8/core/SYSTEM_WORLDS.md'):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                self.fixture(root)
                committed = {path: (root / path).read_bytes() for path in h.EXTERNAL_INPUTS}
                (root / name).write_text('changed')
                with mock.patch.object(h.prior, 'git', side_effect=lambda repo, op, ref: committed[ref.split(':', 1)[1]]):
                    with self.assertRaises(h.freeze.FreezeError):
                        h.external_inventory(root, 'commit')

    def test_prepare_and_build_external_map_must_be_exact(self):
        expected = {name: 'a' * 64 for name in h.EXTERNAL_INPUTS}
        h.external_binding({'external_inputs': expected}, expected)
        variants = [{}, {'external_inputs': {}},
                    {'external_inputs': dict(expected, unexpected='a' * 64)},
                    {'external_inputs': {**expected, h.EXTERNAL_INPUTS[0]: 'b' * 64}}]
        for receipt in variants:
            with self.subTest(receipt=receipt), self.assertRaises(h.freeze.FreezeError):
                h.external_binding(receipt, expected)


if __name__ == '__main__':
    unittest.main()
