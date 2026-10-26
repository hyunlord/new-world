"""Synthetic source/binary binding controls; never builds or executes a simulation."""
from __future__ import annotations
import ast
import json
import os
import subprocess
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace
import core_shadow_freeze as freeze


class FreezeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.repo = self.root / "repo"
        self.repo.mkdir()
        for directory in freeze.PROVENANCE_DIRS:
            path = self.repo / directory / "fixture.rs"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("synthetic source")
        for name in freeze.REQUIRED_FILES:
            path = self.repo / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("synthetic required source")
        self.legacy = self.repo / "rust/crates/sim-systems/src/runtime/decision/system.rs"
        self.legacy.parent.mkdir(parents=True, exist_ok=True)
        self.legacy.write_text("synthetic old colony arms")
        self.seam = self.repo / "rust/crates/sim-systems/src/runtime/jobs/seam.rs"
        self.seam.parent.mkdir(parents=True, exist_ok=True)
        self.seam.write_text("synthetic old seam")
        self.shadow = self.root / "frozen-shadow"
        self.control = self.root / "frozen-control"
        self.shadow.write_text("synthetic binary hash fixture; never executable")
        self.control.write_text("synthetic feature-OFF hash fixture; never executable")
        self.snapshot = self.root / "snapshot.json"
        self.receipt = self.root / "receipt.json"
        self.manifest = self.root / "manifest.json"
        freeze.prepare(self.repo, self.snapshot)
        self.write_receipt()

    def write_receipt(self) -> None:
        self.receipt.write_text(json.dumps({"schema_version": 1, "snapshot_sha256": freeze.digest(self.snapshot),
            "shadow": {"sha256": freeze.digest(self.shadow), "exit_code": 0, "features": ["mind_shadow"], "command": "synthetic builder attestation"},
            "control": {"sha256": freeze.digest(self.control), "exit_code": 0, "features": [], "command": "synthetic builder attestation"}}))

    def seal(self) -> str:
        freeze.seal(self.repo, self.snapshot, freeze.digest(self.snapshot), self.receipt, self.shadow, self.control, self.manifest)
        return freeze.digest(self.manifest)

    def test_exact_sources_and_binaries_pass_after_permitted_legacy_deletion(self) -> None:
        pin = self.seal()
        self.legacy.unlink()
        self.seam.unlink()
        routing = self.repo / "rust/crates/sim-systems/src/lib.rs"
        routing.write_text("new colony routing")
        freeze.check(self.repo, self.manifest, pin)
        data = freeze.load(self.manifest)
        self.assertIn(str(self.legacy.relative_to(self.repo)), data["frozen_files"])

    def test_changed_added_removed_current_sources_are_rejected(self) -> None:
        for change in ("edit", "add", "remove"):
            with self.subTest(change=change):
                path = self.repo / freeze.CURRENT_DIRS[0] / "fixture.rs"
                original = path.read_text()
                pin = self.seal() if not self.manifest.exists() else freeze.digest(self.manifest)
                extra = path.with_name("new.rs")
                if change == "edit": path.write_text("different")
                if change == "add": extra.write_text("new semantic code")
                if change == "remove": path.unlink()
                with self.assertRaises(freeze.FreezeError): freeze.check(self.repo, self.manifest, pin)
                path.write_text(original)
                if extra.exists(): extra.unlink()

    def test_binary_receipt_and_manifest_tampering_fail(self) -> None:
        pin = self.seal()
        for path in (self.shadow, self.control, self.receipt, self.manifest):
            original = path.read_bytes()
            path.write_bytes(original + b"tampered")
            with self.subTest(path=path), self.assertRaises(freeze.FreezeError):
                freeze.check(self.repo, self.manifest, pin)
            path.write_bytes(original)

    def test_prepare_build_source_race_and_wrong_snapshot_pin_fail(self) -> None:
        with self.assertRaises(freeze.FreezeError):
            freeze.seal(self.repo, self.snapshot, "0" * 64, self.receipt, self.shadow, self.control, self.manifest)
        self.legacy.write_text("changed during builds")
        with self.assertRaises(freeze.FreezeError): self.seal()

    def test_missing_required_scope_cannot_be_frozen(self) -> None:
        (self.repo / freeze.REQUIRED_FILES[0]).unlink()
        with self.assertRaises(freeze.FreezeError): freeze.prepare(self.repo, self.root / "missing.json")

    def test_symlink_source_or_binary_is_rejected(self) -> None:
        link = self.repo / freeze.CURRENT_DIRS[0] / "escape.rs"
        link.symlink_to(self.shadow)
        with self.assertRaises(freeze.FreezeError): freeze.prepare(self.repo, self.root / "escape.json")
        link.unlink()
        linked_binary = self.root / "binary-link"
        linked_binary.symlink_to(self.shadow)
        with self.assertRaises(freeze.FreezeError):
            freeze.seal(self.repo, self.snapshot, freeze.digest(self.snapshot), self.receipt, linked_binary, self.control, self.manifest)

    def test_bool_exit_code_duplicate_keys_and_overwrite_are_rejected(self) -> None:
        data = json.loads(self.receipt.read_text())
        data["shadow"]["exit_code"] = False
        self.receipt.write_text(json.dumps(data))
        with self.assertRaises(freeze.FreezeError): self.seal()
        self.receipt.write_text('{"schema_version":1,"schema_version":1}')
        with self.assertRaises(freeze.FreezeError): self.seal()
        with self.assertRaises(freeze.FreezeError): freeze.prepare(self.repo, self.snapshot)

    def test_embedded_content_and_command_dependencies_are_bound(self) -> None:
        pin = self.seal()
        for name in ('rust/crates/sim-core/data/fixture.rs', 'rust/crates/sim-systems/src/commands.rs'):
            path = self.repo / name
            original = path.read_bytes()
            path.write_bytes(original + b'changed')
            with self.assertRaises(freeze.FreezeError):
                freeze.check(self.repo, self.manifest, pin)
            path.write_bytes(original)

    def test_fixture_gate_requires_pinned_config_before_any_execution(self) -> None:
        helper = Path(__file__).with_name('core_gate_checks.sh').resolve()
        environment = {key: value for key, value in os.environ.items()
                       if key not in ('CORE_SHADOW_CONFIG', 'CORE_SHADOW_CONFIG_SHA256')}
        output = self.root / 'not-created'
        command = ['bash', str(helper), 'fixture_run', str(output)]
        result = subprocess.run(command, env=environment, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('external frozen fixture config required', result.stderr)
        self.assertFalse(output.exists())
        config = self.root / 'config.json'
        config.write_text('{}')
        environment.update(CORE_SHADOW_CONFIG=str(config), CORE_SHADOW_CONFIG_SHA256='0' * 64)
        result = subprocess.run(command, env=environment, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('SHA-256 mismatch', result.stderr)
        self.assertFalse(output.exists())

    def test_hashing_ignores_read_access_time_but_rejects_content_metadata_changes(self) -> None:
        before = self.shadow.stat()
        fields = ('st_dev', 'st_ino', 'st_size', 'st_mtime_ns', 'st_ctime_ns', 'st_atime_ns')
        changed = {field: getattr(before, field) for field in fields}
        changed['st_atime_ns'] += 1
        with patch.object(freeze, 'canonical', side_effect=lambda path: path), patch.object(Path, 'stat', side_effect=[before, SimpleNamespace(**changed)]):
            self.assertEqual(len(freeze.digest(self.shadow)), 64)
        changed['st_mtime_ns'] += 1
        with patch.object(freeze, 'canonical', side_effect=lambda path: path), patch.object(Path, 'stat', side_effect=[before, SimpleNamespace(**changed)]):
            with self.assertRaises(freeze.FreezeError):
                freeze.digest(self.shadow)

    def test_evaluation_gate_requires_pin_before_reading_results(self) -> None:
        helper = Path(__file__).with_name('core_gate_checks.sh').resolve()
        environment = {key: value for key, value in os.environ.items()
                       if key not in ('CORE_EVAL_CONFIG', 'CORE_EVAL_CONFIG_SHA256')}
        output = self.root / 'evaluation-not-created'
        command = ['bash', str(helper), 'evaluation', str(output)]
        result = subprocess.run(command, env=environment, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('external held-out/performance config required', result.stderr)
        self.assertFalse(output.exists())
        config = self.root / 'eval-config.json'
        config.write_text('{}')
        environment.update(CORE_EVAL_CONFIG=str(config), CORE_EVAL_CONFIG_SHA256='0' * 64)
        result = subprocess.run(command, env=environment, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('SHA-256 mismatch', result.stderr)
        self.assertFalse(output.exists())

    def test_gate_reports_only_core_ratio_misses_and_rejects_other_failures(self) -> None:
        source = Path(__file__).with_name('core_gate_checks.sh').read_text()
        code = source.split("<<'PYEVAL'\n", 1)[1].split('\nPYEVAL', 1)[0]
        node = next(node for node in ast.parse(code).body
                    if isinstance(node, ast.FunctionDef) and node.name == 'performance_gate')
        namespace = {}
        exec(compile(ast.Module(body=[node], type_ignores=[]), '<gate-performance-mapping>', 'exec'), namespace)
        check = namespace['performance_gate']
        receipt = {'status': 'PASS', 'failures': [], 'saturation_companion': {'status': 'MEASURED'}}
        self.assertEqual(check(receipt, 0), [])
        misses = ['N1000 mind/old decision>0.5', 'N1000 tick ratio exceeds accepted arithmetic bar']
        missed = {**receipt, 'status': 'FAIL', 'failures': misses}
        self.assertEqual(check(missed, 1), misses)
        for bad, exit_code in [
            ({**missed, 'failures': misses + ['standing C3 rss_per_agent']}, 1),
            ({**missed, 'failures': ['n1000/tick_mean_us>1.1']}, 1),
            ({**missed, 'failures': ['unknown failure']}, 1),
            ({**missed, 'saturation_companion': {'status': 'PENDING'}}, 1),
            ({**receipt, 'status': 'PENDING'}, 75),
            ({**receipt, 'status': 'INVALID'}, 2),
            (missed, 0),
        ]:
            with self.subTest(receipt=bad), self.assertRaises(ValueError):
                check(bad, exit_code)

    def test_story_gate_binds_separate_base_instrument_and_current_mind(self) -> None:
        source = Path(__file__).with_name('core_gate_checks.sh').read_text()
        code = source.split("<<'PYEVAL'\n", 1)[1].split('\nPYEVAL', 1)[0]
        node = next(node for node in ast.parse(code).body
                    if isinstance(node, ast.FunctionDef) and node.name == 'story_binding')
        namespace = {}
        exec(compile(ast.Module(body=[node], type_ignores=[]), '<gate-story-binding>', 'exec'), namespace)
        check = namespace['story_binding']
        mind = {'binary_sha256': 'mind-binary', 'source_sha': 'measured-head', 'policy_sha256': 'policy'}
        base = {'binary_sha256': 'instrument-binary', 'base_instrument_provenance_sha256': 'provenance', 'runtime_base_sha': 'nominated-base'}
        args = ('mind-binary', 'instrument-binary', 'measured-head', 'policy', 'provenance', 'nominated-base')
        self.assertIsNone(check(mind, base, *args))
        for original in (mind, base):
            for field in original:
                with self.subTest(field=field):
                    old = original[field]
                    original[field] = 'unrelated'
                    with self.assertRaises(ValueError):
                        check(mind, base, *args)
                    original[field] = old
        for required in ('story_base_report', 'story_base_receipt', 'story_base_binary',
                         'story_base_provenance', 'story_mind_report', 'story_mind_receipt'):
            self.assertIn(required, code.split('if set(config) != required:', 1)[0])

    def test_removing_manifest_source_entries_even_with_new_pin_is_rejected(self) -> None:
        self.seal()
        data = json.loads(self.manifest.read_text())
        data["current_files"].pop(next(iter(data["current_files"])))
        self.manifest.write_text(json.dumps(data))
        with self.assertRaises(freeze.FreezeError):
            freeze.check(self.repo, self.manifest, freeze.digest(self.manifest))


if __name__ == "__main__":
    unittest.main()
