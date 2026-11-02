# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_observations.py
"""Validate native probe sidecars against runner-owned invocation identities.

Parsing is not provenance: callers must bind these bytes to a successful fresh
child, its executable, effective inputs and planned occurrence inventory.
"""
import json
from dataclasses import dataclass

from generic_gate import (
    boundary,
    document,
    integer,
    require,
    text,
    validate_ordinary_safety,
)
from mux2_gate_inputs import Json, Node


@dataclass(frozen=True)
class ProbeWorld:
    scene: str
    seed: int
    occurrence: str
    duration: int


@dataclass(frozen=True)
class Observation:
    initial_tick: int
    terminal_tick: int
    founders: int
    alive: int
    knowledge_enabled: bool
    losses: tuple[tuple[int, str, int | None], ...]
    checkpoints_json: str


def keys(node: Node, expected: set[str]) -> None:
    require(set(node.mapping()) == expected, node.label, "missing or unknown observer fields")


def losses(node: Node, enabled: bool, end_tick: int) -> tuple[tuple[int, str, int | None], ...]:
    if not enabled:
        node.equals(None)
        return ()
    result: list[tuple[int, str, int | None]] = []
    for row in node.array():
        keys(row, {"tick", "node", "holder"})
        tick = integer(row.field("tick"))
        require(tick <= end_tick and (not result or tick >= result[-1][0]), row.label,
                "loss outside endpoint or chronicle order")
        holder = row.field("holder")
        result.append((tick, text(row.field("node")), None if holder.value is None else integer(holder)))
    return tuple(result)


@boundary
def parse_probe(raw: str, expected: ProbeWorld) -> Observation:
    node = document(raw)
    keys(node, {"schema_version", "scene", "seed", "occurrence", "requested_ticks",
                "requested_terminal_tick", "initial_tick", "initial_population", "terminal_tick",
                "terminal_alive", "terminal_complete", "history_complete", "history_source",
                "knowledge_capability", "knowledge_lost", "deaths", "checkpoints"})
    node.field("schema_version").equals(1)
    node.field("scene").equals(expected.scene)
    node.field("seed").equals(expected.seed)
    node.field("occurrence").equals(expected.occurrence)
    require(expected.occurrence in {"primary", "determinism_twin"}, node.label, "unknown probe occurrence")
    require(type(expected.duration) is int and expected.duration > 0, node.label, "invalid planned duration")
    node.field("requested_ticks").equals(expected.duration)
    start = integer(node.field("initial_tick"))
    end = integer(node.field("terminal_tick"))
    require(start == 0, node.label, "probe prior history is not authenticated")
    require(end == start + expected.duration, node.label, "actual endpoint differs from planned duration")
    node.field("requested_terminal_tick").equals(end)
    node.field("terminal_complete").equals(True)
    node.field("history_complete").equals(True)
    node.field("history_source").equals("fresh_world_append_only_chronicle")
    capability = node.field("knowledge_capability")
    keys(capability, {"enabled", "source"})
    capability.field("source").equals("resources.rules.colony.knowledge")
    enabled = capability.field("enabled").value
    require(type(enabled) is bool, capability.label, "knowledge capability must be boolean")
    assert isinstance(enabled, bool), "validated capability"
    loss = losses(node.field("knowledge_lost"), enabled, end)
    deaths = node.field("deaths")
    keys(deaths, {"starvation", "dehydration", "exposure", "combat", "old_age"})
    for name in deaths.mapping():
        _ = integer(deaths.field(name))
    previous_tick = start
    previous_loss_count = 0
    for checkpoint in node.field("checkpoints").array():
        keys(checkpoint, {"phase", "tick", "alive", "script_index", "script_spec",
                          "resolved_command_index", "debug_kill_agent", "history_complete", "knowledge_lost"})
        tick = integer(checkpoint.field("tick"))
        require(previous_tick <= tick <= end, checkpoint.label, "checkpoint outside ordered world history")
        previous_tick = tick
        _ = integer(checkpoint.field("alive"))
        _ = integer(checkpoint.field("script_index"))
        _ = checkpoint.field("script_spec").mapping()
        checkpoint.field("history_complete").equals(True)
        prefix = losses(checkpoint.field("knowledge_lost"), enabled, tick)
        require(loss[:len(prefix)] == prefix, checkpoint.label, "checkpoint losses are not terminal prefix")
        require(len(prefix) >= sum(row[0] < tick for row in loss), checkpoint.label,
                "checkpoint omits earlier native losses")
        require(len(prefix) >= previous_loss_count, checkpoint.label, "checkpoint loss history shrank")
        previous_loss_count = len(prefix)
        phase = text(checkpoint.field("phase"))
        require(phase in {"before_tick_commands", "before_debug_kill"}, checkpoint.label, "unknown checkpoint phase")
        for name in ("resolved_command_index", "debug_kill_agent"):
            if phase == "before_tick_commands":
                checkpoint.field(name).equals(None)
            else:
                _ = integer(checkpoint.field(name))
    return Observation(start, end, integer(node.field("initial_population")),
                       integer(node.field("terminal_alive")), enabled, loss,
                       json.dumps(node.field("checkpoints").value, sort_keys=True))


def ordinary_pair(base: Observation, candidate: Observation) -> dict[str, Json]:
    """Compose strict safety only after both native sidecars were fully validated."""
    require((base.initial_tick, base.terminal_tick, base.founders, base.knowledge_enabled) ==
            (candidate.initial_tick, candidate.terminal_tick, candidate.founders, candidate.knowledge_enabled),
            "/pair", "initial census, horizon or native knowledge capability differs between arms")
    result: dict[str, Json] = {
        "classification": "ordinary", "complete": True,
        "initial_tick": candidate.initial_tick, "end_tick": candidate.terminal_tick,
        "planned_end_tick": candidate.terminal_tick, "founders": candidate.founders,
        "founder_census_tick": candidate.initial_tick,
        "base_alive": base.alive, "candidate_alive": candidate.alive,
        "base_lost": [list(row) for row in base.losses],
        "lost": [list(row) for row in candidate.losses],
        "prior_loss_history_complete": True, "base_prior_loss_history_complete": True,
    }
    validate_ordinary_safety(json.dumps(result))
    return result
