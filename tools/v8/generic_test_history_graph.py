# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_test_history
"""Shared native save ancestry semantics without independent process claims."""
import re
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_gate import boundary, document, require
from generic_sources import safe_path
from generic_test_endpoint import Endpoint
from generic_test_endpoint_types import ExpectedOccurrence, Observation, PendingReason, Snapshot, bound_bytes, fields, snapshot

@dataclass(frozen=True, slots=True)
class HistoryEvidence:
    root: Path
    pid: int
    output_hashes: tuple[tuple[str, str], ...]
    input_root: Path
    inputs: tuple[tuple[str, str], ...]

ROUNDTRIP: Final = 'save_load_roundtrip'
BASELINE: Final = 'save_load_roundtrip_baseline'
DEATHS: Final = 'save_load_roundtrip_through_deaths_and_births'
PREGNANCY: Final = 'save_load_roundtrip_through_a_pregnancy_and_a_due_birth'
DETERMINISTIC: Final = 'save_is_deterministic'
BRIDGE: Final = 'bridge_save_files_round_trip'
FORMAT_SOURCE: Final = 'rust/crates/sim-systems/src/save/mod.rs'
# Exact source-planned coverage, independent of emitted directories.
SAVE_OCCURRENCES: Final = {
    ROUNDTRIP: tuple((seed, branch) for seed in range(8) for branch in ('continuous', 'saved', 'loaded')),
    BASELINE: tuple((seed, branch) for seed in range(2) for branch in ('continuous', 'saved', 'loaded')),
    DEATHS: tuple((0, branch) for branch in ('continuous', 'original', 'loaded_1', 'loaded_2')),
    DETERMINISTIC: tuple((3, branch) for branch in ('original', 'twin', 'loaded_no_ticks')),
    BRIDGE: tuple((0, branch) for branch in ('original', 'loaded')),
    PREGNANCY: tuple((1, branch) for branch in ('pregnancy_continuous', 'pregnancy_saved', 'pregnancy_loaded',
                                              'queued_continuous', 'queued_saved', 'queued_loaded', 'queued_probe')),
}


@dataclass(frozen=True, slots=True)
class PrefixEdge:
    parent: str
    child: str
    checkpoint: str
    checkpoint_sha256: str
    save_artifact: str
    save_sha256: str
    tick: int
    version: int


def identity(row: ExpectedOccurrence) -> str:
    return f'{row.test}_s{row.seed}_{row.branch}'


def source_edge(row: ExpectedOccurrence) -> tuple[str, str] | None:
    """Return the source-declared parent branch/save label; never infer by bytes."""
    from generic_test_steward_scope import steward_edge
    edge = steward_edge(row)
    if edge is not None:
        return edge
    if row.test in {ROUNDTRIP, BASELINE} and row.branch == 'loaded':
        return 'saved', 'main'
    cases = {
        (DEATHS, 'loaded_1'): ('original', 'chain'),
        (DEATHS, 'loaded_2'): ('loaded_1', 'chain'),
        (DETERMINISTIC, 'loaded_no_ticks'): ('original', 'first'),
        (BRIDGE, 'loaded'): ('original', 'bridge_file'),
        (PREGNANCY, 'pregnancy_loaded'): ('pregnancy_saved', 'event'),
        (PREGNANCY, 'queued_loaded'): ('queued_saved', 'event'),
        (PREGNANCY, 'queued_probe'): ('queued_continuous', 'queued_probe'),
    }
    if (row.test, row.branch) in cases:
        return cases[row.test, row.branch]
    external = {
        'genuine_legacy_inventory_preserves_fields_and_current_policy_continuation': ('twin', 'initial'),
        'genuine_v10_saves_gain_defence_defaults_and_continue_identically_after_resave': ('reloaded', 'migrated'),
    }
    if row.test in external:
        child, parent = external[row.test]
        match = re.fullmatch(r'fixture_([0-9]+)_' + child, row.branch)
        if match:
            return f'fixture_{match.group(1)}_{parent}', 'current'
    if row.test == 'actual_h_format9_fixture_loads_as_oak_then_resaves_as_current_format' and row.branch == 'twin':
        return 'migrated', 'current'
    return None


def extends(before: Snapshot, after: Snapshot, label: str) -> None:
    require(before.tick <= after.tick and before.knowledge_enabled == after.knowledge_enabled
            and before.history_order_valid and after.history_order_valid, label, 'invalid history interval/capability')
    require(after.commands[:len(before.commands)] == before.commands, label, 'command sequence prefix changed')
    if before.losses is not None:
        require(after.losses is not None and after.losses[:len(before.losses)] == before.losses,
                label, 'loss sequence prefix changed')
    require(after.chronicle_entries >= before.chronicle_entries and after.births >= before.births
            and all(dict(after.deaths)[key] >= count for key, count in before.deaths), label, 'cumulative history decreased')


@dataclass(frozen=True, slots=True)
class ResolvedHistory:
    expected: ExpectedOccurrence
    founding: Snapshot | None
    history_complete: bool
    pending: tuple[PendingReason, ...]
    edges: tuple[PrefixEdge, ...]


