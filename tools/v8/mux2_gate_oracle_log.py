#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/v8/mux2_gate_oracle_log.py TEST LOG
"""Reject incomplete or errored headless oracle logs; this is not GPU acceptance."""
from pathlib import Path
import re
import sys
from typing import Final
from mux2_gate_inputs import EvidenceError, require

# M3D-SWITCH E2: only the oracles of code that survives the 2D renderer remain (camp framing geometry and the
# visibility validator, whose count grew with the 3D receipt checks).
PREFIXES: Final = {'mux2_camp_framing_test': 'CAMP_FRAMING',
                  'mux2_visibility_validator_test': 'MUX2_VISIBILITY_ORACLE'}

EXPECTED_COUNTS: Final = {'mux2_camp_framing_test': 12,
                          'mux2_visibility_validator_test': 27}

def validate_log(name: str, path: Path) -> None:
    require(name in PREFIXES, 'unknown oracle')
    prefix = PREFIXES[name]
    text = path.read_text()
    require('SCRIPT ERROR' not in text and f'{prefix}_FAIL' not in text
            and f'{prefix} FAIL' not in text, 'oracle runtime failure')
    passed = len(re.findall(rf'^{prefix}(?:_| )PASS ', text, re.M))
    require(passed == EXPECTED_COUNTS[name], 'oracle exact expected coverage mismatch')
    completions = re.findall(rf'^{prefix}_DONE\b[^\n]*$', text, re.M)
    require(completions == [f'{prefix}_DONE checks={passed} failures=0'], 'oracle completion count mismatch')
    print(f'MUX2_ORACLE_PASS {name} checks={passed}')


if __name__ == '__main__':
    try:
        require(len(sys.argv) == 3, 'expected TEST LOG')
        validate_log(sys.argv[1], Path(sys.argv[2]))
    except (EvidenceError, OSError) as error:
        print(f'MUX2_ORACLE_FAIL {error}', file=sys.stderr)
        raise SystemExit(1)
