"""Same-PID rustdoc observation; endpoint provenance, never case/build admission."""
import ctypes
import hashlib
import os
import stat
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import cast

from generic_gate import GateContractError, boundary, document, integer, require, text
from generic_observations import keys
from generic_process import atomic_json, file_hash, real_path
from mux2_gate_inputs import EvidenceError, Json, Node


@dataclass(frozen=True, slots=True)
class Transient:
    lexical: Path
    canonical: Path
    evidence: dict[str, Json]
    closure_plan: Node | None = None


def read(path: Path) -> Node:
    return document(path.read_bytes().decode('utf-8'))


@boundary
def dependency_closure(plan: Node, policy: Node) -> None:
    """Verify v2 member bindings, never confer compiler provenance."""
    inputs = plan.field('inputs').mapping()
    artifacts: dict[str, tuple[Path, str]] = {}
    paths: set[Path] = set()
    for row in plan.field('artifacts').array():
        keys(row, {'id', 'path', 'sha256'})
        ident = text(row.field('id')); path = Path(text(row.field('path')))
        real_path(path, directory=False)
        require(ident not in artifacts and path not in paths and str(path) not in inputs,
                '/dependencies', 'duplicate artifact or overlapping input')
        artifacts[ident] = (path, text(row.field('sha256'))); paths.add(path)
    refs = [text(row) for row in policy.field('artifact_dependencies').array()]
    require(refs == sorted(set(refs)), '/dependencies', 'artifact references not sorted unique')
    used: set[str] = set(); roots: list[Path] = []
    for search in policy.field('dependency_searches').array():
        keys(search, {'path', 'directories', 'files'})
        root = Path(text(search.field('path'))); real_path(root, directory=True)
        require(not any(root.is_relative_to(other) or other.is_relative_to(root) for other in roots),
                '/dependencies', 'overlapping search directories')
        roots.append(root)
        directories: list[str] = []; files: list[str] = []
        for member in root.rglob('*'):
            require(not member.is_symlink(), '/dependencies', 'symlink search member')
            name = str(member.relative_to(root))
            if member.is_dir(): directories.append(name)
            else:
                require(member.is_file(), '/dependencies', 'unsupported search member')
                files.append(name)
        search.field('directories').equals([name for name in sorted(directories)])
        declared: list[str] = []
        for row in search.field('files').array():
            keys(row, {'relative_path', 'binding_kind', 'binding_id', 'sha256'})
            name = text(row.field('relative_path')); relative = Path(name)
            require(not relative.is_absolute() and '..' not in relative.parts and str(relative) == name,
                    '/dependencies', 'noncanonical relative member')
            path = root/relative; declared.append(name)
            ident = text(row.field('binding_id')); digest = text(row.field('sha256'))
            match text(row.field('binding_kind')):
                case 'input':
                    require(ident == str(path) and inputs.get(ident) == digest and path not in paths,
                            '/dependencies', 'unbound input member')
                    require(not path.stat().st_mode & 0o111, '/dependencies', 'executable member requires artifact binding')
                case 'artifact':
                    require(ident in refs and artifacts.get(ident) == (path, digest),
                            '/dependencies', 'unbound artifact member')
                    used.add(ident)
                case _:
                    require(False, '/dependencies', 'unknown binding kind')
            require(file_hash(path) == digest, '/dependencies', 'member bytes changed')
        require(declared == sorted(files), '/dependencies', 'search membership differs')
    require([str(root) for root in roots] == sorted(str(root) for root in roots), '/dependencies', 'search roots not sorted')
    require(used == set(refs), '/dependencies', 'unused or missing artifact references')
    expected: set[Path] = set()
    for row in policy.field('invocations').array():
        argv = [text(arg) for arg in row.field('argv').array()]
        cwd = Path(text(row.field('compiler_cwd')))
        for index, arg in enumerate(argv):
            if arg == '-L':
                require(index+1 < len(argv) and argv[index+1].startswith('dependency='), '/dependencies', 'unsupported search operand')
                expected.add((cwd/argv[index+1].split('=', 1)[1]).resolve(strict=True))
    require(set(roots) == expected, '/dependencies', 'search set differs from invocations')


@boundary
def closure_for(plan: Node) -> None:
    """Shared v2 entry/terminal check; v1 retains its original path."""
    if 'doctest_policy' in plan.mapping() and integer(plan.field('doctest_policy').field('schema_version')) != 1:
        _ = policy_for(plan)


