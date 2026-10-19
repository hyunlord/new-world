"""Write H9 geometry proof from exactly three untouched selected-person sidecars."""
from __future__ import annotations
import hashlib
import json
import sys
from pathlib import Path
from prepare_blind_crops_p13 import crowd_proof


def main(args: list[str]) -> int:
    output, *sources = [Path(value) for value in args]
    if len(sources) != 3 or len(set(path.resolve() for path in sources)) != 3:
        raise ValueError('requires three distinct crowd captures')
    rows = []
    for source in sources:
        image = source.with_name('capture.png')
        rows.append({'source': str(source.resolve()), 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
                     'image_sha256': hashlib.sha256(image.read_bytes()).hexdigest(),
                     **crowd_proof(json.loads(source.read_text()))})
    passed = all(row['zero_coverage_crowd_proven'] for row in rows)
    with output.open('x') as stream:
        json.dump({'captures': rows, 'geometry_pass': passed, 'reader_score': None,
                   'acceptance': 'AWAITING_READER' if passed else 'FAIL'}, stream, indent=2)
        stream.write('\n')
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
