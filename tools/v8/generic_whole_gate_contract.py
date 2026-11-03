# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate
"""Closed shared whole-shell session and returned evidence readers."""
from pathlib import Path

from generic_gate import document, require
from generic_process import file_hash
from generic_sources import safe_path
from generic_whole_gate_journal import ORDER
from mux2_gate_inputs import Node


def hashes(node: Node) -> dict[str, str]:
    return {name: Node(value).text() for name, value in node.mapping().items()}


def load_session(path: Path, expected_sha256: str) -> Node:
    """Authenticate immutable session and the exact retained input bytes, not runtime claims."""
    require(file_hash(path) == expected_sha256, '/session', 'session hash changed')
    session = document(path.read_text())
    session.field('schema').equals('whole_gate_session.v1')
    if 'materialization' in session.mapping():
        from generic_whole_gate_qualification import qualification
        session.field('qualification').equals(qualification(session))
        from generic_whole_gate_source import check_materialization
        check_materialization(session)
    else:
        session.field('qualification').equals('TEST_FIXTURE_NOT_PRODUCT_QUALIFIED')
        require('purpose' not in session.mapping(), '/session', 'fixture cannot request product authority')
    for name, digest in hashes(session.field('external_inputs')).items():
        require(file_hash(Path(name)) == digest, name, 'bound external input changed')
        if 'materialization' in session.mapping():
            require(Path(name).lstat().st_mode == session.field('external_modes').field(name).number(),
                    name, 'external input mode changed')
    for arm in session.field('arms').mapping().values():
        source = Node(arm)
        root = Path(source.field('root').text())
        for name, digest in hashes(source.field('source_inputs')).items():
            require(file_hash(safe_path(root, name)) == digest, name, 'effective source changed')
    harness = session.field('harness')
    for name, digest in hashes(harness.field('inputs')).items():
        require(file_hash(safe_path(Path(harness.field('root').text()), name)) == digest,
                name, 'harness source changed')
    if 'output' in session.mapping():
        from generic_whole_gate_journal import check_auxiliaries
        check_auxiliaries(session)
    return session


def stage_root(session: Node, arm: str, step: int, name: str) -> Path:
    require(arm in {'base', 'candidate'} and step in ORDER, '/step', 'unknown arm or step')
    require(bool(name) and all(ch.isalnum() or ch == '_' for ch in name), '/step/name', 'unsafe name')
    return Path(session.field('output').text()) / 'steps' / arm / f'{step:02d}-{name}'


def current_product_inputs(session: Node, arm: str, kind: str) -> Node:
    """Read an actual stage-issued binary view; never turn a path into build authority."""
    require(arm in {'base', 'candidate'} and kind in {'probe', 'perf', 'bridge'}, '/product', 'unknown product')
    path = Path(session.field('output').text()) / 'products' / arm / f'{kind}.json'
    product = document(path.read_text())
    product.field('source_sha').equals(session.field('arms').field(arm).field('source_sha').text())
    from generic_whole_gate_journal import entries
    prior = entries(session, arm)
    expected_step = 2 if kind == 'bridge' else (10 if arm == 'base' else 11) if kind == 'probe' else 12
    expected = [row for row in prior if row.field('step').value == expected_step]
    require(len(expected) == 1, '/product', 'missing actual source-owned build stage')
    receipt = Path(product.field('build_receipt').text())
    require(file_hash(receipt) == product.field('build_receipt_sha256').text(), '/product', 'build evidence changed')
    build = document(receipt.read_text())
    build.field('native_exit').equals(0)
    build.field('arm').equals(arm)
    require(build.value == expected[0].value, '/product', 'foreign build-stage receipt')
    selected = build.field('selected_artifacts').field(kind)
    selected.field('path').equals(product.field('executable').text())
    selected.field('sha256').equals(product.field('executable_sha256').text())
    executable = Path(product.field('executable').text())
    require(file_hash(executable) == product.field('executable_sha256').text(), '/product', 'selected binary changed')
    for name, digest in hashes(product.field('runtime_inputs')).items():
        require(file_hash(Path(name)) == digest, name, 'runtime input changed')
    require(hashes(product.field('source_inputs')) == hashes(session.field('arms').field(arm).field('source_inputs')),
            '/product', 'product source differs from admitted arm')
    return product


def prior_acceptance(session: Node, step: int) -> Node:
    """Return original returned44 evidence, checking its recorded inventory without refreshing it."""
    require(step == 44, '/prior', 'only original44 is a prior acceptance input')
    root = stage_root(session, 'candidate', 44, 'core_shadow')
    from generic_whole_gate_journal import entries
    prior = entries(session, 'candidate')
    original_rows = [row for row in prior if row.field('step').value == 44]
    require(len(original_rows) == 1, '/prior', 'missing authentic original44')
    terminal = document((root / 'stage.json').read_text())
    require(terminal.value == original_rows[0].value, '/prior', 'foreign stage44')
    terminal.field('state').equals('RETURNED')
    terminal.field('native_exit').equals(0)
    original = terminal.field('acceptance_anchor')
    for name, digest in hashes(original.field('inventory')).items():
        require(file_hash(Path(name)) == digest, name, 'prior acceptance changed')
    return original
