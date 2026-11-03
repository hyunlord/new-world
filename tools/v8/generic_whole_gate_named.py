"""Original13/32 source-qualified no-world mechanisms; no invented native census."""
from pathlib import Path
from typing import Final

from generic_gate import boundary, document, integer, require, text
from generic_process import file_hash
from generic_resolver_run import COMMAND, PINS, checker_counts
from generic_static_run import COMMANDS, DOCS_CHECKERS
from generic_test_endpoint_types import bound_bytes
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from mux2_gate_inputs import Json, Node

RESOLVER_INPUTS: Final = (
    'tools/v8/resolver_parity_fixture.json', 'tools/v8/flora_parity_fixture.json',
    'tools/v8/resolver_parity_expected.json', 'localization/manifest.json',
    'assets/v8art/manifest.json', 'assets/v8art/bindings.json', 'assets/v8art/render_params.json',
    'assets/v8art/materials/construction_styles.schema.json', 'assets/v8art/materials/construction_styles.json',
    'assets/v8art/materials/material_looks.json', 'assets/v8art/materials/category_defaults.json',
    'assets/v8art/flora/species.json', 'assets/v8art/flora/habit_defaults.json',
)
FORBIDDEN: Final = ('WORLDSIM_ORACLE_CHILD_PLAN', 'WORLDSIM_ORACLE_CHILD_PLAN_SHA256',
    'WORLDSIM_ORACLE_SUITE_PLAN', 'WORLDSIM_ORACLE_SUITE_PLAN_SHA256',
    'WORLDSIM_ORACLE_SUITE_REQUEST_FD', 'WORLDSIM_ORACLE_SUITE_CONTROL_FD',
    'WORLDSIM_STALL_CHILD_PLAN', 'WORLDSIM_STALL_CHILD_PLAN_SHA256')


def sources(session: Node, stage: Node, step: int) -> tuple[Path, dict[str, str]]:
    product = session.field('arms').field(stage.field('arm').text())
    root = Path(product.field('root').text())
    inputs = product.field('source_inputs')
    pins = {'tools/v8/bridge_api.py': DOCS_CHECKERS['bridge_api.py']} if step == 13 else {key: value for key, value in PINS.items() if key != 'tools/v8/gate.sh'}
    names: set[str] = {'docs/v8/bridge_api_read.txt'} if step == 13 else set(RESOLVER_INPUTS)
    if step == 13:
        files = {path.relative_to(root).as_posix() for path in (root/'rust/crates/sim-bridge/src').rglob('*.rs')}
        require(files == {key for key in inputs.mapping() if key.startswith('rust/crates/sim-bridge/src/') and key.endswith('.rs')}, '/source', 'bridge source namespace changed')
        names.update(files)
    else:
        for case in document((root/'tools/v8/flora_parity_fixture.json').read_text()).field('cases').array():
            key = case.mapping().get('fixture')
            if key is not None:
                name = text(Node(key))
                require(bool(name) and all(char.isalnum() or char in '_-' for char in name), '/fixture', 'unsafe species fixture')
                names.add('assets/v8art/flora/tests/'+name+'.json')
        require(not set(FORBIDDEN) & stage.field('environment').mapping().keys(), '/environment', 'different bounded-child protocol')
        for relative in ('override.cfg', 'addons/godot_test_mcp'):
            require(not (root/relative).exists() and not (root/relative).is_symlink(), '/source', 'unreviewed local override')
    retained: dict[str, str] = {}
    for name in sorted(names | pins.keys()):
        sha = inputs.field(name).text()
        require(name not in pins or pins[name] == sha, name, 'unreviewed no-world source')
        _ = bound_bytes(root/name, sha)
        retained[name] = sha
    return root, retained


def bridge_terminal(root: Path, raw: str) -> dict[str, Json]:
    expected = {line.strip() for line in (root/'docs/v8/bridge_api_read.txt').read_text().splitlines()
                if line.strip() and not line.startswith('#')}
    require(bool(expected), '/bridge', 'empty original API expectation')
    count = len(expected)
    line = f'bridge api: {count} items; expected {count} ({count} at base minus 0 removed)'
    require(raw.splitlines() == [line], '/bridge', 'original API oracle differs')
    return {'items': count, 'oracle': 'ORIGINAL_PYTHON_CHECK_PASS'}


@boundary
def collect(session: Node, stage: Node) -> dict[str, Json]:
    step = integer(stage.field('step'))
    require(step in (13, 32), '/step', 'unknown named mechanism')
    log = Path(session.field('output').text())/stage.field('arm').text()
    stage.field('argv').equals(list(COMMANDS[13][1]) if step == 13 else ['bash', '-c', COMMAND, str(log)])
    raw, pid = aggregate_output(session, stage)
    root, retained = sources(session, stage, step)
    artifact: dict[str, Json] = {}
    if step == 13:
        result = bridge_terminal(root, raw)
    else:
        result = checker_counts(root, raw.encode())
        output = log/'resolver_godot.json'
        sha = stage.field('observer_artifacts').field(str(output)).text()
        stage.field('observer_modes').field(str(output)).equals(output.lstat().st_mode)
        payload = bound_bytes(output, sha)
        node = document(payload.decode())
        node.field('problems').equals([])
        node.field('flora_problems').equals([])
        require(len(node.field('results').array()) == result['material'] and
                len(node.field('flora_results').array()) == result['flora'], '/resolver', 'retained output count differs')
        marker = f"RESOLVER_PARITY_DONE cases={result['material']} problems=0 flora_cases={result['flora']} flora_problems=0"
        require(raw.splitlines().count(marker) == 1, '/resolver', 'missing actual Godot completion marker')
        require('SCRIPT ERROR' not in raw, '/resolver', 'Godot script error')
        artifact = {'path': str(output), 'sha256': sha, 'mode': output.lstat().st_mode}
        _ = bound_bytes(output, sha)
    for name, sha in retained.items():
        require(file_hash(root/name) == sha, name, 'source changed during original oracle join')
    _ = aggregate_output(session, stage)
    return {'status': 'PASS', 'step': step, 'classification': 'NO_REGISTERED_SIMULATION_WORLD_SOURCE_DERIVED',
            'aggregate_pid': pid, 'qualification': LIMIT, 'native_pid': None,
            'worlds': [], 'founding_census': None, 'original_oracle': result,
            'sources': {name: sha for name, sha in retained.items()}, 'artifact': artifact, 'publication_authority': False}


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> dict[str, Json]:
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    candidate.field('step').equals(base.field('step').value)
    return {'status': 'PASS', 'safety': [], 'base': collect(session, base), 'candidate': collect(session, candidate)}
