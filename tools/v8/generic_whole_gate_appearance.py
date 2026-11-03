"""Seven original appearance worlds with source-qualified aggregate attribution."""
import json
import re
from pathlib import Path
from typing import Final

from gate_test_summary import validate_cases
from generic_gate import boundary, document, integer, require, text
from generic_observations import Observation, ProbeWorld, ordinary_pair, parse_probe
from generic_process import file_hash, real_path
from generic_test_endpoint import authenticate_endpoint
from generic_test_endpoint_types import bound_bytes, fields
from generic_test_history_graph import identity, resolve_graph
from generic_whole_gate_aggregate import aggregate_output, LIMIT
from generic_whole_gate_endpoints import bind_observer
from generic_whole_gate_m1 import sections
from generic_whole_gate_projection import project_endpoint
from generic_whole_gate_appearance_plan import appearance_plan, SAVE_TEST, BASELINE_TEST, KIN_TEST, RENDER_TEST
from mux2_gate_inputs import Json, Node

TESTS: Final = (SAVE_TEST, 'hair_and_variant_spread_elders_take_older_faces_children_the_child_band',
               'a_person_without_sex_counts_as_male_and_tops_follow_apparel', RENDER_TEST, BASELINE_TEST, KIN_TEST)
PINS: Final = {
    'rust/crates/sim-test/src/probe/gate_observation_mirror.rs': '3250c195fd648fd323f57b5d6508ca136cc63d3e940f35f35df65ad11c97b7e3',
    'rust/crates/sim-test/src/probe/gate_observation.rs': '23d328e105091e43c63eb24ae598e7c9901bf537e69d3d1bffb3b6fadc75fe61',
    'rust/crates/sim-test/tests/save_gate_observation/mod.rs': 'ab7c324c94846f231128040f7ccf34fbd2c8a9339a927a65962c1f2c17ce2cdf',
    'tools/v8/u_script.json': '32e9f6423867c858b678bcec9d1a14a170bba043917898c52434bcf00515f5b0',
    'rust/crates/sim-test/tests/engine_appearance_ffi.rs': 'b1a49c176714acc672fe3b4c47a038f98cef86d98751e9d9cc62ac0014107976',
    'rust/crates/sim-test/src/probe/ascii.rs': '8244f93ab9c6441ac13381e2563c42b5e7105b14d2b8cdeccc8e5faa0e7dba66',
}


@boundary
def renderer_mirrors(stage: Node, pid: int, script: tuple[Node, ...]) -> tuple[Observation, ...]:
    root = Path(text(stage.field('environment').field('WORLDSIM_GATE_TEST_OUT')))
    anchors = {name: text(Node(value)) for name, value in stage.field('observer_artifacts').mapping().items()}
    selected: dict[str, Observation] = {}
    sequences: dict[str, int] = {}
    outputs: dict[str, Path] = {}
    for path in root.glob('probe_pid*_world*.json'):
        require(str(path) in anchors, str(path), 'unbound appearance mirror')
        node = document(bound_bytes(path, anchors[str(path)]).decode())
        output = Path(text(node.field('local_output')))
        branch = 'render' if output.name == 'engine_appearance_ffi' else 'bridge-replay'
        if output.name != 'engine_appearance_ffi' and not (output.name == 'bridge-replay' and output.parent.name == 'engine_appearance_ffi'):
            continue
        real_path(path, directory=False)
        fields(node, 'schema process_id world_sequence thread_name local_output local_sidecar observation')
        node.field('schema').equals('worldsim_gate_test_observation_v1')
        node.field('process_id').equals(pid)
        node.field('thread_name').equals(RENDER_TEST)
        sequence = integer(node.field('world_sequence'))
        require(path.name == f'probe_pid{pid}_world{sequence}.json' and output.is_absolute() and '..' not in output.parts,
                str(path), 'foreign source mirror identity')
        node.field('local_sidecar').equals(str(output / 'gate_observations/colony_s100_primary.json'))
        require(branch not in selected, '/appearance', 'duplicate renderer occurrence')
        observation = parse_probe(json.dumps(node.field('observation').value), ProbeWorld('colony', 100, 'primary', 900))
        checkpoints = document('{"rows":'+observation.checkpoints_json+'}').field('rows').array()
        require(len(checkpoints) == len(script), str(path), 'renderer command checkpoints differ')
        for index, (checkpoint, entry) in enumerate(zip(checkpoints, script, strict=True)):
            checkpoint.field('phase').equals('before_tick_commands')
            checkpoint.field('tick').equals(entry.field('tick').value)
            checkpoint.field('script_index').equals(index)
            checkpoint.field('script_spec').equals(entry.value)
        selected[branch] = observation
        sequences[branch] = sequence
        outputs[branch] = output
    require(set(selected) == {'render', 'bridge-replay'}, '/appearance', 'missing original renderer occurrence')
    require(outputs['bridge-replay'] == outputs['render'] / 'bridge-replay', '/appearance', 'renderer directories differ')
    require(sequences['render'] < sequences['bridge-replay'], '/appearance', 'original renderer order changed')
    return selected['render'], selected['bridge-replay']


