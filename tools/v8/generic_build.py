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
from dataclasses import dataclass, replace
from pathlib import Path
from typing import TYPE_CHECKING, Literal

if TYPE_CHECKING:
    from generic_identity_overlay import IdentityContext

from generic_gate import GateContractError, document, require
from generic_overlay_contract import OBSERVER_EXACT_PATHS as OBSERVER_EXACT_PATHS
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    execute,
    file_hash,
    real_path,
)
from generic_sources import freeze_source, git, inventory, manifest_hash, safe_path
from generic_workspace_tests import inspect_inventory
from mux2_gate_inputs import INPUT_DIRS, INPUT_FILES, Json, Node


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


def overlay_files(spec: BuildSpec, identity: 'IdentityContext | None' = None) -> dict[str, tuple[str | None, str]]:
    if spec.overlay is None:
        return {}
    overlay = spec.overlay
    require(file_hash(overlay.patch) == overlay.patch_sha256 and
            file_hash(overlay.manifest) == overlay.manifest_sha256, '/overlay', 'overlay artifact hash mismatch')
    node = document(overlay.manifest.read_text())
    if node.field('schema_version').value == 2:
        from generic_identity_overlay import IdentityContext
        require(isinstance(identity, IdentityContext), '/identity', 'admitted whole-gate identity context required')
        assert identity is not None
        node.field('observer_patch_sha256').equals(overlay.patch_sha256)
        require(overlay.patch.read_bytes() == b'', '/identity', 'identity patch must be exactly zero bytes')
        identity.validate(spec)
        require(file_hash(overlay.patch) == overlay.patch_sha256 and
                file_hash(overlay.manifest) == overlay.manifest_sha256, '/identity', 'identity artifact changed during validation')
        return {}
    require(identity is None, '/overlay', 'identity context cannot authorize a normal overlay')
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
        path = safe_path(spec.root, name, allow_missing_parents=before is None)
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
    """Fresh independent build; warm admission is confined to the owned pair entrypoint."""
    return _build_product(spec, None)


