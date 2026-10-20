#!/usr/bin/env python3
"""Compare two chronicle directories by content (tools/v8/identity.sh, check 2).

usage: compare_chronicles.py <base chronicle dir> <candidate chronicle dir>

Every JSON file of the base must exist in the candidate and every key must be
equal except `git_sha`. The `kinds` vocabulary may only grow at its end: the
base list must be a prefix of the candidate's (M1 D21; worldsim-testing lesson
2026-10-18). Prints one PASS line, or exits non-zero naming the first difference.
"""
import json
import sys
from pathlib import Path


def compare(ref: Path, cand: Path) -> str:
    names = sorted(p.name for p in ref.glob('*.json'))
    if not names or names != sorted(p.name for p in cand.glob('*.json')):
        raise ValueError(f'file sets differ ({len(names)} base files)')
    entries = 0
    for name in names:
        a, b = json.loads((ref / name).read_text()), json.loads((cand / name).read_text())
        for key in sorted(set(a) | set(b)):
            if key == 'git_sha':
                continue
            if key == 'kinds' and isinstance(a.get(key), list) and isinstance(b.get(key), list) \
                    and b[key][:len(a[key])] == a[key]:
                continue
            if a.get(key) != b.get(key):
                raise ValueError(f'{name}: {key} differs')
        entries += len(a.get('entries', []))
    return f'{len(names)} files equal by content (every field but git_sha; {entries} entries)'


def main(argv):
    if len(argv) != 2:
        sys.exit(__doc__)
    try:
        print(compare(Path(argv[0]), Path(argv[1])))
    except ValueError as error:
        sys.exit(str(error))


if __name__ == '__main__':
    main(sys.argv[1:])
