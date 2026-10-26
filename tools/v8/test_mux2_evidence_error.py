#!/usr/bin/env python3
# How to run: python3 -m unittest discover -s tools/v8 -p test_mux2_evidence_error.py
"""Rejected evidence must survive context-manager traceback handling and retain CLI failure."""
from contextlib import redirect_stderr
import io
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from mux2_gate_inputs import EvidenceError, require
from mux2_gate_lock import FD_ENV, lane
import mux2_gate_receipts as receipts


class EvidenceErrorContextTests(unittest.TestCase):
    def test_rejection_survives_real_lane_exit_and_releases_lock(self) -> None:
        environment = {key: value for key, value in os.environ.items() if key != FD_ENV}
        with tempfile.TemporaryDirectory() as temporary, patch.dict(os.environ, environment, clear=True):
            lock = Path(temporary) / 'test-only.lock'
            error = EvidenceError('stall capture exited 1')
            with self.assertRaises(EvidenceError) as caught:
                with lane(lock):
                    raise error
            self.assertIs(caught.exception, error)
            self.assertEqual(caught.exception.reason, 'stall capture exited 1')
            self.assertEqual(str(caught.exception), 'stall capture exited 1')
            with lane(lock) as descriptor:
                self.assertGreaterEqual(descriptor, 0)

    def test_capture_cli_reports_original_rejection_and_nonzero_exit(self) -> None:
        environment = {key: value for key, value in os.environ.items() if key != FD_ENV}
        with tempfile.TemporaryDirectory() as temporary, patch.dict(os.environ, environment, clear=True):
            lock = Path(temporary) / 'test-only.lock'

            def rejected_capture(root: Path, out: Path, kind: str, godot: Path) -> None:
                with lane(lock):
                    require(False, 'stall capture exited 1')

            arguments = ['mux2_gate_receipts.py', 'capture', '--kind', 'stall',
                         '--out', str(Path(temporary) / 'never-created')]
            stderr = io.StringIO()
            with patch('sys.argv', arguments), patch.object(receipts, 'capture', rejected_capture), redirect_stderr(stderr):
                self.assertEqual(receipts.main(), 1)
            self.assertEqual(stderr.getvalue(), 'MUX2_RECEIPT_FAIL stall capture exited 1\n')
            self.assertFalse((Path(temporary) / 'never-created').exists())


if __name__ == '__main__':
    unittest.main()
