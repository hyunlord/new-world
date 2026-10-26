#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/v8/mux2_gate_fixtures.py EXTERNAL_DIRECTORY ORIGINAL_HASH_MANIFEST
"""Bind genuine format10 migration inputs to their original external hash manifest."""
from pathlib import Path
import sys
from mux2_gate_inputs import EvidenceError, Node, check_hash, external, read_json, require


def validate_fixtures(directory: Path, manifest: Path) -> None:
    root = Path(__file__).resolve().parents[2]
    external(directory, root)
    external(manifest, root)
    directory = directory.resolve()
    files = sorted(p for p in directory.iterdir() if p.suffix in ('.save', '.wsave'))
    require(all(p.is_file() and not p.is_symlink() for p in files),
            'fixtures must be regular files within the fixture directory')
    require(len(files) == 4, 'require four genuine old-binary fixtures')
    document = read_json(manifest)
    values = document.mapping()
    hashes: dict[str, str] = {}
    if 'scenes' in values:
        document.field('schema').equals(1)
        scenes = document.field('scenes').array()
        require(len(scenes) == 4, 'require four original capture scenes')
        ids: set[str] = set()
        for scene in scenes:
            identifier = scene.field('id').text()
            require(bool(identifier) and identifier not in ids, 'duplicate or empty scene id')
            ids.add(identifier)
            path = Path(scene.field('save').text())
            require(path.is_absolute() and path.parent == directory
                    and path == path.resolve(), 'scene save path escapes fixture directory')
            require(path.name not in hashes, 'duplicate scene save path')
            hashes[path.name] = scene.field('sha256').text()
    else:
        hashes = {key: Node(value).text() for key, value in values.items()}
    require(set(hashes) == {p.name for p in files}, 'fixture inventory mismatch')
    for path in files:
        check_hash(path, hashes[path.name])
        with path.open('rb') as stream:
            require(stream.read(12) == b'WORLDSIM' + (10).to_bytes(4, 'little'),
                    'fixture is not format10')
    print('MUX2_V10_FIXTURES_PASS files=4')


if __name__ == '__main__':
    try:
        require(len(sys.argv) == 3, 'expected DIRECTORY MANIFEST')
        validate_fixtures(Path(sys.argv[1]), Path(sys.argv[2]))
    except (EvidenceError, OSError, ValueError) as error:
        print(f'MUX2_V10_FIXTURES_FAIL {error}', file=sys.stderr)
        raise SystemExit(1)
