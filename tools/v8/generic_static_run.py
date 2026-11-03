# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""One original static gate command; never a full-gate admission."""
import ast
import fcntl
import os
import re
import shutil
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import NoReturn

from generic_fixture_run import ComponentResult
from generic_gate import document, integer, require
from generic_m6a_structural import resolution_paths
from generic_process import (
    ProcessSpec,
    atomic_json,
    execute,
    file_hash,
    real_path,
    timestamp,
)
from generic_sources import freeze_source, git, inventory, safe_path
from mux2_gate_inputs import BRIDGE, INPUT_DIRS, INPUT_FILES, EvidenceError, Json
from mux2_gate_lock import LOCK

GATE_SHA256 = 'ba949251efa089536c44b9883515d017c7254122b4167756833dfd7070d8c523'
DOCS_SKILLS = ('worldsim-architecture', 'worldsim-godot', 'worldsim-pitfalls', 'worldsim-testing',
               'worldsim-docs-upkeep', 'systematic-debugging', 'verification-before-completion')
DOCS_TOOLS = {'python3', 'cargo', 'rustc', 'dirname', 'grep', 'wc', 'tr', 'cat', 'tail', 'rm',
              'head', 'readlink', 'awk', 'sed', 'xargs', 'stat', 'ls'}
DOCS_CHECKERS = {
    'check_docs.sh': '7b96439637cd407a4b6c370bb10d6a2bbcee9ef56e083f66a694c414f6fd440f',
    'system_table.py': 'bfddc9e6e2876711a8f8445119b2b19fc0c5298a7ed84a63cae84bb1fd4b7a29',
    'check_docs_helpers.py': '1cb534486144404cc03e98649f779d1ec70c0cfbb39f884a85c2944346bcb2f0',
    'bridge_api.py': 'b98e86cdcbf03d704d1bd11df2851dd3cd53f76b8642ae6c0074ae7477526903',
}
DIAGNOSIS_RESULT = 'docs/v8/read/read_30s_result.json'
STRICT_FILES = ('tools/v8/mux21_layout_test.gd', 'tools/v8/mux21_toast_test.gd',
                'tools/v8/mux21_play_capture.gd', 'tools/v8/mux21_why_locale_test.gd')
