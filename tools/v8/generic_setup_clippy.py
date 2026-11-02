# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_setup_clippy
"""Original step6 with optional retained step5 endpoint; no build-product authority."""
import os
import shutil
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Final, Literal

from gate_oracle_child import import_generated
from generic_gate import document, require
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
    real_path,
)
from generic_setup_bridge import (
    CargoWaitEvidence,
    held_lane,
    immutable_inputs,
    qualify_wait,
)
from generic_setup_prepare import COMMANDS, PreparedSetup
from generic_sources import inventory
from generic_workspace_cases import ListingProcess, process_output
from generic_workspace_endpoints import (
    EvidenceAnchor,
    GeneratedState,
    WorkspaceEndpoint,
    anchor,
    check_endpoint,
    generated,
    state_json,
)
from mux2_gate_inputs import Json

CLIPPY_SOURCES: Final = {
    'cargo-main.rs': '09d697445aca0a9c80be088052645e4bb14bd2ba2b916083ef62b1bcb28ed4c8',
    'clippy-main.rs': 'bebe85772a35a2ee5ba3d4d9f632832f255ae0ddc886223c72958a0e32a06818',
    'clippy-driver.rs': '4e1de223b9ce14a968b9c58c303faef087c9c1fc77e047b6ca23d48e12160284',
}
CLIPPY_TOOLS: Final = {
    'cargo-clippy': '64209799ae8899219f187e5ed6b5d917598c3a7a65b02391c355074b6ed00b56',
    'clippy-driver': '1825b12644084b879d4cc43c20a95f750929796ad7b30a444cc54e570cf4de28',
}


@dataclass(frozen=True, slots=True)
class ClippyWaitEvidence:
    cargo: CargoWaitEvidence
    sources: tuple[Path, ...]


@dataclass(frozen=True, slots=True)
class ClippyResult:
    receipt: Path
    native_exit: int
    pid: int
    status: Literal['PASS_STEP6_ENDPOINT_ONLY'] = 'PASS_STEP6_ENDPOINT_ONLY'
    chronology_authority: Literal[False] = False
    build_product_authority: Literal[False] = False
    endpoint: 'ClippyEndpoint | None' = None


@dataclass(frozen=True, slots=True)
class ClippyRequest:
    prepared: PreparedSetup
    wait: ClippyWaitEvidence
    output: Path
    descriptor: int
    predecessor: WorkspaceEndpoint | None = None


@dataclass(frozen=True, slots=True)
class ClippyEndpoint:
    request: ClippyRequest
    predecessor: WorkspaceEndpoint
    result: ClippyResult
    receipt_sha256: str
    before: GeneratedState
    after: GeneratedState
    history: tuple[EvidenceAnchor, ...]


def check_clippy_endpoint(endpoint: ClippyEndpoint) -> None:
    """Validate retained history, without freezing a later producer's target state."""
    request = endpoint.request
    prior = endpoint.predecessor
    require(request.predecessor is prior and prior.prepared is request.prepared
            and prior.step == 5, '/clippy/chain', 'wrong predecessor')
    check_endpoint(prior)
    immutable_inputs(request.prepared)
    for item in endpoint.history: item.recheck()
    result = endpoint.result
    require(result.receipt == request.output/'receipt.json'
            and result.status == 'PASS_STEP6_ENDPOINT_ONLY' and result.native_exit == 0
            and result.pid > 0 and not (request.output/'failure.json').exists(),
            '/clippy/endpoint', 'failed or unrelated endpoint')
    require(file_hash(result.receipt) == endpoint.receipt_sha256, '/clippy/endpoint', 'receipt changed')
    row = document(result.receipt.read_text())
    require(row.field('predecessor_sha256').value == prior.receipt_sha256
            and row.field('chronology').value == 'OBSERVED_ENDPOINTS_ONLY'
            and row.field('generated_before').value == state_json(endpoint.before)
            and row.field('generated_after').value == state_json(endpoint.after)
            and endpoint.before == prior.after
            and endpoint.before.cache == endpoint.after.cache
            and endpoint.before.settings == endpoint.after.settings
            and row.field('status').value == result.status
            and row.field('native_exit').value == result.native_exit
            and row.field('pid').value == result.pid,
            '/clippy/endpoint', 'retained endpoint mismatch')


