"""Idempotently add terrain PNG records without altering other art provenance."""
from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from PIL import Image


@dataclass(frozen=True, slots=True)
class Coverage:
    added: tuple[str, ...]
    terrain_missing: tuple[str, ...]
    nonterrain_missing: tuple[str, ...]


def merge(root: Path) -> Coverage:
    """Run after producers finish; preserve each existing manifest asset record."""
    manifest_path = root / 'assets/art/manifest.json'
    manifest = json.loads(manifest_path.read_text())
    provenance = json.loads((root / 'assets/art/terrain/provenance.json').read_text())
    records = manifest['assets']
    listed = {record['path'] for record in records}
    added: list[str] = []
    for record in provenance:
        path = str(record['path'])
        if not path.startswith('assets/art/terrain/') or not path.endswith('.png'):
            raise RuntimeError(f'Unexpected terrain provenance path: {path}')
        with Image.open(root / path) as image:
            if list(image.size) != record['native_size']:
                raise RuntimeError(f'Terrain native size disagrees with PNG: {path}')
        if path not in listed:
            records.append(record)
            listed.add(path)
            added.append(path)
    if added:
        pending = manifest_path.with_suffix('.terrain-merge.tmp')
        pending.write_text(json.dumps(manifest, indent=2) + '\n')
        pending.replace(manifest_path)
    images = {str(path.relative_to(root)) for path in (root / 'assets/art').rglob('*.png')}
    missing = images - listed
    return Coverage(tuple(added), tuple(sorted(path for path in missing if '/terrain/' in path)),
                    tuple(sorted(path for path in missing if '/terrain/' not in path)))


if __name__ == '__main__':
    result = merge(Path(__file__).resolve().parents[2])
    print(json.dumps({'added': result.added, 'terrain_missing': result.terrain_missing,
                      'nonterrain_missing': result.nonterrain_missing}, indent=2))
    raise SystemExit(1 if result.terrain_missing else 0)
