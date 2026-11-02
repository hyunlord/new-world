"""Pure Cargo receipt joins; authenticated build handoff remains caller-owned."""
import hashlib
import os
import re
import stat
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

import gate_rustdoc_exec as rustdoc
from generic_gate import document, require
from generic_process import (
    ProcessResult,
    ProcessSpec,
    environment_for,
    file_hash,
    real_path,
)
from generic_sources import inventory
from generic_workspace_tests import Executable, Inventory
from mux2_gate_inputs import Json, Node


@dataclass(frozen=True)
class CapturePlan:
    invocation_id: str
    step: int
    arm: str
    profile: str
    parent_spec: ProcessSpec
    parent_receipt_sha256: str
    manifest: Path
    manifest_sha256: str
    inventory: Inventory
    capture_root: Path
    captured_files: tuple[tuple[str, str], ...]
    exec_files: tuple[tuple[str, str], ...] = ()


@dataclass(frozen=True)
class CapturedTarget:
    executable: Executable
    receipt: Path
    receipt_sha256: str
    sequence: int
    launcher_pid: int
    child_pid: int | None
    native_exit: int | None
    capture_complete: bool
    original_environment: tuple[tuple[str, str], ...]


@dataclass(frozen=True, slots=True)
class CapturedProcess:
    receipt: Path
    receipt_sha256: str
    sequence: int
    launcher_pid: int
    child_pid: int | None
    native_exit: int | None
    capture_complete: bool
    original_environment: tuple[tuple[str, str], ...]


@dataclass(frozen=True)
class CapturedExec:
    receipt: Path
    receipt_sha256: str
    wrapper_pid: int
    source_invocation: str | None


@dataclass(frozen=True)
class CapturedTransient:
    receipt: Path
    receipt_sha256: str
    sequence: int
    child_pid: int
    native_exit: int
    exec_intent: Path
    capture_complete: bool
    endpoint_limit: str = 'NO_ATOMIC_EXEC_OR_HOSTILE_PATH_RACE_PROOF'


@dataclass(frozen=True)
class WorkspaceCapture:
    targets: tuple[CapturedTarget, ...]
    missing: tuple[Executable, ...]
    rejected: tuple[tuple[str, str], ...]
    hashes: tuple[tuple[str, str], ...]
    parent_status: str
    parent_exit: int | None
    integrity: str
    completeness: str
    outcome: str
    declared_snapshot: tuple[tuple[str, str], ...] = ()
    source_build_admission: str = 'CALLER_AUTHENTICATED_BUILD_HANDOFF_REQUIRED'
    cargo_environment_qualification: str = 'UNRESOLVED_ACTUAL_CARGO_TRANSFORMS_AND_FDS'
    case_accounting: str = 'INCOMPLETE'
    doctests: str = 'INCOMPLETE'
    world_safety: str = 'INCOMPLETE'
    transients: tuple[CapturedTransient, ...] = ()
    exec_intents: tuple[CapturedExec, ...] = ()
    missing_exec_invocations: tuple[str, ...] = ()
    ledger: tuple[str, ...] = ()
    declared_exec_snapshot: tuple[tuple[str, str], ...] = ()


def read(path: Path) -> Node:
    return document(path.read_bytes().decode('utf-8'))


def integer(node: Node, *, positive: bool = False) -> int:
    value = node.value
    require(type(value) is int, node.label, 'integer required')
    assert isinstance(value, int)
    require(not positive or value > 0, node.label, 'positive integer required')
    return value


def strings(node: Node) -> dict[str, str]:
    return {key: node.field(key).text() for key in node.mapping()}


def snapshot(root: Path) -> dict[str, str]:
    real_path(root, directory=True)
    return inventory(root, [path.name for path in root.iterdir()])


