"""Native performance world safety; process/build authentication belongs to the runner."""
from dataclasses import dataclass
from typing import Literal

from generic_gate import boundary, document, integer, require
from generic_observations import Observation, keys, losses, ordinary_pair
from mux2_gate_inputs import Json

Kind = Literal['bench', 'build_only', 'nav', 'perf']
FIELDS = {'schema_version', 'initial_tick', 'initial_population', 'requested_ticks', 'requested_terminal_tick', 'terminal_tick', 'terminal_alive', 'terminal_complete', 'history_complete', 'history_source', 'knowledge_capability', 'knowledge_lost', 'clock_offset_ticks', 'ticks_per_day', 'deaths'}


@dataclass(frozen=True, slots=True)
class Plan:
    kind: Kind
    ticks: int
    seed: int
    agents: int | None
    raw_path: str
    producer_pid: int
    workload_json: str


@dataclass(frozen=True, slots=True)
class PerfObservation:
    plan: Plan
    world: Observation
    clock_offset: int
    ticks_per_day: int


@boundary
def parse_native(sidecar: str, raw_report: str, plan: Plan) -> PerfObservation:
    """Require the exact raw report bytes and independently planned child identity."""
    require(plan.kind in {'bench', 'build_only', 'nav', 'perf'}, '/plan', 'unknown producer')
    require(type(plan.ticks) is int and plan.ticks >= 0 and
            (plan.ticks == 0) == (plan.kind == 'build_only'), '/plan', 'invalid native horizon')
    require(type(plan.producer_pid) is int and plan.producer_pid > 0, '/plan', 'invalid native PID')
    require(type(plan.seed) is int and plan.seed >= 0, '/plan', 'invalid native seed')
    require((plan.kind == 'perf' and plan.agents is None) or
            (type(plan.agents) is int and plan.agents > 0), '/plan', 'invalid requested population')
    envelope = document(sidecar)
    keys(envelope, {'schema_version','producer_pid','raw_report_path','raw_report_bytes','observation'})
    envelope.field('schema_version').equals(1)
    envelope.field('producer_pid').equals(plan.producer_pid)
    envelope.field('raw_report_path').equals(plan.raw_path)
    envelope.field('raw_report_bytes').equals(raw_report)
    native = envelope.field('observation')
    extra: set[str] = {'warmup_alive','interventions'} if plan.kind in {'bench','nav'} else set()
    keys(native, FIELDS | extra)
    native.field('schema_version').equals(1)
    native.field('initial_tick').equals(0)
    native.field('requested_ticks').equals(plan.ticks)
    native.field('requested_terminal_tick').equals(plan.ticks)
    native.field('terminal_tick').equals(plan.ticks)
    native.field('terminal_complete').equals(True)
    native.field('history_complete').equals(True)
    native.field('history_source').equals('fresh_world_append_only_chronicle')
    capability = native.field('knowledge_capability')
    keys(capability, {'enabled','source'})
    capability.field('source').equals('resources.rules.colony.knowledge')
    enabled = capability.field('enabled').value
    require(type(enabled) is bool, capability.label, 'invalid capability')
    assert isinstance(enabled, bool)
    lost = losses(native.field('knowledge_lost'), enabled, plan.ticks)
    death = native.field('deaths')
    keys(death, {'starvation','dehydration','exposure','combat','old_age'})
    for value in death.mapping():
        _ = integer(death.field(value))
    founders = integer(native.field('initial_population'))
    alive = integer(native.field('terminal_alive'))
    clock = integer(native.field('clock_offset_ticks'))
    tpd = integer(native.field('ticks_per_day'))
    require(tpd > 0, native.label, 'zero day duration')
    report = document(raw_report)
    workload = document(plan.workload_json)
    expected = {'bench': {'label','side','script','wolves'},
                'build_only': {'label','side','repeat'}, 'nav': {'script'},
                'perf': {'mode','scene','side','feed'}}[plan.kind]
    keys(workload, expected)
    for name in expected:
        report.field(name).equals(workload.field(name).value)
    report.field('agents_requested' if plan.kind == 'perf' else 'agents').equals(plan.agents)
    report.field('alive' if plan.kind == 'build_only' else 'alive_end').equals(alive)
    if plan.kind == 'build_only':
        report.field('build_only').equals(True)
        require(founders == alive, native.label, 'construction census differs without ticks')
    else:
        report.field('seed').equals(plan.seed)
    if plan.kind in {'bench','perf'}:
        report.field('founders').equals(founders)
    if plan.kind == 'perf':
        report.field('ticks').equals(plan.ticks)
        report.field('ticks_per_day').equals(tpd)
        report.field('clock_offset_ticks').equals(clock)
    elif plan.kind in {'bench','nav'}:
        warmup = integer(report.field('warmup_ticks'))
        measured = integer(report.field('measured_ticks'))
        require(warmup + measured == plan.ticks, report.label, 'raw horizon differs')
        warm_alive = integer(native.field('warmup_alive'))
        intervention = native.field('interventions')
        names = {'gifts','gift_requested','gift_applied'}
        if plan.kind == 'bench':
            names |= {'wolves_requested','wolves_before','wolves_after'}
            report.field('alive_after_warmup').equals(warm_alive)
            report.field('gifts').equals(intervention.field('gifts').value)
            report.field('wolves').equals(intervention.field('wolves_requested').value)
        keys(intervention, names)
        for name in names:
            _ = integer(intervention.field(name))
    return PerfObservation(plan, Observation(0, plan.ticks, founders, alive, enabled, lost, '[]'), clock, tpd)


def paired_safety(base: PerfObservation, candidate: PerfObservation) -> dict[str, Json]:
    require((base.plan.kind,base.plan.ticks,base.plan.seed,base.plan.agents,base.clock_offset,base.ticks_per_day) ==
            (candidate.plan.kind,candidate.plan.ticks,candidate.plan.seed,candidate.plan.agents,candidate.clock_offset,candidate.ticks_per_day),
            '/pair', 'native performance workload differs')
    require(document(base.plan.workload_json).value == document(candidate.plan.workload_json).value,
            '/pair', 'scene or intervention setup differs')
    return ordinary_pair(base.world,candidate.world)
