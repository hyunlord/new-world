# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_native
"""Native endpoint semantics independent of a process or aggregate lifecycle claim."""
import re
from dataclasses import dataclass
from pathlib import Path

from generic_gate import boundary, document, integer, require, text
from generic_process import real_path
from generic_sources import manifest_hash, safe_path
from mux2_gate_inputs import Node
from generic_test_endpoint_types import (ExpectedOccurrence, PendingReason, Snapshot,
    boolean, bound_bytes, fields, snapshot)

@dataclass(frozen=True, slots=True)
class EndpointEvidence:
    root: Path
    pid: int
    output_hashes: tuple[tuple[str, str], ...]

@dataclass(frozen=True, slots=True)
class Endpoint:
    initial: Snapshot
    terminal: Snapshot
    founding: Snapshot | None
    history_complete: bool
    pending: tuple[PendingReason, ...]
    output_manifest_sha256: str
    stop_reason: str

@boundary
def authenticate_endpoint(evidence: EndpointEvidence, expected: ExpectedOccurrence) -> Endpoint:
    """Read exact planned native endpoints; caller separately proves execution/source."""
    root = evidence.root
    real_path(root, directory=True)
    require(type(evidence.pid) is int and evidence.pid > 0, '/pid', 'positive emitted PID required')
    require(all(re.fullmatch(r'[A-Za-z0-9_]+', value) for value in [expected.test, expected.branch]), '/expected', 'invalid occurrence identity')
    for value in [expected.seed, expected.start_tick, expected.end_tick]:
        _ = integer(Node(value))
    require(expected.end_tick >= expected.start_tick and expected.native_test.split('::')[-1] == expected.test and type(expected.ordinary) is bool, '/expected', 'invalid concrete horizon, classification or native test name')
    hashes = dict(evidence.output_hashes)
    require(len(hashes) == len(evidence.output_hashes) and bool(hashes), '/outputs', 'duplicate or empty output bindings')
    for name, digest in hashes.items():
        _ = bound_bytes(safe_path(root, name), digest)

    def read(name: str) -> Node:
        require(name in hashes, name, 'artifact absent from frozen output manifest')
        return document(bound_bytes(safe_path(root, name), hashes[name]).decode())

    folder = f'save_load_pid{evidence.pid}/{expected.test}_s{expected.seed}_{expected.branch}'
    initial = read(folder + '/initial.json')
    fields(initial, 'schema_version test seed branch pid initial history_complete status')
    for key, value in [('schema_version', 1), ('test', expected.test), ('seed', expected.seed), ('branch', expected.branch), ('pid', evidence.pid), ('status', 'PENDING_NATIVE_TEST_RESULT')]:
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
    return Endpoint(start, end, founding, complete, tuple(pending), manifest_hash(dict(evidence.output_hashes)), text(terminal.field('stop_reason')))
