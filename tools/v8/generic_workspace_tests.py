"""Structural Cargo inventory only; no source/build or world-safety admission."""
from dataclasses import dataclass
from pathlib import Path

from generic_gate import require
from mux2_gate_inputs import Node


@dataclass(frozen=True, slots=True)
class Target:
    package_id: str
    package: str
    name: str
    kinds: tuple[str, ...]
    source: Path
    manifest: Path
    doctest: bool
    test: bool


@dataclass(frozen=True, slots=True)
class Executable:
    target: Target
    binary: Path
    step: int
    profile: str
    filter: str | None = None
    ignored: str = 'exclude'
    threads: int = 8

    @property
    def cwd(self) -> Path:
        return self.target.manifest.parent


@dataclass(frozen=True, slots=True)
class Inventory:
    executables: tuple[Executable, ...]
    doctests: tuple[Target, ...]
    admission: str = 'INCOMPLETE_SOURCE_BUILD_HANDOFF_REQUIRED'
    worlds: str = 'INCOMPLETE_UNCLASSIFIED_NATIVE_WORLDS'
    doctest_execution: str = 'NOT_OBSERVED'


def boolean(node: Node) -> bool:
    require(type(node.value) is bool, node.label, 'expected boolean')
    return node.value is True


def target(node: Node, package: Node) -> Target:
    kinds = tuple(row.text() for row in node.field('kind').array())
    require(bool(kinds) and len(set(kinds)) == len(kinds), '/target', 'empty/duplicate kind')
    require(set(kinds) <= {'lib','rlib','staticlib','cdylib','dylib','proc-macro','bin','test','example','bench','custom-build'},
            '/target', 'unknown target kind')
    required = node.mapping().get('required-features', [])
    require(required == [], '/target', 'conditional target needs authenticated resolved feature selection')
    return Target(package.field('id').text(),package.field('name').text(),node.field('name').text(),kinds,
                  Path(node.field('src_path').text()),Path(package.field('manifest_path').text()),
                  boolean(node.field('doctest')),boolean(node.field('test')))


def inspect_inventory(metadata: Node, messages: tuple[Node, ...], step: int) -> Inventory:
    """Reject incomplete structural sets; authenticated build handoff remains mandatory."""
    require(type(step) is int and step in {4,5}, '/step', 'only original workspace debug/release steps')
    profile = 'debug' if step == 4 else 'release'
    root = Path(metadata.field('workspace_root').text())
    output = Path(metadata.field('target_directory').text())/profile
    require(root.is_absolute() and output.is_absolute() and '..' not in root.parts and '..' not in output.parts, '/paths', 'absolute workspace/target paths required')
    members = tuple(row.text() for row in metadata.field('workspace_members').array())
    require(bool(members) and len(set(members)) == len(members), '/members', 'empty/duplicate members')
    packages: dict[str,Node] = {}
    for package in metadata.field('packages').array():
        key = package.field('id').text()
        require(key not in packages, key, 'duplicate package ID')
        packages[key] = package
    require(set(members) <= set(packages), '/members', 'member absent from metadata packages')
    targets: dict[tuple[str,str,tuple[str,...]],Target] = {}
    for member in members:
        package = packages[member]
        for item in package.field('targets').array():
            row = target(item,package)
            require(row.source.is_absolute() and row.manifest.is_absolute() and '..' not in row.source.parts
                    and '..' not in row.manifest.parts and row.source.is_relative_to(row.manifest.parent) and row.manifest.is_relative_to(root),
                    row.name, 'foreign workspace target source/manifest')
            key = (member,row.name,row.kinds)
            require(key not in targets, row.name, 'duplicate metadata target')
            targets[key] = row
    expected = {key:row for key,row in targets.items() if row.test}
    require(bool(expected), '/targets', 'empty native workspace inventory')
    require(bool(messages) and messages[-1].field('reason').text() == 'build-finished', '/compiler', 'missing final compiler terminal')
    terminal = [row for row in messages if row.field('reason').text() == 'build-finished']
    require(len(terminal) == 1, '/compiler', 'duplicate compiler terminal')
    terminal[0].field('success').equals(True)
    executables: dict[tuple[str,str,tuple[str,...]],Executable] = {}
    seen_paths: set[Path] = set()
    for message in messages:
        reason = message.field('reason').text()
        require(reason in {'compiler-artifact','compiler-message','build-script-executed','build-finished'}, '/compiler', 'unknown compiler message')
        if reason != 'compiler-artifact' or not boolean(message.field('profile').field('test')):
            continue
        member = message.field('package_id').text()
        require(member in members, '/compiler', 'foreign test-profile artifact')
        item = message.field('target')
        key = (member,item.field('name').text(),tuple(row.text() for row in item.field('kind').array()))
        require(key in expected and key not in executables, '/compiler', 'extra/duplicate selected test artifact')
        declared = expected[key]
        item.field('src_path').equals(str(declared.source))
        message.field('manifest_path').equals(str(declared.manifest))
        message.field('fresh').equals(False)
        binary = Path(message.field('executable').text())
        artifact_directory = output/('examples' if declared.kinds == ('example',) else 'deps')
        require(binary.parent == artifact_directory and binary not in seen_paths and '..' not in binary.parts,
                '/binary', 'foreign/shared/noncanonical executable path')
        require(str(binary) in [row.text() for row in message.field('filenames').array()], '/binary', 'executable not among compiler filenames')
        seen_paths.add(binary)
        executables[key] = Executable(declared,binary,step,profile)
    require(set(executables) == set(expected), '/compiler', 'missing selected workspace test artifacts')
    return Inventory(tuple(executables[key] for key in sorted(executables)),
                     tuple(row for row in targets.values() if row.doctest))
