"""Original workspace4/5 wrapped execution; source/build and direct control pending."""
import fcntl
import json
import os
import shlex
import stat
from dataclasses import asdict, replace
from pathlib import Path
from typing import Literal

from generic_gate import document, require
from generic_process import (
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
)
from generic_workspace_capture import (
    CapturePlan,
    WorkspaceCapture,
    read,
    reconcile_capture,
    snapshot,
)
from generic_workspace_cases import (
    CaseLists,
    ListingProcess,
    declarations,
    join_standalone_sections,
    parse_listing,
    process_output,
    reconcile_native,
    standard_harness,
)
from generic_workspace_endpoints import (
    WorkspaceRequest,
    WorkspaceResult,
    begin_workspace,
    finish_endpoint,
    producer_finished,
    recheck_generated,
    state_json,
)
from mux2_gate_inputs import Json
from mux2_gate_lock import LOCK

__all__ = [
    'WorkspaceOwnershipUnknown',
    'WorkspaceRequest',
    'WorkspaceResult',
    'WorkspaceSettledFailure',
    'checked',
    'held_lane',
    'listing_jobs',
    'listing_members',
    'listing_spec',
    'run_workspace_owned',
    'workspace_argv',
]

class WorkspaceOwnershipUnknown(RuntimeError):
    """Caller must keep persistent ownership; no recoverable Popen API is exposed."""
    process_handle: Literal['NOT_EXPOSED_BY_GENERIC_PROCESS'] = 'NOT_EXPOSED_BY_GENERIC_PROCESS'
    persistent_caller_owner_required: Literal[True] = True

    def __init__(self, request: WorkspaceRequest, original: BaseException) -> None:
        super().__init__('workspace completion unknown; retain actual caller lane')
        self.request: WorkspaceRequest = request
        self.original: BaseException = original


class WorkspaceSettledFailure(RuntimeError):
    def __init__(self, request: WorkspaceRequest, original: BaseException) -> None:
        super().__init__('workspace validation failed after bound capture completion')
        self.request: WorkspaceRequest = request
        self.original: BaseException = original


def workspace_argv(cargo: str, step: int) -> tuple[str, ...]:
    require(step in {4, 5}, '/step', 'only original workspace4/5 supported')
    return (cargo, 'test', '--workspace', *(('--release',) if step == 5 else ()), '--no-fail-fast', '--', '--test-threads=8')


def held_lane(descriptor: int) -> None:
    require(type(descriptor) is int and descriptor >= 0, '/lane', 'caller descriptor required')
    actual, canonical = os.fstat(descriptor), LOCK.stat()
    require(stat.S_ISREG(actual.st_mode) and (actual.st_dev, actual.st_ino) == (canonical.st_dev, canonical.st_ino),
            '/lane', 'canonical caller lane required')
    fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)


