# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_setup_bridge
"""Owned original step2; normal-return scope is not arbitrary descendant quiescence."""
import fcntl
import json
import os
import stat
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal

from gate_oracle_child import import_generated
from generic_build import BuildSpec, config_inventory, recheck_workspace_pair
from generic_gate import require
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
)
from generic_setup_prepare import BRIDGE, PreparedSetup, recheck_prepared_source, recipe
from generic_sources import inventory
from generic_workspace_cases import ListingProcess, process_output
from mux2_gate_inputs import Json, Node
from mux2_gate_lock import LOCK

# Reviewed source identities, not product SHA pins or caller-declared PASS rows.
WAIT_SOURCES: Final = {
    'WAIT_TOPOLOGY_AUDIT.md': 'bcd7e28de1bc18043de0897ad63318c078dbde106101e6dd64590245bc4aeeca',
    'cargo-process-builder.rs': '225dad38fa80928269fcd572d55ff21384c365b043c39c6a6090434d995263ae',
    'cargo__src__cargo__core__compiler__build_runner__mod.rs': '47b785e2c580468306e4a08ef0ae46fd43cfe7813630cecfb6263b9e576ac4f1',
    'src__cargo__core__compiler__job_queue__mod.rs': '0459e22cd6f1a7108389d26dc6f7d1bc33ed3d7fb47913929211b836fb1f475a',
    'src__cargo__core__compiler__custom_build.rs': '8537d274d4fac08675e6abe4375633e5e4b13b5e8ba038bdf43613308d9ad53c',
    'rust__compiler__rustc_codegen_ssa__src__back__link.rs': '98703ae5b736ce65a7f6318dbfdf0765d50dcf6703317d3a97c127ef78c6df30',
}
COMMITS: Final = {'cargo': '083ac5135f967fd9dc906ab057a2315861c7a80d',
                  'rustc': '01f6ddf7588f42ae2d7eb0a2f21d44e8e96674cf'}

TOOL_DIGESTS: Final = {
    'bash': 'd59c6f2312e551e9c46497329d435b6ec74fbc46e9155b84170f377127ed004e',
    'cargo': '2be8a4e73901a99470d15b793e5ee814005c3b9ebbd3815ccbf9cf0f292238e7',
    'rustc': '34a8f7034eeb844189a3c36ebb3fe36c2193fdb47b60028eb7f9392cb1f62484',
}


@dataclass(frozen=True, slots=True)
class CargoWaitEvidence:
    sources: tuple[Path, ...]
    cargo_version: ListingProcess
    rustc_version: ListingProcess


@dataclass(frozen=True, slots=True)
class BridgeResult:
    receipt: Path
    native_exit: int
    pid: int
    bridge_sha256: str
    status: Literal['PASS_STEP2_ENDPOINT_ONLY'] = 'PASS_STEP2_ENDPOINT_ONLY'
    build_product_authority: Literal[False] = False


class BridgeFailure(RuntimeError):
    def __init__(self, primary: BaseException, secondary: BaseException | None, receipt: Path | None) -> None:
        super().__init__('original step2 failed; original exception retained')
        self.primary: BaseException = primary
        self.secondary: BaseException | None = secondary
        self.receipt: Path | None = receipt


class BridgeOwnershipUnknown(BridgeFailure):
    """Caller must keep its lane; this adapter never closes/unlocks the caller FD."""


def held_lane(descriptor: int) -> None:
    require(type(descriptor) is int and descriptor >= 0, '/lane', 'caller-held lane descriptor required')
    held = os.fstat(descriptor)
    canonical = LOCK.stat()
    require(stat.S_ISREG(held.st_mode) and (held.st_dev, held.st_ino) == (canonical.st_dev, canonical.st_ino),
            '/lane', 'descriptor does not name canonical lane')
    fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)


