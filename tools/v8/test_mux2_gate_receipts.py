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
import mux2_gate_visual as visual
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
    def test_rejects_runtime_errors_before_downstream_validation(self) -> None:
        # Given a zero-exit capture log containing a Godot diagnostic, even with a completion marker.
        diagnostics = ('ERROR: Error compiling Fragment shader.',
                       'SHADER ERROR: Invalid arguments for operator +.',
                       'ERROR: Condition "variant_stages.is_empty()" is true.')
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp)
            for diagnostic in diagnostics:
                (output / 'command.log').write_text(f'W3D_SHOTS_DONE n=4\n{diagnostic}\n')
                with self.subTest(diagnostic=diagnostic), patch.object(receipts, 'validate_why', return_value=[]) as downstream:
                    # When a receipt is validated, then runtime errors reject it before its success oracle runs.
                    with self.assertRaisesRegex(receipts.EvidenceError, 'capture runtime error'):
                        receipts.validate_output(output, output, 'why')
                    downstream.assert_not_called()

    def test_retired_2d_oracles_are_unknown(self) -> None:
        # M3D-SWITCH E2: the 2D badge, marker, cache and bake oracles went with the 2D renderer; their logs
        # cannot stand in for a current oracle.
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'retired.log'
            for name, prefix in [('mux2_badge_extent_test', 'MUX2_BADGE_EXTENT'),
                                 ('mux2_person_cache_test', 'MUX2_PERSON_CACHE'),
                                 ('mux2_wall_cache_test', 'MUX2_WALL_CACHE'),
                                 ('mux2_bake_receipt_test', 'MUX2_BAKE_RECEIPT')]:
                path.write_text(f'{prefix}_PASS case\n{prefix}_DONE checks=1 failures=0\n')
                with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'unknown oracle'):
                    validate_log(name, path)

    def test_visibility_validator_oracle_exact_coverage(self) -> None:
        prefix = 'MUX2_VISIBILITY_ORACLE'
        count = 27
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'validator.log'
            rows = ''.join(f'{prefix}_PASS case_{i}\n' for i in range(count))
            done = f'{prefix}_DONE checks={count} failures=0\n'
            path.write_text(rows + done)
            self.assertIsNone(validate_log('mux2_visibility_validator_test', path))
            malformed = [rows, rows + done + done, rows + done + f'{prefix}_FAIL deliberate\n',
                         rows + done + 'SCRIPT ERROR: deliberate\n']
            for wrong in (count - 1, count + 1):
                malformed.append(f'{prefix}_PASS case\n' * wrong + f'{prefix}_DONE checks={wrong} failures=0\n')
            for body in malformed:
                with self.subTest(body=body[-80:]):
                    path.write_text(body)
                    with self.assertRaises(ValueError):
                        validate_log('mux2_visibility_validator_test', path)

    def test_camp_framing_exact_pure_coverage(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'camp.log'
            rows = ''.join(f'CAMP_FRAMING PASS case_{i}\n' for i in range(12))
            done = 'CAMP_FRAMING_DONE checks=12 failures=0\n'
            path.write_text(rows + done)
            self.assertIsNone(validate_log('mux2_camp_framing_test', path))
            malformed = [rows, rows + done + done,
                         rows + 'CAMP_FRAMING_DONE failures=0\n',
                         rows + done + 'SCRIPT ERROR: deliberate\n',
                         rows + done + 'CAMP_FRAMING FAIL deliberate\n']
            for count in (11, 13):
                malformed.append('CAMP_FRAMING PASS case\n' * count
                                 + f'CAMP_FRAMING_DONE checks={count} failures=0\n')
            for body in malformed:
                with self.subTest(body=body[-100:]):
                    path.write_text(body)
                    with self.assertRaises(ValueError):
                        validate_log('mux2_camp_framing_test', path)

    def test_locale_why_requires_exact_64_checks(self) -> None:
        rows = 'MUX2_WHY_PASS case\n' * 64
        done = 'MUX2_WHY_DONE failures=0 checks=64\n'
        self.assertIsNone(visual.validate_why_checks(rows + done, 64))
        bodies = [rows, rows + done + done,
                  rows + done + 'MUX2_WHY_FAIL deliberate\n',
                  rows + done + 'SCRIPT ERROR: deliberate\n',
                  rows + 'MUX2_WHY_DONE failures=1 checks=64\n']
        for count in (53, 63, 65):
            bodies.append('MUX2_WHY_PASS case\n' * count + done)
            bodies.append('MUX2_WHY_PASS case\n' * count
                          + f'MUX2_WHY_DONE failures=0 checks={count}\n')
        for body in bodies:
            with self.subTest(body=body[-100:]), self.assertRaises(ValueError):
                visual.validate_why_checks(body, 64)

    def test_why_requires_both_real_scale_names(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            for scale in ('1.0', '1.25'):
                (directory / f'why-{scale}.png').write_bytes(b'PNG fixture')
            path = directory / 'why.log'
            path.write_text('MUX2_WHY_PASS case\n' * 53 + 'MUX2_WHY_OUTPUT ' + str(directory)
                            + '\nMUX2_WHY_DONE failures=0 checks=53\n')
            self.assertEqual(len(validate_why(path)), 2)

    def test_why_rejects_old_wrong_or_incomplete_coverage(self) -> None:
        # Given53 original-plus-placement checks; when evidence is mutated; then reject.
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            for scale in ('1.0', '1.25'):
                (directory / f'why-{scale}.png').write_bytes(b'PNG fixture')
            path = directory / 'why.log'
            rows = 'MUX2_WHY_PASS case\n' * 53
            output = f'MUX2_WHY_OUTPUT {directory}\n'
            done = 'MUX2_WHY_DONE failures=0 checks=53\n'
            bodies = [rows + output, rows + output + done + done,
                      rows + output + done + 'MUX2_WHY_DONE failures=1 checks=53\n',
                      rows + output + done + 'MUX2_WHY_FAIL deliberate\n',
                      rows + output + done + 'SCRIPT ERROR: deliberate\n',
                      rows + done]
            for count in (30, 42, 52, 54):
                bodies.append('MUX2_WHY_PASS case\n' * count + output
                              + f'MUX2_WHY_DONE failures=0 checks={count}\n')
                bodies.append('MUX2_WHY_PASS case\n' * count + output + done)
            for index, body in enumerate(bodies):
                with self.subTest(mutation=index):
                    path.write_text(body)
                    with self.assertRaises(ValueError):
                        validate_why(path)

    def test_why_rejects_missing_scale_images(self) -> None:
        # Given a complete53-check log; when either required scale image is absent; then reject.
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            path = directory / 'why.log'
            path.write_text('MUX2_WHY_PASS case\n' * 53
                            + f'MUX2_WHY_OUTPUT {directory}\n'
                            + 'MUX2_WHY_DONE failures=0 checks=53\n')
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

    def test_visibility_scene_requires_3d_full_figures(self) -> None:
        # Given a complete synthetic 3D scene; when it claims LOD markers (the retired 2D quarter zoom); then reject.
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp)
            for name, half in (('day1_noon_half', True), ('day1_noon_quarter', False)):
                directory = output / name
                directory.mkdir()
                files = {}
                for filename in ('full.png', 'ground.png', 'person-7.png', 'without-7.png', 'restored-7.png'):
                    (directory / filename).write_bytes(filename.encode())
                    files[filename] = receipts.digest(directory / filename)
                minimum = 60 if half else 140
                scene = {'id': name, 'zoom': 0.5 if half else 0.25, 'lod': False, 'viewport': [1920, 1080],
                         'pass': True, 'on_screen_ids': [7], 'off_screen_ids': [8], 'partial': [], 'all_ids': [7, 8],
                         'rows': [{'entity_bits': 7, 'pass': True, 'valid': True, 'required_pixels': minimum,
                                   'qualifying_pixels': minimum, 'full_sha256': files['person-7.png'],
                                   'removed_sha256': files['without-7.png'], 'restored_sha256': files['restored-7.png']}],
                         'full_sha256': files['full.png'], 'ground_sha256': files['ground.png']}
                self.assertIsNone(visual.validate_scene(visual.Node(scene), output))
                broken = copy.deepcopy(scene)
                broken['lod'] = True
                with self.subTest(name=name), self.assertRaises(ValueError):
                    visual.validate_scene(visual.Node(broken), output)

    def test_visibility_rejects_a_2d_receipt(self) -> None:
        # Given every input current; when the report was measured on another renderer; then reject.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'root'
            (root / BRIDGE).parent.mkdir(parents=True)
            (root / BRIDGE).write_bytes(b'current bridge')
            (root / 'tools/v8').mkdir(parents=True)
            (root / 'tools/v8/mux2_visibility_saves.gd').write_text('saves')
            out = Path(tmp) / 'out'
            (out / 'saves').mkdir(parents=True)
            (out / 'run').mkdir()
            scenes = []
            for day in (1, 20):
                for hour, phase in ((12, 'noon'), (22, 'night')):
                    save = out / 'saves' / f'day{day}_{phase}.wsave'
                    save.write_bytes(f'{day}{phase}'.encode())
                    scenes.append({'id': f'day{day}_{phase}', 'day': day, 'hour': hour, 'save': str(save),
                                   'sha256': receipts.digest(save)})
            (out / 'saves/manifest.json').write_text(json.dumps(
                {'schema': 1, 'seed': 0, 'steward': 'on', 'bridge_sha256': receipts.digest(root / BRIDGE),
                 'script_sha256': receipts.digest(root / 'tools/v8/mux2_visibility_saves.gd'), 'scenes': scenes}))
            (out / 'run/visibility.json').write_text(json.dumps({'schema': 2, 'renderer': '2d'}))
            with self.assertRaisesRegex(ValueError, 'renderer'):
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
