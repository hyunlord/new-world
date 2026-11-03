# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_native
"""Source-selected ordinary test endpoints in actual whole-shell aggregates.

No child wait or selected native executable is invented. Complete gate-family
admission still requires the remaining dynamic/native mechanism joins.
"""
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_gate import boundary, integer, require, text
from generic_observations import Observation, ordinary_pair
from generic_process import file_hash
from generic_test_endpoint import Endpoint, authenticate_endpoint
from generic_whole_gate_endpoints import bind_observer
from generic_test_plans import SOURCE_PINS as BASE_PINS, TESTS as BASE_TESTS, Planned, fixed_plan, save_plan, hut_evidence
from generic_test_steward_plans import SOURCE_PINS as STEWARD_PINS, TESTS as STEWARD_TESTS, IGNORED, steward_plan

SOURCE_PINS = {**BASE_PINS, **STEWARD_PINS}
TESTS = {**BASE_TESTS, **STEWARD_TESTS}
from generic_test_history_graph import ResolvedHistory, identity, resolve_graph
from mux2_gate_inputs import Json, Node

FIXED: Final = ('colony_commands', 'colony_world')
TARGETS: Final = {4: (*FIXED, 'save_load', 'colony_jobs', 'colony_steward', 'steward_defence'), 5: (*FIXED, 'save_load', 'colony_jobs', 'colony_steward', 'steward_defence'),
                 17: (*FIXED, 'colony_jobs'), 24: ('save_load',), 37: ('colony_steward', 'steward_defence')}
STEPS: Final = frozenset(TARGETS)
from generic_whole_gate_aggregate import LIMIT as LIMIT, aggregate_output as aggregate_output


@dataclass(frozen=True, slots=True)
class Family:
    target: str
    emitted_pid: int
    aggregate_pid: int
    aggregate_exit: int
    endpoints: tuple[Endpoint, ...]
    histories: tuple[ResolvedHistory, ...]
    plans: tuple[Planned, ...]
    setup: dict[str, Json]


def _files(node: Node) -> dict[str, str]:
    return {name: text(Node(value)) for name, value in node.mapping().items()}


def _project(endpoint: Endpoint, history: ResolvedHistory) -> Observation:
    require(history.history_complete and not history.pending, '/history', 'unresolved native endpoint')
    founding = history.founding
    require(founding is not None and founding.tick == 0, '/founding', 'tick-zero founding required')
    assert founding is not None
    end = endpoint.terminal
    require(founding.knowledge_enabled == end.knowledge_enabled, '/knowledge', 'capability changed')
    losses = () if end.losses is None else tuple((row.tick, row.node, row.holder) for row in end.losses)
    return Observation(0, end.tick, len(founding.alive_ids), len(end.alive_ids),
                       end.knowledge_enabled, losses, '[]')


def _native_segment(stdout: str, target: str) -> str:
    headers = list(re.finditer(r'^\s*Running (?:unittests |tests/)?([^\s(]+)(?:\.rs)?\s+\([^\n]+\)\s*$', stdout, re.MULTILINE))
    selected: list[str] = []
    for index, header in enumerate(headers):
        source = header.group(1).removesuffix('.rs')
        if Path(source).name == target:
            finish = headers[index + 1].start() if index + 1 < len(headers) else len(stdout)
            selected.append(stdout[header.end():finish])
    require(len(selected) == 1, target, 'missing or repeated actual native target output')
    segment = selected[0]
    outcomes = [(row.group(1), row.group(2)) for row in re.finditer(r'^test ([A-Za-z0-9_:]+) \.\.\. (ok|FAILED|ignored)(?:, [^\n]+)?\s*$', segment, re.MULTILINE)]
    require(len(outcomes) == len(TESTS[target]) and {name for name, _ in outcomes} == set(TESTS[target]),
            target, 'missing, extra or duplicate original native outcome')
    ignored: set[str] = {IGNORED} if target == 'colony_steward' else set()
    require(all(status == ('ignored' if name in ignored else 'ok') for name, status in outcomes), target, 'native outcome differs from selected scope')
    summaries = re.findall(r'^test result: ok\. ([0-9]+) passed; 0 failed; ([0-9]+) ignored; 0 measured; 0 filtered out; finished in .+$', segment, re.MULTILINE)
    require(summaries == [(str(len(TESTS[target])-len(ignored)), str(len(ignored)))], target, 'missing original complete test summary')
    return segment


