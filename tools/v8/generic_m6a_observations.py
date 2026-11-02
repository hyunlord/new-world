"""Bind native M6a endpoint evidence to its actual report; parsing is not provenance.

The caller authenticates successful fresh invocation, bytes, binary, selected
content and setup. Event duration is never silently replaced by its maximum.
"""
import json
from dataclasses import dataclass

from generic_gate import boundary, document, integer, require, text
from generic_observations import (
    Observation,
    ProbeWorld,
    keys,
    ordinary_pair,
    parse_probe,
)
from mux2_gate_inputs import Json, Node

FIXED_FIELDS = {'schema_version','kind','seed','days','ticks_per_day','clock_offset_ticks',
                'end_tick','observed_ticks','invariant_checks','violation_count','observed_meals',
                'inputs','final_tallies','r2_spoilage','common','mux2','fingerprint',
                'content_revision','observer_scope'}
CHAIN_FIELDS = {'schema_version','kind','passed','content_revision','placed','initially_growing',
                'diagnostics','category_fallback','ripe_tick','harvest_job_tick','harvested_pieces',
                'haul_job_tick','carried_tick','stored_tick','eaten_from_store_tick','intake_evidence',
                'end_tick','commands','fixture'}
MILESTONES = ('ripe_tick','harvest_job_tick','haul_job_tick','carried_tick','stored_tick','eaten_from_store_tick')


@dataclass(frozen=True, slots=True)
class M6aObservation:
    world: Observation
    seed: int
    kind: str
    requested_maximum: int
    content_revision: str
    native_passed: bool
    condition_met: bool | None = None
    reached_limit: bool | None = None


def boolean(node: Node) -> bool:
    value = node.value
    require(type(value) is bool, node.label, 'expected boolean')
    assert isinstance(value, bool)
    return value


@boundary
def parse_fixed(raw: str, report_raw: str, expected: ProbeWorld) -> M6aObservation:
    """Parse one actual m6a run, not the ordinary colony lifecycle fixture."""
    require(type(expected.seed) is int and expected.seed >= 0, '/plan', 'invalid planned seed')
    require(expected.scene == 'm6a' and expected.occurrence == 'primary', '/plan', 'expected single actual M6a producer')
    world = parse_probe(raw, expected)
    report = document(report_raw)
    keys(report, FIXED_FIELDS)
    report.field('schema_version').equals(1)
    report.field('kind').equals('m6a_food_observation_v1')
    report.field('seed').equals(expected.seed)
    days = integer(report.field('days'))
    ticks_per_day = integer(report.field('ticks_per_day'))
    require(1 <= days <= 72 and ticks_per_day > 0 and days*ticks_per_day == expected.duration,
            report.label, 'actual report horizon differs from invocation')
    report.field('end_tick').equals(world.terminal_tick)
    report.field('observed_ticks').equals(expected.duration)
    report.field('invariant_checks').equals(2*expected.duration)
    violations = integer(report.field('violation_count'))
    _ = integer(report.field('observed_meals'))
    _ = integer(report.field('clock_offset_ticks'))
    report.field('inputs').equals({'steward':True,'defence':True,'story':'on','neighbours':'on','player_orders':False})
    for name in ('final_tallies','r2_spoilage','common','mux2'):
        _ = report.field(name).mapping()
    _ = integer(report.field('fingerprint'))
    _ = text(report.field('observer_scope'))
    require(document('{"checkpoints":' + world.checkpoints_json + '}').field("checkpoints").array() == [], '/sidecar', 'M6a has no DebugKill checkpoints')
    return M6aObservation(world,expected.seed,'fixed',expected.duration,
                          text(report.field('content_revision')),violations == 0)


