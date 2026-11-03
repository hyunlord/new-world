"""Pinned Cargo doctest argument construction; no process or qualification authority."""
import json
import tomllib
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import generic_workspace_arguments as arguments
from generic_build import (
    WorkspaceBuildPair,
    WorkspaceSourceProof,
    recheck_workspace_pair,
    recheck_workspace_source,
    workspace_pair_source,
)
from generic_gate import document, require
from generic_process import file_hash
from generic_setup_bridge import immutable_inputs
from generic_sources import inventory
from generic_workspace_arguments import texts as _texts
from generic_workspace_endpoints import (
    ImportPredecessor,
    WorkspaceChain,
    admit_import,
    check_endpoint,
    generated,
)
from mux2_gate_inputs import Json, Node

match_invocation = arguments.match_invocation
validate_policy = arguments.validate_policy

CARGO_COMMIT = '083ac5135f967fd9dc906ab057a2315861c7a80d'
RUST_COMMIT = '01f6ddf7588f42ae2d7eb0a2f21d44e8e96674cf'
TARGETS = frozenset({'sim-bridge', 'sim-core', 'sim-engine', 'sim-systems', 'sim-test'})


@dataclass(frozen=True, slots=True)
class _PolicyAnchor:
    pair: WorkspaceBuildPair
    profile: str
    source: WorkspaceSourceProof
    policy_json: str
    inputs: tuple[tuple[Path, str], ...]
    trees: tuple[tuple[Path, str], ...]


@dataclass(frozen=True, slots=True)
class PolicyTemplate:
    """Live pair-derived rows; current bytes required, never refreshed after drift."""
    pair: WorkspaceBuildPair
    profile: Literal['debug', 'release']
    source: WorkspaceSourceProof
    policy_json: str
    inputs: tuple[tuple[Path, str], ...]
    trees: tuple[tuple[Path, str], ...]
    retained_anchor: _PolicyAnchor | None = None
    runtime_qualification: Literal['NOT_RUN'] = 'NOT_RUN'


# Retain actual issued objects for this bounded gate process; identity prevents
# copied dataclasses from selecting a replacement validation callback.
_ISSUED: dict[int, tuple[PolicyTemplate, Callable[[PolicyTemplate], None]]] = {}


def _check_issued(template: PolicyTemplate) -> None:
    issued = _ISSUED.get(id(template))
    require(issued is not None and issued[0] is template, '/policy', 'unissued template identity')
    assert issued is not None
    issued[1](template)


def _tree(path: Path) -> str:
    require(path.is_dir() and path == path.resolve(), '/tree', 'canonical generated directory required')
    # inventory rejects links/special files; directory membership is separately retained.
    scopes = [item.name for item in path.iterdir()]
    files = inventory(path, scopes) if scopes else {}
    members = sorted((item.relative_to(path).as_posix(), item.lstat().st_mode) for item in path.rglob('*'))
    return json.dumps({'root_mode': path.lstat().st_mode, 'files': files, 'members': members}, sort_keys=True)


def recheck_policy(template: PolicyTemplate) -> Node:
    anchor = template.retained_anchor
    require(anchor is not None, '/policy', 'retained constructor anchor required')
    assert anchor is not None
    require(template.pair is anchor.pair and template.source is anchor.source
            and (template.profile, template.policy_json, template.inputs, template.trees)
            == (anchor.profile, anchor.policy_json, anchor.inputs, anchor.trees),
            '/policy', 'retained policy identity differs')
    _check_issued(template)
    recheck_workspace_source(template.source)
    require(template.runtime_qualification == 'NOT_RUN', '/policy', 'no runtime authority')
    for path, digest in template.inputs:
        require(file_hash(path) == digest, '/policy', 'bound source/compiler/generated input changed')
    for path, state in template.trees:
        require(_tree(path) == state, '/policy', 'bound dependency/generated tree changed')
    result = document(template.policy_json)
    validate_policy(result)
    _check_issued(template)
    return result


def _manifest(path: Path) -> Node:
    # TOML values are checked through the same strict JSON shape used by the harness.
    return document(json.dumps(tomllib.loads(path.read_text())))


