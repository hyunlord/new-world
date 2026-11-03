"""Authenticate actual native process receipts and separately parsed endpoints."""
import re
from dataclasses import dataclass
from datetime import datetime

from generic_gate import GateContractError, boundary, document, integer, require, text
from generic_process import ProcessSpec, check_digest, environment_for
from mux2_gate_inputs import Json, Node
from generic_test_endpoint_types import (
    PendingReason as PendingReason, ExpectedOccurrence as ExpectedOccurrence,
    Loss as Loss, CommandRecord as CommandRecord, Snapshot as Snapshot, Observation as Observation,
    fields as fields, boolean as boolean, bound_bytes as bound_bytes, snapshot as snapshot,
)

@dataclass(frozen=True, slots=True)
class Binding:
    process: ProcessSpec
    receipt_sha256: str
    pid: int
    output_hashes: tuple[tuple[str, str], ...]


def process_receipt(binding: Binding) -> Node:
    spec = binding.process
    receipt = document(bound_bytes(spec.output_root / 'process.json', binding.receipt_sha256).decode())
    fields(receipt, 'schema_version argv cwd executable expected_executable_sha256 input_root expected_inputs fresh_directories expected_external_inputs before_external_inputs after_external_inputs output_root environment environment_policy started_at pid native_exit status errors build_provenance_verified before_executable_sha256 before_inputs ended_at duration_ns after_executable_sha256 after_inputs stdout_sha256 stderr_sha256 limit')
    receipt.field('schema_version').equals(1)
    receipt.field('fresh_directories').equals(list(spec.fresh_directories))
    identities: tuple[tuple[str, Json], ...] = (('argv', [value for value in spec.argv]), ('cwd', str(spec.cwd)), ('executable', str(spec.executable)),
                          ('input_root', str(spec.input_root)), ('output_root', str(spec.output_root)),
                          ('environment', {key: value for key, value in environment_for(spec).items()}), ('pid', binding.pid))
    for key, expected in identities:
        receipt.field(key).equals(expected)
    require(type(binding.pid) is int and binding.pid > 0, '/pid', 'positive process PID required')
    receipt.field('environment_policy').equals('fixed_defaults_plus_explicit_only_no_inheritance')
    receipt.field('build_provenance_verified').equals(False)
    for key in ['expected_executable_sha256', 'before_executable_sha256', 'after_executable_sha256']:
        receipt.field(key).equals(spec.executable_sha256)
    check_digest(spec.executable_sha256, '/executable')
    for key in ['expected_inputs', 'before_inputs', 'after_inputs']:
        receipt.field(key).equals(dict(spec.inputs))
    for key in ['expected_external_inputs', 'before_external_inputs', 'after_external_inputs']:
        receipt.field(key).equals({str(path): digest for path, digest in spec.external_inputs})
    for stream in ['stdout', 'stderr']:
        _ = bound_bytes(spec.output_root / f'{stream}.log', text(receipt.field(f'{stream}_sha256')))
    try:
        start, end = (datetime.fromisoformat(text(receipt.field(key))) for key in ['started_at', 'ended_at'])
        require(start.tzinfo is not None and end.tzinfo is not None and end >= start, '/process', 'invalid time interval')
    except ValueError as error:
        raise GateContractError('INVALID_TIME', '/process', str(error)) from error
    _ = integer(receipt.field('duration_ns'))
    code = receipt.field('native_exit').value
    require(type(code) is int, '/native_exit', 'completed native process required')
    errors = tuple(text(row) for row in receipt.field('errors').array())
    receipt.field('status').equals('PASS' if code == 0 and not errors else 'FAIL')
    _ = text(receipt.field('limit'))
    return receipt


@boundary
def authenticate(binding: Binding, expected: ExpectedOccurrence) -> Observation:
    receipt = process_receipt(binding)
    root = binding.process.output_root
    environment = environment_for(binding.process)
    require(environment.get('WORLDSIM_GATE_OBSERVER') == '1' and environment.get('WORLDSIM_GATE_TEST_OUT') == str(root), '/environment', 'observer must target this fresh process output root')
    from generic_test_endpoint import EndpointEvidence, authenticate_endpoint
    endpoint = authenticate_endpoint(EndpointEvidence(root, binding.pid, binding.output_hashes), expected)
    pending = list(endpoint.pending)
    code = receipt.field('native_exit').value
    require(type(code) is int, '/native_exit', 'completed child required')
    assert isinstance(code, int), 'integer parser invariant'
    if code != 0 or receipt.field('status').value != 'PASS':
        pending.append(PendingReason.NATIVE_PROCESS_FAILED)
    stdout = bound_bytes(root / 'stdout.log', text(receipt.field('stdout_sha256'))).decode()
    outcomes = re.findall(r'^test ' + re.escape(expected.native_test) + r' \.\.\. (ok|FAILED|ignored)\s*$', stdout, re.MULTILINE)
    require(len(outcomes) <= 1, expected.test, 'duplicate native test result')
    outcome = outcomes[0] if outcomes else None
    if outcome != 'ok':
        pending.append(PendingReason.NATIVE_TEST_OUTCOME_MISSING if outcome is None else PendingReason.NATIVE_TEST_NOT_OK)
    return Observation(endpoint.initial, endpoint.terminal, endpoint.founding, endpoint.history_complete,
                       code, outcome, tuple(pending), endpoint.output_manifest_sha256, endpoint.stop_reason)