@boundary
def policy_for(plan: Node) -> Node:
    policy = plan.field('doctest_policy')
    version = integer(policy.field('schema_version'))
    require(version in (1, 2), '/policy', 'unsupported policy version')
    keys(policy, ({'artifact_dependencies', 'dependency_searches'} if version == 2 else set[str]()) | {'schema_version', 'parent_receipt', 'rustdoc', 'rustdoc_sha256', 'version_receipt',
                  'version_sha256', 'entrypoint', 'entrypoint_sha256', 'wrapper_sha256', 'exec_root', 'temporary_root', 'invocations'})
    inputs = plan.field('inputs').mapping()
    parent = Path(text(policy.field('parent_receipt')))
    require(parent.is_absolute() and parent.name == 'process.json' and parent.parent.resolve() == parent.parent,
            '/parent', 'expected owned canonical parent receipt path required')
    require(all(str(parent.with_name(name)) not in inputs for name in ('process.json', 'process-spawn.json', 'process-start.json')),
            '/parent', 'future process receipts cannot be immutable inputs')
    for key, hash_key in (('rustdoc', 'rustdoc_sha256'), ('version_receipt', 'version_sha256'), ('entrypoint', 'entrypoint_sha256')):
        path = Path(text(policy.field(key)))
        digest = text(policy.field(hash_key))
        require(inputs.get(str(path)) == digest and file_hash(path) == digest, '/policy', 'unbound tool/version input')
    require(file_hash(Path(__file__).resolve()) == text(policy.field('wrapper_sha256')) and
            inputs.get(str(Path(__file__).resolve())) == text(policy.field('wrapper_sha256')), '/wrapper', 'unbound wrapper')
    require(file_hash(Path(sys.executable).resolve()) == text(plan.field('python_sha256')), '/wrapper', 'interpreter changed')
    for name, digest in inputs.items(): require(file_hash(Path(name)) == text(Node(digest, name)), name, 'immutable input changed')
    for key in ('exec_root', 'temporary_root'): real_path(Path(text(policy.field(key))), directory=True)
    if version == 2: dependency_closure(plan, policy)
    return policy


