"""v0 bit-identity (GOAL_1_2.md 4.1): the feature table, targets and harness numbers of the committed sample equal
the golden values computed at the merge commit b20a991f, before the schema-4 extension of `data.py`.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
import math
import unittest

from wm1 import golden_v0


class GoldenV0Tests(unittest.TestCase):
    def test_v0_sample_is_bit_identical_to_the_golden_values(self) -> None:
        golden = json.loads(golden_v0.TARGET.read_text())
        now = golden_v0.compute()
        self.assertEqual(now["episodes"], golden["episodes"])
        self.assertEqual(now["variants"], golden["variants"])
        self.assertEqual(now["harness"], golden["harness"])

    def test_ridge_and_calendar_numbers_equal_the_merge_commit(self) -> None:
        # Review round 1: computed by `compute_more` from the merge commit's code (b20a991f) in a separate worktree.
        # The ridge's linear solve goes through the platform's BLAS (Accelerate on the Mac, OpenBLAS on the DGX), whose
        # last bits differ between machines; its numbers are compared to 1e-12 relative, every other value exactly.
        golden = json.loads(golden_v0.TARGET.read_text())["harness_more"]
        now = golden_v0.compute_more()
        self.assertEqual(set(now), set(golden))
        for k, v in golden.items():
            if k.startswith("ridge_nosev_T"):
                self.assertTrue(math.isclose(float(now[k]), float(v), rel_tol=1e-12, abs_tol=0.0), (k, now[k], v))
            else:
                self.assertEqual(now[k], v, k)


if __name__ == "__main__":
    unittest.main()
