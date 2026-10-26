#!/usr/bin/env python3
"""Verify an explicitly pinned test-only migration through the original checker.

The real checkout is never rewritten. A temporary projected source view restores
exactly two frozen test imports after current-source equality is checked in full.
This verifies an amendment; it does not claim original byte-identical sources.
"""
from __future__ import annotations

import importlib.util
from pathlib import Path
import shutil
import tarfile
import tempfile
from typing import Final

import core_shadow_freeze as frozen

TEST_FILES: Final = (
    'rust/crates/sim-systems/src/runtime/mind/system_tests.rs',
    'rust/crates/sim-systems/src/runtime/mind/system_rest_tests.rs',
)
OWNER: Final = 'rust/crates/sim-systems/src/runtime/mind/mod.rs'
HELPER: Final = 'rust/crates/sim-systems/src/worldgen/mod.rs'
OLD_IMPORT: Final = b'build_world_with, WorldGenConfig'
NEW_IMPORT: Final = b'build_world_without_runtime_for_tests as build_world_with, WorldGenConfig'
HELPER_ADDITION: Final = '''/// Isolated generated world for tests that manually execute their own runtime systems.
#[cfg(test)]
pub(crate) fn build_world_without_runtime_for_tests(
    cfg: &WorldGenConfig,
    rules: WorldRules,
) -> SimEngine {
    build_world_with_registrar(cfg, rules, |_| {})
}

'''


def require(condition: bool, message: str) -> None:
    if not condition:
        raise frozen.FreezeError(message)


def bytes_sha(raw: bytes) -> str:
    import hashlib
    return hashlib.sha256(raw).hexdigest()


def check(repo: Path, manifest: Path, manifest_sha: str,
          amendment_path: Path, amendment_sha: str) -> None:
    """Validate real sources, original bytes and binaries, then clean projection."""
    repo = frozen.canonical(repo)
    frozen.external(repo, amendment_path)
    frozen.pin(amendment_path, amendment_sha)
    frozen.external(repo, manifest)
    frozen.pin(manifest, manifest_sha)
    amendment = frozen.load(amendment_path)
    frozen.version(amendment)
    require(set(amendment) == {'schema_version', 'kind', 'manifest_sha256', 'original_checker',
        'source_archive', 'changes', 'cfg_test_owner_sha256', 'worldgen_test_helper', 'review'},
        'invalid test migration fields')
    require(amendment['kind'] == 'exact_test_import_migration'
            and amendment['manifest_sha256'] == manifest_sha, 'migration manifest mismatch')
    require(isinstance(amendment['review'], str) and bool(amendment['review']), 'missing explicit review')
    original = frozen.load(manifest)
    checker = frozen.read_reference(repo, amendment['original_checker'])
    require(frozen.digest(checker) == original['checker_sha256']
            == frozen.digest(Path(frozen.__file__).resolve()), 'original checker bytes changed')
    archive = frozen.read_reference(repo, amendment['source_archive'])
    changes = amendment['changes']
    require(isinstance(changes, dict) and set(changes) == set(TEST_FILES), 'exact two test paths required')
    before_inventory = frozen.inventory(repo)
    expected = dict(original['current_files'])
    require(set(before_inventory) == set(expected), 'source path inventory changed')
    frozen_bytes: dict[str, bytes] = {}
    with tarfile.open(archive, 'r:gz') as tar:
        names = [member.name for member in tar.getmembers()]
        require(len(names) == len(set(names)), 'duplicate archive members')
        for name in TEST_FILES:
            row = changes[name]
            require(isinstance(row, dict) and set(row) == {'before_sha256', 'after_sha256'}, 'invalid change row')
            member = tar.getmember(name)
            require(member.isfile(), 'nonregular test archive member')
            stream = tar.extractfile(member)
            require(stream is not None, 'missing frozen test bytes')
            with stream:
                before = stream.read()
            require(bytes_sha(before) == row['before_sha256'] == original['current_files'][name]
                    == original['frozen_files'][name], 'wrong before test hash')
            require(before.count(OLD_IMPORT) == 1, 'expected exactly one frozen import')
            after = (repo / name).read_bytes()
            require(after == before.replace(OLD_IMPORT, NEW_IMPORT)
                    and bytes_sha(after) == row['after_sha256'], 'not exact reviewed import substitution')
            expected[name] = row['after_sha256']
            frozen_bytes[name] = before
    require(before_inventory == expected, 'production or unapproved source inventory changed')
    owner = (repo / OWNER).read_bytes()
    require(bytes_sha(owner) == amendment['cfg_test_owner_sha256'] == original['current_files'][OWNER]
            and b'#[cfg(test)]\nmod system_tests;' in owner, 'test owner boundary changed')
    require(b'#[path = "system_rest_tests.rs"]\nmod rest_regressions;' in frozen_bytes[TEST_FILES[0]],
            'nested test module boundary changed')
    helper = amendment['worldgen_test_helper']
    require(isinstance(helper, dict) and set(helper) == {'path', 'before_sha256', 'after_sha256', 'addition'},
            'invalid helper evidence')
    require(helper['path'] == HELPER and helper['addition'] == HELPER_ADDITION, 'helper patch scope mismatch')
    helper_path = frozen.canonical(repo / HELPER)
    helper_bytes = helper_path.read_bytes()
    addition = HELPER_ADDITION.encode()
    require(helper_bytes.count(addition) == 1 and bytes_sha(helper_bytes) == helper['after_sha256']
            and bytes_sha(helper_bytes.replace(addition, b'', 1)) == helper['before_sha256'],
            'helper differs from exact reviewed cfg(test) addition')
    spec = importlib.util.spec_from_file_location('core_original_shadow_checker', checker)
    require(spec is not None and spec.loader is not None, 'cannot load preserved checker')
    original_checker = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(original_checker)
    with tempfile.TemporaryDirectory(prefix='core-test-migration-') as temporary:
        projection = Path(temporary).resolve()
        for directory in frozen.CURRENT_DIRS:
            (projection / directory).mkdir(parents=True, exist_ok=True)
        for name in sorted(before_inventory):
            destination = projection / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            if name in frozen_bytes:
                destination.write_bytes(frozen_bytes[name])
            else:
                shutil.copyfile(repo / name, destination)
        try:
            original_checker.check(projection, manifest, manifest_sha)
        except original_checker.FreezeError as error:
            raise frozen.FreezeError(str(error)) from error
    require(frozen.inventory(repo) == before_inventory, 'source changed during amendment verification')
    require(helper_path.read_bytes() == helper_bytes, 'helper changed during verification')
    frozen.pin(amendment_path, amendment_sha)
    frozen.read_reference(repo, amendment['original_checker'])
    frozen.read_reference(repo, amendment['source_archive'])
    print('TEST_ONLY_AMENDMENT_VERIFIED exact two imports; original manifest/binaries unchanged')