@boundary
def match_invocation(plan: Node, policy: Node, argv: tuple[str, ...], cwd: Path) -> Node:
    version = integer(policy.field('schema_version'))
    matches: list[Node] = []
    for row in policy.field('invocations').array():
        fields = {'id', 'source', 'package_manifest', 'crate_name', 'edition', 'argv'}
        keys(row, fields | ({'package_id', 'source_argument', 'compiler_cwd', 'run_cwd'} if version == 2 else {'cwd'}))
        if text(row.field('compiler_cwd' if version == 2 else 'cwd')) == str(cwd) and tuple(text(arg) for arg in row.field('argv').array()) == argv:
            matches.append(row)
    require(len(matches) == 1, '/rustdoc/argv', 'no unique source-owned exact invocation')
    row = matches[0]
    source = Path(text(row.field('source')))
    package = Path(text(row.field('package_manifest')))
    run_cwd = Path(text(row.field('run_cwd'))) if version == 2 else cwd
    real_path(cwd, directory=True); real_path(run_cwd, directory=True)
    require(package == run_cwd/'Cargo.toml' and source.is_relative_to(run_cwd), '/source', 'foreign source/package')
    if version == 2:
        require(not any(arg.startswith(('--test=', '--test-run-directory=')) for arg in argv),
                '/source', 'joined source/run operands unsupported')
        require(bool(text(row.field('package_id'))), '/source', 'empty package identity')
        real_path(source, directory=False); real_path(package, directory=False)
        source_arg = text(row.field('source_argument'))
        require(argv.count('--test') == 1 and argv.index('--test')+1 < len(argv) and argv[argv.index('--test')+1] == source_arg and argv.count(source_arg) == 1 and (cwd/source_arg).resolve(strict=True) == source, '/source', 'source argument differs')
        values = [argv[i+1] for i, arg in enumerate(argv[:-1]) if arg == '--test-run-directory']
        require(argv.count('--test-run-directory') == 1 and values == [str(run_cwd)], '/source', 'run directory differs')
    for path in (source, package):
        require(plan.field('inputs').field(str(path)).text() == file_hash(path), '/source', 'unbound source/package')
    require(argv.count('--test') == 1 and any(not arg.startswith('-') and (cwd/arg).resolve() == source for arg in argv),
            '/rustdoc/argv', 'not source-qualified --test')
    for flag, value in (('--crate-name', text(row.field('crate_name'))), ('--edition', text(row.field('edition')))):
        values = [arg.split('=', 1)[1] for arg in argv if arg.startswith(flag+'=')]
        values.extend(argv[index+1] for index, arg in enumerate(argv[:-1]) if arg == flag)
        require(values == [value], '/rustdoc/argv', 'crate/edition differs')
    bound = set(plan.field('inputs').mapping())
    if version == 2:
        bound.update(text(item.field('path')) for item in plan.field('artifacts').array()
                     if text(item.field('id')) in [text(ref) for ref in policy.field('artifact_dependencies').array()])
    # Exact declaration is caller source-owned; all concrete file arguments must also be frozen inputs.
    for arg in argv:
        value = arg.split('=', 1)[-1]
        candidate = cwd/value
        if candidate.is_file():
            require(str(candidate.resolve()) in bound, '/rustdoc/argv', 'unbound file argument')
    for index, arg in enumerate(argv):
        require(not arg.startswith('--extern=') and not (arg.startswith('-L') and arg != '-L'),
                '/rustdoc/argv', 'joined dependency grammar unsupported; no argument rewriting')
        if arg in ('--extern', '-L'):
            require(index+1 < len(argv), '/rustdoc/argv', 'missing dependency argument')
            value = argv[index+1].split('=', 1)[-1]
            path = (cwd/value).resolve(strict=True)
            files = tuple(path.rglob('*')) if arg == '-L' else (path,)
            require(bool(files), '/rustdoc/argv', 'empty dependency search path')
            for dependency in files:
                require(not dependency.is_symlink(), '/rustdoc/argv', 'symlink dependency')
                if dependency.is_file():
                    require(str(dependency) in bound, '/rustdoc/argv', 'unbound dependency search input')
    return row


@boundary
def ancestor(policy: Node, cargo_pid: int) -> dict[str, Json]:
    require(cargo_pid > 0, '/parent', 'positive actual Cargo PID required')
    path = Path(text(policy.field('parent_receipt'))).with_name('process-spawn.json')
    raw = path.read_bytes()
    document(raw.decode()).field('pid').equals(cargo_pid)
    return {'expected_terminal': text(policy.field('parent_receipt')), 'spawn_receipt': str(path),
            'spawn_sha256': hashlib.sha256(raw).hexdigest(), 'cargo_pid': cargo_pid,
            'final_parent_qualification': 'PENDING_AUTHENTICATED_PROCESS_RECEIPT_JOIN'}


@boundary
def exec_rustdoc(manifest: Path, digest: str, argv: tuple[str, ...]) -> None:
    raw = manifest.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == digest, '/manifest', 'anchor differs')
    plan = document(raw.decode()); policy = plan.field('doctest_policy')
    root = Path(text(policy.field('exec_root'))); real_path(root, directory=True)
    output = Path(tempfile.mkdtemp(prefix='exec-', dir=root))
    identity: dict[str, Json] = {'schema_version': 1, 'component': 'RUSTDOC_EXEC_INTENT', 'occurrence': output.name,
        'invocation_id': text(plan.field('invocation_id')), 'step': integer(plan.field('step')),
        'arm': text(plan.field('arm')), 'profile': text(plan.field('profile')), 'manifest_sha256': digest,
        'wrapper_pid': os.getpid(), 'cargo_parent_pid': os.getppid(), 'cwd': str(Path.cwd().resolve()),
        'wrapper_argv': [str(Path(__file__).resolve()), *argv], 'rustdoc_argv': [text(policy.field('rustdoc')), *argv],
        'environment': {key: value for key, value in os.environ.items()}, 'started_ns': time.time_ns()}
    atomic_json(output/'exec-entry.json', identity)
    try:
        policy = policy_for(plan)
        identity['source_invocation'] = None
        if '--test' in argv: identity['source_invocation'] = text(match_invocation(plan, policy, argv, Path.cwd().resolve()).field('id'))
        else: require(argv in (('-vV',), ('--version',), ('-V',)), '/rustdoc/argv', 'unsupported non-test probe')
        try: identity['ancestor'] = ancestor(policy, os.getppid())
        except (OSError, GateContractError, EvidenceError, UnicodeError) as error: identity['ancestor'] = {'status': 'PROVISIONAL', 'error': str(error)}
        atomic_json(output/'exec-intent.json', identity)
        os.execve(text(policy.field('rustdoc')), (text(policy.field('rustdoc')), *argv), dict(os.environ))
    except (OSError, GateContractError, EvidenceError, UnicodeError) as error:
        atomic_json(output/'exec-failure.json', {**identity, 'error': str(error), 'failed_ns': time.time_ns()})
        raise


