# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Verify actual release-stage evidence through a read-only product protocol."""
from pathlib import Path
from typing import Protocol

from generic_gate import document, require
from generic_process import file_hash
from mux2_gate_inputs import Node

PERF_COMMAND = '(cd rust && cargo build --release -p sim-test) && bash tools/v8/perf.sh rust/target/release "$0/perf" base > /dev/null && echo "Fresh arm observations; paired performance acceptance is step25/45"'


class StageProduct(Protocol):
    @property
    def root(self) -> Path: ...
    @property
    def binary(self) -> Path: ...
    @property
    def source_sha(self) -> str: ...
    @property
    def binary_sha256(self) -> str: ...
    @property
    def build_receipt(self) -> Path: ...
    @property
    def input_files(self) -> tuple[tuple[str, str], ...]: ...


def verify_stage(product: StageProduct, stage: Node) -> None:
    """Recheck original successful step12 and its genuine process and endpoint joins.

    Materialization authority remains A's current_product_inputs caller. These
    checks do not turn an arbitrary JSON file into source/build authority.
    """
    stage.field('schema').equals('whole_gate_step.v1')
    stage.field('step').equals(12)
    stage.field('name').equals('perf')
    require(stage.field('arm').text() in {'base', 'candidate'}, '/arm', 'foreign build arm')
    stage.field('state').equals('RETURNED')
    stage.field('native_exit').equals(0)
    stage.field('process_status').equals('PASS')
    stage.field('cwd').equals(str(product.root))
    stage.field('environment').field('WORLDSIM_GIT_SHA').equals(product.source_sha)
    argv = tuple(row.text() for row in stage.field('argv').array())
    require(len(argv) == 4 and argv[:3] == ('bash', '-c', PERF_COMMAND), '/argv', 'not original release perf build')
    selected = stage.field('selected_artifacts').field('perf')
    expected = product.root/'rust/target/release/worldsim_perf'
    require(product.binary == expected, '/binary', 'not source-selected release endpoint')
    selected.field('path').equals(str(expected))
    selected.field('sha256').equals(product.binary_sha256)
    receipt = Path(stage.field('process_receipt').text())
    require(receipt == product.build_receipt.parent/'process/process.json', '/process', 'foreign original stage process')
    require(file_hash(receipt) == stage.field('process_receipt_sha256').text(), '/process', 'original build process changed')
    process = document(receipt.read_text())
    for key in ('pid', 'cwd', 'environment', 'native_exit'):
        process.field(key).equals(stage.field(key).value)
    process.field('status').equals('PASS')
    process.field('errors').equals([])
    require(process.field('pid').number() > 0, '/pid', 'missing actual build process')
    process.field('input_root').equals(str(product.root))
    executable = process.field('executable').text()
    process.field('argv').equals([executable, '-c', 'exec "$@" 2>&1', 'whole-step', *argv])
    for key in ('expected_executable_sha256', 'before_executable_sha256', 'after_executable_sha256'):
        process.field(key).equals(file_hash(Path(executable)))
    for name, digest in product.input_files:
        process.field('expected_inputs').field(name).equals(digest)
        process.field('before_inputs').field(name).equals(digest)
        process.field('after_inputs').field(name).equals(digest)
    for channel in ('stdout', 'stderr'):
        process.field(channel+'_sha256').equals(file_hash(receipt.parent/(channel+'.log')))