def _build_product(spec: BuildSpec, prior: '_WorkspacePrefix | None',
                   publications: 'list[_BuildPublication] | None' = None) -> BuildResult:
    """Run a fully explicit Cargo recipe into a new target; retain failed build evidence."""
    def run(process: ProcessSpec) -> ProcessResult:
        if prior is not None:
            prior.check(before=True)
        result = execute(process)
        if publications is not None:
            require(result.receipt == process.output_root/'process.json', '/producer', 'foreign process return')
            actual = document(result.receipt.read_text())
            actual.field('status').equals(result.status)
            actual.field('native_exit').equals(result.native_exit)
            actual.field('pid').equals(result.pid)
            actual.field('argv').equals(list(process.argv))
            actual.field('cwd').equals(str(process.cwd))
            for stream in ('stdout', 'stderr'):
                actual.field(stream+'_sha256').equals(file_hash(process.output_root/(stream+'.log')))
        if prior is not None:
            prior.check(before=process.output_root != spec.output/'cargo-build')
        return result

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
    if prior is None:
        require(not target.exists() and not target.is_symlink(), str(target), 'fresh independent target required')
    else:
        prior.check(before=True)
    require(spec.output.is_absolute() and spec.output.parent.resolve() == spec.output.parent
            and not spec.output.is_relative_to(spec.root), str(spec.output), 'fresh external evidence root required')
    for artifact in spec.artifacts:
        require(re.fullmatch('[A-Za-z0-9_-]+', artifact.name) is not None, '/artifacts', 'invalid target name')
        require(bool(Path(artifact.relative_path).parts) and Path(artifact.relative_path).parts[0] == spec.profile
                and '..' not in Path(artifact.relative_path).parts and not Path(artifact.relative_path).is_absolute(),
                '/artifacts', 'artifact outside selected profile')
    source = freeze_source(spec.root, spec.source_sha) if prior is None else prior.publication.source(spec)
    index_before = git(spec.root, 'ls-files', '--stage', '-z') if prior is None else prior.publication.index_before
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
    if publications is not None:
        receipt.update(source_before_role='ORIGINAL_PRISTINE_BASELINE',
                       overlay_applied_this_build=spec.overlay is not None and prior is None)
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
        changed = overlay_files(spec) if prior is None else dict(prior.publication.changed)
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
            result = run(ProcessSpec((str(executable), '--version', '--verbose'), spec.root / 'rust',
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
            metadata_result = run(ProcessSpec(metadata_argv,spec.root/'rust',spec.cargo,spec.cargo_sha256,
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
        if prior is None:
            require(not target.exists() and not target.is_symlink(), str(target), 'target appeared before build')
        else:
            prior.check(before=True)
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
        result = run(ProcessSpec(tuple(argv), spec.root / 'rust', spec.cargo, spec.cargo_sha256,
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
        if prior is not None:
            prior.check(before=False)
            receipt['antecedent_build_receipt'] = str(prior.result.receipt)
            receipt['antecedent_build_receipt_sha256'] = prior.receipt_sha256
            receipt['target_before'] = dict(prior.target)
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
    if receipt['status'] == 'PASS' and publications is not None:
        expected = json.dumps(receipt, indent=2, sort_keys=True, allow_nan=False)+'\n'
        documents = _artifact_documents(spec, receipt, terminal, hashlib.sha256(expected.encode()).hexdigest(),
                                        tuple(artifacts), workspace_targets)
        publications.append(_BuildPublication(expected, tuple(sorted(_chain_tree(target).items())),
            tuple(sorted(_chain_tree(spec.output).items())), tuple(sorted((name, json.dumps(value, indent=2, sort_keys=True, allow_nan=False)+'\n')
                for name, value in documents.items())), index_before, tuple(sorted(changed.items()))))
    atomic_json(terminal, receipt)
    if receipt['status'] == 'PASS':
        for name, artifact_receipt in _artifact_documents(spec, receipt, terminal, file_hash(terminal),
                                                         tuple(artifacts), workspace_targets).items():
            atomic_json(spec.output/name, artifact_receipt)
    return BuildResult(terminal, 'PASS' if receipt['status'] == 'PASS' else 'FAIL', tuple(artifacts))


def _artifact_documents(spec: BuildSpec, receipt: dict[str, Json], terminal: Path,
                        receipt_sha256: str, artifacts: tuple[tuple[str, Path, str], ...],
                        workspace_targets: dict[str, Json]) -> dict[str, dict[str, Json]]:
    """One source for ordinary publication and the pair's prepublication expectation."""
    documents: dict[str, dict[str, Json]] = {}
    for name, path, digest in artifacts:
        artifact_receipt: dict[str,Json] = {
            'source_sha': spec.source_sha, 'binary_sha256': digest, 'binary': str(path),
            'profile': spec.profile, 'exit_code': receipt['exit_code'], 'source_kind': receipt['source_kind'],
            'build_receipt': str(terminal), 'build_receipt_sha256': receipt_sha256}
        if spec.workspace_tests:
            artifact_receipt.update(build_kind='workspace_tests',workspace_target=workspace_targets[name],
                metadata_process_receipt=receipt['metadata_process_receipt'],
                metadata_receipt_sha256=receipt['metadata_receipt_sha256'],metadata_sha256=receipt['metadata_sha256'])
        documents[f'artifact-{name}.json'] = artifact_receipt
    return documents


@dataclass(frozen=True, slots=True)
class WorkspaceBuildPair:
    receipt: Path
    status: Literal['BUILD_ANTECEDENTS_ONLY', 'FAIL']
    debug: BuildResult | None
    release: BuildResult | None
    retained_anchor: '_PairAnchor | None' = None
    runtime_qualification: Literal['NOT_RUN'] = 'NOT_RUN'
    publication_authority: Literal[False] = False


def _chain_tree(root: Path) -> dict[str, str]:
    """Include empty directories and modes; reject links/specials instead of following them."""
    real_path(root, directory=True)
    found: dict[str, str] = {}
    pending = [root]
    while pending:
        path = pending.pop()
        mode = path.lstat().st_mode
        name = path.relative_to(root).as_posix()
        if stat.S_ISDIR(mode):
            found[name] = f'directory:{mode}'
            pending.extend(sorted(path.iterdir()))
        else:
            require(stat.S_ISREG(mode), str(path), 'nonregular chain member')
            found[name] = f'file:{mode}:{file_hash(path)}'
    return found


@dataclass(frozen=True, slots=True)
class _BuildPublication:
    receipt_text: str
    target: tuple[tuple[str, str], ...]
    evidence: tuple[tuple[str, str], ...]
    sidecars: tuple[tuple[str, str], ...]
    index_before: bytes
    changed: tuple[tuple[str, tuple[str | None, str]], ...]

    def source(self, spec: BuildSpec) -> dict[str, Json]:
        '''Revalidate the retained baseline/index and exact derived bytes without reapplying.'''
        row = document(self.receipt_text)
        row.field('root').equals(str(spec.root))
        row.field('source_sha').equals(spec.source_sha)
        expected_overlay: Json = None
        if spec.overlay is not None:
            overlay = spec.overlay
            require(file_hash(overlay.patch) == overlay.patch_sha256
                    and file_hash(overlay.manifest) == overlay.manifest_sha256,
                    '/overlay', 'retained overlay proof changed')
            expected_overlay = {'patch': str(overlay.patch), 'patch_sha256': overlay.patch_sha256,
                                'manifest': str(overlay.manifest), 'manifest_sha256': overlay.manifest_sha256}
        row.field('overlay').equals(expected_overlay)
        source = row.field('source_before').mapping()
        current = derived_snapshot(spec.root, source, dict(self.changed), self.index_before)
        row.field('effective_source_after').equals(current)
        return source

    def check(self, spec: BuildSpec, result: BuildResult) -> None:
        """Authenticate against producer-time anchors, never a post-callback baseline."""
        _ = self.source(spec)
        require(result.status == 'PASS' and result.receipt == spec.output/'build.json',
                '/publication', 'failed or foreign producer return')
        require(result.receipt.read_text() == self.receipt_text, '/publication', 'producer receipt changed')
        require(_chain_tree(spec.root/'rust/target') == dict(self.target),
                '/publication', 'producer target changed during publication')
        current = _chain_tree(spec.output)
        old = dict(self.evidence)
        additions = {'build.json', *(name for name, _ in self.sidecars)}
        require(current.keys() == old.keys() | additions
                and all(current[name] == value for name, value in old.items()),
                '/publication', 'producer evidence changed during publication')
        for name, expected in self.sidecars:
            require((spec.output/name).read_text() == expected,
                    '/publication', 'artifact sidecar differs from producer-time expectation')


@dataclass(frozen=True, slots=True)
class _WorkspacePrefix:
    spec: BuildSpec
    result: BuildResult
    receipt_sha256: str
    evidence: tuple[tuple[str, str], ...]
    target: tuple[tuple[str, str], ...]
    inputs: tuple[tuple[str, str], ...]
    publication: _BuildPublication

    def check_source(self) -> None:
        real_path(self.spec.root, directory=True)
        real_path(self.spec.cargo_home, directory=True)
        _ = config_inventory(self.spec)
        require(file_hash(self.spec.cargo) == self.spec.cargo_sha256
                and file_hash(self.spec.rustc) == self.spec.rustc_sha256
                and os.access(self.spec.cargo, os.X_OK) and os.access(self.spec.rustc, os.X_OK),
                '/antecedent', 'prior tool changed')
        require(file_hash(self.result.receipt) == self.receipt_sha256
                and _chain_tree(self.spec.output) == dict(self.evidence), '/antecedent', 'prior build evidence changed')
        _ = self.publication.source(self.spec)
        require(inventory(self.spec.root, [*INPUT_DIRS, *INPUT_FILES]) == dict(self.inputs),
                '/antecedent', 'prior source/input changed')
    def check(self, *, before: bool) -> None:
        self.check_source()
        current = _chain_tree(self.spec.root/'rust/target')
        expected = dict(self.target)
        require(all(current.get(name) == value for name, value in expected.items()),
                '/antecedent', 'protected prior target changed')
        if before:
            require(current == expected, '/antecedent', 'prior target membership changed before release')
        else:
            require(all(name == 'release' or name.startswith('release/') for name in current.keys()-expected.keys()),
                    '/antecedent', 'release producer changed non-release target membership')


def _workspace_prefix(spec: BuildSpec, result: BuildResult, publication: _BuildPublication) -> _WorkspacePrefix:
    """Only called with the actual first return in the closed pair execution."""
    require(result.status == 'PASS' and result.receipt == spec.output/'build.json', '/antecedent', 'prior build failed/foreign')
    row = document(result.receipt.read_text())
    for name, value in (('status', 'PASS'), ('profile', 'debug'), ('build_kind', 'workspace_tests'),
                        ('root', str(spec.root)), ('source_sha', spec.source_sha), ('exit_code', 0)):
        row.field(name).equals(value)
    row.field('effective_source_before').equals(row.field('effective_source_after').value)
    actual_inputs = inventory(spec.root, [*INPUT_DIRS, *INPUT_FILES])
    row.field('effective_inputs_before').equals({name: digest for name, digest in actual_inputs.items()})
    row.field('artifacts').equals({name: {'path': str(path), 'sha256': digest} for name, path, digest in result.artifacts})
    require(bool(result.artifacts) and all(file_hash(path) == digest for _, path, digest in result.artifacts),
            '/antecedent', 'prior protected executable changed')
    source = publication.source(spec)
    row.field('source_before').equals(source)
    prefix = _WorkspacePrefix(spec, result, file_hash(result.receipt), tuple(sorted(_chain_tree(spec.output).items())),
        tuple(sorted(_chain_tree(spec.root/'rust/target').items())), tuple(sorted(actual_inputs.items())), publication)
    prefix.check(before=True)
    return prefix


def _publish_pair(output: Path, name: str, row: dict[str, Json], before: dict[str, str]) -> dict[str, str]:
    """Check pair-owned files before/after publication against intended bytes and modes."""
    require(_chain_tree(output) == before, '/pair-output', 'prior pair evidence changed')
    require(name not in before, '/pair-output', 'pair receipt already exists')
    raw = json.dumps(row, indent=2, sort_keys=True, allow_nan=False)+'\n'
    # mkdir(0777) and atomic_json open(0666) share the caller's unchanged umask.
    mode = stat.S_IFREG | (stat.S_IMODE(output.lstat().st_mode) & 0o666)
    expected = {**before, name: f'file:{mode}:{hashlib.sha256(raw.encode()).hexdigest()}'}
    atomic_json(output/name, row)
    require(_chain_tree(output) == expected, '/pair-output', 'pair publication content/membership/mode changed')
    return expected


@dataclass(frozen=True, slots=True)
class _PairAnchor:
    prior: _WorkspacePrefix
    release_spec: BuildSpec
    release_result: BuildResult
    release_publication: _BuildPublication
    release_evidence: tuple[tuple[str, str], ...]
    output: Path
    pair_evidence: tuple[tuple[str, str], ...]

    def check_source(self) -> None:
        self.prior.check_source()
        _ = self.release_publication.source(self.release_spec)
        require(_chain_tree(self.release_spec.output) == dict(self.release_evidence),
                '/pair/source', 'historical release evidence changed')
        require(_chain_tree(self.output) == dict(self.pair_evidence), '/pair/source', 'historical pair evidence changed')

    def check(self) -> None:
        self.prior.check(before=False)
        self.release_publication.check(self.release_spec, self.release_result)
        require(_chain_tree(self.release_spec.output) == dict(self.release_evidence),
                '/pair', 'release evidence drift after return')
        require(_chain_tree(self.output) == dict(self.pair_evidence), '/pair', 'pair evidence drift after return')
        for result in (self.prior.result, self.release_result):
            for _, path, digest in result.artifacts:
                require(file_hash(path) == digest, '/pair', 'returned artifact drift')


def recheck_workspace_pair(result: WorkspaceBuildPair) -> tuple[BuildSpec, BuildSpec]:
    """Recheck retained live-chain provenance; never reconstruct authority from JSON.

    Python constructors/private attributes are not a security boundary. Only the
    source-owned returned anchor is supported; this grants no runtime qualification.
    """
    anchor = result.retained_anchor
    require(result.status == 'BUILD_ANTECEDENTS_ONLY' and anchor is not None
            and result.runtime_qualification == 'NOT_RUN' and result.publication_authority is False,
            '/pair', 'successful live pair anchor required')
    assert anchor is not None
    require(result.receipt == anchor.output/'chain.json' and result.debug is anchor.prior.result
            and result.release is anchor.release_result, '/pair', 'pair result identity differs')
    anchor.check()
    return anchor.prior.spec, anchor.release_spec


def build_workspace_pair(debug: BuildSpec, release: BuildSpec, output: Path) -> WorkspaceBuildPair:
    """Own fresh debug then release antecedents; never substitute them for original gate steps.

    This inherits build_product's process execution/ownership limits. An unexpected
    interruption propagates with the initial INCOMPLETE receipt retained; it is not
    a quiescence claim. No public prior-result or digest-only warm grant is accepted.
    """
    require(debug.profile == 'debug' and release.profile == 'release' and debug.workspace_tests
            and replace(debug, profile='release', output=release.output) == release,
            '/pair', 'same-source workspace debug/release specs required')
    paths = (debug.output, release.output, output)
    for path in paths:
        require(path.is_absolute() and path.parent == path.parent.resolve(strict=True)
                and not path.is_relative_to(debug.root) and not path.exists() and not path.is_symlink(),
                '/pair', 'fresh canonical external outputs required')
    require(all(not left.is_relative_to(right) for left in paths for right in paths if left != right)
            and len(set(paths)) == 3, '/pair', 'pair outputs overlap')
    output.mkdir()
    terminal = output/'chain.json'
    row: dict[str, Json] = {'schema': 'workspace_build_pair.v1', 'status': 'INCOMPLETE',
        'root': str(debug.root), 'source_sha': debug.source_sha, 'debug': None, 'release': None,
        'runtime_qualification': 'NOT_RUN', 'rustdoc_invocations': 'NOT_AVAILABLE',
        'publication_authority': False, 'original_gate_steps_executed': False}
    pair_evidence = _chain_tree(output)
    publications: list[_BuildPublication] = []
    first: BuildResult | None = None
    second: BuildResult | None = None
    anchor: _PairAnchor | None = None
    try:
        pair_evidence = _publish_pair(output, 'started.json', row, pair_evidence)
        first = _build_product(debug, None, publications)
        row['debug'] = {'receipt': str(first.receipt), 'sha256': file_hash(first.receipt), 'status': first.status}
        if first.status != 'PASS':
            pair_evidence = _publish_pair(output, 'debug-returned.json', row, pair_evidence)
        require(first.status == 'PASS', '/pair', 'debug antecedent failed; release NOT_RUN')
        require(len(publications) == 1, '/publication', 'missing debug producer anchor')
        publications[0].check(debug, first)
        prior = _workspace_prefix(debug, first, publications[0])
        pair_evidence = _publish_pair(output, 'debug-returned.json', row, pair_evidence)
        prior.check(before=True)
        row['target_before_release'] = dict(prior.target)
        second = _build_product(release, prior, publications)
        row['release'] = {'receipt': str(second.receipt), 'sha256': file_hash(second.receipt), 'status': second.status}
        require(second.status == 'PASS', '/pair', 'release antecedent failed')
        require(len(publications) == 2, '/publication', 'missing release producer anchor')
        publications[1].check(release, second)
        prior.check(before=False)
        require(all(file_hash(path) == digest for _, path, digest in second.artifacts), '/pair', 'release artifact changed')
        target_end = _chain_tree(debug.root/'rust/target')
        release_end = _chain_tree(release.output)
        row.update(status='BUILD_ANTECEDENTS_ONLY', target_after={name: value for name, value in target_end.items()},
                   debug_evidence=dict(prior.evidence), release_evidence={name: value for name, value in release_end.items()})
        pair_evidence = _publish_pair(output, 'chain.json', row, pair_evidence)
        prior.check(before=False)
        require(_chain_tree(debug.root/'rust/target') == target_end and _chain_tree(release.output) == release_end,
                '/pair', 'pair changed during terminal publication')
        anchor = _PairAnchor(prior, release, second, publications[1], tuple(sorted(release_end.items())),
                             output, tuple(sorted(pair_evidence.items())))
        anchor.check()
    except (OSError, ValueError, KeyError) as error:
        row.update(status='FAIL', error=str(error))
        terminal = output/'chain-failure.json'
        atomic_json(terminal, row)
    return WorkspaceBuildPair(terminal, 'BUILD_ANTECEDENTS_ONLY' if row['status'] == 'BUILD_ANTECEDENTS_ONLY' else 'FAIL',
                              first, second, anchor)


@dataclass(frozen=True, slots=True)
class WorkspaceSourceProof:
    baseline_json: str
    effective_json: str
    effective_inputs: tuple[tuple[str, str], ...]
    source_kind: Literal['PRISTINE_COMMIT', 'DERIVED_OBSERVER']
    retained_anchor: _PairAnchor | None = None
    runtime_qualification: Literal['NOT_RUN'] = 'NOT_RUN'
    publication_authority: Literal[False] = False


def _source_fields(anchor: _PairAnchor) -> tuple[str, str, tuple[tuple[str, str], ...], Literal['PRISTINE_COMMIT', 'DERIVED_OBSERVER']]:
    row = document(anchor.prior.publication.receipt_text)
    return (json.dumps(row.field('source_before').value, sort_keys=True),
            json.dumps(row.field('effective_source_after').value, sort_keys=True), anchor.prior.inputs,
            'PRISTINE_COMMIT' if anchor.prior.spec.overlay is None else 'DERIVED_OBSERVER')


def workspace_pair_source(pair: WorkspaceBuildPair) -> WorkspaceSourceProof:
    """Mint a source-only live proof while the entire actual build pair is still intact."""
    _ = recheck_workspace_pair(pair)
    anchor = pair.retained_anchor
    assert anchor is not None
    proof = WorkspaceSourceProof(*_source_fields(anchor), retained_anchor=anchor)
    recheck_workspace_source(proof)
    _ = recheck_workspace_pair(pair)
    return proof


def recheck_workspace_source(proof: WorkspaceSourceProof) -> None:
    """Recheck source/history, not later generated targets or current artifact bytes.

    Actual producer endpoints govern generated-state transitions. This live proof
    cannot be reconstructed from serialized fields and grants no runtime authority.
    """
    anchor = proof.retained_anchor
    require(anchor is not None and proof.runtime_qualification == 'NOT_RUN' and proof.publication_authority is False,
            '/pair/source', 'retained live source proof required')
    assert anchor is not None
    require((proof.baseline_json, proof.effective_json, proof.effective_inputs, proof.source_kind) == _source_fields(anchor),
            '/pair/source', 'source proof fields differ from actual retained producer')
    anchor.check_source()


@dataclass(frozen=True, slots=True)
class NormalProduct:
    package: str
    package_id: str
    name: str
    source: Path
    manifest: Path
    source_sha: str
    profile: Literal['debug', 'release']
    binary: Path
    sha256: str
    metadata_sha256: str
    compiler_stream_sha256: str
    compiler_line: int
    compiler_line_sha256: str


@dataclass(frozen=True, slots=True)
class _NormalAnchor:
    pair: WorkspaceBuildPair
    profile: Literal['debug', 'release']
    names: tuple[tuple[str, str], ...]
    products: tuple[NormalProduct, ...]


@dataclass(frozen=True, slots=True)
class NormalProducts:
    pair: WorkspaceBuildPair
    profile: Literal['debug', 'release']
    names: tuple[tuple[str, str], ...]
    products: tuple[NormalProduct, ...]
    retained_anchor: _NormalAnchor | None = None
    runtime_qualification: Literal['NOT_RUN'] = 'NOT_RUN'
    publication_authority: Literal[False] = False


def _normal_products(pair: WorkspaceBuildPair, profile: Literal['debug', 'release'],
                     names: tuple[tuple[str, str], ...]) -> tuple[NormalProduct, ...]:
    """V9 exact normal-bin joins, parameterized by authenticated workspace metadata."""
    specs = recheck_workspace_pair(pair)
    pair_anchor = pair.retained_anchor
    assert pair_anchor is not None
    require(profile in ('debug', 'release') and bool(names) and len(set(names)) == len(names),
            '/normal', 'explicit profile and unique nonempty selectors required')
    spec = specs[0] if profile == 'debug' else specs[1]
    result = pair.debug if profile == 'debug' else pair.release
    require(result is not None and spec.profile == profile and spec.workspace_tests, '/normal', 'actual profile result required')
    assert result is not None
    receipt = document(result.receipt.read_text())
    metadata_path = spec.output/'cargo-metadata/stdout.log'
    stream = spec.output/'cargo-build/stdout.log'
    metadata_hash = file_hash(metadata_path); raw = stream.read_bytes()
    stream_hash = hashlib.sha256(raw).hexdigest()
    receipt.field('metadata_sha256').equals(metadata_hash)
    receipt.field('metadata_process_receipt').equals(str(metadata_path.parent/'process.json'))
    receipt.field('process_receipt').equals(str(stream.parent/'process.json'))
    metadata = document(metadata_path.read_text())
    metadata.field('workspace_root').equals(str(spec.root/'rust'))
    metadata.field('target_directory').equals(str(spec.root/'rust/target'))
    members = {node.text() for node in metadata.field('workspace_members').array()}
    messages = tuple((index,line,document(line.decode())) for index,line in enumerate(raw.splitlines()))
    native_test_paths = {path for _,path,_ in result.artifacts}
    products: list[NormalProduct] = []
    for package_name,name in names:
        packages = [node for node in metadata.field('packages').array()
                    if node.field('name').text() == package_name and node.field('id').text() in members]
        require(len(packages) == 1, '/normal/package', 'missing or ambiguous workspace package')
        package = packages[0]; identity = package.field('id').text()
        targets = [node for node in package.field('targets').array()
                   if node.field('name').text() == name and node.field('kind').value == ['bin']]
        require(len(targets) == 1, '/normal/target', 'missing or ambiguous normal bin metadata')
        target = targets[0]
        target.field('crate_types').equals(['bin'])
        source = Path(target.field('src_path').text()); manifest = Path(package.field('manifest_path').text())
        for path in (source,manifest):
            real_path(path,directory=False)
            require(path.is_relative_to(spec.root) and str(path.relative_to(spec.root)) in receipt.field('effective_inputs_before').mapping(),
                    '/normal/source', 'source outside authenticated build input scope')
        selected = [(index,line,node) for index,line,node in messages
                    if node.field('reason').value == 'compiler-artifact'
                    and node.field('package_id').value == identity
                    and node.field('target').field('name').value == name
                    and node.field('target').field('kind').value == ['bin']
                    and node.field('profile').field('test').value is False]
        require(len(selected) == 1, '/normal/compiler', 'missing or ambiguous actual non-test record')
        index,line,node = selected[0]
        node.field('fresh').equals(False)
        require(node.field('fresh').value is False, '/normal/compiler', 'fresh must be false boolean')
        node.field('manifest_path').equals(str(manifest))
        compiled = node.field('target')
        compiled.field('src_path').equals(str(source)); compiled.field('crate_types').equals(['bin'])
        host = receipt.field('toolchain_versions').field('rustc').field('text').text()
        suffix = '.exe' if re.search(r'^host: .*windows[^\n]*$',host,re.MULTILINE) else ''
        expected = spec.root/'rust/target'/profile/(name+suffix)
        require(expected.parent == spec.root/'rust/target'/profile, '/normal/path', 'nested target name')
        node.field('executable').equals(str(expected))
        node.field('filenames').equals([str(expected)])
        real_path(expected,directory=False)
        require(expected not in native_test_paths and os.access(expected,os.X_OK), '/normal/path', 'test harness or nonexecutable output')
        products.append(NormalProduct(package_name,identity,name,source,manifest,spec.source_sha,profile,expected,
            file_hash(expected),metadata_hash,stream_hash,index,hashlib.sha256(line).hexdigest()))
    require(len({item.binary for item in products}) == len(products), '/normal/path', 'shared selected output')
    _ = recheck_workspace_pair(pair)
    require(file_hash(metadata_path) == metadata_hash and file_hash(stream) == stream_hash,
            '/normal', 'metadata/compiler stream changed during selection')
    require(all(file_hash(item.binary) == item.sha256 for item in products),
            '/normal', 'selected binary changed at final boundary')
    pair_anchor.check()
    return tuple(products)


def normal_workspace_products(pair: WorkspaceBuildPair, profile: Literal['debug', 'release'],
                              names: tuple[tuple[str, str], ...]) -> NormalProducts:
    """Read actual live-pair normal bins; no build, alias, test or bridge authority is invented."""
    products = _normal_products(pair,profile,names)
    anchor = _NormalAnchor(pair,profile,names,products)
    return NormalProducts(pair,profile,names,products,anchor)


def recheck_normal_products(result: NormalProducts) -> None:
    """Reject reconstructed/digest-only products; retain the actual source-owned pair."""
    anchor = result.retained_anchor
    require(anchor is not None and result.runtime_qualification == 'NOT_RUN' and result.publication_authority is False,
            '/normal', 'live normal-product anchor required')
    assert anchor is not None
    require(result.pair is anchor.pair and result.profile == anchor.profile and result.names == anchor.names
            and result.products is anchor.products, '/normal', 'retained selection identity changed')
    require(_normal_products(result.pair,result.profile,result.names) == anchor.products,
            '/normal', 'retained products changed')
