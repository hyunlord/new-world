# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""One original static gate command; never a full-gate admission."""
import ast
import fcntl
import hashlib
import json
import os
import re
import shutil
import stat
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Final, NoReturn

from gate_oracle_child import import_generated
from generic_build import WorkspaceSourceProof
from generic_cli_prepare import CliBuilds, PairedSetupSourceClosure, recheck_cli_builds
from generic_embedded_build import EmbeddedEndpoint, check_endpoint
from generic_fixture_run import ComponentResult
from generic_gate import document, integer, require
from generic_m6a_structural import resolution_paths
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
    real_path,
    timestamp,
)
from generic_setup_bridge import immutable_inputs
from generic_setup_prepare import PreparedSetup
from generic_sources import freeze_source, git, inventory, safe_path
from generic_workspace_cases import ListingProcess, process_output
from generic_workspace_endpoints import (
    EvidenceAnchor,
    GeneratedState,
    anchor,
    encoded,
    generated,
    state_json,
)
from mux2_gate_inputs import BRIDGE, INPUT_DIRS, INPUT_FILES, EvidenceError, Json, Node
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


BASE0_SHA = '2f6993009ddb600e9249c28f9768a1a5252c8821'
BASE0_GATE_SHA256 = 'dd3d52db1905af1211d207dcd39796de150d6c2c6eba2595624cef3afbec7938'
STATIC_COMMANDS_SHA256 = 'ec73a616bba44c4cca4eeefecf8832deafbbf96eb1e93f7fef95fcaa71029b77'
BASE0_STATIC_STEPS = (1,7,8,13,14,15,16,20,22,31,33,46,51)


@dataclass(frozen=True, slots=True)
class Base0StaticSource:
    """Live-issued static-only source identity; not runtime or publication authority."""
    root: Path


@dataclass(frozen=True, slots=True)
class _Base0SourceProof:
    contract: Base0StaticSource
    builds: CliBuilds
    prepared: PreparedSetup
    closure: PairedSetupSourceClosure
    derived: WorkspaceSourceProof | None
    source_json: str
    descriptor_json: str


_BASE0_SOURCES: Final[dict[int,_Base0SourceProof]] = {}


@dataclass(frozen=True)
class StaticProduct:
    root: Path
    source_sha: str
    inputs: tuple[tuple[str, str], ...]
    source_contract: Base0StaticSource | None = None


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


def _command_contract() -> None:
    require(tuple(sorted(COMMANDS)) == BASE0_STATIC_STEPS
            and hashlib.sha256(json.dumps(COMMANDS,sort_keys=True,separators=(',',':')).encode()).hexdigest()
            == STATIC_COMMANDS_SHA256, '/static/source', 'reviewed static command projection changed')


def _base0_identity(builds: CliBuilds, prepared: PreparedSetup, closure: PairedSetupSourceClosure) -> None:
    PairedSetupSourceClosure.check(closure,builds)
    require(closure.builds is builds and closure.original is builds.retained_anchor,
            '/static/source', 'foreign original paired source issuer')
    selected=builds.prepared.selectors
    require(selected.purpose=='self_proof' and selected.published_sha==selected.base.source_sha
            ==selected.candidate.source_sha==BASE0_SHA, '/static/source', 'not exact BASE0 self proof')
    binding=document(builds.prepared.binding_json)
    for name,value in (('purpose','self_proof'),('base_product_sha',BASE0_SHA),
                       ('candidate_product_sha',BASE0_SHA),('frozen_main',BASE0_SHA),('merge_base',BASE0_SHA)):
        binding.field(name).equals(value)
    require(prepared.warm_pair is builds.candidate and prepared.warm_pair is not None
            and prepared.inputs.root==selected.candidate.root and prepared.inputs.source_sha==BASE0_SHA,
            '/static/source', 'foreign actual prepared candidate')
    assert prepared.warm_pair is not None
    require(prepared.derived_source is None or prepared.derived_source.retained_anchor is prepared.warm_pair.retained_anchor,
            '/static/source','foreign prepared source proof issuer')
    immutable_inputs(prepared)
    _command_contract()
    require(file_hash(prepared.inputs.root/'tools/v8/gate.sh')==BASE0_GATE_SHA256,
            '/static/source', 'BASE0 gate blob changed')
    baseline=document(prepared.derived_source.baseline_json) if prepared.derived_source is not None else document(json.dumps(freeze_source(prepared.inputs.root,BASE0_SHA)))
    baseline.field('root').equals(str(prepared.inputs.root))
    baseline.field('source_sha').equals(BASE0_SHA)
    baseline.field('tracked_manifest').field('tools/v8/gate.sh').array()[2].equals(BASE0_GATE_SHA256)
    for plan in builds.prepared.arms:
        for build in (plan.debug,plan.release):
            if build.overlay is not None:
                overlay=document(build.overlay.manifest.read_text())
                require('tools/v8/gate.sh' not in tuple(row.field('path').text() for row in overlay.field('files').array()),
                        '/static/source', 'gate cannot be an observer postimage')
    PairedSetupSourceClosure.check(closure,builds)


