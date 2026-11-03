"""Source-bound grants for original play children; observer qualification stays pending."""
import os
import re
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import final

from generic_gate import document, require, text
from generic_process import file_hash, real_path
from generic_sources import freeze_source, git
from mux2_gate_inputs import Json, Node


@dataclass(frozen=True, slots=True)
class HistoricalSource:
    version: str
    commit: str
    tracked_manifest_sha256: str
    lock_sha256: str
    tag_oid: str | None = None


@dataclass(frozen=True, slots=True)
class HistoricalBindings:
    sources: tuple[HistoricalSource, ...]
    git: tuple[Path, str]
    cargo: tuple[Path, str]
    rustc: tuple[Path, str]
    # Explicit presence AND absence; transitive runtime closure remains caller-owned.
    configs: tuple[tuple[Path, str | None], ...]


@dataclass(frozen=True, slots=True)
class BuiltBridge:
    path: Path
    sha256: str
    receipt: Path
    receipt_sha256: str
    pid: int


def check_bindings(bindings: HistoricalBindings) -> None:
    require(tuple(source.version for source in bindings.sources) == ('main', 'm1', 'v7'), '/sources', 'exact historical source sequence required')
    for source in bindings.sources:
        require(re.fullmatch('[0-9a-f]{40}', source.commit) is not None
                and all(re.fullmatch('[0-9a-f]{64}', value) is not None
                        for value in (source.tracked_manifest_sha256, source.lock_sha256)), '/source', 'invalid source digest')
    require(bindings.sources[1].commit == '892edca1d31b26e535a3e9b62b37d62b99bb1130'
            and bindings.sources[2].commit == 'b1ca032f8fa84cee87d68a185e471b8e2c2bddce'
            and bindings.sources[2].tag_oid == '7db2907651b069f2db9dd843599abe1872e581b2'
            and bindings.sources[0].tag_oid is None and bindings.sources[1].tag_oid is None, '/sources', 'historical pin differs')
    require(len(dict(bindings.configs)) == len(bindings.configs), '/configs', 'duplicate configuration binding')
    for path, digest in (bindings.git, bindings.cargo, bindings.rustc):
        real_path(path, directory=False)
        require(file_hash(path) == digest, str(path), 'bound tool changed')
    found = shutil.which('git')
    require(found is not None and Path(found).resolve() == bindings.git[0], '/git', 'source inspector uses another git')
    for path, digest in bindings.configs:
        require(path.is_absolute() and path == path.resolve(), str(path), 'noncanonical configuration path')
        if digest is None:
            require(not path.exists() and not path.is_symlink(), str(path), 'absent configuration appeared')
        else:
            real_path(path, directory=False)
            require(file_hash(path) == digest, str(path), 'configuration changed')


def verify_target(launcher: Path, source: HistoricalSource) -> None:
    ref = {'main': 'origin/v8/main', 'm1': '892edca1', 'v7': 'v7-final-art'}[source.version]
    require(git(launcher, 'rev-parse', '--verify', ref + '^{commit}').decode().strip() == source.commit,
            '/ref', 'actual post-fetch source differs from expected source')
    if source.tag_oid is not None:
        require(git(launcher, 'rev-parse', '--verify', ref).decode().strip() == source.tag_oid
                and git(launcher, 'cat-file', '-t', source.tag_oid).strip() == b'tag',
                '/ref', 'annotated target object differs')


def selected_bridge(root: Path) -> tuple[Path, tuple[str, ...]]:
    values = [match.group(1) for match in re.finditer(r'^macos\.debug *= *"res://([^"\n]+)"$', (root / 'sim_bridge.gdextension').read_text(), re.MULTILINE)]
    require(len(values) == 1, '/gdextension', 'unique original macos.debug path required')
    relative = Path(values[0])
    require(not relative.is_absolute() and '..' not in relative.parts
            and str(relative) == values[0] and len(relative.parts) == 4
            and relative.parts[:2] == ('rust', 'target')
            and relative.parts[2] in ('debug', 'release'), '/gdextension', 'unsupported original bridge path')
    path = root / relative
    require(path.resolve() == path, '/gdextension', 'bridge path aliases another artifact')
    return path, ('cargo', 'build', '-p', 'sim-bridge', *(('--release',) if relative.parts[2] == 'release' else ()))


def validate_build_target(environment: Node, rust_root: Path) -> None:
    """Reject an inherited arm/shared target before granting original Cargo."""
    values=environment.mapping()
    if 'CARGO_TARGET_DIR' not in values:
        return  # Existing opt-out source grants retain their prior contract.
    raw=environment.field('CARGO_TARGET_DIR').text()
    require(bool(raw),'/target','empty explicit Cargo target')
    selected=Path(raw)
    effective=selected if selected.is_absolute() else rust_root/selected
    require(effective.resolve()==rust_root/'target','/target','Cargo target escapes original worktree')


