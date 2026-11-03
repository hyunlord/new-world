"""Six source-selected knowledge/default-policy worlds under original workspace tests."""
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from gate_test_summary import validate_cases
from generic_gate import boundary, integer, require, text
from generic_observations import Observation, ordinary_pair
from generic_process import file_hash
from generic_test_endpoint import authenticate_endpoint
from generic_test_endpoint_types import ExpectedOccurrence
from generic_test_history_graph import identity, resolve_graph
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from generic_whole_gate_endpoints import bind_observer
from generic_whole_gate_m1 import sections
from generic_whole_gate_projection import project_endpoint
from mux2_gate_inputs import Json, Node

ROWS: Final = {
    'engine_knowledge_ffi': (
        ('knowledge_reads_are_empty_in_the_baseline_world', 3, 1, False),
        ('the_tech_view_lists_every_node_in_id_order_with_the_aggregate', 1, 241, True),
        ('node_and_person_views_agree_with_the_component', 1, 241, True),
        ('teach_orders_are_listed_until_cancelled', 1, 243, True)),
    'm6b_presentation_defaults': (
        ('default_steward_policy_matches_published_base', 0, 0, True),
        ('default_steward_does_not_accumulate_experimental_grace_state', 0, 1440, True)),
}
PINS: Final = {
    'rust/crates/sim-core/data/colony/steward.ron': 'f46c569e827dbf7c2fb755d72d86a6141c573f928808cb8c02a81065fbebb6bf',
    'rust/crates/sim-test/tests/fixtures/m6b_base_steward.ron': 'f079e3ef46a7ffa7bf789a5910048b77de2ff818aeb9e98066de8963c5535df6',
    'rust/crates/sim-test/tests/save_gate_observation/mod.rs': 'ab7c324c94846f231128040f7ccf34fbd2c8a9339a927a65962c1f2c17ce2cdf',
    'rust/crates/sim-test/tests/engine_knowledge_ffi.rs': 'd57ea436bc99fe5c804812e92b404156536a9a3f26aa8571965fe87623e6281e',
    'rust/crates/sim-test/tests/m6b_presentation_defaults.rs': 'e1c52685391c4f0944f6766b68370bf5303faa76aedec335944093f79ae1ab53',
}

@dataclass(frozen=True, slots=True)
class KnowledgeDefaults:
    observations: tuple[Observation, ...]
    emitted_pid: int


@boundary
def collect_knowledge_defaults(session: Node, stage: Node, target: str) -> KnowledgeDefaults:
    step = integer(stage.field('step'))
    require(step in {4, 5} and target in ROWS, '/route', 'unregistered original knowledge/defaults route')
    raw, _ = aggregate_output(session, stage)
    product = session.field('arms').field(text(stage.field('arm')))
    root = Path(text(product.field('root')))
    inputs = {name: text(Node(value)) for name, value in product.field('source_inputs').mapping().items()}
    for name, digest in PINS.items():
        require(inputs.get(name) == digest and file_hash(root/name) == digest, name, 'unregistered knowledge/defaults source')
    headers = list(re.finditer(r'^\s*Running [^\n]+$', raw, re.MULTILINE))
    payloads = [raw[header.start():headers[index+1].start() if index+1 < len(headers) else len(raw)]
                for index, header in enumerate(headers) if f'tests/{target}.rs (' in header.group()]
    require(len(payloads) == 1, target, 'missing or duplicated original target')
    selected = sections(payloads[0])
    require(len(selected) == 1 and selected[0].target == target, target, 'foreign original target')
    require(selected[0].profile == ('debug' if step == 4 else 'release'), target, 'original profile differs')
    tests = tuple(row[0] for row in ROWS[target])
    validate_cases(selected[0].summary, tests, ())
    evidence, graph, _ = bind_observer(stage, root, inputs, target, tests)
    expected = tuple(ExpectedOccurrence(test, test, seed, 'primary', 0, end, True)
                     for test, seed, end, _ in ROWS[target])
    endpoints = tuple(authenticate_endpoint(evidence, row) for row in expected)
    histories = resolve_graph(graph, expected, {identity(row): endpoint
                              for row, endpoint in zip(expected, endpoints, strict=True)})
    for (_, _, _, capability), endpoint in zip(ROWS[target], endpoints, strict=True):
        require(endpoint.stop_reason == 'original_endpoint', target, 'original assertion endpoint differs')
        require(endpoint.initial.knowledge_enabled == endpoint.terminal.knowledge_enabled == capability,
                target, 'source-selected baseline/colony capability differs')
    return KnowledgeDefaults(tuple(project_endpoint(endpoint, history)
                                   for endpoint, history in zip(endpoints, histories, strict=True)), evidence.pid)


@boundary
def evaluate_knowledge_defaults(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    candidate.field('step').equals(base.field('step').value)
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    safety: list[Json] = []
    coverage: list[Json] = []
    for target, rows in ROWS.items():
        left, right = collect_knowledge_defaults(session, base, target), collect_knowledge_defaults(session, candidate, target)
        for (test, seed, end, _), before, after in zip(rows, left.observations, right.observations, strict=True):
            safety.append(ordinary_pair(before, after))
            coverage.append({'target': target, 'test': test, 'seed': seed, 'branch': 'primary',
                             'ordinary_safety': True, 'end_tick': end,
                             'base_emitted_pid': left.emitted_pid, 'candidate_emitted_pid': right.emitted_pid})
    return {'status': 'PASS', 'qualification': LIMIT, 'safety': safety, 'coverage': coverage, 'native_outcomes': 6}
