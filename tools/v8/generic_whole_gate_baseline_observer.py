"""One source-owned baseline observer regression; not whole-library admission."""
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from gate_test_summary import parse_log
from generic_gate import boundary, document, integer, require, text
from generic_observations import Observation, ordinary_pair
from generic_process import file_hash, real_path
from generic_test_endpoint_types import bound_bytes, fields
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from generic_whole_gate_baseline_zero import parse_baseline_zero
from mux2_gate_inputs import Json, Node

CASE: Final = 'probe::gate_observation::tests::baseline_capability_is_explicit_and_observation_is_read_only'
PINS: Final = {'rust/crates/sim-core/src/rules/baseline.rs': 'd35d5f273ea51cc60c19dba9894cc62eb447120c6bd15b098035416f60c4a6ef',
 'rust/crates/sim-core/src/rules/mod.rs': '356d87da9a76b43eed2d594b5f4930e5ca3130060ca68b2387d1c0bb2e0336da',
 'rust/crates/sim-engine/src/lib.rs': '4281c820c3fa89b95588de0396bdf1233539f865a3b33856679b8520974003e8',
 'rust/crates/sim-systems/src/worldgen/mod.rs': '11525a81664d3206c72754f3772d1ba4e61cd3f9dbe2ab611be4e4124378b601',
 'rust/crates/sim-test/src/lib.rs': 'd5a126519bca132d236972c8813faf27a8f58dec8c514c1e1a96c95a16ec9a49',
 'rust/crates/sim-test/src/probe/gate_observation.rs': '23d328e105091e43c63eb24ae598e7c9901bf537e69d3d1bffb3b6fadc75fe61',
 'rust/crates/sim-test/src/probe/gate_observation_mirror.rs': '3250c195fd648fd323f57b5d6508ca136cc63d3e940f35f35df65ad11c97b7e3',
 'rust/crates/sim-test/src/probe/gate_observation_tests.rs': '72c72348111f32ff0a71e39cc365cb6e55c83c3eaad3af981850e1f6b4d1171c',
 'rust/crates/sim-test/src/probe/mod.rs': '9d4f6bc6f149b52927f60be6c190fb2526cecfcc1b7d77ebb02b75539039e7ae',
 'rust/crates/sim-test/src/scenes.rs': '7fd693a1db13dafcbce414897ea0f98f2d40e888e8a1085532b8438c13418173'}


@dataclass(frozen=True, slots=True)
class NamedBaseline:
    pid: int
    sequence: int
    observation: Observation


@boundary
def parse_named(path: Path, node: Node) -> NamedBaseline:
    fields(node, 'schema process_id world_sequence thread_name local_output local_sidecar observation')
    node.field('schema').equals('worldsim_gate_test_observation_v1')
    node.field('thread_name').equals(CASE)
    pid = integer(node.field('process_id'))
    sequence = integer(node.field('world_sequence'))
    require(pid > 0 and path.name == f'probe_pid{pid}_world{sequence}.json', '/identity', 'wrong emitted identity')
    output = Path(text(node.field('local_output')))
    require(output.is_absolute() and '..' not in output.parts
            and re.fullmatch(r'gate-observer-' + str(pid) + r'-[0-9]+', output.name) is not None,
            '/output', 'not source-owned temporary output')
    node.field('local_sidecar').equals(str(output / 'gate_observations/baseline_s0_primary.json'))
    observation = parse_baseline_zero(node.field('observation'))
    node.field('observation').field('knowledge_capability').field('enabled').equals(False)
    require(observation.checkpoints_json == '[]', '/checkpoints', 'source case has no intervention')
    return NamedBaseline(pid, sequence, observation)


@boundary
def select_named(rows: tuple[tuple[Path, Node], ...]) -> NamedBaseline:
    selected = [(path, node) for path, node in rows if node.field('thread_name').value == CASE]
    require(len(selected) == 1, '/named-baseline', 'missing or duplicate named native world')
    return parse_named(*selected[0])


@boundary
def check_sources(product: Node) -> None:
    source_root = Path(text(product.field('root')))
    for relative, digest in PINS.items():
        product.field('source_inputs').field(relative).equals(digest)
        require(file_hash(source_root / relative) == digest, relative, 'source classification changed')


@boundary
def collect(session: Node, stage: Node) -> NamedBaseline:
    step = integer(stage.field('step'))
    require(step in {4, 5}, '/route', 'original workspace route required')
    check_sources(session.field('arms').field(text(stage.field('arm'))))
    raw, _ = aggregate_output(session, stage)
    profile = 'debug' if step == 4 else 'release'
    headers = list(re.finditer(r'^\s*(?:Running (?:unittests )?[^\n]+|Doc-tests [^\n]+)\s*$', raw, re.MULTILINE))
    scopes = [(i, h) for i, h in enumerate(headers) if re.fullmatch(
        r'\s*Running unittests src/lib\.rs \([^\n]*/' + profile + r'/deps/sim_test-[^/()]+\)\s*', h.group())]
    require(len(scopes) == 1, '/library', 'missing or duplicate actual library')
    index, header = scopes[0]
    summary = parse_log(raw[header.end():headers[index + 1].start() if index + 1 < len(headers) else len(raw)])
    selected = [row for row in summary.outcomes if row.name == CASE]
    require(summary.summaries == 1 and summary.failed == 0 and len(selected) == 1
            and selected[0].status == 'ok', '/case', 'original selected oracle missing or failed')
    environment = stage.field('environment')
    environment.field('WORLDSIM_GATE_OBSERVER').equals('1')
    root = Path(text(environment.field('WORLDSIM_GATE_TEST_OUT')))
    real_path(root, directory=True)
    require(root.is_relative_to(Path(text(stage.field('process_receipt'))).parent.parent), '/root', 'foreign observer root')
    stage.field('observer_directories').field(str(root)).equals(root.stat().st_mode)
    rows: list[tuple[Path, Node]] = []
    for path in root.glob('probe_pid*_world*.json'):
        digest = text(stage.field('observer_artifacts').field(str(path)))
        rows.append((path, document(bound_bytes(path, digest).decode())))
    return select_named(tuple(rows))


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    candidate.field('step').equals(base.field('step').value)
    left, right = collect(session, base), collect(session, candidate)
    return {'status': 'PASS', 'qualification': LIMIT, 'publication_authority': False,
            'scope': 'ONE_NAMED_BASELINE_OBSERVER_WORLD_ONLY', 'library_complete': False,
            'safety': [ordinary_pair(left.observation, right.observation)],
            'coverage': [{'target': 'sim_test', 'test': CASE, 'scene': 'baseline', 'seed': 0,
                          'ticks': 0, 'base_emitted_pid': left.pid, 'candidate_emitted_pid': right.pid,
                          'base_world_sequence': left.sequence, 'candidate_world_sequence': right.sequence}]}
