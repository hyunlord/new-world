"""Stage minus1 comparison rejects undeclared gameplay differences."""
import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

BASE = "fbc03bddb8f5b68f43fba1b2dad283982e5cf436"
SCRIPT = Path(__file__).with_name("read_normalise.py")


def colony():
    return {"scene": "colony", "days": 2, "seeds": [
        {"seed": 0, "days": 2, "alive_end": 10, "m4": {"lost": []}},
    ]}


class SummaryContractTest(unittest.TestCase):
    def compare(self, base, head, *, historical=False, raw_base=None):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "base.json").write_text(raw_base if raw_base is not None else json.dumps(base))
            (root / "head.json").write_text(json.dumps(head))
            env = dict(os.environ)
            env.pop("READ_GATE_CONFIG", None)
            if historical:
                config = root / "config.json"
                config.write_text(json.dumps({"base_commit": BASE}))
                env["READ_GATE_CONFIG"] = str(config)
            return subprocess.run(
                [sys.executable, str(SCRIPT), str(root / "base.json"), str(root / "head.json")],
                env=env, text=True, capture_output=True, check=False,
            )

    def test_unsupported_recursive_names_stay_comparable(self):
        for key in ("why_not", "timing", "git_sha", "diagnosis_cache", "logs_ground_by_day"):
            with self.subTest(key=key):
                self.assertNotEqual(self.compare({"nested": {key: 1}}, {"nested": {key: 2}}).returncode, 0)

    def test_boolean_does_not_equal_integer(self):
        self.assertNotEqual(self.compare({"alive": True}, {"alive": 1}).returncode, 0)

    def test_duplicate_keys_are_rejected(self):
        self.assertNotEqual(self.compare(None, {"x": 2}, raw_base='{"x":1,"x":2}').returncode, 0)

    def test_nonfinite_numbers_are_rejected(self):
        for value in (float("nan"), float("inf"), float("-inf")):
            with self.subTest(value=value):
                self.assertNotEqual(self.compare({"x": value}, {"x": value}).returncode, 0)

    def test_historical_candidate_only_observers_require_explicit_base(self):
        base = colony()
        head = copy.deepcopy(base)
        head["seeds"][0].update(logs_ground_by_day=[12, 8], cap_target_by_day=[None, None])
        self.assertNotEqual(self.compare(base, head).returncode, 0)
        allowed = self.compare(base, head, historical=True)
        self.assertEqual(allowed.returncode, 0, allowed.stdout + allowed.stderr)

    def test_shared_observer_values_cannot_be_excluded(self):
        base = colony()
        base["seeds"][0].update(logs_ground_by_day=[12, 8], cap_target_by_day=[None, None])
        head = copy.deepcopy(base)
        head["seeds"][0]["logs_ground_by_day"][1] = 9
        self.assertNotEqual(self.compare(base, head, historical=True).returncode, 0)

    def test_historical_observer_requires_complete_valid_days(self):
        for values in ([1], [1, True], [1, -1]):
            head = colony()
            head["seeds"][0].update(logs_ground_by_day=values, cap_target_by_day=[None, None])
            self.assertNotEqual(self.compare(colony(), head, historical=True).returncode, 0)

    def test_historical_observer_does_not_hide_cap_activation(self):
        head = colony()
        head["seeds"][0].update(logs_ground_by_day=[1, 2], cap_target_by_day=[None, {"target": 3, "building_need": 1, "pending_shelter": 2}])
        self.assertNotEqual(self.compare(colony(), head, historical=True).returncode, 0)

    def test_missing_null_empty_and_nested_changes_fail(self):
        for left, right in (({}, {"x": None}), ({"x": []}, {"x": {}}),
                            ({"m4": {"lost": [1]}}, {"m4": {"lost": [2]}})):
            self.assertNotEqual(self.compare(left, right).returncode, 0)

    def test_baseline_timing_leaves_only(self):
        base = {"schema_version": 13, "runs": [{"seed": 0, "wall_seconds": 1.0, "ms_per_agent_tick": 2.0}],
                "timing": {key: {stat: 1.0 for stat in ("median", "p10", "p90", "min", "max")}
                           for key in ("wall_seconds", "ms_per_agent_tick")}}
        head = copy.deepcopy(base)
        head["runs"][0]["wall_seconds"] = 99.0
        head["timing"]["wall_seconds"]["max"] = 99.0
        self.assertEqual(self.compare(base, head).returncode, 0)
        for changed in ("missing", "bool", "extra"):
            invalid = copy.deepcopy(head)
            if changed == "missing":
                del invalid["timing"]["wall_seconds"]["max"]
            elif changed == "bool":
                invalid["runs"][0]["wall_seconds"] = True
            else:
                invalid["timing"]["gameplay"] = 1
            self.assertNotEqual(self.compare(base, invalid).returncode, 0)

    def test_historical_duplicate_seeds_rejected(self):
        base = colony()
        base["seeds"] *= 2
        head = copy.deepcopy(base)
        for row in head["seeds"]:
            row.update(logs_ground_by_day=[1, 2], cap_target_by_day=[None, None])
        self.assertNotEqual(self.compare(base, head, historical=True).returncode, 0)


if __name__ == "__main__":
    unittest.main()