def live_parent_path(pid: int) -> str:
    require(sys.platform == 'darwin', '/live-parent', 'macOS proc_pidpath required')
    library = ctypes.CDLL('/usr/lib/libproc.dylib', use_errno=True)
    prototype = ctypes.CFUNCTYPE(ctypes.c_int, ctypes.c_int, ctypes.c_void_p, ctypes.c_uint32, use_errno=True)
    probe = prototype(('proc_pidpath', library))
    buffer = ctypes.create_string_buffer(4096)
    length = cast(int, probe(pid, buffer, len(buffer)))
    require(0 < length < len(buffer), '/live-parent', f'proc_pidpath failed length={length} errno={ctypes.get_errno()}')
    return bytes(buffer).split(b'\0', 1)[0].decode('utf-8')


def transient_identity(lexical: Path, temporary: Path) -> tuple[Path, dict[str, Json]]:
    require(lexical.is_absolute() and '..' not in lexical.parts, '/transient', 'absolute nonescaping path required')
    canonical = lexical.resolve(strict=True)
    alias: str | None = None
    for parent in (*reversed(lexical.parents), lexical):
        if parent.is_symlink():
            require(parent == Path('/var') and os.readlink(parent) == 'private/var', '/transient', 'unsupported symlink')
            alias = '/var -> /private/var'
    require(canonical.is_relative_to(temporary) and canonical.name == 'rust_out' and
            canonical.parent.name.startswith('rustdoctest'), '/transient', 'outside declared rustdoc temporary scope')
    info = canonical.stat()
    require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid() and bool(info.st_mode & 0o111), '/transient', 'not owned executable regular file')
    return canonical, {'lexical': str(lexical), 'canonical': str(canonical), 'alias': alias, 'device': info.st_dev,
        'inode': info.st_ino, 'uid': info.st_uid, 'size': info.st_size, 'mode': info.st_mode}


