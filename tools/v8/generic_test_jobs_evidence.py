"""Bound, source-shaped colony_jobs checkpoint reading without process inventions."""
from dataclasses import dataclass
from itertools import pairwise

from generic_gate import document, require
from generic_test_endpoint_types import Snapshot, bound_bytes, fields, snapshot
from generic_test_history_graph import HistoryEvidence, extends
from mux2_gate_inputs import Node


@dataclass(frozen=True, slots=True)
class JobEvidence:
    graph: HistoryEvidence
    test: str
    branch: str
    seed: int

    @property
    def folder(self) -> str:
        return f'save_load_pid{self.graph.pid}/{self.test}_s{self.seed}_{self.branch}'

    def read(self, name: str) -> Node:
        relative = self.folder + '/' + name + '.json'
        hashes = dict(self.graph.output_hashes)
        require(relative in hashes, relative, 'missing colony_jobs evidence')
        return document(bound_bytes(self.graph.root / relative, hashes[relative]).decode())

    def checkpoint(self, name: str) -> tuple[Snapshot, Node]:
        node = self.read('checkpoint_' + name)
        fields(node, 'state detail')
        return snapshot(node.field('state')), node.field('detail')

    def chain(self, checkpoints: tuple[str, ...]) -> tuple[Snapshot, Snapshot]:
        expected = {'initial.json', 'terminal.json', *(f'checkpoint_{name}.json' for name in checkpoints)}
        require({path.name for path in (self.graph.root / self.folder).iterdir()} == expected,
                self.folder, 'missing or extra original checkpoint')
        initial = snapshot(self.read('initial').field('initial'))
        terminal = snapshot(self.read('terminal').field('terminal'))
        states = [initial, *(self.checkpoint(name)[0] for name in checkpoints), terminal]
        for before, after in pairwise(states):
            extends(before, after, self.folder)
        require(initial.tick == 0, self.folder, 'founding boundary must precede every tick')
        return initial, terminal


def coordinates(node: Node) -> tuple[int, int]:
    from generic_gate import integer
    values = tuple(integer(value) for value in node.array())
    require(len(values) == 2, node.label, 'two coordinates required')
    return values[0], values[1]