@boundary
def resolve_graph(evidence: HistoryEvidence, expected: tuple[ExpectedOccurrence, ...],
                  observations: Mapping[str, Observation | Endpoint]) -> tuple[ResolvedHistory, ...]:
    """Resolve only registered save edges over already parsed original endpoints."""
    require(bool(expected) and len({identity(row) for row in expected}) == len(expected), '/plan', 'empty or duplicate occurrence plan')
    planned = {identity(row): row for row in expected}
    for test in {row.test for row in expected} & SAVE_OCCURRENCES.keys():
        rows = [row for row in expected if row.test == test]
        require({(row.seed, row.branch) for row in rows} == set(SAVE_OCCURRENCES[test]), test, 'incomplete source occurrence coverage')
        require(len({row.native_test for row in rows}) == 1 and all(row.ordinary for row in rows), test, 'native identity or ordinary classification changed')
    parent_dir = evidence.root / f'save_load_pid{evidence.pid}'
    require(parent_dir.is_dir() and not parent_dir.is_symlink()
            and {path.name for path in parent_dir.iterdir()} == set(planned), '/outputs', 'unplanned or missing occurrence folders')
    require(set(observations) == set(planned), '/observations', 'endpoint coverage differs from plan')
    hashes = dict(evidence.output_hashes)

    def read(path: Path):
        name = path.relative_to(evidence.root).as_posix()
        require(name in hashes, name, 'unbound graph artifact')
        return document(bound_bytes(safe_path(evidence.root, name), hashes[name]).decode())

    terminals = {name: read(parent_dir / name / 'terminal.json') for name in planned}
    versions: list[int] = []

    def current_version() -> int:
        if not versions:
            inputs = dict(evidence.inputs)
            require(FORMAT_SOURCE in inputs, FORMAT_SOURCE, 'source-bound native format constant required')
            source = bound_bytes(safe_path(evidence.input_root, FORMAT_SOURCE), inputs[FORMAT_SOURCE]).decode()
            matches = list(re.finditer(r'^pub const FORMAT_VERSION: u32 = ([0-9]+);$', source, re.MULTILINE))
            require(len(matches) == 1, FORMAT_SOURCE, 'ambiguous native save format constant')
            versions.append(int(matches[0].group(1)))
        return versions[0]

    active: set[str] = set()
    resolved: dict[str, ResolvedHistory] = {}

    def visit(name: str) -> ResolvedHistory:
        require(name not in active, name, 'cyclic save-prefix graph')
        if name in resolved:
            return resolved[name]
        active.add(name)
        row, observation, terminal = planned[name], observations[name], terminals[name]
        prefix = terminal.field('prefix')
        pending = set(observation.pending)
        founding = observation.founding
        edges: tuple[PrefixEdge, ...] = ()
        extends(observation.initial, observation.terminal, name)
        expected_edge = source_edge(row)
        if prefix.value is None:
            require(expected_edge is None, name, 'required loaded occurrence has no save link')
            complete = observation.history_complete and not pending and founding is not None and founding.tick == 0
        elif 'kind' in prefix.mapping():
            require(expected_edge is None, name, 'internal loaded occurrence replaced with external origin')
            require(founding is None, name, 'external origin cannot supply founders')
            complete = False
        else:
            require(expected_edge is not None, name, 'unregistered internal save edge')
            assert expected_edge is not None
            branch, label = expected_edge
            owner = f'{row.test}_s{row.seed}_{branch}'
            require(owner in planned and planned[owner].native_test == row.native_test, name, 'foreign or absent prefix producer')
            saved = parent_dir / owner / f'save_{label}.wsave'
            checkpoint_path = parent_dir / owner / f'save_{label}.json'
            prefix.field('save_artifact').equals(str(saved))
            prefix.field('checkpoint').equals(str(checkpoint_path))
            checkpoint = read(checkpoint_path)
            fields(checkpoint, 'state save_artifact byte_count magic input_version_le_bytes current_format_version external_sha256_required')
            checkpoint.field('state').equals(terminal.field('initial').value)
            saved_name = saved.relative_to(evidence.root).as_posix()
            require(saved_name in hashes, name, 'saved bytes absent from frozen manifest')
            raw = bound_bytes(saved, hashes[saved_name])
            version = current_version()
            require(len(raw) >= 12 and raw[:8] == b'WORLDSIM' and int.from_bytes(raw[8:12], 'little') == version,
                    name, 'saved bytes are not bound current format')
            checkpoint.field('current_format_version').equals(version)
            checkpoint.field('byte_count').equals(len(raw))
            checkpoint.field('save_artifact').equals(str(saved))
            checkpoint.field('magic').equals(list(raw[:8]))
            checkpoint.field('input_version_le_bytes').equals(list(raw[8:12]))
            checkpoint.field('external_sha256_required').equals(True)
            state = snapshot(checkpoint.field('state'))
            parent = visit(owner)
            extends(observations[owner].initial, state, owner)
            extends(state, observations[owner].terminal, owner)
            require(observation.founding == observations[owner].founding == parent.founding,
                    name, 'propagated founding snapshot differs')
            founding = parent.founding
            edges = parent.edges + (PrefixEdge(owner, name, str(checkpoint_path), hashes[checkpoint_path.relative_to(evidence.root).as_posix()],
                                                str(saved), hashes[saved_name], state.tick, version),)
            if parent.history_complete and not (pending - {PendingReason.PREFIX_GRAPH_UNRESOLVED}):
                pending.discard(PendingReason.PREFIX_GRAPH_UNRESOLVED)
            else:
                pending.update(parent.pending)
            complete = not pending and founding is not None and founding.tick == 0
        result = ResolvedHistory(row, founding, complete, tuple(sorted(pending)), edges)
        resolved[name] = result
        active.remove(name)
        return result

    result = tuple(visit(identity(row)) for row in expected)
    # Recheck the actual invocation/streams and every graph output after traversal.
    for name, digest in evidence.output_hashes:
        _ = bound_bytes(safe_path(evidence.root, name), digest)
    if versions:
        _ = bound_bytes(safe_path(evidence.input_root, FORMAT_SOURCE), dict(evidence.inputs)[FORMAT_SOURCE])
    return result
