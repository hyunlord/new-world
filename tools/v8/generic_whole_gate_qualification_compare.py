# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_qualification_compare
"""Compare existing original-off artifacts; timing exclusions never hide state."""
import math
from collections.abc import Callable
from copy import deepcopy
from pathlib import Path

import generic_m6a_observations
import read_summary_normalise as normalizer
from generic_gate import document, integer, require
from generic_m6a_observations import CHAIN_FIELDS, FIXED_FIELDS, chain_condition
from generic_observations import keys
from mux2_gate_inputs import Json, Node

_difference: Callable[[Json, Json], str | None] = normalizer.first_difference
_normalise: Callable[[Json, Json, str | None], tuple[Json, Json, list[str]]] = normalizer.normalise_pair
NAV_TIMING = ('tick_mean_us', 'movement_us', 'job_assign_us', 'job_execute_us', 'run_seconds')
NAV_MEASUREMENT = ('within_budget', 'load_before', 'load_after')
TICK_TIMING = ('mean_ms', 'p50_ms', 'p90_ms', 'p99_ms', 'p999_ms', 'max_ms', 'n_over_8ms', 'n_over_16_67ms')
PERF_TIMING = ('wall_seconds', 'steady_tick_ms_total', 'prune_ms', 'unaccounted_ms')
SYSTEM_TIMING = ('total_ms', 'mean_us_per_call', 'max_us', 'share_of_tick_time', 'total_ms_incl_warmup', 'max_us_incl_warmup')


def measurement(node: Node) -> None:
    value = node.value
    require(type(value) in (int, float), node.label, 'expected numeric measurement, excluding bool')
    if not isinstance(value, (int, float)):
        raise TypeError('validated measurement')
    require(math.isfinite(value) and value >= 0, node.label, 'expected finite nonnegative measurement')


def slowest(node: Node) -> None:
    """Membership is timing-selected; only the closed emitted schema may be omitted."""
    for row in node.array():
        keys(row, {'tick','display_day','clock','total_ms','top_systems','due_periodic'})
        for name in ('tick','display_day'):
            require(integer(row.field(name)) >= 0, row.label, 'negative slow-tick coordinate')
        _ = row.field('clock').text()
        measurement(row.field('total_ms'))
        for system in row.field('top_systems').array():
            keys(system, {'name','ms'})
            _ = system.field('name').text()
            measurement(system.field('ms'))
        for system in row.field('due_periodic').array():
            _ = system.text()


def omit(node: Node, names: tuple[str, ...], prefix: str, excluded: list[str]) -> None:
    for name in names:
        item = node.field(name)
        if name == 'slowest_ticks':
            slowest(item)
        elif name == 'systems_ms':
            for value in item.array():
                measurement(value)
        elif name == 'within_budget':
            require(type(item.value) is bool, item.label, 'expected timing verdict boolean')
        elif name in {'load_before', 'load_after'}:
            _ = item.text()
        else:
            measurement(item)
            if name.startswith('n_over_'):
                _ = integer(item)
        del node.mapping()[name]
        excluded.append(prefix+'/'+name)