def checked(request: WorkspaceRequest) -> tuple[Path, Path | None, dict[str, str]]:
    held_lane(request.held_lane_fd)
    spec = request.spec
    require(spec.argv == workspace_argv(str(spec.executable), request.step)
            and spec.cwd == spec.input_root/'rust' and spec.timeout_seconds is None,
            '/command', 'original Cargo command/cwd/no-timeout differs')
    require(request.output.is_absolute() and not request.output.exists()
            and request.output == request.output.resolve()
            and not request.output.is_relative_to(spec.input_root) and not spec.input_root.is_relative_to(request.output)
            and spec.output_root == request.output/'parent', '/output', 'fresh independent output required')
    env = environment_for(spec)
    require(all(key in dict(spec.environment) and env[key] == '' for key in ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER')),
            '/wrappers', 'explicit empty wrappers must match caller build provenance')
    allowed = {'PATH', 'HOME', 'TMPDIR', 'LANG', 'LC_ALL', 'CARGO_HOME', 'RUSTUP_HOME', 'RUSTC', 'RUSTDOC',
               'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS', 'CARGO_NET_OFFLINE', 'DYLD_LIBRARY_PATH', 'LD_LIBRARY_PATH',
               'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_CAPTURE_MANIFEST', 'CARGO_CAPTURE_MANIFEST_SHA256'}
    if request.chain is not None:
        allowed |= {'GODOT_BIN', 'WORLDSIM_TEST_SETTINGS_DIR', 'PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE',
                    '__CF_USER_TEXT_ENCODING', 'PYTHONCOERCECLOCALE'}
    runners = [key for key in env if key.startswith('CARGO_TARGET_') and key.endswith('_RUNNER')]
    require(len(runners) == 1 and set(dict(spec.environment)) <= allowed | set(runners), '/environment', 'unreviewed overrides')
    runner = shlex.split(env[runners[0]])
    require(len(runner) == 2 and Path(runner[1]).name == 'generic_cargo_child.py', '/runner', 'exact Python/helper required')
    require(env.get('CARGO_CAPTURE_MANIFEST') == str(request.manifest)
            and env.get('CARGO_CAPTURE_MANIFEST_SHA256') == request.manifest_sha256
            and file_hash(request.manifest) == request.manifest_sha256, '/manifest', 'manifest binding differs')
    bound = {str(spec.input_root/name): digest for name, digest in spec.inputs}
    require(len(bound) == len(spec.inputs) and len(dict(spec.external_inputs)) == len(spec.external_inputs), '/inputs', 'duplicate inputs')
    for path, digest in spec.external_inputs:
        require(str(path) not in bound, '/inputs', 'overlapping bindings')
        bound[str(path)] = digest
    bound[str(spec.executable)] = spec.executable_sha256
    manifest = read(request.manifest)
    for key, value in {'step': request.step, 'arm': request.arm, 'invocation_id': request.invocation,
                       'profile': 'debug' if request.step == 4 else 'release'}.items(): manifest.field(key).equals(value)
    capture = Path(manifest.field('output_root').text())
    require(capture == request.output/'capture', '/capture', 'closed capture output required')
    required = {str(request.manifest): request.manifest_sha256, runner[0]: manifest.field('python_sha256').text(),
                runner[1]: manifest.field('launcher_sha256').text()}
    required.update({key: manifest.field('inputs').field(key).text() for key in manifest.field('inputs').mapping()})
    for row in manifest.field('artifacts').array(): required[row.field('path').text()] = row.field('sha256').text()
    for executable in request.inventory.executables:
        for path in (executable.target.source, executable.target.manifest):
            require(str(path) in required, '/source', 'source/manifest absent from immutable manifest')
    exec_root: Path | None = None
    if 'doctest_policy' in manifest.mapping():
        policy = manifest.field('doctest_policy'); policy.field('schema_version').equals(2)
        exec_root = Path(policy.field('exec_root').text())
        require(exec_root == request.output/'rustdoc-execs' and env.get('RUSTDOC') == policy.field('entrypoint').text(),
                '/rustdoc', 'closed exec root/actual wrapper required')
        policy.field('parent_receipt').equals(str(spec.output_root/'process.json'))
    else:
        require(not request.inventory.doctests, '/doctests', 'declared doctests require schema2 capture policy')
    require(all(bound.get(path) == digest and file_hash(Path(path)) == digest for path, digest in required.items()),
            '/inputs', 'manifest/tool/artifact absent or changed in actual parent inputs')
    return capture, exec_root, bound


def listing_spec(request: WorkspaceRequest, label: str, argv: tuple[str, ...], cwd: Path,
                 environment: tuple[tuple[str, str], ...], receipt: Path, digest: str) -> ProcessSpec:
    require(not any('--jobserver-' in value for key, value in environment if key in {'CARGO_MAKEFLAGS', 'MAKEFLAGS', 'MFLAGS'}),
            '/listing', 'jobserver descriptor forwarding unresolved; do not strip environment')
    external = dict(request.spec.external_inputs)
    require(receipt not in external, '/listing', 'receipt input collision')
    external[receipt] = digest
    binary = Path(argv[0])
    return replace(request.spec, argv=argv, executable=binary, executable_sha256=file_hash(binary), cwd=cwd,
                   output_root=request.output/label, environment=environment, external_inputs=tuple(sorted(external.items())))


def listing_jobs(request: WorkspaceRequest, capture: WorkspaceCapture) -> tuple[tuple[str, tuple[str, ...], Path, tuple[tuple[str, str], ...], Path, str], ...]:
    manifest = read(request.manifest)
    jobs: list[tuple[str, tuple[str, ...], Path, tuple[tuple[str, str], ...], Path, str]] = []
    for index, child in enumerate(capture.targets):
        standard_harness(child.executable.target, file_hash(child.executable.target.manifest))
        jobs.append((f'native-{index}', (str(child.executable.binary),), child.executable.cwd,
                     child.original_environment, child.receipt, child.receipt_sha256))
    if 'doctest_policy' in manifest.mapping():
        policy = manifest.field('doctest_policy')
        for row in policy.field('invocations').array():
            identity = row.field('id').text(); intents = [entry for entry in capture.exec_intents if entry.source_invocation == identity]
            require(len(intents) == 1, '/doctests', 'unique actual exec intent required')
            intent = intents[0]; env = read(intent.receipt).field('environment')
            jobs.append((identity, (policy.field('rustdoc').text(), *(arg.text() for arg in row.field('argv').array())),
                Path(row.field('compiler_cwd').text()), tuple((key, env.field(key).text()) for key in sorted(env.mapping())),
                intent.receipt, intent.receipt_sha256))
    return tuple(jobs)


def listing_members(root: Path) -> dict[str, int]:
    """Bind directories as well as files; no links or special output kinds."""
    result: dict[str, int] = {}
    for path in (root, *root.rglob('*')):
        kind = stat.S_IFMT(path.lstat().st_mode)
        require(kind in {stat.S_IFDIR, stat.S_IFREG}, '/listing', 'unsupported output kind')
        result[str(path.relative_to(root))] = kind
    return result


def run_workspace_owned(request: WorkspaceRequest) -> WorkspaceResult:
    """Execute wrapped original command and real listings; caller retains UNKNOWN ownership."""
    capture_root, exec_root, bound = checked(request)
    before = begin_workspace(request) if request.chain is not None else None
    request.output.mkdir(); capture_root.mkdir()
    if exec_root is not None: exec_root.mkdir()
    in_flight = False
    try:
        if before is not None: recheck_generated(before, before.state)
        in_flight = True
        parent = execute(request.spec)
        require(parent.native_exit is not None and parent.native_exit >= 0, '/parent', 'parent wait unresolved')
        # Provisional endpoint only; incomplete child capture still yields UNKNOWN, no handoff.
        after = producer_finished(before) if before is not None else None
        plan = CapturePlan(request.invocation, request.step, request.arm, 'debug' if request.step == 4 else 'release',
            request.spec, file_hash(parent.receipt), request.manifest, request.manifest_sha256, request.inventory,
            capture_root, tuple(sorted(snapshot(capture_root).items())),
            () if exec_root is None else tuple(sorted(snapshot(exec_root).items())))
        capture = reconcile_capture(plan, parent)
        require(capture.integrity == 'PASS' and capture.completeness == 'COMPLETE', '/capture', 'actual child tree incomplete')
        in_flight = False
        if before is not None and after is not None: recheck_generated(before, after)
        frozen = snapshot(request.output)
        members = {path.relative_to(request.output) for path in request.output.rglob('*')}
        listing_bindings: list[ListingProcess] = []; listing_anchors: dict[Path, dict[str, str]] = {}
        listing_membership: dict[Path, dict[str, int]] = {}
        native_cases: list[Json] = []; doctest_cases: Json = None
        doc_pairs: list[CaseLists] = []; listing_roots: set[str] = set(); failed = capture.outcome == 'FAILED'
        jobs = listing_jobs(request, capture)
        for index, (identity, argv, cwd, env, receipt, digest) in enumerate(jobs):
            native = index < len(capture.targets); pairs: list[ListingProcess] = []; raw: list[bytes] = []
            for mode in ('all', 'ignored'):
                suffix = ('--list',) if mode == 'all' else ('--list', '--ignored')
                if not native: suffix = ('--test-args', '--list') if mode == 'all' else ('--test-args', '--list', '--test-args', '--ignored')
                label = f'listing-{index}-{mode}'; listing_roots.add(label)
                spec = listing_spec(request, label, (*argv, *suffix), cwd, env, receipt, digest)
                in_flight = True
                result = execute(spec)
                binding = ListingProcess(spec, result, file_hash(result.receipt))
                raw.append(process_output(binding, spec.argv, cwd, dict(env), {**bound, str(receipt): digest}))
                require(result.native_exit is not None and result.native_exit >= 0, '/listing', 'listing wait unresolved')
                in_flight = False
                require(result.status == 'PASS', '/listing', 'listing failed')
                pairs.append(binding); listing_bindings.append(binding)
                listing_anchors[spec.output_root] = snapshot(spec.output_root)
                listing_membership[spec.output_root] = listing_members(spec.output_root)
                if before is not None and after is not None: recheck_generated(before, after)
            if native:
                section = reconcile_native(capture.targets[index].receipt.with_name('stdout.log').read_bytes(), parse_listing(raw[0]), parse_listing(raw[1]))
                failed |= section.failed > 0
                native_cases.append(document(json.dumps(asdict(section))).value)
            else: doc_pairs.append(CaseLists(identity, pairs[0], pairs[1]))
        if exec_root is not None:
            docs = join_standalone_sections(plan, parent, tuple(doc_pairs))
            require(docs.status == 'COMPLETE_OBSERVED_AND_LISTED_DOCTEST_CASES', '/doctests', 'incomplete case accounting')
            failed |= docs.native_outcome == 'FAILED'
            doctest_cases = document(json.dumps(asdict(docs))).value
        else:
            require((request.spec.output_root/'stdout.log').read_bytes() == b''.join(child.receipt.with_name('stdout.log').read_bytes()
                for child in sorted(capture.targets, key=lambda row: row.sequence)), '/stdout', 'unconsumed workspace output')
            _ = declarations((request.spec.output_root/'stderr.log').read_bytes(), environment_for(request.spec).get('RUSTDOC', str(request.spec.executable)),
                tuple((*shlex.split(next(value for key, value in request.spec.environment if key.startswith('CARGO_TARGET_') and key.endswith('_RUNNER'))),
                       str(child.executable.binary), '--test-threads=8') for child in capture.targets))
        for binding in listing_bindings:
            required = {str(binding.spec.input_root/name): digest for name, digest in binding.spec.inputs}
            required.update({str(path): digest for path, digest in binding.spec.external_inputs})
            _ = process_output(binding, binding.spec.argv, binding.spec.cwd, environment_for(binding.spec), required)
            require(snapshot(binding.spec.output_root) == listing_anchors[binding.spec.output_root]
                    and listing_members(binding.spec.output_root) == listing_membership[binding.spec.output_root],
                    '/listing', 'listing artifact set changed')
        current = snapshot(request.output)
        require({key: value for key, value in current.items() if Path(key).parts[0] not in listing_roots} == frozen
                and {path.relative_to(request.output) for path in request.output.rglob('*')
                     if path.relative_to(request.output).parts[0] not in listing_roots} == members, '/outputs', 'prior evidence changed')
        final = reconcile_capture(plan, parent)
        require(final == capture, '/capture', 'terminal capture changed')
        held_lane(request.held_lane_fd)
        for root, anchor in listing_anchors.items():
            require(snapshot(root) == anchor and listing_members(root) == listing_membership[root],
                    '/listing', 'listing changed during final checks')
        status = 'FAIL' if failed else 'OBSERVED_CASES_ONLY'
        result_path = request.output/'result.json'
        if before is not None and after is not None: recheck_generated(before, after)
        terminal_members = listing_members(request.output) if before is not None else {}
        atomic_json(result_path, {'component': 'WORKSPACE_WRAPPED_RUNTIME', 'status': status, 'step': request.step,
            'invocation_id': request.invocation, 'arm': request.arm, 'profile': plan.profile,
            'generated_before': state_json(before.state) if before is not None else None,
            'generated_after': state_json(after) if after is not None else None,
            'predecessor_sha256': before.predecessor_sha256 if before is not None else None,
            'chronology': 'OBSERVED_ENDPOINTS_ONLY' if before is not None else 'UNQUALIFIED',
            'parent_exit': parent.native_exit, 'native_cases': native_cases, 'doctest_cases': doctest_cases, 'output_inventory': {key: value for key, value in current.items()},
            'source_build_admission': 'PENDING_CALLER_HANDOFF', 'observer_qualification': 'PENDING_ORIGINAL_DIRECT_CONTROL',
            'world_safety': 'INCOMPLETE', 'publication_authority': False, 'persistent_caller_owner_required': True})
        result = WorkspaceResult(result_path, status, capture)
        if before is not None and after is not None:
            require(snapshot(request.output) == {**current, 'result.json':file_hash(result_path)}
                    and listing_members(request.output) == {**terminal_members, 'result.json':stat.S_IFREG},
                    '/outputs', 'evidence changed during endpoint publication')
            endpoint = finish_endpoint(request, result, before, after)
            held_lane(request.held_lane_fd)
            result = replace(result, endpoint=endpoint)
        return result
    except BaseException as error:
        if in_flight: raise WorkspaceOwnershipUnknown(request, error) from error
        if before is not None and (request.output/'result.json').exists():
            try:
                atomic_json(request.output/'endpoint-failure.json', {'status':'FAIL',
                    'chronology':'ENDPOINT_NOT_RETURNED', 'publication_authority':False})
            except BaseException as diagnostic:
                raise WorkspaceSettledFailure(request, error) from diagnostic
        raise WorkspaceSettledFailure(request, error) from error
