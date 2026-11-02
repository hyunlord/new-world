# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_gate_test_summary.py
"""Libtest accounting regression tests use synthetic byte-stream fixtures."""
import json
import shlex
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from gate_test_summary import SummaryError, parse_log, parse_stream, validate_cases

RESULT = 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 490 filtered out; finished in 0.01s\n'
VALID = 'running 1 test\ntest case_a ... ok\n\n' + RESULT


class SummaryTests(unittest.TestCase):
    def test_read_data_only_uses_full_exact_case_filter(self) -> None:
        rows = [shlex.split(line) for line in Path(__file__).with_name('gate.sh').read_text().splitlines()
                if line.startswith('step 49 ')]
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0][:5], ['step', '49', 'read_data_only', 'bash', '-c'])
        self.assertEqual(len(rows[0]), 6)
        self.assertEqual(shlex.split(rows[0][5]), [
            'cd', 'rust', '&&', 'cargo', 'test', '--release', '-p', 'sim-core', '--lib',
            'colony::diagnosis::tests::data_only_check_addition', '--', '--exact',
        ])

    def test_read_data_only_rejects_historical_zero_selection_and_wrong_identity(self) -> None:
        expected = ('colony::diagnosis::tests::data_only_check_addition',)
        # Preserved final-d-03 shape; exit0 did not select the dedicated test.
        zero = ('running 0 tests\n\n'
                'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; '
                '491 filtered out; finished in 0.00s\n')
        parsed = parse_log(zero)
        self.assertEqual((parsed.passed, parsed.failed, parsed.filtered_out), (0, 0, 491))
        with self.assertRaises(SummaryError):
            validate_cases(parsed, expected, ())
        with self.assertRaises(SummaryError):
            validate_cases(parse_log(VALID.replace('case_a', 'diagnosis::tests::data_only_check_addition')),
                           expected, ())
        for filtered in (0, 490, 999):
            with self.subTest(filtered=filtered):
                correct = VALID.replace('case_a', expected[0]).replace('490 filtered', f'{filtered} filtered')
                result = parse_log(correct)
                self.assertIsNone(validate_cases(result, expected, ()))
                self.assertEqual((result.passed, result.failed, result.ignored, result.measured), (1, 0, 0, 0))

    def test_counts_preserve_filtered_and_ignored(self) -> None:
        # Given an ignored mechanism and one ordinary success.
        raw = 'test a ... ok\ntest b ... ignored, mechanism\n' + RESULT.replace('0 ignored', '1 ignored')
        # When the completed stream is decoded.
        result = parse_log(raw)
        # Then filtered tests are distinct from actual outcomes.
        self.assertEqual((result.passed, result.failed, result.ignored, result.measured, result.filtered_out), (1, 0, 1, 0, 490))

    def test_malformed_numeric_counts_reject(self) -> None:
        for value in ('inf', 'NaN', '-1', '1.0', '1e0', 'true'):
            with self.subTest(value=value), self.assertRaises(SummaryError):
                parse_log(VALID.replace('1 passed', value + ' passed'))

    def test_truncated_or_missing_summary_rejects(self) -> None:
        for raw in ('test a ... ok\n', VALID[:-2], 'test result: oktest influence::case ... ok\n'):
            with self.subTest(raw=raw), self.assertRaises(SummaryError):
                parse_log(raw)

    def test_duplicate_summary_cannot_double_count(self) -> None:
        with self.assertRaises(SummaryError):
            parse_log(VALID + RESULT)

    def test_named_count_mismatch_rejects(self) -> None:
        with self.assertRaises(SummaryError):
            parse_log(VALID.replace('1 passed', '2 passed'))

    def test_nested_occurrences_are_not_deduplicated(self) -> None:
        # Given two independently completed invocations of the same test.
        raw = VALID + VALID
        result = parse_log(raw)
        self.assertEqual((result.passed, result.summaries, len(result.outcomes)), (2, 2, 2))

    def test_single_producer_duplicate_case_rejects(self) -> None:
        result = parse_log('test a ... ok\ntest a ... ok\n' + RESULT.replace('1 passed', '2 passed'))
        with self.assertRaises(SummaryError):
            validate_cases(result, ('a', 'b'), ())

    def test_single_producer_expected_cases_and_ignored(self) -> None:
        raw = 'test a ... ok\ntest b ... ignored, intentional\n' + RESULT.replace('0 ignored', '1 ignored')
        result = parse_log(raw)
        self.assertIsNone(validate_cases(result, ('a', 'b'), ('b',)))

    def test_unapproved_ignored_case_rejects(self) -> None:
        raw = 'test a ... ignored\n' + RESULT.replace('1 passed', '0 passed').replace('0 ignored', '1 ignored')
        with self.assertRaises(SummaryError):
            validate_cases(parse_log(raw), ('a',), ())

    def test_echoed_commands_do_not_count(self) -> None:
        raw = '[4 test_debug] echo "' + RESULT.strip() + '"\n' + VALID
        self.assertEqual(parse_log(raw).summaries, 1)

    def test_application_numeric_text_is_not_summary(self) -> None:
        with self.assertRaises(SummaryError):
            parse_log(VALID.replace('test result: ok.', 'application example:'))

    def test_valid_failure_remains_parseable_failure(self) -> None:
        raw = VALID.replace('case_a ... ok', 'case_a ... FAILED').replace('result: ok.', 'result: FAILED.').replace('1 passed; 0 failed', '0 passed; 1 failed')
        self.assertEqual(parse_log(raw).failed, 1)

    def test_summary_status_must_match_failure_count(self) -> None:
        with self.assertRaises(SummaryError):
            parse_log(VALID.replace('result: ok.', 'result: FAILED.'))

    def test_split_prefix_recovers_numeric_count(self) -> None:
        raw = 'test result: oktest influence::case ... ok\n. 1 passed; 0 failed; 0 ignored; 0 measured; 490 filtered out; finished in 0.00s\n'
        result = parse_stream(raw)
        self.assertEqual((result.passed, result.summaries), (1, 1))

    def test_embedded_m1_summary_and_parent_completion(self) -> None:
        raw = 'test parent ... test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 490 filtered outok\n; finished in 0.00s\n'
        result = parse_stream(raw)
        self.assertEqual((result.passed, result.summaries, result.outcomes[0].name), (1, 1, 'parent'))

    def test_named_outcome_between_numeric_payload_and_terminal(self) -> None:
        raw = RESULT.replace('filtered out;', 'filtered outtest parent ... ok\n;')
        self.assertEqual(parse_stream(raw).filtered_out, 490)

    def test_one_pending_parent_closes_after_successful_child(self) -> None:
        raw = 'test parent ... test child ... ok\nok\n' + RESULT
        self.assertEqual(len(parse_stream(raw).outcomes), 2)

    def test_mixed_interleaved_outcomes_are_not_guessed(self) -> None:
        with self.assertRaises(SummaryError):
            parse_stream('test parent ... test child ... FAILED\nok\n' + RESULT)

    def test_two_pending_names_are_ambiguous(self) -> None:
        with self.assertRaises(SummaryError):
            parse_stream('test a ... test b ... \nok\nok\n' + RESULT)

    def test_summary_duration_must_be_finite_unsigned_decimal(self) -> None:
        for duration in ('NaN', 'inf', '-1', '1e3'):
            with self.subTest(duration=duration), self.assertRaises(SummaryError):
                parse_log(VALID.replace('0.01s', duration + 's'))

    def test_pending_name_survives_line_ending(self) -> None:
        raw = 'test parent ... \n' + RESULT.replace('filtered out;', 'filtered outok\n;')
        self.assertEqual(parse_stream(raw).outcomes[0].name, 'parent')

    def test_long_running_notice_is_not_an_outcome(self) -> None:
        raw = 'test case_a has been running for over 60 seconds\n' + VALID
        self.assertEqual(parse_log(raw).passed, 1)

    def test_zero_test_summary_is_valid(self) -> None:
        self.assertEqual(parse_log(RESULT.replace('1 passed', '0 passed')).passed, 0)

    def test_cli_requires_explicit_producer_exit(self) -> None:
        run = subprocess.run([sys.executable, str(Path(__file__).with_name('gate_test_summary.py')), 'unused.log'], capture_output=True, text=True, check=False)
        self.assertEqual(run.returncode, 1)
        self.assertEqual(json.loads(run.stdout)['status'], 'FAIL')

    def test_cli_nonzero_exit_rejects_success_looking_log(self) -> None:
        # Given a real log file and a failed producer exit.
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / 'test.log'
            log.write_text(VALID)
            # When the public CLI parses the actual file.
            run = subprocess.run([sys.executable, str(Path(__file__).with_name('gate_test_summary.py')), str(log), '1'], capture_output=True, text=True, check=False)
        # Then counts remain visible but admission fails.
        self.assertEqual(run.returncode, 1)
        self.assertEqual(json.loads(run.stdout)['passed'], 1)

    def test_cli_success_reports_typed_totals(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / 'test.log'
            log.write_text(VALID)
            run = subprocess.run([sys.executable, str(Path(__file__).with_name('gate_test_summary.py')), str(log), '0'], capture_output=True, text=True, check=False)
        self.assertEqual(run.returncode, 0)
        self.assertEqual(json.loads(run.stdout)['filtered_out'], 490)


if __name__ == '__main__':
    unittest.main()
