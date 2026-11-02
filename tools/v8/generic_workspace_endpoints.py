"""Same-process setup handoff; endpoints never confer build-product authority."""
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from gate_oracle_child import import_generated, import_state
from generic_gate import document, require
from generic_process import ProcessSpec, file_hash
from generic_setup_bridge import immutable_inputs
from generic_setup_import import ImportOwner, ImportResult, authenticate_bridge
from generic_setup_prepare import PreparedSetup
from generic_sources import inventory
from generic_workspace_capture import WorkspaceCapture
from generic_workspace_tests import Inventory
from mux2_gate_inputs import Json


@dataclass(frozen=True, slots=True)
class WorkspaceRequest:
    step: Literal[4, 5]
    arm: Literal['base', 'candidate']
    invocation: str
    spec: ProcessSpec
    inventory: Inventory
    manifest: Path
    manifest_sha256: str
    output: Path
    held_lane_fd: int
    chain: "WorkspaceChain | None" = None


@dataclass(frozen=True, slots=True)
class WorkspaceResult:
    receipt: Path
    status: Literal['OBSERVED_CASES_ONLY', 'FAIL']
    capture: WorkspaceCapture
    source_build_admission: Literal['PENDING_CALLER_HANDOFF'] = 'PENDING_CALLER_HANDOFF'
    observer_qualification: Literal['PENDING_ORIGINAL_DIRECT_CONTROL'] = 'PENDING_ORIGINAL_DIRECT_CONTROL'
    publication_authority: Literal[False] = False
    endpoint: "WorkspaceEndpoint | None" = None



def encoded(value: dict[str, Json]) -> str:
    return json.dumps(value, sort_keys=True, separators=(',', ':'))


@dataclass(frozen=True, slots=True)
class GeneratedState:
    target: str
    cache: str
    settings: str


def generated(prepared: PreparedSetup) -> GeneratedState:
    root = prepared.inputs.root
    return GeneratedState(encoded(import_generated(root/'rust/target')),
                          encoded(import_generated(root/'.godot')),
                          encoded(import_generated(prepared.inputs.settings)))


@dataclass(frozen=True, slots=True)
class EvidenceAnchor:
    root: Path
    state: str

    def recheck(self) -> None:
        require(encoded(import_generated(self.root)) == self.state, '/history', 'historical evidence changed')


def anchor(root: Path) -> EvidenceAnchor:
    require(root.is_dir() and root == root.resolve(), '/history', 'canonical evidence directory required')
    return EvidenceAnchor(root, encoded(import_generated(root)))


@dataclass(frozen=True, slots=True)
class ImportPredecessor:
    owner: ImportOwner
    result: ImportResult
    receipt_sha256: str


@dataclass(frozen=True, slots=True)
class WorkspaceEndpoint:
    request: 'WorkspaceRequest'
    result: 'WorkspaceResult'
    prepared: PreparedSetup
    step: int
    arm: str
    invocation: str
    receipt: Path
    receipt_sha256: str
    before: GeneratedState
    after: GeneratedState
    history: tuple[EvidenceAnchor, ...]


@dataclass(frozen=True, slots=True)
class WorkspaceChain:
    prepared: PreparedSetup
    predecessor: ImportPredecessor | WorkspaceEndpoint


@dataclass(frozen=True, slots=True)
class BeforeEndpoint:
    chain: WorkspaceChain
    state: GeneratedState
    predecessor_sha256: str
    history: tuple[EvidenceAnchor, ...]


