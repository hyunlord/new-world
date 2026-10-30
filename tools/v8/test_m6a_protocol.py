"""Legacy and owner-authorized repair protocol boundaries."""
import tempfile
import unittest
from pathlib import Path

from m6a_evidence import EvidenceError, Node
from m6a_run import round_start, register_event
from m6a_gate_source import validate_schema, FIELDS


class RepairProtocolTests(unittest.TestCase):
    def test_repair_register_limits_started_rounds_to_two(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'register.jsonl'
            for number in (1, 2):
                self.assertEqual(round_start(path, Path(folder) / str(number), 'repair_r1'), number)
                register_event(path, {'event': 'finished', 'round': number, 'evaluation_protocol': 'repair_r1'})
            with self.assertRaises(EvidenceError):
                round_start(path, Path(folder) / '3', 'repair_r1')

    def test_register_cannot_mix_protocols(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'register.jsonl'
            round_start(path, Path(folder) / '1')
            register_event(path, {'event': 'finished', 'round': 1})
            with self.assertRaises(EvidenceError):
                round_start(path, Path(folder) / '2', 'repair_r1')

    def test_repair_schema_has_no_successor_proof(self):
        value = dict.fromkeys(FIELDS, None)
        value.update(schema_version=3, kind='m6a_repair_r1_v1',
                     base_head='5b0c88812374b87e7854cc9203f144ad0ccd9009', base_observer={})
        validate_schema(Node(value))
        value['observer_successor'] = {}
        with self.assertRaises(EvidenceError):
            validate_schema(Node(value))



class RepairEvidenceTests(unittest.TestCase):
    def test_base_survival_is_numeric_and_physical_food_is_explicitly_unsupported(self):
        from m6a_score import base_columns
        rows = [Node({'alive_end': 3}), Node({'alive_end': 0})]
        columns = base_columns(rows, [(True, False, True, True), (False, True, True, True)])
        self.assertEqual(columns['alive']['base'], 1)
        self.assertEqual(columns['ledger']['base'], 2)
        self.assertIsNone(columns['cooked_share']['base'])
        self.assertEqual(columns['cooked_share']['base_status'], 'NOT_SUPPORTED')

    def test_base_survival_missing_is_not_silently_zero(self):
        from m6a_score import base_columns
        with self.assertRaises(EvidenceError):
            base_columns([Node({})], [(True, True, True, True)])

class ProtocolBoundaryTests(unittest.TestCase):
    def test_fresh_seed_protocol_and_deadline_are_finite(self):
        from m6a_protocol import protocol
        self.assertEqual(protocol().heldout, list(range(1296, 1328)))
        self.assertEqual(protocol().rounds, 4)
        self.assertEqual(protocol('repair_r1').heldout, list(range(1328, 1360)))
        self.assertEqual(protocol('repair_r1').deadline, 1793377131)
        with self.assertRaises(EvidenceError):
            protocol('anything')

    def test_freeze_protocol_seeds_and_horizon_are_bound(self):
        from m6a_protocol import protocol, check_freeze
        spec = protocol('repair_r1')
        value = {'kind': spec.freeze_kind, 'evaluation_protocol': spec.name,
                 'heldout_seeds': spec.heldout, 'days': 72}
        check_freeze(Node(value), spec)
        for key, bad in [('heldout_seeds', list(range(1296, 1328))), ('days', 48),
                         ('evaluation_protocol', 'legacy'), ('kind', 'm6a_freeze_v1')]:
            with self.subTest(key=key), self.assertRaises(EvidenceError):
                check_freeze(Node(dict(value, **{key: bad})), spec)

class FreshReceiptTests(unittest.TestCase):
    def test_fresh_receipt_rejects_retired_or_incomplete_seeds(self):
        import json
        from m6a_evidence import reference_json
        from m6a_protocol import protocol
        from m6a_score import receipt
        spec = protocol('repair_r1')
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            proof = root / 'proof'
            proof.write_text('fixture')
            ref = reference_json(proof)
            frozen = root / 'freeze.json'
            frozen.write_text(json.dumps({'kind': spec.freeze_kind, 'evaluation_protocol': spec.name,
                                          'heldout_seeds': spec.heldout, 'days': 72}))
            artifacts = []
            for name in ('summary.json', 'events.jsonl', 'chronicle.json'):
                artifact = root / name
                artifact.write_text('{}')
                artifacts.append(reference_json(artifact))
            payload = {'schema_version': 1, 'kind': 'm6a_cohort_v1', 'evaluation_protocol': spec.name,
                       'arm': 'candidate', 'purpose': 'heldout', 'days': 72, 'exit_code': 0,
                       'source_unchanged': True, 'binary': ref, 'observer': ref, 'scorer': ref,
                       'freeze': reference_json(frozen), 'seeds': spec.heldout,
                       'worlds': [{'seed': seed, 'exit_code': 0, 'log': ref, 'files': artifacts}
                                  for seed in spec.heldout]}
            path = root / 'receipt.json'
            path.write_text(json.dumps(payload))
            self.assertEqual(len(receipt(path, 'candidate', 'heldout')), 32)
            payload['worlds'][0]['files'] = []
            path.write_text(json.dumps(payload))
            with self.assertRaisesRegex(EvidenceError, 'required world artifacts'):
                receipt(path, 'candidate', 'heldout')
            payload['worlds'][0]['files'] = artifacts
            for selected in (list(range(1296, 1328)), spec.heldout[:-1]):
                payload['seeds'] = selected
                path.write_text(json.dumps(payload))
                with self.assertRaises(EvidenceError):
                    receipt(path, 'candidate', 'heldout')

    def test_base_runner_rejects_extended_or_expired_r1_deadline_before_io(self):
        from unittest.mock import patch
        from m6a_base72 import run
        from m6a_protocol import R1_DEADLINE
        with patch('m6a_base72.time.time', return_value=R1_DEADLINE - 1):
            with self.assertRaises(EvidenceError):
                run(Path('/absent'), Path('/absent'), Path('/absent'), Path('/absent'),
                    R1_DEADLINE + 1, 'repair_r1')
        with patch('m6a_base72.time.time', return_value=R1_DEADLINE):
            with self.assertRaises(EvidenceError):
                run(Path('/absent'), Path('/absent'), Path('/absent'), Path('/absent'),
                    R1_DEADLINE, 'repair_r1')



class FinalStallIntegrationTests(unittest.TestCase):
    def test_gate_records_no_waiver_on_strict_pass_and_one_flat_waiver_on_use(self):
        from unittest.mock import patch
        from types import SimpleNamespace
        from m6a_gate_evidence import stall_waivers
        from m6a_r1_stall import evaluate
        from mux2_gate_testdata import stall_fixture
        from test_m6a_r1_stall import changed_frames
        gate = SimpleNamespace(root=Path('/repo'), output=Path('/external'))
        rows = tuple((index, 'step', 0) for index in range(1, 49))
        for raw, count in ((stall_fixture(), 0), (changed_frames({0: {2000: 51}}), 1)):
            with patch('m6a_r1_stall.verify', return_value=evaluate(raw)):
                result = stall_waivers(gate, rows)
                self.assertEqual(len(result), count)
                if count:
                    self.assertEqual(result[0]['step'], 41)
                    self.assertNotIn('waivers', result[0])

    def test_gate_other_failure_cannot_authorize_stall(self):
        from unittest.mock import patch
        from types import SimpleNamespace
        from m6a_gate_evidence import stall_waivers
        from m6a_r1_stall import evaluate
        from mux2_gate_testdata import stall_fixture
        from mux2_gate_inputs import EvidenceError as StallError
        gate = SimpleNamespace(root=Path('/repo'), output=Path('/external'))
        with patch('m6a_r1_stall.verify', return_value=evaluate(stall_fixture())):
            with self.assertRaises(StallError):
                stall_waivers(gate, ((40, 'why', 1), (41, 'stall', 0)))



class FreezeCompletionTests(unittest.TestCase):
    def test_freeze_rejects_incomplete_or_tampered_final_calibration(self):
        import argparse
        import json
        from unittest.mock import patch
        from m6a_evidence import reference_json
        from m6a_run import freeze
        for mutation in ('empty', 'horizon', 'changed', 'failed_world', 'tampered_artifact'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as folder:
                root = Path(folder).resolve()
                binary = root / 'binary'
                binary.write_text('synthetic binary')
                ref = reference_json(binary)
                artifact = root / 'summary.json'
                artifact.write_text('{}')
                artifact_ref = reference_json(artifact)
                worlds = [{'seed': seed, 'exit_code': 0, 'log': ref, 'files': [artifact_ref]}
                          for seed in range(8)]
                data = {'schema_version': 1, 'kind': 'm6a_cohort_v1', 'evaluation_protocol': 'repair_r1',
                        'purpose': 'calibration', 'arm': 'candidate', 'days': 72,
                        'source_unchanged': True, 'source_hash': 'source', 'exit_code': 0,
                        'seeds': list(range(8)), 'worlds': worlds, 'binary': ref,
                        'observer': ref, 'scorer': ref}
                if mutation == 'empty': data['worlds'] = []
                if mutation == 'horizon': data['days'] = 48
                if mutation == 'changed': data['source_unchanged'] = False
                if mutation == 'failed_world': worlds[3]['exit_code'] = 1
                if mutation == 'tampered_artifact': artifact.write_text('{"changed":true}')
                receipt_path = root / 'receipt.json'
                receipt_path.write_text(json.dumps(data))
                register = root / 'register.jsonl'
                register_event(register, {'event': 'started', 'round': 1,
                    'evaluation_protocol': 'repair_r1', 'seeds': list(range(8))})
                register_event(register, {'event': 'finished', 'round': 1,
                    'evaluation_protocol': 'repair_r1', 'receipt': reference_json(receipt_path)})
                args = argparse.Namespace(protocol='repair_r1', calibration_register=register,
                    source_root=root, source_head='head', binary=binary, out=root / 'freeze.json')
                with patch('m6a_run.source_hash', return_value='source'), patch('m6a_run.time.time', return_value=1):
                    with self.assertRaises(EvidenceError):
                        freeze(args)
                self.assertFalse(args.out.exists())



class GateCalibrationCompletionTests(unittest.TestCase):
    def test_gate_rejects_empty_successful_calibration_before_heldout(self):
        import json
        from types import SimpleNamespace
        from m6a_evidence import digest, reference_json
        from m6a_gate_evidence import calibration
        from m6a_protocol import protocol
        spec = protocol('repair_r1')
        source = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            binary = root / 'binary'
            binary.write_text('synthetic binary')
            ref = reference_json(binary)
            receipt_path = root / 'receipt.json'
            receipt_path.write_text(json.dumps({'schema_version': 1, 'kind': 'm6a_cohort_v1',
                'evaluation_protocol': spec.name, 'purpose': 'calibration', 'arm': 'candidate',
                'round': 1, 'days': 72, 'source_unchanged': True, 'source_hash': 'source',
                'exit_code': 0, 'seeds': list(range(8)), 'worlds': [], 'binary': ref,
                'observer': ref, 'scorer': ref}))
            register = root / 'register.jsonl'
            register_event(register, {'event': 'started', 'round': 1, 'out': str(root),
                'evaluation_protocol': spec.name, 'seeds': list(range(8))})
            register_event(register, {'event': 'finished', 'round': 1, 'exit_code': 0,
                'evaluation_protocol': spec.name, 'receipt': reference_json(receipt_path)})
            frozen = root / 'freeze.json'
            frozen.write_text(json.dumps({'schema_version': 1, 'kind': spec.freeze_kind,
                'evaluation_protocol': spec.name, 'calibration_register': reference_json(register),
                'source_hash': 'source', 'binary_sha256': digest(binary),
                'scorer_sha256': digest(source / 'tools/v8/m6a_score.py'),
                'days': 72, 'heldout_seeds': spec.heldout}))
            gate = SimpleNamespace(root=source, protocol=lambda: spec, binary=lambda: binary,
                frozen_source_hash=lambda: 'source',
                data=Node({'calibration': {'register': reference_json(register),
                                          'freeze': reference_json(frozen)}}))
            with self.assertRaisesRegex(EvidenceError, 'missing, duplicate or reordered worlds'):
                calibration(gate)


if __name__ == '__main__':
    unittest.main()