COMMANDS: dict[int, tuple[str, tuple[str, ...]]] = {
    8: ('gdscript_strict', ('bash', 'tools/v8/gdscript_strict_check.sh')),
    46: ('gdscript.four_files', ('bash', 'tools/v8/gdscript_strict_check.sh', *STRICT_FILES)),
    1: ('fmt', ('bash', '-c', 'cd rust && cargo fmt --all -- --check')),
    7: ('forbidden_math', ('bash', '-c', r"cd rust && ! grep -rnE '\.(exp|exp2|ln|log2|log10|powf|powi|sin|cos|tan|tanh|atan2|hypot|cbrt)\(' crates/sim-core/src crates/sim-systems/src crates/sim-engine/src")),
    14: ('docs', ('bash', 'tools/v8/check_docs.sh')),
    13: ('bridge_api', ('python3', 'tools/v8/bridge_api.py', '.', '--check', 'docs/v8/bridge_api_read.txt')),
    15: ('inventory_closure', ('python3', 'tools/v8/inventory.py', 'check', 'docs/v8/V8_0_INVENTORY.md')),
    16: ('python_unittest', ('python3', '-m', 'unittest', 'discover', '-s', 'tools/v8', '-p', 'test_*.py')),
    20: ('uid_check', ('bash', '-c', 'out=$(python3 tools/v8/check_docs_helpers.py uid); [[ -z "$out" ]] || { echo "$out"; exit 1; }; echo "every Godot source has its committed .uid"')),
    22: ('schema_docs', ('python3', 'tools/v8/check_docs_helpers.py', 'schema', '.')),
    31: ('art_manifest', ('python3', 'tools/v8/art_import.py', '--verify')),
    33: ('extensibility_grep', ('python3', '-m', 'unittest', 'tools/v8/test_mart2_extensibility.py', 'tools/v8/test_art_masks.py', 'tools/v8/test_art_import.py')),
    51: ('read_diagnosis_test', ('bash', '-c', '\n    set -euo pipefail\n    RESULT="docs/v8/read/read_30s_result.json"\n    [[ -f "$RESULT" ]] || { echo "result file $RESULT does not exist"; exit 1; }\n    python3 -c "\nimport json, sys\nr = json.load(open(sys.argv[1]))\npassed = sum(1 for q in r[\\"questions\\"] if q[\\"pass\\"])\ntotal = len(r[\\"questions\\"])\nq1 = next(q for q in r[\\"questions\\"] if q[\\"id\\"] == \\"Q1\\")\nq5 = next(q for q in r[\\"questions\\"] if q[\\"id\\"] == \\"Q5\\")\np1 = q1[\\"pass\\"]\np5 = q5[\\"pass\\"]\nprint(f\\"30s test: {passed}/{total}, Q1 pass={p1}, Q5 pass={p5}\\")\nassert passed >= 5, f\\"need >=5/6, got {passed}/{total}\\"\nassert q1[\\"pass\\"], \\"Q1 must pass\\"\nassert q5[\\"pass\\"], \\"Q5 must pass\\"\nprint(\\"READ_DIAGNOSIS_TEST PASS\\")\n" "$RESULT"\n')),
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
    # Native docs/strict checks: caller retains this descriptor, including on unknown ownership.
    held_lane_fd: int | None = None


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


class StrictOwnershipUnknown(RuntimeError):
    """Outer shell settlement does not establish descendant quiescence; retain lane."""


def _strict_unknown(output: Path, receipt: dict[str, Json], reason: str) -> NoReturn:
    try:
        receipt.update(ownership='INCOMPLETE_DESCENDANTS_UNKNOWN', errors=[reason],
                       finished_at=timestamp())
        atomic_json(output/'receipt.json', receipt)
    except BaseException as error:
        # Even diagnostic interruption cannot establish descendant quiescence.
        raise StrictOwnershipUnknown('strict diagnostic write failed; retain lane') from error
    raise StrictOwnershipUnknown(f'{output}: {reason}')


def strict_files(root: Path, step: int) -> tuple[str, ...]:
    require(step in {8, 46}, '/strict', 'unknown strict step')
    names = STRICT_FILES if step == 46 else tuple(sorted(
        path.relative_to(root).as_posix() for path in (root/'scripts').rglob('*.gd')
        if 'test' not in path.relative_to(root).parts))
    require(bool(names), '/strict', 'empty original script selection')
    for name in names:real_path(root/name, directory=False)
    return names


def strict_input_names(root: Path) -> list[str]:
    """Reuse the admitted Godot resolver; require a stable prepared runtime."""
    caches = ['.godot/global_script_class_cache.cfg', '.godot/extension_list.cfg', '.godot/uid_cache.bin']
    real_path(root/'.godot/imported', directory=True)
    names = [*INPUT_DIRS, *INPUT_FILES, BRIDGE, *caches, '.godot/imported']
    names.extend(name for name in sorted(resolution_paths(root)) if not any(
        name == prior or name.startswith(prior+'/') for prior in names))
    return names


def docs_input_names(root: Path, context: ExecutionEnvironment) -> list[str]:
    """Bind regular target trees while preserving the original seven skill links."""
    for name in DOCS_SKILLS:
        link = safe_path(root, '.agents/skills/'+name)
        require(link.is_symlink() and os.readlink(link) == '../../.claude/skills/'+name,
                str(link), 'original skill link differs')
        target = safe_path(root, '.claude/skills/'+name+'/SKILL.md')
        real_path(target, directory=False)
        require((link/'SKILL.md').resolve(strict=True) == target, str(link), 'skill target differs')
    names = [*INPUT_DIRS, *INPUT_FILES, 'docs', '.claude/skills',
             'README.md', 'AGENTS.md', 'CLAUDE.md', 'CHANGELOG.md']
    for path in docs_config_paths(root, context):
        if path.is_relative_to(root) and docs_optional_hash(path) is not None:
            name = path.relative_to(root).as_posix()
            if not any(name == prior or name.startswith(prior+'/') for prior in names):names.append(name)
    return names


def docs_config_paths(root: Path, context: ExecutionEnvironment) -> tuple[Path, ...]:
    """Enumerate the original Cargo cwd/ancestor and explicit Cargo-home lookups."""
    env = dict(context.values)
    require('CARGO_HOME' in env, '/docs', 'explicit Cargo home required')
    paths = {ancestor/'.cargo'/name for ancestor in (root/'rust', *(root/'rust').parents)
             for name in ('config', 'config.toml')}
    paths.update(Path(env['CARGO_HOME'])/name for name in ('config', 'config.toml'))
    return tuple(sorted(paths))


def docs_optional_hash(path: Path) -> str | None:
    """Only a missing file is absent; unreadable config/ignore endpoints fail closed."""
    try:_ = path.lstat()
    except FileNotFoundError:return None
    return file_hash(path)


def docs_bindings(root: Path, context: ExecutionEnvironment) -> dict[str, Json]:
    """Bind original checker and lookup endpoints; caller owns current world classification."""
    for name, digest in DOCS_CHECKERS.items():
        require(file_hash(root/'tools/v8'/name) == digest, '/docs', 'original docs checker changed')
    configs: dict[str, Json] = {}
    external = dict(context.external_inputs)
    require(len(external) == len(context.external_inputs), '/docs', 'duplicate external inputs')
    for path in docs_config_paths(root, context):
        digest = docs_optional_hash(path)
        if path.is_relative_to(root):
            require(path not in external, str(path), 'in-tree Cargo config belongs to product inputs')
        else:
            require(digest is None or external.get(path) == digest, str(path), 'unbound external Cargo config')
        configs[str(path)] = digest
    # check_docs_helpers only asks existence for these source-owned literal paths.
    architecture: dict[str, Json] = {}
    prefixes = ('rust/', 'scripts/', 'scenes/', 'tools/', 'docs/', 'assets/', 'localization/',
                'shaders/', '.claude/', '.agents/')
    for match in re.finditer(r'`([^`\s]+)`', (root/'docs/ARCHITECTURE.md').read_text()):
        name = match[1].split(':')[0].rstrip('.,;)')
        if name.startswith(prefixes) and not any(char in name for char in '*<{'):
            path = root/name
            require(path.resolve().is_relative_to(root), name, 'architecture lookup escapes source')
            architecture[name] = path.exists()
    ignores: dict[str, Json] = {}
    for raw in git(root, 'ls-files', '-z').split(b'\0'):
        name = os.fsdecode(raw)
        if name.endswith(('.gd', '.gdshader')):
            parent = safe_path(root, name).parent
            while parent != root:
                path = parent/'.gdignore'
                ignores[path.relative_to(root).as_posix()] = docs_optional_hash(path)
                parent = parent.parent
    return {'cargo_config_endpoints': configs,
            'architecture_exists': architecture, 'godot_ignore_endpoints': ignores}


def docs_terminal(stdout: str) -> None:
    """Require the original aggregated result; do not replace individual docs checks."""
    lines = [line for line in stdout.splitlines() if line.strip()]
    require(lines.count('docs: PASS') == 1 and lines[-1] == 'docs: PASS' and
            not any(line.startswith('FAIL:') or line == 'docs: FAIL' for line in lines),
            '/docs', 'missing, failed, duplicate or nonterminal original docs marker')


def strict_terminal(stdout: str, stderr: str, count: int) -> None:
    lines = [line for line in stdout.splitlines() if line.strip()]
    require(count > 0 and bool(lines) and lines[-1] == '[gdcheck] ALL CHECKS PASS' and
            lines.count('[gdcheck] ALL CHECKS PASS') == 1, '/strict', 'missing or nonterminal strict marker')
    parse = re.findall(r'^\[gdcheck\] parse \+ warnings: ([0-9]+) file\(s\) clean ✓$', stdout, re.MULTILINE)
    ffi = re.findall(r'^\[gdcheck\] FFI bindings: ([0-9]+) GDScript call\(s\) all match Rust #\[func\] exports ✓$', stdout, re.MULTILINE)
    require(parse == [str(count)] and len(ffi) == 1, '/strict', 'parse count or FFI terminal differs')
    require(not re.search(r'^\[gdcheck\] (?:FAIL|WARN|ERROR)', stdout+'\n'+stderr, re.MULTILINE),
            '/strict', 'original strict oracle reported failure or skipped FFI')


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
    if step in {20, 51}:
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
    if step in {8, 14, 46}:
        descriptor = context.held_lane_fd
        require(type(descriptor) is int and descriptor >= 0, '/lane', 'caller-held descriptor required')
        if descriptor is None:raise StrictOwnershipUnknown('missing caller lane')
        held = os.fstat(descriptor);canonical = LOCK.stat()
        require(stat.S_ISREG(held.st_mode) and (held.st_dev, held.st_ino) == (canonical.st_dev, canonical.st_ino),
                '/lane', 'descriptor is not the canonical native lane')
        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
    if step in {8, 46}:
        required |= {'python3', 'find', 'mktemp', 'rm', 'cp', 'cat', 'wc', 'tr', 'grep', 'head', 'sed', 'sort'}
        allowed = {'PATH', 'LANG', 'LC_ALL', 'CARGO_BUILD_JOBS', 'HOME', 'TMPDIR', 'GODOT_BIN',
                   'PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE', 'PYTHONCOERCECLOCALE',
                   'GIT_CONFIG_GLOBAL', 'GIT_CONFIG_NOSYSTEM', '__CF_USER_TEXT_ENCODING'}
        require(env.keys() <= allowed, '/environment', 'strict unapproved startup/runtime override')
        for key in ('PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE'):
            require(env.get(key) == '1', key, 'strict Python startup policy differs')
        for key in ('HOME', 'TMPDIR'):
            require(key in env, key, 'strict directory missing')
            real_path(Path(env[key]), directory=True)
        require('GODOT_BIN' in env, '/godot', 'explicit Godot required')
        godot = Path(env['GODOT_BIN']);real_path(godot, directory=False)
        require(len(dict(context.external_inputs)) == len(context.external_inputs) and
                dict(context.external_inputs).get(godot) == file_hash(godot), '/godot', 'unbound Godot executable')
    if step == 14:
        required |= DOCS_TOOLS
        allowed = {'PATH', 'LANG', 'LC_ALL', 'CARGO_BUILD_JOBS', 'HOME', 'TMPDIR', 'CARGO_HOME',
                   'RUSTUP_HOME', 'CARGO_TARGET_DIR', 'PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE',
                   'PYTHONCOERCECLOCALE', 'GIT_CONFIG_GLOBAL', 'GIT_CONFIG_NOSYSTEM', '__CF_USER_TEXT_ENCODING'}
        require(env.keys() <= allowed, '/environment', 'docs unapproved startup/runtime override')
        for key in ('PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE'):
            require(env.get(key) == '1', key, 'docs Python startup policy differs')
        for key in ('HOME', 'TMPDIR', 'CARGO_HOME', 'RUSTUP_HOME'):
            require(key in env, key, 'docs explicit directory missing')
            real_path(Path(env[key]), directory=True)
        if 'CARGO_TARGET_DIR' in env:real_path(Path(env['CARGO_TARGET_DIR']), directory=True)
    if step == 51:
        allowed = {'PATH', 'LANG', 'LC_ALL', 'CARGO_BUILD_JOBS', 'HOME', 'TMPDIR',
                   'PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE', 'PYTHONCOERCECLOCALE',
                   'GIT_CONFIG_GLOBAL', 'GIT_CONFIG_NOSYSTEM', '__CF_USER_TEXT_ENCODING'}
        require(env.keys() <= allowed, '/environment', 'step51 unapproved shell or process environment key')
        for key in ('PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE'):
            require(env.get(key) == '1', key, 'step51 Python startup policy differs')
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


def diagnosis_terminal(stdout: str) -> None:
    """Require the original diagnosis marker once, as the last nonblank line."""
    lines = [line for line in stdout.splitlines() if line.strip()]
    require(lines.count('READ_DIAGNOSIS_TEST PASS') == 1 and lines[-1] == 'READ_DIAGNOSIS_TEST PASS',
            '/diagnosis', 'missing, duplicate or nonterminal original diagnosis marker')


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


@dataclass(slots=True)
class _DocsExecution:
    output_created: bool = False
    in_flight: bool = False
    primary: BaseException | None = None


class DocsSettledFailure(RuntimeError):
    def __init__(self, original: BaseException, diagnostic: BaseException | None,
                 receipt: Path | None) -> None:
        super().__init__('docs failed with owned execution settled')
        self.original: BaseException = original
        self.diagnostic: BaseException | None = diagnostic
        self.receipt: Path | None = receipt


class DocsOwnershipUnknown(RuntimeError):
    def __init__(self, original: BaseException, diagnostic: BaseException | None,
                 ownership_receipt: Path | None) -> None:
        super().__init__('docs descendant ownership unknown; retain lane')
        self.original: BaseException = original
        self.diagnostic: BaseException | None = diagnostic
        self.ownership_receipt: Path | None = ownership_receipt


def run_docs_owned(product: StaticProduct, output: Path,
                   execution_environment: ExecutionEnvironment) -> ComponentResult:
    """Original14 only; caller owns source/build admission and the actual held lane.

    A source-qualified successful Bash/Cargo chain settles on bound actual PASS/0.
    Failed/unreturned execution may have skipped a descendant wait; retain ownership.
    Retained receipt parsing does not independently grant this live settlement.
    """
    state=_DocsExecution()
    try:
        result=_run_step(product,14,output,execution_environment,state)
    except BaseException as original:
        primary=state.primary if state.primary is not None else original
        secondary=original if primary is not original else None
        receipt: Path | None=None
        if state.output_created:
            try:
                path=output/('ownership.json' if state.in_flight else 'settled-failure.json')
                atomic_json(path,{'schema':'docs_owned_failure_v1',
                    'ownership':'INCOMPLETE_DESCENDANTS_UNKNOWN' if state.in_flight else 'SETTLED_FAILURE',
                    'final_inventory':False,'publication_authorized':False,
                    'exception_type':type(original).__name__})
                receipt=path
            except BaseException as diagnostic:
                if state.in_flight:raise DocsOwnershipUnknown(primary,diagnostic,None) from diagnostic
                raise DocsSettledFailure(primary,diagnostic,None) from diagnostic
        if state.in_flight:raise DocsOwnershipUnknown(primary,secondary,receipt) from original
        raise DocsSettledFailure(primary,secondary,receipt) from original
    if result.status!='PASS':
        original=state.primary if state.primary is not None else RuntimeError('docs validation failed')
        raise DocsSettledFailure(original,None,result.receipt) from original
    return result


def run_step(product: StaticProduct, step: int, output: Path,
             execution_environment: ExecutionEnvironment) -> ComponentResult:
    """Legacy command API; existing caller ownership responsibilities are unchanged."""
    return _run_step(product,step,output,execution_environment,None)


def _run_step(product: StaticProduct, step: int, output: Path,
              execution_environment: ExecutionEnvironment, state: _DocsExecution | None) -> ComponentResult:
    require(type(step) is int and step in COMMANDS, '/step', 'unsupported required step')
    real_path(output.parent, directory=True)
    require(output.is_absolute() and not output.exists() and not output.is_symlink(), str(output), 'stale output')
    require(not output.is_relative_to(product.root), str(output), 'output overlaps source')
    output.mkdir()
    if state is not None:state.output_created=True
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
    selected: tuple[str, ...] = ()
    project_mode: int | None = None
    docs_before: dict[str, Json] | None = None
    input_names = [*INPUT_DIRS, *INPUT_FILES, *(['docs'] if step == 16 else [DIAGNOSIS_RESULT] if step == 51 else [])]
    try:
        tools = _tools(context, step)
        if step in {8, 46}:
            project_mode = stat.S_IMODE((product.root/'project.godot').stat().st_mode)
            selected = strict_files(product.root, step)
            input_names = strict_input_names(product.root)
        if step == 14:
            input_names = docs_input_names(product.root, context)
            docs_before = docs_bindings(product.root, context)
            receipt.update(reviewed_world_scope='construction_no_tick_advance', reviewed_initial_state_mutations=True,
                           current_product_world_scope='CALLER_REQUIRED', build_provenance='CALLER_REQUIRED',
                           docs_before=docs_before)
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
        try:
            if state is not None:state.in_flight=True
            result = execute(spec)
            if state is not None:
                require(result.status=='PASS' and result.native_exit==0 and type(result.pid) is int and result.pid>0
                        and result.receipt==output/'process/process.json','/docs','aggregate completion unknown')
                native=document(result.receipt.read_text())
                native.field('status').equals('PASS');native.field('native_exit').equals(0)
                native.field('pid').equals(result.pid);native.field('errors').equals([])
                native.field('argv').equals(list(spec.argv));native.field('cwd').equals(str(spec.cwd))
                require(integer(native.field('duration_ns'))>0,'/docs','invalid completion duration')
                state.in_flight=False
        except BaseException:
            if state is not None:raise
            if step in {8, 14, 46}:
                receipt['process_status'] = 'UNKNOWN'
                _strict_unknown(output, receipt, 'docs execution did not return' if step == 14 else 'strict execution did not return')
            raise
        if step in {8, 14, 46} and result.pid is not None and (result.native_exit is None or result.native_exit < 0):
            receipt.update(process_status=result.status, native_exit=result.native_exit, pid=result.pid,
                           process_receipt=str(result.receipt))
            _strict_unknown(output, receipt, 'docs shell completion unknown' if step == 14 else 'strict shell completion unknown')
        if step == 14 and result.pid is not None and not result.receipt.is_file():
            receipt.update(process_status=result.status, native_exit=result.native_exit, pid=result.pid,
                           process_receipt=str(result.receipt))
            _strict_unknown(output, receipt, 'docs terminal receipt missing')
        artifacts = inventory(output, ['process'])
        receipt.update(process_status=result.status, native_exit=result.native_exit, pid=result.pid,
                       process_receipt=str(result.receipt), process_receipt_sha256=file_hash(result.receipt))
        if step == 14:
            receipt['artifacts'] = dict(artifacts)
        native = document(result.receipt.read_text(encoding='utf-8'))
        require(native.field('environment').mapping() == dict(context.values), '/environment', 'effective environment differs')
        if step in {8, 46}:
            strict_terminal((output/'process/stdout.log').read_text(), (output/'process/stderr.log').read_text(), len(selected))
        if step == 14:
            docs_terminal((output/'process/stdout.log').read_text())
        if step == 16:
            terminal = unittest_terminal((output / 'process/stderr.log').read_bytes().decode('utf-8'))
            receipt['unittest'] = terminal
            require(terminal['outcome'] == 'OK' and result.native_exit == 0, '/unittest', 'original discovery failed')
        if step == 51:
            diagnosis_terminal((output / 'process/stdout.log').read_bytes().decode('utf-8'))
        require(result.status == 'PASS' and result.native_exit == 0, '/process', 'original command failed')
    except (OSError, ValueError, SyntaxError, EvidenceError) as error:
        if state is not None:
            if state.in_flight:raise
            state.primary=error
            errors.append('owned docs validation failed')
        else:errors.append(str(error))
    except BaseException as failure:
        if state is not None:raise
        if step == 14 and not (output/'receipt.json').exists():
            try:
                receipt['errors'] = ['unexpected docs validation failure']
                atomic_json(output/'receipt.json', receipt)
            except BaseException as cleanup:raise failure from cleanup
        raise
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
        if step in {8, 46}:
            require(stat.S_IMODE((product.root/'project.godot').stat().st_mode) == project_mode,
                    '/strict', 'project mode changed')
            require(strict_files(product.root, step) == selected, '/strict', 'script selection changed')
            require(strict_input_names(product.root) == input_names, '/strict', 'resource resolution changed')
        if step == 14:
            require(docs_input_names(product.root, context) == input_names, '/docs', 'docs input scope changed')
            docs_after = docs_bindings(product.root, context)
            receipt['docs_end'] = docs_after
            require(docs_before is not None and docs_before == docs_after, '/docs', 'docs lookup bindings changed')
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
        if state is not None:
            if state.primary is None:state.primary=error
            errors.append('owned docs postflight failed')
        else:errors.append('terminal: ' + str(error))
    except BaseException as failure:
        if state is not None:raise
        if step == 14 and not (output/'receipt.json').exists():
            try:
                receipt['errors'] = ['unexpected docs postflight failure']
                atomic_json(output/'receipt.json', receipt)
            except BaseException as cleanup:raise failure from cleanup
        raise
    receipt['artifacts'] = dict(artifacts)
    receipt['errors'] = list(errors)
    receipt['finished_at'] = timestamp()
    receipt['status'] = 'FAIL' if errors else 'PASS'
    atomic_json(output / 'receipt.json', receipt)
    return ComponentResult(output / 'receipt.json', 'FAIL' if errors else 'PASS')