def admit_import(prepared: PreparedSetup, prior: ImportPredecessor) -> tuple[EvidenceAnchor, ...]:
    owner = prior.owner; control = owner.controller; result = prior.result
    require(owner.prepared == prepared and control.closed and control.started,
            '/prior', 'actual completed import owner required')
    require(result.receipt == owner.output/'receipt.json' and not (owner.output/'failure.json').exists(),
            '/prior', 'import failed or receipt path differs')
    require(file_hash(result.receipt) == prior.receipt_sha256, '/prior', 'import receipt changed')
    settled = control.snapshot()
    require(settled.settlement == 'SETTLED' and settled.completed == ('import.first', 'import.second')
            and settled.shell_exit == result.shell_exit, '/prior', 'import actual wait differs')
    row = document(result.receipt.read_text())
    for key, value in {'schema':'setup_import_pair.v1', 'step':3, 'status':result.status,
                       'shell_pid':control.shell.pid, 'shell_exit':result.shell_exit,
                       'first_native_exit':result.first_exit, 'second_native_exit':result.second_exit,
                       'plan_sha256':control.plan.digest, 'source_sha':prepared.inputs.source_sha,
                       'prior_bridge_sha256':owner.prior.receipt_sha256,
                       'ownership':'SETTLED_ORIGINAL_IMMEDIATE_CHILDREN', 'build_product_authority':False}.items():
        row.field(key).equals(value)
    require(result.status == ('PASS_STEP3_ONLY' if result.shell_exit == 0 else 'FAIL_STEP3'),
            '/prior', 'import outcome differs')
    for occurrence, code in zip(control.plan.rows, (result.first_exit, result.second_exit), strict=True):
        document((occurrence.evidence/'native-terminal.json').read_text()).field('native_exit').equals(code)
    row.field('artifacts').equals({k:v for k,v in inventory(owner.output, ['children', 'logs', 'plan.json']).items()})
    checkpoint = document((control.plan.rows[-1].evidence/'import-checkpoint.json').read_text()).mapping()
    require(control.import_checkpoint == checkpoint and import_state(control.plan) == checkpoint,
            '/prior', 'import endpoint changed')
    row.field('cache_terminal').equals(checkpoint['cache']); row.field('settings_terminal').equals(checkpoint['settings'])
    _ = authenticate_bridge(prepared, owner.prior)
    return (anchor(prepared.inputs.output), anchor(owner.output))


def check_endpoint(endpoint: WorkspaceEndpoint) -> None:
    """Historical files only: never revalidate an obsolete target against today's tree."""
    require(endpoint.request.chain is not None and endpoint.request.chain.prepared == endpoint.prepared
            and endpoint.request.step == endpoint.step and endpoint.request.arm == endpoint.arm
            and endpoint.request.invocation == endpoint.invocation
            and endpoint.result.receipt == endpoint.receipt
            and endpoint.result.capture.integrity == 'PASS'
            and endpoint.result.capture.completeness == 'COMPLETE', '/prior', 'retained request/result differs')
    for item in endpoint.history: item.recheck()
    require(not (endpoint.receipt.parent/'endpoint-failure.json').exists(), '/prior', 'failed endpoint')
    require(file_hash(endpoint.receipt) == endpoint.receipt_sha256, '/prior', 'workspace receipt changed')
    prior = endpoint.request.chain
    assert prior is not None
    if isinstance(prior.predecessor, WorkspaceEndpoint): check_endpoint(prior.predecessor)
    # Only past generated inputs may legitimately differ after an admitted later producer.
    # All other historical request inputs remain immutable, including capture manifests.
    protected = {endpoint.request.spec.input_root/name: digest for name,digest in endpoint.request.spec.inputs}
    protected.update(dict(endpoint.request.spec.external_inputs))
    scopes = (endpoint.prepared.inputs.root/'rust/target', endpoint.prepared.inputs.root/'.godot', endpoint.prepared.inputs.settings)
    for path,digest in protected.items():
        if not any(path.is_relative_to(scope) for scope in scopes):
            require(file_hash(path) == digest, '/history', 'historical immutable input changed')
    row = document(endpoint.receipt.read_text())
    row.field('predecessor_sha256').equals(prior.predecessor.receipt_sha256)
    row.field('chronology').equals('OBSERVED_ENDPOINTS_ONLY')
    for key, value in {'component':'WORKSPACE_WRAPPED_RUNTIME', 'step':endpoint.step, 'arm':endpoint.arm,
                       'invocation_id':endpoint.invocation, 'profile':'debug' if endpoint.step == 4 else 'release',
                       'publication_authority':False, 'status':endpoint.result.status}.items(): row.field(key).equals(value)
    require(row.field('status').value in {'OBSERVED_CASES_ONLY', 'FAIL'}, '/prior', 'invalid workspace outcome')
    row.field('generated_after').equals(state_json(endpoint.after))
    row.field('generated_before').equals(state_json(endpoint.before))


