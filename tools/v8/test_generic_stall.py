"""Synthetic filesystem and telemetry tests; never launch Godot or a simulated world."""
import json
import subprocess
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

from generic_stall import CaptureBinding, assess_timing, authenticate_capture
from mux2_gate_inputs import (
    BRIDGE,
    INPUT_DIRS,
    INPUT_FILES,
    EvidenceError,
    Json,
    Node,
    digest,
    snapshot,
)
from mux2_gate_testdata import stall_fixture
from test_mux2_gate_receipts import adjust_intervals


def report_with(interval: float = 25.0, count: int = 0, crowded: bool = False) -> dict[str, Json]:
    raw = Node(json.loads(json.dumps(stall_fixture()))).mapping()
    if crowded:
        adjust_intervals(raw, set(range(500, 505)), 40.0)
    if count:
        adjust_intervals(raw, set(range(600, 600 + count)), interval)
    arm = Node(raw).field('arms').array()[0]
    over50 = sum(row.field('interval_ms').number() > 50 and row.field('elapsed_seconds').number() > 10 for row in arm.field('samples').array())
    score = arm.field('score').mapping()
    score['post_warmup_over50'] = over50
    score['pass'] = over50 == 0 and all(n.number() <= 3 for n in Node(score).field('over33_per_minute').array())
    raw['pass'] = raw['bar_pass'] = score['pass']
    return raw


class TimingTests(unittest.TestCase):
    def test_strict_and_eligible_keep_original_counts(self) -> None:
        # Given real validator-compatible synthetic frames; when scored; then no final admission.
        self.assertTrue(assess_timing(json.dumps(report_with())).strict_pass)
        verdict = assess_timing(json.dumps(report_with(55.0, 1, True)))
        self.assertEqual(verdict.status, 'WAIVER_ELIGIBLE')
        self.assertFalse(verdict.strict_pass)
        self.assertEqual(verdict.arms[0].over33_per_minute[0], 6)
        self.assertEqual(verdict.arms[0].over50[0].interval_ms, 55.0)

    def test_exact_two_and_sixty_boundaries(self) -> None:
        self.assertEqual(assess_timing(json.dumps(report_with(60.0, 2))).status, 'WAIVER_ELIGIBLE')
        for raw in (report_with(60.001, 1), report_with(55.0, 3), report_with(crowded=True)):
            with self.subTest(), self.assertRaises(EvidenceError):
                assess_timing(json.dumps(raw))

    def test_incomplete_bad_telemetry_and_ambiguous_json_rejected(self) -> None:
        mutations: tuple[tuple[str, Json], ...] = (('complete', False), ('errors', ['capture error']), ('pass', False))
        for key, value in mutations:
            raw = report_with()
            raw[key] = value
            with self.subTest(key=key), self.assertRaises(EvidenceError):
                assess_timing(json.dumps(raw))
        for raw in ('{"a":1,"a":2}', '{"a":NaN}', '{"a":1e999}', '[]'):
            with self.subTest(raw=raw), self.assertRaises(EvidenceError):
                assess_timing(raw)


