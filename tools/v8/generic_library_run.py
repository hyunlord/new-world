"""Closed library execution witnesses; build and observer qualification remain external."""
import fcntl
import hashlib
import os
import re
import shlex
import shutil
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from generic_gate import require
from generic_library_siblings import (
    IndependentListings,
    LibraryStep,
    ListingScope,
    SelectedAccounting,
    SelectedEvidence,
    authenticate_listings,
    prepare,
    reconcile_selected,
)
from generic_library_siblings import (
    recipe as library_recipe,
)
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
)
from generic_workspace_capture import (
    CapturedProcess,
    CapturePlan,
    captured_process,
    integer,
    read,
    snapshot,
)
from generic_workspace_cases import ListingProcess, process_output
from generic_workspace_tests import Executable, Inventory
from mux2_gate_inputs import Json
from mux2_gate_lock import LOCK


@dataclass(frozen=True, slots=True)
class Library49Request:
    root: Path
    output: Path
    invocation: str
    arm: Literal['base', 'candidate']
    executable: Executable
    cargo: Path
    bash: Path
    python: Path
    helper: Path
    host: str
    inputs: tuple[tuple[str, str], ...]
    tools: tuple[tuple[Path, str], ...]
    environment: tuple[tuple[str, str], ...]
    held_lane_fd: int


@dataclass(frozen=True, slots=True)
class Library49Result:
    receipt: Path
    status: Literal['PASS', 'FAIL']
    accounting: SelectedAccounting
    source_build_admission: Literal['PENDING_CALLER_HANDOFF'] = 'PENDING_CALLER_HANDOFF'
    observer_qualification: Literal['PENDING_ORIGINAL_DIRECT_CONTROL'] = 'PENDING_ORIGINAL_DIRECT_CONTROL'
    publication_authority: Literal[False] = False


class LibraryOwnershipUnknown(RuntimeError):
    def __init__(self, request: Library49Request, original: BaseException) -> None:
        super().__init__(f'step{request.executable.step} completion unknown; retain the actual caller lane')
        self.request: Library49Request = request
        self.original: BaseException = original


class LibrarySettledFailure(RuntimeError):
    def __init__(self, request: Library49Request, original: BaseException) -> None:
        super().__init__(f'step{request.executable.step} validation failed after authenticated process settlement')
        self.request: Library49Request = request
        self.original: BaseException = original


def checked_request(request: Library49Request) -> dict[str, str]:
    return _checked_request(request, LibraryStep.READ_DATA_ONLY)


def checked42_request(request: Library49Request) -> dict[str, str]:
    require(request.executable.filter == library_recipe(LibraryStep.BASELINE_ISOLATION).filter,
            "/library", "wrong closed filter")
    return _checked_request(request, LibraryStep.BASELINE_ISOLATION)


def checked43_request(request: Library49Request) -> dict[str, str]:
    require(request.executable.filter == library_recipe(LibraryStep.MIND_DATA_ONLY).filter,
            "/library", "wrong closed filter")
    return _checked_request(request, LibraryStep.MIND_DATA_ONLY)


