"""Losslessly pack production8 entity sidecars; restore defaults outside the repo."""
from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import tarfile
from collections.abc import Callable
from dataclasses import asdict, dataclass
from pathlib import Path, PurePosixPath
from typing import Literal, TypeAlias, assert_never

Json: TypeAlias = None | bool | int | float | str | list["Json"] | dict[str, "Json"]
decode_json: Callable[[str], Json] = json.loads


class PackingError(RuntimeError):
    """Evidence validation failed; no unverified sidecar may be removed."""


@dataclass(frozen=True, slots=True)
class Entry:
    path: str
    member: str
    storage: Literal['json', 'gzip']
    gzip_sha256: str
    json_sha256: str
    gzip_bytes: int
    json_bytes: int


@dataclass(frozen=True, slots=True)
class Ledger:
    archive: str
    archive_sha256: str
    entries: tuple[Entry, ...]
    original_root: str
    immutable_backup: str
    schema_version: int = 1


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def safe_path(root: Path, name: str) -> Path:
    relative = PurePosixPath(name)
    if not name or relative.is_absolute() or '..' in relative.parts or '\\' in name:
        raise PackingError(f'unsafe relative path: {name}')
    target = root / relative
    components = [root]
    for part in relative.parts:
        components.append(components[-1] / part)
    for component in components:
        if component.is_symlink():
            raise PackingError(f'symlink path refused: {component}')
    if not target.resolve().is_relative_to(root.resolve()):
        raise PackingError(f'path escapes destination: {name}')
    return target


def parse_ledger(path: Path) -> Ledger:
    data = decode_json(path.read_text())
    if not isinstance(data, dict) or not isinstance(data.get('entries'), list):
        raise PackingError('invalid ledger object or entries')

    def text(row: dict[str, Json], key: str) -> str:
        value = row.get(key)
        if not isinstance(value, str):
            raise PackingError(f'expected text: {key}')
        return value

    def number(row: dict[str, Json], key: str) -> int:
        value = row.get(key)
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            raise PackingError(f'expected nonnegative integer: {key}')
        return value

    entries: list[Entry] = []
    raw_entries = data['entries']
    if not isinstance(raw_entries, list):
        raise PackingError('invalid entries')
    for row in raw_entries:
        if not isinstance(row, dict):
            raise PackingError('invalid entry')
        mode = text(row, 'storage')
        if mode not in ('json', 'gzip'):
            raise PackingError('unknown archive storage mode')
        entry = Entry(text(row, 'path'), text(row, 'member'), mode, text(row, 'gzip_sha256'), text(row, 'json_sha256'), number(row, 'gzip_bytes'), number(row, 'json_bytes'))
        if not entry.path.endswith('.entities.json.gz'):
            raise PackingError('ledger path is not an entity sidecar')
        for digest in (entry.gzip_sha256, entry.json_sha256):
            if len(digest) != 64 or any(c not in '0123456789abcdef' for c in digest):
                raise PackingError('invalid SHA256')
        entries.append(entry)
    ledger = Ledger(text(data, 'archive'), text(data, 'archive_sha256'), tuple(entries), text(data, 'original_root'), text(data, 'immutable_backup'), number(data, 'schema_version'))
    if ledger.schema_version != 1 or not entries:
        raise PackingError('unsupported or empty ledger')
    if len({e.path for e in entries}) != len(entries) or len({e.member for e in entries}) != len(entries):
        raise PackingError('duplicate ledger paths or members')
    return ledger


def payloads(ledger: Ledger, archive_root: Path) -> tuple[tuple[Entry, bytes], ...]:
    archive = safe_path(archive_root, ledger.archive)
    if sha(archive.read_bytes()) != ledger.archive_sha256:
        raise PackingError('archive SHA mismatch')
    decoded: list[tuple[Entry, bytes]] = []
    with tarfile.open(archive, 'r:xz') as tar:
        members = tar.getmembers()
        if len(members) != len(ledger.entries) or {m.name for m in members} != {e.member for e in ledger.entries}:
            raise PackingError('archive member set differs from ledger')
        for member in members:
            _ = safe_path(archive_root, member.name)
            if not member.isfile():
                raise PackingError('archive permits regular files only')
        for entry in ledger.entries:
            stream = tar.extractfile(entry.member)
            if stream is None:
                raise PackingError(f'missing archive member: {entry.member}')
            with stream:
                content = stream.read()
            match entry.storage:
                case 'json':
                    raw, compressed = content, gzip.compress(content, mtime=0)
                case 'gzip':
                    compressed, raw = content, gzip.decompress(content)
                case _:
                    assert_never(entry.storage)
            if sha(raw) != entry.json_sha256 or sha(compressed) != entry.gzip_sha256:
                raise PackingError(f'roundtrip SHA mismatch: {entry.path}')
            if len(raw) != entry.json_bytes or len(compressed) != entry.gzip_bytes:
                raise PackingError(f'roundtrip size mismatch: {entry.path}')
            decoded.append((entry, compressed))
    return tuple(decoded)


def restore(ledger_path: Path, destination: Path) -> int:
    ledger = parse_ledger(ledger_path)
    contents = payloads(ledger, ledger_path.parent)
    # Validate every destination before writing any file; matching files are untouched.
    for entry, compressed in contents:
        path = safe_path(destination, entry.path)
        if path.exists() and (not path.is_file() or path.read_bytes() != compressed):
            raise PackingError(f'refuse overwrite mismatch: {path}')
    for entry, compressed in contents:
        path = safe_path(destination, entry.path)
        if not path.exists():
            path.parent.mkdir(parents=True, exist_ok=True)
            with path.open('xb') as stream:
                _ = stream.write(compressed)
    return len(contents)


