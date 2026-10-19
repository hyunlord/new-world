"""Retrospective calibration binding uses real recorded calibration, no CPU probes."""
from __future__ import annotations

import hashlib
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import session_validity_p13 as validity
from cpu_probe_p13 import KERNEL_ID

CANONICAL = Path(__file__).resolve().parents[2] / 'docs/art/production13/perf/calibration.json'


class CalibrationProvenanceTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.reference = self.directory / 'calibration.json'
        self.reference.write_bytes(CANONICAL.read_bytes())
        self.environment = patch.dict(os.environ, {'P13_REFERENCE': str(self.reference)})
        self.environment.start()
        self.addCleanup(self.environment.stop)
        self.seconds = validity.load_reference(self.reference)
        self.sidecar = self.directory / 'validity.json'
        self.data = {
            'kernel_id': KERNEL_ID, 'exit_code': 0,
            'reference_path': str(self.reference.resolve()),
            'reference_sha256': hashlib.sha256(self.reference.read_bytes()).hexdigest(),
            'reference_seconds': self.seconds,
            'start': {'wall_seconds': self.seconds, 'load_1m': 1},
            'end': {'wall_seconds': self.seconds, 'load_1m': 1},
        }

    def check_sidecar(self):
        self.sidecar.write_text(json.dumps(self.data))
        validity.require_valid(self.sidecar)

    def test_accepts_real_reference_with_matching_path_hash_and_recomputed_R(self):
        self.check_sidecar()

    def test_rejects_self_asserted_inflated_reference(self):
        self.data['reference_seconds'] = self.seconds * 2
        self.data['start']['wall_seconds'] = self.seconds * 2
        with self.assertRaisesRegex(ValueError, 'calibration provenance'):
            self.check_sidecar()

    def test_rejects_missing_hash_on_legacy_sidecar(self):
        del self.data['reference_sha256']
        with self.assertRaisesRegex(ValueError, 'calibration provenance'):
            self.check_sidecar()

    def test_rejects_alternate_reference_path_even_with_identical_bytes(self):
        other = self.directory / 'other.json'
        other.write_bytes(self.reference.read_bytes())
        self.data['reference_path'] = str(other)
        with self.assertRaisesRegex(ValueError, 'calibration provenance'):
            self.check_sidecar()

    def test_rejects_changed_reference_bytes_even_if_R_unchanged(self):
        self.reference.write_bytes(self.reference.read_bytes() + b'\n')
        with self.assertRaisesRegex(ValueError, 'calibration provenance'):
            self.check_sidecar()

    def test_retains_probe_threshold_after_provenance_checks(self):
        self.data['end']['wall_seconds'] = self.seconds * 1.051
        with self.assertRaisesRegex(ValueError, 'contended or unsuccessful'):
            self.check_sidecar()

    def test_default_reference_is_repo_canonical_independent_of_cwd(self):
        os.environ.pop('P13_REFERENCE', None)
        calibration = validity.canonical_calibration()
        self.assertEqual(calibration.path, CANONICAL.resolve())
        self.assertEqual(calibration.sha256, hashlib.sha256(CANONICAL.read_bytes()).hexdigest())


if __name__ == '__main__':
    unittest.main()
