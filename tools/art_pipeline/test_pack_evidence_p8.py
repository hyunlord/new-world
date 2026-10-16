"""Reversible evidence packing without touching repository captures."""
import gzip
import hashlib
import io
import json
import tarfile
import tempfile
import unittest
from dataclasses import asdict, replace
from pathlib import Path

from pack_evidence_p8 import PackingError, pack, parse_ledger, restore


class PackEvidenceTests(unittest.TestCase):
    def test_roundtrip_and_noncanonical(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / 'production8'
            root.mkdir()
            expected = {'a.entities.json.gz': gzip.compress(b'{"a":1}', mtime=0), 'nested/b.entities.json.gz': gzip.compress(b'{"b":2}', mtime=123)}
            for name, data in expected.items():
                path = root / name
                path.parent.mkdir(exist_ok=True)
                _ = path.write_bytes(data)
            ledger = pack(root, base / 'backup')
            self.assertFalse(list(root.rglob('*.entities.json.gz')))
            output = base / 'restore'
            _ = restore(ledger, output)
            for name, data in expected.items():
                self.assertEqual((output / name).read_bytes(), data)
                self.assertEqual((base / 'backup' / name).read_bytes(), data)
                self.assertEqual((base / 'backup' / name).stat().st_mode & 0o222, 0)
            self.assertEqual({r.storage for r in parse_ledger(ledger).entries}, {'json', 'gzip'})
            _ = restore(ledger, output)  # Exact existing files are safe and untouched.
            _ = (output / 'a.entities.json.gz').write_bytes(b'conflict')
            with self.assertRaisesRegex(PackingError, 'overwrite'):
                _ = restore(ledger, output)
            self.assertEqual((output / 'a.entities.json.gz').read_bytes(), b'conflict')

    def test_path_traversal_and_tamper(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / 'production8'
            root.mkdir()
            _ = (root / 'a.entities.json.gz').write_bytes(gzip.compress(b'{}', mtime=0))
            ledger = pack(root, base / 'backup')
            original = ledger.read_text()
            data = parse_ledger(ledger)
            changed = replace(data, entries=(replace(data.entries[0], path='../escaped.entities.json.gz'),))
            _ = ledger.write_text(json.dumps(asdict(changed)))
            with self.assertRaises(PackingError):
                _ = restore(ledger, base / 'restore')
            self.assertFalse((base / 'escaped.entities.json.gz').exists())
            _ = ledger.write_text(original)
            archive = root / data.archive
            _ = archive.write_bytes(archive.read_bytes() + b'changed')
            with self.assertRaisesRegex(PackingError, 'archive SHA'):
                _ = restore(ledger, base / 'restore')

    def test_tar_symlink_rejected_even_with_matching_archive_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / 'production8'
            root.mkdir()
            _ = (root / 'a.entities.json.gz').write_bytes(gzip.compress(b'{}', mtime=0))
            ledger = pack(root, base / 'backup')
            data = parse_ledger(ledger)
            archive = root / data.archive
            with tarfile.open(archive, 'w:xz') as tar:
                member = tarfile.TarInfo(data.entries[0].member)
                member.type = tarfile.SYMTYPE
                member.linkname = '/tmp/escape'
                tar.addfile(member, io.BytesIO())
            data = replace(data, archive_sha256=hashlib.sha256(archive.read_bytes()).hexdigest())
            _ = ledger.write_text(json.dumps(asdict(data)))
            with self.assertRaisesRegex(PackingError, 'regular'):
                _ = restore(ledger, base / 'restore')

    def test_symlink_destination_and_wrong_scope(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / 'production8'
            root.mkdir()
            (root / 'nested').mkdir()
            _ = (root / 'nested/a.entities.json.gz').write_bytes(gzip.compress(b'{}', mtime=0))
            ledger = pack(root, base / 'backup')
            out = base / 'restore'
            out.mkdir()
            (out / 'nested').symlink_to(base, target_is_directory=True)
            with self.assertRaisesRegex(PackingError, 'symlink'):
                _ = restore(ledger, out)
            with self.assertRaisesRegex(PackingError, 'production8'):
                _ = pack(base, base / 'other')


if __name__ == '__main__':
    _ = unittest.main()
