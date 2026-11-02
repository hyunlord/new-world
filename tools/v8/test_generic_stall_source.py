"""Tiny real Git source fixtures; no WorldSim, build or capture qualification."""
import hashlib
import json
import subprocess
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_stall_source as source
from generic_build import Overlay, derived_snapshot
from generic_gate import GateContractError
from generic_sources import freeze_source
from mux2_gate_inputs import Json


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class Fixture:
    def __init__(self, home: Path) -> None:
        self.root: Path = home / 'product'
        self.root.mkdir()
        _ = self.run('init', '-q')
        _ = self.run('config', 'user.email', 'source@example.invalid')
        _ = self.run('config', 'user.name', 'Source test')
        (self.root / 'tools/v8').mkdir(parents=True)
        self.old: Path = self.root / 'tools/v8/observer.py'
        _ = self.old.write_text('old\n')
        _ = (self.root / 'stable').write_text('stable\n')
        _ = self.run('add', '.')
        _ = self.run('commit', '-qm', 'Origin fixture')
        self.sha: str = self.run('rev-parse', 'HEAD').decode().strip()
        _ = self.old.write_text('new\n')
        self.add: Path = self.root / 'tools/v8/added.py'
        _ = self.add.write_text('added\n')
        self.patch: Path = home / 'overlay.patch'
        _ = self.patch.write_bytes(self.run('diff', '--no-ext-diff') + (
            b'diff --git a/tools/v8/added.py b/tools/v8/added.py\nnew file mode 100644\n'
            b'--- /dev/null\n+++ b/tools/v8/added.py\n@@ -0,0 +1 @@\n+added\n'))
        self.manifest: Path = home / 'overlay.json'
        self.rows: list[Json] = [
            {'path':'tools/v8/observer.py', 'operation':'modify', 'base0_sha256':digest(b'old\n'),
             'observer_sha256':digest(b'new\n')},
            {'path':'tools/v8/added.py', 'operation':'add', 'base0_sha256':None,
             'observer_sha256':digest(b'added\n')}]
        self.write_manifest()

    def run(self, *args: str) -> bytes:
        return subprocess.run(['git', '-C', str(self.root), *args], check=True, capture_output=True).stdout

    def write_manifest(self) -> None:
        _ = self.manifest.write_text(json.dumps({'schema_version':1, 'product_base_sha':self.sha,
            'observer_patch_sha256':digest(self.patch.read_bytes()),
            'allowed_paths':['tools/v8/observer.py','tools/v8/added.py'], 'files':self.rows}))

    def spec(self) -> source.DerivedSourceSpec:
        return source.DerivedSourceSpec(self.root, self.sha, Overlay(self.patch, digest(self.patch.read_bytes()),
                                                                    self.manifest, digest(self.manifest.read_bytes())))


