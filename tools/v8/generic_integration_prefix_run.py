"""Executable original28/36 native prefixes, never full compound-step authority."""
import fcntl
import json
import os
import re
import shlex
import shutil
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from generic_gate import require
from generic_integration_accounting import CASES, PrefixAccounting, account36
from generic_library_run import (
    Library49Request,
    LibraryOwnershipUnknown,
    LibrarySettledFailure,
)
from generic_library_siblings import (
    IndependentListings,
    ListingScope,
    authenticate_listings,
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
from generic_workspace_cases import (
    ListingProcess,
    digest,
    parse_listing,
    process_output,
    reconcile_native,
)
from generic_workspace_tests import Inventory
from mux2_gate_inputs import Json
from mux2_gate_lock import LOCK

GATE28 = 'step 28 knowledge bash -c \'set -euo pipefail; cd rust && cargo test --release -p sim-test --test colony_knowledge -- --test-threads=8 && cd .. && rust/target/release/worldsim_probe --scene colony --seeds 0..8 --days 4 --workers 4 --commands tools/v8/m4_f10_script.json --out "$0/knowledge_f10" > /dev/null && python3 docs/v8/m4/m4_score.py --f10 "$0/knowledge_f10/summary.json" | tee /dev/stderr | grep -q "^F10: 8/8 PASS"\' "$LOG"'
SCRIPT36_SHA = '0f27e5d899e359a1a891682d1afbd3ce2090ebb8251ed65556bf5a59c3e2234e'
GODOT_ENV_SHA = '04b137eff9db572813d48f0f1c50f383f61581a1b701b1f8a1f6082b64ecc500'
TOPOLOGY36_SHA = 'b5dc52c185bff0aae8ddbce84089f2510c077ff4441c31d880345c37785b8ea2'


@dataclass(frozen=True, slots=True)
class NativePrefixResult:
    receipt: Path
    status: Literal['PASS_PREFIX_ONLY', 'FAIL']
    accounting: PrefixAccounting
    compound_tail: Literal['NOT_RUN'] = 'NOT_RUN'
    source_build_admission: Literal['PENDING_CALLER_HANDOFF'] = 'PENDING_CALLER_HANDOFF'
    ordinary_safety: Literal['PENDING_SEPARATE_COMPONENT'] = 'PENDING_SEPARATE_COMPONENT'
    publication_authority: Literal[False] = False


def recipe(step: int) -> tuple[str, str, int, tuple[str, ...]]:
    require(type(step) is int and step in {28, 36}, '/step', 'only two reviewed integration prefixes')
    target, profile, threads = ('colony_knowledge', 'release', 8) if step == 28 else ('data_only_ext', 'debug', 1)
    argv = ('cargo', 'test', *(('--release',) if step == 28 else ()), '-p', 'sim-test', '--test', target, '--', f'--test-threads={threads}')
    return target, profile, threads, argv


def checked_request(request: Library49Request, step: int) -> dict[str, str]:
    held, canonical = os.fstat(request.held_lane_fd), LOCK.stat()
    require(stat.S_ISREG(held.st_mode) and (held.st_dev, held.st_ino) == (canonical.st_dev, canonical.st_ino), '/lane', 'actual held lane required')
    fcntl.flock(request.held_lane_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    name, profile, threads, _argv = recipe(step)
    item = request.executable
    require(item.step == step and item.profile == profile and item.threads == threads and item.filter is None
            and item.ignored == 'exclude' and item.target.package == 'sim-test' and item.target.name == name
            and item.target.kinds == ('test',) and item.target.test and not item.target.doctest,
            '/target', 'wrong unfiltered integration target')
    require(item.target.source == request.root/f'rust/crates/sim-test/tests/{name}.rs'
            and item.target.manifest == request.root/'rust/crates/sim-test/Cargo.toml', '/target', 'source paths differ')
    bound = {str(request.root/name): value for name, value in request.inputs}
    require(len(bound) == len(request.inputs) and len(dict(request.tools)) == len(request.tools), '/inputs', 'duplicates')
    for path, value in request.tools:
        require(str(path) not in bound, '/inputs', 'source/tool overlap')
        bound[str(path)] = value
    require(request.helper.name == 'generic_cargo_child.py' and re.fullmatch(r'[A-Za-z0-9_-]+', request.host) is not None, '/tools', 'wrong helper/host')
    required = {request.cargo, request.bash, request.python, request.helper, item.binary, item.target.source,
                item.target.manifest, request.root/'tools/v8/gate.sh', *request.helper.parent.glob('*.py')}
    if step == 36:
        required.update((request.root/'tools/v8/data_only_ext.sh', request.root/'tools/v8/godot_bin.sh'))
    require(all(str(path) in bound for path in required), '/inputs', 'missing source/tool closure')
    for path, value in bound.items():
        require(file_hash(Path(path)) == value, path, 'input changed')
    gate = (request.root/'tools/v8/gate.sh').read_text().splitlines()
    require([line for line in gate if line.startswith('step 28 knowledge ')] == [GATE28], '/source', 'original28 changed')
    require([line for line in gate if line.startswith('step 36 data_only_ext ')] == ['step 36 data_only_ext bash tools/v8/data_only_ext.sh "$LOG/data_only_ext"'], '/source', 'original36 changed')
    if step == 36:
        require(bound[str(request.root/'tools/v8/data_only_ext.sh')] == SCRIPT36_SHA
                and bound[str(request.root/'tools/v8/godot_bin.sh')] == GODOT_ENV_SHA, '/source', 'original36 wrapper changed')
        source = item.target.source.read_text()
        start = source.index('#[test]\nfn selected_pack_runs_')
        end = source.index('\n#[test]\nfn selected_pack_child', start)
        require(digest(source[start:end].encode()) == TOPOLOGY36_SHA
                and source.count('const CHILD: &str = "WORLDSIM_DATA_ONLY_CASE";') == 1
                and source.count('std::process::Command::') == 1, '/source', 'reviewed synchronous topology changed')
    env = dict(request.environment)
    allowed = {'PATH', 'HOME', 'TMPDIR', 'LANG', 'LC_ALL', 'CARGO_HOME', 'RUSTUP_HOME', 'RUSTC', 'RUSTDOC',
               'DYLD_LIBRARY_PATH', 'LD_LIBRARY_PATH', 'CARGO_TARGET_DIR', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'}
    require(len(env) == len(request.environment) and set(env) <= allowed
            and all(value == '' for key, value in env.items() if key in {'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'}), '/environment', 'unreviewed environment')
    for tool in (() if step == 28 else ('mkdir', 'cat')):
        found = shutil.which(tool, path=env.get('PATH', '/usr/bin:/bin:/usr/sbin:/sbin'))
        require(found is not None, '/tool', 'missing shell utility')
        assert found is not None
        resolved = Path(found).resolve(strict=True)
        require(bound.get(str(resolved)) == file_hash(resolved), '/tool', 'unbound shell utility')
    require(not request.output.exists() and not request.output.is_relative_to(request.root)
            and not request.root.is_relative_to(request.output), '/output', 'fresh external output required')
    return bound


def children(plan: CapturePlan, parent: ProcessResult, cargo: Path) -> tuple[CapturedProcess, CapturedProcess]:
    _name, profile, threads, argv = recipe(plan.step)
    require(plan.profile == profile and len(plan.inventory.executables) == 1 and not plan.inventory.doctests, '/chain', 'wrong inventory')
    item = plan.inventory.executables[0]
    artifacts = read(plan.manifest).field('artifacts').array()
    entries = {row.field('path').text(): row for row in artifacts}
    require(len(artifacts) == 2 and set(entries) == {str(cargo), str(item.binary)}, '/chain', 'wrong artifact closure')
    first, second = plan.capture_root/'000001', plan.capture_root/'000002'
    pending = read(first/'pending.json')
    launcher, ancestor = integer(pending.field('launcher_pid'), positive=True), integer(pending.field('parent_pid'), positive=True)
    require((launcher == parent.pid or ancestor == parent.pid) if plan.step == 28 else parent.native_exit == 0,
            '/chain', 'direct Bash edge or normal source-owned subshell wait missing')
    top = captured_process(plan, ancestor, entries[str(cargo)], cargo, plan.parent_spec.input_root/'rust', (str(cargo), *argv[1:]), first)
    require(top.capture_complete and top.child_pid is not None and top.native_exit is not None and top.native_exit >= 0, '/chain', 'Cargo wait unresolved')
    native = captured_process(plan, top.child_pid, entries[str(item.binary)], item.binary, item.cwd,
                              (str(item.binary), f'--test-threads={threads}'), second)
    require(native.capture_complete and native.native_exit is not None and native.native_exit >= 0, '/chain', 'native wait unresolved')
    claims = {'artifact-'+digest(str(path).encode())+'.claim' for path in (cargo, item.binary)}
    require({path.name for path in plan.capture_root.iterdir()} == {'sequence.lock', '000001', '000002', *claims}
            and (plan.capture_root/'sequence.lock').read_bytes() == b'2', '/chain', 'extra or omitted captured child')
    return top, native


def output_state(root: Path) -> dict[str, tuple[str, str]]:
    """Freeze files and directories; special paths never become accepted evidence."""
    files = snapshot(root)
    state: dict[str, tuple[str, str]] = {}
    for path in sorted(root.rglob('*')):
        name, mode = str(path.relative_to(root)), path.lstat().st_mode
        require(stat.S_ISDIR(mode) or stat.S_ISREG(mode), '/outputs', 'special output path')
        state[name] = ('directory', '') if stat.S_ISDIR(mode) else ('file', files[name])
    require({name for name, (kind, _sha) in state.items() if kind == 'file'} == set(files), '/outputs', 'output changed during inventory')
    return state


def run28_owned(request: Library49Request) -> NativePrefixResult:
    return run_owned(request, 28)


def run36_owned(request: Library49Request) -> NativePrefixResult:
    return run_owned(request, 36)


def run_owned(request: Library49Request, step: int) -> NativePrefixResult:
    """Run only the bound prefix; UNKNOWN retains the actual caller-owned lane."""
    bound = checked_request(request, step)
    _name, profile, _threads, argv = recipe(step)
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
                             {'id': 'selected-integration', 'path': str(request.executable.binary), 'sha256': bound[str(request.executable.binary)]}]
    atomic_json(manifest, {'schema_version': 1, 'invocation_id': request.invocation, 'step': step, 'arm': request.arm,
        'profile': profile, 'output_root': str(capture), 'launcher_sha256': bound[str(request.helper)],
        'python_sha256': bound[str(request.python)], 'inputs': {key: value for key, value in immutable.items()}, 'artifacts': artifacts, 'observer_variable': None})
    env = dict(request.environment)
    env.update({'PATH': str(shim_dir)+':'+env.get('PATH', '/usr/bin:/bin:/usr/sbin:/sbin'),
        'CARGO_TARGET_'+request.host.upper().replace('-', '_')+'_RUNNER': shlex.join((str(request.python), str(request.helper))),
        'CARGO_CAPTURE_MANIFEST': str(manifest), 'CARGO_CAPTURE_MANIFEST_SHA256': file_hash(manifest)})
    command = 'set -euo pipefail; cd rust && '+shlex.join(argv)
    if step == 36:
        command = 'set -euo pipefail\nROOT="$PWD"\nOUT="$0"\nmkdir -p "$OUT"\nOUT="$(cd "$OUT" && pwd -P)"\ncd "$ROOT"\nsource tools/v8/godot_bin.sh\n(\n    cd rust\n    '+shlex.join(argv)+'\n) > "$OUT/simulation.log" 2>&1\ncat "$OUT/simulation.log"'
    external = (*request.tools, (shim, file_hash(shim)), (manifest, file_hash(manifest)))
    spec = ProcessSpec((str(request.bash), '-c', command, str(request.output)), request.root, request.bash,
        bound[str(request.bash)], request.root, request.inputs, request.output/'outer', tuple(sorted(env.items())), external)
    active = False
    try:
        active = True
        result = execute(spec)
        parent = ListingProcess(spec, result, file_hash(result.receipt))
        required = {**bound, str(shim): file_hash(shim), str(manifest): file_hash(manifest)}
        raw = process_output(parent, spec.argv, request.root, environment_for(spec), required)
        plan = CapturePlan(request.invocation, step, request.arm, profile, spec, parent.receipt_sha256, manifest,
            file_hash(manifest), Inventory((request.executable,), ()), capture, tuple(sorted(snapshot(capture).items())))
        cargo, native = children(plan, result, request.cargo)
        require(result.native_exit is not None and result.native_exit >= 0 and result.native_exit == cargo.native_exit, '/outer', 'prefix wait/outcome differs')
        native_raw = (native.receipt.parent/'stdout.log').read_bytes()
        require(native_raw == (cargo.receipt.parent/'stdout.log').read_bytes(), '/stream', 'native/Cargo forwarding differs')
        require(raw == (native_raw if step == 28 else (request.output/'simulation.log').read_bytes()), '/stream', 'prefix forwarding differs')
        if step == 36:
            require(result.native_exit == native.native_exit == 0
                    and not {'WORLDSIM_DATA_ONLY_CASE', 'WORLDSIM_CONTENT_PACK'} & dict(native.original_environment).keys(),
                    '/descendants', 'normal unselected parent with synchronous child success required')
        active = False
        frozen = output_state(request.output)
        listings: list[ListingProcess] = []
        native_env = read(native.receipt).field('environment')
        environment = tuple((key, native_env.field(key).text()) for key in sorted(native_env.mapping()))
        for label, suffix in (('full', ('--list',)), ('ignored', ('--list', '--ignored'))):
            require(output_state(request.output) == frozen, '/outputs', 'completed evidence changed before listing')
            listing_spec = ProcessSpec((str(request.executable.binary), *suffix), request.executable.cwd,
                request.executable.binary, bound[str(request.executable.binary)], request.root, request.inputs,
                request.output/label, environment, external)
            active = True
            listing = execute(listing_spec)
            completed = output_state(listing_spec.output_root)
            binding = ListingProcess(listing_spec, listing, file_hash(listing.receipt))
            _ = process_output(binding, listing_spec.argv, listing_spec.cwd, dict(environment), required)
            require(listing.native_exit == 0, '/listing', 'listing did not complete normally')
            active = False
            frozen.update({label: ('directory', ''), **{label+'/'+name: value for name, value in completed.items()}})
            require(output_state(request.output) == frozen, '/outputs', 'completed evidence changed during listing or authentication')
            listings.append(binding)
        evidence = authenticate_listings(ListingScope(request.executable.binary, bound[str(request.executable.binary)],
            request.executable.cwd, environment, tuple((Path(key), value) for key, value in required.items())), IndependentListings(listings[0], listings[1]))
        full, ignored = parse_listing(evidence.full), parse_listing(evidence.ignored)
        require(bool(full.names), '/cases', 'zero-test prefix forbidden')
        accounting = (PrefixAccounting(reconcile_native(native_raw, full, ignored), digest(native_raw))
                      if step == 28 else account36(native_raw, full, ignored))
        require(output_state(request.output) == frozen, '/outputs', 'completed evidence changed during accounting')
        _ = process_output(parent, spec.argv, request.root, environment_for(spec), required)
        require(output_state(request.output) == frozen, '/outputs', 'completed evidence changed during terminal authentication')
        current = {name: sha for name, (kind, sha) in frozen.items() if kind == 'file'}
        status = 'PASS_PREFIX_ONLY' if result.native_exit == native.native_exit == 0 and accounting.section.failed == 0 else 'FAIL'
        receipt = request.output/'result.json'
        payload: dict[str, Json] = {'schema_version': 1, 'component': 'INTEGRATION_PREFIX', 'step': step, 'status': status,
            'output_inventory': {key: value for key, value in current.items()},
            'output_membership': {name: kind for name, (kind, _sha) in frozen.items()}, 'outer_pid': result.pid,
            'cargo_pid': cargo.child_pid, 'native_pid': native.child_pid, 'outer_exit': result.native_exit, 'native_exit': native.native_exit,
            'nested_ranges': [[start, end, sha] for start, end, sha in accounting.nested_ranges],
            'outer_to_cargo_edge': 'SOURCE_OWNED_NORMAL_SUBSHELL_WAIT' if step == 36 else 'DIRECT_PID_JOIN',
            'subshell_pid': None, 'nested_case_order_source_inferred': list(CASES) if step == 36 else [], 'nested_pids': None, 'nested_environments': None,
            'nested_wait_basis': 'REVIEWED_STATUS_CHAIN_CONDITIONAL_ON_CALLER_BUILD_ADMISSION' if step == 36 else 'NOT_APPLICABLE',
            'source_build_admission': 'PENDING_CALLER_HANDOFF', 'ordinary_safety': 'PENDING_SEPARATE_COMPONENT',
            'compound_tail': 'NOT_RUN', 'observer_qualification': 'PENDING_ORIGINAL_DIRECT_CONTROL',
            'observer_transformation': 'PATH_CARGO_SHIM_AND_HOST_RUNNER_WITH_SOURCE_OWNED_PREFIX_SPLIT',
            'original_environment_byte_identity': False, 'runtime_toolchain_qualification': 'PENDING_CALLER_HANDOFF',
            'publication_authority': False}
        receipt_hash = digest((json.dumps(payload, indent=2, sort_keys=True, allow_nan=False)+'\n').encode())
        atomic_json(receipt, payload)
        require(output_state(request.output) == {**frozen, 'result.json': ('file', receipt_hash)},
                '/outputs', 'completed evidence or result changed during publication')
        return NativePrefixResult(receipt, status, accounting)
    except BaseException as error:
        if active:
            raise LibraryOwnershipUnknown(request, error) from error
        raise LibrarySettledFailure(request, error) from error
