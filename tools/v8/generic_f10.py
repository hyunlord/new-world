"""Bind the historical fault oracle to the actual scripted intervention."""

from generic_gate import boundary, decode_json, document, integer, require, text
from generic_observations import Observation, ordinary_pair
from mux2_gate_inputs import Json, Node

KILL: dict[str, Json] = {"tick": 1800, "cmd": "debug_kill", "resolve": "holders_of", "knowledge": "doorway"}


@boundary
def companion_delta(source: str, companion: str) -> None:
    original = document('{"rows":' + source + '}').field("rows").array()
    other = document('{"rows":' + companion + '}').field("rows").array()
    values = [row.value for row in original]
    require(values.count(KILL) == 1, "/script", "expected exactly one registered F10 intervention")
    require([row.value for row in other] == [row for row in values if row != KILL],
            "/companion", "companion must remove only the registered kill entry")


@boundary
def injection(observed: Observation, script: str, command_log: str, summary_seed: str) -> Observation:
    """Return the pre-intervention state; terminal losses stay in the fault result."""
    script_rows = document('{"rows":' + script + '}').field("rows").array()
    sorted_rows = sorted(script_rows, key=lambda row: integer(row.field("tick")))
    values = [row.value for row in sorted_rows]
    require(values.count(KILL) == 1, "/script", "wrong F10 intervention")
    index = values.index(KILL)
    checkpoints = Node(decode_json(observed.checkpoints_json), "/checkpoints").array()
    routed = [row for row in checkpoints if row.field("phase").value == "before_tick_commands"]
    resolved = [row for row in checkpoints if row.field("phase").value == "before_debug_kill"]
    require(len(routed) == 1 and bool(resolved), "/checkpoints", "missing or repeated injection boundary")
    for row in checkpoints:
        row.field("tick").equals(1800)
        row.field("script_index").equals(index)
        row.field("script_spec").equals(KILL)
        row.field("knowledge_lost").equals([])
    agents = [integer(row.field("debug_kill_agent")) for row in resolved]
    require(len(agents) == len(set(agents)), "/checkpoints", "duplicate resolved kill target")
    require([integer(row.field("resolved_command_index")) for row in resolved] == list(range(len(resolved))),
            "/checkpoints", "missing or reordered resolved kill command")
    commands = document('{"rows":' + command_log + '}').field("rows").array()
    kills = [row for row in commands if row.field("cmd").value == "debug_kill"]
    require(len(kills) == len(agents), "/commands", "native kill receipt count differs from resolution")
    for row, agent in zip(kills, agents, strict=True):
        row.field("tick").equals(1800)
        row.field("agent").equals(agent)
        row.field("result").field("accepted").equals(True)
    require(observed.knowledge_enabled and bool(observed.losses), "/losses", "fault did not produce native knowledge loss")
    require(all(1800 <= tick <= 2040 and name == "doorway" for tick, name, _ in observed.losses),
            "/losses", "unexpected knowledge loss beyond the registered fault")
    summary = document(summary_seed)
    m4 = summary.field("m4")
    m4.field("lost").equals([list(row) for row in observed.losses])
    names = [row.text() for row in m4.field("node_names").array()]
    require(names.count("doorway") == 1, "/m4/node_names", "missing or duplicate doorway")
    statuses = m4.field("status_end").array()
    require(len(statuses) == len(names), "/m4/status_end", "status coverage differs")
    statuses[names.index("doorway")].equals(4)
    waits = [row.array() for row in m4.field("waits_end").array()]
    for row in waits:
        require(len(row) == 4, "/m4/waits_end", "invalid native wait row length")
        _ = integer(row[0], 2)
        for field in row[1:3]:
            require(type(field.value) is int and -(1 << 63) <= field.value < (1 << 63),
                    field.label, "expected native signed coordinate or order id")
        _ = text(row[3])
    require(any(row[0].value == 1 and row[3].value == "doorway" for row in waits),
            "/m4/waits_end", "F10 has no door waiting on lost knowledge")
    return Observation(observed.initial_tick, 1800, observed.founders,
                       integer(routed[0].field("alive")), True, (), "[]")


def paired_precondition(base: Observation, candidate: Observation) -> dict[str, Json]:
    """Called with authenticated injection() outputs, plus separately checked no-kill worlds."""
    return ordinary_pair(base, candidate)
