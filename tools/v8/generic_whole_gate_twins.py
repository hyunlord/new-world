"""Source-qualified workspace twin collection; aggregate receipts never imply child waits."""
import re
from pathlib import Path

from gate_test_summary import parse_log
from generic_gate import boundary, document, integer, require, text
from generic_process import file_hash, real_path
from generic_sources import manifest_hash
from generic_test_endpoint_types import bound_bytes
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from generic_whole_gate_twins_plan import CASES, Twin, pair_twins, select_twins
from generic_whole_gate_twins_sources import PINS, SIM_TEST_TREE
from mux2_gate_inputs import Json, Node


@boundary
def check_sources(product: Node) -> None:
    root = Path(text(product.field('root')))
    inputs = product.field('source_inputs')
    for name, digest in PINS.items():
        inputs.field(name).equals(digest)
        require(file_hash(root / name) == digest, name, 'twin source classification changed')
    prefix = 'rust/crates/sim-test/src/'
    actual: dict[str, Json] = {p.relative_to(root).as_posix(): file_hash(p)
                               for p in (root / prefix).rglob('*.rs')}
    declared = {name: value for name, value in inputs.mapping().items()
                if name.startswith(prefix) and name.endswith('.rs')}
    require(actual == declared and manifest_hash(actual) == SIM_TEST_TREE,
            '/sources', 'reviewed Temp prefix producer namespace changed')


@boundary
def library_scope(raw: str, profile: str) -> None:
    headers = list(re.finditer(r'^\s*(?:Running (?:unittests )?[^\n]+|Doc-tests [^\n]+)\s*$', raw, re.MULTILINE))
    scopes = [(i, h) for i, h in enumerate(headers) if re.fullmatch(
        r'\s*Running unittests src/lib\.rs \([^\n]*/' + profile + r'/deps/sim_test-[^/()]+\)\s*', h.group())]
    require(len(scopes) == 1, '/library', 'missing or duplicate source-selected library')
    index, header = scopes[0]
    summary = parse_log(raw[header.end():headers[index + 1].start() if index + 1 < len(headers) else len(raw)])
    names = {row.name: row.status for row in summary.outcomes}
    require(summary.summaries == 1 and summary.failed == 0 and len(names) == len(summary.outcomes)
            and all(names.get(name) == 'ok' for name in CASES.values()),
            '/case', 'missing, duplicate or failed original twin assertions')


@boundary
def collect(session: Node, stage: Node) -> tuple[Twin, ...]:
    step = integer(stage.field('step'))
    require(step in {4, 5}, '/route', 'twins only selected by original workspace routes; step17 excludes them')
    check_sources(session.field('arms').field(text(stage.field('arm'))))
    raw, _ = aggregate_output(session, stage)
    library_scope(raw, 'debug' if step == 4 else 'release')
    environment = stage.field('environment')
    environment.field('WORLDSIM_GATE_OBSERVER').equals('1')
    root = Path(text(environment.field('WORLDSIM_GATE_TEST_OUT')))
    real_path(root, directory=True)
    require(root.is_relative_to(Path(text(stage.field('process_receipt'))).parent.parent), '/root', 'foreign mirror root')
    stage.field('observer_directories').field(str(root)).equals(root.stat().st_mode)
    rows: list[tuple[Path, Node]] = []
    paths = set(root.glob('probe_pid*_world*.json'))
    anchors = stage.field('observer_artifacts').mapping()
    expected = {Path(name) for name in anchors if Path(name).parent == root
                and re.fullmatch(r'probe_pid[0-9]+_world[0-9]+\.json', Path(name).name)}
    require(paths == expected, '/mirrors', 'stage-bound mirror inventory differs')
    for path in sorted(paths):
        digest = text(Node(anchors[str(path)]))
        rows.append((path, document(bound_bytes(path, digest).decode())))
    return select_twins(tuple(rows))


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    candidate.field('step').equals(base.field('step').value)
    result = pair_twins(collect(session, base), collect(session, candidate))
    result.update(qualification=LIMIT, publication_authority=False,
                  scope='EIGHT_SOURCE_SELECTED_TWIN_WORLDS_ONLY',
                  original_oracle='SOURCE_ASSERTION_ONLY_SUMMARY_OR_RAW_SAMPLE_BYTES',
                  independent_child_wait='NOT_RECORDED')
    return result