def _prepared_static_source(prepared: PreparedSetup) -> dict[str,Json]:
    immutable_inputs(prepared)
    return document(prepared.derived_source.effective_json).mapping() if prepared.derived_source is not None else freeze_source(prepared.inputs.root,prepared.inputs.source_sha)


def base0_static_source(builds: CliBuilds, prepared: PreparedSetup,
                        paired_closure: PairedSetupSourceClosure) -> Base0StaticSource:
    """Mint before setup changes generated endpoints; later use checks source/history only."""
    recheck_cli_builds(builds)
    _base0_identity(builds,prepared,paired_closure)
    source=json.dumps(_prepared_static_source(prepared),sort_keys=True)
    descriptor: dict[str,Json]={'schema':'base0_static_source.v1','purpose':'self_proof',
        'published_sha':BASE0_SHA,'source_sha':BASE0_SHA,'root':str(prepared.inputs.root),
        'gate_sha256':BASE0_GATE_SHA256,'commands_sha256':STATIC_COMMANDS_SHA256,
        'steps':list(BASE0_STATIC_STEPS),'source_kind':prepared.source_kind,
        'baseline_digest':prepared.source_digest,'effective_digest':prepared.effective_source_digest,
        'source_json_sha256':hashlib.sha256(source.encode()).hexdigest(),
        'scope':'STATIC_COMMAND_SOURCE_ONLY','publication_authority':False}
    contract=Base0StaticSource(prepared.inputs.root)
    proof=_Base0SourceProof(contract,builds,prepared,paired_closure,prepared.derived_source,source,json.dumps(descriptor,sort_keys=True))
    recheck_cli_builds(builds)
    _base0_identity(builds,prepared,paired_closure)
    require(json.dumps(_prepared_static_source(prepared),sort_keys=True)==source,'/static/source','source changed during issuance')
    PairedSetupSourceClosure.check(paired_closure,builds)
    _BASE0_SOURCES[id(contract)]=proof
    return contract


def _base0_proof(product: StaticProduct) -> _Base0SourceProof:
    contract=product.source_contract
    proof=_BASE0_SOURCES.get(id(contract))
    require(proof is not None and proof.contract is contract,'/static/source','unissued or copied BASE0 contract')
    assert proof is not None and contract is not None
    require(contract.root==product.root==proof.prepared.inputs.root and product.source_sha==BASE0_SHA
            and proof.prepared.derived_source is proof.derived, '/static/source','replaced source contract binding')
    _base0_identity(proof.builds,proof.prepared,proof.closure)
    require(json.dumps(_prepared_static_source(proof.prepared),sort_keys=True)==proof.source_json,
            '/static/source','original effective source changed')
    PairedSetupSourceClosure.check(proof.closure,proof.builds)
    return proof


def source_contract_descriptor(product: StaticProduct) -> dict[str,Json] | None:
    """Source-only historical join; never rechecks obsolete generated build endpoints."""
    if product.source_contract is None:return None
    return document(_base0_proof(product).descriptor_json).mapping()


def static_source(product: StaticProduct) -> dict[str,Json]:
    if product.source_contract is None:
        require(file_hash(product.root/'tools/v8/gate.sh')==GATE_SHA256,'/gate','command source changed')
        return freeze_source(product.root,product.source_sha)
    return document(_base0_proof(product).source_json).mapping()