def parent_binding(plan: CapturePlan, result: ProcessResult) -> dict[str, str]:
    spec = plan.parent_spec
    require(result.receipt == spec.output_root/'process.json' and file_hash(result.receipt) == plan.parent_receipt_sha256,
            '/parent', 'receipt path/hash differs')
    node = read(result.receipt)
    require(type(result.pid) is int and result.pid > 0 and type(result.native_exit) is int, '/parent', 'missing actual outcome/PID')
    require(spec.timeout_seconds is None, '/parent', 'original Cargo invocation has no timeout')
    argv = (str(spec.executable), 'test', '--workspace', *(('--release',) if plan.step == 5 else ()), '--no-fail-fast', '--', '--test-threads=8')
    require(spec.argv == argv and spec.cwd == spec.input_root/'rust', '/parent', 'original Cargo argv/cwd differs')
    status = 'PASS' if result.native_exit == 0 else 'FAIL'
    require(result.status == status, '/parent', 'native outcome/status differs')
    expected: dict[str, Json] = {'schema_version': 1, 'status': status, 'native_exit': result.native_exit, 'pid': result.pid,
        'argv': list(argv), 'cwd': str(spec.cwd), 'executable': str(spec.executable), 'input_root': str(spec.input_root),
        'output_root': str(spec.output_root), 'environment': dict(environment_for(spec)),
        'environment_policy': 'fixed_defaults_plus_explicit_only_no_inheritance', 'fresh_directories': list(spec.fresh_directories),
        'expected_executable_sha256': spec.executable_sha256, 'before_executable_sha256': spec.executable_sha256,
        'after_executable_sha256': spec.executable_sha256, 'errors': [] if result.native_exit == 0 else [f'native exit {result.native_exit}']}
    for key, value in expected.items(): node.field(key).equals(value)
    require('timed_out' not in node.mapping() and 'timeout_seconds' not in node.mapping(), '/parent', 'changed timeout policy')
    for key in ('started_at', 'ended_at'):
        require(datetime.fromisoformat(node.field(key).text()).tzinfo is not None, '/parent', 'timestamp lacks timezone')
    _ = integer(node.field('duration_ns'), positive=True)
    require(len(dict(spec.inputs)) == len(spec.inputs) and len(dict(spec.external_inputs)) == len(spec.external_inputs), '/parent', 'duplicate inputs')
    bound = {str(spec.executable): spec.executable_sha256}
    for suffix, rows in (('inputs', {name: digest for name, digest in spec.inputs}),
                         ('external_inputs', {str(path): digest for path, digest in spec.external_inputs})):
        for prefix in ('expected_', 'before_', 'after_'): node.field(prefix+suffix).equals({key: value for key, value in rows.items()})
        for name, digest in rows.items(): bound[str(spec.input_root/name) if suffix == 'inputs' else name] = digest
    env = environment_for(spec)
    require(env.get('CARGO_CAPTURE_MANIFEST') == str(plan.manifest) and env.get('CARGO_CAPTURE_MANIFEST_SHA256') == plan.manifest_sha256,
            '/parent', 'manifest not tied to actual outer environment')
    require(bound.get(str(plan.manifest)) == plan.manifest_sha256, '/parent', 'manifest absent from immutable parent inputs')
    for name in ('stdout', 'stderr'): node.field(name+'_sha256').equals(file_hash(spec.output_root/(name+'.log')))
    started = read(spec.output_root/'process-start.json')
    for key in ('argv', 'cwd', 'environment', 'started_at'): started.field(key).equals(node.field(key).value)
    read(spec.output_root/'process-spawn.json').field('pid').equals(result.pid)
    for path, digest in bound.items(): require(file_hash(Path(path)) == digest, path, 'bound parent input changed')
    return {**bound, **{str(spec.output_root/name): digest for name, digest in snapshot(spec.output_root).items()}}


def child_binding(plan: CapturePlan, parent: ProcessResult, entry: Node, executable: Executable, directory: Path) -> CapturedTarget:
    captured = captured_process(plan, parent.pid, entry, executable.binary, executable.cwd,
                                 (str(executable.binary), '--test-threads=8'), directory)
    return CapturedTarget(executable, captured.receipt, captured.receipt_sha256, captured.sequence,
                          captured.launcher_pid, captured.child_pid, captured.native_exit,
                          captured.capture_complete, captured.original_environment)


