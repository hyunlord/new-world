"""Admission boundaries; run with python3 -m unittest discover -s tools/art_pipeline -p test_cpu_probe_p13.py."""
import unittest
import json
import os
import tempfile
from pathlib import Path
from unittest.mock import patch
import cpu_probe_p13 as probe
import session_validity_p13 as validity


class ProbeTest(unittest.TestCase):
    def test_reference_uses_median_of_fastest_five(self):
        self.assertEqual(probe.reference_seconds([float(i) for i in range(15, 0, -1)]), 3.0)

    def test_fixed_kernel_is_deterministic(self):
        self.assertEqual(probe.kernel(1), probe.kernel(1))

    def test_inclusive_probe_exclusive_load_boundaries(self):
        sample = probe.Probe(0.0, 2.1, 7.99, "foreign 100%", 1)
        self.assertTrue(validity.valid_snapshot(sample, 2.0))
        self.assertFalse(validity.valid_snapshot(probe.Probe(0, 2.10001, 1, "", 1), 2.0))
        self.assertFalse(validity.valid_snapshot(probe.Probe(0, 2, 8, "", 1), 2.0))

    def test_nan_probe_is_invalid(self):
        self.assertFalse(validity.valid_snapshot(probe.Probe(0, float('nan'), 1, "", 1), 2.0))

    def test_retry_is_refused_inside_thirty_minutes(self):
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory) / 'ledger.json'
            with patch.dict(os.environ, {'P13_ATTEMPT_LEDGER': str(ledger), 'P13_ITEM': 'unit-world'}):
                self.assertTrue(validity.record_attempt(Path('first.json')))
                self.assertFalse(validity.record_attempt(Path('second.json')))
            self.assertEqual(len(json.loads(ledger.read_text())['events']), 2)

    def test_shared_wait_counts_union_once(self):
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory) / 'ledger.json'
            with patch.dict(os.environ, {'P13_ATTEMPT_LEDGER': str(ledger)}):
                self.assertTrue(validity.update_ledger(None, (100, 1900)))
                self.assertTrue(validity.update_ledger(None, (100, 1900)))
                self.assertTrue(validity.update_ledger(None, (1800, 2000)))
            self.assertEqual(json.loads(ledger.read_text())['waiting_seconds'], 1900)

    def test_reference_rejects_changed_kernel(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'reference.json'
            path.write_text(json.dumps({'complete': True, 'kernel_id': 'different'}))
            with self.assertRaises(ValueError):
                validity.load_reference(path)

    def test_valid_evidence_is_recomputed_not_trusted(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'session.json'
            sample = {'wall_seconds': 2, 'load_1m': 1}
            data = {'kernel_id': probe.KERNEL_ID, 'reference_seconds': 2,
                    'start': sample, 'end': dict(sample, wall_seconds=3),
                    'exit_code': 0, 'valid': True}
            path.write_text(json.dumps(data))
            with self.assertRaises(ValueError):
                validity.require_valid(path)


if __name__ == '__main__':
    unittest.main()
