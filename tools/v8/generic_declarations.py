# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_declarations.py
"""Read committed fixture declarations with real ancestry and clean-source evidence."""
import hashlib
import re
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Final, Literal

from generic_gate import boundary, document, require, text
from generic_sources import freeze_source, git
from mux2_gate_inputs import Json

PROTECTED: Final = frozenset({
    "baseline_identity", "f10_expected_control", "identity.baseline_samples",
    "identity.baseline_twins", "determinism.baseline", "core.baseline10",
    "knowledge.f10_injected",
})


@dataclass(frozen=True, slots=True)
class Change:
    fixture_id: str
    base: str
    behavior_commit: str
    reason: str
    declaration_path: str
    declaration_commit: str
    behavior_paths: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class Declarations:
    candidate: str
    base: str
    purpose: str
    known_fixtures: tuple[str, ...]
    ancestors: tuple[str, ...]
    changes: tuple[Change, ...]
    files: tuple[tuple[str, str, str], ...]
    ignored_historical: tuple[str, ...]
    source_manifest_sha256: str

    def validator_fields(self, fixture_id: str) -> dict[str, Json]:
        """Return Git-derived fields; comparator still supplies equality and safety."""
        require(fixture_id in self.known_fixtures, fixture_id, 'unknown fixture')
        selected = next((row for row in self.changes if row.fixture_id == fixture_id), None)
        entries: list[Json] = []
        if selected is not None:
            entries.append({'fixture_id': selected.fixture_id, 'base': selected.base,
                            'behavior_commit': selected.behavior_commit, 'reason': selected.reason})
        return {'purpose': self.purpose, 'base': self.base, 'candidate': self.candidate,
                'known_fixtures': list(self.known_fixtures), 'fixture_id': fixture_id,
                'candidate_ancestors_after_base': list(self.ancestors), 'declarations': entries,
                'file_committed': selected is not None,
                'file_commit': selected.declaration_commit if selected is not None else None}


def declaration_path(name: str) -> None:
    path = PurePosixPath(name)
    require(path.as_posix() == name and not path.is_absolute() and len(path.parts) >= 3
            and path.parts[0] == 'docs' and path.parts[-1] == 'CHANGED_FIXTURES.json'
            and all(part not in {'.', '..'} and re.fullmatch(r'[A-Za-z0-9_.-]+', part)
                    for part in path.parts[1:-1]), name, 'expected docs/<pass>/CHANGED_FIXTURES.json')


def commit(root: Path, value: str) -> None:
    require(re.fullmatch('[0-9a-f]{40}', value) is not None, '/commit', 'full literal commit SHA required')
    require(git(root, 'rev-parse', '--verify', value + '^{commit}').decode().strip() == value,
            value, 'not a commit object')


def tree_files(root: Path, source: str) -> dict[str, tuple[str, str]]:
    result: dict[str, tuple[str, str]] = {}
    for raw in git(root, 'ls-tree', '-rz', source, '--', 'docs').split(b'\0'):
        if raw:
            header, filename = raw.split(b'\t', 1)
            name = filename.decode()
            if PurePosixPath(name).name == 'CHANGED_FIXTURES.json':
                declaration_path(name)
                mode, kind, oid = header.decode().split()
                require(kind == 'blob' and mode in {'100644', '100755'}, name, 'declaration must be regular file')
                result[name] = (mode, oid)
    return result


@boundary
def read_declarations(root: Path, candidate: str, base: str,
                      purpose: Literal['publication', 'self_proof', 'defect_proof'],
                      known_fixtures: tuple[str, ...]) -> Declarations:
    """Authenticate active declarations; unchanged historical allowlists never carry forward."""
    require(purpose in {'publication', 'self_proof', 'defect_proof'}, '/purpose', 'unknown purpose')
    require(bool(known_fixtures) and len(set(known_fixtures)) == len(known_fixtures)
            and all(name.strip() for name in known_fixtures), '/known_fixtures', 'empty or duplicate registry')
    commit(root, candidate)
    commit(root, base)
    _ = git(root, 'merge-base', '--is-ancestor', base, candidate)
    source_before = freeze_source(root, candidate)
    ancestors = tuple(git(root, 'rev-list', '--topo-order', candidate, '^' + base).decode().splitlines())
    base_files, current_files = tree_files(root, base), tree_files(root, candidate)
    changes: list[Change] = []
    files: list[tuple[str, str, str]] = []
    ignored: list[str] = []
    seen: set[str] = set()
    for name, (_, blob) in sorted(current_files.items()):
        if name in base_files and base_files[name][1] == blob:
            ignored.append(name)
            continue
        payload = git(root, 'show', candidate + ':' + name)
        node = document(payload.decode())
        require(set(node.mapping()) == {'schema_version', 'base_commit', 'changes'}, name, 'unexpected declaration schema')
        node.field('schema_version').equals(1)
        node.field('base_commit').equals(base)
        file_commit = git(root, 'log', '-1', '--format=%H', candidate, '--', name).decode().strip()
        commit(root, file_commit)
        require(file_commit in ancestors, name, 'declaration not committed after BASE')
        require(git(root, 'show', file_commit + ':' + name) == payload,
                name, 'last file commit does not contain candidate declaration bytes')
        files.append((name, file_commit, hashlib.sha256(payload).hexdigest()))
        for row in node.field('changes').array():
            require(set(row.mapping()) == {'fixture_id', 'behavior_commit', 'reason'}, name, 'unexpected change schema')
            fixture = text(row.field('fixture_id'))
            require(fixture in known_fixtures and fixture not in seen, fixture, 'unknown or duplicate fixture ID')
            require(fixture not in PROTECTED, fixture, 'protected fixture cannot be declared changed')
            behavior = text(row.field('behavior_commit'))
            commit(root, behavior)
            require(behavior in ancestors and behavior != base, fixture, 'behavior is not post-BASE candidate ancestor')
            _ = git(root, 'merge-base', '--is-ancestor', base, behavior)
            require(behavior != file_commit, fixture, 'declaration must be a later commit than behavior')
            _ = git(root, 'merge-base', '--is-ancestor', behavior, file_commit)
            paths = tuple(part.decode() for part in git(root, 'diff-tree', '--no-commit-id',
                          '--name-only', '-r', '-z', behavior + '^1', behavior).split(b'\0') if part)
            require(bool(paths), behavior, 'behavior commit contains no changed paths')
            require(any(not path.startswith('docs/') and not path.lower().endswith('.md') for path in paths),
                    behavior, 'documentation-only commit is not behavior evidence')
            reason = text(row.field('reason'))
            changes.append(Change(fixture, base, behavior, reason, name, file_commit, paths))
            seen.add(fixture)
    require(purpose == 'publication' or not changes, '/changes', 'proofs require zero declarations')
    source_after = freeze_source(root, candidate)
    require(source_after == source_before, '/source', 'candidate source changed while reading declarations')
    manifest = source_before['tracked_manifest_sha256']
    require(isinstance(manifest, str), '/source', 'source manifest digest missing')
    assert isinstance(manifest, str)
    return Declarations(candidate, base, purpose, known_fixtures, ancestors, tuple(changes),
                        tuple(files), tuple(ignored), manifest)