def captured_process(plan: CapturePlan, parent_pid: int | None, entry: Node, binary: Path,
                      cwd: Path, argv: tuple[str, ...], directory: Path) -> CapturedProcess:
    sequence = int(directory.name)
    pending = read(directory/'pending.json')
    expected: dict[str, Json] = {'schema_version': 1, 'component': 'NATIVE_CAPTURE_COMPONENT', 'invocation_id': plan.invocation_id,
        'step': plan.step, 'arm': plan.arm, 'profile': plan.profile, 'sequence': sequence, 'parent_pid': parent_pid,
        'argv': list(argv), 'cwd': str(cwd), 'manifest_sha256': plan.manifest_sha256}
    for key, value in expected.items(): pending.field(key).equals(value)
    launcher = integer(pending.field('launcher_pid'), positive=True)
    _ = integer(pending.field('started_ns'), positive=True)
    claim = plan.capture_root/('artifact-'+hashlib.sha256(str(binary).encode()).hexdigest()+'.claim')
    require(claim.read_bytes() == str(sequence).encode(), '/claim', 'claim/sequence differs')
    terminal_path = directory/'receipt.json'
    if not terminal_path.exists():
        partial_pid: int | None = None
        if (directory/'started.json').exists():
            partial = read(directory/'started.json')
            for key, value in pending.mapping().items(): partial.field(key).equals(value)
            partial_pid = integer(partial.field('child_pid'), positive=True)
            partial.field('artifact_id').equals(entry.field('id').text())
        return CapturedProcess(directory/'pending.json', file_hash(directory/'pending.json'), sequence, launcher, partial_pid, None, False, ())
    terminal = read(terminal_path)
    for key, value in pending.mapping().items(): terminal.field(key).equals(value)
    actual = snapshot(directory)
    terminal.field('output_inventory').equals({key: digest for key, digest in actual.items() if key != 'receipt.json'})
    _ = integer(terminal.field('ended_ns'), positive=True)  # wall-clock values, not monotonic ordering evidence
    terminal.field('artifact_id').equals(entry.field('id').text())
    digest = entry.field('sha256').text()
    for key in ('before_binary_sha256', 'after_binary_sha256'): terminal.field(key).equals(digest)
    require(file_hash(binary) == digest, '/binary', 'artifact changed')
    manifest = read(plan.manifest)
    inputs = strings(manifest.field('inputs'))
    for key in ('expected_inputs', 'after_inputs'): terminal.field(key).equals({key: value for key, value in inputs.items()})
    for path, checksum in inputs.items(): require(file_hash(Path(path)) == checksum, path, 'immutable child input changed')
    require({path.name for path in directory.iterdir()} <= {'pending.json', 'started.json', 'receipt.json', 'stdout.log', 'stderr.log', 'observer'}, '/child', 'unknown output file')
    original = strings(terminal.field('original_environment'))
    require(original.get('CARGO_CAPTURE_MANIFEST') == str(plan.manifest) and original.get('CARGO_CAPTURE_MANIFEST_SHA256') == plan.manifest_sha256,
            '/child/environment', 'Cargo manifest propagation differs')
    variable = manifest.field('observer_variable').value
    require(variable is None or variable == 'WORLDSIM_GATE_TEST_OUT', '/observer', 'unsupported observer')
    additions = {} if variable is None else {'WORLDSIM_GATE_TEST_OUT': str(directory/'observer')}
    require(not set(additions) & original.keys(), '/observer', 'observer overwrote original Cargo environment')
    terminal.field('observer_additions').equals({key: value for key, value in additions.items()})
    terminal.field('environment').equals({**original, **additions})
    started = read(directory/'started.json')
    for key in started.mapping(): started.field(key).equals(terminal.field(key).value)
    for key in (*pending.mapping(), 'child_pid', 'artifact_id', 'before_binary_sha256', 'environment', 'observer_additions'):
        started.field(key).equals(terminal.field(key).value)
    pid = integer(terminal.field('child_pid'), positive=True)
    code = integer(terminal.field('native_exit'))
    terminal.field('native_signal').equals(-code if code < 0 else None)
    for signal in terminal.field('received_signals').array(): _ = integer(signal, positive=True)
    for key in ('workspace_case_accounting', 'doctests', 'world_coverage'): terminal.field(key).equals('INCOMPLETE')
    errors = tuple(row.text() for row in terminal.field('errors').array())
    terminal.field('capture_status').equals('FAIL' if errors else 'COMPLETE')
    for name in ('stdout', 'stderr'): terminal.field(name+'_sha256').equals(file_hash(directory/(name+'.log')))
    return CapturedProcess(terminal_path, file_hash(terminal_path), sequence, launcher, pid, code, not errors, tuple(sorted(original.items())))