def qualify_wait(prepared: PreparedSetup, evidence: CargoWaitEvidence) -> tuple[tuple[Path, str], ...]:
    """Authenticate prior version receipts and the reviewed normal-path source anchors."""
    sources = {path.name: path for path in evidence.sources}
    require(set(sources) == set(WAIT_SOURCES) and len(sources) == len(evidence.sources), '/wait', 'incomplete source anchors')
    bound: dict[Path, str] = {}
    for name, path in sources.items():
        require(file_hash(path) == WAIT_SOURCES[name], '/wait', 'wait source changed')
        bound[path] = WAIT_SOURCES[name]
    tools = {tool.name: tool for tool in prepared.inputs.tools}
    require(all(tools[name].sha256 == digest and file_hash(tools[name].path) == digest
                for name, digest in TOOL_DIGESTS.items()), '/wait', 'tool binaries outside reviewed normal-path profile')
    for name, _ in prepared.inputs.configs:
        config = Node(tomllib.loads(Path(name).read_text())).mapping()
        require(config.keys() <= {'build', 'net'}, '/wait', 'unreviewed Cargo config table')
        if 'build' in config:
            build = Node(config['build']).mapping()
            require(build.keys() <= {'jobs'} and ('jobs' not in build or str(build['jobs']) == dict(prepared.inputs.environment)['CARGO_BUILD_JOBS']),
                    '/wait', 'unreviewed build config')
        if 'net' in config:
            require(Node(config['net']).mapping() == {'offline': True}, '/wait', 'unreviewed net config')
    for name, binding in (('cargo', evidence.cargo_version), ('rustc', evidence.rustc_version)):
        tool = tools[name]
        require(binding.result.native_exit == 0 and binding.spec.executable == tool.path
                and binding.spec.executable_sha256 == tool.sha256, '/version', 'wrong tool/version outcome')
        raw = process_output(binding, (str(tool.path), '--version', '--verbose'), binding.spec.cwd,
                             environment_for(binding.spec), {str(tool.path): tool.sha256}).decode()
        require(raw.splitlines().count('commit-hash: '+COMMITS[name]) == 1
                and raw.splitlines().count('release: 1.93.1') == 1
                and raw.splitlines().count('host: aarch64-apple-darwin') == 1, '/version', 'unreviewed tool source identity')
        for path in binding.spec.output_root.iterdir():
            require(path.is_file() and not path.is_symlink(), '/version', 'unexpected receipt member')
            bound[path] = file_hash(path)
    return tuple(sorted(bound.items()))


def immutable_inputs(prepared: PreparedSetup) -> None:
    """Recheck initial source anchors without incorrectly requiring target to stay absent."""
    context = prepared.inputs
    recheck_prepared_source(prepared)
    require(inventory(context.root, list(dict(prepared.materialized))) == dict(prepared.materialized),
            '/inputs', 'effective inputs changed')
    for path, digest in prepared.external:
        require(file_hash(path) == digest, str(path), 'external input changed')
    tools = {tool.name: tool for tool in context.tools}
    spec = BuildSpec(context.root, context.source_sha, tools['cargo'].path, tools['cargo'].sha256,
                     tools['rustc'].path, tools['rustc'].sha256, context.cargo_home, context.output,
                     (), (), config_inputs=context.configs)
    _ = config_inventory(spec)


@dataclass(frozen=True, slots=True)
class EndpointAnchor:
    root: Path
    encoded: str

    @classmethod
    def freeze(cls, root: Path) -> 'EndpointAnchor':
        return cls(root, json.dumps(import_generated(root), sort_keys=True))

    def recheck(self, receipt: Path | None = None, expected: bytes | None = None) -> None:
        state = import_generated(self.root)
        if receipt is not None:
            require(receipt.parent == self.root and expected is not None
                    and receipt.is_file() and not receipt.is_symlink()
                    and receipt.read_bytes() == expected, '/endpoint', 'terminal receipt changed')
            require(state.pop(receipt.name, None) is not None, '/endpoint', 'terminal receipt missing')
        require(json.dumps(state, sort_keys=True) == self.encoded,
                '/endpoint', 'producer endpoint bytes, kind or membership changed')


