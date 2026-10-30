#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_m6a_r1_stall_capture.py
"""Mocked subprocess receipts test sealing and rejection, never GPU performance."""
from contextlib import ExitStack, nullcontext
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import m6a_r1_stall_capture as capture
from m6a_r1_stall import evaluate
from mux2_gate_inputs import EvidenceError, Node
from test_m6a_r1_stall import changed_frames


def r1_environment():
    """The R1 fixtures pin R1's deadline: drop a gate-exported M6A_DEADLINE_EPOCH (R2) from this
    process's environment copy only, as P12 isolated the other historical fixtures."""
    return patch.dict(capture.os.environ, {key: value for key, value in capture.os.environ.items()
                                           if key != 'M6A_DEADLINE_EPOCH'}, clear=True)


class CaptureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.verdict = evaluate(changed_frames({0: {2000: 51}}))

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.root = self.directory / 'project'
        self.root.mkdir()
        self.out = self.directory / 'capture'
        self.godot = self.directory / 'Godot'
        self.godot.write_bytes(b'fake executable, never launched')
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.stack.enter_context(r1_environment())
        self.stack.enter_context(patch.object(capture, 'clean_sources'))
        self.stack.enter_context(patch.object(capture, 'git', return_value='a' * 40))
        self.snapshot = self.stack.enter_context(patch.object(capture, 'snapshot', return_value={'input': 'b' * 64}))
        self.stack.enter_context(patch.object(capture, 'lane', return_value=nullcontext(99)))
        self.stack.enter_context(patch.object(capture.time, 'time', return_value=capture.R1_DEADLINE - 1000))
        self.validation = self.stack.enter_context(patch.object(capture, 'validate_output', return_value=self.verdict))
        self.process = self.stack.enter_context(patch.object(capture.subprocess, 'run', side_effect=self.process_result))

    def process_result(self, argv, **kwargs):
        (self.out / 'run').mkdir()
        (self.out / 'run/stall-report.json').write_text('{"pass":false}\n')
        return subprocess.CompletedProcess(argv, 1)

    def test_complete_strict_fail_seals_hashes_and_preserves_exit(self):
        verdict = capture.capture(self.root, self.out, self.godot)
        receipt = json.loads((self.out / 'receipt.json').read_text())
        self.assertTrue(receipt['complete'])
        self.assertEqual(receipt['exit_code'], 1)
        self.assertFalse(receipt['verdict']['strict_pass'])
        self.assertTrue(receipt['artifacts'])
        self.assertEqual(capture.verify_capture(self.root, self.out / 'receipt.json'), verdict)
        environment = self.process.call_args.kwargs['env']
        self.assertEqual(environment['CORE_P7_DEADLINE_EPOCH'], str(capture.R1_DEADLINE))
        self.assertEqual(environment['CORE_P7_RUN'], '1')

    def test_wrong_exit_or_invalid_capture_stays_incomplete(self):
        for code in (0, 2, -9):
            with self.subTest(code=code), tempfile.TemporaryDirectory(dir=self.directory) as folder:
                self.out = Path(folder) / 'capture'
                self.process.side_effect = lambda argv, **kwargs: subprocess.CompletedProcess(argv, code)
                with self.assertRaisesRegex(EvidenceError, 'exit differs'):
                    capture.capture(self.root, self.out, self.godot)
                receipt = json.loads((self.out / 'receipt.json').read_text())
                self.assertFalse(receipt['complete'])
                self.assertEqual(receipt['exit_code'], code)
        self.out = self.directory / 'invalid'
        self.validation.side_effect = EvidenceError('capture error')
        with self.assertRaisesRegex(EvidenceError, 'capture error'):
            capture.capture(self.root, self.out, self.godot)
        self.assertFalse(json.loads((self.out / 'receipt.json').read_text())['complete'])

    def test_capture_rejects_changed_source(self):
        self.snapshot.side_effect = [{'input': 'b' * 64}, {'input': 'c' * 64}, {'input': 'c' * 64}]
        with self.assertRaisesRegex(EvidenceError, 'input content'):
            capture.capture(self.root, self.out, self.godot)
        self.assertFalse(json.loads((self.out / 'receipt.json').read_text())['complete'])

    def test_verify_rejects_changed_extra_and_forged_artifacts(self):
        capture.capture(self.root, self.out, self.godot)
        raw = self.out / 'run/stall-report.json'
        saved = raw.read_text()
        raw.write_text('changed')
        with self.assertRaisesRegex(EvidenceError, 'changed artifact'):
            capture.verify_capture(self.root, self.out / 'receipt.json')
        raw.write_text(saved)
        extra = self.out / 'extra'
        extra.write_text('unexpected')
        with self.assertRaisesRegex(EvidenceError, 'inventory changed'):
            capture.verify_capture(self.root, self.out / 'receipt.json')
        extra.unlink()
        receipt_path = self.out / 'receipt.json'
        receipt = json.loads(receipt_path.read_text())
        receipt['verdict']['strict_pass'] = True
        receipt_path.write_text(json.dumps(receipt))
        with self.assertRaises(EvidenceError):
            capture.verify_capture(self.root, receipt_path)

    def test_verify_rejects_changed_binary_and_source(self):
        capture.capture(self.root, self.out, self.godot)
        self.godot.write_bytes(b'changed')
        with self.assertRaisesRegex(EvidenceError, 'changed artifact'):
            capture.verify_capture(self.root, self.out / 'receipt.json')
        self.snapshot.return_value = {'input': 'c' * 64}
        with self.assertRaisesRegex(EvidenceError, 'input content'):
            capture.verify_capture(self.root, self.out / 'receipt.json')

    def test_command_retains_ordinary_window_and_deadline(self):
        argv = capture.command(self.root, self.out, self.godot)
        self.assertEqual(argv[2], '1000')
        self.assertIn('--windowed', argv)
        self.assertIn('--always-on-top', argv)
        self.assertNotIn('--headless', argv)
        with patch.object(capture.time, 'time', return_value=capture.R1_DEADLINE - 649):
            with self.assertRaisesRegex(EvidenceError, 'cannot fit'):
                capture.command(self.root, self.out, self.godot)