def argument_rows(metadata: Node, messages: tuple[Node, ...], root: Path, profile: str,
          runner: tuple[Path, Path]) -> list[Json]:
    members = set(_texts(metadata.field('workspace_members')))
    packages = [p for p in metadata.field('packages').array() if p.field('id').text() in members]
    require({p.field('name').text() for p in packages} == set(TARGETS), '/metadata', 'unsupported workspace membership')
    lock = _manifest(root/'Cargo.lock').field('package').array()
    rows: list[Json] = []
    for package in packages:
        manifest = Path(package.field('manifest_path').text()); raw = _manifest(manifest)
        require('lints' not in raw.mapping(), '/manifest', 'unsupported lint arguments')
        declared = package.field('features').mapping()
        source_features = raw.mapping().get('features', {})
        require(source_features == declared and 'default' not in declared, '/features', 'metadata/manifest feature declarations differ')
        names = sorted(declared)
        require(all(name and all(c.isalnum() or c in '_-' for c in name) for name in names), '/features', 'unsupported feature spelling')
        targets = [t for t in package.field('targets').array() if t.field('doctest').value is True]
        require(len(targets) == 1, '/metadata', 'one doctest library per member required')
        target = targets[0]; name = target.field('name').text(); source = Path(target.field('src_path').text())
        types = ['cdylib', 'rlib'] if package.field('name').text() == 'sim-bridge' else ['lib']
        target.field('crate_types').equals([*types]); target.field('edition').equals('2021')
        require(source.is_relative_to(manifest.parent) and manifest.parent.is_relative_to(root), '/source', 'foreign source')
        selected = [m for m in messages if m.field('reason').value == 'compiler-artifact'
                    and m.field('package_id').value == package.field('id').value
                    and m.field('target').field('name').value == name and m.field('profile').field('test').value is False]
        require(len(selected) == 1, '/compiler', 'unique normal workspace library required')
        selected[0].field('features').equals([])
        for field in ('name', 'src_path', 'crate_types', 'edition'):
            selected[0].field('target').field(field).equals(target.field(field).value)
        dependency_names = {name}
        for dep in package.field('dependencies').array():
            if dep.field('kind').value == 'build':
                continue
            require(dep.field('target').value is None and dep.field('optional').value is False
                    and dep.field('rename').value is None, '/dependencies', 'unsupported target/optional/renamed dependency')
            dependency_names.add(dep.field('name').text().replace('-', '_'))
        externs: dict[str, Json] = {}
        for dep_name in sorted(dependency_names):
            candidates = [m for m in messages if m.field('reason').value == 'compiler-artifact'
                and m.field('target').field('name').value == dep_name and m.field('profile').field('test').value is False
                and 'custom-build' not in _texts(m.field('target').field('kind'))]
            require(len(candidates) == 1, '/externs', 'missing/ambiguous compiled dependency')
            compiled = candidates[0]
            identity = compiled.field('package_id').text()
            workspace_matches = [p for p in packages if p.field('name').text().replace('-', '_') == dep_name]
            if workspace_matches:
                require(len(workspace_matches) == 1 and identity == workspace_matches[0].field('id').text(),
                        '/externs', 'foreign workspace package identity')
            else:
                locked = [p for p in lock if p.field('name').text().replace('-', '_') == dep_name]
                require(len(locked) == 1, '/externs', 'ambiguous locked dependency')
                package_lock = locked[0]
                expected = package_lock.field('source').text()+'#'+package_lock.field('name').text()+'@'+package_lock.field('version').text()
                require(identity == expected, '/externs', 'compiler package differs from lock')
            compiled_types = _texts(compiled.field('target').field('crate_types'))
            require(compiled_types in (('lib',), ('rlib',), ('cdylib', 'rlib')), '/externs', 'unsupported Rust-linkable target types')
            require(compiled.field('fresh').value is False, '/externs', 'actual nonfresh compiler record required')
            files = [Path(f) for f in _texts(compiled.field('filenames')) if f.endswith('.rlib')]
            require(len(files) == 1 and files[0].parent == root/'target'/profile/'deps', '/externs', 'unique admitted rlib required')
            _ = file_hash(files[0])
            externs[dep_name] = str(files[0])
        prefix = ['--edition=2021']
        for kind in types:
            prefix.extend(['--crate-type', kind])
        prefix.extend(['--color', 'auto', '--crate-name', name, '--test', source.relative_to(root).as_posix(),
            '--test-run-directory', str(manifest.parent), '--test-runtool', str(runner[0]), '--test-runtool-arg', str(runner[1]),
            '-L', 'dependency='+str(root/'target'/profile/'deps'), '-L', 'dependency='+str(root/'target'/profile/'deps'),
            '--test-args', '--test-threads=8'])
        feature_text = 'cfg(feature, values('+', '.join(json.dumps(n) for n in names)+'))'
        rows.append({'id': json.dumps([package.field('id').text(), name, str(source)], separators=(',', ':')),
            'package_id': package.field('id').value, 'source': str(source), 'source_argument': source.relative_to(root).as_posix(),
            'package_manifest': str(manifest), 'crate_name': name, 'edition': '2021', 'compiler_cwd': str(root),
            'run_cwd': str(manifest.parent), 'argument_policy': {'prefix': [*prefix], 'externs': externs,
            'suffix': ['-C', 'embed-bitcode=no' if profile == 'debug' else 'lto=thin', '--check-cfg', 'cfg(docsrs,test)',
                       '--check-cfg', feature_text, '--error-format', 'human']}})
    return rows