def projection(kind: str, raw: str) -> tuple[Json, list[str]]:
    value = document(raw)
    excluded: list[str] = []
    if kind == 'm6a':
        keys(value, FIXED_FIELDS)
        value.field('schema_version').equals(1)
        value.field('kind').equals('m6a_food_observation_v1')
        value.field('days').equals(72)
        value.field('ticks_per_day').equals(1440)
        value.field('end_tick').equals(72*1440)
        value.field('observed_ticks').equals(72*1440)
        value.field('invariant_checks').equals(2*72*1440)
        _ = integer(value.field('fingerprint'))
    elif kind == 'm6a-data-only':
        keys(value, CHAIN_FIELDS)
        value.field('schema_version').equals(1)
        value.field('kind').equals('m6a_data_only_v1')
        value.field('passed').equals(chain_condition(value, integer(value.field('end_tick'))))
        require(bool(value.field('commands').array()), '/commands', 'missing actual command sequence')
    elif kind == 'nav':
        for name in ('agents', 'seed', 'warmup_ticks', 'measured_ticks', 'alive_end', 'requests', 'memo_hits', 'searches', 'paths_now', 'nav_version'):
            _ = integer(value.field(name))
        omit(value, NAV_TIMING, '', excluded)
        omit(value, NAV_MEASUREMENT, '', excluded)
    elif kind == 'perf-index':
        value.field('pass').equals(15)
        require(len(value.field('runs').array()) == 1, '/runs', 'single run required')
        for i, row in enumerate(value.field('runs').array()):
            row.field('status').equals('ok')
            omit(row, ('mean_ms','p999_ms','max_ms','wall_seconds'), f'/runs/{i}', excluded)
    elif kind == 'perf':
        value.field('pass').equals(15)
        for name in ('seed', 'ticks', 'fingerprint', 'alive_end', 'founders', 'settlements_end', 'chronicle_entries_end', 'buildings_end'):
            _ = integer(value.field(name))
        omit(value, PERF_TIMING, '', excluded)
        omit(value.field('warmup'), ('tick0_ms',), '/warmup', excluded)
        omit(value.field('warmup').field('summary'), TICK_TIMING, '/warmup/summary', excluded)
        omit(value.field('tick_summary'), TICK_TIMING, '/tick_summary', excluded)
        for i, row in enumerate(value.field('systems').array()):
            omit(row, SYSTEM_TIMING, f'/systems/{i}', excluded)
        for i, row in enumerate(value.field('per_day').array()):
            omit(row, ('tick_ms', 'systems_ms'), f'/per_day/{i}', excluded)
        snapshots = value.field('snapshot_timings')
        if snapshots.value is not None:
            for name, row in snapshots.mapping().items():
                omit(Node(row), ('median_us', 'p90_us'), '/snapshot_timings/'+name, excluded)
        # Slowest-tick membership and ordering are selected by measured timing.
        omit(value, ('slowest_ticks',), '', excluded)
    else:
        require(False, '/comparator', 'unsupported original artifact schema')
    return value.value, excluded


def compare(kind: str, left: str, right: str) -> list[str]:
    if kind == 'baseline':
        a, b = document(left), document(right)
        for value in (a,b):
            value.field('schema_version').equals(13)
            require(bool(value.field('runs').array()), '/runs', 'empty baseline')
        before, after, excluded = _normalise(deepcopy(a.value), deepcopy(b.value), None)
    else:
        before, excluded = projection(kind, left)
        after, other = projection(kind, right)
        require(excluded == other, '/comparison', 'different measurement schema')
    difference = _difference(before, after)
    require(difference is None, '/comparison', 'deterministic product state differs: '+str(difference))
    return excluded

HEADER = b"\n| Seed | Wall seconds | ms / agent-tick |\n|---|---:|---:|\n"
END = b"\n## Death census\n"

def markdown_simulation(raw: bytes, summary: str) -> bytes:
    """Replace only a table whose exact stopwatch rendering matches raw JSON."""
    require(raw.count(HEADER) == 1 and raw.count(END) == 1, '/markdown', 'timing boundaries differ')
    start = raw.index(HEADER) + len(HEADER)
    stop = raw.index(END, start)
    rows = document(summary).field('runs').array()
    expected = bytearray()
    stable = bytearray()
    for row in rows:
        seed = row.field('seed').value
        wall = row.field('wall_seconds').value
        tick = row.field('ms_per_agent_tick').value
        require(type(seed) is int and type(wall) in (int, float) and type(tick) in (int, float),
                '/markdown', 'invalid numeric timing row')
        if not isinstance(wall, (int, float)) or not isinstance(tick, (int, float)):
            raise TypeError('unreachable numeric timing type')
        expected.extend(f'| {seed} | {wall:.6f} | {tick:.9f} |\n'.encode())
        stable.extend(f'| {seed} | 0.000000 | 0.000000000 |\n'.encode())
    require(raw[start:stop] == bytes(expected), '/markdown', 'measurement table is not raw-summary rendering')
    return raw[:start] + bytes(stable) + raw[stop:]



def imported_sources() -> tuple[tuple[str, Path], ...]:
    return (('tools/v8/read_summary_normalise.py', Path(normalizer.__file__).resolve(strict=True)),
            ('tools/v8/generic_m6a_observations.py', Path(generic_m6a_observations.__file__).resolve(strict=True)))
