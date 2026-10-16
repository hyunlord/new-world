# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow"]
# ///
# How to run: python3 test_activity_protocol_p11.py
"""Offline protocol checks; synthetic rectangles never count as capture evidence."""
from __future__ import annotations
import json
import math
import tempfile
import unittest
from pathlib import Path

from plan_activity_blind_p11 import covered_fraction, main as prepare_c
from prepare_activity_villages_p11 import main as prepare_d
from activity_rules import EvidenceError
from prepare_blind_crops import inventory
from run_activity_captures_p11 import bridge_identity, InvalidInput


class ProtocolTests(unittest.TestCase):
    def test_explicit_debug_and_strict_release_identity(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            checkout = Path(temp)
            debug = checkout / 'rust/target/debug/libsim_bridge.dylib'
            release = checkout / 'rust/target/release/libsim_bridge.dylib'
            debug.parent.mkdir(parents=True)
            debug.write_bytes(b'debug-test-fixture')
            identity = bridge_identity(checkout, 'debug')
            self.assertEqual(identity.build_profile, 'debug')
            self.assertIsNone(identity.release_sha256)
            self.assertTrue(identity.loaded_bridge_sha256)
            with self.assertRaises(InvalidInput):
                bridge_identity(checkout, 'release')
            with self.assertRaises(InvalidInput):
                bridge_identity(checkout, '')
            release.parent.mkdir(parents=True)
            release.write_bytes(b'different-release-fixture')
            self.assertIsNotNone(bridge_identity(checkout, 'debug').release_sha256)
            with self.assertRaises(InvalidInput):
                bridge_identity(checkout, 'release')
            release.write_bytes(debug.read_bytes())
            identity = bridge_identity(checkout, 'release')
            self.assertEqual(identity.loaded_bridge_sha256, identity.release_sha256)

    def test_union_coverage(self) -> None:
        self.assertEqual(covered_fraction([0, 0, 10, 10], [[0, 0, 5, 10]]), .5)
        self.assertEqual(covered_fraction([0, 0, 10, 10], [[0, 0, 5, 10]] * 2), .5)
        self.assertEqual(covered_fraction([0, 0, 10, 10], [[0, 0, 6, 10]]), .6)
        self.assertEqual(covered_fraction([0, 0, 10, 10], [[0, 0, 5, 10], [5, 0, 5, 10]]), 1)
        self.assertEqual(covered_fraction([0, 0, 10, 10], [[10, 0, 5, 10]]), 0)

    def test_geometry_equals_original_archive(self) -> None:
        for w in [8, 20, 31.5, 64, 72]:
            for h in [12, 32, 65.5, 96, 128]:
                row = {'visible': True, 'visible_in_tree': True, 'fully_inside_viewport': True,
                       'viewport_rect': [200.25, 300.75, w, h], 'kind': 0, 'agent_id': 1,
                       'entity_bits': 1, 'metadata': {'activity_overrides': []}}
                original = inventory([('fixture', {'viewport': [1920, 1080], 'activity_holders': [row]})])['crops'][0]
                side = math.ceil(max(w, h) + 16)
                left = math.floor(200.25 + w / 2 - side / 2)
                top = math.floor(300.75 + h / 2 - side / 2)
                self.assertEqual(original['box'], [left, top, left + side, top + side])
                self.assertEqual((original['upscale'], original['resampling']), (4, 'nearest'))

    def test_c_duplicate_capture_refused_before_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            out = Path(temp) / 'output'
            with self.assertRaises(EvidenceError):
                prepare_c([temp, str(out), 'same.json', 'same.json', 'third.json'])
            self.assertFalse(out.exists())

    def test_d_duplicate_capture_refused(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            answers = Path(temp) / 'answers.json'
            answers.write_text(json.dumps({'frozen': True}))
            out = Path(temp) / 'output'
            with self.assertRaises(RuntimeError):
                prepare_d([str(answers), str(out), 'same.json', 'same.json'])
            self.assertFalse(out.exists())


if __name__ == '__main__':
    unittest.main()
