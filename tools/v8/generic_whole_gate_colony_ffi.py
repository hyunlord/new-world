"""Two source-selected tick-zero colony FFI worlds, including explicit Carry insertion."""
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from gate_test_summary import validate_cases
from generic_gate import boundary, document, integer, require, text
from generic_observations import Observation, ordinary_pair
from generic_process import file_hash
from generic_test_endpoint import Endpoint, authenticate_endpoint
from generic_test_endpoint_types import (
    ExpectedOccurrence,
    bound_bytes,
    fields,
    snapshot,
)
from generic_test_history_graph import identity, resolve_graph
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from generic_whole_gate_endpoints import bind_observer
from generic_whole_gate_m1 import sections
from generic_whole_gate_projection import project_endpoint
from mux2_gate_inputs import Json, Node

TARGET: Final = 'engine_colony_ffi'
TESTS: Final = ('colony_reads_follow_the_orders', 'agent_job_columns_match_the_snapshot_rows')
PINS: Final = {'rust/crates/sim-bridge/src/ffi/colony.rs': '8ac2a8e5e595bf4ec1bafe70d3121af27a77dfdf7bf855e9943882651d7017da',
 'rust/crates/sim-bridge/src/ffi/mod.rs': 'e81cd6a9f329474c579b8f3d03e6ab43069d4464d139ae4fac192c54c85ba945',
 'rust/crates/sim-core/src/colony/command.rs': 'e34ac1d2a0715d971f957b9760609475e15f0f09bb2a5ba2b16741b41b2de05b',
 'rust/crates/sim-core/src/rules/colony.rs': '3fa9680569223cb974acdcf7247f0aabf8296da707752f17ac24e94f02471946',
 'rust/crates/sim-systems/src/commands.rs': '43ca9cf7be9f8e4dce24535566bf781dbe836e4624b4df119e41ac6259682897',
 'rust/crates/sim-systems/src/worldgen/mod.rs': '11525a81664d3206c72754f3772d1ba4e61cd3f9dbe2ab611be4e4124378b601',
 'rust/crates/sim-test/src/lib.rs': 'd5a126519bca132d236972c8813faf27a8f58dec8c514c1e1a96c95a16ec9a49',
 'rust/crates/sim-test/src/probe/colony.rs': 'b04d435eaf9be48eaee11a631982fa44b932fbca5f484da7c9e263cfdae30365',
 'rust/crates/sim-test/src/scenes.rs': '7fd693a1db13dafcbce414897ea0f98f2d40e888e8a1085532b8438c13418173',
 'rust/crates/sim-test/tests/engine_colony_ffi.rs': '0666f7abbb4fd7deb11cabb0ec8347c231a5c546cee16d7c94561a327230d366',
 'rust/crates/sim-test/tests/save_gate_observation/external.rs': '8ddf8a602f0eb81f819c9929a3ed1cc54133262c2d264dc126030f60ee9e2bcc',
 'rust/crates/sim-test/tests/save_gate_observation/mod.rs': 'ab7c324c94846f231128040f7ccf34fbd2c8a9339a927a65962c1f2c17ce2cdf'}


@dataclass(frozen=True, slots=True)
class ColonyFFI:
    observations: tuple[Observation, ...]
    emitted_pid: int
    commands: tuple[Json, ...]
    intervention: Json


def command_evidence(endpoint: Endpoint) -> tuple[Json, ...]:
    """Three actual player command records; coordinates remain actual evidence."""
    require(not endpoint.initial.commands and len(endpoint.terminal.commands) == 3, '/commands', 'original three accepted commands required')
    commands = endpoint.terminal.commands
    values = tuple(document(row.command_json) for row in commands)
    for row, node, name in zip(commands, values, ('designate_chop_tiles', 'add_stockpile', 'place_blueprint'), strict=True):
        require(row.tick == 0 and row.accepted, '/commands', 'wrong command tick or rejected original command')
        node.field('tick').equals(0)
        node.field('cmd').equals(name)
    fields(values[0], 'tick cmd tiles')
    tiles = values[0].field('tiles').array()
    require(len(tiles) == 3 and commands[0].count == 3, '/tiles', 'three tree orders required')
    coordinates: list[tuple[int, ...]] = []
    for tile in tiles:
        coords = tuple(integer(value) for value in tile.array())
        require(len(coords) == 2, '/tiles', 'tile is not a coordinate pair')
        coordinates.append(coords)
    require(len(set(coordinates)) == 3, '/tiles', 'duplicate selected tree')
    fields(values[1], 'tick cmd x0 y0 x1 y1')
    x0, y0, x1, y1 = (integer(values[1].field(key)) for key in ('x0', 'y0', 'x1', 'y1'))
    require(x1 == x0 + 1 and y1 == y0 + 1 and commands[1].count == 4, '/stockpile', 'source two-by-two stockpile differs')
    fields(values[2], 'tick cmd kind x y')
    values[2].field('kind').equals('hut')
    require(commands[2].count == 1, '/blueprint', 'one accepted hut blueprint required')
    _ = integer(values[2].field('x'))
    _ = integer(values[2].field('y'))
    return tuple(node.value for node in values)