def run_bridge_owned(prepared: PreparedSetup, wait: CargoWaitEvidence, descriptor: int) -> BridgeResult:
    """Execute exact step2 under caller ownership; nonzero/unreturned branches retain UNKNOWN.

    Supported settlement: authenticated actual normal zero return on the reviewed
    Cargo/compiler path. No nonzero-drain inference, timeout, retry or lock release.
    Runtime/cache/system closure and source admission are upstream prerequisites.
    """
    output = prepared.inputs.output
    receipt_path: Path | None = None
    in_flight = False
    result: ProcessResult | None = None
    row: dict[str, Json] = {'schema': 'setup_bridge_owned.v1', 'step': 2, 'status': 'FAIL',
        'ownership': 'NOT_STARTED', 'build_product_authority': False, 'full_gate_complete': False,
        'descendant_scope': 'REVIEWED_NORMAL_CARGO_PATH_ONLY', 'compiler_artifact_stream': 'NOT_AVAILABLE',
        'runtime_closure': 'CALLER_OWNED_UNQUALIFIED', 'target_inventory': None, 'bridge_sha256': None}
    try:
        held_lane(descriptor)
        original = recipe(prepared, 2)
        initial_generated = tuple(EndpointAnchor.freeze(path) for path in
            (prepared.inputs.root/'rust/target', prepared.inputs.root/'.godot', prepared.inputs.settings))
        wait_inputs = qualify_wait(prepared, wait)
        external = dict(prepared.external)
        for path, digest in wait_inputs:
            require(not path.is_relative_to(prepared.inputs.root), '/wait', 'evidence overlaps product')
            require(path not in external or external[path] == digest, '/wait', 'conflicting evidence hash')
            external[path] = digest
        bash = next(tool for tool in prepared.inputs.tools if tool.name == 'bash')
        spec = ProcessSpec(original.argv, original.cwd, bash.path, bash.sha256, prepared.inputs.root,
                           prepared.materialized, output/'process', prepared.inputs.environment, tuple(sorted(external.items())))
        output.mkdir(exist_ok=False)
        receipt_path = output/'receipt.json'
        row.update(source_sha=prepared.inputs.source_sha, source_digest=prepared.source_digest,
                   original_argv=list(spec.argv), environment={key: value for key, value in environment_for(spec).items()},
                   wait_inputs={str(path): digest for path, digest in wait_inputs})
        initial_output = EndpointAnchor.freeze(output)
        expected_started = (json.dumps(row, indent=2, sort_keys=True, allow_nan=False)+'\n').encode()
        atomic_json(output/'started.json', row)
        require(qualify_wait(prepared, wait) == wait_inputs, '/wait', 'wait evidence changed before launch')
        immutable_inputs(prepared)
        if prepared.warm_pair is not None:
            _ = recheck_workspace_pair(prepared.warm_pair)
        held_lane(descriptor)
        initial_output.recheck(output/'started.json', expected_started)
        for anchor in initial_generated: anchor.recheck()
        in_flight = True
        result = execute(spec)
        # Freeze the actual producer endpoint before any authentication callback.
        output_anchor = EndpointAnchor.freeze(output)
        generated_anchors = tuple(EndpointAnchor.freeze(path) for path in
            (prepared.inputs.root/'rust/target', prepared.inputs.root/'.godot', prepared.inputs.settings))
        row.update(native_exit=result.native_exit, pid=result.pid, process_status=result.status,
                   process_receipt=str(result.receipt))
        # Never equate an arbitrary nonzero Cargo return with a fully drained build.
        require(result.native_exit == 0 and result.status == 'PASS', '/wait', 'normal successful Cargo return not established')
        raw_inventory = inventory(output, ['process'])
        required = {str(prepared.inputs.root/name): digest for name, digest in prepared.materialized}
        required.update({str(path): digest for path, digest in external.items()})
        required[str(bash.path)] = bash.sha256
        binding = ListingProcess(spec, result, file_hash(result.receipt))
        _ = process_output(binding, original.argv, original.cwd, environment_for(spec), required)
        require(type(result.pid) is int and result.pid > 0, '/pid', 'missing actual parent PID')
        assert result.pid is not None
        # The live execute call returned through child.wait; retained receipts alone cannot grant this transition.
        immutable_inputs(prepared)
        require(qualify_wait(prepared, wait) == wait_inputs, '/wait', 'wait evidence changed')
        output_anchor.recheck()
        for anchor in generated_anchors: anchor.recheck()
        in_flight = False
        row['ownership'] = 'NORMAL_CARGO_RETURN'
        row['process_artifacts'] = {key: value for key, value in raw_inventory.items()}
        immutable_inputs(prepared)
        require(qualify_wait(prepared, wait) == wait_inputs, '/wait', 'wait evidence changed')
        target = inventory(prepared.inputs.root, ['rust/target'])
        bridge_hash = target.get(BRIDGE)
        require(bridge_hash is not None, '/bridge', 'original build did not leave the selected debug bridge')
        assert bridge_hash is not None
        row.update(target_inventory={key: value for key, value in target.items()}, bridge_sha256=bridge_hash, process_receipt_sha256=binding.receipt_sha256)
        require(inventory(output, ['process']) == raw_inventory, '/process', 'late process artifact mutation')
        require(inventory(prepared.inputs.root, ['rust/target']) == target, '/target', 'target changed during endpoint validation')
        _ = process_output(binding, original.argv, original.cwd, environment_for(spec), required)
        held_lane(descriptor)
        output_anchor.recheck()
        for anchor in generated_anchors: anchor.recheck()
        row['status'] = 'PASS_STEP2_ENDPOINT_ONLY'
        expected_receipt = (json.dumps(row, indent=2, sort_keys=True, allow_nan=False)+'\n').encode()
        atomic_json(receipt_path, row)
        require(qualify_wait(prepared, wait) == wait_inputs, "/wait", "wait evidence changed")
        immutable_inputs(prepared)
        held_lane(descriptor)
        output_anchor.recheck(receipt_path, expected_receipt)
        for anchor in generated_anchors: anchor.recheck()
        return BridgeResult(receipt_path, 0, result.pid, bridge_hash)
    except BaseException as primary:
        # Set ownership before any exception formatting or diagnostic I/O.
        row['ownership'] = 'UNKNOWN_KEEP_CALLER_LANE' if in_flight else 'NO_INFLIGHT_OWNED_CALL'
        row['status'] = 'FAIL'
        row['target_inventory'] = None
        row['bridge_sha256'] = None
        if result is not None:
            row.update(native_exit=result.native_exit, pid=result.pid, process_status=result.status,
                       process_receipt=str(result.receipt))
        failure = BridgeOwnershipUnknown if in_flight else BridgeFailure
        if receipt_path is not None:
            try:
                if receipt_path.exists():receipt_path = output/'failure.json'
                atomic_json(receipt_path, row)
            except BaseException as diagnostic:
                raise failure(primary, diagnostic, receipt_path) from diagnostic
        raise failure(primary, None, receipt_path) from primary