def recheck_chain(request: ClippyRequest, expected: GeneratedState) -> None:
    prior = request.predecessor
    assert prior is not None
    immutable_inputs(request.prepared)
    check_endpoint(prior)
    require(generated(request.prepared) == expected, '/clippy/chain', 'generated state changed outside producer')


class ClippyFailure(RuntimeError):
    def __init__(self, primary: BaseException, secondary: BaseException | None, receipt: Path | None) -> None:
        super().__init__('original step6 failed; original exception retained')
        self.primary: BaseException = primary
        self.secondary: BaseException | None = secondary
        self.receipt: Path | None = receipt


class ClippyOwnershipUnknown(ClippyFailure):
    """Caller must retain its actual lane; no inferred or guessed descriptor close."""


def qualify_clippy(prepared: PreparedSetup, evidence: ClippyWaitEvidence) -> tuple[tuple[Path, str], ...]:
    """Extend the reviewed successful Cargo path with the exact Clippy entry/driver."""
    bound = dict(qualify_wait(prepared, evidence.cargo))
    sources = {path.name: path for path in evidence.sources}
    require(len(sources) == len(evidence.sources) and sources.keys() == CLIPPY_SOURCES.keys(), '/clippy', 'incomplete Clippy source anchors')
    for name, path in sources.items():
        require(file_hash(path) == CLIPPY_SOURCES[name], '/clippy', 'Clippy source changed')
        require(path not in bound or bound[path] == CLIPPY_SOURCES[name], '/clippy', 'conflicting wait source')
        bound[path] = CLIPPY_SOURCES[name]
    tools = {tool.name: tool for tool in prepared.inputs.tools}
    for name, digest in CLIPPY_TOOLS.items():
        tool = tools[name]
        require(file_hash(tool.path) == tool.sha256 == digest and os.access(tool.path, os.X_OK), '/clippy', 'unreviewed Clippy binary')
        bound[tool.path] = digest
    require(tools['cargo-clippy'].path.with_name('clippy-driver') == tools['clippy-driver'].path,
            '/clippy', 'driver is not the plugin sibling selected by pinned source')
    env = dict(prepared.inputs.environment)
    directories = [Path(name) for name in env['PATH'].split(os.pathsep)]
    home_bin = prepared.inputs.cargo_home/'bin'
    if home_bin not in directories:directories.insert(0, home_bin)
    selected = next((directory/'cargo-clippy' for directory in directories
                     if (directory/'cargo-clippy').is_file() and os.access(directory/'cargo-clippy', os.X_OK)), None)
    require(selected == tools['cargo-clippy'].path, '/clippy', 'Cargo-home/PATH plugin resolution mismatch')
    return tuple(sorted(bound.items()))


