"""Pure ordinary renderer comparison, deliberately not qualification admission."""
from collections import Counter
from dataclasses import dataclass

from generic_gate import document, integer, require
from mux2_gate_inputs import Node


@dataclass(frozen=True, slots=True)
class RendererArtifacts:
    text: bytes
    godot: bytes
    commands: bytes


def tiles(node: Node) -> Counter[tuple[int, ...]]:
    rows = node.array()
    require(bool(rows), node.label, 'empty person set')
    result: Counter[tuple[int, ...]] = Counter()
    for row in rows:
        coordinates = tuple(integer(value) for value in row.array())
        require(len(coordinates) == 2, row.label, 'expected tile pair')
        result[coordinates] += 1
    return result


def appearances(node: Node) -> list[tuple[int, int, int]]:
    rows = [(integer(row.field('agent_id')), integer(row.field('look')),
             integer(row.field('variant'))) for row in node.array()]
    require(bool(rows), node.label, 'empty appearance set')
    require(len({row[0] for row in rows}) == len(rows), node.label, 'duplicate agent id')
    return sorted(rows)


def ordinary(value: RendererArtifacts, expected_commands: bytes) -> None:
    text = document(value.text.decode())
    godot = document(value.godot.decode())
    for node in (text, godot):
        require(integer(node.field('seed')) == 100, node.label, 'expected seed 100')
        require(integer(node.field('tick')) == 900, node.label, 'expected tick 900')
        require(integer(node.field('count')) == len(node.field('people').array()),
                node.label, 'person count differs')
    require(godot.field('settled').value is True, '/settled', 'renderer not settled')
    require(tiles(text.field('people')) == tiles(godot.field('people')),
            '/people', 'text/Godot person tiles differ')
    require(appearances(text.field('appearance')) == appearances(godot.field('detail')),
            '/appearance', 'text/Godot appearance differs')
    require(value.commands == expected_commands, '/commands', 'concrete command bytes differ')
    commands = document('{"commands":'+value.commands.decode()+'}').field('commands').array()
    require(bool(commands), '/commands', 'empty concrete commands')
    current_tick = 0
    for command in commands:
        tick = integer(command.field('tick'))
        require(current_tick <= tick < 900, '/commands', 'command outside monotonic half-open replay')
        current_tick = tick
        _ = command.field('cmd').text()
        require(command.field('result').field('accepted').value is True,
                '/commands', 'concrete command rejected')


def compare_renderer(left: RendererArtifacts, right: RendererArtifacts,
                     expected_commands: bytes) -> None:
    """Retain all bytes including unknown fields; authenticate producers separately."""
    ordinary(left, expected_commands)
    ordinary(right, expected_commands)
    for name, before, after in (('text', left.text, right.text),
                                 ('godot', left.godot, right.godot),
                                 ('commands', left.commands, right.commands)):
        require(before == after, '/renderer/'+name, 'complete artifact bytes differ')
