"""P7 orchestration regressions; synthetic IO only, no binaries or measurements."""
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch

import core_p7_bench as bench
from core_perf_check import EvidenceError, Json
from test_core_p7_acceptance import fixture


class P7BenchTests(unittest.TestCase):
    def setUp(self) -> None:
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.root = Path(self.stack.enter_context(tempfile.TemporaryDirectory())).resolve()
        self.scope = self.root / 'session'
        self.out = self.scope / 'full-gate/bench'
        self.out.parent.mkdir(parents=True)
        self.head = 'a' * 40
        (self.scope / 'publication-head.txt').write_text(self.head)
        self.gate = self.out.parent / 'gate.txt'
        self.gate.write_text('head ' + self.head + '\n')
        self.quote = 'P7 side64 only conditional exception'
        (self.root / 'CHANGELOG.md').write_text(self.quote)
        self.template = self.root / 'template.json'
        self.template.write_text(json.dumps({'final_head': 'old', 'heldout': 'retained'}))
        self.evaluation = self.root / 'evaluation.json'
        self.config = self.root / 'input.json'
        self.config.write_text('{}')
        self.receipt = fixture()
        self.env = {
            'CORE_P7_RUN': str(self.scope), 'BENCH_JUSTIFIED': self.quote,
            'BENCH_JUSTIFIED_SCENES': 'n200_side64', 'BENCH_BASE': 'M-ART2.1',
            'CORE_P7_PERF_CONFIG': str(self.config),
            'CORE_P7_EVAL_TEMPLATE': str(self.template),
            'CORE_EVAL_CONFIG': str(self.evaluation),
        }
        self.stack.enter_context(patch.dict(os.environ, self.env))
        self.stack.enter_context(patch.object(bench, 'ROOT', self.root))
        self.git = self.stack.enter_context(patch.object(bench, 'git', return_value=self.head))
        self.measure = self.stack.enter_context(patch.object(bench, 'run', side_effect=self.fake_run))
        self.ledger = self.stack.enter_context(
            patch.object(bench.subprocess, 'run', return_value=Mock(returncode=0)))

    def fake_run(self, config: Path, session: Path) -> int:
        self.assertEqual(config, self.config)
        self.assertEqual(session, self.out / 'perf-session')
        session.mkdir()
        for name, value in [('receipt.json', self.receipt), ('config.json', {'fresh': True}),
                            ('new-summary.json', {'role': 'new'}),
                            ('base-summary.json', {'role': 'base'})]:
            (session / name).write_text(json.dumps(value))
        return 1

    def read_adjudication(self) -> Json:
        return json.loads((self.out / 'p7-adjudication.json').read_text())

    def test_other_paired_failure_never_writes_evaluation(self) -> None:
        self.receipt['standing_ratios'][0].update(new=120., ratio=1.2)
        self.assertEqual(bench.main(self.out), 1)
        self.assertFalse(self.evaluation.exists())
        verdict = self.read_adjudication()
        self.assertEqual(verdict['status'], 'FAIL')
        self.assertIn('non-side64', verdict['paired_error'])

    def test_ledger_failure_never_writes_evaluation(self) -> None:
        self.ledger.return_value.returncode = 1
        self.assertEqual(bench.main(self.out), 1)
        self.assertFalse(self.evaluation.exists())
        self.assertEqual(self.read_adjudication()['ledger_exit_code'], 1)
        self.assertEqual(self.read_adjudication()['status'], 'FAIL')

    def test_side64_success_binds_current_head_and_actual_new_session(self) -> None:
        self.assertEqual(bench.main(self.out), 0)
        result = json.loads(self.evaluation.read_text())
        self.assertEqual(result['final_head'], self.head)
        self.assertEqual(result['heldout'], 'retained')
        for field, filename in [('perf_config', 'config.json'), ('perf_receipt', 'receipt.json')]:
            path = self.out / 'perf-session' / filename
            self.assertEqual(result[field], {'path': str(path),
                                            'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                                            'size': path.stat().st_size})
        verdict = self.read_adjudication()
        self.assertEqual(verdict['status'], 'JUSTIFIED')
        self.assertEqual(verdict['raw_failure_labels'], self.receipt['failures'])
        command = self.ledger.call_args.args[0]
        self.assertEqual(command[command.index('--justified-scenes') + 1], 'n200_side64')
        self.assertEqual(command[2:4], ['check', str(self.out / 'perf-session/new-summary.json')])
        self.measure.assert_called_once_with(self.config, self.out / 'perf-session')

    def test_existing_evaluation_is_never_overwritten(self) -> None:
        self.evaluation.write_text('existing evidence')
        with self.assertRaises((EvidenceError, FileExistsError)):
            bench.main(self.out)
        self.assertEqual(self.evaluation.read_text(), 'existing evidence')

    def test_foreign_output_path_refused_before_measurement(self) -> None:
        with self.assertRaisesRegex(EvidenceError, 'belong'):
            bench.main(self.scope / 'other')
        self.measure.assert_not_called()

    def test_existing_session_refused_before_measurement(self) -> None:
        self.out.mkdir()
        with self.assertRaisesRegex(EvidenceError, 'never resumes'):
            bench.main(self.out)
        self.measure.assert_not_called()

    def test_publication_head_mismatch_refused(self) -> None:
        (self.scope / 'publication-head.txt').write_text('other')
        with self.assertRaisesRegex(EvidenceError, 'publication head'):
            bench.main(self.out)
        self.measure.assert_not_called()

    def test_gate_head_mismatch_refused(self) -> None:
        self.gate.write_text('head other\n')
        with self.assertRaisesRegex(EvidenceError, 'gate head'):
            bench.main(self.out)
        self.measure.assert_not_called()

    def test_absent_quote_refused(self) -> None:
        with patch.dict(os.environ, {'BENCH_JUSTIFIED': 'not in changelog'}):
            with self.assertRaisesRegex(EvidenceError, 'justification absent'):
                bench.main(self.out)
        self.measure.assert_not_called()

    def test_other_justified_scenes_refused(self) -> None:
        with patch.dict(os.environ, {'BENCH_JUSTIFIED_SCENES': 'n200_side64,n1000'}):
            with self.assertRaisesRegex(EvidenceError, 'only side64'):
                bench.main(self.out)
        self.measure.assert_not_called()

    def test_head_change_during_measurement_refused(self) -> None:
        self.git.side_effect = [self.head, 'b' * 40]
        with self.assertRaisesRegex(EvidenceError, 'head changed'):
            bench.main(self.out)
        self.assertFalse(self.evaluation.exists())

    def test_incomplete_measurement_propagates_without_ledger_or_evaluation(self) -> None:
        self.measure.side_effect = None
        self.measure.return_value = 75
        self.assertEqual(bench.main(self.out), 75)
        self.ledger.assert_not_called()
        self.assertFalse(self.evaluation.exists())


if __name__ == '__main__':
    unittest.main()