def begin_workspace(request: 'WorkspaceRequest') -> BeforeEndpoint:
    chain = request.chain
    require(chain is not None, '/chain', 'missing chain')
    assert chain is not None
    prepared = chain.prepared; prior = chain.predecessor
    immutable_inputs(prepared)
    require(request.spec.input_root == prepared.inputs.root, '/chain', 'foreign product root')
    prior_roots = tuple(item.root for item in prior.history) if isinstance(prior, WorkspaceEndpoint) else (prior.owner.output, prepared.inputs.output)
    for root in (prepared.inputs.settings, prepared.inputs.cargo_home, *prior_roots):
        require(not request.output.is_relative_to(root) and not root.is_relative_to(request.output),
                '/output', 'workspace output overlaps chain state')
    env = dict(request.spec.environment); original = dict(prepared.inputs.environment)
    require(original.get('RUSTC_WRAPPER') == original.get('RUSTC_WORKSPACE_WRAPPER') == ''
            and original.get('CARGO_BUILD_JOBS') == '8', '/environment', 'explicit original empty wrappers/jobs8 required')
    require(all(env.get(key) == value for key, value in original.items()), '/environment', 'setup environment changed')
    require(env.get('CARGO_TARGET_DIR', str(prepared.inputs.root/'rust/target')) == str(prepared.inputs.root/'rust/target'),
            '/target', 'redirected shared target')
    rustc = next(tool for tool in prepared.inputs.tools if tool.name == 'rustc')
    require(env.get('RUSTC', str(rustc.path)) == str(rustc.path), '/tool', 'workspace rustc differs from setup')
    cargo = next(tool for tool in prepared.inputs.tools if tool.name == 'cargo')
    require(request.spec.executable == cargo.path and request.spec.executable_sha256 == cargo.sha256,
            '/tool', 'workspace Cargo differs from setup')
    bound = {str(request.spec.input_root/name): digest for name, digest in request.spec.inputs}
    bound.update({str(path): digest for path, digest in request.spec.external_inputs})
    expected = {str(prepared.inputs.root/name): digest for name, digest in prepared.materialized}
    expected.update({str(path): digest for path, digest in prepared.external})
    require(all(bound.get(path) == digest for path, digest in expected.items()), '/inputs', 'setup bindings missing')
    if isinstance(prior, ImportPredecessor):
        require(request.step == 4, '/chain', 'imports must precede debug workspace')
        history = admit_import(prepared, prior); digest = prior.receipt_sha256
    else:
        require(request.step == 5 and prior.step == 4 and prior.arm == request.arm
                and prior.prepared == prepared, '/chain', 'wrong workspace predecessor')
        check_endpoint(prior)
        require(generated(prepared) == prior.after, '/chain', 'generated state changed between producers')
        history = prior.history; digest = prior.receipt_sha256
    state = generated(prepared)
    for item in history: item.recheck()
    return BeforeEndpoint(chain, state, digest, history)


def producer_finished(before: BeforeEndpoint) -> GeneratedState:
    immutable_inputs(before.chain.prepared)
    for item in before.history: item.recheck()
    return generated(before.chain.prepared)


def recheck_generated(before: BeforeEndpoint, after: GeneratedState) -> None:
    immutable_inputs(before.chain.prepared)
    for item in before.history: item.recheck()
    require(generated(before.chain.prepared) == after, '/generated', 'generated state changed outside producer')


def state_json(state: GeneratedState) -> dict[str, Json]:
    return {name: document(value).value for name, value in
            (('target', state.target), ('cache', state.cache), ('settings', state.settings))}


def finish_endpoint(request: 'WorkspaceRequest', result: 'WorkspaceResult', before: BeforeEndpoint,
                    after: GeneratedState) -> WorkspaceEndpoint:
    recheck_generated(before, after)
    require(result.receipt == request.output/'result.json' and result.capture.integrity == 'PASS'
            and result.capture.completeness == 'COMPLETE', '/endpoint', 'actual complete result required')
    endpoint = WorkspaceEndpoint(request, result, before.chain.prepared, request.step, request.arm, request.invocation,
        result.receipt, file_hash(result.receipt), before.state, after, (*before.history, anchor(request.output)))
    check_endpoint(endpoint); recheck_generated(before, after)
    return endpoint
