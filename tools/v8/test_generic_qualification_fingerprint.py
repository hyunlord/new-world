# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -B -m unittest test_generic_qualification_fingerprint
"""Source-shaped scalar fixtures, not native qualification evidence."""
import json
import unittest
from copy import deepcopy
from typing import final

import test_generic_whole_gate_qualification_compare as fixtures
from generic_gate import GateContractError
from generic_m6a_observations import FIXED_FIELDS
from generic_whole_gate_qualification_compare import compare
from mux2_gate_inputs import EvidenceError, Json


@final
class FingerprintTests(unittest.TestCase):
    def samples(self) -> tuple[tuple[str, dict[str, Json], str], ...]:
        perf = fixtures.ComparatorTests().perf()
        perf['fingerprint'] = '0123456789abcdef'
        m6a: dict[str, Json] = dict.fromkeys(FIXED_FIELDS)
        m6a.update(schema_version=1, kind='m6a_food_observation_v1', days=72,
                    ticks_per_day=1440, end_tick=103680, observed_ticks=103680,
                    invariant_checks=207360, fingerprint='0123456789abcdef-fedcba9876543210')
        return (('perf', perf, '1123456789abcdef'),
                ('m6a', m6a, '0123456789abcdef-fedcba9876543211'))

    def test_source_shaped_fingerprints_accept_identical_and_reject_changed_state(self) -> None:
        for kind, original, changed in self.samples():
            with self.subTest(kind=kind):
                _ = compare(kind, json.dumps(original), json.dumps(original))
                other = deepcopy(original)
                other['fingerprint'] = changed
                with self.assertRaises(GateContractError):
                    _ = compare(kind, json.dumps(original), json.dumps(other))

    def test_malformed_fingerprints_reject_even_when_equal(self) -> None:
        invalid: tuple[Json, ...] = (None, True, 1, 1.0, [], {}, '', '0', 'g'*16,
                                    'ABCDEF0123456789', '0'*15, '0'*17, '0'*16+'\n')
        for kind, original, _ in self.samples():
            wrong_shape = '0'*16 if kind == 'm6a' else '0'*16+'-'+'0'*16
            for value in (*invalid, wrong_shape):
                with self.subTest(kind=kind, value=value):
                    malformed = deepcopy(original)
                    malformed['fingerprint'] = value
                    with self.assertRaises((GateContractError, EvidenceError)):
                        _ = compare(kind, json.dumps(malformed), json.dumps(malformed))
