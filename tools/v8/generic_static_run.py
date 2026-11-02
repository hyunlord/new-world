# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""One original static gate command; never a full-gate admission."""
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
    result: dict[str, Path] = {}
    for name, path, digest in context.tools:
        require(name not in result and '/' not in name, '/tools', 'duplicate or unsafe tool name')
        resolved = shutil.which(name, path=env['PATH'])
        require(resolved is not None and Path(resolved).resolve(strict=True) == path, name, 'PATH tool differs')
        require(file_hash(path) == digest, name, 'tool changed')
        result[name] = path
    require(required <= result.keys(), '/tools', 'missing helper tool binding')
    for path, digest in context.external_inputs:
        require(file_hash(path) == digest, str(path), 'external config/dependency changed')
    return result


def _refs(product: StaticProduct, step: int) -> dict[str, str]:
    names: list[str] = []
    if step == 15:
        names.append('v7-final-art')
    if step == 33:
        names.append(document((product.root / 'tools/v8/art_import.json').read_text(encoding='utf-8')).field('source_commit').text())
    return {name: git(product.root, 'rev-parse', '--verify', name + '^{commit}').decode().strip() for name in names}


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
    try:
        tools = _tools(context, step)
        for path, digest in context.prior_receipts:
            verify_receipt(path, digest)
        require(file_hash(product.root / 'tools/v8/gate.sh') == GATE_SHA256, '/gate', 'command source changed')
        before = freeze_source(product.root, product.source_sha)
        inputs = inventory(product.root, list(INPUT_DIRS + INPUT_FILES))
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
        require(result.status == 'PASS' and result.native_exit == 0, '/process', 'original command failed')
    except (OSError, ValueError, EvidenceError) as error:
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
        ending = inventory(product.root, list(INPUT_DIRS + INPUT_FILES))
        receipt['inputs_end'] = dict(ending)
        require(ending == dict(product.inputs), '/inputs', 'effective inputs changed')
        refs_end = _refs(product, step)
        receipt['refs_end'] = dict(refs_end)
        require(refs_end == refs, '/refs', 'historical reference changed')
        _ = _tools(context, step)
        for path, digest in context.prior_receipts:
            verify_receipt(path, digest)
        require(inventory(output, ['process']) == artifacts, '/artifacts', 'late child output mutation')
    except (OSError, ValueError, EvidenceError) as error:
        errors.append('terminal: ' + str(error))
    receipt['artifacts'] = dict(artifacts)
    receipt['errors'] = list(errors)
    receipt['finished_at'] = timestamp()
    receipt['status'] = 'FAIL' if errors else 'PASS'
    atomic_json(output / 'receipt.json', receipt)
    return ComponentResult(output / 'receipt.json', 'FAIL' if errors else 'PASS')
