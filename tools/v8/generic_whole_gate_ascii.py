"""Source-bound ASCII original worlds; broad workspace completion stays separate."""
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from gate_test_summary import validate_cases
from generic_gate import boundary, integer, require, text
from generic_observations import Observation, ordinary_pair
from generic_process import file_hash
from generic_test_endpoint import authenticate_endpoint
from generic_test_history_graph import identity, resolve_graph
from generic_test_plans import Planned
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from generic_whole_gate_ascii_plan import CARRIER, TESTS, ascii_plan, command_boundary
from generic_whole_gate_endpoints import bind_observer
from generic_whole_gate_m1 import sections
from generic_whole_gate_projection import project_endpoint
from mux2_gate_inputs import Json, Node

PINS: Final = {
    'rust/crates/sim-test/src/probe/colony.rs': '97d9bc0216240cfb91f39e52871d4227919ba7d379cfd9c3ee8f9eb8f9111cfd',
    'rust/crates/sim-test/tests/save_gate_observation/mod.rs': 'ab7c324c94846f231128040f7ccf34fbd2c8a9339a927a65962c1f2c17ce2cdf',
    'rust/crates/sim-test/tests/engine_ascii.rs': '4c3d79cd188e62126d7614a2498f102afb500344568dda24187da37abd7dfb63',
    'rust/crates/sim-test/src/probe/ascii.rs': '3875c7069915139df4202ed1fcb754c742e33323508c0bb8edd9772433333b4d',
}

@dataclass(frozen=True, slots=True)
class AsciiFamily:
    plans: tuple[Planned, ...]
    observations: tuple[Observation, ...]
    emitted_pid: int


@boundary
def collect_ascii(session: Node, stage: Node) -> AsciiFamily:
    step = integer(stage.field('step'))
    require(step in {4, 5}, '/route', 'ASCII requires original workspace stage')
    raw, _ = aggregate_output(session, stage)
    product = session.field('arms').field(text(stage.field('arm')))
    root = Path(text(product.field('root')))
    inputs = {name: text(Node(value)) for name, value in product.field('source_inputs').mapping().items()}
    for name, digest in PINS.items():
        require(inputs.get(name) == digest and file_hash(root/name) == digest, name, 'unregistered ASCII source')
    headers = list(re.finditer(r'^\s*Running [^\n]+$', raw, re.MULTILINE))
    payloads = [raw[header.start():headers[index+1].start() if index+1 < len(headers) else len(raw)]
                for index, header in enumerate(headers) if 'tests/engine_ascii.rs (' in header.group()]
    require(len(payloads) == 1, '/ascii', 'missing or duplicated original target')
    selected = sections(payloads[0])
    require(len(selected) == 1 and selected[0].target == 'engine_ascii', '/ascii', 'foreign original target')
    require(selected[0].profile == ('debug' if step == 4 else 'release'), '/ascii', 'original profile differs')
    validate_cases(selected[0].summary, TESTS, ())
    evidence, graph, _ = bind_observer(stage, root, inputs, 'engine_ascii', TESTS)
    plans = ascii_plan(graph)
    endpoints = tuple(authenticate_endpoint(evidence, plan.occurrence) for plan in plans)
    histories = resolve_graph(graph, tuple(plan.occurrence for plan in plans),
                              {identity(plan.occurrence): endpoint for plan, endpoint in zip(plans, endpoints, strict=True)})
    for plan, endpoint in zip(plans, endpoints, strict=True):
        require(endpoint.stop_reason == plan.stop, '/ascii', 'original stop differs')
        require(endpoint.initial.knowledge_enabled and endpoint.terminal.knowledge_enabled,
                '/ascii', 'source-selected colony capability differs')
        command_boundary(endpoint.initial)
        command_boundary(endpoint.terminal)
    return AsciiFamily(plans, tuple(project_endpoint(endpoint, history)
                                   for endpoint, history in zip(endpoints, histories, strict=True)), evidence.pid)


@boundary
def evaluate_ascii(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    candidate.field('step').equals(base.field('step').value)
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    left, right = collect_ascii(session, base), collect_ascii(session, candidate)
    coverage: list[Json] = []
    safety: list[Json] = []
    unsupported: list[Json] = []
    for a, b, before, after in zip(left.plans, right.plans, left.observations, right.observations, strict=True):
        require(identity(a.occurrence) == identity(b.occurrence), '/ascii', 'source occurrence differs between arms')
        require(not after.losses, '/ascii', 'ordinary candidate has KnowledgeLost, including an unresolved event window')
        matched = before.terminal_tick == after.terminal_tick
        if matched:
            safety.append(ordinary_pair(before, after))
        else:
            require(a.occurrence.test == CARRIER, '/ascii', 'fixed source endpoint differs')
            unsupported.append(f'engine_ascii carrier actual first-success horizons differ: BASE {before.terminal_tick}, candidate {after.terminal_tick}; no companion substituted')
        coverage.append({'target': 'engine_ascii', 'test': a.occurrence.test, 'seed': 100,
                         'branch': a.occurrence.branch, 'ordinary_safety': matched,
                         'status': 'PASS' if matched else 'INCOMPLETE',
                         'base_emitted_pid': left.emitted_pid, 'candidate_emitted_pid': right.emitted_pid,
                         'base_end_tick': before.terminal_tick, 'candidate_end_tick': after.terminal_tick})
    return {'status': 'INCOMPLETE' if unsupported else 'PASS', 'qualification': LIMIT,
            'safety': safety, 'coverage': coverage, 'unsupported': unsupported, 'native_outcomes': 6}
