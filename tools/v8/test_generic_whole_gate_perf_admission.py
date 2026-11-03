"""Injected-clock/load tests only; no waiting, measurement or native jobs."""
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from core_p7_admission import AdmissionError
from generic_gate import GateContractError, document
from generic_whole_gate_perf_admission import admit_measurement, deadline_epoch
from mux2_gate_inputs import Node


class PerfAdmissionTests(unittest.TestCase):
    def test_finite_positive_deadline(self) -> None:
        self.assertEqual(deadline_epoch(Node(100)), 100.)
        for value in (True, False, 0, -1, float('inf'), float('nan'), '100'):
            with self.subTest(value=value), self.assertRaises(GateContractError):
                _ = deadline_epoch(Node(value))

    def test_high_load_waits_then_exact_threshold_and_timestamp(self) -> None:
        with TemporaryDirectory() as directory:
            log = Path(directory)/'admission.jsonl'
            elapsed: list[float] = []
            samples = iter(((9.42, 9., 9.), (8., 9., 9.), (7.99, 20., 20.)))
            value = admit_measurement(log, 'natural-r1-s19600', 4000., sample=lambda: next(samples),
                                      clock=lambda: sum(elapsed), sleep=elapsed.append)
            self.assertEqual(value, (7.99, 20., 20.))
            self.assertEqual(elapsed, [900., 900.])
            rows = [document(line) for line in log.read_text().splitlines()]
            self.assertEqual([r.field('utc').number() for r in rows], [0, 900, 1800])
            self.assertEqual([r.field('status').text() for r in rows], ['NOT_RUN', 'NOT_RUN', 'ADMITTED'])

    def test_expired_or_insufficient_window_never_sleeps(self) -> None:
        with TemporaryDirectory() as directory:
            for now, load, deadline in ((100., 1., 100.), (100., 9.42, 999.)):
                with self.subTest(now=now, load=load), self.assertRaises(AdmissionError):
                    _ = admit_measurement(Path(directory)/'admission.jsonl', 'blocked', deadline,
                        sample=lambda load=load: (load, 2., 3.), clock=lambda now=now: now,
                        sleep=lambda _: self.fail('must not sleep'))

    def test_deadline_crossing_during_sample_rejects_before_launch(self) -> None:
        with TemporaryDirectory() as directory:
            times = iter((99., 101.))
            with self.assertRaises(AdmissionError):
                _ = admit_measurement(Path(directory)/'admission.jsonl', 'crossing', 100.,
                    sample=lambda: (1., 2., 3.), clock=lambda: next(times),
                    sleep=lambda _: self.fail('must not sleep'))

    def test_invalid_sample_rejects(self) -> None:
        bad: tuple[tuple[float, float, float], ...] = ((True, 1., 1.), (float('nan'), 1., 1.), (-1., 1., 1.))
        for values in bad:
            with self.subTest(values=values), TemporaryDirectory() as directory, self.assertRaises(AdmissionError):
                _ = admit_measurement(Path(directory)/'admission.jsonl', 'invalid', 100.,
                    sample=lambda values=values: values, clock=lambda: 0.,
                    sleep=lambda _: self.fail('must not sleep'))



if __name__ == '__main__':
    _ = unittest.main()
