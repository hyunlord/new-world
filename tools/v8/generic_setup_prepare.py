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

from generic_build import BuildSpec, config_inventory
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
    """Bind clean current source/config/inputs without creating or executing anything."""
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
    for name in ('rust/target', '.godot', 'override.cfg', 'addons/godot_test_mcp'):
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
            and env.keys() <= required | {'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_NET_OFFLINE'},
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
        require(shutil.which(name, path=env['PATH']) == str(tools[name].path), name, 'PATH tool mismatch')
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
    for path in (inputs.output, inputs.settings, root/'rust/target', root/'.godot'):
        absent(path)
    return PreparedSetup(inputs, manifest_hash(source), tuple(sorted(effective.items())), tuple(sorted(external.items())), candidates)


def recipe(prepared: PreparedSetup, step: Literal[2, 3, 6]) -> SetupRecipe:
    """Recheck the preparation, then return a declaration, not a launch grant."""
    require(prepare(prepared.inputs) == prepared, '/preparation', 'stale preparation')
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