def build_policy(pair: WorkspaceBuildPair, profile: Literal['debug', 'release'],
                 runner: tuple[Path, Path]) -> PolicyTemplate:
    """Mint before setup changes target; actual pair is required, not receipt rows."""
    specs = recheck_workspace_pair(pair)
    require(profile in ('debug', 'release'), '/profile', 'closed workspace profile')
    spec = specs[0] if profile == 'debug' else specs[1]
    require(spec.workspace_tests and not spec.features, '/build', 'default workspace build required')
    source = workspace_pair_source(pair)
    receipt = document((spec.output/'build.json').read_text())
    cargo_version = receipt.field('toolchain_versions').field('cargo').field('text').text()
    require('commit-hash: '+CARGO_COMMIT+'\n' in cargo_version
            and 'host: aarch64-apple-darwin\n' in cargo_version, '/tool', 'unsupported Cargo toolchain')
    rust_version = receipt.field('toolchain_versions').field('rustc').field('text').text()
    require('commit-hash: '+RUST_COMMIT+'\n' in rust_version and 'host: aarch64-apple-darwin\n' in rust_version,
            '/tool', 'unsupported Rust compiler')
    workspace = spec.root/'rust'
    config = _manifest(workspace/'Cargo.toml')
    config.field('profile').field('release').equals({'lto': 'thin', 'codegen-units': 1})
    require(set(config.field('profile').mapping()) <= {'dev', 'release'} and 'lints' not in config.mapping(),
            '/profile', 'unsupported workspace profile/lints')
    if 'dev' in config.field('profile').mapping():
        config.field('profile').field('dev').equals({'package': {'sim-systems': {'opt-level': 1}}})
    for path, _digest in spec.config_inputs:
        settings = _manifest(Path(path)).mapping()
        require(set(settings) <= {'build', 'net'} and settings.get('build', {'jobs': 8}) == {'jobs': 8}
                and settings.get('net', {'offline': True}) == {'offline': True}, '/config', 'unsupported Cargo config')
    expected_env: dict[str, Json] = {'PATH': str(spec.rustc.parent)+':/usr/bin:/bin:/usr/sbin:/sbin',
        'CARGO_HOME': str(spec.cargo_home), 'CARGO_TARGET_DIR': str(workspace/'target'), 'RUSTC': str(spec.rustc),
        'RUSTC_WRAPPER': '', 'RUSTC_WORKSPACE_WRAPPER': ''}
    receipt.field('environment').equals(expected_env)
    document((spec.output/'cargo-build/process.json').read_text()).field('environment').equals(
        {'LANG': 'C', 'LC_ALL': 'C', **expected_env})
    metadata_file = spec.output/'cargo-metadata/stdout.log'; stream = spec.output/'cargo-build/stdout.log'
    metadata = document(metadata_file.read_text())
    metadata.field('workspace_root').equals(str(workspace)); metadata.field('target_directory').equals(str(workspace/'target'))
    messages = tuple(document(line) for line in stream.read_text().splitlines())
    trees = {workspace/'target'/profile/'deps'}
    trees.update(item.output for item in specs)
    trees.add(pair.receipt.parent)
    evidence = {metadata_file, stream, spec.output/'build.json', spec.output/'cargo-build/process.json', Path(__file__).resolve(), Path(arguments.__file__).resolve(), *runner}
    require(runner[1].name == 'generic_cargo_child.py', '/runner', 'closed host runner required')
    members = set(_texts(metadata.field('workspace_members')))
    for message in messages:
        if message.field('reason').value != 'build-script-executed':
            continue
        message.field('linked_paths').equals([])
        out = Path(message.field('out_dir').text())
        require(out.name == 'out' and out.parent.parent == workspace/'target'/profile/'build', '/build-script', 'foreign generated directory')
        raw = out.parent/'output'
        require(raw.is_file(), '/build-script', 'actual retained raw output required')
        evidence.add(raw); trees.add(out)
        if message.field('package_id').text() in members:
            package = next(p for p in metadata.field('packages').array() if p.field('id').value == message.field('package_id').value)
            package.field('name').equals('sim-bridge')
            message.field('cfgs').equals([]); message.field('env').equals([]); message.field('linked_libs').equals([])
            lines = raw.read_text().splitlines()
            require(bool(lines) and all(line.startswith('cargo:rerun-if-changed=') for line in lines),
                    '/build-script', 'unsupported workspace script contribution')
            require((out/'material_files.rs').is_file(), '/build-script', 'missing generated material source')
    # Metadata must not declare an unaccounted workspace build script.
    for package in metadata.field('packages').array():
        if package.field('id').text() in members and any(t.field('kind').value == ['custom-build'] for t in package.field('targets').array()):
            require(any(m.field('reason').value == 'build-script-executed' and m.field('package_id').value == package.field('id').value for m in messages),
                    '/build-script', 'missing actual workspace script record')
    rows = argument_rows(metadata, messages, workspace, profile, runner)
    policy_json = json.dumps({'schema_version': 4, 'invocations': rows}, sort_keys=True)
    evidence.update(spec.root/name for name, _digest in source.effective_inputs)
    evidence.update(Path(path) for path, _digest in spec.config_inputs)
    evidence.update((spec.cargo, spec.rustc))
    for tree in trees:
        evidence.update(path for path in tree.rglob('*') if path.is_file())
    inputs = tuple((path, file_hash(path)) for path in sorted(evidence))
    frozen_trees = tuple((path, _tree(path)) for path in sorted(trees))
    modes = tuple((path, path.lstat().st_mode) for path, _digest in inputs)
    def issued_check(candidate: PolicyTemplate) -> None:
        # Issued lexical evidence is not a second caller-replaceable dataclass.
        require(candidate.pair is pair and candidate.source is source and candidate.profile == profile
                and candidate.inputs == inputs and candidate.trees == frozen_trees,
                '/policy', 'foreign issued template/coverage')
        recheck_workspace_source(source)
        derived = json.dumps({'schema_version': 4, 'invocations': argument_rows(
            document(metadata_file.read_text()), tuple(document(line) for line in stream.read_text().splitlines()),
            workspace, profile, runner)}, sort_keys=True)
        require(candidate.policy_json == policy_json == derived, '/policy', 'source-derived complete obligations differ')
        # All semantic callbacks precede this comparison with the original raw closure.
        for path, digest in inputs:
            require(file_hash(path) == digest, '/policy', 'issued input bytes changed')
        for path, mode in modes:
            require(path.lstat().st_mode == mode, '/policy', 'issued input mode changed')
        for path, state in frozen_trees:
            require(_tree(path) == state, '/policy', 'issued tree membership/mode changed')
    anchor = _PolicyAnchor(pair, profile, source, policy_json, inputs, frozen_trees)
    template = PolicyTemplate(pair, profile, source, policy_json, inputs, frozen_trees, anchor)
    _ISSUED[id(template)] = (template, issued_check)
    _ = recheck_policy(template)
    _ = recheck_workspace_pair(pair)
    # Last check is raw retained closure, not a refreshed snapshot.
    for path, digest in inputs:
        require(file_hash(path) == digest, '/policy', 'input changed during constructor callback')
    for path, state in frozen_trees:
        require(_tree(path) == state, '/policy', 'tree changed during constructor callback')
    _check_issued(template)
    return template


