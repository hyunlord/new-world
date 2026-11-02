"""Pair authenticated native test histories; no build or publication authority."""
from dataclasses import dataclass

from generic_gate import require
from generic_observations import Observation as OrdinaryObservation
from generic_observations import ordinary_pair
from generic_test_history import ResolvedOccurrence, resolve
from generic_test_observations import Binding, ExpectedOccurrence
from mux2_gate_inputs import Json


@dataclass(frozen=True, slots=True)
class PairedOccurrence:
    base: ResolvedOccurrence
    candidate: ResolvedOccurrence
    safety: dict[str, Json]


@dataclass(frozen=True, slots=True)
class PairedSafety:
    base_binding: Binding
    candidate_binding: Binding
    occurrences: tuple[PairedOccurrence, ...]


def _key(row: ExpectedOccurrence) -> tuple[str, str, int, str]:
    return row.test, row.native_test, row.seed, row.branch


def _project(row: ResolvedOccurrence) -> OrdinaryObservation:
    label = str(_key(row.expected))
    require(row.expected.ordinary and row.history_complete and not row.pending,
            label, 'nonordinary or unresolved native history')
    native = row.observation
    require(native.native_exit == 0 and native.native_test_outcome == 'ok',
            label, 'native process/test did not pass')
    founding = row.founding
    require(founding is not None and founding.tick == 0, label, 'tick-zero founding required')
    assert founding is not None
    terminal = native.terminal
    for state in (founding, native.initial, terminal):
        require(state.alive_ids == state.raw_agent_ids, label, 'ordinary live/raw Agent census differs')
    require(founding.knowledge_enabled == terminal.knowledge_enabled, label, 'founding capability differs')
    if terminal.knowledge_enabled:
        require(terminal.losses is not None, label, 'enabled knowledge lacks full history')
        assert terminal.losses is not None
        losses = tuple((loss.tick, loss.node, loss.holder) for loss in terminal.losses)
    else:
        require(terminal.losses is None, label, 'disabled knowledge must retain null history')
        losses = ()
    # Native save checkpoints remain in ResolvedOccurrence.edges/observation;
    # they are not the probe's scripted-command checkpoints.
    return OrdinaryObservation(founding.tick, terminal.tick, len(founding.alive_ids),
                               len(terminal.alive_ids), terminal.knowledge_enabled, losses, '[]')


def pair(base: Binding, base_plan: tuple[ExpectedOccurrence, ...],
         candidate: Binding, candidate_plan: tuple[ExpectedOccurrence, ...]) -> PairedSafety:
    """Resolve both actual invocations internally and apply unchanged ordinary bars.

    Caller still authenticates independent source/build provenance and completeness
    of its source plans. Binding input manifests and raw graph edges are retained.
    Dynamic horizons must match; a later parent cannot replace a child endpoint.
    """
    require(base.process.output_root != candidate.process.output_root
            and base.process.input_root != candidate.process.input_root,
            '/bindings', 'independent invocations and materialized inputs required')
    require(bool(base_plan) and len({_key(row) for row in base_plan}) == len(base_plan)
            and len({_key(row) for row in candidate_plan}) == len(candidate_plan)
            and {_key(row) for row in base_plan} == {_key(row) for row in candidate_plan},
            '/plans', 'missing, duplicate or mismatched occurrence coverage')
    left = resolve(base, base_plan)
    right = {_key(row.expected): row for row in resolve(candidate, candidate_plan)}
    pairs: list[PairedOccurrence] = []
    for original in left:
        other = right[_key(original.expected)]
        safety = ordinary_pair(_project(original), _project(other))
        pairs.append(PairedOccurrence(original, other, safety))
    return PairedSafety(base, candidate, tuple(pairs))