def exec_binding(plan: CapturePlan, parent: ProcessResult, manifest: Node, policy: Node, directory: Path) -> CapturedExec:
    require({p.name for p in directory.iterdir()} == {'exec-entry.json', 'exec-intent.json'}, '/exec', 'missing or failed exec lifecycle')
    path = directory/'exec-intent.json'
    intent = read(path)
    entry = read(directory/'exec-entry.json')
    for key, value in entry.mapping().items(): intent.field(key).equals(value)
    for key, value in {'schema_version': 1, 'component': 'RUSTDOC_EXEC_INTENT', 'occurrence': directory.name,
                      'manifest_sha256': plan.manifest_sha256, 'cargo_parent_pid': parent.pid,
                      'invocation_id': plan.invocation_id, 'step': plan.step, 'arm': plan.arm, 'profile': plan.profile}.items():
        intent.field(key).equals(value)
    _ = integer(intent.field('started_ns'), positive=True)
    pid = integer(intent.field('wrapper_pid'), positive=True)
    argv = tuple(row.text() for row in intent.field('rustdoc_argv').array())
    require(bool(argv) and argv[0] == policy.field('rustdoc').text(), '/exec', 'wrong rustdoc')
    intent.field('wrapper_argv').equals([str(Path(rustdoc.__file__).resolve()), *argv[1:]])
    environment = strings(intent.field('environment'))
    require(environment.get('CARGO_CAPTURE_MANIFEST') == str(plan.manifest) and
            environment.get('CARGO_CAPTURE_MANIFEST_SHA256') == plan.manifest_sha256, '/exec', 'manifest propagation differs')
    ancestry = intent.field('ancestor')
    ancestry.field('expected_terminal').equals(str(parent.receipt))
    ancestry.field('cargo_pid').equals(parent.pid)
    ancestry.field('spawn_receipt').equals(str(parent.receipt.with_name('process-spawn.json')))
    ancestry.field('spawn_sha256').equals(file_hash(parent.receipt.with_name('process-spawn.json')))
    ancestry.field('final_parent_qualification').equals('PENDING_AUTHENTICATED_PROCESS_RECEIPT_JOIN')
    source: str | None = None
    if '--test' in argv:
        source = rustdoc.match_invocation(manifest, policy, argv[1:], Path(intent.field('cwd').text())).field('id').text()
    else:
        require(argv[1:] in (('-vV',), ('--version',), ('-V',)), '/exec', 'unknown rustdoc probe')
    intent.field('source_invocation').equals(source)
    return CapturedExec(path, file_hash(path), pid, source)


