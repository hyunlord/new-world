"""Synthetic semantic fixture, with no process receipt or native execution claim."""
import json
import tempfile
from pathlib import Path
from typing import final

import generic_process
from generic_process import file_hash
from generic_test_history_graph import HistoryEvidence
from generic_whole_gate_knowledge_fresh import ALL_TESTS, occurrences
from generic_whole_gate_knowledge_fresh import PINS as INPUTS
from jobs_fixture import state
from mux2_gate_inputs import Json, Node

ROOT = Path(generic_process.__file__).resolve().parents[2]
DRAFT = Path(__file__).resolve().parents[2]


@final
class Fixture:
    def __init__(self, root: Path) -> None:
        self.root = root.resolve()
        self.source = self.root / 'source'
        self.observer = self.root / 'stage/observer'
        self.parent = self.observer / 'save_load_pid123'
        self.parent.mkdir(parents=True)
        self.inputs: dict[str, str] = {}
        for name, digest in INPUTS.items():
            source = DRAFT / name if (DRAFT / name).exists() else ROOT / name
            path = self.source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_bytes(source.read_bytes())
            assert file_hash(path) == digest
            self.inputs[name] = digest
        for row in occurrences():
            self.world(row.test, row.seed, row.end_tick)

    def world(self, test: str, seed: int, end: int) -> None:
        folder = self.parent / f'{test}_s{seed}_primary'
        folder.mkdir()
        initial = state(0, 10, True)
        terminal = state(end, 10, True)
        self.write(folder / 'initial.json', {'schema_version': 1, 'test': test, 'seed': seed, 'branch': 'primary', 'pid': 123,
                                            'initial': initial, 'history_complete': True, 'status': 'PENDING_NATIVE_TEST_RESULT'})
        self.write(folder / 'terminal.json', {'schema_version': 1, 'initial': initial, 'founding': initial, 'terminal': terminal,
                                             'completed_ticks': end, 'stop_reason': 'original_endpoint', 'prefix': None,
                                             'history_complete': True, 'loaded_prefix_requires_external_authentication': False,
                                             'status': 'ENDPOINT_REACHED_NATIVE_TEST_RESULT_REQUIRED'})

    @staticmethod
    def write(path: Path, value: Json) -> None:
        _ = path.write_text(json.dumps(value))

    def graph(self) -> HistoryEvidence:
        files = tuple((p.relative_to(self.observer).as_posix(), file_hash(p)) for p in self.observer.rglob('*') if p.is_file())
        return HistoryEvidence(self.observer, 123, files, self.source, tuple(self.inputs.items()))

    def session(self) -> Node:
        return Node({'arms': {arm: {'root': str(self.source), 'source_inputs': dict(self.inputs)} for arm in ('base', 'candidate')}})

    def stage(self) -> Node:
        return Node({'step': 4, 'arm': 'base', 'process_receipt': str(self.root / 'stage/process/process.json'),
                     'environment': {'WORLDSIM_GATE_OBSERVER': '1', 'WORLDSIM_GATE_TEST_OUT': str(self.observer)},
                     'observer_artifacts': {str(p): file_hash(p) for p in self.observer.rglob('*') if p.is_file()},
                     'observer_directories': {str(p): p.stat().st_mode for p in (self.observer, *self.observer.rglob('*')) if p.is_dir()}})

    def raw(self) -> str:
        return 'Running tests/colony_knowledge.rs (target/debug/deps/colony_knowledge-synthetic)\nrunning 17 tests\n' + ''.join(f'test {name} ... ok\n' for name in ALL_TESTS) + 'test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n'


def source_path(name: str) -> Path:
    draft = DRAFT / name
    return draft if draft.exists() else ROOT / name


def child_emission() -> str:
    with tempfile.TemporaryDirectory() as directory:
        fixture = Fixture(Path(directory))
        payload = {str(path.relative_to(fixture.parent)): path.read_text() for path in fixture.parent.rglob('*.json')}
    return "\nfor name, raw in " + repr(payload) + ".items():\n p=parent/name;p.parent.mkdir(exist_ok=True);r=json.loads(raw)\n if 'pid' in r:r['pid']=os.getpid()\n p.write_text(json.dumps(r))\n"
