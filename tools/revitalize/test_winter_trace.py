"""Run: python3 -m unittest discover -s tools/revitalize -p test_winter_trace.py."""
import tempfile
import unittest
from pathlib import Path

from test_evaluate_pass6 import OFFSET, TPD, write_round
from winter_trace import first_winter, main, pick_seeds, render


def dark_nights(seed, rows):
    """Seed 1: also dark on the second-to-last winter evening; seed 2: never dark."""
    for r in rows:
        day = (r["tick"] + OFFSET) // TPD
        minute = r["minute_of_day"]
        evening = day - 1 if minute < 420 else day
        if seed == 2:
            r["fire_lit"], r["cold_members"] = 1, 0
        if seed == 1 and evening % 28 == 26 and (minute >= 1260 or minute < 420):
            r["fire_lit"], r["cold_members"] = 0, 12


class WinterTraceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_picks_most_and_fewest_with_ties_to_the_lower_seed(self) -> None:
        winters = [{"seed": 3, "nights_out": 2}, {"seed": 1, "nights_out": 2}, {"seed": 2, "nights_out": 0},
                   {"seed": 0, "nights_out": 0}, {"seed": 1, "nights_out": 0}]
        self.assertEqual(pick_seeds(winters), (1, 0))

    def test_trace_tells_both_winters(self) -> None:
        root = write_round(self.root / "r", 4, 120, True, dark_nights)
        text = render(root, 39.0)
        self.maxDiff = 200
        self.assertIn("## Seed 1 — most winter nights out (8 over the run, 2 in the first winter)", text[:3000])
        self.assertIn("## Seed 2 — fewest winter nights out (0 over the run, 0 in the first winter)", text)
        self.assertIn("The wood ran out on the night of display day 27: the fire died at 22:00", text[:3000])
        self.assertIn("dark nights (fire out at 02:00) fell on display day 27, display day 28.", text)
        self.assertIn("The fire stayed lit through every winter night.", text)
        # Display days 22-28 (12 samples a day) plus the last night up to 07:00 (00:00..06:00, 4 samples).
        from evaluate_pass6 import load_round
        seed = load_round(root, 120).seeds[1]
        self.assertEqual(len(first_winter(seed, 7)), 7 * 12 + 4)
        for paragraph in [line for line in text.splitlines() if line.startswith("Seed ")]:
            self.assertEqual(paragraph.count(". ") + 1, 5, paragraph)

    def test_cli_rejects_rounds_without_seasons(self) -> None:
        def no_seasons(seed, rows):
            for r in rows:
                r["days_per_season"] = 0
        root = write_round(self.root / "flat", 2, 120, True, no_seasons)
        self.assertEqual(main(["--round120", str(root), "--out", str(self.root / "t.md")]), 2)
        root = write_round(self.root / "ok", 3, 120, True, dark_nights)
        self.assertEqual(main(["--round120", str(root), "--out", str(self.root / "t.md")]), 0)
        self.assertTrue((self.root / "t.md").read_text().startswith("# A winter in detail"))


if __name__ == "__main__":
    unittest.main()
