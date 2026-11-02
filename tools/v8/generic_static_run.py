# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""One original static gate command; never a full-gate admission."""
import ast
import re
import shutil
from dataclasses import dataclass
from pathlib import Path

from generic_fixture_run import ComponentResult
from generic_gate import document, require
from generic_process import (
    ProcessSpec,
    atomic_json,
    execute,
    file_hash,
    real_path,
    timestamp,
)
from generic_sources import freeze_source, git, inventory
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, EvidenceError, Json

GATE_SHA256 = 'e794a514289ee13b58b76a1745f6b42c5040ae1acb0af20bf43c5261b7e1cc1d'
COMMANDS: dict[int, tuple[str, tuple[str, ...]]] = {
    1: ('fmt', ('bash', '-c', 'cd rust && cargo fmt --all -- --check')),
    7: ('forbidden_math', ('bash', '-c', r"cd rust && ! grep -rnE '\.(exp|exp2|ln|log2|log10|powf|powi|sin|cos|tan|tanh|atan2|hypot|cbrt)\(' crates/sim-core/src crates/sim-systems/src crates/sim-engine/src")),
    13: ('bridge_api', ('python3', 'tools/v8/bridge_api.py', '.', '--check', 'docs/v8/bridge_api_read.txt')),
    15: ('inventory_closure', ('python3', 'tools/v8/inventory.py', 'check', 'docs/v8/V8_0_INVENTORY.md')),
    16: ('python_unittest', ('python3', '-m', 'unittest', 'discover', '-s', 'tools/v8', '-p', 'test_*.py')),
    20: ('uid_check', ('bash', '-c', 'out=$(python3 tools/v8/check_docs_helpers.py uid); [[ -z "$out" ]] || { echo "$out"; exit 1; }; echo "every Godot source has its committed .uid"')),
    22: ('schema_docs', ('python3', 'tools/v8/check_docs_helpers.py', 'schema', '.')),
    31: ('art_manifest', ('python3', 'tools/v8/art_import.py', '--verify')),
    33: ('extensibility_grep', ('python3', '-m', 'unittest', 'tools/v8/test_mart2_extensibility.py', 'tools/v8/test_art_masks.py', 'tools/v8/test_art_import.py')),
}


@dataclass(frozen=True)
class StaticProduct:
    root: Path
    source_sha: str
    inputs: tuple[tuple[str, str], ...]


@dataclass(frozen=True)
class ExecutionEnvironment:
    values: tuple[tuple[str, str], ...]
    # Names resolved through explicit PATH, including cargo-fmt and real rustfmt.
    tools: tuple[tuple[str, Path, str], ...]
    # Caller supplies Python/toolchain/config dependency files; no inferred closure.
    external_inputs: tuple[tuple[Path, str], ...]
    prior_receipts: tuple[tuple[Path, str], ...] = ()


def verify_receipt(path: Path, digest: str) -> None:
    """Authenticate this adapter's fixed schema and complete child artifact tree."""
    require(file_hash(path) == digest, str(path), 'receipt changed')
    node = document(path.read_text(encoding='utf-8'))
    require(node.field('schema').text() == 'generic_static_run.v1', str(path), 'wrong receipt schema')
    expected = {key: value.text() for key, value in
                ((key, node.field('artifacts').field(key)) for key in node.field('artifacts').mapping())}
    actual = inventory(path.parent, ['process']) if (path.parent / 'process').exists() else {}
    require(actual == expected, str(path), 'prior artifacts changed')
    require({p.name for p in path.parent.iterdir()} == ({'receipt.json', 'process'} if actual else {'receipt.json'}),
            str(path), 'unexpected receipt output')


