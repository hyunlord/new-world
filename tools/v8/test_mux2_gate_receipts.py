#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_mux2_gate_receipts.py
"""Receipt counterexamples use synthetic data, never runtime acceptance."""
import copy
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import mux2_gate_receipts as receipts
from mux2_gate_visual import validate_why, validate_visibility
from mux2_gate_oracle_log import validate_log
from mux2_gate_inputs import snapshot, INPUT_DIRS, INPUT_FILES, BRIDGE
from mux2_gate_fixtures import validate_fixtures


from mux2_gate_testdata import stall_fixture


def adjust_intervals(report, indexes, interval):
    arm = report['arms'][0]
    elapsed = 0.0
    counts = [0] * 5
    slow = []
    for i, row in enumerate(arm['samples']):
        if i in indexes:
            row['interval_ms'] = interval
        elapsed = round(elapsed + row['interval_ms'] / 1000, 6)
        row['elapsed_seconds'] = elapsed
        row['post_draw_usec'] = 1000000 + round(elapsed * 1000000)
        row['telemetry']['frame_phases']['pre_to_post_draw_usec'] = round(row['interval_ms'] * 1000) - 4000
        if row['interval_ms'] > 33:
            slow.append(row)
            if elapsed > 10:
                counts[min(4, max(0, math.ceil((elapsed - 10) / 60) - 1))] += 1
    arm['score'].update(actual_play_seconds=elapsed, actual_post_warmup_seconds=elapsed-10,
                        post_warmup_over33=sum(counts), over33_per_minute=counts,
                        all_over33_frames=slow)


class CaptureCommandTests(unittest.TestCase):
    @patch.dict(os.environ, {}, clear=True)
    def test_stall_keeps_normal_windowed_rendering_and_full_duration(self) -> None:
        root = Path('/fixture/project')
        output = Path('/fixture/evidence')
        godot = Path('/fixture/Godot')
        command = receipts.commands(root, output, 'stall', godot)
        self.assertEqual(command, [[
            'bash', str(root / 'tools/v8/run_bounded.sh'), '960', str(godot),
            '--path', str(root), '--resolution', '1920x1080',
            '--always-on-top', '--windowed', '--script',
            'res://tools/v8/mux2_stall_test.gd', '--', str(output / 'run'),
        ]])
        self.assertNotIn('--headless', command[0])
        self.assertFalse(any('force_draw' in argument for argument in command[0]))

    def test_foreground_stall_change_preserves_visibility_and_why_commands(self) -> None:
        root = Path('/fixture/project')
        output = Path('/fixture/evidence')
        godot = Path('/fixture/Godot')
        base = ['bash', str(root / 'tools/v8/run_bounded.sh')]
        engine = [str(godot), '--path', str(root), '--resolution', '1920x1080']
        self.assertEqual(receipts.commands(root, output, 'visibility', godot), [
            base + ['1800'] + engine + ['--headless', '--script',
                'res://tools/v8/mux2_visibility_saves.gd', '--', str(output / 'saves')],
            base + ['1200'] + engine + ['--script', 'res://tools/v8/mux2_visibility_test.gd',
                '--', '--manifest', str(output / 'saves/manifest.json'),
                '--out', str(output / 'run')],
        ])
        self.assertEqual(receipts.commands(root, output, 'why', godot), [
            base + ['180'] + engine + ['--script', 'res://tools/v8/mux2_why_input_test.gd'],
        ])


