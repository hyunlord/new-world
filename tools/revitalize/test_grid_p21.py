"""Pass 21: the grid's pre-registered order and selection rule (brief 7.2)."""
from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from grid_p21 import CONFIGS, qualifies, select  # noqa: E402


def report(index: int, *, estranged: float | None = 0.1, quarrel: float = 0.5, estrangements: int = 4,
           reconciled: float | None = 0.75, delivering: float = 0.5, worlds: int = 2) -> dict:
    r = {"label": f"G{index}", "index": index, "estranged_share_at_end": estranged, "share_quarrel": quarrel,
         "estrangements": estrangements, "reconciliations": round((reconciled or 0) * estrangements),
         "reconciled_share": reconciled, "share_delivering": delivering,
         "worlds": [{"multi_village": True, "delivered_trips": 1}] * worlds + [{"multi_village": True, "delivered_trips": 0}]}
    r["qualifies"], r["why_not"] = qualifies(r)
    return r


class GridTest(unittest.TestCase):
    def test_configurations_follow_the_brief_order(self) -> None:
        self.assertEqual(len(CONFIGS), 12)
        self.assertEqual(CONFIGS[0], (40, 30, 3000, 2000))
        self.assertEqual(CONFIGS[1], (40, 30, 2500, 2000))
        self.assertEqual(CONFIGS[2], (40, 30, 3000, 2500))
        self.assertEqual(CONFIGS[3], (40, 20, 3000, 2000))
        self.assertEqual(CONFIGS[6], (60, 30, 3000, 2000))
        self.assertEqual(CONFIGS[11], (60, 20, 3000, 2500))

    def test_bars(self) -> None:
        self.assertTrue(report(1)["qualifies"])
        self.assertFalse(report(1, estranged=0.26)["qualifies"])
        self.assertTrue(report(1, estranged=0.25)["qualifies"])
        self.assertFalse(report(1, quarrel=0.14)["qualifies"])
        self.assertFalse(report(1, reconciled=0.25)["qualifies"])
        # Fewer than three estrangements: the reconciliation bar does not apply.
        self.assertTrue(report(1, estrangements=2, reconciled=0.0)["qualifies"])

    def test_highest_delivering_share_ties_to_the_earlier(self) -> None:
        rs = [report(1, delivering=0.5), report(2, delivering=0.7), report(3, delivering=0.7), report(4, delivering=0.9, estranged=0.3)]
        self.assertEqual(select(rs)["label"], "G2")
        self.assertFalse(select(rs)["fallback"])

    def test_fallback_most_delivering_worlds_within_half_estranged(self) -> None:
        rs = [report(1, estranged=0.4, worlds=2), report(2, estranged=0.6, worlds=5), report(3, estranged=0.45, worlds=3),
              report(4, estranged=0.5, worlds=3)]
        choice = select(rs)
        self.assertTrue(choice["fallback"])
        self.assertEqual(choice["label"], "G3")


if __name__ == "__main__":
    unittest.main()
