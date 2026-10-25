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

PREFIXES: Final = {'mux2_visibility_validator_test': 'MUX2_VISIBILITY_ORACLE',
                  'mux2_marker_layout_test': 'MUX2_MARKER_LAYOUT',
                  'mux2_coincident_test': 'MUX2_COINCIDENT',
                  'mux2_badge_index_test': 'MUX2_BADGE_INDEX',
                  'mux2_badge_packing_test': 'MUX2_BADGE_PACKING',
                  'mux2_overlap_test': 'MUX2_OVERLAP',
                  'mux2_badge_grid_test': 'MUX2_BADGE_GRID',
                  'mux2_badge_aggregate_test': 'MUX2_BADGE_AGGREGATE',
                  'mux2_badge_local_test': 'MUX2_BADGE_LOCAL',
                  'mux2_bake_receipt_test': 'MUX2_BAKE_RECEIPT',
                  'mux2_badge_extent_test': 'MUX2_BADGE_EXTENT',
                  'mux2_person_cache_test': 'MUX2_PERSON_CACHE',
                  'mux2_extent_owner_test': 'MUX2_EXTENT_OWNER',
                  'mux2_wall_cache_test': 'MUX2_WALL_CACHE'}

EXPECTED_COUNTS: Final = {'mux2_visibility_validator_test': 20,
                          'mux2_marker_layout_test': 21, 'mux2_coincident_test': 30,
                          'mux2_badge_index_test': 116, 'mux2_badge_packing_test': 114,
                          'mux2_overlap_test': 81, 'mux2_badge_grid_test': 514,
                          'mux2_badge_aggregate_test': 97, 'mux2_badge_local_test': 205,
                          'mux2_bake_receipt_test': 20, 'mux2_badge_extent_test': 75,
                          'mux2_person_cache_test': 182, 'mux2_extent_owner_test': 99}

def validate_log(name: str, path: Path) -> None:
    require(name in PREFIXES, 'unknown oracle')
    prefix = PREFIXES[name]
    text = path.read_text()
    require('SCRIPT ERROR' not in text and f'{prefix}_FAIL' not in text
            and f'{prefix} FAIL' not in text, 'oracle runtime failure')
    passed = len(re.findall(rf'^{prefix}(?:_| )PASS ', text, re.M))
    if name == 'mux2_wall_cache_test':
        labels = re.findall(r'^MUX2_WALL_CACHE_HASHES (\w+) ', text, re.M)
        expected = {'base', 'cache_hit', 'material_alias', 'changed_tint', 'changed_top',
                    'reloaded_images', 'after_invalidation'}
        require(len(labels) == 7 and set(labels) == expected, 'wall oracle coverage mismatch')
        passed = len(labels)
        require(re.findall(rf'^{prefix}_DONE failures=\[\]$', text, re.M)
                == [f'{prefix}_DONE failures=[]'], 'wall oracle incomplete')
    else:
        require(passed == EXPECTED_COUNTS[name], 'oracle exact expected coverage mismatch')
        suffix = 'RESULT' if name == 'mux2_badge_extent_test' else 'DONE'
        completions = re.findall(rf'^{prefix}_(?:DONE|RESULT)\b[^\n]*$', text, re.M)
        require(completions == [f'{prefix}_{suffix} checks={passed} failures=0'],
                'oracle completion count mismatch')
    print(f'MUX2_ORACLE_PASS {name} checks={passed}')


if __name__ == '__main__':
    try:
        require(len(sys.argv) == 3, 'expected TEST LOG')
        validate_log(sys.argv[1], Path(sys.argv[2]))
    except (EvidenceError, OSError) as error:
        print(f'MUX2_ORACLE_FAIL {error}', file=sys.stderr)
        raise SystemExit(1)
