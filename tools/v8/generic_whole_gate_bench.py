# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -B -m unittest test_generic_whole_gate_acceptance
"""Reauthenticate the original interleaved benchmark; never run another workload."""
from pathlib import Path

from generic_bench_run import authenticate_result
from generic_gate import document, require, text
from generic_process import file_hash
from generic_whole_gate_contract import current_product_inputs, hashes
from generic_whole_gate_paired import closure, recheck
from mux2_gate_inputs import Json, Node

from generic_bench import Product


def _product(session: Node, arm: str) -> Product:
    selected = current_product_inputs(session, arm, 'perf')
    from generic_whole_gate_bench_product import from_view
    return from_view(session, arm, selected)


def evaluate(session: Node, stage: Node) -> dict[str, Json]:
    """Join A's actual build views and B's original closure to the standing audit."""
    stage.field('step').equals(25)
    stage.field('arm').equals('candidate')
    stage.field('state').equals('RETURNED')
    stage.field('native_exit').equals(0)
    acceptance = Path(text(stage.field('process_receipt'))).parent.parent / 'acceptance/acceptance.json'
    digest = text(stage.field('observer_artifacts').field(str(acceptance)))
    require(file_hash(acceptance) == digest, '/bench', 'original acceptance changed')
    node = document(acceptance.read_text())
    node.field('schema').equals('whole_gate_acceptance.v1')
    node.field('step').equals(25)
    node.field('status').equals('COMPONENT_PASS')
    node.field('component_status').equals('COMPONENT_PASS')
    original = closure(node, acceptance)
    result = acceptance.parent / 'component/result.json'
    node.field('component_receipt').equals(str(result))
    node.field('component_receipt_sha256').equals(dict(original.files)[str(result)])
    config = session.field('acceptance')
    script = config.field('bench_script')
    evaluated = authenticate_result(_product(session, 'base'), _product(session, 'candidate'),
        text(session.field('purpose')), Path(text(script.field('path'))), text(script.field('sha256')),
        result.parent, receipt_sha256=dict(original.files)[str(result)],
        justifications=tuple(hashes(row) for row in config.field('justifications').array()))
    require(evaluated.status == 'COMPONENT_PASS' and not evaluated.errors,
            '/bench', 'original child journal/numeric/safety audit rejected: ' + '; '.join(evaluated.errors))
    recheck(original)
    require(file_hash(acceptance) == digest, '/bench', 'acceptance changed during audit')
    return {'status': 'PASS', 'scope': 'original_six_interleaved_aggregates_and_all_native_children',
            'receipt': str(result), 'sha256': dict(original.files)[str(result)],
            'publication_authority': False}