def authenticate_source_receipt(product: StaticProduct, receipt: Node) -> None:
    """Join explicit contract history without treating serialized labels as authority."""
    if product.source_contract is None:
        require('source_contract' not in receipt.mapping(),'/static/source','unexpected source contract')
        return
    proof=_base0_proof(product)
    receipt.field('source_contract').equals(document(proof.descriptor_json).value)
    for name in ('source_before','source_end'):
        if name in receipt.mapping():receipt.field(name).equals(document(proof.source_json).value)
    if receipt.field('status').text()=='PASS':
        require({'source_before','source_end'} <= receipt.mapping().keys(),'/static/source','missing source endpoints')
    _= _base0_proof(product)


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


def _tools(context: ExecutionEnvironment, step: int, *, live_lane: bool = True) -> dict[str, Path]:
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
    if step in {8, 14, 46} and live_lane:
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


@dataclass(frozen=True, slots=True)
class DocsSuccessorRequest:
    product: StaticProduct
    environment: ExecutionEnvironment
    prepared: PreparedSetup
    predecessor: EmbeddedEndpoint
    output: Path
    builds: CliBuilds
    source_closure: PairedSetupSourceClosure

    @property
    def static_output(self) -> Path:
        return self.output.with_name(self.output.name+'-static')


@dataclass(frozen=True, slots=True)
class DocsSuccessor:
    request: DocsSuccessorRequest
    component: ComponentResult
    process: ListingProcess
    before: GeneratedState
    after: GeneratedState
    receipt: Path
    receipt_sha256: str
    docs_receipt_sha256: str
    history: tuple[EvidenceAnchor, ...]


@dataclass(frozen=True, slots=True)
class _DocsProof:
    successor: DocsSuccessor
    request: DocsSuccessorRequest
    source: str
    bindings: str
    files: tuple[tuple[Path, int, str], ...]
    source_check: Callable[[CliBuilds | None], None]


_DOCS_ISSUED: Final[dict[int, _DocsProof]] = {}


def _docs_files(request: DocsSuccessorRequest) -> tuple[tuple[Path, int, str], ...]:
    bindings = {request.product.root/name:digest for name,digest in request.product.inputs}
    for path,digest in (*request.environment.external_inputs,
                        *((path,digest) for _,path,digest in request.environment.tools)):
        require(path not in bindings or bindings[path] == digest, '/docs-successor', 'conflicting immutable binding')
        bindings[path] = digest
    for _,path,_ in request.environment.tools:
        require(os.access(path,os.X_OK), '/docs-successor', 'nonexecutable tool')
    result: list[tuple[Path,int,str]] = []
    for path,digest in sorted(bindings.items()):
        require(file_hash(path) == digest, '/docs-successor', 'immutable file changed')
        result.append((path,path.stat().st_mode,digest))
    return tuple(result)


def _docs_source(request: DocsSuccessorRequest) -> dict[str, Json]:
    if request.product.source_contract is not None:
        proof=_base0_proof(request.product)
        require(proof.builds is request.builds and proof.prepared is request.prepared and proof.closure is request.source_closure,
                '/static/source', 'docs successor has foreign source issuer')
        return document(proof.source_json).mapping()
    immutable_inputs(request.prepared)
    proof = request.prepared.derived_source
    if proof is not None:
        return document(proof.effective_json).mapping()
    return freeze_source(request.product.root, request.product.source_sha)


