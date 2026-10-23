# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image", "pytest"]
# ///
# How to run: python -m pytest art/pipeline5/prod2_test_verify_new.py
"""Repeated-evaluation inputs must match terminal generation identity and counts."""
import json
from pathlib import Path

import pytest

from art.pipeline5.attempt_integrity import digest
from art.pipeline5.prod2_verify_new import generation_sources


def test_terminal_generation_binding_rejects_changes_and_wrong_count(tmp_path: Path) -> None:
    look = 'adult_M_hide_wrap_short'
    generation = tmp_path / 'generation' / look / 'try1'
    diagnostic = tmp_path / 'new-production' / look / 'try1'
    generation.mkdir(parents=True)
    diagnostic.mkdir(parents=True)
    raw, prompt = generation / 'raw.png', generation / 'prompt.txt'
    _ = raw.write_bytes(b'preserved source bytes')
    _ = prompt.write_text('frozen prompt')
    record = {'passed': False, 'passing_panels': 8, 'raw_sha256': digest(raw), 'prompt_sha256': digest(prompt)}
    intent = {**record, 'look': look, 'attempt': 1, 'status': 'rejected', 'raw': str(raw),
              'prompt_path': str(prompt), 'diagnostics': str(diagnostic)}
    _ = (generation / 'intent.json').write_text(json.dumps(intent))
    _ = (diagnostic / 'sheet_checks.json').write_text(json.dumps(record))
    assert generation_sources(tmp_path, 1)[0].look == look
    with pytest.raises(ValueError, match='expected 2'):
        _ = generation_sources(tmp_path, 2)
    _ = prompt.write_text('altered prompt')
    with pytest.raises(ValueError, match='mismatch'):
        _ = generation_sources(tmp_path, 1)
    _ = prompt.write_text('frozen prompt')
    _ = (generation / 'intent.json').write_text(json.dumps(intent | {'status': 'reserved'}))
    with pytest.raises(ValueError, match='mismatch'):
        _ = generation_sources(tmp_path, 1)
