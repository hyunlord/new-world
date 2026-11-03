"""Original16 headless no-world oracles through source-bound aggregate evidence."""
from pathlib import Path

from gate_oracle_child import SUITE_NAMES
from generic_gate import boundary, require
from generic_process import file_hash
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from mux2_gate_inputs import Json, Node

from generic_oracle_semantics import NO_WORLD_PINS, PROTOCOL_PINS, audit_log
from generic_whole_gate_renderer import original_anchors


@boundary
def collect(session: Node, stage: Node) -> dict[str, Json]:
    stage.field('step').equals(38)
    stdout, pid = aggregate_output(session, stage)
    arm = stage.field('arm').text()
    product = session.field('arms').field(arm)
    root = Path(product.field('root').text())
    log = Path(session.field('output').text()) / arm / 'mux2-oracles'
    pins = {**NO_WORLD_PINS, **{'tools/v8/' + name: sha for name, sha in PROTOCOL_PINS.items()}}
    for name, sha in pins.items():
        product.field('source_inputs').field(name).equals(sha)
        require(file_hash(root / name) == sha, name, 'unreviewed source-owned no-world oracle')
    anchors = original_anchors(stage)
    anchors.subtree(log)
    names = {('stall-self-test' if name == 'mux2_stall_test' else name) + '.log' for name in SUITE_NAMES}
    require({p.name for p in log.iterdir()} == names, '/oracles', 'missing or extra original oracle logs')
    rows: list[Json] = []
    for name in SUITE_NAMES:
        filename = ('stall-self-test' if name == 'mux2_stall_test' else name) + '.log'
        path = log / filename
        _ = anchors.read(path)
        result = audit_log(name, path)
        if result:
            require(stdout.splitlines().count(result.strip()) == 1, '/oracle', 'original checker completion absent or repeated')
        rows.append({'name': name, 'log_sha256': file_hash(path), 'classification': 'source_verified_no_world'})
    # Source-fixed oracle scripts do not boot main.tscn. Any emitted native world
    # contradicts that classification and requires a new explicit source review.
    for key in ('WORLDSIM_GATE_TEST_OUT', 'WORLDSIM_GATE_UI_OUT'):
        folder = Path(stage.field('environment').field(key).text())
        anchors.subtree(folder)
        require(not tuple(folder.iterdir()), '/worlds', 'unexpected native world in source-classified oracle suite')
    anchors.subtree(log)
    _ = aggregate_output(session, stage)
    return {'status': 'PASS', 'coverage': rows, 'qualification': LIMIT, 'aggregate_pid': pid,
            'classification': 'no_normal_colony_world', 'gpu_acceptance': 'NOT_CLAIMED'}


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    return {'status': 'PASS', 'base': collect(session, base), 'candidate': collect(session, candidate)}
