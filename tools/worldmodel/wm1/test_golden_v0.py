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


if __name__ == "__main__":
    unittest.main()
