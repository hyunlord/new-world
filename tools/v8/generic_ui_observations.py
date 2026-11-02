"""Strict native UI instance decoding; loaded history needs external save proof."""
import re
from dataclasses import dataclass

from generic_gate import boundary, document, integer, require, text
from generic_observations import Observation, keys
from mux2_gate_inputs import Node


def unsigned_string(node: Node) -> int:
    value = text(node)
    require(re.fullmatch(r"0|[1-9][0-9]*", value) is not None, node.label, "expected canonical unsigned decimal string")
    result = int(value)
    require(result < 1 << 64, node.label, "unsigned value out of range")
    return result


@dataclass(frozen=True)
class UIInstance:
    node_id: int
    ordinal: int
    reason: str
    seed: int
    initial_agent_ids: tuple[int, ...]
    terminal_agent_ids: tuple[int, ...]
    closed: bool
    history_complete: bool
    history_gap: str | None
    chronicle_cursor: int
    state: Observation


@boundary
def parse_native(raw: str, expected_node_id: int) -> tuple[UIInstance, ...]:
    node = document(raw)
    keys(node, {"schema_version", "status", "node_instance_id", "instances"})
    node.field("schema_version").equals(1)
    node.field("status").equals("OBSERVED")
    require(unsigned_string(node.field("node_instance_id")) == expected_node_id, node.label, "wrong native node")
    result: list[UIInstance] = []
    rows = node.field("instances").array()
    require(bool(rows), node.label, "missing native instances")
    for index, row in enumerate(rows):
        keys(row, {"instance_ordinal", "reason", "seed", "initial_tick", "initial_population",
                   "initial_agent_ids", "terminal_tick", "terminal_alive", "terminal_agent_ids", "closed", "history_complete",
                   "history_gap", "chronicle_cursor", "knowledge_enabled", "knowledge_capability_source",
                   "knowledge_lost"})
        row.field("instance_ordinal").equals(index)
        start = unsigned_string(row.field("initial_tick"))
        end = unsigned_string(row.field("terminal_tick"))
        require(end >= start, row.label, "reversed native horizon")
        ids = tuple(unsigned_string(agent) for agent in row.field("initial_agent_ids").array())
        require(ids == tuple(sorted(set(ids))), row.label, "duplicate or unsorted native Agent census")
        require(integer(row.field("initial_population")) == len(ids), row.label, "initial census count differs")
        terminal_ids = tuple(unsigned_string(agent) for agent in row.field("terminal_agent_ids").array())
        require(terminal_ids == tuple(sorted(set(terminal_ids))), row.label,
                "duplicate or unsorted terminal Agent census")
        require(integer(row.field("terminal_alive")) == len(terminal_ids), row.label,
                "terminal census count differs")
        closed = row.field("closed").value
        complete = row.field("history_complete").value
        enabled = row.field("knowledge_enabled").value
        require(type(closed) is bool and type(complete) is bool and type(enabled) is bool, row.label, "nonboolean native status")
        if not isinstance(closed, bool) or not isinstance(complete, bool) or not isinstance(enabled, bool):
            raise TypeError("validated native booleans")
        require(closed == (index < len(rows) - 1), row.label, "closed/current instance lifecycle mismatch")
        reason = text(row.field("reason"))
        if reason == "load":
            require(not complete, row.label, "loaded history cannot self-authenticate")
        gap_value = row.field("history_gap").value
        require(gap_value is None or isinstance(gap_value, str) and bool(gap_value), row.label, "invalid history gap")
        gap = gap_value if isinstance(gap_value, str) else None
        require((complete and gap is None and start == 0) or (not complete and gap is not None),
                row.label, "inconsistent native history evidence")
        row.field("knowledge_capability_source").equals("resources.rules.colony.knowledge")
        loss_rows: list[tuple[int, str, int | None]] = []
        if enabled:
            for loss in row.field("knowledge_lost").array():
                keys(loss, {"tick", "node", "node_index", "holder"})
                tick = unsigned_string(loss.field("tick"))
                require(tick <= end and (not loss_rows or tick >= loss_rows[-1][0]), loss.label, "invalid native loss order")
                _ = integer(loss.field("node_index"))
                holder = loss.field("holder")
                loss_rows.append((tick, text(loss.field("node")), None if holder.value is None else unsigned_string(holder)))
        else:
            row.field("knowledge_lost").equals(None)
        state = Observation(start, end, len(ids), integer(row.field("terminal_alive")), enabled, tuple(loss_rows), "[]")
        result.append(UIInstance(expected_node_id, index, reason, unsigned_string(row.field("seed")), ids, terminal_ids,
                                 closed, complete, gap, integer(row.field("chronicle_cursor")), state))
    return tuple(result)


def require_fresh_history(instance: UIInstance) -> Observation:
    require(instance.history_complete and instance.history_gap is None, "/history",
            "native loaded/incomplete history requires authenticated save-prefix continuity")
    return instance.state


def validate_progress(previous: tuple[UIInstance, ...], current: tuple[UIInstance, ...]) -> None:
    """Repeated reads may extend history, never replace or silently omit instances."""
    require(len(current) >= len(previous), "/instances", "native instance history shrank")
    for before, after in zip(previous, current):
        require((before.node_id, before.ordinal, before.reason, before.seed, before.initial_agent_ids,
                 before.state.initial_tick, before.state.knowledge_enabled) ==
                (after.node_id, after.ordinal, after.reason, after.seed, after.initial_agent_ids,
                 after.state.initial_tick, after.state.knowledge_enabled), "/instances", "native initial identity changed")
        require(after.state.terminal_tick >= before.state.terminal_tick
                and after.chronicle_cursor >= before.chronicle_cursor
                and after.state.losses[:len(before.state.losses)] == before.state.losses,
                "/instances", "native time or loss history regressed")
        require(before.history_complete or not after.history_complete, "/instances",
                "incomplete native history cannot become authenticated by a later read")
        if before.closed:
            require(before == after, "/instances", "closed native instance changed")
