# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/blind_p6.py RESULT1 RESULT2 OUTPUT
"""Combine independently frozen observer results without averaging failures away."""
from __future__ import annotations
import json
from pathlib import Path
import sys
from ingame_qa_pixels import Check, result


def check_blind(path: Path | None) -> Check:
    """Each observer must independently meet all three unchanged/new thresholds."""
    if path is None or not path.exists():
        return Check('A11', 'N/A', 'Two frozen observer labels and confusion matrices required')
    document = json.loads(path.read_text())
    observers = document.get('observers', [])
    errors = []
    if len(observers) != 2:
        errors.append('Exactly two observers required')
    identities = [row.get('observer_id', '') for row in observers]
    if len(set(identities)) != 2 or not all(identities):
        errors.append('Two distinct observer identities required; matching answers are allowed')
    for index, row in enumerate(observers):
        carry = row.get('confusion', {}).get('2x', {}).get('carry', {})
        eat = row.get('confusion', {}).get('2x', {}).get('eat', {})
        valid = (row.get('count') == 24 and row.get('accuracy_4x', 0) >= .85
                 and row.get('accuracy_2x', 0) >= .70 and carry.get('carry', 0) >= 2
                 and sum(carry.values()) == 3 and sum(eat.values()) >= 3
                 and row.get('observer_labels_sha256') and not row.get('absent_activities'))
        if not valid:
            errors.append(f'observer {index + 1} fails one or more thresholds/provenance constraints')
    document['errors'] = errors
    return result('A11', not errors, json.dumps(document, ensure_ascii=False))


if __name__ == '__main__':
    first, second, output = map(Path, sys.argv[1:])
    output.write_text(json.dumps({'observers': [json.loads(first.read_text()), json.loads(second.read_text())]}, indent=2) + '\n')
    print(check_blind(output))
