"""Pinned historical producer origins for full serialized Chronicle admission."""
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_gate import GateContractError, boundary, document, require
from generic_process import file_hash
from generic_sources import git
from generic_test_endpoint_types import bound_bytes
from generic_workspace_cases import digest
from mux2_gate_inputs import Json, Node

COMMITS: Final = {
    'historical/m55': 'ae443aa4a498680ef56211faa6b993533019a6d2',
    'historical/mart2': '32cd61a77cb26886be5f13b9924fa55b7d64cbe0',
    'base': 'ca9f531ca6ef700ca223d99dba14d834b94b45da',
    'b4/h-format9-fixture': 'bdbd4fb2ea9c5b446648778ea7b0b2da14635049',
}
CRITICAL: Final = ('rust/crates/sim-core/src/chronicle.rs',
    'rust/crates/sim-systems/src/save/mod.rs',
    'rust/crates/sim-systems/src/runtime/knowledge/aggregate.rs',
    'rust/crates/sim-engine/src/lib.rs',
    'rust/crates/sim-systems/src/worldgen/colony.rs',
    'rust/crates/sim-test/src/probe/colony.rs',
    'rust/crates/sim-test/src/probe/runner.rs',
    'rust/crates/sim-test/src/scenes.rs')
FREEZE9: Final = '1c97cab1b5645cac669c92df02775020bc9c6ac1a08cdfef1477b36aa52d1bb3'


@dataclass(frozen=True, slots=True)
class Origin:
    save: Path
    save_sha256: str
    version: int
    source_commit: str | None
    source_blobs: tuple[tuple[str, str], ...]
    retained: tuple[tuple[Path, str], ...]
    missing: tuple[str, ...]

    def report(self) -> dict[str, Json]:
        return {'status': 'INCOMPLETE_FULL_HISTORY_ORIGIN' if self.missing else 'SOURCE_QUALIFIED',
                'save': str(self.save), 'save_sha256': self.save_sha256, 'version': self.version,
                'source_commit': self.source_commit, 'source_blobs': dict(self.source_blobs),
                'retained_provenance': {str(path): sha for path, sha in self.retained},
                'missing': list(self.missing),
                'policy': 'reviewed_append_only_Chronicle_complete_resources_serialization_no_producer_reset'}


def recheck(origin: Origin) -> None:
    for path, sha in origin.retained:
        _ = bound_bytes(path, sha)


@boundary
def authenticate(session: Node, directory: Path, save: Path, version: int) -> Origin:
    """Source-qualified provenance, never a caller's full-history-complete flag.

    The closed historical commit set is reviewed in the full-history origin audit.
    v8's dirty parent and v10's unbound bridge are intentionally not substituted.
    """
    anchors = session.field('external_inputs')
    retained: list[tuple[Path, str]] = []
    missing: list[str] = []

    def read(path: Path, expected: str | None = None) -> bytes | None:
        if str(path) not in anchors.mapping():
            missing.append('unbound retained origin input: ' + str(path))
            return None
        sha = anchors.field(str(path)).text()
        require(expected is None or sha == expected, str(path), 'origin provenance hash differs')
        data = bound_bytes(path, sha)
        retained.append((path, sha))
        return data

    save_bytes = read(save)
    require(save_bytes is not None, '/save', 'migration save lacks original session binding')
    assert save_bytes is not None
    require(save_bytes[:12] == b'WORLDSIM' + version.to_bytes(4, 'little'), '/save', 'wrong legacy origin version')
    save_sha = digest(save_bytes)
    if version in (8, 10):
        reason = ('dirty format8 producer closure absent; clean format7 parent is not producer'
                  if version == 8 else 'original format10 bridge/capture source closure absent')
        return Origin(save, save_sha, version, None, (), tuple(retained), (reason,))
    relative = save.relative_to(directory).as_posix()
    scope = next((name for name in COMMITS if relative.startswith(name + '/')), None)
    require(scope is not None, '/origin', 'save outside reviewed historical source scope')
    assert scope is not None
    commit = COMMITS[scope]
    expected_version = 6 if scope.startswith('historical/') else 7 if scope == 'base' else 9
    require(version == expected_version, '/origin', 'origin source/version mismatch')
    manifest: Node | None = None
    if version == 6:
        acquisition = read(directory / 'historical/acquisition-receipt.json',
                           'd75171a4d51a9867e37c5bb670412511b16e05c394dbf47ae55f2ec05640508e')
        source = read(directory / scope / 'source-commit.txt')
        if acquisition is not None:
            row = document(acquisition.decode()).field(scope.split('/')[1])
            row.field('source').equals(commit)
            row.field('format').equals(6)
            tick = save.stem.removeprefix('tick-')
            row.field('saves').field(tick).equals(save_sha)
        if source is not None:
            require(source.decode().strip() == commit, '/source', 'historical source commit changed')
    elif version == 7:
        provenance = read(directory / 'base/provenance.json',
                          '109c3dda2e1b2d5583d0fab115d4df31677716b3d29f2bed566abb4213c17db4')
        if provenance is not None:
            document(provenance.decode()).field('head').equals(commit)
        hashes = read(directory / 'base/source-sha256.json')
        if hashes is not None:
            manifest = document(hashes.decode())
    else:
        receipt = read(directory / 'b4/h-format9-fixture/receipt.json',
                       '2fdae7bcf9481347afaeda967f49144febadff727258f3f6eba8e38d2cf15bbc')
        if receipt is not None:
            row = document(receipt.decode())
            row.field('save_sha256').equals(save_sha)
            row.field('source_freeze_sha256').equals(FREEZE9)
            row.field('exit_code').equals(0)
        freeze = read(directory / 'b4/BPTH/source-freeze.json', FREEZE9)
        if freeze is not None:
            manifest = document(freeze.decode()).field('files')
    selector_path = Path(session.field('selector_path').text())
    selector = document(bound_bytes(selector_path, session.field('selector_sha256').text()).decode())
    repository = Path(selector.field('selectors').field('harness_root').text())
    blobs: list[tuple[str, str]] = []
    try:
        _ = git(repository, 'cat-file', '-e', commit + '^{commit}')
    except GateContractError:
        missing.append('reviewed historical Git commit unavailable: ' + commit)
    else:
        for name in CRITICAL:
            sha = digest(git(repository, 'show', commit + ':' + name))
            if manifest is not None:
                manifest.field(name).equals(sha)
            blobs.append((name, sha))
    require(file_hash(selector_path) == session.field('selector_sha256').text(), '/selector', 'origin repository binding changed')
    result = Origin(save, save_sha, version, commit, tuple(blobs), tuple(retained), tuple(missing))
    recheck(result)
    return result
