# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 test_extra_activity_p11.py
"""No-launch validation for the one-case supplemental capture wrapper."""
import unittest
from run_extra_activity_p11 import parse_case, InvalidInput


class ExtraCaptureTests(unittest.TestCase):
    def test_two_base_worlds_and_two_hours(self) -> None:
        for seed, day in [(20261010, 30), (20261018, 60)]:
            for hour in [11, 14]:
                for kind in range(7):
                    case = parse_case([str(seed), str(day), str(hour), str(kind)], 20261018)
                    self.assertEqual(case.capture.zoom, 4)
                    self.assertEqual(case.center_kind, kind)

    def test_other_worlds_hours_and_kinds_rejected(self) -> None:
        for values in [['20261010', '60', '11', '0'], ['20261018', '30', '11', '0'],
                       ['20261019', '60', '11', '0'], ['20261010', '30', '10', '0'],
                       ['20261010', '30', '14', '7'], ['20261010', '30', '14', '-1']]:
            with self.assertRaises(InvalidInput):
                parse_case(values, 20261018)


if __name__ == '__main__':
    unittest.main()
