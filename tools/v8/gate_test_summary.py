# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/v8/gate_test_summary.py LOG PROCESS_EXIT
"""Account for libtest output; the caller supplies the trusted producer exit."""
import json
import re
import sys
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final


class SummaryError(ValueError):
    """The stream cannot support unambiguous numeric accounting."""


@dataclass(frozen=True, slots=True)
class Outcome:
    name: str
    status: str


@dataclass(frozen=True, slots=True)
class TestSummary:
    passed: int
    failed: int
    ignored: int
    measured: int
    filtered_out: int
    summaries: int
    outcomes: tuple[Outcome, ...]


_NAME: Final = re.compile(r'test (?!result:)(\S+) \.\.\.[ \t]*')
_COMPLETE: Final = re.compile(r'test (?!result:)(\S+) \.\.\. (ok|FAILED|ignored)(?:, [^\n]*)?$')
_SUMMARY: Final = re.compile(
    r'test result: (ok|FAILED)\s*\. ([0-9]+) passed; ([0-9]+) failed; '
    r'([0-9]+) ignored; ([0-9]+) measured; ([0-9]+) filtered out\s*; '
    r'finished in ([0-9]+(?:\.[0-9]+)?)s'
)


def parse_stream(text: str) -> TestSummary:
    """Decode payloads, including excerpts; use parse_log for full accounting."""
    totals = [0, 0, 0, 0, 0]
    outcomes: list[Outcome] = []
    pending: str | None = None
    fragment = ''
    summaries = 0
    for number, raw in enumerate(text.splitlines(), 1):
        line = raw.strip()
        if not line:
            continue
        if re.fullmatch(r'test \S+ has been running for over [0-9]+ seconds', line):
            continue
        active = line.startswith('test ') or bool(fragment) or line in ('ok', 'FAILED', 'ignored')
        if not active:
            continue
        complete = _COMPLETE.search(line)
        if complete is not None:
            earlier = _NAME.search(line[:complete.start()])
            if (pending is not None or earlier is not None) and complete[2] != 'ok':
                raise SummaryError(f'line {number}: ambiguous interleaved outcome')
            outcomes.append(Outcome(complete[1], complete[2]))
            line = line[:complete.start()]
        names = list(_NAME.finditer(line))
        if names:
            if pending is not None or len(names) != 1:
                raise SummaryError(f'line {number}: multiple unfinished test names')
            pending = names[0][1]
            line = line[:names[0].start()] + line[names[0].end():]
        if line in ('ok', 'FAILED', 'ignored') or line.endswith('filtered outok'):
            status = 'ok' if line.endswith('filtered outok') else line
            if pending is None or status != 'ok':
                raise SummaryError(f'line {number}: ambiguous detached outcome')
            outcomes.append(Outcome(pending, status))
            pending = None
            line = line[:-2] if line.endswith('filtered outok') else ''
        line = line.strip()
        if not line:
            continue
        if fragment:
            fragment += ' ' + line
        elif line.startswith('test result:'):
            fragment = line
        elif line.startswith('test '):
            raise SummaryError(f'line {number}: malformed test record')
        else:
            continue
        match = _SUMMARY.fullmatch(fragment)
        if match is not None:
            counts = [int(match[index]) for index in range(2, 7)]
            if (match[1] == 'FAILED') != (counts[1] > 0):
                raise SummaryError(f'line {number}: status disagrees with failed count')
            totals = [left + right for left, right in zip(totals, counts)]
            summaries += 1
            fragment = ''
        elif 'finished in' in fragment or fragment.count('test result:') > 1:
            raise SummaryError(f'line {number}: malformed summary')
    if pending is not None or fragment:
        raise SummaryError('truncated test record or summary')
    if summaries == 0:
        raise SummaryError('no complete numeric summaries')
    return TestSummary(totals[0], totals[1], totals[2], totals[3], totals[4], summaries, tuple(outcomes))


def parse_log(text: str) -> TestSummary:
    """Require named outcomes to reconcile with all embedded summary payloads."""
    result = parse_stream(text)
    named = Counter(outcome.status for outcome in result.outcomes)
    if (named['ok'], named['FAILED'], named['ignored']) != (result.passed, result.failed, result.ignored):
        raise SummaryError('named outcomes do not reconcile with summary totals')
    return result


def validate_cases(summary: TestSummary, expected: tuple[str, ...], allowed_ignored: tuple[str, ...]) -> None:
    """Apply a caller-owned case inventory to a single producer only."""
    names = [outcome.name for outcome in summary.outcomes]
    ignored = {outcome.name for outcome in summary.outcomes if outcome.status == 'ignored'}
    if summary.summaries != 1 or len(names) != len(set(names)) or set(names) != set(expected):
        raise SummaryError('single-producer case inventory differs')
    if not ignored.issubset(set(allowed_ignored)):
        raise SummaryError('unapproved ignored case')


def main(argv: list[str]) -> int:
    if len(argv) != 2 or re.fullmatch(r'[0-9]+', argv[1]) is None:
        print(json.dumps({'status': 'FAIL', 'error': 'usage: LOG PROCESS_EXIT (unsigned integer)'}))
        return 1
    process_exit = int(argv[1])
    try:
        result = parse_log(Path(argv[0]).read_text(encoding='utf-8'))
    except (SummaryError, OSError, UnicodeError) as error:
        print(json.dumps({'status': 'FAIL', 'process_exit': process_exit, 'error': str(error)}))
        return 1
    passed = process_exit == 0 and result.failed == 0
    print(json.dumps({**asdict(result), 'status': 'PASS' if passed else 'FAIL', 'process_exit': process_exit}, sort_keys=True))
    return 0 if passed else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
