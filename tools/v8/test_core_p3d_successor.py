"""Synthetic rejection controls; never use fresh held-out worlds."""
import copy
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import core_p3d_successor as subject


class BehavioralContractTests(unittest.TestCase):
    def test_only_successful_explicit_default_build_is_accepted(self):
        subject.default_build({'build_features': [], 'exit_code': 0})
        for receipt in ({'exit_code': 0}, {'build_features': ['mind_shadow'], 'exit_code': 0},
                        {'build_features': None, 'exit_code': 0},
                        {'build_features': [], 'exit_code': False},
                        {'build_features': [], 'exit_code': 1}):
            with self.subTest(receipt=receipt), self.assertRaises(subject.freeze.FreezeError):
                subject.default_build(receipt)

    def contract(self):
        names = 'schema_version kind seed_protocol historical_view preserving_parent candidate_commit inventory changes build_receipt binary source_freeze host_contract host_evidence host_manifest calibration safety steward reference_safety capture_receipt heldout'
        data = dict.fromkeys(names.split(), {})
        data.update(schema_version=1, kind='behavior_changing_p3d_successor',
                    seed_protocol='p3d_rethink_v1', calibration={'off': {}, 'on': {}})
        return data

    def test_explicit_protocol_shape(self):
        subject.fresh_contract(self.contract())

    def test_original_protocol_cannot_be_relabelled(self):
        data = self.contract()
        data['seed_protocol'] = 'core_original_v1'
        with self.assertRaises(subject.freeze.FreezeError):
            subject.fresh_contract(data)

    def test_equivalence_claim_rejected(self):
        data = self.contract()
        data['kind'] = 'measured_runtime_successor'
        with self.assertRaises(subject.freeze.FreezeError):
            subject.fresh_contract(data)

    def test_missing_mode_or_extra_exemption_rejected(self):
        for mutate in (lambda d: d['calibration'].pop('off'), lambda d: d.update(skip_parity=True)):
            data = self.contract()
            mutate(data)
            with self.assertRaises(subject.freeze.FreezeError):
                subject.fresh_contract(data)

    def test_real_retained_and_full_counts_required(self):
        subject.parity_counts(dict(compared_person_ticks=7, retained_person_ticks=3,
                                   full_person_ticks=4, mismatches=0))

    def test_observation_only_missing_duplicate_boolean_counts_rejected(self):
        valid = dict(compared_person_ticks=7, retained_person_ticks=3, full_person_ticks=4, mismatches=0)
        for key, value in [('retained_person_ticks', 0), ('compared_person_ticks', 8),
                           ('mismatches', 1), ('mismatches', False), ('full_person_ticks', -1)]:
            data = copy.deepcopy(valid)
            data[key] = value
            with self.subTest(key=key, value=value), self.assertRaises(subject.freeze.FreezeError):
                subject.parity_counts(data)

    def test_uncommitted_parity_tool_fails_before_invocation(self):
        with mock.patch.object(subject, 'committed_tools', return_value={}), mock.patch.object(subject, 'invoke') as invoke:
            with self.assertRaises(subject.freeze.FreezeError):
                subject.host_and_shadow(Path('/tmp'), {'candidate_commit': 'commit'}, 'source')
            invoke.assert_not_called()

    def test_exact_tree_includes_unexpected_generated_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            tools = root / 'tools/v8'
            tools.mkdir(parents=True)
            (tools / 'checker.py').write_text('original')
            before = subject.tree(root, ('tools/v8',))
            (tools / 'checker.pyc').write_bytes(b'extra')
            self.assertNotEqual(before, subject.tree(root, ('tools/v8',)))

    def test_symlink_tree_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'tools/v8').mkdir(parents=True)
            (root / 'outside').write_text('x')
            (root / 'tools/v8/alias').symlink_to(root / 'outside')
            with self.assertRaises(subject.freeze.FreezeError):
                subject.tree(root, ('tools/v8',))

    def test_historical_subprocess_forbids_bytecode(self):
        with mock.patch.object(subject.subprocess, 'run') as run:
            subject.invoke(Path('/tmp'), 'python3', '-B', 'check.py')
            self.assertEqual(run.call_args.kwargs['env']['PYTHONDONTWRITEBYTECODE'], '1')
            self.assertTrue(run.call_args.kwargs['check'])

    def test_fresh_receipt_rejects_old_seed_schema_and_pre_freeze_capture(self):
        frozen = {'frozen_utc': '2026-10-27T03:00:00+00:00'}
        good = {'schema_version': 2, 'seed_protocol': subject.PROTOCOL,
                'policy_freeze_sha256': 'pin', 'started_utc': '2026-10-27T04:00:00+00:00',
                'finished_utc': '2026-10-27T05:00:00+00:00', 'source_sha': 'candidate',
                'binary_sha256': 'binary'}
        data = {'heldout': {}, 'source_freeze': {'sha256': 'pin'}, 'candidate_commit': 'candidate'}
        for delta in ({'schema_version': 1}, {'seed_protocol': 'core_original_v1'},
                      {'started_utc': '2026-10-27T02:00:00+00:00'}, {'binary_sha256': 'wrong'}):
            with self.subTest(delta=delta), mock.patch.object(subject.prior, 'pinned_tree', return_value=Path('/evidence')), mock.patch.object(subject.freeze, 'read_reference', return_value=Path('/freeze')), mock.patch.object(subject.freeze, 'load', side_effect=[frozen, {**good, **delta}]), mock.patch.object(subject, 'invoke') as run:
                with self.assertRaises(subject.freeze.FreezeError):
                    subject.fresh_outcomes(Path('/repo'), data, 'binary')
                run.assert_not_called()




