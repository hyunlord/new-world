"""Retained p111 stdout excerpt and synthetic ambiguity negatives; no native runs."""
import unittest

from gate_test_summary import SummaryError, parse_log, parse_stream

PARENT = 'colony::mind::tests::revision_frames_registry_names_instead_of_delimiters'
CHILD = 'colony::knowledge::registry_tests::uninstalled_child'
NUMERIC = '1 passed; 0 failed; 0 ignored; 0 measured; 490 filtered out; finished in 0.00s'
# Exact nonblank lines272/274/275 of retained p111 BASE17 stdout, preserving split.
EXCERPT = f'test {PARENT} ... test {CHILD} ... ok\n\ntest result: ok\nok. {NUMERIC}\n'
OUTER = 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n'


class SplitSummaryTests(unittest.TestCase):
    def test_retained_split_accounts_both_success_tokens(self) -> None:
        excerpt = parse_stream(EXCERPT)
        self.assertEqual((excerpt.passed, excerpt.summaries), (1, 1))
        self.assertEqual([(item.name, item.status) for item in excerpt.outcomes], [(CHILD, 'ok'), (PARENT, 'ok')])
        complete = parse_log(EXCERPT + OUTER)
        self.assertEqual((complete.passed, complete.failed, complete.ignored, complete.summaries), (2, 0, 0, 2))

    def test_missing_pending_parent_rejects(self) -> None:
        with self.assertRaises(SummaryError):
            _ = parse_log(EXCERPT.replace(f'test {PARENT} ... ', '') + OUTER)

    def test_failed_or_ignored_tokens_are_not_guessed(self) -> None:
        for fragment in (EXCERPT.replace('result: ok', 'result: FAILED'),
                         EXCERPT.replace('\nok.', '\nFAILED.'),
                         EXCERPT.replace('\nok.', '\nignored.'),
                         EXCERPT.replace(f'{CHILD} ... ok', f'{CHILD} ... FAILED'),
                         EXCERPT.replace('0 failed', '1 failed'),
                         EXCERPT.replace('0 ignored', '1 ignored')):
            with self.subTest(fragment=fragment), self.assertRaises(SummaryError):
                _ = parse_log(fragment + OUTER)

    def test_partial_malformed_or_extra_numeric_payload_rejects(self) -> None:
        for fragment in (EXCERPT.replace('1 passed', 'NaN passed'),
                         EXCERPT.replace('0.00s', 'NaNs'), EXCERPT.replace('0.00s', '0.00s extra'),
                         EXCERPT.replace('finished in 0.00s', ''),
                         EXCERPT.replace('ok. 1 passed', 'okok. 1 passed'),
                         EXCERPT.replace('1 passed', '-1 passed')):
            with self.subTest(fragment=fragment), self.assertRaises(SummaryError):
                _ = parse_log(fragment + OUTER)

    def test_reconciliation_and_multiple_pending_remain_strict(self) -> None:
        for fragment in (EXCERPT, EXCERPT + OUTER + OUTER,
                         EXCERPT.replace('1 passed', '2 passed') + OUTER,
                         'test another ... \n' + EXCERPT + OUTER):
            with self.subTest(fragment=fragment), self.assertRaises(SummaryError):
                _ = parse_log(fragment)

    def test_ordinary_split_needs_no_extra_parent_outcome(self) -> None:
        ordinary = f'test {CHILD} ... ok\ntest result: ok\n. {NUMERIC}\n'
        result = parse_log(ordinary)
        self.assertEqual((result.passed, len(result.outcomes)), (1, 1))


if __name__ == '__main__':
    _ = unittest.main()