def pack(root: Path, backup: Path) -> Path:
    if root.name != 'production8' or root.is_symlink():
        raise PackingError('pack scope must be a real production8 directory')
    if backup.resolve().is_relative_to(root.resolve()):
        raise PackingError('immutable backup must be outside production8')
    archive = root / 'entities.tar.xz'
    ledger_path = root / 'entities.archive.json'
    if archive.exists() or ledger_path.exists() or backup.exists():
        raise PackingError('archive, ledger and backup must be new; refuse overwrite')
    entries: list[Entry] = []
    originals: list[tuple[Path, bytes]] = []
    buffer = io.BytesIO()
    paths = sorted(root.rglob('*.entities.json.gz'))
    if not paths:
        raise PackingError('no entity sidecars found')
    with tarfile.open(fileobj=buffer, mode='w:xz', preset=6) as tar:
        for path in paths:
            name = str(path.relative_to(root))
            _ = safe_path(root, name)
            compressed = path.read_bytes()
            raw = gzip.decompress(compressed)
            canonical = gzip.compress(raw, mtime=0) == compressed
            storage: Literal['json', 'gzip'] = 'json' if canonical else 'gzip'
            member_name = name.removesuffix('.gz') if canonical else name
            content = raw if canonical else compressed
            member = tarfile.TarInfo(member_name)
            member.size, member.mode = len(content), 0o644
            tar.addfile(member, io.BytesIO(content))
            entries.append(Entry(name, member_name, storage, sha(compressed), sha(raw), len(compressed), len(raw)))
            originals.append((path, compressed))
    archive_bytes = buffer.getvalue()
    ledger = Ledger(archive.name, sha(archive_bytes), tuple(entries), str(root.resolve()), str(backup.resolve()))
    # Publish recovery material first. Failure leaves originals intact.
    for entry, (_, compressed) in zip(entries, originals, strict=True):
        target = safe_path(backup, entry.path)
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open('xb') as stream:
            _ = stream.write(compressed)
        target.chmod(0o444)
        if sha(target.read_bytes()) != entry.gzip_sha256:
            raise PackingError('backup SHA mismatch')
    with archive.open('xb') as stream:
        _ = stream.write(archive_bytes)
    with ledger_path.open('x') as stream:
        json.dump(asdict(ledger), stream, indent=2)
        _ = stream.write('\n')
    restored = payloads(parse_ledger(ledger_path), root)
    for (entry, compressed), (path, original) in zip(restored, originals, strict=True):
        if compressed != original or path.read_bytes() != original:
            raise PackingError(f'source changed or exact reconstruction failed: {entry.path}')
    # All originals have verified archive and immutable external copies at this point.
    for path, original in originals:
        if path.read_bytes() != original:
            raise PackingError(f'source changed before removal: {path}')
        path.unlink()
    return ledger_path


def restore_diagnostics(root: Path, destination: Path) -> int:
    data = decode_json((root / 'diagnostic-gifs-and-logs.archive.json').read_text())
    if not isinstance(data, dict):
        raise PackingError('invalid diagnostic ledger')
    archive_name, digest, entries = data.get('archive'), data.get('archive_sha256'), data.get('entries')
    if not isinstance(archive_name, str) or not isinstance(digest, str) or not isinstance(entries, list):
        raise PackingError('invalid diagnostic archive fields')
    archive = safe_path(root, archive_name)
    if sha(archive.read_bytes()) != digest:
        raise PackingError('diagnostic archive SHA mismatch')
    restored: list[tuple[Path, bytes]] = []
    with tarfile.open(archive, 'r:xz') as tar:
        for row in entries:
            if not isinstance(row, dict):
                raise PackingError('invalid diagnostic entry')
            name, member_name, expected = row.get('path'), row.get('member'), row.get('sha256')
            if not isinstance(name, str) or not isinstance(member_name, str) or not isinstance(expected, str):
                raise PackingError('invalid diagnostic path/hash')
            target = safe_path(destination, name)
            _ = safe_path(destination, member_name)
            if not tar.getmember(member_name).isfile():
                raise PackingError('diagnostic member must be regular file')
            stream = tar.extractfile(member_name)
            if stream is None:
                raise PackingError('missing diagnostic member')
            with stream:
                content = stream.read()
            if sha(content) != expected or (target.exists() and target.read_bytes() != content):
                raise PackingError('diagnostic SHA/overwrite mismatch')
            restored.append((target, content))
    for target, content in restored:
        if not target.exists():
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open('xb') as stream:
                _ = stream.write(content)
    return len(restored)


class Options(argparse.Namespace):
    command: str = ""
    root: Path = Path("docs/art/production8")
    backup: Path = Path("/Users/rexxa/work/art-production8-evidence/entity-gzip-originals")
    destination: Path = Path("/Users/rexxa/work/art-production8-evidence/entity-restore-verify")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument('command', choices=['pack', 'restore', 'restore-diagnostics'])
    _ = parser.add_argument('--root', type=Path, default=Path('docs/art/production8'))
    _ = parser.add_argument('--backup', type=Path, default=Path('/Users/rexxa/work/art-production8-evidence/entity-gzip-originals'))
    _ = parser.add_argument('--destination', type=Path, default=Path('/Users/rexxa/work/art-production8-evidence/entity-restore-verify'))
    args = parser.parse_args(namespace=Options())
    if args.command == 'pack':
        print(pack(args.root, args.backup))
    elif args.command == 'restore-diagnostics':
        print(f'Restored {restore_diagnostics(args.root, args.destination)} exact diagnostic files into {args.destination}')
    else:
        print(f'Restored {restore(args.root / "entities.archive.json", args.destination)} exact gzip sidecars into {args.destination}')


if __name__ == '__main__':
    main()