def transient_binding(plan: CapturePlan, manifest: Node, policy: Node, directory: Path,
                      intents: tuple[CapturedExec, ...]) -> CapturedTransient:
    pending = read(directory/'pending.json')
    terminal = read(directory/'receipt.json')
    started = read(directory/'started.json')
    for key, value in pending.mapping().items(): terminal.field(key).equals(value)
    for key, value in started.mapping().items(): terminal.field(key).equals(value)
    for key in (*pending.mapping(), 'child_pid', 'artifact_id', 'before_binary_sha256', 'environment', 'observer_additions'):
        started.field(key).equals(terminal.field(key).value)
    sequence = int(directory.name)
    for key, value in {'schema_version': 1, 'component': 'NATIVE_CAPTURE_COMPONENT', 'sequence': sequence,
                      'invocation_id': plan.invocation_id, 'step': plan.step, 'arm': plan.arm, 'profile': plan.profile,
                      'manifest_sha256': plan.manifest_sha256, 'artifact_id': None}.items(): terminal.field(key).equals(value)
    for key in ('launcher_pid', 'parent_pid', 'started_ns', 'ended_ns'): _ = integer(terminal.field(key), positive=True)
    argv = tuple(row.text() for row in terminal.field('argv').array())
    require(len(argv) == 1, '/transient', 'changed transient argv')
    admission = read(directory/'transient-admission.json')
    evidence = terminal.field('transient')
    for key, value in admission.mapping().items(): evidence.field(key).equals(value)
    evidence.field('classification').equals('RUSTDOC_TRANSIENT')
    evidence.field('endpoint_limit').equals('NO_ATOMIC_EXEC_OR_HOSTILE_PATH_RACE_PROOF')
    evidence.field('doctest_cases').equals('INCOMPLETE')
    matches = [row for row in intents if str(row.receipt) == evidence.field('exec_intent').text()]
    require(len(matches) == 1, '/transient', 'missing unique authenticated exec intent')
    intent = matches[0]
    require(intent.source_invocation is not None, '/transient', 'probe cannot produce a transient')
    evidence.field('exec_sha256').equals(intent.receipt_sha256)
    terminal.field('parent_pid').equals(intent.wrapper_pid)
    evidence.field('live_parent_pid').equals(intent.wrapper_pid)
    source = next(row for row in policy.field('invocations').array() if row.field('id').text() == intent.source_invocation)
    terminal.field('cwd').equals(source.field('run_cwd').value)
    evidence.field('compiler_cwd').equals(source.field('compiler_cwd').value)
    evidence.field('run_cwd').equals(source.field('run_cwd').value)
    evidence.field('ancestor').equals(read(intent.receipt).field('ancestor').value)
    probe = read(directory/'live-parent.json')
    probe.field('parent_before').equals(intent.wrapper_pid); probe.field('parent_after').equals(intent.wrapper_pid)
    probe.field('error').equals(None)
    probe.field('path').equals(evidence.field('live_parent_path').value)
    require(Path(probe.field('path').text()).resolve() == Path(policy.field('rustdoc').text()), '/transient', 'wrong retained live image')
    identity = evidence.field('identity_before')
    evidence.field('identity_after').equals(identity.value)
    identity.field('lexical').equals(argv[0])
    lexical = Path(argv[0]); canonical = Path(identity.field('canonical').text())
    temporary = Path(policy.field('temporary_root').text())
    evidence.field('temporary_root').equals(str(temporary))
    require(lexical.is_absolute() and '..' not in lexical.parts and canonical.is_absolute() and '..' not in canonical.parts and
            canonical.is_relative_to(temporary) and canonical.name == 'rust_out' and canonical.parent.name.startswith('rustdoctest'),
            '/transient', 'retained endpoint outside temporary scope')
    alias = identity.field('alias').value
    require((alias is None and lexical == canonical) or (alias == '/var -> /private/var' and lexical.is_relative_to('/var') and
            canonical == Path('/private')/lexical.relative_to('/')), '/transient', 'unsupported retained alias')
    # Same-user/runtime replay restores the producer's ownership predicate.
    identity.field('uid').equals(os.getuid())
    mode = integer(identity.field('mode'))
    require(stat.S_ISREG(mode) and bool(mode & 0o111), '/transient', 'retained endpoint not executable regular file')
    for key in ('device', 'inode', 'uid', 'size'): require(integer(identity.field(key)) >= 0, '/transient', 'negative identity field')
    saved = directory/'transient-binary'
    evidence.field('saved_binary').equals(str(saved))
    digest = file_hash(saved)
    evidence.field('sha256').equals(digest)
    require(saved.stat().st_size == integer(identity.field('size')), '/transient', 'retained size differs')
    for key in ('before_binary_sha256', 'after_binary_sha256'): terminal.field(key).equals(digest)
    inputs = strings(manifest.field('inputs'))
    for key in ('expected_inputs', 'after_inputs'): terminal.field(key).equals({k: v for k, v in inputs.items()})
    original = strings(terminal.field('original_environment'))
    # The pinned Rustdoc runtool branch forwards its environment unchanged.
    terminal.field('original_environment').equals(read(intent.receipt).field('environment').value)
    require(original.get('CARGO_CAPTURE_MANIFEST') == str(plan.manifest) and
            original.get('CARGO_CAPTURE_MANIFEST_SHA256') == plan.manifest_sha256, '/transient', 'manifest environment differs')
    variable = manifest.field('observer_variable').value
    require(variable is None or variable == 'WORLDSIM_GATE_TEST_OUT', '/observer', 'unsupported observer')
    additions = {} if variable is None else {'WORLDSIM_GATE_TEST_OUT': str(directory/'observer')}
    require(not set(additions) & original.keys(), '/observer', 'observer overwrote environment')
    terminal.field('observer_additions').equals({k: v for k, v in additions.items()})
    terminal.field('environment').equals({**original, **additions})
    code = integer(terminal.field('native_exit')); pid = integer(terminal.field('child_pid'), positive=True)
    terminal.field('native_signal').equals(-code if code < 0 else None)
    for row in terminal.field('received_signals').array(): _ = integer(row, positive=True)
    for key in ('workspace_case_accounting', 'doctests', 'world_coverage'): terminal.field(key).equals('INCOMPLETE')
    errors = tuple(row.text() for row in terminal.field('errors').array())
    terminal.field('capture_status').equals('FAIL' if errors else 'COMPLETE')
    for name in ('stdout', 'stderr'): terminal.field(name+'_sha256').equals(file_hash(directory/(name+'.log')))
    terminal.field('output_inventory').equals({key: value for key, value in snapshot(directory).items() if key != 'receipt.json'})
    require({p.name for p in directory.iterdir()} <= {'pending.json', 'started.json', 'receipt.json', 'stdout.log', 'stderr.log',
            'observer', 'transient-binary', 'transient-admission.json', 'live-parent.json'}, '/transient', 'unknown child artifact')
    claim = plan.capture_root/('artifact-'+hashlib.sha256(argv[0].encode()).hexdigest()+'.claim')
    require(claim.read_bytes() == str(sequence).encode(), '/claim', 'transient claim differs')
    return CapturedTransient(directory/'receipt.json', file_hash(directory/'receipt.json'), sequence, pid, code, intent.receipt, not errors)


