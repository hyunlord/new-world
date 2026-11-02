# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_build.py
"""Fresh-target build evidence; observer-derived products remain explicitly derived.

This authenticates endpoint inputs and actual build outputs, not hermetic execution
or bit reproducibility. Caller owns worktrees and semantic overlay qualification.
"""
import hashlib
import json
import os
import re
import stat
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal

from generic_gate import GateContractError, document, require
from generic_process import ProcessSpec, atomic_json, execute, file_hash, real_path
from generic_sources import freeze_source, git, inventory, manifest_hash, safe_path
from generic_workspace_tests import inspect_inventory
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Json, Node

# These are the reviewed full-suite and normal-boot lifecycle seams, not a
# blanket permission to overlay gameplay scripts or arbitrary integration tests.
OBSERVER_EXACT_PATHS: Final = frozenset({
    'scripts/core/main_bootstrap.gd',
    'rust/crates/sim-test/tests/save_load.rs',
    'rust/crates/sim-test/tests/save_gate_observation/mod.rs',
    'rust/crates/sim-test/tests/colony_world.rs',
    'rust/crates/sim-test/tests/colony_commands.rs',
    'rust/crates/sim-test/tests/colony_jobs.rs',
    'rust/crates/sim-test/tests/colony_steward.rs',
    'rust/crates/sim-test/tests/colony_steward_housing/construction.rs',
    'rust/crates/sim-test/tests/steward_defence.rs',
    'rust/crates/sim-test/tests/legacy_migration_current.rs',
    'rust/crates/sim-test/tests/steward_defence_migration.rs',
    'rust/crates/sim-test/tests/b5_flora.rs',
    'rust/crates/sim-test/tests/engine_appearance_ffi.rs',
    'rust/crates/sim-test/tests/save_gate_observation/external.rs',
})


@dataclass(frozen=True, slots=True)
class Overlay:
    patch: Path
    patch_sha256: str
    manifest: Path
    manifest_sha256: str


@dataclass(frozen=True, slots=True)
class Artifact:
    name: str
    relative_path: str


@dataclass(frozen=True, slots=True)
class BuildSpec:
    root: Path
    source_sha: str
    cargo: Path
    cargo_sha256: str
    rustc: Path
    rustc_sha256: str
    cargo_home: Path
    output: Path
    packages: tuple[str, ...]
    artifacts: tuple[Artifact, ...]
    features: tuple[str, ...] = ()
    profile: Literal['release', 'debug'] = 'release'
    overlay: Overlay | None = None
    config_inputs: tuple[tuple[str, str], ...] = ()
    test_targets: tuple[str, ...] = ()
    workspace_tests: bool = False


@dataclass(frozen=True, slots=True)
class BuildResult:
    receipt: Path
    status: Literal['PASS', 'FAIL']
    artifacts: tuple[tuple[str, Path, str], ...]


def config_inventory(spec: BuildSpec) -> dict[str, str]:
    """Bind Cargo's ancestor and CARGO_HOME config files, without reading credentials."""
    paths = {parent / '.cargo' / name for parent in (spec.root / 'rust', *(spec.root / 'rust').parents)
             for name in ('config', 'config.toml')}
    paths.update(spec.cargo_home / name for name in ('config', 'config.toml'))
    actual = {str(path): file_hash(path) for path in paths if path.exists() or path.is_symlink()}
    require(len(spec.config_inputs) == len(dict(spec.config_inputs)), '/cargo-config', 'duplicate config binding')
    require(actual == dict(spec.config_inputs), '/cargo-config', 'unbound or changed Cargo config')
    return actual


