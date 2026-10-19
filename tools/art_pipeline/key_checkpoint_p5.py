# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/key_checkpoint_p5.py CANDIDATES_JSON REVIEWS_JSON
"""Bind the first key sheet checkpoint to sixteen physical cells and reviews."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sys

from key_candidates_p5 import KEY_ERRORS
from key_plan_p5 import OUTPUT


def checkpoint(ledger_path: Path, review_path: Path) -> Path:
    """Stop further key calls unless both literal section 8.6 thresholds pass."""
    ledger = json.loads(ledger_path.read_text())
    records = [r for r in ledger['candidates'] if r['source_kind'] == 'generated']
    if len(records) != 16 or not ledger['key_hair']:
        message = 'Checkpoint requires sixteen physical key-sheet candidates'
        raise ValueError(message)
    verdicts = json.loads(review_path.read_text())['verdicts']
    by_id = {r['candidate_id']: r for r in verdicts}
    if len(by_id) != 16 or set(by_id) != {r['id'] for r in records}:
        message = 'Checkpoint requires exact sixteen-candidate review coverage'
        raise ValueError(message)
    clean = [r['id'] for r in records if not KEY_ERRORS.intersection(r['guards']['errors']) and r['conversion_error'] is None]
    approved = [r['id'] for r in records if by_id[r['id']]['verdict'] == 'ok']
    path = OUTPUT / 'keys/checkpoint_first_key.json'
    if path.exists():
        message = 'Refusing to overwrite first-key checkpoint'
        raise ValueError(message)
    path.write_text(json.dumps({'schema_version': 1, 'attempted_cells': 16,
        'ledger_path': str(ledger_path), 'ledger_sha256': hashlib.sha256(ledger_path.read_bytes()).hexdigest(),
        'review_path': str(review_path), 'review_sha256': hashlib.sha256(review_path.read_bytes()).hexdigest(),
        'key_error_free_ids': clean, 'key_error_free_count': len(clean), 'approved_ids': approved,
        'approved_count': len(approved), 'minimum_key_error_free': 12, 'minimum_approved': 8,
        'stop_key_generation': len(clean) < 12 or len(approved) < 8,
        'child_sheet_still_allowed': True}, indent=2) + '\n')
    return path


if __name__ == '__main__':
    print(checkpoint(Path(sys.argv[1]), Path(sys.argv[2])))