def _checked_request(request: Library49Request, step: LibraryStep) -> dict[str, str]:
    held = os.fstat(request.held_lane_fd)
    canonical = LOCK.stat()
    require(stat.S_ISREG(held.st_mode) and (held.st_dev, held.st_ino) == (canonical.st_dev, canonical.st_ino),
            '/lane', 'actual caller-held gate lane required')
    fcntl.flock(request.held_lane_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    selected = request.executable; recipe = library_recipe(step)
    require(selected.step == step and selected.profile == recipe.profile and selected.target.package == recipe.package
            and selected.target.name == recipe.target_name and selected.target.kinds == ('lib',), '/library', 'wrong library')
    require(selected.target.manifest == request.root/'rust/crates'/recipe.package/'Cargo.toml'
            and selected.target.source == request.root/recipe.target_source, '/library', 'wrong source target')
    require(request.helper.name == 'generic_cargo_child.py' and re.fullmatch(r'[A-Za-z0-9_-]+', request.host) is not None,
            '/tools', 'wrong helper or host')
    bound = {str(request.root/name): digest for name, digest in request.inputs}
    require(len(bound) == len(request.inputs) and len(dict(request.tools)) == len(request.tools), '/inputs', 'duplicates')
    for path, digest in request.tools:
        require(str(path) not in bound, '/inputs', 'overlapping source/tool binding')
        bound[str(path)] = digest
    required = {request.cargo, request.bash, request.python, selected.binary, request.helper,
                selected.target.source, selected.target.manifest, *request.helper.parent.glob('*.py')}
    require(all(str(path) in bound for path in required), '/inputs', 'source/tool closure missing')
    for path, digest in bound.items():
        require(file_hash(Path(path)) == digest, path, 'input differs')
    allowed = {'PATH', 'HOME', 'TMPDIR', 'LANG', 'LC_ALL', 'CARGO_HOME', 'RUSTUP_HOME', 'RUSTC', 'RUSTDOC',
               'DYLD_LIBRARY_PATH', 'LD_LIBRARY_PATH', 'CARGO_TARGET_DIR', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'}
    require(len(dict(request.environment)) == len(request.environment)
            and set(dict(request.environment)) <= allowed, '/environment', 'unreviewed environment override')
    require(all(value == '' for key, value in request.environment
                if key in {'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'}),
            '/environment', 'only explicit empty compiler wrappers are supported')
    if step != LibraryStep.READ_DATA_ONLY:
        _wrapper_tools(request, bound, step)
    if step == LibraryStep.BASELINE_ISOLATION:
        require(all(str(path) in bound for path in (request.root/'rust/crates/sim-systems/src/runtime').rglob('*.rs')),
                '/scanner', 'unbound source scanner input')
    require(not request.output.exists() and not request.output.is_relative_to(request.root)
            and not request.root.is_relative_to(request.output), '/output', 'fresh external output required')
    return bound


def _wrapper_tools(request: Library49Request, bound: dict[str, str], step: LibraryStep) -> None:
    script = request.root/'tools/v8/core_gate_checks.sh'
    require(bound.get(str(script)) == '481d2c10e89cee95709e96c70833287ac99bd5a2b29b1fb7b01057cd4fa6e652',
            '/wrapper', 'original core gate script missing or changed')
    names = ('dirname', 'tee', 'grep', 'python3') if step == LibraryStep.BASELINE_ISOLATION else ('dirname', 'tee', 'grep')
    for name in names:
        selected = shutil.which(name, path=dict(request.environment).get('PATH', '/usr/bin:/bin:/usr/sbin:/sbin'))
        require(selected is not None, '/wrapper', 'missing original shell tool')
        assert selected is not None
        path = Path(selected).resolve(strict=True)
        require(bound.get(str(path)) == file_hash(path), '/wrapper', 'unbound original shell tool')
        require(name != 'python3' or path == request.python, '/wrapper', 'source scanner interpreter differs')


@dataclass(frozen=True, slots=True)
class _ScannerState:
    decision_exists: bool
    seam_exists: bool
    baseline_is_dir: bool
    rust_inputs: tuple[tuple[str, str], ...]


def _scanner_state(root: Path) -> _ScannerState:
    """Mirror every filesystem predicate and input used by the original scanner."""
    return _ScannerState((root/'decision').exists(), (root/'jobs/seam.rs').exists(),
                         (root/'baseline').is_dir(),
                         tuple((str(path), file_hash(path)) for path in sorted(root.rglob('*.rs'))))


def library49_children(plan: CapturePlan, parent: ProcessResult, cargo: Path) -> tuple[CapturedProcess, CapturedProcess]:
    """Legacy fixed49 join; other closed recipes use the shared implementation."""
    require(plan.step == 49, '/library49', 'wrong closed step')
    return _library_children(plan, parent, cargo)


def _library_children(plan: CapturePlan, parent: ProcessResult, cargo: Path) -> tuple[CapturedProcess, CapturedProcess]:
    step = LibraryStep(plan.step); recipe = library_recipe(step)
    require(plan.profile == recipe.profile and len(plan.inventory.executables) == 1
            and not plan.inventory.doctests, '/library49', 'wrong closed inventory')
    executable = plan.inventory.executables[0]
    selected = prepare(step, plan.parent_spec.input_root, plan.capture_root/'unused.log')
    require(executable.target.package == recipe.package and executable.target.name == recipe.target_name
            and executable.target.kinds == ('lib',) and executable.profile == recipe.profile
            and executable.step == step and executable.filter == selected.recipe.filter,
            '/library49', 'wrong selected library')
    manifest = read(plan.manifest)
    artifacts = manifest.field('artifacts').array()
    require(len(artifacts) == 2, '/library49', 'exact Cargo and libtest artifacts required')
    entries = {item.field('path').text(): item for item in artifacts}
    require(set(entries) == {str(cargo), str(executable.binary)}, '/library49', 'unexpected artifact')
    top_dir, native_dir = plan.capture_root/'000001', plan.capture_root/'000002'
    first = read(top_dir/'pending.json')
    launcher, ancestor = integer(first.field('launcher_pid'), positive=True), integer(first.field('parent_pid'), positive=True)
    require(launcher == parent.pid or ancestor == parent.pid, '/library49', 'outer Bash lineage differs')
    top = captured_process(plan, ancestor, entries[str(cargo)], cargo, selected.cargo_cwd,
                            (str(cargo), *selected.cargo_argv[1:]), top_dir)
    require(top.capture_complete and top.child_pid is not None and top.native_exit is not None
            and top.native_exit >= 0, '/library49', 'Cargo completion unresolved')
    native = captured_process(plan, top.child_pid, entries[str(executable.binary)], executable.binary,
                               executable.cwd, (str(executable.binary), selected.recipe.filter, '--nocapture' if recipe.nocapture else '--exact'), native_dir)
    require(native.capture_complete and native.native_exit is not None and native.native_exit >= 0,
            '/library49', 'libtest completion unresolved')
    claims = {'artifact-'+hashlib.sha256(str(path).encode()).hexdigest()+'.claim' for path in (cargo, executable.binary)}
    require({path.name for path in plan.capture_root.iterdir()} == {'sequence.lock', '000001', '000002', *claims}
            and (plan.capture_root/'sequence.lock').read_bytes() == b'2', '/library49', 'extra or omitted capture')
    return top, native


def run49_owned(request: Library49Request) -> Library49Result:
    return _run_owned(request, LibraryStep.READ_DATA_ONLY)


def run42_owned(request: Library49Request) -> Library49Result:
    return _run_owned(request, LibraryStep.BASELINE_ISOLATION)


def run43_owned(request: Library49Request) -> Library49Result:
    return _run_owned(request, LibraryStep.MIND_DATA_ONLY)


def _run_owned(request: Library49Request, step: LibraryStep) -> Library49Result:
    """Caller retains the actual lane on UNKNOWN for every closed original recipe."""
    bound = _checked_request(request, step)
    scanner_root = request.root/'rust/crates/sim-systems/src/runtime'
    scanner = _scanner_state(scanner_root) if step == LibraryStep.BASELINE_ISOLATION else None
    if scanner is not None:
        require(all(bound.get(path) == digest for path, digest in scanner.rust_inputs), '/scanner', 'unbound scanner input before launch')
    recipe = prepare(step, request.root, request.output/'original.log')
    require(request.executable.filter == recipe.recipe.filter, '/step49', 'wrong filter')
    request.output.mkdir()
    capture = request.output/'capture'; capture.mkdir()
    shim_dir = request.output/'shim'; shim_dir.mkdir()
    shim = shim_dir/'cargo'
    _ = shim.write_text('#!'+str(request.bash)+'\nexec '+shlex.join((str(request.python), str(request.helper), str(request.cargo)))+' "$@"\n')
    shim.chmod(0o755)
    manifest = request.output/'capture-manifest.json'
    immutable = {key: value for key, value in bound.items() if key not in {str(request.cargo), str(request.executable.binary)}}
    immutable[str(shim)] = file_hash(shim)
    artifacts: list[Json] = [{'id': 'cargo-witness', 'path': str(request.cargo), 'sha256': bound[str(request.cargo)]},
                             {'id': 'selected-libtest', 'path': str(request.executable.binary), 'sha256': bound[str(request.executable.binary)]}]
    atomic_json(manifest, {'schema_version': 1, 'invocation_id': request.invocation, 'step': int(step), 'arm': request.arm,
        'profile': recipe.recipe.profile, 'output_root': str(capture), 'launcher_sha256': bound[str(request.helper)],
        'python_sha256': bound[str(request.python)], 'inputs': {key: value for key, value in immutable.items()}, 'artifacts': artifacts, 'observer_variable': None})
    env = dict(request.environment)
    runner = 'CARGO_TARGET_'+request.host.upper().replace('-', '_')+'_RUNNER'
    env.update({'PATH': str(shim_dir)+':'+env.get('PATH', '/usr/bin:/bin:/usr/sbin:/sbin'),
                runner: shlex.join((str(request.python), str(request.helper))),
                'CARGO_CAPTURE_MANIFEST': str(manifest), 'CARGO_CAPTURE_MANIFEST_SHA256': file_hash(manifest)})
    external = (*request.tools, (shim, file_hash(shim)), (manifest, file_hash(manifest)))
    spec = ProcessSpec((str(request.bash), *recipe.outer_argv[1:]), request.root, request.bash,
                       bound[str(request.bash)], request.root, request.inputs, request.output/'outer', tuple(sorted(env.items())), external)
    in_flight = False
    try:
        in_flight = True
        result = execute(spec)
        parent = ListingProcess(spec, result, file_hash(result.receipt))
        required = {**bound, str(shim): file_hash(shim), str(manifest): file_hash(manifest)}
        raw = process_output(parent, spec.argv, request.root, environment_for(spec), required)
        plan = CapturePlan(request.invocation, int(step), request.arm, recipe.recipe.profile, spec, parent.receipt_sha256,
                           manifest, file_hash(manifest), Inventory((request.executable,), ()), capture,
                           tuple(sorted(snapshot(capture).items())))
        cargo, native = _library_children(plan, result, request.cargo)
        require(result.native_exit is not None and result.native_exit >= 0, '/outer', 'Bash completion unresolved')
        native_raw = (native.receipt.parent/'stdout.log').read_bytes()
        if step == LibraryStep.READ_DATA_ONLY:
            require(result.native_exit == cargo.native_exit, '/outer', 'Bash/Cargo waited outcome differs')
            require(raw == native_raw == (cargo.receipt.parent/'stdout.log').read_bytes(), '/stdout', 'forwarded selected output differs')
        else:
            require(native_raw == (cargo.receipt.parent/'stdout.log').read_bytes(), '/stdout', 'Cargo/native selected output differs')
            prefix = b'BASELINE_ISOLATION source PASS\n' if step == LibraryStep.BASELINE_ISOLATION else b''
            require(raw == prefix+(request.output/'original.log').read_bytes(), '/stdout', 'original source-prefix/tee log differs')
            require(result.native_exit != 0 or cargo.native_exit == 0, '/outer', 'successful wrapper lost Cargo failure')
        in_flight = False
        if scanner is not None:
            require(_scanner_state(scanner_root) == scanner, '/scanner', 'source scanner state changed')
        frozen = snapshot(request.output)
        frozen_members = {path.relative_to(request.output) for path in request.output.rglob("*")}
        listings: list[ListingProcess] = []
        native_env = read(native.receipt).field('environment')
        environment = tuple((key, native_env.field(key).text()) for key in sorted(native_env.mapping()))
        for name, suffix in (('full', ('--list',)), ('ignored', ('--list', '--ignored'))):
            listing_spec = ProcessSpec((str(request.executable.binary), *suffix), request.executable.cwd,
                request.executable.binary, bound[str(request.executable.binary)], request.root, request.inputs,
                request.output/name, environment, external)
            in_flight = True
            listing_result = execute(listing_spec)
            binding = ListingProcess(listing_spec, listing_result, file_hash(listing_result.receipt))
            _ = process_output(binding, listing_spec.argv, listing_spec.cwd, dict(environment), required)
            require(listing_result.native_exit is not None and listing_result.native_exit >= 0, '/listing', 'unsettled listing')
            in_flight = False
            if scanner is not None:
                require(_scanner_state(scanner_root) == scanner, '/scanner', 'source scanner state changed')
            listings.append(binding)
        evidence = authenticate_listings(ListingScope(request.executable.binary, bound[str(request.executable.binary)],
            request.executable.cwd, environment, tuple((Path(key), value) for key, value in required.items())),
            IndependentListings(listings[0], listings[1]))
        accounting = reconcile_selected(SelectedEvidence(step, evidence.full, evidence.ignored, native_raw, native.native_exit))
        current = snapshot(request.output)
        require({key: value for key, value in current.items() if Path(key).parts[0] not in {'full', 'ignored'}} == frozen,
                '/outputs', 'original evidence bytes or membership changed')
        current_members = {path.relative_to(request.output) for path in request.output.rglob('*')
                           if path.relative_to(request.output).parts[0] not in {'full', 'ignored'}}
        require(current_members == frozen_members, '/outputs', 'original directory membership changed')
        _ = process_output(parent, spec.argv, request.root, environment_for(spec), required)
        if scanner is not None:
            require(_scanner_state(scanner_root) == scanner, '/scanner', 'source scanner state changed')
        status = 'PASS' if result.native_exit == 0 and accounting.accounting_status == 'PASS' else 'FAIL'
        receipt = request.output/'result.json'
        atomic_json(receipt, {'schema_version': 1, 'component': f'LIBRARY{int(step)}_RUNTIME', 'status': status,
            'step': int(step), 'invocation_id': request.invocation, 'arm': request.arm, 'output_inventory': {key: value for key, value in current.items()},
            'outer_pid': result.pid, 'cargo_pid': cargo.child_pid, 'native_pid': native.child_pid,
            'native_exit': native.native_exit, 'outer_exit': result.native_exit, 'source_build_admission': 'PENDING_CALLER_HANDOFF',
            'observer_qualification': 'PENDING_ORIGINAL_DIRECT_CONTROL', 'observer_transformation': 'PATH_CARGO_SHIM_AND_HOST_RUNNER',
            'original_environment_byte_identity': False, 'recipe_scope': recipe.recipe.source_classification,
            'world_admission': 'PENDING_SOURCE_BUILD_HANDOFF', 'publication_authority': False})
        return Library49Result(receipt, status, accounting)
    except BaseException as error:
        if in_flight:
            raise LibraryOwnershipUnknown(request, error) from error
        raise LibrarySettledFailure(request, error) from error
