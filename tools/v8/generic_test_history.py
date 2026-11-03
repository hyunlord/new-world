"""Authenticate actual native test processes then resolve shared save ancestry."""
from dataclasses import dataclass

from generic_gate import boundary
from generic_sources import safe_path
from generic_test_observations import Binding, ExpectedOccurrence, Observation, PendingReason, Snapshot, authenticate, bound_bytes, process_receipt
from generic_test_history_graph import HistoryEvidence, resolve_graph
from generic_test_history_graph import (
    ROUNDTRIP as ROUNDTRIP,
    BASELINE as BASELINE,
    DEATHS as DEATHS,
    PREGNANCY as PREGNANCY,
    DETERMINISTIC as DETERMINISTIC,
    BRIDGE as BRIDGE,
    FORMAT_SOURCE as FORMAT_SOURCE,
    SAVE_OCCURRENCES as SAVE_OCCURRENCES,
    PrefixEdge as PrefixEdge,
    identity as identity,
    source_edge as source_edge,
    extends as extends,
)

@dataclass(frozen=True, slots=True)
class ResolvedOccurrence:
    expected: ExpectedOccurrence
    observation: Observation
    founding: Snapshot | None
    history_complete: bool
    pending: tuple[PendingReason, ...]
    edges: tuple[PrefixEdge, ...]


@boundary
def resolve(binding: Binding, expected: tuple[ExpectedOccurrence, ...]) -> tuple[ResolvedOccurrence, ...]:
    """Preserve actual per-process authentication before shared semantic validation."""
    observations = {identity(row): authenticate(binding, row) for row in expected}
    evidence = HistoryEvidence(binding.process.output_root, binding.pid, binding.output_hashes,
                               binding.process.input_root, binding.process.inputs)
    resolved = resolve_graph(evidence, expected, observations)
    _ = process_receipt(binding)
    for name, digest in binding.output_hashes:
        _ = bound_bytes(safe_path(binding.process.output_root, name), digest)
    return tuple(ResolvedOccurrence(row.expected, observations[identity(row.expected)], row.founding,
                                    row.history_complete, row.pending, row.edges) for row in resolved)
