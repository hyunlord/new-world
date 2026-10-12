"""v0 bit-identity (GOAL_1_2.md 4.1): the feature table, targets and harness numbers of the committed sample equal
the golden values computed at the merge commit b20a991f, before the schema-4 extension of `data.py`.

Run: python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'
"""
import json
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
        golden = json.loads(golden_v0.TARGET.read_text())
        self.assertEqual(golden_v0.compute_more(), golden["harness_more"])


if __name__ == "__main__":
    unittest.main()