class RawIdentityTests(unittest.TestCase):
    def test_only_git_label_and_declared_empty_fields_may_differ(self):
        import json
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            before, after = root / 'before', root / 'after'
            before.mkdir(); after.mkdir()
            removed = root / 'removed.txt'; removed.write_text('retired # standing exemption\n')
            left = before / 'sample.jsonl'; right = after / 'sample.jsonl'
            left.write_text(json.dumps({'git_sha': 'base', 'alive': 3, 'retired': 0}) + '\n')
            right.write_text(json.dumps({'git_sha': 'candidate', 'alive': 3}) + '\n')
            self.assertEqual(subject.raw_identity(before, removed), subject.raw_identity(after, removed))
            right.write_text(json.dumps({'git_sha': 'candidate', 'alive': 4}) + '\n')
            self.assertNotEqual(subject.raw_identity(before, removed), subject.raw_identity(after, removed))
            right.write_text(json.dumps({'git_sha': 'candidate', 'alive': 3, 'retired': 1}) + '\n')
            with self.assertRaises(subject.freeze.FreezeError): subject.raw_identity(after, removed)

    def test_missing_extra_and_empty_raw_inventory_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            removed = root / 'removed.txt'; removed.write_text('')
            before, after = root / 'before', root / 'after'
            before.mkdir(); after.mkdir()
            with self.assertRaises(subject.freeze.FreezeError): subject.raw_identity(before, removed)
            (before / 'a.jsonl').write_text('{}\n')
            (after / 'a.jsonl').write_text('{}\n')
            (after / 'extra.jsonl').write_text('{}\n')
            self.assertNotEqual(subject.raw_identity(before, removed), subject.raw_identity(after, removed))
            (after / 'a.jsonl').unlink()
            self.assertNotEqual(subject.raw_identity(before, removed), subject.raw_identity(after, removed))



class HeldoutTerminalTests(unittest.TestCase):
    def check_terminal(self, arm=None, code=0, log='DETERMINISM PASS\n', missing_exit=False):
        import json
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            frozen = root / 'freeze.json'
            frozen.write_text(json.dumps({'frozen_utc': '2026-01-01T00:00:00+00:00'}))
            for role in ('mind', 'base'):
                receipt = {'schema_version': 2, 'seed_protocol': subject.PROTOCOL,
                           'policy_freeze_sha256': 'freeze-pin', 'probe_exit': 0,
                           'started_utc': '2026-01-02T00:00:00+00:00',
                           'finished_utc': '2026-01-03T00:00:00+00:00',
                           'source_sha': 'candidate', 'binary_sha256': 'binary'}
                if role == arm:
                    receipt['probe_exit'] = code
                    if missing_exit: receipt.pop('probe_exit')
                (root / (role + '-receipt.json')).write_text(json.dumps(receipt))
                text = log if role == arm else 'DETERMINISM PASS\n'
                if text is not None: (root / (role + '-probe.log')).write_text(text)
            for name in ('outcomes.json', 'why.json'): (root / name).write_text('{}')
            data = {'heldout': {}, 'source_freeze': {'sha256': 'freeze-pin'}, 'candidate_commit': 'candidate'}
            def scorer(repo, *args):
                Path(args[-1]).write_text('{}')
            with mock.patch.object(subject.prior, 'pinned_tree', return_value=root), \
                 mock.patch.object(subject.freeze, 'read_reference', return_value=frozen), \
                 mock.patch.object(subject, 'invoke', side_effect=scorer) as invoke:
                try:
                    subject.fresh_outcomes(root, data, 'binary')
                except (subject.freeze.FreezeError, OSError):
                    invoke.assert_not_called()
                    raise
                self.assertEqual(invoke.call_count, 2)

    def test_successful_both_arms_reach_score_recomputation(self):
        self.check_terminal()

    def test_each_arm_rejects_failed_boolean_missing_or_noninteger_exit(self):
        for arm in ('mind', 'base'):
            for code in (1, -1, False, True, '0', None):
                with self.subTest(arm=arm, code=code), self.assertRaises(subject.freeze.FreezeError):
                    self.check_terminal(arm, code=code)
            with self.subTest(arm=arm, missing=True), self.assertRaises(subject.freeze.FreezeError):
                self.check_terminal(arm, missing_exit=True)

    def test_each_arm_requires_existing_pass_log_without_any_fail(self):
        for arm in ('mind', 'base'):
            for log in (None, '', 'DETERMINISM FAIL\n', 'DETERMINISM PASS\nDETERMINISM FAIL\n'):
                with self.subTest(arm=arm, log=log), self.assertRaises((subject.freeze.FreezeError, OSError)):
                    self.check_terminal(arm, log=log)

if __name__ == '__main__':
    unittest.main()
