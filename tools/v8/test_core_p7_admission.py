"""Admission boundaries use injected clocks; never wait or run a benchmark."""
import json
from pathlib import Path
import tempfile
import unittest

from core_p7_admission import AdmissionError, admit


class AdmissionTests(unittest.TestCase):
    def test_high_five_and_fifteen_do_not_exclude_quiet_one(self):
        with tempfile.TemporaryDirectory() as folder:
            loads = admit(Path(folder) / 'log', 'quiet', deadline=100,
                          sample=lambda: (7.999, 80., 80.), clock=lambda: 0.)
            self.assertEqual(loads[0], 7.999)

    def test_exact_eight_waits_one_full_window_and_records_both(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / 'log'
            samples = iter([(8., 2., 3.), (7., 2., 3.)])
            waits = []
            admit(path, 'pair', deadline=10000, sample=lambda: next(samples),
                  clock=lambda: sum(waits), sleep=waits.append)
            self.assertEqual(waits, [900])
            rows = [json.loads(line) for line in path.read_text().splitlines()]
            self.assertEqual([r['status'] for r in rows], ['NOT_RUN', 'ADMITTED'])
            self.assertEqual([r['utc'] for r in rows], [0, 900])

    def test_eight_windows_stop_without_running(self):
        with tempfile.TemporaryDirectory() as folder:
            waits = []
            with self.assertRaisesRegex(AdmissionError, 'eight'):
                admit(Path(folder) / 'log', 'busy', deadline=10000,
                      sample=lambda: (8., 1., 1.), clock=lambda: sum(waits), sleep=waits.append)
            self.assertEqual(waits, [900] * 8)

    def test_deadline_and_invalid_samples_fail_closed(self):
        with tempfile.TemporaryDirectory() as folder:
            for loads, deadline in [((float('nan'), 1., 1.), 10000),
                                    ((-1., 1., 1.), 10000), ((1., 1., 1.), 0),
                                    ((8., 1., 1.), 899)]:
                with self.subTest(loads=loads, deadline=deadline), self.assertRaises(AdmissionError):
                    admit(Path(folder) / 'log', 'bad', deadline=deadline,
                          sample=lambda: loads, clock=lambda: 0.,
                          sleep=lambda _: self.fail('must not sleep'))


if __name__ == '__main__':
    unittest.main()
