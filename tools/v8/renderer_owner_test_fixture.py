"""Synthetic owner sequencing only; admission seams do not confer product authority.

No executable is launched. Real hashes, inventories, ProcessSpecs, process receipt
validation, ordinary renderer comparison, lock ownership and publication remain.
"""
import json
from collections.abc import Generator
from contextlib import ExitStack, contextmanager
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_renderer_qualification_owner as owner
import generic_setup_bridge as bridge_module
from generic_build import BuildResult, NormalProduct, NormalProducts, WorkspaceBuildPair
from generic_gate import document
from generic_process import ProcessResult, ProcessSpec, file_hash
from generic_setup_bridge import BridgeResult
from generic_setup_import import ImportFailure, ImportOwner, ImportResult, PriorBridge
from generic_setup_prepare import BRIDGE, PreparedSetup, SetupInputs, Tool
from generic_sources import inventory
from mux2_gate_inputs import Json, Node
from test_generic_renderer_run import process_receipt
from test_generic_setup_bridge import unused_wait


class OwnerFixture:
    def __init__(self, home: Path) -> None:
        self.home: Path = home
        self.root: Path = home/'product'
        self.output: Path = home/'run'
        self.events: list[str] = []
        self.failure: str = ''
        self.lock: Path = home/'lane'
        _ = self.lock.write_text('synthetic private lane')
        probe = self.root/'rust/target/release/worldsim_probe'
        bridge = self.root/BRIDGE
        for path in (probe, bridge, home/'godot'):
            path.parent.mkdir(parents=True, exist_ok=True)
            _ = path.write_text('synthetic bytes, never executed')
        for name in ('pair', 'debug', 'release'):
            (home/name).mkdir()
            _ = (home/name/'receipt.json').write_text('{}')
        debug = BuildResult(home/'debug/receipt.json', 'PASS', ())
        release = BuildResult(home/'release/receipt.json', 'PASS', ())
        pair = WorkspaceBuildPair(home/'pair/receipt.json', 'BUILD_ANTECEDENTS_ONLY', debug, release)
        tools = (Tool('godot', home/'godot', file_hash(home/'godot')),)
        inputs = SetupInputs(self.root, 'a'*40, home/'cargo', tools, (), (), (),
                             home/'origin', 'b'*64, home/'settings', home/'bridge')
        self.prepared: PreparedSetup = PreparedSetup(inputs, 'c'*64, (), (), (), warm_pair=pair)
        selected = NormalProduct('sim-test', 'synthetic', 'worldsim_probe', self.root/'main.rs',
                                 self.root/'Cargo.toml', 'a'*40, 'release', probe, file_hash(probe),
                                 'd'*64, 'e'*64, 1, 'f'*64)
        self.normal: NormalProducts = NormalProducts(pair, 'release', (('sim-test', 'worldsim_probe'),), (selected,))
        self.bridge: BridgeResult = BridgeResult(home/'bridge/receipt.json', 0, 101, file_hash(bridge))
        # Uninitialized real class identity: only patched finish is callable. Never admitted as runtime proof.
        self.import_owner: ImportOwner = ImportOwner.__new__(ImportOwner)
        self.import_result: ImportResult = ImportResult(self.output/'imports/receipt.json', 0, 0, 0, 'PASS_STEP3_ONLY')
        self.session: Node = Node({'materialization': {}})
        self.read_count: int = 0
        self.execute_count: int = 0
        self.session_reads: int = 0
        self.artifact_root: Path = Path(__file__).parent/'fixtures/renderer_qualification'

    def bridge_run(self, prepared: PreparedSetup, wait: bridge_module.CargoWaitEvidence, descriptor: int) -> BridgeResult:
        assert prepared is self.prepared and descriptor >= 0 and wait.sources == ()
        self.events.append('bridge')
        if self.failure == 'bridge':
            raise RuntimeError('synthetic bridge failure')
        self.bridge.receipt.parent.mkdir()
        _ = self.bridge.receipt.write_text('{}')
        return self.bridge

    def start(self, prepared: PreparedSetup, prior: PriorBridge, output: Path, descriptor: int) -> ImportOwner:
        assert prepared is self.prepared and prior.result is self.bridge and descriptor >= 0
        assert prior.receipt_sha256 == file_hash(self.bridge.receipt) and output == self.output/'imports'
        self.events.append('import-start')
        if self.failure == 'start':
            raise ImportFailure(RuntimeError('synthetic startup failure'), self.import_owner)
        output.mkdir()
        return self.import_owner

    def finish(self) -> ImportResult:
        self.events.append('import-finish')
        if self.failure == 'finish':
            raise ImportFailure(RuntimeError('synthetic finish failure'), self.import_owner)
        _ = self.import_result.receipt.write_text('{}')
        return self.import_result

    def read(self, prepared: PreparedSetup, prior: PriorBridge, path: Path, digest: str) -> Node:
        assert prepared is self.prepared and prior.result is self.bridge
        assert path == self.import_result.receipt and digest == file_hash(path)
        self.read_count += 1
        self.events.append('checkpoint')
        assert not (self.output/'renderer.json').exists()
        if self.failure == 'final-checkpoint' and self.read_count == 5:
            raise RuntimeError('synthetic final checkpoint failure')
        if self.read_count == 5:
            if self.failure == 'history':
                _ = (self.home/'release/receipt.json').write_text('{"changed":true}')
            if self.failure == 'receipt':
                path = self.output/'generator/process.json'
                _ = path.write_text(path.read_text()+' ')
        target: dict[str, Json] = {name: digest for name, digest in inventory(self.root, ['rust/target']).items()}
        return Node({'target': target, 'cache': {}, 'settings': {}})

    def execute(self, spec: ProcessSpec) -> ProcessResult:
        kind = spec.output_root.name
        self.execute_count += 1
        self.events.append(kind)
        assert not (self.output/'renderer.json').exists()
        spec.output_root.mkdir()
        _ = (spec.output_root/'stdout.log').write_text('RENDERER_AGREEMENT_GODOT_DONE\n' if kind == 'godot' else '')
        _ = (spec.output_root/'stderr.log').write_text('')
        if kind == 'generator':
            (spec.output_root/'commands').mkdir()
            _ = (spec.output_root/'commands/colony_s100_commands.json').write_bytes(
                (self.artifact_root/'colony_s100_commands.json').read_bytes())
        else:
            name = 'text_people.json' if kind == 'ascii' else 'godot_people.json'
            _ = (spec.output_root/name).write_bytes((self.artifact_root/name).read_bytes())
        result = process_receipt(spec, 100+self.execute_count)
        if self.failure == kind:
            row = document(result.receipt.read_text())
            row.mapping().update(status='FAIL', native_exit=7)
            _ = result.receipt.write_text(json.dumps(row.value))
            return replace(result, status='FAIL', native_exit=7)
        return result

    def session_read(self, path: Path, digest: str) -> Node:
        assert path == self.home/'session' and digest == 'a'*64
        self.session_reads += 1
        assert not (self.output/'renderer.json').exists()
        if self.session_reads == 2 and self.failure == 'session':
            raise RuntimeError('synthetic final session failure')
        if self.session_reads == 2 and self.failure == 'artifact':
            path = self.output/'ascii/text_people.json'
            _ = path.write_text(path.read_text()+' ')
        return self.session

    @contextmanager
    def owner(self, descriptor: int) -> Generator[owner.RendererOwner, None, None]:
        request = owner.RendererRequest(self.home/'session', 'a'*64, 'base', 'derived_off',
                                       self.prepared, unused_wait(self.prepared), self.output)
        # Only orchestration is under test: all synthetic admission substitutions are explicit here.
        with ExitStack() as stack:
            _ = stack.enter_context(patch.object(bridge_module, 'LOCK', self.lock))
            for name, value in (('mode_source', None),
                                ('normal_workspace_products', self.normal), ('recheck_normal_products', None),
                                ('selected_godot', (self.home/'godot', file_hash(self.home/'godot')))):
                _ = stack.enter_context(patch.object(owner, name, return_value=value))
            for name, callback in (('load_session', self.session_read), ('run_bridge_owned', self.bridge_run), ('start_import_pair', self.start),
                                   ('read_import_closure', self.read), ('execute', self.execute)):
                _ = stack.enter_context(patch.object(owner, name, side_effect=callback))
            _ = stack.enter_context(patch.object(ImportOwner, 'finish', side_effect=self.finish))
            yield owner.RendererOwner(request, descriptor)
