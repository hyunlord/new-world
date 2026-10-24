"""Negative controls for MUX fixture coverage; run with unittest discovery."""
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
# How to run: python3 -m unittest tools.v8.test_mux_safety
from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.v8.mux_safety_validate import FIXTURES, SafetyError, validate_colonies, validate_identity


class MuxSafetyTests(unittest.TestCase):
    """Use small synthetic outputs to prove each coverage guard can fail."""

    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "base"
        self.root.mkdir()
        for fixture in FIXTURES:
            directory = self.root / "colony" / fixture.name
            directory.mkdir(parents=True)
            rows = [
                {"seed": seed, "ledger_violation_ticks": 0,
                 "reservation_violation_ticks": 0}
                for seed in range(fixture.count)
            ]
            document = {"scene": "colony", "days": fixture.days, "seeds": rows}
            (directory / "summary.json").write_text(json.dumps(document), encoding="utf-8")

    def target(self) -> Path:
        return self.root / "colony" / "m55_pressure" / "summary.json"

    def test_all_nine_exact_seed_sets_are_required(self) -> None:
        validate_colonies(self.root)
        self.target().unlink()
        with self.assertRaises(SafetyError):
            validate_colonies(self.root)

    def test_empty_seed_set_is_rejected(self) -> None:
        self.target().write_text('{"scene":"colony","days":48,"seeds":[]}', encoding="utf-8")
        with self.assertRaises(SafetyError):
            validate_colonies(self.root)

    def test_duplicate_or_reordered_seed_is_rejected(self) -> None:
        original = self.target().read_text(encoding="utf-8")
        for replacement in ('"seed": 0', '"seed": 8'):
            with self.subTest(replacement=replacement):
                self.target().write_text(original.replace('"seed": 1', replacement), encoding="utf-8")
                with self.assertRaises(SafetyError):
                    validate_colonies(self.root)

    def test_wrong_days_are_rejected(self) -> None:
        self.target().write_text(self.target().read_text().replace('"days": 48', '"days": 24'))
        with self.assertRaises(SafetyError):
            validate_colonies(self.root)

    def test_boolean_seed_is_not_integer_zero(self) -> None:
        self.target().write_text(self.target().read_text().replace('"seed": 0', '"seed": false'))
        with self.assertRaises(SafetyError):
            validate_colonies(self.root)

    def test_each_invariant_is_checked(self) -> None:
        original = self.target().read_text()
        for field in ("ledger_violation_ticks", "reservation_violation_ticks"):
            with self.subTest(field=field):
                self.target().write_text(original.replace(f'"{field}": 0', f'"{field}": 1'))
                with self.assertRaises(SafetyError):
                    validate_colonies(self.root)

    def test_missing_invariant_is_rejected(self) -> None:
        self.target().write_text(self.target().read_text().replace('"ledger_violation_ticks": 0, ', ''))
        with self.assertRaises(SafetyError):
            validate_colonies(self.root)

    def test_malformed_json_is_rejected(self) -> None:
        self.target().write_text('{"scene":')
        with self.assertRaises(SafetyError):
            validate_colonies(self.root)

    def test_duplicate_json_key_is_rejected(self) -> None:
        original = self.target().read_text()
        self.target().write_text(original.replace('"days": 48', '"days": 24, "days": 48'))
        with self.assertRaises(SafetyError):
            validate_colonies(self.root)

    def test_empty_identity_capture_is_rejected(self) -> None:
        with self.assertRaises(SafetyError):
            validate_identity(self.root)

    def make_identity(self) -> None:
        probe = self.root / "identity" / "probe"
        chronicle = self.root / "identity" / "chronicle"
        probe.mkdir(parents=True)
        chronicle.mkdir()
        for seed in range(32):
            for suffix in ("", "_deaths"):
                (probe / f"game_v26_s{seed}{suffix}.jsonl").write_text("{}\n")
            for kind in ("chronicle", "relations"):
                (chronicle / f"{kind}_{seed}.json").write_text("{}")
        document = {"scene": "game_v26", "seeds": {"start": 0, "end_exclusive": 32},
                    "days": {str(day): {} for day in range(1, 121)},
                    "runs": [{"seed": seed} for seed in range(32)]}
        (probe / "summary.json").write_text(json.dumps(document))

    def test_missing_raw_seed_cannot_pass_identity(self) -> None:
        self.make_identity()
        validate_identity(self.root)
        probe = self.root / "identity" / "probe"
        (probe / "game_v26_s17.jsonl").unlink()
        with self.assertRaises(SafetyError):
            validate_identity(self.root)

    def make_capture(self) -> None:
        self.make_identity()
        inputs = self.root / "inputs"
        inputs.mkdir()
        names = ("soak_script.json", "m5_u_script.json", "m55_u_script.json",
                 "m4_f10_script.json", "m55_u_nodraft_script.json")
        digest = hashlib.sha256(b"[]").hexdigest()
        for name in names:
            (inputs / name).write_text("[]")
        (self.root / "input_sha256.txt").write_text("".join(f"{digest}  {name}\n" for name in names))
        (self.root / "bin").mkdir()
        (self.root / "bin" / "worldsim_probe").write_bytes(b"test fixture")
        digest = hashlib.sha256(b"test fixture").hexdigest()
        (self.root / "probe_sha256.txt").write_text(f"{digest}  worldsim_probe\n")
        (self.root / "capture_complete.txt").write_text("complete\n")
        (self.root / "soak.log").write_text("DETERMINISM PASS\n")
        (self.root / "identity" / "determinism.txt").write_text("DETERMINISM PASS\n")

    def compare_capture(self, candidate: Path) -> subprocess.CompletedProcess[str]:
        script = Path(__file__).with_name("mux_safety.sh")
        return subprocess.run(["bash", str(script), "compare", str(self.root), str(candidate)],
                              check=False, capture_output=True, text=True)

    def test_literal_comparison_rejects_changed_value_or_whitespace(self) -> None:
        self.make_capture()
        candidate = Path(self.temp.name) / "candidate"
        shutil.copytree(self.root, candidate)
        result = self.compare_capture(candidate)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        target = candidate / "colony" / "m55_pressure" / "summary.json"
        original = target.read_text()
        for changed in (original + "\n", original.replace('"seed": 0', '"seed": 99')):
            with self.subTest(changed=changed[-20:]):
                target.write_text(changed)
                self.assertNotEqual(self.compare_capture(candidate).returncode, 0)

    def test_comparing_capture_to_itself_is_rejected(self) -> None:
        self.make_capture()
        self.assertNotEqual(self.compare_capture(self.root).returncode, 0)


if __name__ == "__main__":
    unittest.main()
