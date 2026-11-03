"""Owned renderer prerequisite execution; original-off H projection remains unavailable.

Call with an actual retained workspace pair prepared before step2. This alternative
uses the original non-JSON bridge recipe and never converts it to generic_build.
The owner retains live bridge/import handles after exceptions; caller keeps its lane.
"""
from dataclasses import dataclass, replace
from pathlib import Path

from generic_build import normal_workspace_products, recheck_normal_products
from generic_fixtures import ProbeProduct
from generic_gate import document, require
from generic_import_runtime_reader import read_import_closure
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_renderer_qualification_specs import independent_roots, renderer_spec
from generic_renderer_run import PINS, bind_process, inspect_godot, inspect_probe
from generic_setup_bridge import (
    BridgeResult,
    CargoWaitEvidence,
    held_lane,
    immutable_inputs,
    run_bridge_owned,
)
from generic_setup_import import (
    ImportFailure,
    ImportOwner,
    ImportResult,
    PriorBridge,
    start_import_pair,
)
from generic_setup_prepare import BRIDGE, PreparedSetup, absent
from generic_sources import inventory
from generic_whole_gate_contract import load_session
from generic_whole_gate_qualification import MODES, matrix
from generic_whole_gate_renderer_builds import selected_godot
from generic_whole_gate_renderer_compare import RendererArtifacts, ordinary
from mux2_gate_inputs import Json, Node


@dataclass(frozen=True, slots=True)
class RendererRequest:
    session: Path
    session_sha256: str
    arm: str
    mode: str
    prepared: PreparedSetup
    wait: CargoWaitEvidence
    output: Path


def mode_source(session: Node, arm_name: str, mode: str, prepared: PreparedSetup) -> None:
    """Bind exact admitted H/P/O bytes; fail before execution for unresolved original H projection."""
    require(mode in MODES and arm_name in {'base', 'candidate'}, '/selection', 'unknown arm/mode')
    require(mode != 'original_off', '/original_off',
            'UNAVAILABLE: original P lacks import hooks; authenticated H-only projection not implemented')
    arm = session.field('arms').field(arm_name)
    arm.field('source_sha').equals(prepared.inputs.source_sha)
    proof = prepared.derived_source
    require(proof is not None, '/source', 'actual derived workspace source proof required')
    assert proof is not None
    document(proof.baseline_json).field('tracked_manifest').equals(arm.field('source_before').field('tracked_manifest').value)
    document(proof.effective_json).equals(arm.field('effective_source').value)
    required = matrix(session)
    for name, raw in required.field('observer_postimages').mapping().items():
        arm.field('effective_source').field('manifest').field(name).equals(raw)
    materialized = dict(prepared.materialized)
    for name in ('tools/v8/renderer_agreement.sh', 'tools/v8/u_script.json'):
        require(materialized.get(name) == PINS[name], '/workload', 'original renderer recipe/U-script changed')
    for name, raw in arm.field('source_inputs').mapping().items():
        require(materialized.get(name) == Node(raw).text(), '/source', 'prepared source differs from admitted H/P/O')
    # Executed Python is H, including this successor after integration into a selected H.
    harness = session.field('harness')
    for name in ('generic_renderer_qualification_owner.py', 'generic_renderer_qualification_specs.py',
                 'generic_setup_bridge.py', 'generic_setup_import.py', 'generic_import_runtime_reader.py',
                 'generic_import_checkpoint_reader.py', 'gate_oracle_child.py', 'generic_renderer_run.py',
                 'generic_whole_gate_renderer_compare.py'):
        loaded = Path(__file__).resolve().parent/name
        require(file_hash(loaded) == harness.field('inputs').field('tools/v8/'+name).text(),
                '/harness', 'loaded producer/reader differs from exact admitted H')
    immutable_inputs(prepared)


