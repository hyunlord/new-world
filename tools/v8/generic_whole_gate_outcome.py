# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_outcome
"""Join original native exits with the exact collector-qualified stall exception."""
from pathlib import Path

from generic_gate import document, require
from mux2_gate_inputs import Node

from generic_process import file_hash
from generic_whole_gate_journal import ORDER, REFERENCE


def native_clean(process: Node, allowed: int) -> bool:
    """No postflight/input errors may hide behind an allowed native exit."""
    expected = [] if allowed == 0 else [f'native exit {allowed}']
    return (process.field('native_exit').value == allowed and process.field('errors').value == expected
            and process.field('status').value == ('PASS' if allowed == 0 else 'FAIL'))


def arm_clean(session: Node, arm: str, rows: tuple[Node, ...], process: Node,
              evaluation: Node) -> bool:
    """Validate native failures and original postchecks without changing any receipt."""
    expected = REFERENCE if arm == 'base' else ORDER
    if tuple(row.field('step').value for row in rows) != expected:
        return False
    waived: set[tuple[str, int]] = set()
    if evaluation.field('status').text() == 'PASS':
        for row in evaluation.field('waived_stage_exits').array():
            require(set(row.mapping()) == {'arm', 'step', 'native_exit'}, '/waiver', 'foreign waiver fields')
            row.field('arm').equals('candidate')
            row.field('step').equals(41)
            row.field('native_exit').equals(1)
            key = (row.field('arm').text(), 41)
            require(key not in waived, '/waiver', 'duplicate waiver')
            waived.add(key)
        for row in evaluation.field('reference_stage_exits').array():
            require(set(row.mapping()) == {'arm', 'step', 'native_exit'}, '/reference', 'foreign reference outcome')
            row.field('arm').equals('base')
            row.field('step').equals(41)
            row.field('native_exit').equals(1)
            require(('base', 41) not in waived, '/reference', 'duplicate reference outcome')
            waived.add(('base', 41))
    failures = 0
    for row in rows:
        if row.field('state').text() != 'RETURNED':
            return False
        allowed = 1 if (arm, int(row.field('step').number())) in waived else 0
        receipt = Path(row.field('process_receipt').text())
        require(file_hash(receipt) == row.field('process_receipt_sha256').text(), '/process', 'original process changed')
        if row.field('native_exit').value != allowed or not native_clean(document(receipt.read_text()), allowed):
            return False
        failures |= allowed
    summary = Path(session.field('output').text())/arm/'shell-summary.json'
    if not summary.exists():
        return False
    actual = document(summary.read_text())
    actual.field('arm').equals(arm)
    if actual.field('postcheck_status').value != 0 or actual.field('stage_status').value != failures:
        return False
    return native_clean(process, failures)