def _docs_admit(request: DocsSuccessorRequest) -> None:
    prepared = request.prepared; context = request.environment
    require(request.source_closure.builds is request.builds and prepared.warm_pair is request.builds.candidate
            and prepared.warm_pair is not None
            and prepared.inputs.root == request.builds.prepared.selectors.candidate.root
            and prepared.inputs.source_sha == request.builds.prepared.selectors.candidate.source_sha,
            '/docs-successor', 'foreign original build/source issuer')
    request.source_closure.check(request.builds)
    require(request.predecessor.request.prepared is prepared and request.predecessor.request.step == 12,
            '/docs-successor', 'actual same-setup step12 predecessor required')
    require(request.product.root == prepared.inputs.root and request.product.source_sha == prepared.inputs.source_sha,
            '/docs-successor', 'foreign source')
    check_endpoint(request.predecessor)
    immutable_inputs(prepared)
    env = dict(context.values); original = dict(prepared.inputs.environment)
    # Docs intentionally excludes setup-only Godot/settings/empty-wrapper keys.
    omitted = {'GODOT_BIN', 'WORLDSIM_TEST_SETTINGS_DIR', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'}
    require(all(env.get(key) == value for key,value in original.items() if key not in omitted),
            '/docs-successor', 'setup environment projection differs')
    require(all(original.get(key, '') == '' for key in ('RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER')),
            '/docs-successor', 'nonempty setup compiler wrapper')
    require(env.get('CARGO_HOME') == str(prepared.inputs.cargo_home)
            and env.get('CARGO_TARGET_DIR', str(prepared.inputs.root/'rust/target')) == str(prepared.inputs.root/'rust/target'),
            '/docs-successor', 'foreign Cargo home or target')
    tools = {name:(path,digest) for name,path,digest in context.tools}
    for tool in prepared.inputs.tools:
        if tool.name in {'bash','python3','cargo','rustc'}:
            require(tools.get(tool.name) == (tool.path,tool.sha256), '/docs-successor', 'setup tool differs')
    bound = {request.product.root/name:digest for name,digest in request.product.inputs}
    bound.update(dict(context.external_inputs))
    for name,digest in prepared.materialized:
        require(bound.get(prepared.inputs.root/name) == digest, '/docs-successor', 'missing setup source binding')
    for path,digest in prepared.external:
        require(bound.get(path) == digest, '/docs-successor', 'missing setup external binding')


def _docs_raw(proof: _DocsProof, current: GeneratedState | None = None) -> None:
    """Terminal concrete comparisons after all semantic callbacks; no new baseline."""
    if current is not None:
        prepared = proof.request.prepared
        for path, expected in ((prepared.inputs.root/'rust/target', current.target),
                               (prepared.inputs.root/'.godot', current.cache),
                               (prepared.inputs.settings, current.settings)):
            require(encoded(import_generated(path)) == expected,
                    '/docs-successor', 'late generated state changed')
    for path,mode,digest in proof.files:
        require(file_hash(path) == digest and path.stat().st_mode == mode,
                '/docs-successor', 'late immutable file bytes/mode changed')
    for item in proof.successor.history:
        require(import_generated(item.root) == document(item.state).mapping(),
                '/docs-successor', 'late historical namespace changed')


def _docs_history(proof: _DocsProof) -> None:
    endpoint = proof.successor; request = proof.request
    require(endpoint.request is request and endpoint.component.receipt == request.static_output/'receipt.json'
            and endpoint.receipt == request.output/'receipt.json', '/docs-successor', 'retained request differs')
    _docs_admit(request)
    require(json.dumps(_docs_source(request), sort_keys=True) == proof.source,
            '/docs-successor', 'source admission changed')
    require(json.dumps(docs_bindings(request.product.root,request.environment), sort_keys=True) == proof.bindings,
            '/docs-successor', 'lookup bindings changed')
    _ = _tools(request.environment,14,live_lane=False)
    require(inventory(request.product.root,docs_input_names(request.product.root,request.environment)) == dict(request.product.inputs),
            '/docs-successor', 'docs inputs changed')
    for path,digest in request.environment.prior_receipts: verify_receipt(path,digest)
    spec = endpoint.process.spec
    required = {str(spec.input_root/name):digest for name,digest in spec.inputs}
    required.update({str(path):digest for path,digest in spec.external_inputs})
    required[str(spec.executable)] = spec.executable_sha256
    _ = process_output(endpoint.process,spec.argv,spec.cwd,environment_for(spec),required)
    verify_receipt(endpoint.component.receipt,endpoint.docs_receipt_sha256)
    require(file_hash(endpoint.receipt) == endpoint.receipt_sha256, '/docs-successor', 'successor receipt changed')
    # Authentication callbacks cannot replace the original source/lookup baseline.
    require(json.dumps(_docs_source(request),sort_keys=True) == proof.source,
            '/docs-successor', 'late source admission changed')
    require(json.dumps(docs_bindings(request.product.root,request.environment),sort_keys=True) == proof.bindings,
            '/docs-successor', 'late lookup bindings changed')
    require(inventory(request.product.root,docs_input_names(request.product.root,request.environment)) == dict(request.product.inputs),
            '/docs-successor', 'late docs inputs changed')
    for _,path,digest in request.environment.tools:
        require(file_hash(path) == digest, '/docs-successor', 'late tool mutation')
    for path,digest in request.environment.external_inputs:
        require(file_hash(path) == digest, '/docs-successor', 'late external input mutation')
    require(_docs_files(request) == proof.files, '/docs-successor', 'immutable file mode changed')
    for item in endpoint.history: item.recheck()
    proof.source_check(request.builds)
    _docs_raw(proof)


def check_docs_successor_history(successor: DocsSuccessor) -> None:
    """Original issued identity and immutable history; later target evolution is allowed."""
    proof = _DOCS_ISSUED.get(id(successor))
    require(proof is not None and proof.successor is successor, '/docs-successor', 'unissued successor')
    assert proof is not None
    _docs_history(proof)


def check_docs_successor_current(successor: DocsSuccessor) -> None:
    """History plus the exact generated endpoint; never refresh the old baseline."""
    check_docs_successor_history(successor)
    require(generated(successor.request.prepared) == successor.after, '/docs-successor', 'generated state changed')
    for item in successor.history: item.recheck()
    proof = _DOCS_ISSUED[id(successor)]
    proof.source_check(successor.request.builds)
    _docs_raw(proof,successor.after)


@dataclass(slots=True)
class _DocsExecution:
    output_created: bool = False
    in_flight: bool = False
    primary: BaseException | None = None
    successor_request: DocsSuccessorRequest | None = None
    spec: ProcessSpec | None = None
    result: ProcessResult | None = None
    before: GeneratedState | None = None
    after: GeneratedState | None = None
    process_anchor: EvidenceAnchor | None = None
    initial_output: EvidenceAnchor | None = None
    returned_output: EvidenceAnchor | None = None
    binding: ListingProcess | None = None
    source: str | None = None
    bindings: str | None = None
    intended_receipt: str | None = None
    history: tuple[EvidenceAnchor, ...] = ()
    files: tuple[tuple[Path, int, str], ...] = ()
    source_check: Callable[[CliBuilds | None], None] | None = None
    oracle_failed: bool = False
    integrity_failed: bool = False


class DocsSettledFailure(RuntimeError):
    def __init__(self, original: BaseException, diagnostic: BaseException | None,
                 receipt: Path | None) -> None:
        super().__init__('docs failed with owned execution settled')
        self.original: BaseException = original
        self.diagnostic: BaseException | None = diagnostic
        self.receipt: Path | None = receipt
        self.successor: DocsSuccessor | None = None


class DocsOwnershipUnknown(RuntimeError):
    def __init__(self, original: BaseException, diagnostic: BaseException | None,
                 ownership_receipt: Path | None) -> None:
        super().__init__('docs descendant ownership unknown; retain lane')
        self.original: BaseException = original
        self.diagnostic: BaseException | None = diagnostic
        self.ownership_receipt: Path | None = ownership_receipt
        self.execution: _DocsExecution | None = None


def run_docs_owned(product: StaticProduct, output: Path,
                   execution_environment: ExecutionEnvironment) -> ComponentResult:
    """Original14 only; caller owns source/build admission and the actual held lane.

    A source-qualified successful Bash/Cargo chain settles on bound actual PASS/0.
    Failed/unreturned execution may have skipped a descendant wait; retain ownership.
    Retained receipt parsing does not independently grant this live settlement.
    """
    return _owned_docs(product,output,execution_environment,_DocsExecution())


def _owned_docs(product: StaticProduct, output: Path,
                execution_environment: ExecutionEnvironment, state: _DocsExecution) -> ComponentResult:
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
    if state is not None:
        state.output_created=True
        if state.successor_request is not None: state.initial_output=anchor(output)
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
        descriptor=source_contract_descriptor(product)
        if descriptor is not None:receipt['source_contract']=descriptor
        else:require(file_hash(product.root / 'tools/v8/gate.sh') == GATE_SHA256, '/gate', 'command source changed')
        before = _docs_source(state.successor_request) if state is not None and state.successor_request is not None else static_source(product)
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
            if state is not None:
                state.spec=spec
                if state.successor_request is not None: _docs_before_launch(state)
                state.in_flight=True
            result = execute(spec)
            if state is not None:
                state.result=result
                if state.successor_request is not None: _docs_returned(state)

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
            try: docs_terminal((output/'process/stdout.log').read_text())
            except (OSError, ValueError, EvidenceError):
                if state is not None: state.oracle_failed=True
                raise
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
            state.integrity_failed=not state.oracle_failed
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
        after = _docs_source(state.successor_request) if state is not None and state.successor_request is not None else static_source(product)
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
            state.integrity_failed=True
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
    if state is not None and state.successor_request is not None:
        state.intended_receipt=json.dumps(receipt,sort_keys=True,indent=2)+'\n'
    atomic_json(output / 'receipt.json', receipt)
    if product.source_contract is not None:
        authenticate_source_receipt(product,document((output/'receipt.json').read_text()))
    return ComponentResult(output / 'receipt.json', 'FAIL' if errors else 'PASS')


def _docs_before_launch(state: _DocsExecution) -> None:
    request = state.successor_request
    assert request is not None and state.initial_output is not None
    _docs_admit(request)
    state.source = json.dumps(_docs_source(request),sort_keys=True)
    state.bindings = json.dumps(docs_bindings(request.product.root,request.environment),sort_keys=True)
    state.before = generated(request.prepared)
    require(state.before == request.predecessor.after, '/docs-successor', 'prelaunch generated state differs')
    require(_docs_files(request) == state.files, '/docs-successor', 'prelaunch immutable file mode changed')
    for item in state.history: item.recheck()
    state.initial_output.recheck()
    assert state.source_check is not None
    state.source_check(request.builds)


def _docs_returned(state: _DocsExecution) -> None:
    request = state.successor_request; result = state.result; spec = state.spec
    assert request is not None and result is not None and spec is not None
    require(result.status == 'PASS' and result.native_exit == 0 and type(result.pid) is int and result.pid > 0,
            '/docs-successor', 'normal actual return required')
    # Capture once at actual return, before any authentication callback.
    state.after = generated(request.prepared)
    state.process_anchor = anchor(spec.output_root)
    state.returned_output = anchor(request.static_output)
    state.binding = ListingProcess(spec,result,file_hash(result.receipt))
    assert state.before is not None and state.initial_output is not None
    process_state = document(state.process_anchor.state)
    expected_members = {'.','process-start.json','process-spawn.json','process.json','stdout.log','stderr.log'}
    require(set(process_state.mapping()) == expected_members, '/docs-successor', 'unexpected actual process member')
    initial = document(state.initial_output.state).field('.')
    process_state.field('.').equals({'mode':stat.S_IFDIR | (integer(initial.field('mode')) & 0o2700),'sha256':None})
    expected_file_mode = stat.S_IFREG | (integer(initial.field('mode')) & 0o666)
    for name in expected_members-{'.'}:
        process_state.field(name).field('mode').equals(expected_file_mode)
    output_state = document(state.returned_output.state).mapping()
    require({key:value for key,value in output_state.items() if key != 'process' and not key.startswith('process/')} ==
            document(state.initial_output.state).mapping(), '/docs-successor', 'producer changed output namespace')
    require(state.before.cache == state.after.cache and state.before.settings == state.after.settings,
            '/docs-successor', 'docs changed cache/settings')


def _finish_docs(state: _DocsExecution, component: ComponentResult) -> DocsSuccessor:
    request = state.successor_request
    assert request is not None
    require(not state.in_flight and not state.integrity_failed, '/docs-successor', 'unsettled or invalid producer')
    assert state.before is not None and state.after is not None and state.binding is not None
    assert state.returned_output is not None and state.process_anchor is not None
    assert state.source is not None and state.bindings is not None and state.intended_receipt is not None
    require(component.receipt.read_text() == state.intended_receipt, '/docs-successor', 'docs receipt publication changed')
    returned = document(state.returned_output.state).mapping()
    actual = import_generated(request.static_output)
    require({key:value for key,value in actual.items() if key != 'receipt.json'} == returned,
            '/docs-successor', 'late docs output membership/mode change')
    assert state.initial_output is not None
    expected_file_mode = stat.S_IFREG | (integer(document(state.initial_output.state).field('.').field('mode')) & 0o666)
    require(actual.get('receipt.json') == {'mode':expected_file_mode,'sha256':file_hash(component.receipt)},
            '/docs-successor', 'docs receipt mode/kind differs')
    history = (*state.history,anchor(request.static_output))
    output = request.output
    output.mkdir(mode=0o700)
    empty = anchor(output)
    receipt = output/'receipt.json'
    assert request.builds.candidate is not None
    row: dict[str, Json] = {
        'cli_base_pair_receipt':str(request.builds.base.receipt),
        'cli_base_pair_sha256':file_hash(request.builds.base.receipt),
        'cli_candidate_pair_receipt':str(request.builds.candidate.receipt),
        'cli_candidate_pair_sha256':file_hash(request.builds.candidate.receipt),
        'prepared_source_digest':request.prepared.source_digest,
        'prepared_effective_source_digest':request.prepared.effective_source_digest,
        'source_kind':request.prepared.source_kind,
        'schema':'docs_successor.v1','step':14,'status':component.status,
        'scope':'GENERATED_TRANSITION_ONLY','build_product_authority':False,'normal_product_authority':False,
        'publication_authority':False,'chronology':'INTERVENING_COMPONENT_HANDOFF_PENDING',
        'source_sha':request.product.source_sha,'predecessor_sha256':request.predecessor.receipt_sha256,
        'pid':state.binding.result.pid,'native_exit':state.binding.result.native_exit,
        'docs_receipt':str(component.receipt),'docs_receipt_sha256':file_hash(component.receipt),
        'process_receipt':str(state.binding.result.receipt),'process_receipt_sha256':state.binding.receipt_sha256,
        'generated_before':state_json(state.before),'generated_after':state_json(state.after),
        'history':[{'root':str(item.root),'state':document(item.state).value} for item in history],
    }
    expected = json.dumps(row,sort_keys=True,indent=2)+'\n'
    atomic_json(receipt,row)
    require(receipt.read_text() == expected, '/docs-successor', 'successor publication content changed')
    written = import_generated(output)
    require({key:value for key,value in written.items() if key != 'receipt.json'} == document(empty.state).mapping()
            and written.get('receipt.json') == {'mode':expected_file_mode,'sha256':file_hash(receipt)},
            '/docs-successor', 'successor publication membership/mode changed')
    endpoint = DocsSuccessor(request,component,state.binding,state.before,state.after,receipt,file_hash(receipt),
                             file_hash(component.receipt),(*history,anchor(output)))
    assert state.source_check is not None
    proof = _DocsProof(endpoint,request,state.source,state.bindings,state.files,state.source_check)
    _docs_history(proof)
    require(generated(request.prepared) == state.after, '/docs-successor', 'late generated mutation')
    state.process_anchor.recheck()
    for item in endpoint.history: item.recheck()
    proof.source_check(request.builds)
    _docs_raw(proof,state.after)
    _DOCS_ISSUED[id(endpoint)] = proof
    return endpoint


def run_docs_successor(request: DocsSuccessorRequest) -> DocsSuccessor:
    """Run original14 once with its existing owner; issue only generated/history authority.

    The legacy receipt is in the sibling ``<output>-static`` directory. A settled
    docs oracle FAIL remains a typed failure, with a successor only if the independent
    transition checks complete. UNKNOWN retains the actual owner and caller descriptor.
    """
    state = _DocsExecution(successor_request=request)
    try:
        state.source_check = request.source_closure.check
        state.source_check(request.builds)
        state.files = _docs_files(request)
        state.history = (*request.predecessor.history,
                         *(anchor(path.parent) for path,_ in request.environment.prior_receipts))
        _docs_admit(request)
        real_path(request.output.parent,directory=True)
        require(request.output.is_absolute() and request.output == request.output.resolve()
                and not request.output.exists() and not request.output.is_symlink(), '/docs-successor', 'stale output')
        selectors = request.builds.prepared.selectors
        for protected in (request.product.root,request.prepared.inputs.settings,request.prepared.inputs.cargo_home,
                          selectors.base.root,selectors.base.cargo_home,selectors.harness_root,selectors.output,
                          *(item.root for item in state.history)):
            for output in (request.output,request.static_output):
                require(not output.is_relative_to(protected) and not protected.is_relative_to(output),
                        '/docs-successor', 'output overlap')
        require(generated(request.prepared) == request.predecessor.after,
                '/docs-successor', 'stale predecessor current state')
    except BaseException as error:
        raise DocsSettledFailure(error,None,None) from error
    try:
        component = _owned_docs(request.product,request.static_output,request.environment,state)
    except DocsOwnershipUnknown as error:
        state.after = None
        error.execution = state
        raise
    except DocsSettledFailure as failure:
        if state.oracle_failed and not state.integrity_failed and failure.receipt == request.static_output/'receipt.json':
            try:
                failure.successor = _finish_docs(state,ComponentResult(request.static_output/'receipt.json','FAIL'))
            except BaseException as secondary:
                raise DocsSettledFailure(failure.original,secondary,failure.receipt) from secondary
        raise
    try:
        return _finish_docs(state,component)
    except BaseException as error:
        raise DocsSettledFailure(error,None,component.receipt) from error