class SourceTests(unittest.TestCase):
    def test_exact_add_modify_is_read_only_and_has_no_build_authority(self) -> None:
        # Given an applied observer fixture with a pristine origin index.
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            before = (f.run('ls-files', '--stage', '-z'), f.run('diff'), f.old.read_bytes(), f.add.read_bytes())
            # When source consistency is authenticated and rechecked.
            result = source.authenticate_derived_source(f.spec())
            self.assertEqual(source.recheck_derived_source(result), result)
            # Then source only is bound and no mutation occurred.
            self.assertEqual((result.build_admission, result.runtime_admission, result.observer_semantics),
                             ('PENDING', 'PENDING', 'NOT_RUN'))
            self.assertFalse(result.publication_authority)
            self.assertEqual(before, (f.run('ls-files', '--stage', '-z'), f.run('diff'), f.old.read_bytes(), f.add.read_bytes()))
            with self.assertRaises(GateContractError):
                _ = freeze_source(f.root, f.sha)

    def test_changed_worktree_boundaries_reject(self) -> None:
        for mutation in ('unchanged_blob', 'postimage', 'mode', 'symlink', 'untracked', 'index', 'head', 'missing'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temp:
                # Given a valid overlay with one changed boundary.
                f = Fixture(Path(temp).resolve())
                spec = f.spec()
                if mutation == 'unchanged_blob':
                    _ = (f.root/'stable').write_text('changed')
                elif mutation == 'postimage':
                    _ = f.old.write_text('wrong')
                elif mutation == 'mode':
                    f.old.chmod(0o755)
                elif mutation == 'symlink':
                    f.old.unlink()
                    f.old.symlink_to(f.add)
                elif mutation == 'untracked':
                    _ = (f.root/'extra').write_text('extra')
                elif mutation == 'index':
                    _ = f.run('add', 'tools/v8/observer.py')
                elif mutation == 'head':
                    _ = f.run('commit', '--allow-empty', '-qm', 'Different origin')
                else:
                    f.add.unlink()
                # When authenticated, then the changed boundary is rejected.
                with self.assertRaises((GateContractError, ValueError)):
                    _ = source.authenticate_derived_source(spec)

    def test_evidence_changes_reject_initial_and_recheck(self) -> None:
        for name in ('patch', 'manifest'):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                initial = source.authenticate_derived_source(f.spec())
                path = f.patch if name == 'patch' else f.manifest
                _ = path.write_bytes(path.read_bytes() + b'\n')
                with self.assertRaises(GateContractError):
                    _ = source.recheck_derived_source(initial)

    def test_resealed_invalid_manifest_rejects(self) -> None:
        for field, value in (('base0_sha256', '0'*64), ('observer_sha256','1'*64),
                             ('operation','delete'), ('path','stable')):
            with self.subTest(field=field), tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                f.rows[0] = {'path':'tools/v8/observer.py', 'operation':'modify',
                             'base0_sha256':digest(b'old\n'), 'observer_sha256':digest(b'new\n'), field:value}
                f.write_manifest()
                with self.assertRaises((GateContractError, ValueError)):
                    _ = source.authenticate_derived_source(f.spec())

    def test_unsupported_patch_forms_reject_even_resealed(self) -> None:
        for extra in ('old mode 100644\nnew mode 100755\n', 'GIT binary patch\n', 'rename from old\n',
                      'deleted file mode 100644\n'):
            with self.subTest(extra=extra), tempfile.TemporaryDirectory() as temp:
                f = Fixture(Path(temp).resolve())
                _ = f.patch.write_text(f.patch.read_text() + extra)
                f.write_manifest()
                with self.assertRaises(GateContractError):
                    _ = source.authenticate_derived_source(f.spec())

    def test_constructed_binding_cannot_supply_its_own_digest(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            bound = source.authenticate_derived_source(f.spec())
            with self.assertRaises(GateContractError):
                _ = source.recheck_derived_source(replace(bound, effective_source_sha256='0'*64))

    def test_mutation_during_scan_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            original = derived_snapshot
            calls = 0
            def mutate(root: Path, rows: dict[str, Json], changed: dict[str, tuple[str | None, str]],
                       index: bytes) -> dict[str, Json]:
                nonlocal calls
                result = original(root, rows, changed, index)
                calls += 1
                if calls == 1:
                    _ = f.old.write_text('late mutation')
                return result
            with patch.object(source, 'derived_snapshot', side_effect=mutate), self.assertRaises(GateContractError):
                _ = source.authenticate_derived_source(f.spec())

    def test_origin_symlink_target_drift_rejects(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            link = f.root / 'link'
            link.symlink_to('stable')
            _ = f.run('add', 'link')
            _ = f.run('commit', '-qm', 'Origin symlink fixture')
            f.sha = f.run('rev-parse', 'HEAD').decode().strip()
            f.write_manifest()
            bound = source.authenticate_derived_source(f.spec())
            link.unlink()
            link.symlink_to('tools/v8/observer.py')
            with self.assertRaises(GateContractError):
                _ = source.recheck_derived_source(bound)

    def test_origin_and_external_path_boundaries_reject(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            with self.assertRaises(GateContractError):
                _ = source.authenticate_derived_source(replace(f.spec(), origin_sha='HEAD'))
            alias = Path(temp).resolve() / 'alias'
            alias.symlink_to(f.root, target_is_directory=True)
            with self.assertRaises(GateContractError):
                _ = source.authenticate_derived_source(replace(f.spec(), root=alias))
            spec = f.spec()
            f.patch.unlink()
            with self.assertRaises(GateContractError):
                _ = source.authenticate_derived_source(spec)

    def test_same_path_unrelated_hunk_never_confers_patch_application_proof(self) -> None:
        # Given independently pinned patch and byte manifest with different hunk content.
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            _ = f.patch.write_text(f.patch.read_text().replace('+new\n', '+unrelated\n'))
            f.write_manifest()
            # When only source consistency is authenticated.
            bound = source.authenticate_derived_source(f.spec())
            # Then source bytes are admitted without claiming how they were produced.
            self.assertEqual(f.old.read_text(), 'new\n')
            self.assertEqual(bound.patch_application_provenance, 'NOT_VERIFIED')
            self.assertEqual(bound.build_admission, 'PENDING')
            self.assertFalse(bound.publication_authority)

    def test_overlay_path_scope_is_not_a_semantic_approval(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            f = Fixture(Path(temp).resolve())
            result = source.authenticate_derived_source(f.spec())
            self.assertEqual(result.source_kind, 'DERIVED_OBSERVER')
            self.assertEqual(result.observer_semantics, 'NOT_RUN')


if __name__ == '__main__':
    _ = unittest.main()
