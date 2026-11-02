# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_setup_prepare
"""Read-only setup preparation. Checkpoints describe chronology, never build authority."""
import configparser
import os
import re
import shutil
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal

from generic_build import (
    BuildSpec,
    WorkspaceBuildPair,
    config_inventory,
    recheck_workspace_pair,
)
from generic_gate import GateContractError, document, require
from generic_process import check_digest, file_hash, real_path
from generic_sources import freeze_source, inventory, manifest_hash
from mux2_gate_inputs import Json, Node

BRIDGE: Final = 'rust/target/debug/libsim_bridge.dylib'
LIBRARIES: Final = (
    ('linux.debug.x86_64', '"res://rust/target/debug/libsim_bridge.so"'),
    ('linux.release.x86_64', '"res://rust/target/release/libsim_bridge.so"'),
    ('linux.debug.arm64', '"res://rust/target/debug/libsim_bridge.so"'),
    ('linux.release.arm64', '"res://rust/target/release/libsim_bridge.so"'),
    ('macos.debug', '"res://rust/target/debug/libsim_bridge.dylib"'),
    ('macos.release', '"res://rust/target/release/libsim_bridge.dylib"'),
    ('windows.debug.x86_64', '"res://rust/target/debug/sim_bridge.dll"'),
    ('windows.release.x86_64', '"res://rust/target/release/sim_bridge.dll"'),
)
IMPORT: Final = 'bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --import --editor --quit'
COMMANDS: Final = ((2, 'bridge_build', 'cd rust && cargo build -p sim-bridge'),
                  (3, 'godot_import', IMPORT+'; '+IMPORT),
                  (6, 'clippy', 'cd rust && cargo clippy --workspace --all-targets -- -D warnings'))
WORKSPACE_LINES: Final = (
    "step 4 test_debug bash -c 'cd rust && cargo test --workspace --no-fail-fast -- --test-threads=8'",
    "step 5 test_release bash -c 'cd rust && cargo test --workspace --release --no-fail-fast -- --test-threads=8'",
)
PHASES: Final = ('initial', 'after2', 'after3_first', 'after3_second', 'after4', 'after5', 'after6')
TOOLS: Final = frozenset({'bash', 'python3', 'cargo', 'rustc', 'cargo-clippy', 'clippy-driver', 'godot'})


@dataclass(frozen=True, slots=True)
class Tool:
    name: str
    path: Path
    sha256: str


@dataclass(frozen=True, slots=True)
class SetupInputs:
    root: Path
    source_sha: str
    cargo_home: Path
    tools: tuple[Tool, ...]
    environment: tuple[tuple[str, str], ...]
    effective_inputs: tuple[tuple[str, str], ...]
    configs: tuple[tuple[str, str], ...]
    lock_origin: Path
    lock_origin_sha256: str
    settings: Path
    output: Path


@dataclass(frozen=True, slots=True)
class PreparedSetup:
    inputs: SetupInputs
    source_digest: str
    materialized: tuple[tuple[str, str], ...]
    external: tuple[tuple[Path, str], ...]
    config_candidates: tuple[str, ...]
    status: Literal['PREPARED_ONLY'] = 'PREPARED_ONLY'
    execution_authority: Literal[False] = False
    producer_authentication: Literal['NOT_RUN'] = 'NOT_RUN'
    runtime_closure: Literal['CALLER_OWNED_UNQUALIFIED'] = 'CALLER_OWNED_UNQUALIFIED'
    warm_pair: WorkspaceBuildPair | None = None
    preparation_kind: Literal['FRESH', 'WARM_BUILD_ANTECEDENTS'] = 'FRESH'
    environment_delta: tuple[tuple[str, str | None, str | None], ...] = ()
    environment_equal: Literal[False] = False


@dataclass(frozen=True, slots=True)
class SetupRecipe:
    step: Literal[2, 3, 6]
    argv: tuple[str, ...]
    cwd: Path
    occurrences: tuple[str, ...]
    timeout_seconds: int | None