def _tools(context: ExecutionEnvironment, step: int) -> dict[str, Path]:
    env = dict(context.values)
    require(len(env) == len(context.values), '/environment', 'duplicate environment key')
    require({'PATH', 'LANG', 'LC_ALL', 'CARGO_BUILD_JOBS'} <= env.keys(), '/environment', 'explicit gate environment missing')
    require(env['CARGO_BUILD_JOBS'].isdigit() and int(env['CARGO_BUILD_JOBS']) > 0, '/environment', 'invalid job count')
    require(all(Path(part).is_absolute() for part in env['PATH'].split(':')), '/PATH', 'relative/empty search path')
    required = {'git', COMMANDS[step][1][0]}
    if step == 1:
        required |= {'cargo', 'cargo-fmt', 'rustfmt'}
    if step == 7:
        required.add('grep')
    if step == 20:
        required.add('python3')
    if step == 16:
        required |= {'bash', 'grep', 'sh', 'sleep', 'dirname', 'mkdir', 'sed', 'date', 'touch', 'env', 'shasum', 'cmp', 'perl'}
        for key, value in {'PYTHONDONTWRITEBYTECODE': '1', 'PYTHONNOUSERSITE': '1',
                           'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_CONFIG_NOSYSTEM': '1'}.items():
            require(env.get(key) == value, key, 'step16 startup policy differs')
        for key in ('HOME', 'TMPDIR'):
            require(key in env, key, 'step16 directory policy missing')
            real_path(Path(env[key]), directory=True)
        require(not ({'PYTHONPATH', 'PYTHONHOME', 'PERL5LIB', 'PERL5OPT', 'WORLDSIM_MUX2_LANE_FD'} & env.keys()),
                '/environment', 'step16 inherited runtime or lane override')
    result: dict[str, Path] = {}
    for name, path, digest in context.tools:
        require(name not in result and '/' not in name, '/tools', 'duplicate or unsafe tool name')
        resolved = shutil.which(name, path=env['PATH'])
        require(resolved is not None and Path(resolved).resolve(strict=True) == path, name, 'PATH tool differs')
        require(file_hash(path) == digest, name, 'tool changed')
        result[name] = path
    require(required <= result.keys(), '/tools', 'missing helper tool binding')
    if step == 16:
        for name in ('bash', 'sh', 'sleep'):
            require(result[name] == Path('/bin', name).resolve(strict=True), name, 'absolute test executable differs')
        require(result['perl'] == Path('/usr/bin/perl').resolve(strict=True), 'perl', 'shasum interpreter differs')
    for path, digest in context.external_inputs:
        require(file_hash(path) == digest, str(path), 'external config/dependency changed')
    return result


def validate_environment(context: ExecutionEnvironment, step: int) -> dict[str, Path]:
    """Apply the same tool/startup policy before a coordinator launches any child."""
    require(type(step) is int and step in COMMANDS, '/step', 'unsupported required step')
    return _tools(context, step)


def _refs(product: StaticProduct, step: int) -> dict[str, str]:
    names: list[str] = []
    if step == 15:
        names.append('v7-final-art')
    if step in {16, 33}:
        names.append(document((product.root / 'tools/v8/art_import.json').read_text(encoding='utf-8')).field('source_commit').text())
    if step == 16:
        module = ast.parse((product.root / 'tools/v8/art_masks.py').read_text(encoding='utf-8'))
        values = [node.value.value for node in module.body if isinstance(node, ast.Assign)
                  and any(isinstance(target, ast.Name) and target.id == 'SOURCE_COMMIT' for target in node.targets)
                  and isinstance(node.value, ast.Constant) and isinstance(node.value.value, str)]
        require(len(values) == 1 and re.fullmatch('[0-9a-f]{40}', values[0]) is not None,
                '/refs', 'art mask source commit is not one literal full SHA')
        names.extend(values)
    return {name: git(product.root, 'rev-parse', '--verify', name + '^{commit}').decode().strip() for name in names}


def unittest_terminal(stderr: str) -> dict[str, Json]:
    """Read only the original unittest terminal; declaration counts are not outcomes."""
    matches = list(re.finditer(r'^Ran ([1-9][0-9]*) tests? in ([0-9]+(?:\.[0-9]+)?)s$', stderr, re.MULTILINE))
    require(len(matches) == 1, '/unittest', 'missing, empty or duplicate test count')
    count = int(matches[0][1])
    terminal = stderr[matches[0].end():].strip()
    status = re.fullmatch(r'(OK|FAILED)(?: \(([^\n]+)\))?', terminal)
    require(status is not None, '/unittest', 'missing or malformed final outcome')
    assert status is not None
    counts = {name: 0 for name in ('skipped', 'expected failures', 'failures', 'errors', 'unexpected successes')}
    seen: set[str] = set()
    if status[2]:
        for item in status[2].split(', '):
            row = re.fullmatch(r'([a-z ]+)=([1-9][0-9]*)', item)
            require(row is not None, '/unittest', 'malformed outcome count')
            assert row is not None
            name = row[1]
            require(name in counts and name not in seen, '/unittest', 'unknown or duplicate outcome count')
            seen.add(name)
            counts[name] = int(row[2])
    failed = any(counts[name] for name in ('failures', 'errors', 'unexpected successes'))
    require((status[1] == 'FAILED') == failed, '/unittest', 'contradictory outcome counts')
    require(counts['skipped'] + counts['expected failures'] <= count, '/unittest', 'success categories exceed test count')
    return {'tests_run': count, 'outcome': status[1], 'counts': {name: value for name, value in counts.items()},
            'count_authority': 'actual_unittest_terminal_not_source_declarations'}


