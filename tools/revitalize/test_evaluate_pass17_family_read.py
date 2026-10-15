from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from evaluate_pass17_data import CohortSpec, InputError
from evaluate_pass17_read import load_cohort
from test_evaluate_pass17_family_fixtures import write_family_cohort
import evaluate_pass17_evidence as evidence
import evaluate_pass17_stage_b as stage_b
import evaluate_pass17_report as report


class FamilyInputTests(unittest.TestCase):
    def test_complete_trace_preserves_day60_day20_fallback_and_fission_denominators(self) -> None:
        from evaluate_pass17_family_read import load_families
        with tempfile.TemporaryDirectory() as temporary:
            root = write_family_cohort(Path(temporary) / "v22")
            cohort = load_cohort(root, CohortSpec("game_v22", (0, 1, 2), 60))
            worlds = load_families(cohort)
            self.assertEqual(len(worlds), 3)
            self.assertEqual(worlds[0].day60.tick, 86399)
            self.assertEqual(len(worlds[0].snapshots), 1440)
            self.assertEqual([(b.day, b.fallback) for b in worlds[0].births], [(19, True), (20, False)])
            self.assertEqual(worlds[0].fissions[0].couples, 1)

    def test_missing_or_inconsistent_family_data_is_error(self) -> None:
        from evaluate_pass17_family_read import load_families
        edits = (
            ("family_meta.json", lambda rows: rows.pop("day60")),
            ("family_meta.json", lambda rows: rows["day60"].update(tick=86400)),
            ("family_meta.json", lambda rows: rows.update(observation_misses=1)),
            ("family_meta.json", lambda rows: rows.pop("births_total")),
            ("family_meta.json", lambda rows: rows.update(source_departures_total=2)),
            ("family_meta.json", lambda rows: rows["births_by_day"][19].update(couple_births=0)),
            ("family_meta.json", lambda rows: rows["births_by_day"][19].update(couple_births=True)),
            ("family_hours.jsonl", lambda rows: rows.pop()),
            ("family_hours.jsonl", lambda rows: rows[0]["people"].pop()),
            ("family_events.jsonl", lambda rows: rows[1].pop("fallback")),
            ("family_events.jsonl", lambda rows: rows[1].update(simulation_day=19)),
            ("family_fissions.jsonl", lambda rows: rows[0].update(couples_split=1)),
            ("family_fissions.jsonl", lambda rows: rows.clear()),
        )
        for suffix, edit in edits:
            with self.subTest(suffix=suffix), tempfile.TemporaryDirectory() as temporary:
                root = write_family_cohort(Path(temporary) / "v22")
                cohort = load_cohort(root, CohortSpec("game_v22", (0, 1, 2), 60))
                path = root / f"game_v22_s0_{suffix}"
                rows = json.loads(path.read_text()) if suffix.endswith(".json") else [json.loads(line) for line in path.read_text().splitlines()]
                edit(rows)
                path.write_text(json.dumps(rows) if suffix.endswith(".json") else "".join(json.dumps(row) + "\n" for row in rows))
                with self.assertRaises(InputError):
                    load_families(cohort)

    def test_stage_b_report_contains_all_rules_numeric_failures_and_family_provenance(self) -> None:
        from evaluate_pass17_family_read import load_families
        from evaluate_pass17_family_report import build_family_report
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            candidate = load_cohort(write_family_cohort(root / "v22"), CohortSpec("game_v22", (0, 1, 2), 60))
            baseline = load_cohort(write_family_cohort(root / "v21", "game_v21"), CohortSpec("game_v21", (0, 1, 2), 60))
            families = load_families(candidate)
            measured = stage_b.evaluate(candidate, baseline, families)
            pending = evidence.load_evidence(None, "identity", True, "b")
            result = build_family_report(candidate, baseline, measured, families, pending, pending, True)
            report.write_report(result, root / "out.json", root / "out.md")
            self.assertEqual(set(result["rules"]), {f"F{i}" for i in range(1, 9)})
            self.assertEqual(result["stage"], "b")
            self.assertEqual(result["verdict"], "FAIL")
            self.assertEqual(result["rules"]["F1"], "FAIL")
            self.assertEqual(result["metrics"]["F1"]["value"], 1 / 3)
            self.assertEqual(result["metrics"]["F2"]["value"], 1)
            self.assertEqual(result["metrics"]["F4"]["value"], 1)
            self.assertEqual(result["metrics"]["F5"]["value"], 0)
            self.assertEqual(result["rules"]["F6"], "PASS")
            self.assertEqual(result["rules"]["F7"], "PENDING")
            self.assertEqual(len(result["family"]["births_by_day"]), 60)
            self.assertEqual(len(result["family"]["inputs_sha256"]), 15)
            self.assertIn("Stage B", (root / "out.md").read_text())
            self.assertTrue(all(value["value"] is not None for value in result["metrics"].values()))


if __name__ == "__main__":
    unittest.main()