def import_exit(first: int | None, second: int | None) -> int | None:
    """Original aggregate only; caller must retain both raw outcomes separately."""
    return second if first is not None and second is not None else None


def absent(path: Path) -> None:
    require(not path.exists() and not path.is_symlink(), str(path), 'expected absent path')


def prepare(inputs: SetupInputs) -> PreparedSetup:
    return _prepare(inputs, None)


def prepare_warm(inputs: SetupInputs, pair: WorkspaceBuildPair) -> PreparedSetup:
    """Admit live build antecedents only; actual step2 and runtime remain unqualified."""
    return _prepare(inputs, pair)


def warm_projection(inputs: SetupInputs, pair: WorkspaceBuildPair) -> tuple[tuple[str, str | None, str | None], ...]:
    specs = recheck_workspace_pair(pair)
    tools = {tool.name: tool for tool in inputs.tools}
    env = dict(inputs.environment)
    require(all(key in env and env[key] == '' for key in ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER')),
            '/warm/environment', 'both wrappers must be explicitly present-empty')
    require(env.get('LANG') == env.get('LC_ALL') == 'C', '/warm/environment', 'build locale differs')
    jobs: list[int] = []
    for name, _ in inputs.configs:
        config = Node(tomllib.loads(Path(name).read_text())).mapping()
        require(config.keys() <= {'build', 'net'}, '/warm/config', 'unreviewed Cargo config table')
        if 'build' in config:
            build = Node(config['build']).mapping()
            require(build.keys() <= {'jobs'}, '/warm/config', 'unreviewed build config')
            if 'jobs' in build:
                count = build['jobs']
                require(type(count) is int and count > 0 and str(count) == env.get('CARGO_BUILD_JOBS'),
                        '/warm/jobs', 'explicit config/setup jobs differ')
                assert isinstance(count, int)
                jobs.append(count)
        if 'net' in config: require(Node(config['net']).mapping() == {'offline': True}, '/warm/config', 'unreviewed net config')
    require(bool(jobs), '/warm/jobs', 'default antecedent jobs are not qualified')
    actuals: list[dict[str, str]] = []
    for spec, result in zip(specs, (pair.debug, pair.release), strict=True):
        require(result is not None and spec.root == inputs.root and spec.source_sha == inputs.source_sha
                and spec.cargo_home == inputs.cargo_home and spec.overlay is None
                and dict(spec.config_inputs) == dict(inputs.configs), '/warm', 'foreign source/config/build kind')
        assert result is not None
        require(all(name in tools and (tools[name].path, tools[name].sha256) == (path, checksum)
                    for name, path, checksum in (('cargo', spec.cargo, spec.cargo_sha256), ('rustc', spec.rustc, spec.rustc_sha256))),
                '/warm/tools', 'build/setup tool identity differs')
        for fresh in (inputs.output, inputs.settings):
            for previous in (spec.output, pair.receipt.parent):
                require(not fresh.is_relative_to(previous) and not previous.is_relative_to(fresh), '/warm/paths', 'antecedent evidence overlap')
        row = document(result.receipt.read_text())
        previous_inputs = row.field('effective_inputs_before').mapping()
        require(all(dict(inputs.effective_inputs).get(name) == Node(value, name).text() for name, value in previous_inputs.items()),
                '/warm/inputs', 'antecedent effective input differs')
        process = document(Path(row.field('process_receipt').text()).read_text())
        actual = {name: Node(value, name).text() for name, value in process.field('environment').mapping().items()}
        expected = {'PATH': f'{spec.rustc.parent}:/usr/bin:/bin:/usr/sbin:/sbin', 'LANG': 'C', 'LC_ALL': 'C',
                    'CARGO_HOME': str(spec.cargo_home), 'CARGO_TARGET_DIR': str(inputs.root/'rust/target'),
                    'RUSTC': str(spec.rustc), 'RUSTC_WRAPPER': '', 'RUSTC_WORKSPACE_WRAPPER': ''}
        require(actual == expected, '/warm/environment', 'unreviewed actual antecedent environment')
        actuals.append(actual)
    require(actuals[0] == actuals[1], '/warm/environment', 'profile environments differ')
    before = actuals[0]
    if 'CARGO_NET_OFFLINE' in env:
        require(any(Node(tomllib.loads(Path(name).read_text())).mapping().get('net') == {'offline': True}
                    for name, _ in inputs.configs), '/warm/environment', 'offline presence lacks shared config')
    _ = recheck_workspace_pair(pair)
    return tuple((key, before.get(key), env.get(key)) for key in sorted(before.keys() | env.keys()) if before.get(key) != env.get(key))


def _prepare(inputs: SetupInputs, pair: WorkspaceBuildPair | None) -> PreparedSetup:
    """Bind clean current source/config/inputs without creating or executing anything."""
    delta = warm_projection(inputs, pair) if pair is not None else ()
    root = inputs.root
    real_path(root, directory=True)
    real_path(inputs.cargo_home, directory=True)
    source = freeze_source(root, inputs.source_sha)
    tracked = Node(source).field('tracked_manifest').mapping()
    required_source = {'tools/v8/gate.sh', 'tools/v8/run_bounded.sh', 'rust/rust-toolchain.toml',
                       'rust/crates/sim-bridge/Cargo.toml', 'sim_bridge.gdextension', 'project.godot'}
    require(required_source <= tracked.keys(), '/source', 'recipe/product source must be tracked')
    effective = dict(inputs.effective_inputs)
    require(len(effective) == len(inputs.effective_inputs) and tracked.keys() <= effective.keys(),
            '/inputs', 'complete unique tracked/effective input inventory required')
    require(inventory(root, list(effective)) == effective, '/inputs', 'effective input changed')
    require(not any(name.startswith(('rust/target/', '.godot/')) for name in effective),
            '/inputs', 'mutable outputs cannot be immutable inputs')
    for name in (('.godot', 'override.cfg', 'addons/godot_test_mcp') if pair is not None else ('rust/target', '.godot', 'override.cfg', 'addons/godot_test_mcp')):
        absent(root/name)
    for path in (inputs.output, inputs.settings):
        require(path.is_absolute() and path.parent == path.parent.resolve(strict=True)
                and not path.is_relative_to(root), str(path), 'fresh canonical external path required')
        absent(path)
    require(not inputs.output.is_relative_to(inputs.settings) and not inputs.settings.is_relative_to(inputs.output),
            '/paths', 'settings/evidence overlap')
    tools = {tool.name: tool for tool in inputs.tools}
    require(frozenset(tools) == TOOLS and len(tools) == len(inputs.tools), '/tools', 'exact tool set required')
    for tool in inputs.tools:
        check_digest(tool.sha256, tool.name)
        require(file_hash(tool.path) == tool.sha256 and os.access(tool.path, os.X_OK), tool.name, 'tool changed/not executable')
    env = dict(inputs.environment)
    required = {'HOME', 'PATH', 'LANG', 'LC_ALL', 'CARGO_HOME', 'CARGO_BUILD_JOBS', 'GODOT_BIN',
                'WORLDSIM_TEST_SETTINGS_DIR', 'PYTHONNOUSERSITE', 'PYTHONDONTWRITEBYTECODE'}
    require(len(env) == len(inputs.environment) and required <= env.keys()
            and env.keys() <= required | {'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_NET_OFFLINE', '__CF_USER_TEXT_ENCODING', 'PYTHONCOERCECLOCALE'},
            '/environment', 'missing/duplicate/undeclared environment key')
    require(env['CARGO_HOME'] == str(inputs.cargo_home) and env['GODOT_BIN'] == str(tools['godot'].path)
            and env['WORLDSIM_TEST_SETTINGS_DIR'] == str(inputs.settings)
            and env['PYTHONNOUSERSITE'] == env['PYTHONDONTWRITEBYTECODE'] == '1'
            and re.fullmatch('[1-9][0-9]*', env['CARGO_BUILD_JOBS']) is not None,
            '/environment', 'environment binding mismatch')
    require(all(env.get(key, '') == '' for key in ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'))
            and env.get('CARGO_NET_OFFLINE', 'true') == 'true', '/environment', 'unreviewed wrapper/network mode')
    real_path(Path(env['HOME']), directory=True)
    require(bool(env['LANG']) and bool(env['LC_ALL']), '/environment', 'empty locale')
    for directory in env['PATH'].split(os.pathsep):
        real_path(Path(directory), directory=True)
    for name in TOOLS-{'godot'}:
        found = shutil.which(name, path=env['PATH'])
        require(found is not None and Path(found).resolve(strict=True) == tools[name].path, name, 'PATH tool mismatch')
    spec = BuildSpec(root, inputs.source_sha, tools['cargo'].path, tools['cargo'].sha256,
                     tools['rustc'].path, tools['rustc'].sha256, inputs.cargo_home, inputs.output,
                     (), (), config_inputs=inputs.configs)
    configs = config_inventory(spec)
    candidates = tuple(sorted({str(parent/'.cargo'/name) for parent in (root/'rust', *(root/'rust').parents)
                               for name in ('config', 'config.toml')} |
                              {str(inputs.cargo_home/name) for name in ('config', 'config.toml')}))
    require('rust/Cargo.lock' in effective, '/lock', 'ignored lock must be explicitly bound')
    require(file_hash(inputs.lock_origin) == inputs.lock_origin_sha256, '/lock', 'origin receipt changed')
    origin = document(inputs.lock_origin.read_text())
    origin.field('path').equals(str(root/'rust/Cargo.lock'))
    origin.field('sha256').equals(effective['rust/Cargo.lock'])
    # This binds the supplied origin record; it does not certify its upstream authority.
    gate = (root/'tools/v8/gate.sh').read_text().splitlines()
    for number, name, command in COMMANDS:
        expected = f"step {number} {name} bash -c '{command}'"
        require(gate.count(expected) == 1 and sum(line.startswith(f'step {number} ') for line in gate) == 1,
                '/recipe', 'original command source changed/duplicated')
    for number, line in enumerate(WORKSPACE_LINES, 4):
        require(gate.count(line) == 1 and sum(row.startswith(f'step {number} ') for row in gate) == 1,
                '/recipe', 'original intermediate workspace command changed')
    ordered = [f"step {n} {name} bash -c '{command}'" for n, name, command in COMMANDS]
    ordered[2:2] = WORKSPACE_LINES
    positions = [gate.index(line) for line in ordered]
    require(positions == sorted(positions), '/recipe', 'source order changed')
    channel = Node(tomllib.loads((root/'rust/rust-toolchain.toml').read_text())).field('toolchain').field('channel').text()
    require(re.fullmatch(r'[0-9]+\.[0-9]+(?:\.[0-9]+)?', channel) is not None, '/toolchain', 'numeric channel required')
    manifest = Node(tomllib.loads((root/'rust/crates/sim-bridge/Cargo.toml').read_text()))
    manifest.field('package').field('name').equals('sim-bridge')
    manifest.field('lib').field('crate-type').equals(['cdylib', 'rlib'])
    extension = configparser.ConfigParser(interpolation=None, strict=True)
    extension.optionxform = lambda optionstr: optionstr
    try:
        extension.read_string((root/'sim_bridge.gdextension').read_text())
    except configparser.Error as error:
        raise GateContractError('EXTENSION_SOURCE', '/bridge', str(error)) from error
    require(not extension.defaults() and extension.has_option('libraries', 'macos.debug'),
            '/bridge', 'missing or ambiguous source debug bridge mapping')
    require(dict(extension.items('libraries')) == dict(LIBRARIES),
            '/bridge', 'source debug bridge path or reviewed library selector map changed')
    external = {Path(name): digest for name, digest in configs.items() if not Path(name).is_relative_to(root)}
    external.update({tool.path: tool.sha256 for tool in inputs.tools})
    external[inputs.lock_origin] = inputs.lock_origin_sha256
    for path, digest in external.items():
        require(not path.is_relative_to(root), str(path), 'external input belongs in materialized source')
        require(file_hash(path) == digest, str(path), 'external input changed')
    require(all(effective.get(Path(name).relative_to(root).as_posix()) == digest
                for name, digest in configs.items() if Path(name).is_relative_to(root)), '/configs', 'in-tree config unbound')
    require(config_inventory(spec) == configs and freeze_source(root, inputs.source_sha) == source
            and inventory(root, list(effective)) == effective, '/inputs', 'inputs changed during preparation')
    for path, digest in external.items():
        require(file_hash(path) == digest, str(path), 'external input changed at final check')
    for path in (inputs.output, inputs.settings, root/'.godot'):
        absent(path)
    if pair is None: absent(root/'rust/target')
    else: require(warm_projection(inputs, pair) == delta, '/warm', 'antecedent projection changed')
    return PreparedSetup(inputs, manifest_hash(source), tuple(sorted(effective.items())), tuple(sorted(external.items())), candidates,
                         warm_pair=pair, preparation_kind='FRESH' if pair is None else 'WARM_BUILD_ANTECEDENTS', environment_delta=delta)


def recipe(prepared: PreparedSetup, step: Literal[2, 3, 6]) -> SetupRecipe:
    """Recheck the preparation, then return a declaration, not a launch grant."""
    require(_prepare(prepared.inputs, prepared.warm_pair) == prepared, '/preparation', 'stale preparation')
    selected = [command for number, _, command in COMMANDS if number == step]
    require(len(selected) == 1, '/step', 'unsupported step')
    bash = next(tool.path for tool in prepared.inputs.tools if tool.name == 'bash')
    return SetupRecipe(step, (str(bash), '-c', selected[0]), prepared.inputs.root,
                       ('import.first', 'import.second') if step == 3 else (f'step{step}',), 600 if step == 3 else None)


@dataclass(frozen=True, slots=True)
class TargetCheckpoint:
    root: Path
    source_sha: str
    phase: str
    predecessor_sha256: str
    inventory_sha256: str
    bridge_sha256: str | None

    def digest(self) -> str:
        value: dict[str, Json] = {'root': str(self.root), 'source_sha': self.source_sha, 'phase': self.phase,
                                'predecessor': self.predecessor_sha256, 'inventory': self.inventory_sha256,
                                'bridge': self.bridge_sha256}
        return manifest_hash(value)


def validate_transition_shape(before: TargetCheckpoint, after: TargetCheckpoint) -> Literal['PRODUCER_AUTHENTICATION_REQUIRED']:
    """Validate only closed chronology; caller inventories confer no production authority."""
    for point in (before, after):
        real_path(point.root, directory=True)
        require(re.fullmatch('[0-9a-f]{40}', point.source_sha) is not None, '/source', 'invalid checkpoint source SHA')
        require(point.phase != 'initial' or (point.bridge_sha256 is None and point.predecessor_sha256 == '0'*64),
                '/initial', 'initial checkpoint requires no bridge and zero predecessor')
        require(point.phase in PHASES, '/phase', 'unknown checkpoint')
        check_digest(point.inventory_sha256, '/inventory')
        check_digest(point.predecessor_sha256, '/predecessor')
        if point.bridge_sha256 is not None:
            check_digest(point.bridge_sha256, '/bridge')
    require(before.root == after.root and before.source_sha == after.source_sha
            and after.predecessor_sha256 == before.digest()
            and PHASES.index(after.phase) == PHASES.index(before.phase)+1, '/chronology', 'invalid transition')
    return 'PRODUCER_AUTHENTICATION_REQUIRED'
