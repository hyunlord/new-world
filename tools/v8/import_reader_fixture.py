"""Synthetic disk records; upstream bridge authority is explicitly mocked in tests."""
import json
import sys
from pathlib import Path
from typing import final

import gate_oracle_child as controller
from generic_process import file_hash
from generic_setup_bridge import BridgeResult
from generic_setup_import import PriorBridge
from generic_setup_prepare import PreparedSetup, SetupInputs, Tool
from generic_sources import inventory
from mux2_gate_inputs import Json, Node
from test_generic_setup_bridge import unused_wait


@final
class ImportFixture:
    def __init__(self, home: Path) -> None:
        self.root = home/'product'
        self.output = home/'imports'
        self.settings = home/'settings'
        self.root.mkdir()
        self.output.mkdir()
        (self.output/'children').mkdir()
        (self.output/'logs').mkdir()
        source = Path(controller.__file__).parent
        names = ('run_bounded.sh','gate_oracle_child.py','gate_stall_child.py','generic_gate.py',
                 'generic_process.py','generic_sources.py','mux2_gate_inputs.py','mux2_gate_lock.py',
                 'gate.sh','generic_setup_import.py')
        for name in names:
            path = self.root/'tools/v8'/name
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_bytes((source/name).read_bytes())
        bridge = self.root/'rust/target/debug/libsim_bridge.dylib'
        bridge.parent.mkdir(parents=True)
        _ = bridge.write_text('synthetic bridge bytes, never executed')
        materialized = tuple((str(path.relative_to(self.root)), file_hash(path))
                             for path in sorted((self.root/'tools/v8').iterdir()))
        python = Path(sys.executable).resolve()
        tools = (Tool('godot', python, file_hash(python)), Tool('python3', python, file_hash(python)),
                 Tool('bash', Path('/bin/bash').resolve(), file_hash(Path('/bin/bash'))))
        environment = (('PATH', str(python.parent)+':/bin:/usr/bin'), ('GODOT_BIN', str(python)),
                       ('WORLDSIM_TEST_SETTINGS_DIR', str(self.settings)))
        context = SetupInputs(self.root, 'synthetic-source', home/'cargo-home', tools, environment,
                              materialized, (), home/'unused', '0'*64, self.settings, home/'bridge-evidence')
        self.prepared = PreparedSetup(context, 'synthetic-digest', materialized, (), ())
        bridge_receipt = home/'bridge.json'
        _ = bridge_receipt.write_text('{}')
        self.prior = PriorBridge(BridgeResult(bridge_receipt, 0, 7, file_hash(bridge)),
                                 file_hash(bridge_receipt), unused_wait(self.prepared))
        self.bridge_inputs = ((bridge, file_hash(bridge)), (bridge_receipt, file_hash(bridge_receipt)))
        inputs = dict(self.bridge_inputs)
        inputs.update({self.root/name: digest for name, digest in materialized})
        inputs.update({tool.path: tool.sha256 for tool in tools})
        self.write(self.output/'plan.json', {'schema': controller.IMPORT_SCHEMA, 'root': str(self.root),
            'evidence': str(self.output/'children'), 'logs': str(self.output/'logs'), 'godot': str(python),
            'inputs': {str(path): digest for path, digest in inputs.items()},
            'environment': {key: value for key, value in environment}})
        self.plan = controller.load_suite(self.output/'plan.json', file_hash(self.output/'plan.json'))
        common: dict[str, Json] = {'schema': controller.SUITE_SCHEMA, 'plan_sha256': self.plan.digest}
        self.write(self.plan.evidence/'suite-prepared.json', {**common, 'owner_pid': 10})
        self.write(self.plan.evidence/'suite-start.json', {**common, 'owner_pid': 10, 'shell_pid': 11,
            'cwd': str(self.root), 'argv': ['bash','-c',controller.IMPORT_COMMAND+'; '+controller.IMPORT_COMMAND]})
        _ = (self.plan.logs/'imports.log').write_text('first\nsecond\n')
        (self.root/'.godot').mkdir()
        _ = (self.root/'.godot/imported').write_text('original cache')
        checkpoint = controller.import_state(self.plan)
        for row, first, last in zip(self.plan.rows, (0, 6), (6, 13)):
            row.evidence.mkdir()
            info = row.log.stat()
            request: dict[str, Json] = {**common, 'ordinal': row.ordinal, 'identity': row.identity,
                'wrapper_pid': 20+row.ordinal, 'parent_pid': 11, 'cwd': str(self.root),
                'argv': list(row.argv), 'limit': 600, 'log_device': info.st_dev, 'log_inode': info.st_ino}
            self.write(row.evidence/'request.json', request)
            self.write(row.evidence/'wrapper-start.json', {'schema': controller.SUITE_SCHEMA,
                'request': request, 'wrapper_wait_observed': False})
            self.write(row.evidence/'native-start.json', {'schema': controller.SUITE_SCHEMA,
                'request': request, 'native_pid': 30+row.ordinal,
                'environment': {k: v for k, v in controller.suite_environment(self.plan).items()}, 'stream_start': first})
            from gate_stall_child import range_hash
            self.write(row.evidence/'native-terminal.json', {'schema': controller.SUITE_SCHEMA,
                'request': request, 'native_pid': 30+row.ordinal, 'native_exit': 0, 'timed_out': False,
                'settlement': 'SETTLED', 'error': '', 'stream_start': first, 'stream_end': last,
                'stream_sha256': range_hash(row.log, first, last), 'started_monotonic_ns': row.ordinal,
                'ended_monotonic_ns': row.ordinal+1})
            self.write(row.evidence/'import-checkpoint.json', checkpoint)
        self.write(self.plan.evidence/'suite-terminal.json', {**common, 'shell_pid': 11, 'shell_exit': 0,
            'settlement': 'SETTLED', 'completed': ['import.first','import.second'],
            'closed_log_sha256': {str(self.plan.rows[0].log): file_hash(self.plan.rows[0].log)},
            'wrapper_waits_observed': False, 'oracle_admission': 'CALLER_REQUIRED', 'error': ''})
        self.receipt = Node({'schema': 'setup_import_pair.v1', 'status': 'PASS_STEP3_ONLY', 'step': 3,
            'first_native_exit': 0, 'second_native_exit': 0, 'timed_out': [False,False], 'shell_exit': 0,
            'shell_pid': 11, 'ownership': 'SETTLED_ORIGINAL_IMMEDIATE_CHILDREN',
            'prior_bridge_sha256': self.prior.receipt_sha256, 'source_sha': context.source_sha,
            'bridge_sha256': self.prior.result.bridge_sha256, 'plan_sha256': self.plan.digest,
            'cache_terminal': checkpoint['cache'], 'settings_terminal': checkpoint['settings'],
            'runtime_closure': 'CALLER_OWNED_UNQUALIFIED', 'build_product_authority': False,
            'steps4_5_6': 'NOT_RUN', 'full_gate_complete': False})
        _ = self.seal()

    def write(self, path: Path, value: Json) -> None:
        _ = path.write_text(json.dumps(value))

    def seal(self) -> str:
        self.receipt.mapping()['artifacts'] = {k: v for k, v in inventory(self.output, ['children','logs','plan.json']).items()}
        self.write(self.output/'receipt.json', self.receipt.value)
        return file_hash(self.output/'receipt.json')