@boundary
def admit_transient(plan: Node, digest: str, argv: tuple[str, ...], cwd: Path, output: Path) -> Transient:
    policy = policy_for(plan)
    require(len(argv) == 1, '/transient/argv', 'only original executable-only rustdoc request supported')
    parent = os.getppid()
    matches: list[tuple[Path, Node, str]] = []
    for path in Path(text(policy.field('exec_root'))).glob('*/exec-intent.json'):
        raw = path.read_bytes(); row = document(raw.decode('utf-8'))
        if integer(row.field('wrapper_pid')) == parent: matches.append((path, row, hashlib.sha256(raw).hexdigest()))
    require(len(matches) == 1, '/exec-intent', 'missing or ambiguous immediate-parent intent')
    path, row, intent_hash = matches[0]
    require(not path.with_name('exec-failure.json').exists(), '/exec-intent', 'exec failed')
    row.field('schema_version').equals(1); row.field('component').equals('RUSTDOC_EXEC_INTENT')
    row.field('occurrence').equals(path.parent.name)
    row.field('manifest_sha256').equals(digest)
    for name in ('invocation_id', 'step', 'arm', 'profile'): row.field(name).equals(plan.field(name).value)
    compiler_cwd = Path(text(row.field('cwd')))
    if integer(policy.field('schema_version')) == 1: row.field('cwd').equals(str(cwd))
    args = tuple(text(arg) for arg in row.field('rustdoc_argv').array())
    require(bool(args) and args[0] == text(policy.field('rustdoc')), '/exec-intent', 'rustdoc path differs')
    row.field('wrapper_argv').equals([str(Path(__file__).resolve()), *args[1:]])
    row.field('environment').field('CARGO_CAPTURE_MANIFEST_SHA256').equals(digest)
    source = match_invocation(plan, policy, args[1:], compiler_cwd)
    if integer(policy.field('schema_version')) == 2: source.field('run_cwd').equals(str(cwd))
    row.field('source_invocation').equals(text(source.field('id')))
    ancestry = ancestor(policy, integer(row.field('cargo_parent_pid')))
    try:
        observed = live_parent_path(parent)
    except (OSError, GateContractError, UnicodeError) as error:
        atomic_json(output/'live-parent.json', {'parent_before': parent, 'parent_after': os.getppid(), 'error': str(error), 'path': None})
        raise
    parent_after = os.getppid()
    atomic_json(output/'live-parent.json', {'parent_before': parent, 'parent_after': parent_after, 'path': observed, 'error': None})
    require(parent_after == parent, '/live-parent', 'immediate parent changed')
    require(file_hash(path) == intent_hash, '/exec-intent', 'validated intent changed during probe')
    require(Path(observed).resolve() == Path(text(policy.field('rustdoc'))) and
            file_hash(Path(observed).resolve()) == text(policy.field('rustdoc_sha256')), '/live-parent', 'wrong rustdoc image')
    lexical = Path(argv[0]); canonical, identity = transient_identity(lexical, Path(text(policy.field('temporary_root'))))
    before = file_hash(canonical)
    saved = output/'transient-binary'
    with canonical.open('rb') as source_file, saved.open('xb') as destination:
        while block := source_file.read(65536): _ = destination.write(block)
        destination.flush(); os.fsync(destination.fileno())
    require(transient_identity(lexical, Path(text(policy.field('temporary_root'))))[1] == identity and
            file_hash(canonical) == before == file_hash(saved), '/transient', 'binary changed while retaining')
    evidence: dict[str, Json] = {'classification': 'RUSTDOC_TRANSIENT', 'exec_intent': str(path), 'exec_sha256': intent_hash,
        'ancestor': ancestry, 'live_parent_pid': parent, 'live_parent_path': observed, 'identity_before': identity,
        'saved_binary': str(saved), 'sha256': before, 'temporary_root': text(policy.field('temporary_root')),
        'endpoint_limit': 'NO_ATOMIC_EXEC_OR_HOSTILE_PATH_RACE_PROOF', 'doctest_cases': 'INCOMPLETE'}
    require(file_hash(path) == intent_hash, '/exec-intent', 'validated intent changed before admission')
    if integer(policy.field('schema_version')) == 2:
        evidence['compiler_cwd'] = str(compiler_cwd); evidence['run_cwd'] = str(cwd)
    atomic_json(output/'transient-admission.json', evidence)
    return Transient(lexical, canonical, evidence, plan if integer(policy.field('schema_version')) == 2 else None)


@boundary
def postflight(admission: Transient) -> dict[str, Json]:
    if admission.closure_plan is not None: closure_for(admission.closure_plan)
    _, identity = transient_identity(admission.lexical, Path(text(Node(admission.evidence['temporary_root'], '/temporary'))))
    require(identity == admission.evidence['identity_before'], '/transient', 'endpoint identity changed')
    require(file_hash(admission.canonical) == admission.evidence['sha256'], '/transient', 'endpoint bytes changed')
    require(file_hash(Path(text(Node(admission.evidence['exec_intent'], '/intent')))) == admission.evidence['exec_sha256'], '/intent', 'intent changed')
    ancestor_row = Node(admission.evidence['ancestor'], '/ancestor')
    require(file_hash(Path(text(ancestor_row.field('spawn_receipt')))) == text(ancestor_row.field('spawn_sha256')), '/ancestor', 'spawn receipt changed')
    require(file_hash(Path(text(Node(admission.evidence['saved_binary'], '/saved')))) == admission.evidence['sha256'], '/saved', 'retained binary changed')
    return identity


def main() -> None:
    try: exec_rustdoc(Path(os.environ.get('CARGO_CAPTURE_MANIFEST', '')), os.environ.get('CARGO_CAPTURE_MANIFEST_SHA256', ''), tuple(sys.argv[1:]))
    except (GateContractError, OSError, UnicodeError) as error:
        print('rustdoc exec rejected: '+str(error), file=sys.stderr)
        raise SystemExit(125) from error


if __name__ == '__main__':
    main()