def run_step(product: StaticProduct, step: int, output: Path,
             execution_environment: ExecutionEnvironment) -> ComponentResult:
    require(type(step) is int and step in COMMANDS, '/step', 'unsupported required step')
    real_path(output.parent, directory=True)
    require(output.is_absolute() and not output.exists() and not output.is_symlink(), str(output), 'stale output')
    require(not output.is_relative_to(product.root), str(output), 'output overlaps source')
    output.mkdir()
    name, original = COMMANDS[step]
    receipt: dict[str, Json] = {
        'schema': 'generic_static_run.v1', 'step': step, 'name': name,
        'scope': 'COMPONENT_ONLY', 'original_order_executed': False,
        'full_gate_complete': False, 'source_admission_granted': False,
        'source_root': str(product.root), 'source_sha': product.source_sha,
        'source_invocation': list(original), 'started_at': timestamp(),
        'status': 'FAIL', 'process_status': 'NOT_RUN', 'artifacts': {},
        'tool_dependency_closure': 'caller_supplied_not_independently_complete',
    }
    errors: list[str] = []
    artifacts: dict[str, str] = {}
    before: dict[str, Json] | None = None
    refs: dict[str, str] | None = None
    context = execution_environment
    input_names = [*INPUT_DIRS, *INPUT_FILES, *(['docs'] if step == 16 else [])]
    try:
        tools = _tools(context, step)
        for path, digest in context.prior_receipts:
            verify_receipt(path, digest)
        require(file_hash(product.root / 'tools/v8/gate.sh') == GATE_SHA256, '/gate', 'command source changed')
        before = freeze_source(product.root, product.source_sha)
        inputs = inventory(product.root, input_names)
        require(len(dict(product.inputs)) == len(product.inputs) and inputs == dict(product.inputs), '/inputs', 'effective inventory differs')
        refs = _refs(product, step)
        receipt['source_before'] = before
        receipt['inputs_before'] = dict(inputs)
        receipt['refs_before'] = dict(refs)
        receipt['tool_bindings'] = {name: {'path': str(path), 'sha256': digest}
                                    for name, path, digest in context.tools}
        receipt['prior_receipts'] = {str(path): digest for path, digest in context.prior_receipts}
        externals = dict(context.external_inputs)
        for _, path, digest in context.tools:
            require(path not in externals or externals[path] == digest, str(path), 'conflicting tool binding')
            externals[path] = digest
        entry = tools[original[0]]
        spec = ProcessSpec((str(entry), *original[1:]), product.root, entry, file_hash(entry),
                           product.root, product.inputs, output / 'process', context.values,
                           tuple(externals.items()))
        result = execute(spec)
        artifacts = inventory(output, ['process'])
        receipt.update(process_status=result.status, native_exit=result.native_exit, pid=result.pid,
                       process_receipt=str(result.receipt), process_receipt_sha256=file_hash(result.receipt))
        native = document(result.receipt.read_text(encoding='utf-8'))
        require(native.field('environment').mapping() == dict(context.values), '/environment', 'effective environment differs')
        if step == 16:
            terminal = unittest_terminal((output / 'process/stderr.log').read_bytes().decode('utf-8'))
            receipt['unittest'] = terminal
            require(terminal['outcome'] == 'OK' and result.native_exit == 0, '/unittest', 'original discovery failed')
        require(result.status == 'PASS' and result.native_exit == 0, '/process', 'original command failed')
    except (OSError, ValueError, SyntaxError, EvidenceError) as error:
        errors.append(str(error))
    # Always check all endpoint bindings, even following a failed child.
    try:
        if (output / 'process').exists():
            current = inventory(output, ['process'])
            if artifacts:
                require(current == artifacts, '/artifacts', 'child output changed')
            else:
                artifacts = current
        _ = _tools(context, step)
        for path, digest in context.prior_receipts:
            verify_receipt(path, digest)
        after = freeze_source(product.root, product.source_sha)
        receipt['source_end'] = after
        require(before is not None and after == before, '/source', 'source binding differs')
        ending = inventory(product.root, input_names)
        receipt['inputs_end'] = dict(ending)
        require(ending == dict(product.inputs), '/inputs', 'effective inputs changed')
        refs_end = _refs(product, step)
        receipt['refs_end'] = dict(refs_end)
        require(refs_end == refs, '/refs', 'historical reference changed')
        _ = _tools(context, step)
        for path, digest in context.prior_receipts:
            verify_receipt(path, digest)
        require(inventory(output, ['process']) == artifacts, '/artifacts', 'late child output mutation')
    except (OSError, ValueError, SyntaxError, EvidenceError) as error:
        errors.append('terminal: ' + str(error))
    receipt['artifacts'] = dict(artifacts)
    receipt['errors'] = list(errors)
    receipt['finished_at'] = timestamp()
    receipt['status'] = 'FAIL' if errors else 'PASS'
    atomic_json(output / 'receipt.json', receipt)
    return ComponentResult(output / 'receipt.json', 'FAIL' if errors else 'PASS')