def chain_condition(report: Node, end: int) -> bool:
    """Re-evaluate the production passed predicate; never trust its standalone flag."""
    milestones: list[int | None] = []
    for name in MILESTONES:
        node = report.field(name)
        value = None if node.value is None else integer(node)
        require(value is None or value <= end, node.label, 'milestone beyond actual endpoint')
        milestones.append(value)
    placed = report.field('placed').array()
    require(bool(placed), report.label, 'no naturally placed selected-pack species')
    for tile in placed:
        coordinates = tile.array()
        require(len(coordinates) == 2, tile.label, 'invalid tile')
        for coordinate in coordinates:
            _ = integer(coordinate)
    events = report.field('intake_evidence').array()
    intake_ticks: list[int] = []
    for event in events:
        keys(event, {'tick','agent','stock_tile','pieces_before','pieces_after','meal'})
        tick = integer(event.field('tick'))
        require(tick <= end, event.label, 'intake beyond endpoint')
        require(integer(event.field('pieces_before')) > integer(event.field('pieces_after')),
                event.label, 'intake has no native stock decrease')
        _ = integer(event.field('agent'))
        coordinates = event.field('stock_tile').array()
        require(len(coordinates) == 2, event.label, 'invalid stock tile')
        for coordinate in coordinates:
            _ = integer(coordinate)
        event.field('meal').field('tick').equals(tick)
        intake_ticks.append(tick)
    eaten = milestones[-1]
    require((eaten is None and not intake_ticks) or (bool(intake_ticks) and eaten == min(intake_ticks)),
            report.label, 'stored intake milestone differs from native events')
    fallback = boolean(report.field('category_fallback'))
    growing = boolean(report.field('initially_growing'))
    harvested = integer(report.field('harvested_pieces'))
    return fallback and growing and harvested > 0 and all(value is not None for value in milestones)


@boundary
def parse_selected_pack(raw: str, report_raw: str, maximum_ticks: int) -> M6aObservation:
    """Keep a completed-at-limit failure as diagnostic data, never a passing fixture."""
    require(type(maximum_ticks) is int and maximum_ticks > 0, '/plan', 'invalid requested maximum')
    native = document(raw)
    native.field('schema_version').equals(2)
    native.field('scene').equals('m6a-data-only')
    native.field('seed').equals(0)
    native.field('occurrence').equals('primary')
    native.field('requested_ticks').equals(maximum_ticks)
    native.field('initial_tick').equals(0)
    native.field('requested_terminal_tick').equals(maximum_ticks)
    end = integer(native.field('terminal_tick'))
    require(0 < end <= maximum_ticks, native.label, 'actual event endpoint outside requested maximum')
    event = native.field('event_horizon')
    keys(event, {'condition','condition_met','reached_limit'})
    event.field('condition').equals('m6a_food_chain_complete')
    condition = boolean(event.field('condition_met'))
    reached_limit = boolean(event.field('reached_limit'))
    require(reached_limit == (end == maximum_ticks), event.label, 'claimed limit differs from actual endpoint')
    require(condition or reached_limit, event.label, 'neither actual condition nor limit completed')
    report = document(report_raw)
    keys(report, CHAIN_FIELDS)
    report.field('schema_version').equals(1)
    report.field('kind').equals('m6a_data_only_v1')
    report.field('end_tick').equals(end)
    computed = chain_condition(report, end)
    require(boolean(report.field('passed')) == computed == condition, report.label, 'native report/condition predicate mismatch')
    _ = report.field('diagnostics').array()
    _ = report.field('commands').array()
    _ = text(report.field('fixture'))
    # All original event bounds above are checked before reusing the fixed-endpoint
    # shape validator. This projection is not a replacement native receipt.
    projection = dict(native.mapping())
    del projection['event_horizon']
    projection.update(schema_version=1, requested_ticks=end, requested_terminal_tick=end)
    world = parse_probe(json.dumps(projection),ProbeWorld('m6a-data-only',0,'primary',end))
    require(document('{"checkpoints":' + world.checkpoints_json + '}').field("checkpoints").array() == [], '/sidecar', 'selected-pack producer has no DebugKill checkpoints')
    return M6aObservation(world,0,'selected_pack',maximum_ticks,
                          text(report.field('content_revision')),computed,condition,reached_limit)