class StallReceiptTests(unittest.TestCase):
    def test_accepts_complete_two_arm_samples(self):
        # Given complete synthetic measurements; when parsed; then both arms pass.
        self.assertIsNone(receipts.validate_stall(stall_fixture()))

    def test_rejects_invalid_acceptance_shapes(self):
        # Given one broken receipt field; when parsed; then reject each counterexample.
        for field, value in [("purpose", "diagnostic_120_seconds_zoom1"), ("complete", False),
                             ("profile_complete", False), ("errors", ["incomplete"]), ("seed", 1)]:
            with self.subTest(field=field):
                report = stall_fixture()
                report[field] = value
                with self.assertRaises(ValueError):
                    receipts.validate_stall(report)

    def test_rejects_one_arm_and_duplicate_zoom(self):
        for duplicate in (False, True):
            report = stall_fixture()
            report["arms"] = [report["arms"][0]] * (2 if duplicate else 1)
            with self.assertRaises(ValueError):
                receipts.validate_stall(report)

    def test_rejects_short_or_nan_samples(self):
        for key, value in [("elapsed_seconds", 309), ("interval_ms", float("nan"))]:
            report = stall_fixture()
            report["arms"][0]["samples"][-1][key] = value
            with self.assertRaises(ValueError):
                receipts.validate_stall(report)

    def test_rejects_one_long_frame_even_if_summary_claims_pass(self):
        report = stall_fixture()
        adjust_intervals(report, {2000}, 51)
        with self.assertRaisesRegex(ValueError, "stall acceptance bar"):
            receipts.validate_stall(report)

    def test_rejects_four_slow_frames_in_one_minute(self):
        report = stall_fixture()
        adjust_intervals(report, set(range(2000, 2004)), 34)
        with self.assertRaisesRegex(ValueError, "stall acceptance bar"):
            receipts.validate_stall(report)

    def test_allows_exact_fifty_and_three_slow_frames(self):
        report = stall_fixture()
        adjust_intervals(report, {2000, 2100, 2200}, 50)
        self.assertIsNone(receipts.validate_stall(report))

    def test_rejects_missing_native_status_despite_complete_flag(self):
        report = stall_fixture()
        report['arms'][0]['instrumentation']['native'] = 'NOT_MEASURED'
        with self.assertRaises(ValueError):
            receipts.validate_stall(report)

    def test_rejects_empty_native_coverage(self):
        report = stall_fixture()
        report["arms"][0]["instrumentation"]["complete"] = False
        with self.assertRaises(ValueError):
            receipts.validate_stall(report)

    def test_rejects_false_summary_and_headless(self):
        for target, key, value in [("score", "post_warmup_over50", 1),
                                   ("environment", "render_driver", "dummy")]:
            report = stall_fixture()
            node = report["environment"] if target == "environment" else report["arms"][0]["score"]
            node[key] = value
            with self.assertRaises(ValueError):
                receipts.validate_stall(report)

    def test_cli_rejects_missing_receipt(self):
        # Given no receipt; when CLI executes; then a real nonzero process result.
        result = subprocess.run([sys.executable, str(Path(receipts.__file__)), "verify",
                                 "--receipt", "/missing/receipt.json", "--kind", "stall"],
                                capture_output=True, text=True, check=False)
        self.assertNotEqual(result.returncode, 0)


