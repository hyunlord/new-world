"""Synthetic archive controls for the separately authenticated projection lock."""
import hashlib
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
import core_shadow_successor as successor
import core_shadow_freeze as freeze

class ProjectionLockTests(unittest.TestCase):

    def test_exact_projection_excludes_lock_from_measured_digest(self) -> None:
        self.exercise({'rust/a.rs': b'source', 'rust/Cargo.lock': b'lock'})

    def test_missing_lock_rejected(self) -> None:
        self.exercise({'rust/a.rs': b'source'}, valid=False)

    def test_wrong_lock_rejected(self) -> None:
        self.exercise({'rust/a.rs': b'source', 'rust/Cargo.lock': b'wrong'}, valid=False)

    def test_extra_member_rejected(self) -> None:
        self.exercise({'rust/a.rs': b'source', 'rust/Cargo.lock': b'lock', 'rust/extra.rs': b'extra'}, valid=False)

    def test_changed_source_rejected(self) -> None:
        self.exercise({'rust/a.rs': b'changed', 'rust/Cargo.lock': b'lock'}, valid=False)

    def test_duplicate_member_rejected(self) -> None:
        self.exercise({'rust/a.rs': b'source', 'rust/Cargo.lock': b'lock'}, valid=False, duplicate=True)

    def test_symlink_lock_rejected(self) -> None:
        self.exercise({'rust/a.rs': b'source', 'rust/Cargo.lock': b'lock'}, valid=False, symlink=True)

    def exercise(self, files: dict[str, bytes], valid: bool=True, duplicate: bool=False, symlink: bool=False) -> None:
        inventory = {'rust/a.rs': hashlib.sha256(b'source').hexdigest()}
        digest = successor.source_digest(inventory)
        with tempfile.TemporaryDirectory(prefix='SYNTHETIC-lock-projection-') as tmp:
            root = Path(tmp).resolve()
            archive = root / 'archive.tar.gz'
            projection = root / 'projection'
            projection.mkdir()
            with tarfile.open(archive, 'w:gz') as tar:
                for name, raw in files.items():
                    info = tarfile.TarInfo(name)
                    info.size = len(raw)
                    if symlink and name == 'rust/Cargo.lock':
                        info.type = tarfile.SYMTYPE
                        info.linkname = '/outside'
                        info.size = 0
                    tar.addfile(info, io.BytesIO(raw))
                    if duplicate and name == 'rust/a.rs':
                        tar.addfile(info, io.BytesIO(raw))
            args = (archive, projection, inventory, hashlib.sha256(b'lock').hexdigest())
            if valid:
                successor.project_predecessor(*args)
                self.assertEqual((projection / 'rust/Cargo.lock').read_bytes(), b'lock')
                self.assertEqual((projection / 'rust/a.rs').read_bytes(), b'source')
                self.assertEqual(successor.source_digest(inventory), digest)
                self.assertNotIn('rust/Cargo.lock', inventory)
            else:
                with self.assertRaises(freeze.FreezeError):
                    successor.project_predecessor(*args)
if __name__ == '__main__':
    unittest.main(verbosity=2)
