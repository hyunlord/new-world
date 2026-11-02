"""Authenticate native test endpoints, never promote a loaded prefix into complete history.

Trusted caller supplies the executed ProcessSpec, pinned receipt/PID, independently
frozen output hashes and concrete planned horizons. This is not build or gate authority.
"""
import hashlib
import json
import re
from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum
from pathlib import Path

from generic_gate import GateContractError, boundary, document, integer, require, text
from generic_process import ProcessSpec, check_digest, environment_for, real_path
from generic_sources import manifest_hash, safe_path
from mux2_gate_inputs import Json, Node


class PendingReason(StrEnum):
    PREFIX_GRAPH_UNRESOLVED = 'PREFIX_GRAPH_UNRESOLVED'
    EXTERNAL_ORIGIN_UNRESOLVED = 'EXTERNAL_ORIGIN_UNRESOLVED'
    NONZERO_INITIAL_WITHOUT_PREFIX = 'NONZERO_INITIAL_WITHOUT_PREFIX'
    NATIVE_PROCESS_FAILED = 'NATIVE_PROCESS_FAILED'
    NATIVE_TEST_OUTCOME_MISSING = 'NATIVE_TEST_OUTCOME_MISSING'
    NATIVE_TEST_NOT_OK = 'NATIVE_TEST_NOT_OK'


@dataclass(frozen=True, slots=True)
class Binding:
    process: ProcessSpec
    receipt_sha256: str
    pid: int
    output_hashes: tuple[tuple[str, str], ...]


@dataclass(frozen=True, slots=True)
class ExpectedOccurrence:
    test: str
    native_test: str
    seed: int
    branch: str
    start_tick: int
    end_tick: int
    ordinary: bool


@dataclass(frozen=True, slots=True)
class Loss:
    tick: int
    node: str
    holder: int | None


@dataclass(frozen=True, slots=True)
class CommandRecord:
    tick: int
    command_json: str
    accepted: bool
    count: int
    result_debug: str


@dataclass(frozen=True, slots=True)
class Snapshot:
    tick: int
    alive_ids: tuple[int, ...]
    raw_agent_ids: tuple[int, ...]
    knowledge_enabled: bool
    losses: tuple[Loss, ...] | None
    history_order_valid: bool
    chronicle_entries: int
    commands: tuple[CommandRecord, ...]
    births: int
    deaths: tuple[tuple[str, int], ...]


@dataclass(frozen=True, slots=True)
class Observation:
    initial: Snapshot
    terminal: Snapshot
    founding: Snapshot | None
    history_complete: bool
    native_exit: int
    native_test_outcome: str | None
    pending: tuple[PendingReason, ...]
    output_manifest_sha256: str
    stop_reason: str


def fields(node: Node, names: str) -> None:
    require(set(node.mapping()) == set(names.split()), node.label, 'missing or unregistered fields')


def boolean(node: Node) -> bool:
    value = node.value
    require(type(value) is bool, node.label, 'expected boolean')
    return value is True


def bound_bytes(path: Path, expected: str) -> bytes:
    check_digest(expected, str(path))
    real_path(path, directory=False)
    data = path.read_bytes()
    require(hashlib.sha256(data).hexdigest() == expected, str(path), 'artifact hash mismatch')
    real_path(path, directory=False)
    return data


