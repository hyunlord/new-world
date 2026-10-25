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
    files = sorted(p for p in directory.iterdir() if p.suffix in ('.save', '.wsave'))
    require(len(files) == 4, 'require four genuine old-binary fixtures')
    hashes = read_json(manifest).mapping()
    require(set(hashes) == {p.name for p in files}, 'fixture inventory mismatch')
    for path in files:
        check_hash(path, Node(hashes[path.name]).text())
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
