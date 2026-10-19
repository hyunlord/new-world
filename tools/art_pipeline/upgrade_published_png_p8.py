"""Replace larger P8 published JPEGs with byte-exact existing 1920 PNGs.

Original JPEGs and manifest bytes are preserved in a new external backup directory.
No resampling or encoding occurs. Run again with a different label for merged GOD.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import TypeAlias

from pack_evidence_p8 import parse_ledger, payloads, safe_path
from PIL import Image

Json: TypeAlias = None | bool | int | float | str | list['Json'] | dict[str, 'Json']
decode_json: Callable[[str], Json] = json.loads


@dataclass(frozen=True, slots=True)
class Upgrade:
    old: Path
    source: Path
    target: Path
    old_sha: str
    new_sha: str
    width: int
    height: int


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def upgrade(root: Path, folders: list[str], backup: Path, label: str) -> Path:
    if root.name != 'production8' or backup.exists() or backup.resolve().is_relative_to(root.resolve()):
        raise ValueError('require production8 scope and new external backup')
    if Path(label).name != label or label in ('', '.', '..'):
        raise ValueError('unsafe ledger label')
    ledger = root / f'png-upgrades-{label}.json'
    if ledger.exists():
        raise ValueError('refuse existing ledger')
    archived_entities: set[str] = set()
    archive_ledger = root / 'entities.archive.json'
    if archive_ledger.exists():
        archived_entities = {entry.path for entry, _ in payloads(parse_ledger(archive_ledger), root)}
    manifests: dict[Path, dict[str, Json]] = {}
    upgrades: dict[Path, Upgrade] = {}
    for folder in folders:
        directory = safe_path(root, folder)
        if directory.is_symlink():
            raise ValueError('symlink directory refused')
        for path in sorted(directory.glob('manifest*.json')):
            data = decode_json(path.read_text())
            if not isinstance(data, dict) or not isinstance(data.get('captures'), list):
                raise TypeError(f'invalid manifest {path}')
            captures = data['captures']
            if not isinstance(captures, list):
                raise TypeError('invalid captures')
            for row in captures:
                if not isinstance(row, dict):
                    raise TypeError('invalid capture')
                entity_name = row.get('entities')
                if not isinstance(entity_name, str):
                    raise TypeError('missing entity reference')
                entity = directory / entity_name
                archived = entity.is_relative_to(root) and str(entity.relative_to(root)) in archived_entities
                if not entity.is_file() and not archived:
                    raise ValueError('missing entity reference or verified archive member')
                published_name, standard_name = row.get('published_file'), row.get('standard_file')
                if not isinstance(published_name, str):
                    raise TypeError('missing published path')
                published = directory / published_name
                if not published.is_file():
                    raise ValueError(f'missing published reference {published}')
                if published.suffix.lower() not in ('.jpg', '.jpeg'):
                    continue
                if Path(published_name).name != published_name or not isinstance(standard_name, str):
                    raise ValueError('unsafe JPEG path or missing standard source')
                source = Path(standard_name)
                if not source.name.endswith('.1920.png') or not source.is_file():
                    raise ValueError('missing standard1920 PNG')
                if source.stat().st_size >= published.stat().st_size:
                    continue
                target = published.with_suffix('.png')
                if target.exists() or published.is_symlink() or source.is_symlink():
                    raise ValueError(f'collision or symlink: {target}')
                with Image.open(source) as image:
                    width, height = image.size
                    if image.format != 'PNG' or (width, height) != (1920, 1080):
                        raise ValueError('unexpected standard dimensions/format')
                candidate = Upgrade(published, source, target, sha(published), sha(source), width, height)
                if published in upgrades and upgrades[published] != candidate:
                    raise ValueError('conflicting duplicate manifest source')
                upgrades[published] = candidate
                row['published_file'] = target.name
                row['published_width'], row['published_height'] = width, height
                row['published_sha256'] = candidate.new_sha
            manifests[path] = data
    targets = [u.target for u in upgrades.values()]
    if len(set(targets)) != len(targets):
        raise ValueError('duplicate upgrade target')
    for path in (*manifests.keys(), *upgrades.keys()):
        target = backup / path.relative_to(root)
        target.parent.mkdir(parents=True, exist_ok=True)
        with path.open('rb') as src, target.open('xb') as dst:
            shutil.copyfileobj(src, dst)
        if sha(path) != sha(target):
            raise ValueError('backup hash mismatch')
        target.chmod(0o444)
    for item in upgrades.values():
        with item.source.open('rb') as src, item.target.open('xb') as dst:
            shutil.copyfileobj(src, dst)
        if sha(item.target) != item.new_sha:
            raise ValueError('installed PNG byte hash mismatch')
        with Image.open(item.source) as original, Image.open(item.target) as installed:
            if original.convert('RGBA').tobytes() != installed.convert('RGBA').tobytes():
                raise ValueError('installed PNG pixel mismatch')
    records = [{'old_path': str(u.old.relative_to(root)), 'new_path': str(u.target.relative_to(root)), 'source': str(u.source), 'source_sha256': u.new_sha, 'old_sha256': u.old_sha, 'new_sha256': sha(u.target), 'old_bytes': u.old.stat().st_size, 'new_bytes': u.target.stat().st_size, 'pixel_and_byte_identity': True, 'resize': False} for u in upgrades.values()]
    manifest_records: list[dict[str, str]] = []
    for path, data in manifests.items():
        old = backup / path.relative_to(root)
        manifest_records.append({'path': str(path.relative_to(root)), 'original_sha256': sha(old)})
        _ = path.write_text(json.dumps(data, indent=2) + '\n')
        manifest_records[-1]['updated_sha256'] = sha(path)
    _ = ledger.write_text(json.dumps({'backup': str(backup.resolve()), 'upgrades': records, 'manifests': manifest_records}, indent=2) + '\n')
    for item in upgrades.values():
        if sha(item.old) != item.old_sha or sha(item.target) != item.new_sha:
            raise ValueError('source or target changed before JPEG removal')
        item.old.unlink()
    return ledger


class Options(argparse.Namespace):
    root: Path = Path('docs/art/production8')
    backup: Path = Path()
    label: str = ''

    def __init__(self) -> None:
        super().__init__()
        self.folders: list[str] = []


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument('folders', nargs='+')
    _ = parser.add_argument('--root', type=Path, default=Path('docs/art/production8'))
    _ = parser.add_argument('--label', required=True)
    _ = parser.add_argument('--backup', type=Path, required=True)
    args = parser.parse_args(namespace=Options())
    print(upgrade(args.root, args.folders, args.backup, args.label))


if __name__ == '__main__':
    main()
