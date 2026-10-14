#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_run_perf_matrix_p9.py
"""Resume tests use a temporary runner tree and fake processes, never Godot or locks."""
from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

from test_summarize_perf_p9 import fixture

SOURCE = Path(__file__).resolve().parent
WORLDS = ("W0", "W500", "W1", "W200", "W1000", "W2000")


class MatrixResumeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="p9-resume-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.tools = self.root / "tools/art_pipeline"
        self.tools.mkdir(parents=True)
        for name in ("run_perf_matrix_p9.sh", "summarize_perf_p9.py"):
            shutil.copy2(SOURCE / name, self.tools / name)
        self.evidence = self.root / "evidence"
        self.evidence.mkdir()
        self.fixtures = self.root / "fixtures"
        self.fixtures.mkdir()
        for world in WORLDS:
            (self.fixtures / f"{world}.json").write_text(json.dumps(fixture(world)))
            self.accepted(world)
        self.launches = self.root / "launches.txt"
        self.fake = self.root / "fake-godot"
        self.fake.write_text("""#!/usr/bin/env python3
import os, pathlib, shutil, sys
args = sys.argv
world = args[args.index('--world') + 1]
output = pathlib.Path(args[args.index('--out') + 1])
with pathlib.Path(os.environ['MOCK_LAUNCHES']).open('a') as stream:
    stream.write(output.name + '\\n')
shutil.copyfile(pathlib.Path(os.environ['MOCK_FIXTURES']) / (world + '.json'), output)
print('synthetic session complete')
sys.exit(int(os.environ.get('MOCK_EXIT', '0')))
""")
        self.fake.chmod(0o755)
        (self.tools / "with_quiet_lock_p9.sh").write_text('#!/usr/bin/env bash\nshift 2\nexec "$@"\n')
        self.environment = dict(os.environ, GODOT_BIN=str(self.fake), MOCK_FIXTURES=str(self.fixtures), MOCK_LAUNCHES=str(self.launches))
        self.environment.pop("P9_DEADLINE_EPOCH", None)

    def path(self, world: str, attempt: int = 1, suffix: str = "json") -> Path:
        return self.evidence / f"baseline-{world}-attempt{attempt}.{suffix}"

    def accepted(self, world: str, attempt: int = 1) -> None:
        shutil.copyfile(self.fixtures / f"{world}.json", self.path(world, attempt))
        self.path(world, attempt, "log").write_text("synthetic complete\n")

    def run_matrix(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(["bash", str(self.tools / "run_perf_matrix_p9.sh"), "baseline", str(self.root), str(self.evidence)], env=self.environment, capture_output=True, text=True, timeout=30, check=False)

    def launched(self) -> list[str]:
        return self.launches.read_text().splitlines() if self.launches.exists() else []

    def test_completed_sessions_are_validated_without_relaunch_or_overwrite(self) -> None:
        original = {path.name: path.read_bytes() for path in self.evidence.iterdir()}
        result = self.run_matrix()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.launched(), [])
        self.assertEqual(original, {path.name: path.read_bytes() for path in self.evidence.iterdir()})
        self.assertIn("Resuming with accepted evidence", result.stdout)

    def test_malformed_json_is_preserved_and_attempt_two_runs(self) -> None:
        self.path("W0").write_text('{"world":')
        result = self.run_matrix()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.path("W0").read_text(), '{"world":')
        self.assertEqual(self.launched(), ["baseline-W0-attempt2.json"])

    def test_error_log_is_rejected_even_with_complete_json(self) -> None:
        self.path("W0", suffix="log").write_text("SCRIPT ERROR: synthetic failure\n")
        result = self.run_matrix()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Rejected session with ERROR log", result.stderr)
        self.assertEqual(self.launched(), ["baseline-W0-attempt2.json"])

    def test_missing_log_and_failed_validation_use_remaining_attempts(self) -> None:
        self.path("W0", suffix="log").unlink()
        self.accepted("W0", 2)
        bad = fixture("W0")
        bad["restoration"] = {}
        self.path("W0", 2).write_text(json.dumps(bad))
        result = self.run_matrix()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.launched(), ["baseline-W0-attempt3.json"])
        self.assertIn("Unproven runtime restoration", result.stderr)

    def test_highest_attempt_is_not_backfilled_and_three_strikes_are_final(self) -> None:
        self.path("W0").unlink()
        self.path("W0", suffix="log").unlink()
        self.path("W0", 3, "log").write_text("interrupted\n")
        result = self.run_matrix()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(self.launched(), [])
        self.assertIn("Incomplete prior session", result.stderr)

    def test_nonzero_exit_receipts_prevent_future_acceptance(self) -> None:
        self.path("W0").unlink()
        self.path("W0", suffix="log").unlink()
        self.environment["MOCK_EXIT"] = "23"
        result = self.run_matrix()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(len(self.launched()), 3)
        for attempt in (1, 2, 3):
            self.assertEqual(self.path("W0", attempt, "exit").read_text(), "23\n")
        self.environment["MOCK_EXIT"] = "0"
        resumed = self.run_matrix()
        self.assertEqual(resumed.returncode, 1)
        self.assertEqual(len(self.launched()), 3)
        self.assertIn("Rejected nonzero session status", resumed.stderr)

    def test_wrong_world_is_rejected(self) -> None:
        shutil.copyfile(self.fixtures / "W500.json", self.path("W0"))
        result = self.run_matrix()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.launched(), ["baseline-W0-attempt2.json"])
        self.assertIn("mismatched world or phase", result.stderr)

    def test_invalid_flag_is_not_accepted(self) -> None:
        bad = fixture("W0")
        bad["valid"] = False
        self.path("W0").write_text(json.dumps(bad))
        result = self.run_matrix()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.launched(), ["baseline-W0-attempt2.json"])

    def test_deadline_still_reuses_completed_sessions(self) -> None:
        self.environment["P9_DEADLINE_EPOCH"] = "1"
        result = self.run_matrix()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.launched(), [])


if __name__ == "__main__":
    unittest.main()
