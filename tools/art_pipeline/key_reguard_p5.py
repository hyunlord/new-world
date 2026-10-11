# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/key_reguard_p5.py ORIGINAL_CANDIDATES_JSON
"""Append a guard-only correction for successfully converted first-key evidence."""
from __future__ import annotations

from collections import defaultdict
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

from PIL import Image

from salvage_v4 import CandidateRecord, guard_data, guarded_candidate
from salvage_verdicts import candidate_fingerprint
from v4_contracts import guard_frame, guard_group


def correct(source: Path) -> Path:
    """Preserve every source image and prior ledger while dropping stale errors."""
    ledger = json.loads(source.read_text())
    before = deepcopy(ledger['candidates'])
    records = ledger['candidates']
    corrections = []
    image_hashes = {path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
                    for row in records for path in row['images'].values()}
    for row in records:
        if row['conversion_error'] is not None or 'conversion_failure' not in row['guards']['errors']:
            continue
        for kind in ('native', 'sprite', 'hair_mask', 'sprite_hair_mask', 'key_quantised'):
            with Image.open(row['images'][kind]) as image:
                image.verify()
        old = deepcopy(row['guards'])
        row['guards']['errors'] = [e for e in row['guards']['errors'] if e != 'conversion_failure']
        row['guards'] = guard_data(guard_frame(guarded_candidate(row)))
        row['guards']['warnings'] = list(dict.fromkeys(row['guards']['warnings'] + row['metrics']['warnings']))
        corrections.append({'id': row['id'], 'old_guards': old, 'new_guards': row['guards']})
    groups: defaultdict[tuple[str, str], list[CandidateRecord]] = defaultdict(list)
    for row in records:
        groups[row['activity'], row['view']].append(row)
    for group in groups.values():
        report = guard_data(guard_group([guarded_candidate(r) for r in group if r['guards']['passed']]))
        for row in group:
            row['group_guards'] = report
    for path, digest in image_hashes.items():
        if hashlib.sha256(Path(path).read_bytes()).hexdigest() != digest:
            message = f'Image changed during guard correction: {path}'
            raise ValueError(message)
    ledger['guard_correction'] = {'reason': 'Clear stale first-pass conversion_failure after successful key re-fit; no image or verdict changes.',
        'source_path': str(source), 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
        'source_candidate_fingerprints': {r['id']: candidate_fingerprint(r) for r in before},
        'image_sha256_unchanged': image_hashes, 'corrections': corrections}
    ledger['candidate_fingerprints'] = {r['id']: candidate_fingerprint(r) for r in records}
    output = source.with_name('candidates_reguarded.json')
    if output.exists():
        message = 'Refusing to overwrite a guard correction'
        raise ValueError(message)
    output.write_text(json.dumps(ledger, indent=2) + '\n')
    index = {'candidates_sha256': hashlib.sha256(output.read_bytes()).hexdigest(),
             'candidate_fingerprints': ledger['candidate_fingerprints'],
             'images_sha256': {b['path']: b['sha256'] for b in ledger['boards']},
             'source_sha256': {str(source): hashlib.sha256(source.read_bytes()).hexdigest()},
             'guard_correction_source': str(source)}
    output.with_name('review_index_reguarded.json').write_text(json.dumps(index, indent=2) + '\n')
    return output


if __name__ == '__main__':
    print(correct(Path(sys.argv[1])))