class OutputTests(unittest.TestCase):
    def setUp(self):
        environment = r1_environment()
        environment.start()
        self.addCleanup(environment.stop)

    def test_output_rejects_bridge_and_source_hash_mismatch(self):
        raw = changed_frames({0: {2000: 51}})
        raw['p7_admission'] = {'deadline_epoch': capture.R1_DEADLINE}
        raw['bridge']['sha256'] = 'a' * 64
        raw['source_sha256'] = {'source.gd': 'a' * 64}
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'command.log').write_text('complete process')
            (root / 'run').mkdir()
            (root / 'run/stall-report.json').write_text('raw fixture')
            (root / 'settings').mkdir()
            for zoom in ('1.0', '0.5'):
                (root / f'settings/stall-123-{zoom}.cfg').write_text('arm settings')
            raw['environment']['settings_dir'] = str(root / 'settings')
            with patch.object(capture, 'read_json', return_value=Node(raw)), \
                 patch.object(capture, 'validate_p7_admission'), \
                 patch.object(capture, 'producer_sources', return_value={'source.gd'}), \
                 patch.object(capture, 'digest', return_value='b' * 64):
                with self.assertRaises(EvidenceError):
                    capture.validate_output(root, root)
            with patch.object(capture, 'read_json', return_value=Node(raw)), \
                 patch.object(capture, 'validate_p7_admission'), \
                 patch.object(capture, 'producer_sources', return_value={'source.gd'}), \
                 patch.object(capture, 'digest', return_value='a' * 64):
                with self.assertRaisesRegex(EvidenceError, 'changed artifact'):
                    capture.validate_output(root, root)
            with patch.object(capture, 'read_json', return_value=Node(raw)), \
                 patch.object(capture, 'validate_p7_admission'), \
                 patch.object(capture, 'producer_sources', return_value={'source.gd', 'missing.gd'}), \
                 patch.object(capture, 'digest', return_value='a' * 64):
                with self.assertRaisesRegex(EvidenceError, 'source hash inventory'):
                    capture.validate_output(root, root)


class RequiredArtifactTests(unittest.TestCase):
    def test_missing_raw_or_arm_settings_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder)
            with self.assertRaisesRegex(EvidenceError, 'required capture artifact'):
                capture.required_artifacts(output)
            (output / 'command.log').write_text('completed')
            (output / 'run').mkdir()
            (output / 'run/stall-report.json').write_text('report')
            (output / 'settings').mkdir()
            with self.assertRaisesRegex(EvidenceError, 'per-arm settings'):
                capture.required_artifacts(output)
            for zoom in ('1.0', '0.5'):
                (output / f'settings/stall-1-{zoom}.cfg').write_text('settings')
            capture.required_artifacts(output)

    def test_literal_source_manifest_is_required_and_unique(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / 'tools/v8/mux2_stall_test.gd'
            source.parent.mkdir(parents=True)
            source.write_text('const SOURCE_PATHS: Array[String] = ["one", "two",]')
            self.assertEqual(capture.producer_sources(root), {'one', 'two'})
            for value in ('[]', '["one", "one"]', '[dynamic_source]'):
                source.write_text('const SOURCE_PATHS: Array[String] = ' + value)
                with self.assertRaises(EvidenceError):
                    capture.producer_sources(root)


if __name__ == '__main__':
    unittest.main()


class RepairDeadlineTests(unittest.TestCase):
    def test_capture_follows_the_gate_exported_repair_deadline(self):
        # R2 brief: the stall lane is bounded by the active repair round's deadline.
        with patch.dict(capture.os.environ, {'M6A_DEADLINE_EPOCH': '1793414280'}):
            self.assertEqual(capture.deadline(), 1793414280)
        with r1_environment():
            self.assertEqual(capture.deadline(), capture.R1_DEADLINE)
