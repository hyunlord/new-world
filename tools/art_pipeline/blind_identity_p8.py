# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by measure_p8.py after verify_blind_identity_p7.py writes P8 proof
"""Bind activity observer results to the exact final calibration screenshot bytes."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

from ingame_qa_pixels import Check, result


def final_identity(raw: Path, proof_path: Path) -> Check:
    """Missing proof or a changed final lineup cannot inherit a blind PASS."""
    if not proof_path.is_file():
        return result('A11_FINAL_PIXEL_IDENTITY', False, 'Missing current final calibration identity proof')
    proof = json.loads(proof_path.read_text())
    sources = proof.get('source_sha256', {})
    expected = {'a_lineup_z4.png', 'a_lineup_z2.png'}
    valid = bool(proof.get('passed')) and set(sources) == expected
    errors: list[str] = []
    for name in expected:
        path = raw / name
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != sources.get(name):
            valid = False
            errors.append(name + ': final bytes missing or changed since pixel-identity verification')
    return result('A11_FINAL_PIXEL_IDENTITY', valid, json.dumps({'proof': str(proof_path),
                  'source_sha256': sources, 'errors': errors,
                  'scope': 'Final calibration pixels versus frozen observer grid; no natural recognition inference'}))


def guard_activity(checks: list[Check], identity: Check) -> list[Check]:
    """Preserve activity grading failures and invalidate passes without identity."""
    return [Check('A11', 'FAIL', 'Final activity crops lack verified identity to frozen observer pixels')
            if check.code == 'A11' and identity.status != 'PASS' else check for check in checks] + [identity]
