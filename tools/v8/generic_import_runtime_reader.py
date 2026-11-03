"""Authenticate retained original import closure; no renderer runtime admission."""
from pathlib import Path

import gate_oracle_child as controller
import generic_setup_import as producer
from generic_gate import document, require
from generic_import_checkpoint_reader import controller_checkpoint
from generic_process import file_hash, real_path
from generic_setup_import import PriorBridge, authenticate_bridge
from generic_setup_prepare import PreparedSetup, absent
from generic_sources import inventory
from mux2_gate_inputs import Json, Node


def read_import_closure(prepared: PreparedSetup, prior: PriorBridge,
                        receipt_path: Path, receipt_sha256: str) -> Node:
    """Return the original last checkpoint only after all retained joins and rechecks."""
    real_path(receipt_path, directory=False)
    require(receipt_path.name == 'receipt.json' and file_hash(receipt_path) == receipt_sha256,
            '/receipt', 'original import receipt changed')
    output = receipt_path.parent
    absent(output/'failure.json')
    receipt = document(receipt_path.read_text())
    context = prepared.inputs
    for module in (controller, producer):
        assert module.__file__ is not None
        loaded = Path(module.__file__).resolve(strict=True)
        original = context.root/'tools/v8'/loaded.name
        require(dict(prepared.materialized).get('tools/v8/'+loaded.name) == file_hash(loaded)
                and file_hash(original) == file_hash(loaded), '/source', 'unreviewed import producer source')
    bound = dict(authenticate_bridge(prepared, prior))
    expected: dict[str, Json] = {'schema': 'setup_import_pair.v1', 'status': 'PASS_STEP3_ONLY', 'step': 3,
            'first_native_exit': 0, 'second_native_exit': 0, 'timed_out': [False, False],
            'shell_exit': 0, 'ownership': 'SETTLED_ORIGINAL_IMMEDIATE_CHILDREN',
            'prior_bridge_sha256': prior.receipt_sha256, 'source_sha': context.source_sha,
            'bridge_sha256': prior.result.bridge_sha256,
            'runtime_closure': 'CALLER_OWNED_UNQUALIFIED', 'build_product_authority': False,
            'steps4_5_6': 'NOT_RUN', 'full_gate_complete': False}
    for key, value in expected.items():
        receipt.field(key).equals(value)
    artifacts = inventory(output, ['children', 'logs', 'plan.json'])
    receipt.field('artifacts').equals({key: value for key, value in artifacts.items()})
    plan = controller.load_suite(output/'plan.json', receipt.field('plan_sha256').text())
    require(plan.import_pair and plan.root == context.root and plan.evidence == output/'children'
            and plan.logs == output/'logs', '/plan', 'foreign original import plan')
    require(dict(plan.environment) == dict(context.environment), '/environment', 'original import environment differs')
    godot = next(tool for tool in context.tools if tool.name == 'godot')
    require(all(row.argv[0] == str(godot.path) for row in plan.rows), '/godot', 'foreign import executable')
    bound.update({context.root/name: digest for name, digest in prepared.materialized})
    bound.update({tool.path: tool.sha256 for tool in context.tools})
    require(dict(plan.inputs) == bound, '/inputs', 'import plan differs from complete prior source/tool/build closure')
    checkpoint = controller_checkpoint(plan, receipt)
    # Rechecking current bytes against the original checkpoint never establishes a new baseline.
    checkpoint.equals(controller.import_state(plan))
    controller.suite_current(plan)
    require(file_hash(receipt_path) == receipt_sha256, '/receipt', 'receipt changed during verification')
    require(inventory(output, ['children', 'logs', 'plan.json']) == artifacts,
            '/artifacts', 'original import evidence changed during verification')
    return checkpoint