def snapshot(node: Node) -> Snapshot:
    fields(node, 'tick alive alive_ids raw_agent_population raw_agent_ids census_semantics knowledge_capability knowledge_lost history_order_valid chronicle_entries command_log births deaths')
    tick = integer(node.field('tick'))
    populations: list[tuple[int, ...]] = []
    for count, ids in [('alive', 'alive_ids'), ('raw_agent_population', 'raw_agent_ids')]:
        values = tuple(integer(row) for row in node.field(ids).array())
        require(values == tuple(sorted(set(values))) and len(values) == integer(node.field(count)), node.label, 'invalid census')
        populations.append(values)
    require(set(populations[0]) <= set(populations[1]), node.label, 'live census outside raw Agent census')
    node.field('census_semantics').equals('alive excludes BodyHealth::is_dead; raw includes every Agent; wrapper must check ordinary endpoint agreement')
    capability = node.field('knowledge_capability')
    fields(capability, 'enabled source')
    capability.field('source').equals('resources.rules.colony.knowledge')
    enabled = boolean(capability.field('enabled'))
    losses: list[Loss] = []
    if enabled:
        for row in node.field('knowledge_lost').array():
            fields(row, 'tick node holder')
            holder = row.field('holder')
            loss = Loss(integer(row.field('tick')), text(row.field('node')), None if holder.value is None else integer(holder))
            require(loss.tick <= tick and (not losses or loss.tick >= losses[-1].tick), row.label, 'unordered or future loss')
            losses.append(loss)
    else:
        node.field('knowledge_lost').equals(None)
    commands: list[CommandRecord] = []
    for row in node.field('command_log').array():
        fields(row, 'tick command accepted count result_debug')
        command = row.field('command')
        _ = command.mapping()
        at = integer(row.field('tick'))
        require(at <= tick, row.label, 'future command')
        commands.append(CommandRecord(at, json.dumps(command.value, sort_keys=True), boolean(row.field('accepted')), integer(row.field('count')), text(row.field('result_debug'))))
    deaths = node.field('deaths')
    fields(deaths, 'starvation dehydration exposure combat old_age')
    count = integer(node.field('chronicle_entries'))
    require(count >= len(losses), node.label, 'loss count exceeds Chronicle count')
    return Snapshot(tick, populations[0], populations[1], enabled, tuple(losses) if enabled else None,
                    boolean(node.field('history_order_valid')), count, tuple(commands), integer(node.field('births')),
                    tuple((key, integer(deaths.field(key))) for key in sorted(deaths.mapping())))


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
    real_path(root, directory=True)
    environment = environment_for(binding.process)
    require(environment.get('WORLDSIM_GATE_OBSERVER') == '1' and environment.get('WORLDSIM_GATE_TEST_OUT') == str(root), '/environment', 'observer must target this fresh process output root')
    require(all(re.fullmatch(r'[A-Za-z0-9_]+', value) for value in [expected.test, expected.branch]), '/expected', 'invalid occurrence identity')
    for value in [expected.seed, expected.start_tick, expected.end_tick]:
        _ = integer(Node(value))
    require(expected.end_tick >= expected.start_tick and expected.native_test.split('::')[-1] == expected.test and type(expected.ordinary) is bool, '/expected', 'invalid concrete horizon, classification or native test name')
    hashes = dict(binding.output_hashes)
    require(len(hashes) == len(binding.output_hashes) and bool(hashes), '/outputs', 'duplicate or empty output bindings')
    for name, digest in hashes.items():
        _ = bound_bytes(safe_path(root, name), digest)

    def read(name: str) -> Node:
        require(name in hashes, name, 'artifact absent from frozen output manifest')
        return document(bound_bytes(safe_path(root, name), hashes[name]).decode())

    folder = f'save_load_pid{binding.pid}/{expected.test}_s{expected.seed}_{expected.branch}'
    initial = read(folder + '/initial.json')
    fields(initial, 'schema_version test seed branch pid initial history_complete status')
    for key, value in [('schema_version', 1), ('test', expected.test), ('seed', expected.seed), ('branch', expected.branch), ('pid', binding.pid), ('status', 'PENDING_NATIVE_TEST_RESULT')]:
        initial.field(key).equals(value)
    start = snapshot(initial.field('initial'))
    terminal = read(folder + '/terminal.json')
    fields(terminal, 'schema_version initial founding terminal completed_ticks stop_reason prefix history_complete loaded_prefix_requires_external_authentication status')
    terminal.field('schema_version').equals(1)
    terminal.field('status').equals('ENDPOINT_REACHED_NATIVE_TEST_RESULT_REQUIRED')
    terminal.field('initial').equals(initial.field('initial').value)
    end = snapshot(terminal.field('terminal'))
    require((start.tick, end.tick) == (expected.start_tick, expected.end_tick), folder, 'incomplete or different planned horizon')
    require(integer(terminal.field('completed_ticks')) == end.tick - start.tick, folder, 'incorrect completed ticks')
    require(start.knowledge_enabled == end.knowledge_enabled, folder, 'knowledge capability changed')
    if start.losses is not None and end.losses is not None:
        require(end.losses[:len(start.losses)] == start.losses, folder, 'inherited losses removed or changed')
    if expected.ordinary:
        require(start.alive_ids == start.raw_agent_ids and end.alive_ids == end.raw_agent_ids, folder, 'ordinary live/raw endpoint disagreement')
    founding_node = terminal.field('founding')
    founding = None if founding_node.value is None else snapshot(founding_node)
    prefix = terminal.field('prefix')
    pending: list[PendingReason] = []
    initial.field('history_complete').equals(start.tick == 0)
    complete = boolean(terminal.field('history_complete'))
    require(complete == (prefix.value is None and start.tick == 0 and end.history_order_valid), folder, 'false completeness claim')
    terminal.field('loaded_prefix_requires_external_authentication').equals(prefix.value is not None or start.tick != 0)
    if prefix.value is not None:
        external = 'kind' in prefix.mapping()
        if external:
            fields(prefix, 'kind save_artifact checkpoint founding_census external_hash_authentication_required')
            prefix.field('kind').equals('external_legacy_save')
            prefix.field('founding_census').equals(None)
            require(founding is None, folder, 'external origin cannot invent founders')
            read(folder + '/external-prefix.json').equals(prefix.value)
        else:
            fields(prefix, 'save_artifact checkpoint native_prefix_equal external_hash_authentication_required')
            prefix.field('native_prefix_equal').equals(True)
            link = read(folder + '/load-link.json')
            fields(link, 'prefix founding initial')
            for key in ['prefix', 'founding', 'initial']:
                link.field(key).equals(terminal.field(key).value)
        prefix.field('external_hash_authentication_required').equals(True)
        names: list[str] = []
        for key in ['save_artifact', 'checkpoint']:
            path = Path(text(prefix.field(key)))
            require(path.is_absolute() and path.is_relative_to(root) and '..' not in path.parts, folder, 'prefix escapes invocation')
            names.append(path.relative_to(root).as_posix())
        if all(name in hashes for name in names):
            saved = bound_bytes(safe_path(root, names[0]), hashes[names[0]])
            checkpoint = read(names[1])
            fields(checkpoint, 'state save_artifact byte_count magic input_version_le_bytes current_format_version external_sha256_required')
            checkpoint.field('state').equals(initial.field('initial').value)
            checkpoint.field('save_artifact').equals(prefix.field('save_artifact').value)
            require(integer(checkpoint.field('byte_count')) == len(saved) and saved[:8] == b'WORLDSIM' and len(saved) >= 12, folder, 'invalid saved bytes')
            checkpoint.field('magic').equals(list(saved[:8]))
            checkpoint.field('input_version_le_bytes').equals(list(saved[8:12]))
            _ = integer(checkpoint.field('current_format_version'))
            checkpoint.field('external_sha256_required').equals(True)
        pending.append(PendingReason.EXTERNAL_ORIGIN_UNRESOLVED if external else PendingReason.PREFIX_GRAPH_UNRESOLVED)
    else:
        require(founding == start, folder, 'unlinked founding census differs from initial')
        if start.tick != 0:
            pending.append(PendingReason.NONZERO_INITIAL_WITHOUT_PREFIX)
    code = receipt.field('native_exit').value
    require(type(code) is int, '/native_exit', 'completed child required')
    assert isinstance(code, int), 'integer parser invariant'
    if code != 0 or receipt.field('status').value != 'PASS':
        pending.append(PendingReason.NATIVE_PROCESS_FAILED)
    stdout = bound_bytes(root / 'stdout.log', text(receipt.field('stdout_sha256'))).decode()
    outcomes = re.findall(r'^test ' + re.escape(expected.native_test) + r' \.\.\. (ok|FAILED|ignored)\s*$', stdout, re.MULTILINE)
    require(len(outcomes) <= 1, folder, 'duplicate native test result')
    outcome = outcomes[0] if outcomes else None
    if outcome != 'ok':
        pending.append(PendingReason.NATIVE_TEST_OUTCOME_MISSING if outcome is None else PendingReason.NATIVE_TEST_NOT_OK)
    return Observation(start, end, founding, complete, code, outcome, tuple(pending), manifest_hash(dict(binding.output_hashes)), text(terminal.field('stop_reason')))
