# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_test_observations
"""Native endpoint values and strict snapshot parsing shared across process scopes."""
import hashlib
import json
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path

from generic_gate import integer, require, text
from generic_process import check_digest, real_path
from mux2_gate_inputs import Node

class PendingReason(StrEnum):
    PREFIX_GRAPH_UNRESOLVED = 'PREFIX_GRAPH_UNRESOLVED'
    EXTERNAL_ORIGIN_UNRESOLVED = 'EXTERNAL_ORIGIN_UNRESOLVED'
    NONZERO_INITIAL_WITHOUT_PREFIX = 'NONZERO_INITIAL_WITHOUT_PREFIX'
    NATIVE_PROCESS_FAILED = 'NATIVE_PROCESS_FAILED'
    NATIVE_TEST_OUTCOME_MISSING = 'NATIVE_TEST_OUTCOME_MISSING'
    NATIVE_TEST_NOT_OK = 'NATIVE_TEST_NOT_OK'


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