def reconcile_capture(plan: CapturePlan, parent_result: ProcessResult) -> WorkspaceCapture:
    rejected: list[tuple[str, str]] = []
    targets: list[CapturedTarget] = []
    hashes: dict[str, str] = {}
    seen: set[Path] = set()
    transients: list[CapturedTransient] = []
    intents: list[CapturedExec] = []
    missing_exec: tuple[str, ...] = ()
    ledger: tuple[str, ...] = ()
    policy: Node | None = None
    exec_root: Path | None = None
    parent_files: dict[str, str] | None = None
    try:
        require(plan.step in {4, 5} and plan.profile == ('debug' if plan.step == 4 else 'release') and plan.arm in {'base', 'candidate'}, '/identity', 'unsupported step/profile/arm')
        require(re.fullmatch(r'[A-Za-z0-9_.-]+', plan.invocation_id) is not None, '/identity', 'invalid invocation')
        ledger = tuple(str(path) for path in sorted(plan.capture_root.iterdir()))
        hashes = {str(plan.capture_root/name): digest for name, digest in snapshot(plan.capture_root).items()}
        require(len(dict(plan.captured_files)) == len(plan.captured_files) and snapshot(plan.capture_root) == dict(plan.captured_files), '/capture', 'frozen capture inventory differs')
        hashes.update(parent_binding(plan, parent_result))
        parent_files = snapshot(plan.parent_spec.output_root)
        require(file_hash(plan.manifest) == plan.manifest_sha256, '/manifest', 'manifest changed')
        hashes[str(plan.manifest)] = plan.manifest_sha256
        manifest = read(plan.manifest)
        require(set(manifest.mapping()) == ({'doctest_policy'} if 'doctest_policy' in manifest.mapping() else set[str]()) | {'schema_version', 'invocation_id', 'step', 'arm', 'profile', 'output_root', 'launcher_sha256', 'python_sha256', 'inputs', 'artifacts', 'observer_variable'}, '/manifest', 'schema keys differ')
        for key in ('launcher_sha256', 'python_sha256'):
            require(manifest.field(key).text() in hashes.values(), key, 'launcher/interpreter digest absent from parent bindings')
        for key, value in {'schema_version': 1, 'invocation_id': plan.invocation_id, 'step': plan.step, 'arm': plan.arm,
                           'profile': plan.profile, 'output_root': str(plan.capture_root)}.items(): manifest.field(key).equals(value)
        for row in manifest.field('artifacts').array():
            require(set(row.mapping()) == {'id', 'path', 'sha256'}, '/artifact', 'artifact keys differ')
        entries = {row.field('path').text(): row for row in manifest.field('artifacts').array()}
        expected = {str(row.binary): row for row in plan.inventory.executables}
        require(bool(expected) and len(expected) == len(plan.inventory.executables) and set(entries) == set(expected)
                and len(entries) == len(manifest.field('artifacts').array()), '/inventory', 'missing/extra/duplicate manifest artifact')
        require(len({row.field('id').text() for row in entries.values()}) == len(entries), '/inventory', 'duplicate artifact ID')
        for binary, row in expected.items():
            require((row.step, row.profile, row.filter, row.ignored, row.threads) == (plan.step, plan.profile, None, 'exclude', 8), '/inventory', 'selector/profile differs')
            require(row.target.test and row.target.manifest.is_relative_to(plan.parent_spec.cwd) and row.target.source.is_relative_to(row.cwd), '/inventory', 'foreign target source/package')
            digest = entries[binary].field('sha256').text()
            require(file_hash(row.binary) == digest, binary, 'binary differs from caller-bound manifest')
            hashes[binary] = digest
        for path, digest in strings(manifest.field('inputs')).items():
            require(file_hash(Path(path)) == digest, path, 'source input differs')
            hashes[path] = digest
        if 'doctest_policy' in manifest.mapping():
            manifest.field('doctest_policy').field('schema_version').equals(2)
            policy = rustdoc.policy_for(manifest)
            policy.field('parent_receipt').equals(str(parent_result.receipt))
            exec_root = Path(policy.field('exec_root').text())
            ledger += tuple(str(path) for path in sorted(exec_root.iterdir()))
            require(len(dict(plan.exec_files)) == len(plan.exec_files) and snapshot(exec_root) == dict(plan.exec_files), '/exec', 'frozen exec inventory differs')
            hashes.update({str(exec_root/name): digest for name, digest in plan.exec_files})
            invocation_rows = policy.field('invocations').array()
            expected_docs = {(row.package_id, str(row.manifest), str(row.source)) for row in plan.inventory.doctests}
            declared_docs = [(row.field('package_id').text(), row.field('package_manifest').text(), row.field('source').text()) for row in invocation_rows]
            require(set(declared_docs) == expected_docs and len(declared_docs) == len(expected_docs), '/exec', 'doctest target inventory differs')
            declared = [row.field('id').text() for row in invocation_rows]
            require(len(declared) == len(set(declared)), '/exec', 'duplicate source invocation')
            for directory in sorted(exec_root.iterdir()):
                try:
                    real_path(directory, directory=True)
                    intents.append(exec_binding(plan, parent_result, manifest, policy, directory))
                except (OSError, ValueError, IndexError) as error: rejected.append((str(directory), str(error)))
            observed = [row.source_invocation for row in intents if row.source_invocation is not None]
            require(len(observed) == len(set(observed)), '/exec', 'duplicate source execution')
            missing_exec = tuple(name for name in declared if name not in observed)
        directories = sorted(path for path in plan.capture_root.iterdir() if path.is_dir())
        counter = (plan.capture_root/'sequence.lock').read_bytes()
        require(counter == str(len(directories)).encode() or (not directories and counter == b''), '/sequence', 'counter differs')
        for sequence, directory in enumerate(directories, 1):
            try:
                require(directory.name == f'{sequence:06d}', '/sequence', 'nonconsecutive six-digit directory')
                pending = read(directory/'pending.json')
                binary = Path(pending.field('argv').array()[0].text())
                if str(binary) not in expected and policy is not None:
                    require(binary not in seen, '/transient', 'duplicate transient invocation')
                    seen.add(binary)
                    transient = transient_binding(plan, manifest, policy, directory, tuple(intents))
                    transients.append(transient)
                    if not transient.capture_complete: rejected.append((str(directory), 'transient capture integrity failed'))
                    continue
                require(str(binary) in expected and binary not in seen, '/artifact', 'foreign or duplicate invocation')
                seen.add(binary)
                target = child_binding(plan, parent_result, entries[str(binary)], expected[str(binary)], directory)
                targets.append(target)
                if not target.capture_complete and target.receipt.name == 'receipt.json':
                    rejected.append((str(directory), 'native capture integrity failed: ' + str(read(target.receipt).field('errors').value)))
            except (OSError, ValueError, IndexError) as error: rejected.append((str(directory), str(error)))
        claims = {('artifact-'+hashlib.sha256(str(binary).encode()).hexdigest()+'.claim') for binary in seen}
        actual_roots = {path.name for path in plan.capture_root.iterdir()}
        for name in sorted(actual_roots - {'sequence.lock', *(path.name for path in directories), *claims}):
            rejected.append((str(plan.capture_root/name), 'extra or claim-only root record'))
    except (OSError, ValueError) as error: rejected.append(('parent/manifest/capture', str(error)))
    if policy is not None and exec_root is not None:
        try:
            rustdoc.closure_for(read(plan.manifest))
            require(snapshot(exec_root) == dict(plan.exec_files), '/exec', 'late exec output mutation')
        except (OSError, ValueError) as error: rejected.append(('exec-final', str(error)))
    for path, digest in tuple(hashes.items()):
        try: require(file_hash(Path(path)) == digest, path, 'late evidence mutation')
        except (OSError, ValueError) as error: rejected.append((path, str(error)))
    if parent_files is not None:
        try: require(snapshot(plan.parent_spec.output_root) == parent_files, '/parent', 'late parent artifact set changed')
        except (OSError, ValueError) as error: rejected.append(('parent-final', str(error)))
    try: require(snapshot(plan.capture_root) == dict(plan.captured_files), '/capture', 'late output mutation')
    except (OSError, ValueError) as error: rejected.append(('capture-final', str(error)))
    missing = tuple(row for row in plan.inventory.executables if row.binary not in {target.executable.binary for target in targets})
    complete = not rejected and not missing and not missing_exec and all(row.capture_complete for row in targets) and all(row.capture_complete for row in transients)
    failed = parent_result.native_exit != 0 or any(row.native_exit not in (None, 0) for row in (*targets, *transients))
    return WorkspaceCapture(tuple(targets), missing, tuple(rejected), tuple(sorted(hashes.items())), parent_result.status,
                            parent_result.native_exit, 'FAIL' if rejected else 'PASS', 'COMPLETE' if complete else 'INCOMPLETE',
                            'FAILED' if failed else 'SUCCEEDED' if complete else 'INCOMPLETE', plan.captured_files, transients=tuple(transients), exec_intents=tuple(intents), missing_exec_invocations=missing_exec, ledger=ledger, declared_exec_snapshot=plan.exec_files)