@boundary
def collect_family(session: Node, stage: Node, target: str) -> Family:
    """Consume source-bound fixed family emitted under a real aggregate return."""
    require(integer(stage.field('step')) in STEPS and target in TARGETS[integer(stage.field('step'))], '/family', 'unsupported source route')
    raw, aggregate_pid = aggregate_output(session, stage)
    arm = text(stage.field('arm'))
    product = session.field('arms').field(arm)
    source = f'rust/crates/sim-test/tests/{target}.rs'
    inputs = _files(product.field('source_inputs'))
    root = Path(text(product.field('root')))
    require(inputs.get(source) == SOURCE_PINS[target] and file_hash(root / source) == SOURCE_PINS[target],
            source, 'unregistered source-specific native plan')
    _ = _native_segment(raw, target)
    evidence, graph, folder = bind_observer(stage, root, inputs, target, TESTS[target])
    emitted_pid = evidence.pid
    setup: dict[str, Json] = {}
    match target:
        case 'save_load': plans = save_plan(graph)
        case 'colony_jobs':
            plan, setup = hut_evidence(graph)
            plans = (plan,)
        case 'colony_commands' | 'colony_world': plans = fixed_plan(target)
        case 'colony_steward' | 'steward_defence': plans, setup = steward_plan(graph, target)
        case _: raise AssertionError('source route checked above')
    expected_names = {identity(plan.occurrence) for plan in plans}
    require({p.name for p in folder.iterdir()} == expected_names, target, 'unexpected or missing ordinary occurrence')
    endpoints = tuple(authenticate_endpoint(evidence, plan.occurrence) for plan in plans)
    histories = resolve_graph(graph, tuple(plan.occurrence for plan in plans),
                              {identity(plan.occurrence): endpoint for plan, endpoint in zip(plans, endpoints, strict=True)})
    for plan, endpoint, history in zip(plans, endpoints, histories, strict=True):
        require(endpoint.stop_reason == plan.stop, target, 'different native stop condition')
        if plan.occurrence.ordinary:
            _ = _project(endpoint, history)
    return Family(target, emitted_pid, aggregate_pid, 0, endpoints, histories, plans, setup)


@boundary
def evaluate_family(session: Node, base_stage: Node, candidate_stage: Node) -> dict[str, Json]:
    """Check known ordinary subsets without upgrading them to whole-family coverage."""
    step = integer(base_stage.field('step'))
    candidate_stage.field('step').equals(step)
    base_stage.field('arm').equals('base')
    candidate_stage.field('arm').equals('candidate')
    if step in {42, 43, 49}:
        from generic_whole_gate_library import evaluate_library
        return evaluate_library(session, base_stage, candidate_stage)
    if step not in STEPS:
        return {'status': 'INCOMPLETE', 'safety': [], 'coverage': [],
                'unsupported': [f'original native stage {step}: dynamic/mechanism source route not yet bound'], 'qualification': LIMIT}
    mechanism: list[Json] = []
    if step in {4, 5, 37}:
        for arm, stage in (('base', base_stage), ('candidate', candidate_stage)):
            raw, pid = aggregate_output(session, stage)
            product = session.field('arms').field(arm)
            root = Path(text(product.field('root')))
            for target, digest in STEWARD_PINS.items():
                source = f'rust/crates/sim-test/tests/{target}.rs'
                product.field('source_inputs').field(source).equals(digest)
                require(file_hash(root/source) == digest, source, 'unregistered gate37 mechanism map')
                _ = _native_segment(raw, target)
            mechanism.append({'arm': arm, 'aggregate_pid': pid, 'selected_active': 70, 'selected_ignored': 1,
                              'policy': 'source-mapped partial/lethal/data assertions retained; no ordinary floor for mechanisms'})
    safety: list[Json] = []
    coverage: list[Json] = []
    for target in TARGETS[step]:
        left, right = collect_family(session, base_stage, target), collect_family(session, candidate_stage, target)
        require(left.plans == right.plans, target, 'actual BASE and candidate horizons differ')
        require(left.setup == right.setup, target, 'declared gifted setup differs between arms')
        for plan, before, after, left_history, right_history in zip(left.plans, left.endpoints, right.endpoints, left.histories, right.histories, strict=True):
            if plan.occurrence.ordinary:
                safety.append(ordinary_pair(_project(before, left_history), _project(after, right_history)))
            coverage.append({'target': target, 'test': plan.occurrence.test, 'seed': plan.occurrence.seed,
                             'branch': plan.occurrence.branch, 'ordinary_safety': plan.occurrence.ordinary, 'base_emitted_pid': left.emitted_pid,
                             'candidate_emitted_pid': right.emitted_pid})
    replay_scope: dict[str, Json] = {}
    if step in {4, 5, 17}:
        from generic_whole_gate_native_probes import evaluate_replays
        replay_scope = evaluate_replays(session, base_stage, candidate_stage)
        safety.extend(replay_scope['safety'] if isinstance(replay_scope['safety'], list) else [])
        coverage.extend(replay_scope['coverage'] if isinstance(replay_scope['coverage'], list) else [])
    if step == 17:
        from generic_whole_gate_m1 import collect_m1
        mechanism.extend([collect_m1(session, stage) for stage in (base_stage, candidate_stage)])
    migrations: dict[str, Json] = {}
    if step == 37:
        from generic_whole_gate_migrations import evaluate_migrations
        migrations = evaluate_migrations(session, base_stage, candidate_stage)
        coverage.extend(migrations['coverage'] if isinstance(migrations['coverage'], list) else [])
    gaps: list[Json] = []
    if step in {4, 5}:
        gaps = ['workspace outcome accounting outside the source-mapped fixed/save/hut/steward families remains unjoined']
    return {'status': 'PASS' if step in {17, 24} else 'INCOMPLETE', 'safety': safety, 'coverage': coverage,
            'unsupported': migrations.get('unsupported', []) if step == 37 else gaps,
            'mechanisms': mechanism, 'native_probe_replays': replay_scope, 'migrations': migrations, 'qualification': LIMIT}
