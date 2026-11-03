# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate
"""Actual successful build-stage endpoint views; not compiler artifact attestations."""
from pathlib import Path

from generic_gate import document, require
from mux2_gate_inputs import Json, Node

from generic_process import atomic_json, file_hash


def selected(session: Node, arm: str, step: int, native_exit: int | None) -> dict[str, Json]:
    """Select only binaries from the source-owned stages that build those products."""
    if native_exit != 0:
        return {}
    kinds = ('bridge',) if step == 2 else ('probe',) if arm == 'base' and step == 10 else (
        ('probe',) if arm == 'candidate' and step == 11 else ('perf',) if step == 12 else ())
    root = Path(session.field('arms').field(arm).field('root').text())
    result: dict[str, Json] = {}
    for kind in kinds:
        path = root/'rust/target/debug/libsim_bridge.dylib' if kind == 'bridge' else root/f'rust/target/release/worldsim_{kind}'
        if 'materialization' not in session.mapping() and not path.exists():
            continue
        result[kind] = {'path': str(path), 'sha256': file_hash(path)}
    return result


def publish_products(session: Node, arm: str, terminal: Path) -> None:
    """Retain the exact real stage and selected artifact hashes for acceptance consumers."""
    stage = document(terminal.read_text())
    product = session.field('arms').field(arm)
    for kind, raw in stage.field('selected_artifacts').mapping().items():
        selected_artifact = Node(raw)
        path = Path(selected_artifact.field('path').text())
        digest = selected_artifact.field('sha256').text()
        require(file_hash(path) == digest, '/product', 'built endpoint changed before publication')
        output = Path(session.field('output').text())/'products'/arm
        output.mkdir(parents=True, exist_ok=True)
        atomic_json(output/f'{kind}.json', {
            'source_sha': product.field('source_sha').value,
            'source_inputs': product.field('source_inputs').value,
            'runtime_inputs': {str(path): digest},
            'executable': str(path), 'executable_sha256': digest,
            'build_receipt': str(terminal), 'build_receipt_sha256': file_hash(terminal),
            'provenance': 'SOURCE_OWNED_BUILD_STAGE_ENDPOINT_NOT_COMPILER_JSON'})
