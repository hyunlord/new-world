"""Offline selection contracts using real listing/log parsers."""
import hashlib
import unittest

from gate_test_summary import SummaryError
from generic_gate import GateContractError
from generic_selected_cases import Selection, reconcile_selected
from generic_workspace_cases import Listing, parse_listing


def inventory(names: tuple[str, ...]) -> Listing:
    body = ''.join(f'{name}: test\n' for name in names)
    return parse_listing((body + ('\n' if names else '') +
                          f'{len(names)} '+('test' if len(names) == 1 else 'tests')+
                          ', 0 benchmarks\n').encode())


def output(rows: tuple[tuple[str, str], ...], filtered: int = 0,
           diagnostic: str = '') -> bytes:
    passed = sum(status == 'ok' for _, status in rows)
    failed = sum(status == 'FAILED' for _, status in rows)
    ignored = sum(status == 'ignored' for _, status in rows)
    body = ''.join(f'test {name} ... {status}\n' for name, status in rows)
    return (f'\nrunning {len(rows)} tests\n' + body + diagnostic +
            f'\ntest result: {"FAILED" if failed else "ok"}. {passed} passed; ' +
            f'{failed} failed; {ignored} ignored; 0 measured; {filtered} filtered out; ' +
            'finished in 0.01s\n\n').encode()


class SelectedCasesTests(unittest.TestCase):
    def test_default_keeps_ignored(self) -> None:
        raw = output((('jobs::one', 'ok'), ('old', 'ignored')))
        result = reconcile_selected(raw, inventory(('jobs::one', 'old')), inventory(('old',)), Selection('default'))
        self.assertEqual(tuple(row.status for row in result.outcomes), ('ok', 'ignored'))

    def test_substring_matches_anywhere_and_retains_ignored(self) -> None:
        result = reconcile_selected(output((('nested::jobs::one', 'ok'), ('jobs::old', 'ignored')), 1),
                                    inventory(('nested::jobs::one', 'jobs::old', 'other')),
                                    inventory(('jobs::old',)), Selection('substring', 'jobs::'))
        self.assertEqual(len(result.outcomes), 2)

    def test_ignored_only_runs_ignored_cases(self) -> None:
        result = reconcile_selected(output((('old', 'ok'),), 1), inventory(('ordinary', 'old')),
                                    inventory(('old',)), Selection('ignored-only'))
        self.assertEqual(result.outcomes[0].status, 'ok')

    def test_exact_ignored_does_not_select_similar_name(self) -> None:
        result = reconcile_selected(output((('b5', 'ok'),), 2), inventory(('b5', 'b5_extra', 'ordinary')),
                                    inventory(('b5', 'b5_extra')), Selection('exact-ignored', 'b5'))
        self.assertEqual(tuple(row.name for row in result.outcomes), ('b5',))

    def test_failure_diagnostics_remain_byte_bound(self) -> None:
        raw = output((('old', 'FAILED'),), 1, '\nfailures:\n\n---- old stdout ----\nassertion failed: marker\n\nfailures:\n    old\n')
        result = reconcile_selected(raw, inventory(('ordinary', 'old')), inventory(('old',)), Selection('ignored-only'))
        self.assertEqual((result.failed, result.sha256), (1, hashlib.sha256(raw).hexdigest()))
        self.assertEqual(result.outcomes[0].status, 'FAILED')

    def test_missing_or_duplicate_outcomes_rejected(self) -> None:
        for rows in ((('a', 'ok'),), (('a', 'ok'), ('a', 'ok'))):
            with self.subTest(rows=rows), self.assertRaises(GateContractError):
                _ = reconcile_selected(output(rows), inventory(('a', 'b')), inventory(()), Selection('default'))

    def test_extra_outcome_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            _ = reconcile_selected(output((('old', 'ok'), ('ordinary', 'ok'))), inventory(('ordinary', 'old')),
                               inventory(('old',)), Selection('ignored-only'))

    def test_wrong_filtered_count_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            _ = reconcile_selected(output((('old', 'ok'),)), inventory(('ordinary', 'old')),
                               inventory(('old',)), Selection('ignored-only'))

    def test_selected_ignored_cannot_remain_ignored(self) -> None:
        with self.assertRaises(GateContractError):
            _ = reconcile_selected(output((('old', 'ignored'),), 1), inventory(('ordinary', 'old')),
                               inventory(('old',)), Selection('ignored-only'))

    def test_default_cannot_execute_ignored(self) -> None:
        with self.assertRaises(GateContractError):
            _ = reconcile_selected(output((('old', 'ok'),)), inventory(('old',)), inventory(('old',)), Selection('default'))

    def test_zero_selection_rejected_in_each_mode(self) -> None:
        for selection in (Selection('default'), Selection('substring', 'missing'), Selection('ignored-only'), Selection('exact-ignored', 'missing')):
            with self.subTest(selection=selection), self.assertRaises(GateContractError):
                _ = reconcile_selected(output(()), inventory(()), inventory(()), selection)

    def test_inconsistent_filters_rejected(self) -> None:
        for selection in (Selection('default', 'x'), Selection('ignored-only', 'x'), Selection('substring'),
                          Selection('substring', ''), Selection('exact-ignored'), Selection('exact-ignored', '')):
            with self.subTest(selection=selection), self.assertRaises(GateContractError):
                _ = reconcile_selected(output((('x', 'ok'),)), inventory(('x',)), inventory(()), selection)

    def test_exact_nonignored_name_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            _ = reconcile_selected(output((('x', 'ok'),)), inventory(('x',)), inventory(()), Selection('exact-ignored', 'x'))

    def test_invalid_independent_inventory_rejected(self) -> None:
        cases = ((inventory(('x',)), inventory(('missing',))),
                 (Listing(('x', 'x'), (), '0'*64), inventory(())),
                 (inventory(('x',)), Listing(('x', 'x'), (), '0'*64)),
                 (Listing(('x',), ('bench',), '0'*64), inventory(())),
                 (inventory(('x',)), Listing((), ('bench',), '0'*64)))
        for all_cases, ignored in cases:
            with self.subTest(all_cases=all_cases, ignored=ignored), self.assertRaises(GateContractError):
                _ = reconcile_selected(output((('x', 'ok'),)), all_cases, ignored, Selection('default'))

    def test_multiple_summaries_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            _ = reconcile_selected(output((('a', 'ok'),))+output((('b', 'ok'),)), inventory(('a', 'b')), inventory(()), Selection('default'))

    def test_numeric_outcome_mismatch_rejected_by_shared_parser(self) -> None:
        raw = output((('x', 'FAILED'),)).replace(b'0 passed; 1 failed', b'1 passed; 0 failed')
        with self.assertRaises(SummaryError):
            _ = reconcile_selected(raw, inventory(('x',)), inventory(()), Selection('default'))

    def test_measured_count_rejected(self) -> None:
        raw = output((('x', 'ok'),)).replace(b'0 measured', b'1 measured')
        with self.assertRaises(GateContractError):
            _ = reconcile_selected(raw, inventory(('x',)), inventory(()), Selection('default'))


if __name__ == '__main__':
    _ = unittest.main()