def paired_safety(base: M6aObservation, candidate: M6aObservation) -> dict[str, Json]:
    """Require a real matched endpoint. Independent early completions are not companions.

Differing endpoints require paired_selected_safety with the additional native
companion; neither original chain oracle is replaced.
"""
    require(base.kind == candidate.kind and base.seed == candidate.seed, '/pair', 'producer or seed mismatch')
    require(base.requested_maximum == candidate.requested_maximum, '/pair', 'requested maximum differs')
    require(base.content_revision == candidate.content_revision, '/pair', 'selected content revision differs')
    require(base.native_passed and candidate.native_passed, '/pair', 'native M6a fixture failed')
    require(base.world.terminal_tick == candidate.world.terminal_tick, '/pair',
            'matched-horizon BASE companion required; independent event completions are not comparable')
    return ordinary_pair(base.world,candidate.world)


@boundary
def parse_matched_companion(raw: str, report_raw: str, ticks: int,
                            maximum_ticks: int) -> M6aObservation:
    """Parse the additional fixed run, never a replacement primary chain oracle."""
    require(type(ticks) is int and type(maximum_ticks) is int
            and 0 < ticks <= maximum_ticks, '/plan', 'invalid companion horizon')
    world = parse_probe(raw, ProbeWorld('m6a-data-only-companion', 0, 'primary', ticks))
    report = document(report_raw)
    keys(report, (CHAIN_FIELDS - {'passed'}) | {'workload', 'requested_ticks', 'maximum_ticks',
                                              'chain_condition_met', 'primary_oracle_satisfied', 'horizon_complete'})
    report.field('schema_version').equals(1)
    report.field('kind').equals('m6a_data_only_companion_v1')
    report.field('workload').equals('fixed_horizon_companion_primary_chain_oracle_required_separately')
    report.field('primary_oracle_satisfied').equals(False)
    report.field('horizon_complete').equals(True)
    require(integer(report.field('requested_ticks')) == ticks
            and integer(report.field('maximum_ticks')) == maximum_ticks
            and integer(report.field('end_tick')) == ticks, report.label, 'companion report horizon differs')
    condition = chain_condition(report, ticks)
    require(boolean(report.field('chain_condition_met')) == condition, report.label, 'companion chain predicate differs')
    _ = report.field('diagnostics').array()
    _ = report.field('commands').array()
    _ = text(report.field('fixture'))
    require(document('{"checkpoints":' + world.checkpoints_json + '}').field("checkpoints").array() == [], '/sidecar', 'companion has no DebugKill checkpoints')
    return M6aObservation(world, 0, 'matched_companion', maximum_ticks,
                          text(report.field('content_revision')), True, condition, ticks == maximum_ticks)


def paired_selected_safety(base_primary: M6aObservation, candidate_primary: M6aObservation,
                           base_companion: M6aObservation) -> dict[str, Json]:
    """Retain both original chain oracles before using the additional matched run."""
    require(base_primary.kind == candidate_primary.kind == 'selected_pack'
            and base_companion.kind == 'matched_companion', '/pair', 'wrong primary or companion producer')
    require(all(row.native_passed for row in (base_primary, candidate_primary, base_companion)),
            '/pair', 'primary chain oracle or companion horizon failed')
    require(len({row.seed for row in (base_primary, candidate_primary, base_companion)}) == 1
            and len({row.requested_maximum for row in (base_primary, candidate_primary, base_companion)}) == 1
            and len({row.content_revision for row in (base_primary, candidate_primary, base_companion)}) == 1,
            '/pair', 'selected input, seed or maximum differs')
    require((base_primary.world.initial_tick, base_primary.world.founders, base_primary.world.knowledge_enabled) ==
            (base_companion.world.initial_tick, base_companion.world.founders, base_companion.world.knowledge_enabled),
            '/pair', 'BASE companion initial census or capability differs')
    return ordinary_pair(base_companion.world, candidate_primary.world)