@dataclass(slots=True)
class RendererOwner:
    """One mode executor. Failure never releases the caller's native lane or discards live owners."""
    request: RendererRequest
    descriptor: int
    bridge: BridgeResult | None = None
    prior: PriorBridge | None = None
    imports: ImportOwner | None = None
    imported: ImportResult | None = None
    processes: tuple[tuple[ProcessSpec, ProcessResult], ...] = ()
    process_hashes: tuple[str, ...] = ()

    def run(self) -> RendererArtifacts:
        request = self.request
        prepared = request.prepared
        held_lane(self.descriptor)
        session = load_session(request.session, request.session_sha256)
        require('materialization' in session.mapping(), '/session', 'real materialized session required')
        mode_source(session, request.arm, request.mode, prepared)
        pair = prepared.warm_pair
        require(pair is not None, '/pair', 'actual retained debug/release workspace pair required')
        assert pair is not None
        normal = normal_workspace_products(pair, 'release', (('sim-test', 'worldsim_probe'),))
        recheck_normal_products(normal)
        selected = normal.products[0]
        require(selected.binary == prepared.inputs.root/'rust/target/release/worldsim_probe',
                '/probe', 'foreign actual normal-bin selection')
        godot, godot_hash = selected_godot(session)
        require(next(tool for tool in prepared.inputs.tools if tool.name == 'godot').path == godot,
                '/godot', 'setup tool differs from session')
        output = request.output
        require(output.is_absolute() and output.parent == output.parent.resolve(strict=True),
                '/output', 'fresh canonical external output required')
        require(not output.is_relative_to(prepared.inputs.root)
                and not prepared.inputs.root.is_relative_to(output), '/output', 'runtime output/source overlap')
        absent(output)
        output.mkdir()
        # Actual immutable build evidence is frozen before the original bridge may update target.
        assert pair.debug is not None and pair.release is not None
        history = tuple((path, tuple(sorted(inventory(path, [p.name for p in path.iterdir()]).items())))
                        for path in (pair.receipt.parent, pair.debug.receipt.parent, pair.release.receipt.parent))
        self.bridge = run_bridge_owned(prepared, request.wait, self.descriptor)
        self.prior = PriorBridge(self.bridge, file_hash(self.bridge.receipt), request.wait)
        try:
            self.imports = start_import_pair(prepared, self.prior, output/'imports', self.descriptor)
        except ImportFailure as error:
            self.imports = error.owner
            raise
        self.imported = self.imports.finish()
        import_hash = file_hash(self.imported.receipt)
        checkpoint = read_import_closure(prepared, self.prior, self.imported.receipt, import_hash)
        require(file_hash(selected.binary) == selected.sha256, '/probe', 'original bridge changed selected probe')
        checkpoint.field('target').field(BRIDGE).equals(self.bridge.bridge_sha256)
        runtime_inputs = dict(prepared.materialized)
        runtime_inputs.update({name: Node(raw).text() for name, raw in checkpoint.field('target').mapping().items()})
        for name, raw in checkpoint.field('cache').mapping().items():
            digest = Node(raw).field('sha256').value
            if digest is not None:
                runtime_inputs['.godot/'+name] = Node(digest).text()
        probe = ProbeProduct(prepared.inputs.source_sha, selected.binary, selected.sha256,
                             prepared.inputs.root, tuple(sorted(runtime_inputs.items())))
        view = ProbeProduct(probe.source_sha, godot, godot_hash, probe.input_root, probe.input_files)
        commands = output/'generator/commands/colony_s100_commands.json'
        external = dict(prepared.external)
        external.update({tool.path: tool.sha256 for tool in prepared.inputs.tools})
        for path, frozen in (*history, (output/'imports', tuple(sorted(inventory(output/'imports', [p.name for p in (output/'imports').iterdir()]).items())))):
            external.update({path/name: digest for name, digest in frozen})
        original_commands: bytes | None = None
        for kind in ('generator', 'ascii', 'godot'):
            checkpoint.equals(read_import_closure(prepared, self.prior, self.imported.receipt, import_hash).value)
            spec = renderer_spec(view if kind == 'godot' else probe, kind, output/kind, commands, request.mode)
            bound = dict(external)
            bound.update(dict(spec.external_inputs))
            spec = replace(spec, external_inputs=tuple(sorted(bound.items())))
            result = execute(spec)
            self.processes += ((spec, result),)
            self.process_hashes += (file_hash(result.receipt),)
            bind_process(spec, result)
            if kind == 'generator':
                original_commands = commands.read_bytes()
            if request.mode == 'derived_on':
                if kind == 'godot':
                    _ = inspect_godot(spec, result, commands)
                else:
                    _ = inspect_probe(spec, result, 1440 if kind == 'generator' else 900)
        assert original_commands is not None
        artifact_paths = (output/'ascii/text_people.json', output/'godot/godot_people.json', commands)
        artifact_hashes = tuple(file_hash(path) for path in artifact_paths)
        artifacts = RendererArtifacts((output/'ascii/text_people.json').read_bytes(),
                                      (output/'godot/godot_people.json').read_bytes(), commands.read_bytes())
        ordinary(artifacts, original_commands)
        stdout = (output/'godot/stdout.log').read_text()
        stderr = (output/'godot/stderr.log').read_text()
        require(not any(value in stdout+stderr for value in ('SCRIPT ERROR', 'Parse Error', 'ERROR:'))
                and sum(line.startswith('RENDERER_AGREEMENT_GODOT_DONE') for line in stdout.splitlines()) == 1,
                '/godot', 'original completion oracle failed')
        checkpoint.equals(read_import_closure(prepared, self.prior, self.imported.receipt, import_hash).value)
        for path, frozen in history:
            require(inventory(path, [p.name for p in path.iterdir()]) == dict(frozen), '/history', 'build evidence changed')
        for (spec, result), digest in zip(self.processes, self.process_hashes, strict=True):
            require(file_hash(result.receipt) == digest, '/process', 'original process receipt changed')
            bind_process(spec, result)
        _ = load_session(request.session, request.session_sha256)
        require(tuple(file_hash(path) for path in artifact_paths) == artifact_hashes,
                '/artifacts', 'compared renderer bytes changed')
        row: dict[str, Json] = {'schema': 'renderer_mode_execution.v1', 'status': 'ORDINARY_RENDERER_RETURNED',
            'mode': request.mode, 'arm': request.arm, 'session': str(request.session),
            'session_sha256': request.session_sha256, 'root': str(prepared.inputs.root),
            'bridge_receipt': str(self.bridge.receipt), 'bridge_sha256': self.prior.receipt_sha256,
            'import_receipt': str(self.imported.receipt), 'import_sha256': import_hash,
            'processes': [{'path': str(result.receipt), 'sha256': file_hash(result.receipt)} for _, result in self.processes],
            'artifacts': {str(path): digest for path, digest in zip(artifact_paths, artifact_hashes, strict=True)},
            'qualification': 'NOT_RUN', 'matrix_admission': 'UNAVAILABLE', 'publication_authority': False}
        atomic_json(output/'renderer.json', row)
        return artifacts


def prepare_modes(requests: tuple[RendererRequest, ...], descriptor: int) -> tuple[RendererOwner, ...]:
    """Full path stays blocked before native work until original-off H projection exists."""
    require(tuple(request.mode for request in requests) == MODES, '/modes', 'exact ordered three-mode requests required')
    independent_roots(tuple(request.prepared.inputs.root for request in requests))
    require(len({(request.session, request.session_sha256, request.arm) for request in requests}) == 1,
            '/selection', 'three modes must use the same selected H/P/O arm')
    # Validate every mode before beginning any bridge or import process.
    for request in requests:
        session = load_session(request.session, request.session_sha256)
        require('materialization' in session.mapping(), '/session', 'real materialized session required')
        mode_source(session, request.arm, request.mode, request.prepared)
    return tuple(RendererOwner(request, descriptor) for request in requests)