def bind_policy(template: PolicyTemplate, chain: WorkspaceChain, step: Literal[4, 5]) -> Node:
    """Join current actual predecessor, never reinterpret a historical target as current."""
    prepared = chain.prepared
    require(prepared.warm_pair is template.pair and
            template.profile == ('debug' if step == 4 else 'release'), '/chain', 'foreign pair/profile')
    _ = immutable_inputs(prepared)
    environment = dict(prepared.inputs.environment)
    require(not any('FLAGS' in key or key.startswith(('CARGO_PROFILE_', 'CARGO_ENCODED_'))
                    or key in {'CARGO_BUILD_TARGET', 'CARGO_TERM_COLOR', 'CARGO_TERM_QUIET'} for key in environment),
            '/environment', 'unsupported runtime flag/profile override')
    prior = chain.predecessor
    if isinstance(prior, ImportPredecessor):
        require(step == 4, '/chain', 'imports must precede debug')
        history = admit_import(prepared, prior)
    else:
        require(step == 5 and prior.step == 4 and prior.prepared is prepared, '/chain', 'debug endpoint required')
        check_endpoint(prior)
        require(generated(prepared) == prior.after, '/chain', 'generated endpoint changed')
        history = prior.history
    before = generated(prepared)
    result = recheck_policy(template)
    for item in history:
        item.recheck()
    require(generated(prepared) == before, '/chain', 'endpoint changed during binding')
    for path, digest in template.inputs:
        require(file_hash(path) == digest, '/chain', 'protected input changed during endpoint checks')
    for path, state in template.trees:
        require(_tree(path) == state, '/chain', 'protected tree changed during endpoint checks')
    assert template.retained_anchor is not None
    _check_issued(template)
    return result
