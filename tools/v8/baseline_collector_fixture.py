"""Synthetic disk receipts only; never native execution or qualification evidence."""
import json
from pathlib import Path
from typing import final

from generic_process import file_hash
from generic_whole_gate_baseline_observer import CASE, PINS
from generic_whole_gate_workspace_plan import __file__ as plan_file
from mux2_gate_inputs import Json, Node
from test_generic_whole_gate_baseline_observer import fixture


def raw_log(profile: str = 'debug') -> str:
    return (f'Running unittests src/lib.rs (target/{profile}/deps/sim_test-abcd)\n'
            f'running 1 test\ntest {CASE} ... ok\n'
            'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n')


@final
class SyntheticRun:
    """No subprocess runs: fixtures exercise authentic readers with synthetic bytes."""
    def __init__(self, root: Path, step: int = 4) -> None:
        self.root = root.resolve()
        self.step = step
        self.source = Path(plan_file).resolve().parents[2]
        self.output = self.root / 'output'
        self.command: list[Json] = ['bash', '-c', 'cargo test --workspace ' +
                                  ('--release ' if step == 5 else '') + '--no-fail-fast -- --test-threads=8']
        gate = self.root / 'harness/tools/v8/gate.sh'
        gate.parent.mkdir(parents=True)
        _ = gate.write_text(f'step {step} workspace bash -c "{self.command[2]}"\n')
        product: dict[str, Json] = {'root': str(self.source), 'source_inputs': dict(PINS)}
        self.session: dict[str, Json] = {
            'arms': {'base': product, 'candidate': dict(product)}, 'output': str(self.output),
            'harness': {'root': str(self.root / 'harness'), 'inputs': {'tools/v8/gate.sh': file_hash(gate)}},
            'bash': {'path': '/bin/bash', 'sha256': file_hash(Path('/bin/bash'))}}
        self.stages: dict[str, dict[str, Json]] = {}
        self.receipts: dict[str, dict[str, Json]] = {}
        for arm in ('base', 'candidate'):
            owner = self.output / arm / 'stage'
            process = owner / 'process/process.json'
            process.parent.mkdir(parents=True)
            observer = owner / 'observer'
            observer.mkdir()
            environment: dict[str, Json] = {'WORLDSIM_GATE_OBSERVER': '1', 'WORLDSIM_GATE_TEST_OUT': str(observer)}
            stage: dict[str, Json] = {'state': 'RETURNED', 'native_exit': 0, 'arm': arm, 'cwd': str(self.source),
                                     'step': step, 'argv': self.command, 'pid': 987, 'environment': environment,
                                     'process_receipt': str(process), 'observer_directories': {str(observer): observer.stat().st_mode},
                                     'observer_artifacts': {}}
            self.stages[arm] = stage
            digest = file_hash(Path('/bin/bash'))
            self.receipts[arm] = {'argv': ['/bin/bash', '-c', 'exec "$@" 2>&1', 'whole-step', *self.command],
                                  'executable': '/bin/bash', 'expected_executable_sha256': digest,
                                  'before_executable_sha256': digest, 'after_executable_sha256': digest,
                                  'pid': 987, 'native_exit': 0, 'cwd': str(self.source), 'environment': environment,
                                  'status': 'PASS', 'errors': []}
            _ = process.with_name('stderr.log').write_text('')
            self.write_log(arm, raw_log('release' if step == 5 else 'debug'))
            _ = self.write_mirror(arm, fixture())

    def process(self, arm: str) -> Path:
        return self.output / arm / 'stage/process/process.json'

    def observer(self, arm: str) -> Path:
        return self.output / arm / 'stage/observer'

    def seal(self, arm: str) -> None:
        process = self.process(arm)
        for channel in ('stdout', 'stderr'):
            self.receipts[arm][channel + '_sha256'] = file_hash(process.with_name(channel + '.log'))
        _ = process.write_text(json.dumps(self.receipts[arm]))
        self.stages[arm]['process_receipt_sha256'] = file_hash(process)

    def write_log(self, arm: str, raw: str) -> None:
        _ = self.process(arm).with_name('stdout.log').write_text(raw)
        self.seal(arm)

    def write_mirror(self, arm: str, node: Node, name: str = 'probe_pid123_world2.json') -> Path:
        path = self.observer(arm) / name
        _ = path.write_text(json.dumps(node.value))
        anchors = dict(Node(self.stages[arm]['observer_artifacts']).mapping())
        anchors[str(path)] = file_hash(path)
        self.stages[arm]['observer_artifacts'] = anchors
        return path
