"""Execute the production shell step function with failing admission children."""
from pathlib import Path
import os
import subprocess
import tempfile
import unittest


class GateStopTests(unittest.TestCase):
    def run_step(self, number: int, name: str, code: int, p7: bool):
        source = Path(__file__).with_name('gate.sh').read_text()
        start = source.index('step() {\n')
        function = source[start:source.index('\n}\n', start) + 3]
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            script = function + '\nstatus=0\nstep "$1" "$2" bash -c "exit $3"\ntouch "$LOG/continued"\n'
            env = dict(os.environ, LOG=folder, SUMMARY=str(root / 'gate.txt'))
            env.pop('CORE_P7_RUN', None)
            if p7:
                env['CORE_P7_RUN'] = folder
            result = subprocess.run(['bash', '-c', script, 'test', str(number), name, str(code)],
                                    env=env, capture_output=True, text=True, check=False)
            return result.returncode, (root / 'continued').exists(), (root / 'gate.txt').read_text()

    def test_p7_refused_admission_stops_report_only_steps(self):
        for number, name in [(12, 'perf'), (27, 'nav_bench')]:
            with self.subTest(name=name):
                code, continued, report = self.run_step(number, name, 75, True)
                self.assertEqual(code, 75)
                self.assertFalse(continued)
                self.assertIn('remaining steps NOT_RUN', report)

    def test_p7_nonexempt_bench_failure_stops(self):
        code, continued, _ = self.run_step(25, 'bench', 1, True)
        self.assertEqual(code, 1)
        self.assertFalse(continued)

    def test_legacy_report_only_continuation_is_unchanged(self):
        code, continued, _ = self.run_step(12, 'perf', 75, False)
        self.assertEqual(code, 0)
        self.assertTrue(continued)


if __name__ == '__main__':
    unittest.main()
