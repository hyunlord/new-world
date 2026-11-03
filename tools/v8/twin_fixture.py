"""Disk-bound synthetic receipts only: no native process is executed or attested."""
import json
from pathlib import Path
from typing import final

from generic_gate import document
from generic_process import __file__ as process_file
from generic_process import file_hash
from generic_whole_gate_twins_plan import CASES
from generic_whole_gate_twins_sources import PINS
from mux2_gate_inputs import Json, Node
from test_generic_whole_gate_twins import rows


@final
class Fixture:
    def __init__(self, root: Path, step: int = 4) -> None:
        self.root = root.resolve()
        self.source = self.root / 'source'
        original = Path(process_file).resolve().parents[2]
        inputs: dict[str, Json] = dict(PINS)
        inputs.update({p.relative_to(original).as_posix(): file_hash(p)
                       for p in (original / 'rust/crates/sim-test/src').rglob('*.rs')})
        for relative in inputs:
            target = self.source / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            _ = target.write_bytes((original / relative).read_bytes())
        gate = self.root / 'harness/tools/v8/gate.sh'
        gate.parent.mkdir(parents=True)
        command: list[Json] = ['bash', '-c', 'synthetic original aggregate command']
        _ = gate.write_text(f'step {step} original bash -c "{command[2]}"\n')
        product: dict[str, Json] = {'root': str(self.source), 'source_inputs': inputs}
        digest = file_hash(Path('/bin/bash'))
        self.session: dict[str, Json] = {'arms': {'base': product, 'candidate': dict(product)},
            'output': str(self.root / 'output'),
            'harness': {'root': str(self.root / 'harness'), 'inputs': {'tools/v8/gate.sh': file_hash(gate)}},
            'bash': {'path': '/bin/bash', 'sha256': digest}}
        self.stages: dict[str, dict[str, Json]] = {}
        self.receipts: dict[str, dict[str, Json]] = {}
        profile = 'debug' if step == 4 else 'release'
        raw = f'Running unittests src/lib.rs (target/{profile}/deps/sim_test-abcd)\nrunning 2 tests\n'
        raw += ''.join(f'test {test} ... ok\n' for test in CASES.values())
        raw += 'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n'
        for arm, offset in (('base', 0), ('candidate', 1000)):
            process = self.process(arm)
            process.parent.mkdir(parents=True)
            self.observer(arm).mkdir()
            env: dict[str, Json] = {'WORLDSIM_GATE_OBSERVER': '1', 'WORLDSIM_GATE_TEST_OUT': str(self.observer(arm))}
            self.stages[arm] = {'state': 'RETURNED', 'native_exit': 0, 'arm': arm, 'cwd': str(self.source), 'step': step,
                'argv': command, 'pid': 987, 'environment': env, 'process_receipt': str(process)}
            self.receipts[arm] = {'argv': ['/bin/bash', '-c', 'exec "$@" 2>&1', 'whole-step', *command],
                'executable': '/bin/bash', 'expected_executable_sha256': digest, 'before_executable_sha256': digest,
                'after_executable_sha256': digest, 'pid': 987, 'native_exit': 0, 'cwd': str(self.source), 'environment': env,
                'status': 'PASS', 'errors': []}
            _ = process.with_name('stdout.log').write_text(raw)
            _ = process.with_name('stderr.log').write_text('')
            for path, node in rows(offset):
                self.write(self.observer(arm) / path, node.value)
            self.seal(arm)

    def process(self, arm: str) -> Path:
        return self.root / 'output' / arm / 'stage/process/process.json'

    def observer(self, arm: str) -> Path:
        return self.root / 'output' / arm / 'stage/observer'

    @staticmethod
    def write(path: Path, value: Json) -> None:
        _ = path.write_text(json.dumps(value))

    @staticmethod
    def read(path: Path) -> dict[str, Json]:
        return dict(document(path.read_text()).mapping())

    def seal(self, arm: str) -> None:
        process = self.process(arm)
        for channel in ('stdout', 'stderr'):
            self.receipts[arm][channel + '_sha256'] = file_hash(process.with_name(channel + '.log'))
        self.write(process, self.receipts[arm])
        self.stages[arm]['process_receipt_sha256'] = file_hash(process)
        observer = self.observer(arm)
        self.stages[arm]['observer_artifacts'] = {str(path): file_hash(path) for path in observer.glob('*.json')}
        self.stages[arm]['observer_directories'] = {str(observer): observer.stat().st_mode}

    def nodes(self) -> tuple[Node, Node, Node]:
        return Node(self.session), Node(self.stages['base']), Node(self.stages['candidate'])
