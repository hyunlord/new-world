"""Pure closed libtest selection accounting; no process or gate authority.

Callers authenticate the independent listings, actual argv and raw process output.
A returned section may contain failures; its digest binds the unchanged full log,
including failure diagnostics. Native exit and world safety remain caller-owned.
"""
from dataclasses import dataclass
from typing import Literal

from gate_test_summary import parse_log
from generic_gate import require
from generic_workspace_cases import Listing, Outcome, Section, digest


@dataclass(frozen=True, slots=True)
class Selection:
    mode: Literal['default', 'substring', 'ignored-only', 'exact-ignored']
    filter: str | None = None


def reconcile_selected(raw: bytes, all_cases: Listing, ignored_cases: Listing,
                       selection: Selection) -> Section:
    """Account for one mandatory invocation using independently listed cases."""
    names, ignored = set(all_cases.names), set(ignored_cases.names)
    require(not all_cases.benchmarks and not ignored_cases.benchmarks and
            len(names) == len(all_cases.names) and len(ignored) == len(ignored_cases.names) and
            ignored <= names, '/selection', 'invalid independent test inventory')
    match selection.mode:
        case 'default':
            require(selection.filter is None, '/selection', 'default forbids a filter')
            selected, expected_ignored = names, ignored
        case 'substring':
            needle = selection.filter
            require(needle is not None and bool(needle), '/selection', 'nonempty substring required')
            assert needle is not None
            selected = {name for name in names if needle in name}
            expected_ignored = selected & ignored
        case 'ignored-only':
            require(selection.filter is None, '/selection', 'ignored-only forbids a filter')
            selected, expected_ignored = ignored, set[str]()
        case 'exact-ignored':
            needle = selection.filter
            require(needle is not None and bool(needle) and needle in ignored,
                    '/selection', 'exact case must occur in ignored inventory')
            assert needle is not None
            selected, expected_ignored = {needle}, set[str]()
    require(bool(selected), '/selection', 'mandatory invocation selected zero tests')
    summary = parse_log(raw.decode('utf-8'))
    actual = [row.name for row in summary.outcomes]
    require(summary.summaries == 1 and len(actual) == len(set(actual)) and
            set(actual) == selected, '/selection', 'selected outcome inventory differs')
    require({row.name for row in summary.outcomes if row.status == 'ignored'} == expected_ignored,
            '/selection', 'ignored outcomes differ from selected inventory')
    require(summary.measured == 0 and summary.filtered_out == len(names - selected),
            '/selection', 'measured or filtered count differs')
    return Section(tuple(Outcome(row.name, row.status, None) for row in summary.outcomes),
                   digest(raw), summary.failed)
