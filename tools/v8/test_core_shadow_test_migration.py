"""Synthetic controls for the exact test-only freeze amendment; no simulations."""
from __future__ import annotations

import json
from pathlib import Path
import shutil
import tarfile
import unittest
from unittest.mock import patch

import core_shadow_freeze as freeze
import core_shadow_test_migration as migration
import test_core_shadow_freeze as fixtures


class MigrationTests(unittest.TestCase):
    def setUp(self) -> None:
        fixture = fixtures.FreezeTests(methodName='runTest')
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        self.fixture = fixture
        self.repo, self.root = fixture.repo, fixture.root
        first = b'use crate::worldgen::{build_world_with, WorldGenConfig};\n'
        for name in migration.TEST_FILES:
            path = self.repo / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(first)
        (self.repo / migration.TEST_FILES[0]).write_bytes(first + b'#[path = "system_rest_tests.rs"]\nmod rest_regressions;\n')
        owner = self.repo / migration.OWNER
        owner.write_text('#[cfg(test)]\nmod system_tests;\n')
        helper = self.repo / migration.HELPER
        helper.parent.mkdir(parents=True, exist_ok=True)
        helper.write_text('synthetic pre-migration worldgen\n')
        fixture.snapshot = self.root / 'migration-snapshot.json'
        freeze.prepare(self.repo, fixture.snapshot)
        fixture.write_receipt()
        self.manifest_sha = fixture.seal()
        self.archive = self.root / 'source.tar.gz'
        with tarfile.open(self.archive, 'w:gz') as tar:
            for path in sorted(self.repo.rglob('*')):
                if path.is_file():
                    tar.add(path, arcname=path.relative_to(self.repo).as_posix())
        checker = self.root / 'original_checker.py'
        shutil.copyfile(Path(freeze.__file__), checker)
        changes = {}
        for name in migration.TEST_FILES:
            path = self.repo / name
            before = path.read_bytes()
            path.write_bytes(before.replace(migration.OLD_IMPORT, migration.NEW_IMPORT))
            changes[name] = {'before_sha256': migration.bytes_sha(before), 'after_sha256': freeze.digest(path)}
        before = freeze.digest(helper)
        helper.write_text(helper.read_text() + migration.HELPER_ADDITION)
        self.data = {'schema_version': 1, 'kind': 'exact_test_import_migration',
            'manifest_sha256': self.manifest_sha, 'original_checker': freeze.reference(self.repo, checker),
            'source_archive': freeze.reference(self.repo, self.archive), 'changes': changes,
            'cfg_test_owner_sha256': freeze.digest(owner), 'review': 'synthetic exact test-only migration',
            'worldgen_test_helper': {'path': migration.HELPER, 'before_sha256': before,
                'after_sha256': freeze.digest(helper), 'addition': migration.HELPER_ADDITION}}
        self.amendment = self.root / 'amendment.json'
        self.write_amendment()

    def write_amendment(self) -> None:
        self.amendment.write_text(json.dumps(self.data))
        self.amendment_sha = freeze.digest(self.amendment)

    def check(self) -> None:
        migration.check(self.repo, self.fixture.manifest, self.manifest_sha,
                        self.amendment, self.amendment_sha)

    def test_exact_transition_passes_original_checker_still_rejects_real_tree(self) -> None:
        with self.assertRaises(freeze.FreezeError):
            freeze.check(self.repo, self.fixture.manifest, self.manifest_sha)
        self.check()
        for name in migration.TEST_FILES:
            self.assertIn(migration.NEW_IMPORT, (self.repo / name).read_bytes())

    def test_production_policy_owner_and_third_file_changes_fail(self) -> None:
        for name in (freeze.REQUIRED_FILES[0], migration.OWNER, migration.TEST_FILES[0], migration.HELPER):
            path = self.repo / name
            before = path.read_bytes()
            path.write_bytes(before + b'changed')
            with self.subTest(name=name), self.assertRaises(freeze.FreezeError):
                self.check()
            path.write_bytes(before)
        extra = self.repo / freeze.CURRENT_DIRS[0] / 'new_test.rs'
        extra.write_text('new file')
        with self.assertRaises(freeze.FreezeError):
            self.check()
        extra.unlink()

    def test_wrong_hash_extra_path_and_missing_path_fail_even_if_repinned(self) -> None:
        original = json.dumps(self.data)
        for mutation in ('before', 'after', 'extra', 'missing'):
            self.data = json.loads(original)
            changes = self.data['changes']
            if mutation in ('before', 'after'):
                changes[migration.TEST_FILES[0]][mutation + '_sha256'] = '0' * 64
            if mutation == 'extra':
                changes['third.rs'] = changes[migration.TEST_FILES[0]]
            if mutation == 'missing':
                changes.pop(migration.TEST_FILES[1])
            self.write_amendment()
            with self.subTest(mutation=mutation), self.assertRaises(freeze.FreezeError):
                self.check()

    def test_tampered_manifest_archive_checker_binary_and_amendment_fail(self) -> None:
        for path in (self.fixture.manifest, self.archive, Path(self.data['original_checker']['path']),
                     self.fixture.shadow, self.amendment):
            before = path.read_bytes()
            path.write_bytes(before + b'tamper')
            with self.subTest(path=path), self.assertRaises(freeze.FreezeError):
                self.check()
            path.write_bytes(before)

    def test_duplicate_amendment_keys_fail_even_if_repinned(self) -> None:
        self.amendment.write_text(self.amendment.read_text().replace(
            '"schema_version": 1', '"schema_version": 1, "schema_version": 1', 1))
        self.amendment_sha = freeze.digest(self.amendment)
        with self.assertRaises(freeze.FreezeError):
            self.check()

    def test_symlink_and_deleted_test_fail(self) -> None:
        path = self.repo / migration.TEST_FILES[0]
        before = path.read_bytes()
        path.unlink()
        with self.assertRaises(freeze.FreezeError):
            self.check()
        target = self.root / 'outside-test.rs'
        target.write_bytes(before)
        path.symlink_to(target)
        with self.assertRaises(freeze.FreezeError):
            self.check()

    def test_mutation_during_projection_fails_and_temp_view_is_cleaned(self) -> None:
        copy = shutil.copyfile
        mutated = False
        destinations = []
        def change(source, destination):
            nonlocal mutated
            result = copy(source, destination)
            destinations.append(Path(destination))
            if not mutated:
                (self.repo / migration.TEST_FILES[0]).write_bytes(b'concurrent modification')
                mutated = True
            return result
        with patch.object(migration.shutil, 'copyfile', side_effect=change):
            with self.assertRaises(freeze.FreezeError):
                self.check()
        self.assertTrue(destinations)
        self.assertTrue(all(not path.exists() for path in destinations))


if __name__ == '__main__':
    unittest.main()
