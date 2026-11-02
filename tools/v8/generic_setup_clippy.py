# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_setup_clippy
"""Original step6 endpoint only; shared-target chronology is not yet authenticated."""
import os
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal

from generic_gate import require
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
    """Run the unchanged command on the existing shared target, under caller ownership.

    This endpoint does not admit preceding steps4/5, a runnable build product,
    runtime/cache closure, paired gate coverage or publication. An actual normal
    zero return plus pinned source/receipt checks settles only this owned call.
    """
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
        in_flight = True
        result = execute(spec)
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
        row['status'] = 'PASS_STEP6_ENDPOINT_ONLY'
        atomic_json(receipt_path, row)
        return ClippyResult(receipt_path, 0, result.pid)
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
