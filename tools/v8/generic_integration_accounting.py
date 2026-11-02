"""Closed step36 nested libtest envelope; child identities remain unobserved."""
import re
from dataclasses import dataclass

from gate_test_summary import parse_log
from generic_gate import require
from generic_workspace_cases import Listing, Section, digest, lines, reconcile_native

PARENT = 'selected_pack_runs_all_simulation_families_and_rejects_bad_boundaries'
CASES = ('base_before', 'craft_build_learn', 'pressure', 'animal', 'duplicate', 'malformed', 'missing', 'overflow', 'base_after')


@dataclass(frozen=True, slots=True)
class PrefixAccounting:
    section: Section
    raw_sha256: str
    nested_ranges: tuple[tuple[int, int, str], ...] = ()


def account36(raw: bytes, full: Listing, ignored: Listing) -> PrefixAccounting:
    """Accept only the complete normal nine-child envelope, retaining byte ranges."""
    _ = lines(raw)
    require(set(full.names) == {'selected_pack_child', PARENT} and not ignored.names
            and not full.benchmarks and not ignored.benchmarks, '/nested', 'wrong top-level inventory')
    prefix = ('\nrunning 2 tests\ntest selected_pack_child ... ok\ntest '+PARENT+' ... ').encode()
    require(raw.startswith(prefix), '/nested', 'wrong original sequential parent prefix')
    cursor = len(prefix)
    ranges: list[tuple[int, int, str]] = []
    for _case in CASES:
        match = re.match(rb'\nrunning 1 test\ntest selected_pack_child \.\.\. ok\n\ntest result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in [0-9]+(?:\.[0-9]+)?s\n\n', raw[cursor:])
        require(match is not None, '/nested', 'missing, failed or unsupported child section')
        assert match is not None
        section = match[0]
        parsed = parse_log(section.decode())
        require(parsed.summaries == 1 and parsed.passed == 1 and parsed.filtered_out == 1,
                '/nested', 'child counts differ')
        ranges.append((cursor, cursor+len(section), digest(section)))
        cursor += len(section)
    remaining = raw[cursor:]
    require(re.fullmatch(rb'ok\n\ntest result: ok\. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in [0-9]+(?:\.[0-9]+)?s\n\n', remaining) is not None,
            '/nested', 'missing parent terminal or additional output')
    outer = prefix+remaining
    section = reconcile_native(outer, full, ignored)
    return PrefixAccounting(section, digest(raw), tuple(ranges))