def overlay_files(spec: BuildSpec) -> dict[str, tuple[str | None, str]]:
    if spec.overlay is None:
        return {}
    overlay = spec.overlay
    require(file_hash(overlay.patch) == overlay.patch_sha256 and
            file_hash(overlay.manifest) == overlay.manifest_sha256, '/overlay', 'overlay artifact hash mismatch')
    node = document(overlay.manifest.read_text())
    node.field('schema_version').equals(1)
    node.field('product_base_sha').equals(spec.source_sha)
    node.field('observer_patch_sha256').equals(overlay.patch_sha256)
    allowed = [row.text() for row in node.field('allowed_paths').array()]
    require(bool(allowed) and len(set(allowed)) == len(allowed), '/overlay', 'empty or duplicate paths')
    rows: dict[str, tuple[str | None, str]] = {}
    for row in node.field('files').array():
        name = row.field('path').text()
        require(name not in rows and name in allowed, '/overlay', 'unlisted or duplicate file')
        require(name in OBSERVER_EXACT_PATHS or name.startswith(
                    ('rust/crates/sim-test/src/', 'rust/crates/sim-bridge/src/', 'tools/v8/', 'docs/v8/')),
                name, 'observer compatibility path outside tooling/bridge scope')
        before = row.field('base0_sha256').value
        after = row.field('observer_sha256').text()
        require(before is None or (isinstance(before, str) and re.fullmatch('[0-9a-f]{64}', before) is not None),
                name, 'invalid overlay preimage')
        require(re.fullmatch('[0-9a-f]{64}', after) is not None and before != after,
                name, 'invalid or unchanged overlay postimage')
        row.field('operation').equals('add' if before is None else 'modify')
        path = safe_path(spec.root, name)
        require((not path.exists() and not path.is_symlink()) if before is None else file_hash(path) == before,
                name, 'overlay preimage differs')
        if before is not None and not isinstance(before, str):
            raise GateContractError('OVERLAY', name, 'invalid preimage')
        rows[name] = (before, after)
    require(set(rows) == set(allowed), '/overlay', 'manifest path coverage differs')
    changed_paths: set[str] = set()
    for raw in git(spec.root, 'apply', '--numstat', '-z', str(overlay.patch)).split(b'\0'):
        if raw:
            fields = raw.split(b'\t', 2)
            require(len(fields) == 3 and bool(fields[2]), '/overlay', 'unsupported patch path encoding')
            name = os.fsdecode(fields[2])
            require(name not in changed_paths, '/overlay', 'duplicate patch path')
            changed_paths.add(name)
    require(changed_paths == set(rows), '/overlay', 'actual patch paths differ from approved manifest')
    _ = git(spec.root, 'apply', '--check', str(overlay.patch))
    _ = git(spec.root, 'apply', str(overlay.patch))
    return rows


def derived_snapshot(root: Path, source: dict[str, Json], changed: dict[str, tuple[str | None, str]],
                     index_before: bytes) -> dict[str, Json]:
    """No dirty-source exception: every original blob and listed postimage is checked."""
    require(git(root, 'rev-parse', 'HEAD').decode().strip() == source['source_sha'], str(root), 'source HEAD drift')
    require(git(root, 'ls-files', '--stage', '-z') == index_before, str(root), 'index drift')
    original = document(json.dumps(source)).field('tracked_manifest').mapping()
    expected: dict[str, tuple[str, str]] = {}
    for name, raw in original.items():
        fields = document(json.dumps({'row': raw})).field('row').array()
        require(len(fields) == 3, name, 'invalid source manifest row')
        mode, digest = fields[0].text(), fields[2].text()
        expected[name] = (mode, changed[name][1] if name in changed else digest)
    for name, (before, after) in changed.items():
        require((name in original) == (before is not None), name, 'overlay add/modify mismatch')
        if before is None:
            expected[name] = ('100644', after)
    manifest: dict[str, Json] = {}
    for name, (mode, digest) in sorted(expected.items()):
        path = safe_path(root, name)
        info = path.lstat()
        if mode == '120000':
            require(stat.S_ISLNK(info.st_mode), name, 'tracked symlink changed')
            actual = hashlib.sha256(os.fsencode(os.readlink(path))).hexdigest()
        else:
            require(stat.S_ISREG(info.st_mode) and bool(info.st_mode & 0o111) == (mode == '100755'),
                    name, 'tracked mode changed')
            actual = file_hash(path)
        require(actual == digest, name, 'source differs from committed or authenticated overlay bytes')
        manifest[name] = [mode, actual]
    untracked = {os.fsdecode(name) for name in git(root, 'ls-files', '--others', '--exclude-standard', '-z').split(b'\0') if name}
    require(untracked == {name for name, (before, _) in changed.items() if before is None},
            str(root), 'unregistered untracked source')
    return {'manifest': manifest, 'sha256': manifest_hash(manifest)}


