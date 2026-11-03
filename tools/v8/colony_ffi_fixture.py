"""Synthetic endpoint and aggregate receipts, never claimed as native execution."""
import json
from pathlib import Path
from typing import final

from generic_process import file_hash
from generic_whole_gate_colony_ffi import PINS, TARGET, TESTS
from generic_whole_gate_workspace_plan import __file__ as plan_file
from mux2_gate_inputs import Json


def state() -> dict[str, Json]:
    return {'tick': 0, 'alive': 3, 'alive_ids': [1, 2, 3], 'raw_agent_population': 3,
            'raw_agent_ids': [1, 2, 3], 'census_semantics': 'alive excludes BodyHealth::is_dead; raw includes every Agent; wrapper must check ordinary endpoint agreement',
            'knowledge_capability': {'enabled': True, 'source': 'resources.rules.colony.knowledge'},
            'knowledge_lost': [], 'history_order_valid': True, 'chronicle_entries': 0, 'command_log': [],
            'births': 0, 'deaths': {key: 0 for key in ('starvation', 'dehydration', 'exposure', 'combat', 'old_age')}}


def command_rows() -> list[Json]:
    payloads: tuple[dict[str, Json], ...] = (
        {'cmd': 'designate_chop_tiles', 'tiles': [[1, 2], [2, 3], [3, 4]]},
        {'cmd': 'add_stockpile', 'x0': 1, 'y0': 1, 'x1': 2, 'y1': 2},
        {'cmd': 'place_blueprint', 'kind': 'hut', 'x': 10, 'y': 10})
    return [{'tick': 0, 'command': dict(value, tick=0), 'accepted': True, 'count': count,
             'result_debug': 'synthetic accepted command'} for value, count in zip(payloads, (3, 4, 1), strict=True)]


@final
class Fixture:
    def __init__(self, root: Path, step: int = 4) -> None:
        self.root = root.resolve()
        self.step = step
        original = Path(plan_file).resolve().parents[2]
        draft = Path(__file__).resolve().parent
        self.source = self.root / 'source'
        for relative, digest in PINS.items():
            local = draft / relative
            source = local if local.exists() else original / relative
            target = self.source / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            _ = target.write_bytes(source.read_bytes())
            assert file_hash(target) == digest
        gate = self.root / 'harness/tools/v8/gate.sh'
        gate.parent.mkdir(parents=True)
        command: list[Json] = ['bash', '-c', 'synthetic original aggregate command']
        _ = gate.write_text(f'step {step} original bash -c "{command[2]}"\n')
        product: dict[str, Json] = {'root': str(self.source), 'source_inputs': dict(PINS)}
        self.session: dict[str, Json] = {'arms': {'base': product, 'candidate': dict(product)},
            'output': str(self.root / 'output'), 'harness': {'root': str(self.root / 'harness'), 'inputs': {'tools/v8/gate.sh': file_hash(gate)}},
            'bash': {'path': '/bin/bash', 'sha256': file_hash(Path('/bin/bash'))}}
        self.stages: dict[str, dict[str, Json]] = {}
        self.receipts: dict[str, dict[str, Json]] = {}
        profile = 'debug' if step == 4 else 'release'
        raw = f'Running tests/{TARGET}.rs (target/{profile}/deps/{TARGET}-abcd)\nrunning 2 tests\n'
        raw += ''.join(f'test {test} ... ok\n' for test in TESTS)
        raw += 'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n'
        for arm in ('base', 'candidate'):
            process = self.process(arm)
            process.parent.mkdir(parents=True)
            env: dict[str, Json] = {'WORLDSIM_GATE_OBSERVER': '1', 'WORLDSIM_GATE_TEST_OUT': str(self.observer(arm))}
            self.stages[arm] = {'state': 'RETURNED', 'native_exit': 0, 'arm': arm, 'cwd': str(self.source), 'step': step,
                'argv': command, 'pid': 987, 'environment': env, 'process_receipt': str(process)}
            digest = file_hash(Path('/bin/bash'))
            self.receipts[arm] = {'argv': ['/bin/bash', '-c', 'exec "$@" 2>&1', 'whole-step', *command],
                'executable': '/bin/bash', 'expected_executable_sha256': digest, 'before_executable_sha256': digest,
                'after_executable_sha256': digest, 'pid': 987, 'native_exit': 0, 'cwd': str(self.source), 'environment': env,
                'status': 'PASS', 'errors': []}
            _ = process.with_name('stdout.log').write_text(raw)
            _ = process.with_name('stderr.log').write_text('')
            for seed, test in enumerate(TESTS):
                folder = self.folder(arm, seed)
                folder.mkdir(parents=True)
                initial = state()
                terminal = state()
                terminal['command_log'] = command_rows() if seed == 0 else []
                self.write(folder / 'initial.json', {'schema_version': 1, 'test': test, 'seed': seed, 'branch': 'primary',
                    'pid': 123, 'initial': initial, 'history_complete': True, 'status': 'PENDING_NATIVE_TEST_RESULT'})
                self.write(folder / 'terminal.json', {'schema_version': 1, 'initial': initial, 'founding': initial,
                    'terminal': terminal, 'completed_ticks': 0, 'stop_reason': 'original_endpoint', 'prefix': None,
                    'history_complete': True, 'loaded_prefix_requires_external_authentication': False,
                    'status': 'ENDPOINT_REACHED_NATIVE_TEST_RESULT_REQUIRED'})
                if seed == 1:
                    self.write(folder / 'checkpoint_carry_intervention.json', {'state': terminal,
                        'detail': {'kind': 'direct_carry_insert', 'selected_row': 2, 'selected_entity_bits': 103,
                                   'snapshot_entity_bits': [101, 102, 103], 'jobs': [[0, -1, 0], [0, -1, 0], [0, 0, 4]],
                                   'item': 'Log', 'count': 4}})
            self.seal(arm)

    def process(self, arm: str) -> Path:
        return self.root / 'output' / arm / 'stage/process/process.json'

    def observer(self, arm: str) -> Path:
        return self.root / 'output' / arm / 'stage/observer'

    def folder(self, arm: str, seed: int) -> Path:
        return self.observer(arm) / 'save_load_pid123' / f'{TESTS[seed]}_s{seed}_primary'

    @staticmethod
    def write(path: Path, value: Json) -> None:
        _ = path.write_text(json.dumps(value))

    @staticmethod
    def read(path: Path) -> dict[str, Json]:
        from generic_gate import document
        return dict(document(path.read_text()).mapping())

    def seal(self, arm: str) -> None:
        process = self.process(arm)
        for channel in ('stdout', 'stderr'):
            self.receipts[arm][channel + '_sha256'] = file_hash(process.with_name(channel + '.log'))
        self.write(process, self.receipts[arm])
        self.stages[arm]['process_receipt_sha256'] = file_hash(process)
        observer = self.observer(arm)
        self.stages[arm]['observer_artifacts'] = {str(path): file_hash(path) for path in observer.rglob('*') if path.is_file()}
        self.stages[arm]['observer_directories'] = {str(path): path.stat().st_mode for path in (observer, *observer.rglob('*')) if path.is_dir()}