@final
class SourceGrants:
    def __init__(self, bindings: HistoricalBindings, launcher: Path, play_root: Path) -> None:
        check_bindings(bindings)
        self.bindings = bindings
        self.launcher = launcher
        self.play_root = play_root
        self.started: dict[str, dict[str, Json]] = {}
        self.built: dict[str, BuiltBridge] = {}

    def declaration(self) -> dict[str, Json]:
        check_bindings(self.bindings)
        return {'schema': 'play_historical_bindings_v1',
            'sources': [{'version': source.version, 'commit': source.commit,
                'tracked_manifest_sha256': source.tracked_manifest_sha256, 'lock_sha256': source.lock_sha256, 'tag_oid': source.tag_oid}
                for source in self.bindings.sources],
            'tools': {name: {'path': str(path), 'sha256': digest}
                for name, (path, digest) in (('git', self.bindings.git), ('cargo', self.bindings.cargo), ('rustc', self.bindings.rustc))},
            'configs': {str(path): digest for path, digest in self.bindings.configs},
            'expected_source_authority': 'CALLER_FROZEN_BEFORE_LAUNCH',
            'transitive_runtime_closure': 'CALLER_OBLIGATION', 'observer_world_qualification': 'INCOMPLETE'}

    def source(self, version: str) -> HistoricalSource:
        values = [source for source in self.bindings.sources if source.version == version]
        require(len(values) == 1, '/version', 'unknown historical source')
        return values[0]

    def inspect(self, version: str, *, generated: bool) -> dict[str, Json]:
        check_bindings(self.bindings)
        source = self.source(version); root = self.play_root / version
        real_path(root, directory=True)
        common = Path(os.fsdecode(git(root, 'rev-parse', '--path-format=absolute', '--git-common-dir')).strip())
        launcher_common = Path(os.fsdecode(git(self.launcher, 'rev-parse', '--path-format=absolute', '--git-common-dir')).strip())
        require(common == launcher_common, '/worktree', 'foreign Git administration')
        configs = dict(self.bindings.configs)
        worktree_configs = {Path(os.fsdecode(git(path, 'rev-parse', '--absolute-git-dir')).strip()) / 'config.worktree'
                            for path in (root, self.launcher)}
        require(worktree_configs <= set(configs), '/configs', 'worktree Git configuration presence bindings missing')
        registered = git(self.launcher, 'worktree', 'list', '--porcelain', '-z').split(b'\0')
        require(os.fsencode('worktree ' + str(root)) in registered, '/worktree', 'unregistered historical worktree')
        verify_target(self.launcher, source)
        snapshot = freeze_source(root, source.commit)
        require(snapshot['tracked_manifest_sha256'] == source.tracked_manifest_sha256, '/source', 'independently frozen source manifest differs')
        require(file_hash(root / 'rust/Cargo.lock') == source.lock_sha256, '/lock', 'historical lock differs')
        ignored = [os.fsdecode(value) for value in git(root, 'ls-files', '--others', '--ignored', '--exclude-standard', '-z').split(b'\0') if value]
        configs = dict(self.bindings.configs)
        for name in ignored:
            bound_config = root / name in configs and Path(name).name in ('config', 'config.toml') and Path(name).parent.name == '.cargo'
            require(name == 'rust/Cargo.lock' or bound_config
                    or (generated and name.startswith(('rust/target/', '.godot/'))),
                    '/source', 'unbound ignored source: ' + name)
        if not generated:
            require(not (root / 'rust/target').exists() and not (root / '.godot').exists(), '/source', 'fresh original build/import required')
        require(common / 'config' in configs, '/configs', 'Git administration config missing')
        return snapshot

    def check_environment(self, payload: Node) -> None:
        environment = payload.field('environment').mapping()
        observed_git = shutil.which('git', path=text(Node(environment.get('PATH'))))
        require(observed_git is not None and Path(observed_git).resolve() == self.bindings.git[0],
                '/git', 'actual launcher child resolves another git')
        require(not any(key.startswith('GIT_') for key in environment), '/environment', 'Git override not qualified')

    def request(self, version: str, role: str, entry: Node, payload: Node) -> dict[str, Json]:
        root = self.play_root / version
        snapshot = self.inspect(version, generated=version in self.started)
        bridge, build_argv = selected_bridge(root)
        environment = payload.field('environment').mapping()
        self.check_environment(payload)
        home = Path(text(Node(environment.get('HOME'))))
        cargo_home = Path(text(Node(environment.get('CARGO_HOME', str(home / '.cargo')))))
        required = {base / '.cargo' / name for base in (root / 'rust', *(root / 'rust').parents) for name in ('config', 'config.toml')}
        required |= {cargo_home / name for name in ('config', 'config.toml')}
        required |= {home / '.gitconfig', home / '.config/git/config', Path('/etc/gitconfig').resolve()}
        require(required <= set(dict(self.bindings.configs)), '/configs', 'ancestral Cargo/Git configuration presence bindings missing')
        check_bindings(self.bindings)
        if role == 'build':
            validate_build_target(payload.field('environment'),root/'rust')
            require(version not in self.started, '/build', 'duplicate original build')
            payload.field('argv').equals(list(build_argv)); payload.field('cwd').equals(str(root / 'rust'))
            entry.field('executable').equals(str(self.bindings.cargo[0])); entry.field('executable_sha256').equals(self.bindings.cargo[1])
            entry.field('artifacts').equals([str(bridge)])
            for name, expected in (('cargo', self.bindings.cargo[0]), ('rustc', self.bindings.rustc[0])):
                resolved = shutil.which(name, path=text(Node(environment.get('PATH'))))
                require(resolved is not None and Path(resolved).resolve() == expected, '/tools', 'original tool resolution differs')
            require(environment.get('RUSTC', str(self.bindings.rustc[0])) == str(self.bindings.rustc[0])
                    and all(environment.get(key, '') == '' for key in ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER')),
                    '/tools', 'unqualified compiler override')
            self.started[version] = snapshot
        else:
            require(role in ('import-1', 'import-2', 'headless') and version in self.built, '/build', 'no authenticated original build return')
            self.check_built(version)
            args = ['--headless', '--path', '.', '--quit-after', '300'] if role == 'headless' else ['--headless', '--path', '.', '--import', '--editor', '--quit']
            actual = [text(value) for value in payload.field('argv').array()]
            require(len(actual) == len(args) + 1 and actual[1:] == args, '/argv', 'original Godot arguments changed')
            payload.field('cwd').equals(str(root))
        return {'schema': 'play_source_grant_v1', 'version': version, 'role': role, 'source': snapshot,
                'bridge_path': str(bridge), 'qualification': 'SOURCE_BINDING_ONLY', 'observer_world_qualification': 'INCOMPLETE'}

    def returned(self, version: str, receipt: Path) -> dict[str, Json]:
        require(version in self.started and version not in self.built, '/build', 'unexpected original build return')
        snapshot = self.inspect(version, generated=True)
        require(snapshot == self.started[version], '/source', 'source changed across actual build')
        bridge, argv = selected_bridge(self.play_root / version)
        raw_hash = file_hash(receipt); row = document(receipt.read_text())
        row.field('native_exit').equals(0); row.field('timed_out').equals(False); row.field('errors').equals([])
        row.field('argv').equals(list(argv)); row.field('cwd').equals(str(self.play_root / version / 'rust'))
        row.field('executable').equals(str(self.bindings.cargo[0])); row.field('executable_sha256').equals(self.bindings.cargo[1])
        row.field('declared_artifacts').equals([str(bridge)])
        digest = file_hash(bridge)
        row.field('artifact_hashes').equals({str(bridge): digest})
        pid = row.field('pid').value
        require(type(pid) is int and pid > 0, '/pid', 'missing actual build PID')
        assert isinstance(pid, int)
        require(file_hash(receipt) == raw_hash, '/receipt', 'build receipt changed while read')
        self.built[version] = BuiltBridge(bridge, digest, receipt, raw_hash, pid)
        return {'schema': 'play_build_join_v1', 'version': version, 'source': snapshot, 'bridge': str(bridge),
                'bridge_sha256': digest, 'build_receipt': str(receipt), 'build_receipt_sha256': raw_hash,
                'build_pid': pid, 'authority': 'JOINED_ORIGINAL_CHILD_RECEIPT_NOT_GENERIC_BUILD_RESULT',
                'observer_world_qualification': 'INCOMPLETE'}

    def check_built(self, version: str) -> None:
        built = self.built[version]
        require(file_hash(built.path) == built.sha256 and file_hash(built.receipt) == built.receipt_sha256,
                '/build', 'actual original build evidence changed')
        require(selected_bridge(self.play_root / version)[0] == built.path, '/bridge', 'loaded bridge selection changed')

    def check(self) -> None:
        check_bindings(self.bindings)
        for version, snapshot in self.started.items():
            require(self.inspect(version, generated=True) == snapshot, '/source', 'admitted historical source changed')
        for version in self.built:
            self.check_built(version)