def build_product(spec: BuildSpec) -> BuildResult:
    """Run a fully explicit Cargo recipe into a new target; retain failed build evidence."""
    real_path(spec.root, directory=True)
    real_path(spec.cargo_home, directory=True)
    require(spec.profile in {'release', 'debug'}, '/profile', 'unsupported profile')
    for values, label in ((spec.packages, 'packages'), (spec.features, 'features')):
        require(len(set(values)) == len(values) and all(re.fullmatch('[A-Za-z0-9_/-]+', x) for x in values),
                label, 'invalid or duplicate Cargo selector')
    require(type(spec.workspace_tests) is bool, '/build', 'workspace mode must be boolean')
    if spec.workspace_tests:
        require(not (spec.packages or spec.features or spec.artifacts or spec.test_targets),
                '/build', 'workspace mode excludes package/feature/target narrowing and normal artifacts')
    else:
        require(bool(spec.packages) and bool(spec.artifacts or spec.test_targets), '/build', 'empty packages or artifacts')
    require(not (spec.artifacts and spec.test_targets), '/build', 'test and normal artifacts require separate builds')
    require(len(set(spec.test_targets)) == len(spec.test_targets)
            and all(re.fullmatch('[A-Za-z0-9_-]+', name) for name in spec.test_targets),
            '/test-targets', 'invalid or duplicate integration test target')
    require(len({a.name for a in spec.artifacts}) == len(spec.artifacts), '/artifacts', 'duplicate artifact name')
    require(len({a.relative_path for a in spec.artifacts}) == len(spec.artifacts), '/artifacts', 'duplicate artifact path')
    target = spec.root / 'rust/target'
    require(not target.exists() and not target.is_symlink(), str(target), 'fresh independent target required')
    require(spec.output.is_absolute() and spec.output.parent.resolve() == spec.output.parent
            and not spec.output.is_relative_to(spec.root), str(spec.output), 'fresh external evidence root required')
    for artifact in spec.artifacts:
        require(re.fullmatch('[A-Za-z0-9_-]+', artifact.name) is not None, '/artifacts', 'invalid target name')
        require(bool(Path(artifact.relative_path).parts) and Path(artifact.relative_path).parts[0] == spec.profile
                and '..' not in Path(artifact.relative_path).parts and not Path(artifact.relative_path).is_absolute(),
                '/artifacts', 'artifact outside selected profile')
    source = freeze_source(spec.root, spec.source_sha)
    index_before = git(spec.root, 'ls-files', '--stage', '-z')
    configs = config_inventory(spec)
    require(file_hash(spec.cargo) == spec.cargo_sha256 and file_hash(spec.rustc) == spec.rustc_sha256,
            '/toolchain', 'tool executable hash mismatch')
    spec.output.mkdir(exist_ok=False)
    receipt: dict[str, Json] = {'schema_version': 1, 'source_sha': spec.source_sha,
        'root': str(spec.root), 'profile': spec.profile, 'packages': list(spec.packages),
        'features': list(spec.features), 'source_before': source, 'target_dir': str(target),
        'source_kind': 'PRISTINE_COMMIT' if spec.overlay is None else 'DERIVED_OBSERVER',
        'observer_semantic_qualification': 'NOT_RUN' if spec.overlay else 'NOT_APPLICABLE',
        'status': 'FAIL', 'exit_code': None, 'publication_authority': False,
        'cargo_sha256': spec.cargo_sha256, 'rustc_sha256': spec.rustc_sha256, 'cargo_configs': {key: value for key, value in configs.items()}}
    if spec.test_targets:
        receipt.update(build_kind='integration_tests', test_targets=list(spec.test_targets))
    if spec.workspace_tests:
        receipt.update(build_kind='workspace_tests', workspace_runtime='NOT_RUN',
                       workspace_world_safety='INCOMPLETE', workspace_environment_equivalence='NOT_RUN',
                       workspace_doctest_execution='NOT_RUN')
    artifacts: list[tuple[str, Path, str]] = []
    workspace_targets: dict[str,Json] = {}
    workspace_evidence: dict[str,str] = {}
    metadata: Node | None = None
    changed: dict[str, tuple[str | None, str]] = {}
    before: dict[str, Json] | None = None
    inputs: dict[str, str] | None = None
    try:
        changed = overlay_files(spec)
        receipt['overlay'] = None if spec.overlay is None else {
            'patch': str(spec.overlay.patch), 'patch_sha256': spec.overlay.patch_sha256,
            'manifest': str(spec.overlay.manifest), 'manifest_sha256': spec.overlay.manifest_sha256}
        before = derived_snapshot(spec.root, source, changed, index_before)
        receipt['effective_source_before'] = before
        scopes = [*INPUT_DIRS, *INPUT_FILES]
        inputs = inventory(spec.root, scopes)
        receipt['effective_inputs_before'] = {key: value for key, value in inputs.items()}
        environment = (('PATH', f'{spec.rustc.parent}:/usr/bin:/bin:/usr/sbin:/sbin'),
                       ('CARGO_HOME', str(spec.cargo_home)), ('CARGO_TARGET_DIR', str(target)),
                       ('RUSTC', str(spec.rustc)), ('RUSTC_WRAPPER', ''), ('RUSTC_WORKSPACE_WRAPPER', ''))
        toolchain = document(json.dumps(tomllib.loads((spec.root / 'rust/rust-toolchain.toml').read_text())))
        channel = toolchain.field('toolchain').field('channel').text()
        require(re.fullmatch(r'[0-9]+\.[0-9]+(?:\.[0-9]+)?', channel) is not None,
                '/toolchain', 'build requires a pinned numeric source toolchain')
        versions: dict[str, Json] = {}
        for name, executable, digest in (('cargo', spec.cargo, spec.cargo_sha256), ('rustc', spec.rustc, spec.rustc_sha256)):
            result = execute(ProcessSpec((str(executable), '--version', '--verbose'), spec.root / 'rust',
                             executable, digest, spec.root, tuple(sorted(inputs.items())),
                             spec.output / (name + '-version'), environment))
            require(result.status == 'PASS', name, 'toolchain version command failed')
            version_file = result.receipt.parent / 'stdout.log'
            version = version_file.read_text()
            require(bool(version.strip()), name, 'empty toolchain version')
            releases = [match.group(1) for match in re.finditer(r'^release: ([0-9]+\.[0-9]+\.[0-9]+)$', version, re.MULTILINE)]
            require(len(releases) == 1 and (releases[0] == channel or releases[0].startswith(channel + '.')),
                    name, 'actual executable version differs from pinned source toolchain')
            versions[name] = {'text': version, 'process_receipt': str(result.receipt),
                              'process_receipt_sha256': file_hash(result.receipt)}
            if spec.workspace_tests:
                workspace_evidence.update(inventory(spec.output,[name+'-version']))
        receipt['toolchain_versions'] = versions
        if spec.workspace_tests:
            metadata_argv = (str(spec.cargo),'metadata','--locked','--format-version=1','--no-deps')
            metadata_result = execute(ProcessSpec(metadata_argv,spec.root/'rust',spec.cargo,spec.cargo_sha256,
                spec.root,tuple(sorted(inputs.items())),spec.output/'cargo-metadata',environment))
            receipt['metadata_process_receipt'] = str(metadata_result.receipt)
            receipt['metadata_receipt_sha256'] = file_hash(metadata_result.receipt)
            receipt['metadata_exit_code'] = metadata_result.native_exit
            require(metadata_result.status == 'PASS', '/metadata', 'metadata failed or source changed')
            metadata_path = metadata_result.receipt.parent/'stdout.log'
            receipt['metadata_sha256'] = file_hash(metadata_path)
            metadata = document(metadata_path.read_bytes().decode())
            metadata.field('version').equals(1)
            metadata.field('workspace_root').equals(str(spec.root/'rust'))
            metadata.field('target_directory').equals(str(target))
            for package in metadata.field('packages').array():
                names = [package.field('manifest_path').text(),
                         *(item.field('src_path').text() for item in package.field('targets').array())]
                for name in names:
                    path = Path(name)
                    real_path(path,directory=False)
                    require(path.is_relative_to(spec.root) and path.relative_to(spec.root).as_posix() in inputs,
                            name, 'metadata target/manifest absent from authenticated source inputs')
            workspace_evidence.update(inventory(spec.output,['cargo-metadata']))
        require(not target.exists() and not target.is_symlink(), str(target), 'target appeared before build')
        argv = [str(spec.cargo), 'test' if spec.test_targets or spec.workspace_tests else 'build', '--locked', '--message-format=json-render-diagnostics']
        if spec.workspace_tests:
            argv.extend(('--workspace','--no-run'))
        if spec.test_targets:
            argv.append('--no-run')
            for name in spec.test_targets:
                argv.extend(('--test', name))
        if spec.profile == 'release':
            argv.append('--release')
        for package in spec.packages:
            argv.extend(('-p', package))
        if spec.features:
            argv.extend(('--features', ','.join(spec.features)))
        result = execute(ProcessSpec(tuple(argv), spec.root / 'rust', spec.cargo, spec.cargo_sha256,
                         spec.root, tuple(sorted(inputs.items())), spec.output / 'cargo-build', environment))
        receipt.update(exit_code=result.native_exit, argv=[part for part in argv], process_receipt=str(result.receipt),
                       process_receipt_sha256=file_hash(result.receipt), environment={key: value for key, value in environment})
        require(result.status == 'PASS', '/cargo-build', 'native build failed or inputs changed')
        messages = [document(line) for line in (result.receipt.parent / 'stdout.log').read_text().splitlines()]
        terminals = [row for row in messages if row.field('reason').value == 'build-finished']
        require(len(terminals) == 1, '/cargo-build', 'missing or duplicate build terminal')
        terminals[0].field('success').equals(True)
        require(messages[-1] is terminals[0], '/cargo-build', 'build terminal not last')
        for artifact in spec.artifacts:
            path = safe_path(target, artifact.relative_path)
            matches = [row for row in messages if row.field('reason').value == 'compiler-artifact'
                       and row.field('target').field('name').value == artifact.name
                       and str(path) in [item.text() for item in row.field('filenames').array()]]
            require(len(matches) == 1, str(path), 'missing or duplicate artifact compiler receipt')
            matches[0].field('fresh').equals(False)
            artifacts.append((artifact.name, path, file_hash(path)))
        for name in spec.test_targets:
            matches = [row for row in messages if row.field('reason').value == 'compiler-artifact'
                       and row.field('target').field('name').value == name
                       and row.field('target').field('kind').value == ['test']]
            require(len(matches) == 1, name, 'missing or duplicate integration test compiler receipt')
            row = matches[0]
            row.field('fresh').equals(False)
            row.field('profile').field('test').equals(True)
            path = Path(row.field('executable').text())
            require(path.is_absolute() and path == path.resolve(strict=True)
                    and path.parent == target / spec.profile / 'deps', name, 'foreign integration test executable')
            require(str(path) in [item.text() for item in row.field('filenames').array()],
                    name, 'test executable absent from compiler filenames')
            require(path.is_file() and os.access(path, os.X_OK), name, 'test executable is not executable')
            require(path not in {prior for _, prior, _ in artifacts}, name, 'duplicate test executable')
            artifacts.append((name, path, file_hash(path)))
        if spec.workspace_tests:
            assert metadata is not None
            selected = inspect_inventory(metadata,tuple(messages),4 if spec.profile == 'debug' else 5)
            receipt['workspace_doctests'] = [
                {'package_id':row.package_id,'package':row.package,'name':row.name,'kinds':list(row.kinds),
                 'source':str(row.source),'manifest':str(row.manifest)} for row in selected.doctests]
            inodes: set[tuple[int,int]] = set()
            for executable in selected.executables:
                row = executable.target
                identity: dict[str,Json] = {'package_id':row.package_id,'package':row.package,
                    'name':row.name,'kinds':list(row.kinds),'source':str(row.source),
                    'manifest':str(row.manifest),'profile':spec.profile}
                name = 'workspace-'+hashlib.sha256(json.dumps(identity,sort_keys=True).encode()).hexdigest()
                path = executable.binary
                real_path(path,directory=False)
                require(os.access(path,os.X_OK),str(path),'workspace test artifact not executable')
                inode = (path.stat().st_dev,path.stat().st_ino)
                require(inode not in inodes, str(path), 'workspace test artifacts share inode')
                inodes.add(inode)
                workspace_targets[name] = identity
                artifacts.append((name,path,file_hash(path)))
            receipt['workspace_targets'] = workspace_targets
            workspace_evidence.update(inventory(spec.output,['cargo-build']))
        after = derived_snapshot(spec.root, source, changed, index_before)
        receipt['effective_source_after'] = after
        require(before == after, '/source', 'source changed during build')
        require(inventory(spec.root, scopes) == inputs, '/inputs', 'effective inputs changed during build')
        require(config_inventory(spec) == configs, '/configs', 'Cargo configs changed during build')
        require(file_hash(spec.cargo) == spec.cargo_sha256 and file_hash(spec.rustc) == spec.rustc_sha256,
                '/toolchain', 'toolchain changed during build')
        if spec.overlay is not None:
            require(file_hash(spec.overlay.patch) == spec.overlay.patch_sha256 and
                    file_hash(spec.overlay.manifest) == spec.overlay.manifest_sha256, '/overlay', 'overlay proof changed')
        if spec.workspace_tests:
            require(inventory(spec.output,['cargo-version','rustc-version','cargo-metadata','cargo-build']) == workspace_evidence,
                    '/workspace-evidence', 'earlier metadata/compiler evidence changed')
            receipt['workspace_evidence_sha256'] = {path:digest for path,digest in workspace_evidence.items()}
        receipt['artifacts'] = {name: {'path': str(path), 'sha256': digest} for name, path, digest in artifacts}
        receipt['binary_sha256'] = artifacts[0][2]
        receipt['status'] = 'PASS'
    except (OSError, ValueError, KeyError) as error:
        receipt['error'] = str(error)
    if receipt['status'] != 'PASS':
        try:
            receipt['effective_source_after'] = derived_snapshot(spec.root, source, changed, index_before)
            receipt['effective_inputs_after'] = {key: value for key, value in inventory(spec.root, [*INPUT_DIRS, *INPUT_FILES]).items()}
            receipt['cargo_configs_after'] = {key: value for key, value in config_inventory(spec).items()}
            receipt['cargo_sha256_after'] = file_hash(spec.cargo)
            receipt['rustc_sha256_after'] = file_hash(spec.rustc)
        except (OSError, ValueError, KeyError) as error:
            receipt['postflight_error'] = str(error)
    receipt['limits'] = ['Endpoint provenance, not sandboxed/hermetic execution or bit reproducibility.',
                         'Registry/dependency integrity relies on locked Cargo and its checksum verification.',
                         'Observer-only behavior must be qualified independently; no semantic claim follows from path/hash validation.',
                         'Caller must bind full gate, independent product arms and runtime safety.']
    terminal = spec.output / 'build.json'
    atomic_json(terminal, receipt)
    if receipt['status'] == 'PASS':
        for name, path, digest in artifacts:
            artifact_receipt: dict[str,Json] = {
                'source_sha': spec.source_sha, 'binary_sha256': digest, 'binary': str(path),
                'profile': spec.profile, 'exit_code': receipt['exit_code'], 'source_kind': receipt['source_kind'],
                'build_receipt': str(terminal), 'build_receipt_sha256': file_hash(terminal)}
            if spec.workspace_tests:
                artifact_receipt.update(build_kind='workspace_tests',workspace_target=workspace_targets[name],
                    metadata_process_receipt=receipt['metadata_process_receipt'],
                    metadata_receipt_sha256=receipt['metadata_receipt_sha256'],metadata_sha256=receipt['metadata_sha256'])
            atomic_json(spec.output / f'artifact-{name}.json', artifact_receipt)
    return BuildResult(terminal, 'PASS' if receipt['status'] == 'PASS' else 'FAIL', tuple(artifacts))