class FreezeReceiptTests(unittest.TestCase):
    def test_rejects_missing_deleted_changed_or_added_inputs(self):
        for current in [{}, {"a": "b" * 64}, {"a": "a" * 64, "b": "b" * 64}]:
            with self.assertRaises(ValueError):
                receipts.same_inputs({"a": "a" * 64}, current)

    def test_rejects_failed_or_incomplete_capture(self):
        for changes in [{"complete": False}, {"exit_code": 2}, {"kind": "why"}]:
            receipt = {"schema": 1, "complete": True, "exit_code": 0, "kind": "stall"}
            receipt.update(changes)
            with self.assertRaises(ValueError):
                receipts.validate_capture_status(receipt, "stall")

    def test_rejects_changed_artifact(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "report.json"
            path.write_text(json.dumps({"changed": True}))
            with self.assertRaises(ValueError):
                receipts.check_hash(path, "0" * 64)


class VisualReceiptTests(unittest.TestCase):
    def test_extent_requires_its_exact_result_marker(self) -> None:
        # Given all75 producer checks; when parsed with RESULT; then accept.
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'extent.log'
            path.write_text(''.join(f'MUX2_BADGE_EXTENT_PASS case_{i}\n' for i in range(75))
                            + 'MUX2_BADGE_EXTENT_RESULT checks=75 failures=0\n')
            self.assertIsNone(validate_log('mux2_badge_extent_test', path))

    def test_extent_rejects_wrong_missing_duplicate_or_failed_completion(self) -> None:
        # Given mutated extent coverage/completions; when parsed; then reject.
        prefix = 'MUX2_BADGE_EXTENT'
        good = f'{prefix}_RESULT checks=75 failures=0\n'
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'extent.log'
            rows = ''.join(f'{prefix}_PASS case_{i}\n' for i in range(75))
            endings = ['', good.replace('RESULT', 'DONE'), good + good,
                       good + good.replace('RESULT', 'DONE'),
                       good.replace('failures=0', 'failures=1'),
                       good + 'SCRIPT ERROR: deliberate\n', good + f'{prefix}_FAIL case\n',
                       good + f'{prefix}_RESULT checks=75 failures=1\n']
            for index, ending in enumerate(endings):
                with self.subTest(mutation=index):
                    path.write_text(rows + ending)
                    with self.assertRaises(ValueError):
                        validate_log('mux2_badge_extent_test', path)
            for count in (74, 76):
                with self.subTest(count=count):
                    path.write_text(''.join(f'{prefix}_PASS case_{i}\n' for i in range(count))
                                    + f'{prefix}_RESULT checks={count} failures=0\n')
                    with self.assertRaises(ValueError):
                        validate_log('mux2_badge_extent_test', path)

    def test_done_oracle_rejects_result_alias(self) -> None:
        # Given a non-extent fixture; when RESULT replaces DONE; then reject.
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'person.log'
            path.write_text(''.join(f'MUX2_PERSON_CACHE_PASS case_{i}\n' for i in range(182))
                            + 'MUX2_PERSON_CACHE_RESULT checks=182 failures=0\n')
            with self.assertRaises(ValueError):
                validate_log('mux2_person_cache_test', path)

    def test_geometry_oracles_require_current_exact_coverage(self) -> None:
        # Given current producer counts; when parsed; then all geometry oracles pass.
        cases = [('mux2_coincident_test', 'MUX2_COINCIDENT', 30),
                 ('mux2_badge_index_test', 'MUX2_BADGE_INDEX', 116),
                 ('mux2_badge_packing_test', 'MUX2_BADGE_PACKING', 114),
                 ('mux2_overlap_test', 'MUX2_OVERLAP', 81),
                 ('mux2_badge_grid_test', 'MUX2_BADGE_GRID', 514),
                 ('mux2_badge_aggregate_test', 'MUX2_BADGE_AGGREGATE', 97),
                 ('mux2_badge_local_test', 'MUX2_BADGE_LOCAL', 205),
                 ('mux2_bake_receipt_test', 'MUX2_BAKE_RECEIPT', 20),
                 ('mux2_person_cache_test', 'MUX2_PERSON_CACHE', 182),
                 ('mux2_extent_owner_test', 'MUX2_EXTENT_OWNER', 99)]
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'geometry.log'
            for name, prefix, count in cases:
                with self.subTest(name=name):
                    path.write_text(''.join(f'{prefix}_PASS case_{index}\n'
                                            for index in range(count))
                                    + f'{prefix}_DONE checks={count} failures=0\n')
                    self.assertIsNone(validate_log(name, path))

    def test_geometry_oracles_reject_missing_extra_and_failed_checks(self) -> None:
        # Given truncated, stale, extra or failed logs; when parsed; then reject.
        cases = [('mux2_coincident_test', 'MUX2_COINCIDENT', 30),
                 ('mux2_badge_index_test', 'MUX2_BADGE_INDEX', 116),
                 ('mux2_badge_packing_test', 'MUX2_BADGE_PACKING', 114),
                 ('mux2_overlap_test', 'MUX2_OVERLAP', 81),
                 ('mux2_badge_grid_test', 'MUX2_BADGE_GRID', 514),
                 ('mux2_badge_aggregate_test', 'MUX2_BADGE_AGGREGATE', 97),
                 ('mux2_badge_local_test', 'MUX2_BADGE_LOCAL', 205),
                 ('mux2_bake_receipt_test', 'MUX2_BAKE_RECEIPT', 20),
                 ('mux2_person_cache_test', 'MUX2_PERSON_CACHE', 182),
                 ('mux2_extent_owner_test', 'MUX2_EXTENT_OWNER', 99)]
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'geometry.log'
            for name, prefix, count in cases:
                valid = ''.join(f'{prefix}_PASS case_{index}\n' for index in range(count))
                done = f'{prefix}_DONE checks={count} failures=0\n'
                malformed = [valid, valid + done + done,
                             valid + done + f'{prefix}_FAIL deliberate\n',
                             valid + done + 'SCRIPT ERROR: deliberate\n']
                for wrong_count in (17, 177, count - 1, count + 1):
                    malformed.append(''.join(f'{prefix}_PASS case_{index}\n'
                                              for index in range(wrong_count))
                                     + f'{prefix}_DONE checks={wrong_count} failures=0\n')
                for index, body in enumerate(malformed):
                    with self.subTest(name=name, mutation=index):
                        path.write_text(body)
                        with self.assertRaises(ValueError):
                            validate_log(name, path)

    def test_wall_oracle_uses_real_hash_marker_contract(self):
        # Given canonical wall logs without per-check PASS lines; when parsed; then accept.
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'wall.log'
            names = ['base', 'cache_hit', 'material_alias', 'changed_tint', 'changed_top',
                     'reloaded_images', 'after_invalidation']
            path.write_text(''.join('MUX2_WALL_CACHE_HASHES ' + name + ' {}\n' for name in names)
                            + 'MUX2_WALL_CACHE_DONE failures=[]\n')
            self.assertIsNone(validate_log('mux2_wall_cache_test', path))

    def test_why_requires_both_real_scale_names(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            for scale in ('1.0', '1.25'):
                (directory / f'why-{scale}.png').write_bytes(b'PNG fixture')
            path = directory / 'why.log'
            path.write_text('MUX2_WHY_PASS case\n' * 42 + 'MUX2_WHY_OUTPUT ' + str(directory)
                            + '\nMUX2_WHY_DONE failures=0 checks=42\n')
            self.assertEqual(len(validate_why(path)), 2)

    def test_why_rejects_old_wrong_or_incomplete_coverage(self) -> None:
        # Given42 original-plus-placement checks; when evidence is mutated; then reject.
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            for scale in ('1.0', '1.25'):
                (directory / f'why-{scale}.png').write_bytes(b'PNG fixture')
            path = directory / 'why.log'
            rows = 'MUX2_WHY_PASS case\n' * 42
            output = f'MUX2_WHY_OUTPUT {directory}\n'
            done = 'MUX2_WHY_DONE failures=0 checks=42\n'
            bodies = [rows + output, rows + output + done + done,
                      rows + output + done + 'MUX2_WHY_DONE failures=1 checks=42\n',
                      rows + output + done + 'MUX2_WHY_FAIL deliberate\n',
                      rows + output + done + 'SCRIPT ERROR: deliberate\n',
                      rows + done]
            for count in (30, 41, 43):
                bodies.append('MUX2_WHY_PASS case\n' * count + output
                              + f'MUX2_WHY_DONE failures=0 checks={count}\n')
                bodies.append('MUX2_WHY_PASS case\n' * count + output + done)
            for index, body in enumerate(bodies):
                with self.subTest(mutation=index):
                    path.write_text(body)
                    with self.assertRaises(ValueError):
                        validate_why(path)

    def test_why_rejects_missing_scale_images(self) -> None:
        # Given a complete42-check log; when either required scale image is absent; then reject.
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            path = directory / 'why.log'
            path.write_text('MUX2_WHY_PASS case\n' * 42
                            + f'MUX2_WHY_OUTPUT {directory}\n'
                            + 'MUX2_WHY_DONE failures=0 checks=42\n')
            for missing in ('1.0', '1.25'):
                for scale in ('1.0', '1.25'):
                    image = directory / f'why-{scale}.png'
                    image.unlink(missing_ok=True)
                    if scale != missing:
                        image.write_bytes(b'PNG fixture')
                with self.subTest(missing=missing):
                    with self.assertRaises(ValueError):
                        validate_why(path)

    def test_visibility_rejects_old_bridge_before_accepting_saved_scene(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'root'
            (root / BRIDGE).parent.mkdir(parents=True)
            (root / BRIDGE).write_bytes(b'current bridge')
            out = Path(tmp) / 'out'
            (out / 'saves').mkdir(parents=True)
            (out / 'saves/manifest.json').write_text(json.dumps(
                {'schema': 1, 'seed': 0, 'steward': 'on', 'bridge_sha256': '0' * 64}))
            with self.assertRaisesRegex(ValueError, 'bridge_sha256'):
                validate_visibility(root, out)

    def test_freeze_detects_new_source_and_changed_import(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name in INPUT_DIRS:
                (root / name).mkdir(parents=True, exist_ok=True)
            for name in (*INPUT_FILES, BRIDGE, '.godot/imported/texture.ctex'):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b'original')
            before = snapshot(root)
            (root / 'scripts/added.gd').write_text('new source')
            with self.assertRaises(ValueError):
                receipts.same_inputs(before, snapshot(root))
            (root / 'scripts/added.gd').unlink()
            (root / '.godot/imported/texture.ctex').write_bytes(b'changed')
            with self.assertRaises(ValueError):
                receipts.same_inputs(before, snapshot(root))

    def test_genuine_fixtures_require_inventory_and_old_format(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            hashes = {}
            for i in range(4):
                path = root / f'{i}.wsave'
                path.write_bytes(b'WORLDSIM' + (10).to_bytes(4, 'little'))
                hashes[path.name] = receipts.digest(path)
            manifest = root / 'manifest.json'
            manifest.write_text(json.dumps(hashes))
            self.assertIsNone(validate_fixtures(root, manifest))
            (root / '0.wsave').write_bytes(b'WORLDSIM' + (11).to_bytes(4, 'little'))
            with self.assertRaises(ValueError):
                validate_fixtures(root, manifest)


if __name__ == "__main__":
    unittest.main()