def carry_evidence(node: Node, endpoint: Endpoint) -> Json:
    """Source assertion + retained annotation; no claim this mutation is a player command."""
    fields(node, 'state detail')
    require(snapshot(node.field('state')) == endpoint.terminal, '/carry', 'annotation is not actual terminal state')
    require(not endpoint.initial.commands and not endpoint.terminal.commands, '/carry', 'direct Carry insertion must not be relabelled as a command')
    detail = node.field('detail')
    fields(detail, 'kind selected_row selected_entity_bits snapshot_entity_bits jobs item count')
    detail.field('kind').equals('direct_carry_insert')
    detail.field('item').equals('Log')
    detail.field('count').equals(4)
    detail.field('selected_row').equals(2)
    entities = [integer(value) for value in detail.field('snapshot_entity_bits').array()]
    require(len(entities) == len(set(entities)) == len(endpoint.terminal.raw_agent_ids) and len(entities) > 2,
            '/carry', 'actual snapshot row census differs')
    detail.field('selected_entity_bits').equals(entities[2])
    jobs = detail.field('jobs').array()
    require(len(jobs) == len(entities), '/carry', 'job/snapshot row alignment differs')
    for index, job in enumerate(jobs):
        job.equals([0, 0, 4] if index == 2 else [0, -1, 0])
    return detail.value


@boundary
def collect(session: Node, stage: Node) -> ColonyFFI:
    step = integer(stage.field('step'))
    require(step in {4, 5, 17}, '/route', 'unregistered original colony FFI route')
    raw, _ = aggregate_output(session, stage)
    product = session.field('arms').field(text(stage.field('arm')))
    root = Path(text(product.field('root')))
    inputs = {name: text(Node(value)) for name, value in product.field('source_inputs').mapping().items()}
    for name, digest in PINS.items():
        require(inputs.get(name) == digest and file_hash(root / name) == digest, name, 'source classification changed')
    headers = list(re.finditer(r'^\s*(?:Running (?:unittests )?[^\n]+|Doc-tests [^\n]+)\s*$', raw, re.MULTILINE))
    payloads = [raw[header.start():headers[index + 1].start() if index + 1 < len(headers) else len(raw)]
                for index, header in enumerate(headers) if f'Running tests/{TARGET}.rs (' in header.group()]
    require(len(payloads) == 1, TARGET, 'missing or duplicate original target')
    selected = sections(payloads[0])
    require(len(selected) == 1 and selected[0].target == TARGET
            and selected[0].profile == ('debug' if step == 4 else 'release'), TARGET, 'wrong target or profile')
    validate_cases(selected[0].summary, TESTS, ())
    evidence, graph, folder = bind_observer(stage, root, inputs, TARGET, TESTS)
    expected = tuple(ExpectedOccurrence(test, test, seed, 'primary', 0, 0, seed == 0) for seed, test in enumerate(TESTS))
    endpoints = tuple(authenticate_endpoint(evidence, row) for row in expected)
    histories = resolve_graph(graph, expected, {identity(row): endpoint for row, endpoint in zip(expected, endpoints, strict=True)})
    for endpoint in endpoints:
        require(endpoint.stop_reason == 'original_endpoint' and endpoint.initial.knowledge_enabled
                and endpoint.terminal.knowledge_enabled, TARGET, 'wrong endpoint or colony knowledge capability')
        require(endpoint.initial.alive_ids == endpoint.initial.raw_agent_ids == endpoint.terminal.alive_ids
                == endpoint.terminal.raw_agent_ids, TARGET, 'tick-zero source actions changed live/raw census')
    commands = command_evidence(endpoints[0])
    path = folder / identity(expected[1]) / 'checkpoint_carry_intervention.json'
    relative = path.relative_to(evidence.root).as_posix()
    anchors = dict(evidence.output_hashes)
    require(relative in anchors, '/carry', 'missing retained intervention annotation')
    intervention = carry_evidence(document(bound_bytes(path, anchors[relative]).decode()), endpoints[1])
    return ColonyFFI(tuple(project_endpoint(endpoint, history) for endpoint, history in zip(endpoints, histories, strict=True)),
                     evidence.pid, commands, intervention)


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    candidate.field('step').equals(base.field('step').value)
    left, right = collect(session, base), collect(session, candidate)
    safety: list[Json] = [ordinary_pair(a, b) for a, b in zip(left.observations, right.observations, strict=True)]
    coverage: list[Json] = [{'target': TARGET, 'test': test, 'seed': seed, 'ticks': 0,
                            'classification': 'PLAYER_COMMANDS_TICK0' if seed == 0 else 'CONTROLLED_DIRECT_CARRY_INSERT_TICK0',
                            'strict_safety_applied': True, 'base_emitted_pid': left.emitted_pid,
                            'candidate_emitted_pid': right.emitted_pid} for seed, test in enumerate(TESTS)]
    return {'status': 'PASS', 'qualification': LIMIT, 'safety': safety, 'coverage': coverage,
            'base_commands': list(left.commands), 'candidate_commands': list(right.commands),
            'base_carry': left.intervention, 'candidate_carry': right.intervention,
            'scope': 'TWO_ORIGINAL_COLONY_FFI_CASES_ONLY', 'publication_authority': False}
