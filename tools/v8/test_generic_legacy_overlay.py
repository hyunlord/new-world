"""Synthetic Git exports exercise legacy overlay admission without a real world."""

import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import generic_legacy_overlay as overlay


class LegacyOverlayTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "source"
        self.repo.mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "Fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        (self.repo / "project.godot").write_text('[application]\nrun/main_scene="res://scenes/main.tscn"\n[autoload]\nLocale="*res://locale.gd"\n')
        (self.repo / "sim_bridge.gdextension").write_text('[libraries]\nmacos.debug="res://rust/target/debug/libsim_bridge.dylib"\n')
        (self.repo / "locale.gd").write_text("extends Node\n")
        (self.repo / "scenes").mkdir()
        (self.repo / "scenes/main.tscn").write_text('[gd_scene format=3]\n[node name="WorldSim" type="WorldSimNode"]\n')
        self.git("add", "project.godot", "sim_bridge.gdextension", "locale.gd", "scenes/main.tscn")
        self.git("commit", "-qm", "Freeze synthetic source")
        self.commit = self.git("rev-parse", "HEAD")
        self.tree = self.git("rev-parse", "HEAD^{tree}")
        self.binary = self.root / "bridge.dylib"
        self.binary.write_bytes(b"synthetic native binary - never executed")
        self.observer = self.root / "observer.gd"
        self.observer.write_text("extends Node\n")
        self.receipt = self.root / "build.json"
        self.receipt.write_text(json.dumps({"schema_version": 1, "product_commit": self.commit, "product_tree": self.tree, "binary_sha256": overlay.sha256(self.binary), "build_status": "PASS"}))
        self.policy = patch.dict(overlay.SUPPORTED, {self.commit: "synthetic_preknowledge"})
        self.policy.start()
        self.addCleanup(self.policy.stop)

    def git(self, *args: str) -> str:
        return subprocess.run(["git", "-C", str(self.repo), *args], check=True, capture_output=True, text=True).stdout.strip()

    def materialize(self, destination: Path | None = None) -> Path:
        target = destination or self.root / "export"
        return overlay.materialize(overlay.Request(self.repo, self.commit, target, self.binary, self.receipt, overlay.sha256(self.receipt), self.observer))

    def test_exports_committed_source_and_keeps_binary_and_scene_exact(self) -> None:
        (self.repo / "locale.gd").write_text("dirty source must not leak")
        target = self.materialize()
        self.assertEqual((target / "locale.gd").read_text(), "extends Node\n")
        self.assertEqual((target / "scenes/main.tscn").read_bytes(), (self.repo / "scenes/main.tscn").read_bytes())
        self.assertEqual((target / "rust/target/debug/libsim_bridge.dylib").read_bytes(), self.binary.read_bytes())
        config = (target / "project.godot").read_text()
        self.assertIn('GateLegacyObserver="*res://gate_observer/legacy.gd"', config)
        receipt = json.loads((target / "gate_observer/provenance.json").read_text())
        self.assertEqual(receipt["runtime_status"], "NOT_RUN")
        self.assertEqual(receipt["launch_args"], ["--headless", "--path", str(target), "--quit-after", "300"])
        self.assertFalse(receipt["pristine_export"])

    def test_unknown_or_abbreviated_product_rejected(self) -> None:
        for commit in (self.commit[:8], "0" * 40):
            with self.subTest(commit=commit), self.assertRaises(overlay.OverlayError):
                overlay.materialize(overlay.Request(self.repo, commit, self.root / "rejected", self.binary, self.receipt, overlay.sha256(self.receipt), self.observer))
        self.assertFalse((self.root / "rejected").exists())

    def test_binary_or_receipt_tamper_rejected_before_export(self) -> None:
        digest = overlay.sha256(self.receipt)
        self.receipt.write_text(self.receipt.read_text() + "\n")
        with self.assertRaises(overlay.OverlayError):
            overlay.materialize(overlay.Request(self.repo, self.commit, self.root / "bad", self.binary, self.receipt, digest, self.observer))
        self.binary.write_bytes(b"changed")
        with self.assertRaises(overlay.OverlayError):
            self.materialize()
        self.assertFalse((self.root / "export").exists())

    def test_existing_or_registered_worktree_destination_rejected(self) -> None:
        existing = self.root / "existing"
        existing.mkdir()
        for target in (existing, self.repo / "nested-export"):
            with self.subTest(target=target), self.assertRaises(overlay.OverlayError):
                self.materialize(target)

    def test_overlay_marker_collision_rejected(self) -> None:
        with self.assertRaises(overlay.OverlayError):
            overlay.autoload_delta('[autoload]\nGateLegacyObserver="*res://old.gd"\n')

    def test_no_autoload_section_is_added_without_touching_main(self) -> None:
        source = '[application]\nrun/main_scene="res://scenes/main.tscn"\n'
        result = overlay.autoload_delta(source)
        self.assertTrue(result.startswith(source))
        self.assertEqual(result.count("[autoload]"), 1)

    def _repin_fixture(self) -> None:
        self.git("add", "-u")
        self.git("commit", "-qm", "Exercise rejected export source")
        self.commit = self.git("rev-parse", "HEAD")
        self.tree = self.git("rev-parse", "HEAD^{tree}")
        overlay.SUPPORTED[self.commit] = "synthetic_preknowledge"
        self.receipt.write_text(json.dumps({"schema_version": 1, "product_commit": self.commit, "product_tree": self.tree, "binary_sha256": overlay.sha256(self.binary), "build_status": "PASS"}))

    def test_archive_attributes_cannot_omit_committed_files(self) -> None:
        (self.repo / ".gitattributes").write_text("locale.gd export-ignore\n")
        self.git("add", ".gitattributes")
        self._repin_fixture()
        with self.assertRaisesRegex(overlay.OverlayError, "omitted or transformed"):
            self.materialize()
        self.assertFalse((self.root / "export").exists())

    def test_escaping_symlink_rejected_before_export(self) -> None:
        (self.repo / "escape").symlink_to("../../outside")
        self.git("add", "escape")
        self._repin_fixture()
        with self.assertRaisesRegex(overlay.OverlayError, "symlink leaves"):
            self.materialize()
        self.assertFalse((self.root / "export").exists())

    def test_wrong_build_tree_rejected(self) -> None:
        receipt = json.loads(self.receipt.read_text())
        receipt["product_tree"] = "0" * 40
        self.receipt.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(overlay.OverlayError, "product tree"):
            self.materialize()


if __name__ == "__main__":
    unittest.main()