class CaptureTests(unittest.TestCase):
    def __init__(self, methodName: str = 'runTest') -> None:
        super().__init__(methodName)
        self.root = Path('/uninitialized/source')
        self.capture = Path('/uninitialized/capture')
        self.godot = Path('/uninitialized/godot')
        self.sha = ''
        self.argv: tuple[str, ...] = ()
        self.receipt: dict[str, Json] = {}

    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        top = Path(temporary.name).resolve()
        self.root, self.capture = top / 'source', top / 'capture'
        self.root.mkdir()
        for name in INPUT_DIRS:
            (self.root / name).mkdir(parents=True, exist_ok=True)
        (self.root / '.godot/imported').mkdir(parents=True)
        for name in (*INPUT_FILES, BRIDGE):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('synthetic input')
        (self.root / 'scripts/producer.gd').write_text('# synthetic source')
        (self.root / 'tools/v8/mux2_stall_test.gd').write_text('const SOURCE_PATHS: Array[String] = ["scripts/producer.gd"]')
        def git(*args: str) -> str:
            return subprocess.check_output(['git', '-C', str(self.root), *args], stderr=subprocess.DEVNULL).decode().strip()
        git('init', '-q')
        git('add', '.')
        git('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '-qm', 'synthetic source')
        self.sha = git('rev-parse', 'HEAD')
        self.godot = top / 'godot'
        self.godot.write_text('never executable or launched')
        (self.capture / 'run').mkdir(parents=True)
        (self.capture / 'settings').mkdir()
        for zoom in ('1.0', '0.5'):
            (self.capture / f'settings/stall-1-{zoom}.cfg').write_text('synthetic settings')
        self.argv = ('bash', str(self.root / 'tools/v8/run_bounded.sh'), '960', str(self.godot),
                     '--path', str(self.root), '--windowed', '--script',
                     'res://tools/v8/mux2_stall_test.gd', '--', str(self.capture / 'run'))

    def prepare(self, strict: bool = False) -> CaptureBinding:
        raw = report_with() if strict else report_with(55.0, 1, True)
        Node(raw).field('environment').mapping()['settings_dir'] = str(self.capture / 'settings')
        Node(raw).field('bridge').mapping()['sha256'] = digest(self.root / BRIDGE)
        raw['source_sha256'] = {'scripts/producer.gd': digest(self.root / 'scripts/producer.gd')}
        report = self.capture / 'run/stall-report.json'
        report.write_text(json.dumps(raw))
        (self.capture / 'command.log').write_text(json.dumps(self.argv) + '\nMUX2_STALL_DONE complete=true pass=' + ('true' if strict else 'false') + ' errors=[] evidence=' + str(report) + '\n')
        inputs = snapshot(self.root)
        self.receipt = {'schema': 1, 'kind': 'stall', 'source_head': self.sha, 'root': str(self.root),
                        'inputs': inputs, 'inputs_end': inputs, 'godot': str(self.godot),
                        'godot_sha256': digest(self.godot), 'started_utc': '2026-11-02T00:00:00+00:00',
                        'finished_utc': '2026-11-02T00:11:00+00:00', 'exit_code': 0 if strict else 1,
                        'complete': strict, 'artifacts': {}}
        if strict:
            self.receipt['artifacts'] = {str(p): digest(p) for p in self.capture.rglob('*') if p.is_file()}
        return self.seal()

    def seal(self) -> CaptureBinding:
        path = self.capture / 'receipt.json'
        path.write_text(json.dumps(self.receipt))
        return CaptureBinding(self.sha, self.argv, 0 if self.receipt['complete'] is True else 1, digest(path))

    def test_complete_authentication_preserves_strict_and_failed_receipts(self) -> None:
        binding = self.prepare()
        evidence = authenticate_capture(self.root, self.capture, binding)
        self.assertFalse(evidence.publication_authority)
        self.assertEqual(evidence.original_exit, 1)
        self.assertFalse(evidence.original_complete)
        self.assertEqual(evidence.verdict.status, 'WAIVER_ELIGIBLE')

    def test_strict_capture_requires_sealed_artifacts(self) -> None:
        binding = self.prepare(True)
        self.assertTrue(authenticate_capture(self.root, self.capture, binding).verdict.strict_pass)
        self.receipt['artifacts'] = {}
        with self.assertRaises(EvidenceError):
            authenticate_capture(self.root, self.capture, self.seal())

    def test_process_receipt_and_source_binding_reject_mismatch(self) -> None:
        binding = self.prepare()
        for changed in (replace(binding, process_exit=0), replace(binding, process_exit=2),
                        replace(binding, process_exit=True), replace(binding, source_sha='f' * 40),
                        replace(binding, receipt_sha256='f' * 64), replace(binding, argv=('--headless',))):
            with self.subTest(binding=changed), self.assertRaises(EvidenceError):
                authenticate_capture(self.root, self.capture, changed)

    def test_changed_input_rejected_even_with_refreshed_receipt_hash(self) -> None:
        self.prepare()
        self.receipt['inputs_end'] = {'wrong': 'f' * 64}
        with self.assertRaises(EvidenceError):
            authenticate_capture(self.root, self.capture, self.seal())

    def test_missing_settings_or_script_error_cannot_be_waived(self) -> None:
        binding = self.prepare()
        command = self.capture / 'command.log'
        original = command.read_text()
        command.write_text(original + 'SCRIPT ERROR: fake\n')
        with self.assertRaises(EvidenceError):
            authenticate_capture(self.root, self.capture, binding)
        command.write_text(original)
        (self.capture / 'settings/stall-1-0.5.cfg').unlink()
        with self.assertRaises(EvidenceError):
            authenticate_capture(self.root, self.capture, binding)


if __name__ == '__main__':
    unittest.main()
