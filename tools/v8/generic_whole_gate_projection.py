"""Project authenticated endpoints after complete native history resolution."""
from generic_gate import require
from generic_observations import Observation
from generic_test_endpoint import Endpoint
from generic_test_history_graph import ResolvedHistory


def project_endpoint(endpoint: Endpoint, history: ResolvedHistory) -> Observation:
    require(history.history_complete and not history.pending, '/history', 'unresolved native endpoint')
    founding = history.founding
    require(founding is not None and founding.tick == 0, '/founding', 'tick-zero founding required')
    assert founding is not None
    end = endpoint.terminal
    require(founding.knowledge_enabled == end.knowledge_enabled, '/knowledge', 'capability changed')
    losses = () if end.losses is None else tuple((row.tick, row.node, row.holder) for row in end.losses)
    return Observation(0, end.tick, len(founding.alive_ids), len(end.alive_ids),
                       end.knowledge_enabled, losses, '[]')
