# test_gate_play_root.py: tools/v8/gate.sh refuses to start without a gate-owned PLAY_ROOT.
# How to run: python3 -m unittest tools/v8/test_gate_play_root.py (no cargo, Godot or lock: the guard runs first).
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OWNER = "/Users/rexxa/work/new-world-play"


def run_gate(play_root: str | None, out: Path) -> subprocess.CompletedProcess[str]:
    env = {k: v for k, v in os.environ.items() if k not in ("PLAY_ROOT", "WORLDSIM_MUX2_LANE_FD")}
    if play_root is not None:
        env["PLAY_ROOT"] = play_root
    return subprocess.run(["bash", str(ROOT / "tools/v8/gate.sh"), str(out)], cwd=ROOT, env=env,
                          capture_output=True, text=True, timeout=60)


class GatePlayRootGuard(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.out = Path(self.tmp.name) / "gate-out"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def assert_refused(self, play_root: str | None, words: str) -> None:
        result = run_gate(play_root, self.out)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn(words, result.stderr)
        self.assertFalse(self.out.exists(), "the guard must stop before the gate writes anything")

    def test_unset_play_root_refused(self) -> None:
        self.assert_refused(None, "PLAY_ROOT is not set")

    def test_empty_play_root_refused(self) -> None:
        self.assert_refused("", "PLAY_ROOT is not set")

    def test_owner_play_checkouts_refused(self) -> None:
        for root in (OWNER, OWNER + "/", OWNER + "2-dev", OWNER + "/main"):
            with self.subTest(root=root):
                self.assert_refused(root, "owner's play checkouts")

    def test_symlink_into_owner_checkouts_refused(self) -> None:
        if not Path(OWNER).is_dir():
            self.skipTest("owner play directory absent on this machine")
        link = Path(self.tmp.name) / "innocent"
        link.symlink_to(OWNER)
        self.assert_refused(str(link), "owner's play checkouts")


if __name__ == "__main__":
    unittest.main()