@boundary
def collect_appearance(session: Node, stage: Node) -> tuple[Observation, ...]:
    step = integer(stage.field('step'))
    require(step in {4, 5}, '/route', 'appearance requires original workspace stage')
    raw, _ = aggregate_output(session, stage)
    product = session.field('arms').field(text(stage.field('arm')))
    root = Path(text(product.field('root')))
    inputs = {name: text(Node(value)) for name, value in product.field('source_inputs').mapping().items()}
    for name, digest in PINS.items():
        require(inputs.get(name) == digest and file_hash(root/name) == digest, name, 'unregistered appearance source')
    headers = list(re.finditer(r'^\s*Running [^\n]+$', raw, re.MULTILINE))
    payloads = [raw[header.start():headers[index+1].start() if index+1 < len(headers) else len(raw)]
                for index, header in enumerate(headers) if 'tests/engine_appearance_ffi.rs (' in header.group()]
    require(len(payloads) == 1, '/appearance', 'missing or duplicated original target')
    selected = list(sections(payloads[0]))
    require(len(selected) == 1, '/appearance', 'missing or duplicated original target')
    require(selected[0].profile == ('debug' if step == 4 else 'release'), '/appearance', 'original profile differs')
    validate_cases(selected[0].summary, TESTS, ())
    evidence, graph, _ = bind_observer(stage, root, inputs, 'engine_appearance_ffi', TESTS)
    plans = appearance_plan(graph)
    endpoints = tuple(authenticate_endpoint(evidence, plan.occurrence) for plan in plans)
    histories = resolve_graph(graph, tuple(plan.occurrence for plan in plans),
                              {identity(plan.occurrence): endpoint for plan, endpoint in zip(plans, endpoints, strict=True)})
    for plan, endpoint in zip(plans, endpoints, strict=True):
        require(endpoint.stop_reason == plan.stop, '/appearance', 'original stop differs')
        require(endpoint.initial.knowledge_enabled == endpoint.terminal.knowledge_enabled == (plan.occurrence.test == SAVE_TEST),
                '/appearance', 'source-selected baseline/colony capability differs')
    return tuple(project_endpoint(endpoint, history) for endpoint, history in zip(endpoints, histories, strict=True)) + renderer_mirrors(stage, evidence.pid, tuple(document('{"rows":'+(root/'tools/v8/u_script.json').read_text()+'}').field('rows').array()))


@boundary
def evaluate_appearance(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    candidate.field('step').equals(base.field('step').value)
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    left, right = collect_appearance(session, base), collect_appearance(session, candidate)
    identities = ((SAVE_TEST, 3, 'primary'), (SAVE_TEST, 3, 'twin'), (SAVE_TEST, 3, 'loaded'),
                  (BASELINE_TEST, 0, 'primary'), (KIN_TEST, 3, 'primary'),
                  (RENDER_TEST, 100, 'render'), (RENDER_TEST, 100, 'bridge-replay'))
    return {'status': 'PASS', 'qualification': LIMIT, 'observed_worlds': 7,
            'safety': [ordinary_pair(a, b) for a, b in zip(left, right, strict=True)],
            'coverage': [{'target': 'engine_appearance_ffi', 'test': test, 'seed': seed, 'branch': branch,
                          'ordinary_safety': True, 'history_complete': True,
                          'base_end_tick': before.terminal_tick, 'candidate_end_tick': after.terminal_tick}
                         for (test, seed, branch), before, after in zip(identities, left, right, strict=True)],
            'mechanisms': list(TESTS[1:3]), 'native_outcomes': 6}
