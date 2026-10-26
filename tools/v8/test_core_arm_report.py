"""Counterexamples for CORE arm-report completeness, coverage, and integer typing."""
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_core_arm_report.py
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import core_arm_report as report


def fixture(spec: report.RunSpec) -> report.JSON:
    """Small synthetic observations, never represented as measured simulation data."""
    return {"scene": "colony", "days": spec.days, "inputs": spec.inputs(), "seeds": [
        {"seed": seed, "profile": {"schema_version": 1, "ticks": spec.days * 1440,
         "timing": report.TIMING, "scopes": [
             {"name": "a.evaluate", "calls": seed + 1, "inclusive_ns": 10 * (seed + 1)},
             {"name": "b.unhit", "calls": 0, "inclusive_ns": 0}]}}
        for seed in range(spec.count)]}


def write_fixture(directory: Path, spec: report.RunSpec) -> Path:
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / "perf_arms.json"
    path.write_text(json.dumps(fixture(spec)))
    (directory / "summary.json").write_text(json.dumps({"scene": "colony", "days": spec.days,
        "seeds": [{"seed": seed, "ledger_violation_ticks": 0, "reservation_violation_ticks": 0}
                  for seed in range(spec.count)]}))
    return path


class CoreArmReportTests(unittest.TestCase):
    def test_zero_scopes_remain_and_each_scope_is_summed_independently(self) -> None:
        # Given two worlds with overlapping parent/evaluation timing.
        spec = report.RunSpec("small", 2, 1, False)
        with tempfile.TemporaryDirectory() as directory:
            path = write_fixture(Path(directory), spec)
            # When the complete fixture is parsed.
            run = report.read_run(path, spec, ("a.evaluate", "b.unhit"))
            # Then counts/time are exact per scope and zero scopes remain explicit.
            self.assertEqual(run.scopes, (report.Scope("a.evaluate", 3, 30), report.Scope("b.unhit", 0, 0)))

    def test_malformed_worlds_and_registry_drift_fail(self) -> None:
        # Given a valid small fixture, alter one independent coverage invariant.
        spec = report.RunSpec("small", 2, 1, False)
        cases = [
            ("missing_seed", lambda d: d["seeds"].pop()),
            ("duplicate_seed", lambda d: d["seeds"][1].update(seed=0)),
            ("wrong_seed", lambda d: d["seeds"][1].update(seed=7)),
            ("bool_seed", lambda d: d["seeds"][1].update(seed=True)),
            ("partial_ticks", lambda d: d["seeds"][0]["profile"].update(ticks=1439)),
            ("bool_ticks", lambda d: d["seeds"][0]["profile"].update(ticks=True)),
            ("bool_schema", lambda d: d["seeds"][0]["profile"].update(schema_version=True)),
            ("bool_days", lambda d: d.update(days=True)),
            ("negative_calls", lambda d: d["seeds"][0]["profile"]["scopes"][0].update(calls=-1)),
            ("bool_calls", lambda d: d["seeds"][0]["profile"]["scopes"][0].update(calls=True)),
            ("float_time", lambda d: d["seeds"][0]["profile"]["scopes"][0].update(inclusive_ns=1.5)),
            ("negative_time", lambda d: d["seeds"][0]["profile"]["scopes"][0].update(inclusive_ns=-1)),
            ("zero_calls_time", lambda d: d["seeds"][0]["profile"]["scopes"][1].update(inclusive_ns=1)),
            ("missing_zero_scope", lambda d: d["seeds"][0]["profile"]["scopes"].pop()),
            ("duplicate_scope", lambda d: d["seeds"][0]["profile"]["scopes"].append(d["seeds"][0]["profile"]["scopes"][0])),
            ("extra_scope", lambda d: d["seeds"][0]["profile"]["scopes"].append({"name":"extra", "calls":0,"inclusive_ns":0})),
            ("steward_mismatch", lambda d: d["inputs"].update(steward=True)),
            ("bool_input_integer", lambda d: d["inputs"].update(steward=0)),
            ("script_mismatch", lambda d: d["inputs"].update(commands="wrong.json")),
            ("scene_mismatch", lambda d: d.update(scene="baseline")),
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = write_fixture(Path(directory), spec)
            for name, change in cases:
                with self.subTest(case=name):
                    data = json.loads(json.dumps(fixture(spec)))
                    change(data)
                    path.write_text(json.dumps(data))
                    # When/Then: malformed observations never produce a valid run.
                    with self.assertRaises(ValueError):
                        report.read_run(path, spec, ("a.evaluate", "b.unhit"))

    def test_missing_completion_summary_fails(self) -> None:
        # Given an arm export but absent terminal summary.
        spec = report.RunSpec("small", 2, 1, False)
        with tempfile.TemporaryDirectory() as directory:
            path = write_fixture(Path(directory), spec)
            (path.parent / "summary.json").unlink()
            # When/Then: the partial output cannot be called complete.
            with self.assertRaises(OSError):
                report.read_run(path, spec, ("a.evaluate", "b.unhit"))

    def test_summary_corruption_and_conservation_fail(self) -> None:
        # Given a complete arm file paired with corrupt terminal evidence.
        spec = report.RunSpec("small", 2, 1, False)
        changes = [lambda d: d["seeds"].pop(), lambda d: d["seeds"][1].update(seed=0),
                   lambda d: d["seeds"][0].update(ledger_violation_ticks=1),
                   lambda d: d["seeds"][0].update(reservation_violation_ticks=True),
                   lambda d: d.update(days=True)]
        with tempfile.TemporaryDirectory() as directory:
            for change in changes:
                path = write_fixture(Path(directory), spec)
                summary = path.with_name("summary.json")
                data = json.loads(summary.read_text())
                change(data)
                summary.write_text(json.dumps(data))
                # When/Then: incomplete/invalid terminal evidence fails closed.
                with self.assertRaises(ValueError):
                    report.read_run(path, spec, ("a.evaluate", "b.unhit"))

    def test_nonfinite_json_and_duplicate_nested_keys_fail(self) -> None:
        # Given nonstandard numbers or duplicate nested keys.
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bad.json"
            for raw in ('{"value":NaN}', '{"value":Infinity}', '{"nested":{"n":0,"n":1}}'):
                path.write_text(raw)
                # When/Then: decoding rejects them before interpreting counters.
                with self.assertRaises(ValueError):
                    report.read_json(path)

    def test_complete_cli_writes_both_artifacts_and_missing_group_writes_neither(self) -> None:
        # Given all eleven required groups with a supplied two-scope registry.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = root / "registry.json"
            manifest.write_text(json.dumps({"schema_version": 1, "scopes": ["a.evaluate", "b.unhit"]}))
            specs = report.specifications()
            for spec in specs[:-2]:
                write_fixture(root / "off" / "colony" / spec.name, spec)
            for spec in specs[-2:]:
                write_fixture(root / spec.name, spec)
            command = [sys.executable, str(Path(report.__file__)), "--registry", str(manifest),
                "--off-root", str(root / "off"), "--steward-dir", str(root / "steward_on"),
                "--inventory-dir", str(root / "inventory_on")]
            # When the actual command processes complete inputs.
            good = subprocess.run(command + ["--json-out", str(root / "good.json"),
                "--markdown-out", str(root / "good.md")], capture_output=True, text=True, check=False)
            # Then both outputs carry exact independent arithmetic, without a time total.
            self.assertEqual(good.returncode, 0, good.stderr)
            data = json.loads((root / "good.json").read_text())
            self.assertEqual(data["worlds"], 66)
            self.assertEqual(data["scopes"][0]["calls"], sum(s.count*(s.count+1)//2 for s in specs))
            self.assertEqual(data["scopes"][1]["calls"], 0)
            self.assertNotIn("total_ns", data)
            self.assertIn("possibly contended", (root / "good.md").read_text())
            # Given one missing required fixture, a new invocation must fail closed.
            (root / "off/colony/soak/perf_arms.json").unlink()
            bad = subprocess.run(command + ["--json-out", str(root / "bad.json"),
                "--markdown-out", str(root / "bad.md")], capture_output=True, text=True, check=False)
            self.assertNotEqual(bad.returncode, 0)
            self.assertNotIn("CORE_ARMS VALID", bad.stdout)
            self.assertFalse((root / "bad.json").exists())
            self.assertFalse((root / "bad.md").exists())

    def test_duplicate_json_keys_are_rejected(self) -> None:
        # Given duplicate keys; when parsed, then fail before any aggregation.
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bad.json"
            path.write_text('{"schema_version":1,"schema_version":1}')
            with self.assertRaisesRegex(ValueError, "duplicate"):
                report.read_json(path)

    def test_registry_requires_unique_nonempty_runtime_names(self) -> None:
        # Given malformed manifests; when parsed, then no registry is inferred.
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "manifest.json"
            for scopes in ([], ["a", "a"], [True], [""]):
                with self.subTest(scopes=scopes):
                    path.write_text(json.dumps({"schema_version": 1, "scopes": scopes}))
                    with self.assertRaises(ValueError):
                        report.read_registry(path)


if __name__ == "__main__":
    unittest.main()
