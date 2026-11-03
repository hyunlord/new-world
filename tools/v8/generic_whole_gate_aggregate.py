"""Actual whole-stage process receipt boundary without inferred child receipts."""
import re
import shlex
from pathlib import Path
from generic_gate import boundary, document, integer, require, text
from generic_process import file_hash
from mux2_gate_inputs import Node

LIMIT = 'SOURCE_BOUND_AGGREGATE_ONLY_NO_CHILD_WAIT_OR_BINARY_PROOF'

@boundary
def aggregate_output(session: Node, stage: Node) -> tuple[str, int]:
    """Authenticate only the actual Bash process; never infer child settlement."""
    stage.field('state').equals('RETURNED')
    stage.field('native_exit').equals(0)
    arm = text(stage.field('arm'))
    require(arm in {'base', 'candidate'}, '/arm', 'unknown product arm')
    product = session.field('arms').field(arm)
    root = Path(text(product.field('root')))
    stage.field('cwd').equals(str(root))
    harness = session.field('harness')
    gate = Path(text(harness.field('root'))) / 'tools/v8/gate.sh'
    require(file_hash(gate) == text(harness.field('inputs').field('tools/v8/gate.sh')),
            '/gate', 'source-owned shell changed')
    number = integer(stage.field('step'))
    lines = [shlex.split(line) for line in gate.read_text().splitlines()
             if re.match(r'^step ' + str(number) + r' ', line)]
    require(len(lines) == 1, '/gate', 'ambiguous original native command')
    command = [str(Path(text(session.field('output'))) / arm) + word[4:] if word == '$LOG' or word.startswith('$LOG/') else word
               for word in lines[0][3:]]
    stage.field('argv').equals([word for word in command])
    receipt_path = Path(text(stage.field('process_receipt')))
    require(receipt_path.is_relative_to(Path(text(session.field('output')))), '/process', 'foreign aggregate output')
    require(file_hash(receipt_path) == text(stage.field('process_receipt_sha256')), '/process', 'aggregate receipt changed')
    receipt = document(receipt_path.read_text())
    bash = session.field('bash')
    receipt.field('argv').equals([text(bash.field('path')), '-c', 'exec "$@" 2>&1', 'whole-step', *command])
    receipt.field('executable').equals(bash.field('path').value)
    for key in ('expected_executable_sha256', 'before_executable_sha256', 'after_executable_sha256'):
        receipt.field(key).equals(bash.field('sha256').value)
    for key in ('pid', 'native_exit', 'cwd', 'environment'):
        receipt.field(key).equals(stage.field(key).value)
    receipt.field('status').equals('PASS')
    receipt.field('errors').equals([])
    aggregate_pid = integer(receipt.field('pid'))
    require(aggregate_pid > 0, '/pid', 'positive aggregate PID required')
    for channel in ('stdout', 'stderr'):
        require(file_hash(receipt_path.parent / f'{channel}.log') == text(receipt.field(f'{channel}_sha256')),
                channel, 'aggregate raw stream changed')
    return (receipt_path.parent / 'stdout.log').read_text(), aggregate_pid