def clippy_command(prepared: PreparedSetup) -> tuple[str, ...]:
    """Check later-phase immutable context without rerunning initial target-absence rules."""
    immutable_inputs(prepared)
    tools = {tool.name: tool for tool in prepared.inputs.tools}
    env = dict(prepared.inputs.environment)
    required = {'HOME', 'PATH', 'LANG', 'LC_ALL', 'CARGO_HOME', 'CARGO_BUILD_JOBS', 'GODOT_BIN',
                'WORLDSIM_TEST_SETTINGS_DIR', 'PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE'}
    require(len(env) == len(prepared.inputs.environment) and required <= env.keys()
            and env.keys() <= required | {'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_NET_OFFLINE', '__CF_USER_TEXT_ENCODING', 'PYTHONCOERCECLOCALE'}, '/environment', 'undeclared/missing environment')
    require(all(env.get(key, '') == '' for key in ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'))
            and env.get('CARGO_NET_OFFLINE', 'true') == 'true' and env['CARGO_HOME'] == str(prepared.inputs.cargo_home), '/environment', 'unreviewed compiler/network/home')
    for name in ('bash', 'cargo', 'rustc', 'cargo-clippy', 'clippy-driver'):
        require(shutil.which(name, path=env['PATH']) == str(tools[name].path), '/tools', 'tool resolution changed')
    command = COMMANDS[2][2]
    lines = (prepared.inputs.root/'tools/v8/gate.sh').read_text().splitlines()
    require(lines.count(f"step 6 clippy bash -c '{command}'") == 1, '/recipe', 'original step6 command missing/changed')
    return (str(tools['bash'].path), '-c', command)


def run_clippy_owned(prepared: PreparedSetup, wait: ClippyWaitEvidence, output: Path, descriptor: int) -> ClippyResult:
    """Preserve the original fresh API; preceding chronology remains unqualified."""
    return run_clippy_request(ClippyRequest(prepared, wait, output, descriptor))


def run_clippy_request(request: ClippyRequest) -> ClippyResult:
    """Execute one actual Clippy producer, optionally after an actual step5 endpoint."""
    prepared, wait, output, descriptor = request.prepared, request.wait, request.output, request.descriptor
    prior = request.predecessor
    before: GeneratedState | None = None
    after: GeneratedState | None = None
    process_anchor: EvidenceAnchor | None = None
    output_state: dict[str, Json] | None = None
    receipt_path: Path | None = None
    in_flight = False
    result: ProcessResult | None = None
    row: dict[str, Json] = {'schema': 'setup_clippy_owned.v1', 'step': 6, 'status': 'FAIL',
        'ownership': 'NOT_STARTED', 'chronology': 'AFTER4_AFTER5_PRODUCER_AUTHENTICATION_REQUIRED',
        'chronology_authority': False, 'build_product_authority': False, 'full_gate_complete': False,
        'descendant_scope': 'REVIEWED_NORMAL_CLIPPY_CARGO_PATH_ONLY',
        'runtime_closure': 'CALLER_OWNED_UNQUALIFIED', 'target_before': None, 'target_after': None}
    try:
        held_lane(descriptor)
        argv = clippy_command(prepared)
        wait_inputs = qualify_clippy(prepared, wait)
        require(output.is_absolute() and not output.exists() and not output.is_symlink(), '/output', 'fresh absolute output required')
        _ = real_path(output.parent, directory=True)
        require(output == output.resolve() and not output.is_relative_to(prepared.inputs.root)
                and not prepared.inputs.root.is_relative_to(output)
                and not output.is_relative_to(prepared.inputs.settings)
                and not prepared.inputs.settings.is_relative_to(output)
                and not output.is_relative_to(prepared.inputs.cargo_home)
                and not prepared.inputs.cargo_home.is_relative_to(output), '/output', 'output overlaps product/settings/cache')
        if prior is not None:
            require(prior.prepared is prepared and prior.step == 5, '/clippy/chain', 'step5 of same setup required')
            check_endpoint(prior)
            for item in prior.history:
                require(not output.is_relative_to(item.root) and not item.root.is_relative_to(output),
                        '/output', 'output overlaps historical evidence')
            before = generated(prepared)
            require(before == prior.after, '/clippy/chain', 'state drift since step5')
            row.update(chronology='OBSERVED_ENDPOINTS_ONLY', predecessor_sha256=prior.receipt_sha256,
                       generated_before=state_json(before), generated_after=None)
        target_before = inventory(prepared.inputs.root, ['rust/target'])
        require(bool(target_before), '/target', 'existing target required; prior producer admission remains separate')
        external = dict(prepared.external)
        for path, digest in wait_inputs:
            require(not path.is_relative_to(prepared.inputs.root), '/wait', 'wait evidence overlaps product')
            require(path not in external or external[path] == digest, '/wait', 'conflicting evidence hash')
            external[path] = digest
        bash = next(tool for tool in prepared.inputs.tools if tool.name == 'bash')
        spec = ProcessSpec(argv, prepared.inputs.root, bash.path, bash.sha256, prepared.inputs.root,
                           prepared.materialized, output/'process', prepared.inputs.environment, tuple(sorted(external.items())))
        output.mkdir(exist_ok=False)
        receipt_path = output/'receipt.json'
        row.update(source_sha=prepared.inputs.source_sha, source_digest=prepared.source_digest,
                   original_argv=list(argv), environment={key: value for key, value in environment_for(spec).items()},
                   wait_inputs={str(path): digest for path, digest in wait_inputs}, target_before={key: value for key, value in target_before.items()})
        atomic_json(output/'started.json', row)
        require(inventory(prepared.inputs.root, ['rust/target']) == target_before, '/target', 'target changed before launch')
        if before is not None: recheck_chain(request, before)
        in_flight = True
        result = execute(spec)
        # Freeze immediately on producer return, before any authentication callbacks.
        if before is not None:
            after = generated(prepared)
            process_anchor = anchor(output/'process')
            output_state = import_generated(output)
            require(after.cache == before.cache and after.settings == before.settings,
                    '/clippy/chain', 'Clippy changed cache/settings')
        row.update(native_exit=result.native_exit, pid=result.pid, process_status=result.status, process_receipt=str(result.receipt))
        require(result.native_exit == 0 and result.status == 'PASS', '/wait', 'successful normal Clippy return not established')
        process_inventory = inventory(output, ['process'])
        binding = ListingProcess(spec, result, file_hash(result.receipt))
        required = {str(prepared.inputs.root/name): digest for name, digest in prepared.materialized}
        required.update({str(path): digest for path, digest in external.items()})
        required[str(bash.path)] = bash.sha256
        _ = process_output(binding, argv, spec.cwd, environment_for(spec), required)
        require(type(result.pid) is int and result.pid > 0, '/pid', 'missing actual producer PID')
        assert result.pid is not None
        require(clippy_command(prepared) == argv and qualify_clippy(prepared, wait) == wait_inputs, '/wait', 'normal-path source anchors changed')
        # Only this live execute return and authenticated normal-source path can clear ownership.
        if after is not None:
            recheck_chain(request, after)
            assert process_anchor is not None
            process_anchor.recheck()
        in_flight = False
        row['ownership'] = 'NORMAL_CLIPPY_RETURN'
        target_after = inventory(prepared.inputs.root, ['rust/target'])
        row.update(target_after={key: value for key, value in target_after.items()},
                   process_artifacts={key: value for key, value in process_inventory.items()}, process_receipt_sha256=binding.receipt_sha256)
        require(inventory(output, ['process']) == process_inventory, '/process', 'late output mutation')
        require(inventory(prepared.inputs.root, ['rust/target']) == target_after, '/target', 'late target mutation')
        require(clippy_command(prepared) == argv and qualify_clippy(prepared, wait) == wait_inputs, '/wait', 'late source mutation')
        _ = process_output(binding, argv, spec.cwd, environment_for(spec), required)
        held_lane(descriptor)
        if after is not None:
            recheck_chain(request, after)
            assert process_anchor is not None
            process_anchor.recheck()
            require(import_generated(output) == output_state, '/clippy/output', 'output changed during authentication')
            row['generated_after'] = state_json(after)
        row['status'] = 'PASS_STEP6_ENDPOINT_ONLY'
        atomic_json(receipt_path, row)
        if prior is not None:
            published = import_generated(output)
            require({key: value for key, value in published.items() if key != 'receipt.json'} == output_state,
                    '/clippy/output', 'output changed during publication')
            require(document(receipt_path.read_bytes().decode('utf-8')).mapping() == row,
                    '/clippy/endpoint', 'published receipt differs from actual result')
        returned = ClippyResult(receipt_path, 0, result.pid)
        if prior is not None:
            assert before is not None and after is not None and process_anchor is not None
            recheck_chain(request, after)
            process_anchor.recheck()
            endpoint = ClippyEndpoint(request, prior, returned, file_hash(receipt_path), before, after,
                                      (*prior.history, anchor(output)))
            check_clippy_endpoint(endpoint)
            recheck_chain(request, after)
            for item in endpoint.history: item.recheck()
            returned = replace(returned, endpoint=endpoint)
        return returned
    except BaseException as primary:
        row['ownership'] = 'UNKNOWN_KEEP_CALLER_LANE' if in_flight else 'NO_INFLIGHT_OWNED_CALL'
        row['status'] = 'FAIL'
        row['target_after'] = None
        if result is not None:
            row.update(native_exit=result.native_exit, pid=result.pid, process_status=result.status, process_receipt=str(result.receipt))
        failure = ClippyOwnershipUnknown if in_flight else ClippyFailure
        if receipt_path is not None:
            try:
                if receipt_path.exists():receipt_path = output/'failure.json'
                atomic_json(receipt_path, row)
            except BaseException as diagnostic:
                raise failure(primary, diagnostic, receipt_path) from diagnostic
        raise failure(primary, None, receipt_path) from primary
