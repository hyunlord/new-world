"""Synthetic authentic-format files only: no Godot/native child execution claims."""
import json
import unittest
from collections.abc import Generator
from contextlib import contextmanager
from dataclasses import dataclass, replace
from unittest.mock import patch

import test_generic_stall as capture_fixtures
from generic_gate import GateContractError
from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from generic_stall import parse
from generic_stall_outcome import StallOutcomePlan, validate_outcome
from mux2_gate_inputs import Json, Node, snapshot
from test_generic_stall import report_with


@dataclass(frozen=True, slots=True)
class Synthetic:
    capture: capture_fixtures.CaptureTests
    plan: StallOutcomePlan
    result: ProcessResult
    receipt: dict[str, Json]

    def reseal(self) -> StallOutcomePlan:
        """Synthetic coordinator freeze, never a native provenance claim."""
        _ = self.result.receipt.write_text(json.dumps(self.receipt))
        return replace(self.plan, native_receipt_sha256=file_hash(self.result.receipt),
                       capture_receipt_sha256=file_hash(self.plan.capture_root / "receipt.json"),
                       captured_files=tuple(sorted((p.relative_to(self.plan.capture_root).as_posix(), file_hash(p))
                                                   for p in self.plan.capture_root.rglob("*") if p.is_file())))


@contextmanager
def fixture(strict: bool = False) -> Generator[Synthetic]:
    capture = capture_fixtures.CaptureTests()
    capture.setUp()
    try:
        argv = (str(capture.godot), "--path", str(capture.root), "--resolution", "1920x1080",
                "--always-on-top", "--windowed", "--script", "res://tools/v8/mux2_stall_test.gd",
                "--", str(capture.capture / "run"))
        capture.argv = ("bash", str(capture.root / "tools/v8/run_bounded.sh"), "960", *argv)
        binding = capture.prepare(strict)
        output = capture.root.parent / "native"
        output.mkdir()
        _ = (output / "stdout.log").write_text("synthetic raw native stdout")
        _ = (output / "stderr.log").write_text("")
        inputs = tuple((name, Node(value).text()) for name, value in sorted(snapshot(capture.root).items()))
        spec = ProcessSpec(argv, capture.root, capture.godot, file_hash(capture.godot), capture.root, inputs, output,
                           environment=(("WORLDSIM_TEST_SETTINGS_DIR", str(capture.capture / "settings")),
                                        ("WORLDSIM_MUX2_PROFILE", "1")))
        native_exit = 0 if strict else 1
        status = "PASS" if strict else "FAIL"
        receipt: dict[str, Json] = {
            "schema_version": 1, "status": status, "native_exit": native_exit, "pid": 12345,
            "argv": list(argv), "cwd": str(spec.cwd), "executable": str(spec.executable),
            "expected_executable_sha256": spec.executable_sha256, "input_root": str(spec.input_root),
            "output_root": str(output), "fresh_directories": [], "environment": dict(environment_for(spec)),
            "environment_policy": "fixed_defaults_plus_explicit_only_no_inheritance",
            "expected_inputs": dict(inputs), "before_inputs": dict(inputs), "after_inputs": dict(inputs),
            "expected_external_inputs": {}, "before_external_inputs": {}, "after_external_inputs": {},
            "before_executable_sha256": spec.executable_sha256, "after_executable_sha256": spec.executable_sha256,
            "errors": [] if strict else ["native exit 1"], "build_provenance_verified": False,
            "started_at": "2026-11-03T00:00:00+00:00", "ended_at": "2026-11-03T00:11:00+00:00",
            "duration_ns": 660_000_000_000,
            "stdout_sha256": file_hash(output / "stdout.log"), "stderr_sha256": file_hash(output / "stderr.log"),
        }
        path = output / "process.json"
        _ = path.write_text(json.dumps(receipt))
        result = ProcessResult(path, status, native_exit, 12345)
        plan = StallOutcomePlan("synthetic-stall", "NONE", capture.sha, capture.capture, spec,
                                binding.receipt_sha256, file_hash(path),
                                tuple(sorted((p.relative_to(capture.capture).as_posix(), file_hash(p))
                                             for p in capture.capture.rglob("*") if p.is_file())))
        yield Synthetic(capture, plan, result, receipt)
    finally:
        _ = capture.doCleanups()


class OutcomeTests(unittest.TestCase):
    def test_strict_and_eligible_retain_raw_outcome_without_safety_authority(self) -> None:
        for strict in (True, False):
            with self.subTest(strict=strict), fixture(strict) as f:
                result = validate_outcome(f.plan, f.result)
                self.assertEqual(result.native_exit, 0 if strict else 1)
                self.assertEqual(result.verdict.status, "STRICT_PASS" if strict else "WAIVER_ELIGIBLE")
                self.assertEqual(result.capture_wrapper_complete, strict)
                self.assertEqual(result.safety, "PENDING")
                self.assertFalse(result.publication_authority)
                self.assertIn("receipt.json", dict(result.artifacts))

    def test_native_receipt_integrity_boundaries(self) -> None:
        mutations: tuple[tuple[str, Json], ...] = (
            ("errors", []), ("errors", ["native exit 1", "timeout"]), ("status", "PASS"),
            ("native_exit", 0), ("native_exit", -15), ("pid", 987), ("pid", True),
            ("cwd", "/foreign"), ("argv", ["godot", "--headless"]), ("environment", {}),
            ("after_inputs", {}), ("after_external_inputs", {"foreign": "0" * 64}),
            ("after_executable_sha256", "0" * 64), ("timed_out", True),
            ("ended_at", "2026-11-02T00:00:00+00:00"), ("duration_ns", True),
            ("stdout_sha256", "0" * 64), ("environment_policy", "inherited"),
        )
        for key, value in mutations:
            with self.subTest(key=key, value=value), fixture() as f:
                f.receipt[key] = value
                with self.assertRaises(GateContractError):
                    _ = validate_outcome(f.reseal(), f.result)

    def test_reverse_exit_or_missing_result_rejected(self) -> None:
        for strict in (False, True):
            with fixture(strict) as f:
                for result in (replace(f.result, native_exit=1 if strict else 0),
                               replace(f.result, pid=None), replace(f.result, pid=0),
                               replace(f.result, native_exit=None), replace(f.result, receipt=f.plan.capture_root / "receipt.json")):
                    with self.subTest(result=result), self.assertRaises(GateContractError):
                        _ = validate_outcome(f.plan, result)

    def test_frozen_failed_capture_and_native_logs_cannot_change(self) -> None:
        for relative in ("command.log", "run/stall-report.json", "settings/stall-1-1.0.cfg", "receipt.json"):
            with self.subTest(relative=relative), fixture() as f:
                path = f.plan.capture_root / relative
                _ = path.write_text(path.read_text() + " ")
                with self.assertRaises(GateContractError):
                    _ = validate_outcome(f.plan, f.result)
        for name in ("stdout.log", "stderr.log", "process.json"):
            with self.subTest(name=name), fixture() as f:
                _ = (f.plan.native_spec.output_root / name).write_text("changed")
                with self.assertRaises(GateContractError):
                    _ = validate_outcome(f.plan, f.result)

    def test_manifest_missing_extra_symlink_traversal_duplicate(self) -> None:
        with fixture() as f:
            for manifest in (f.plan.captured_files[:-1], f.plan.captured_files + f.plan.captured_files[:1],
                             f.plan.captured_files + (("../outside", "0" * 64),)):
                with self.subTest(manifest=manifest), self.assertRaises(GateContractError):
                    _ = validate_outcome(replace(f.plan, captured_files=manifest), f.result)
            extra = f.plan.capture_root / "unexpected"
            _ = extra.write_text("extra")
            with self.assertRaises(GateContractError):
                _ = validate_outcome(f.plan, f.result)
            extra.unlink()
            extra.symlink_to(f.capture.godot)
            with self.assertRaises(GateContractError):
                _ = validate_outcome(f.plan, f.result)

    def test_original_capture_environment_cannot_be_dropped_or_changed(self) -> None:
        with fixture() as f:
            for key, wrong in (("WORLDSIM_TEST_SETTINGS_DIR", "/foreign"), ("WORLDSIM_MUX2_PROFILE", "0")):
                for omit in (True, False):
                    entries = tuple((name, wrong if name == key else value)
                                    for name, value in f.plan.native_spec.environment if not (omit and name == key))
                    spec = replace(f.plan.native_spec, environment=entries)
                    f.receipt["environment"] = dict(environment_for(spec))
                    sealed = f.reseal()
                    with self.subTest(key=key, omit=omit), self.assertRaises(GateContractError):
                        _ = validate_outcome(replace(sealed, native_spec=spec), f.result)

    def test_p7_never_activated_by_ambient_environment(self) -> None:
        with fixture() as f, patch.dict("os.environ", {"CORE_P7_RUN": "1", "CORE_P7_DEADLINE_EPOCH": "1"}):
            self.assertEqual(validate_outcome(f.plan, f.result).safety, "PENDING")
            for key in ("CORE_P7_RUN", "CORE_P7_DEADLINE_EPOCH", "CORE_P7_OTHER"):
                spec = replace(f.plan.native_spec, environment=(*f.plan.native_spec.environment, (key, "1")))
                with self.assertRaises(GateContractError):
                    _ = validate_outcome(replace(f.plan, native_spec=spec), f.result)
            path = f.plan.capture_root / "run/stall-report.json"
            report = parse(path.read_text()).mapping()
            report["p7_admission"] = {}
            _ = path.write_text(json.dumps(report))
            with self.assertRaises(GateContractError):
                _ = validate_outcome(f.reseal(), f.result)

    def test_existing_capture_authentication_rejects_unsealed_invalid_telemetry(self) -> None:
        for report in (report_with(crowded=True), report_with(55, 3), report_with(61, 1)):
            with fixture() as f:
                _ = (f.plan.capture_root / "run/stall-report.json").write_text(json.dumps(report))
                with self.assertRaises(GateContractError):
                    _ = validate_outcome(f.reseal(), f.result)
        mutations: tuple[tuple[str, Json], ...] = (("complete", False), ("arms", []), ("errors", ["engine error"]))
        for key, value in mutations:
            with fixture() as f:
                path = f.plan.capture_root / "run/stall-report.json"
                report = parse(path.read_text()).mapping()
                report[key] = value
                _ = path.write_text(json.dumps(report))
                with self.assertRaises(GateContractError):
                    _ = validate_outcome(f.reseal(), f.result)

    def test_gpu_settings_zooms_and_protocol_fail_closed(self) -> None:
        for key, value in (("gpu", ""), ("render_driver", "headless"), ("settings_dir", "/foreign")):
            with self.subTest(key=key), fixture() as f:
                path = f.plan.capture_root / "run/stall-report.json"
                report = parse(path.read_text())
                report.field("environment").mapping()[key] = value
                _ = path.write_text(json.dumps(report.value))
                with self.assertRaises(GateContractError):
                    _ = validate_outcome(f.reseal(), f.result)
        with fixture() as f:
            for plan in (replace(f.plan, protocol="P7"), replace(f.plan, source_sha="abc"),
                         replace(f.plan, native_receipt_sha256="bad"),
                         replace(f.plan, capture_root=f.plan.native_spec.output_root)):
                with self.subTest(plan=plan), self.assertRaises(GateContractError):
                    _ = validate_outcome(plan, f.result)
            path = f.plan.capture_root / "run/stall-report.json"
            report = parse(path.read_text())
            arms = report.field("arms").array()
            report.mapping()["arms"] = [arm.value for arm in arms] + [arms[0].value]
            _ = path.write_text(json.dumps(report.value))
            with self.assertRaises(GateContractError):
                _ = validate_outcome(f.reseal(), f.result)

    def test_capture_godot_and_native_inventory_are_the_same_inputs(self) -> None:
        with fixture() as f:
            f.capture.receipt["godot"] = str(f.capture.root / "tools/play.sh")
            f.capture.receipt["godot_sha256"] = file_hash(f.capture.root / "tools/play.sh")
            _ = f.capture.seal()
            with self.assertRaises(GateContractError):
                _ = validate_outcome(f.reseal(), f.result)
        with fixture() as f:
            plan = replace(f.plan, native_spec=replace(f.plan.native_spec, inputs=()))
            for key in ("expected_inputs", "before_inputs", "after_inputs"):
                f.receipt[key] = {}
            sealed = f.reseal()
            with self.assertRaises(GateContractError):
                _ = validate_outcome(replace(plan, native_receipt_sha256=sealed.native_receipt_sha256), f.result)

    def test_existing_authenticator_checks_source_bridge_settings_and_terminal(self) -> None:
        for name in ("scripts/producer.gd", "sim_bridge.gdextension", "rust/target/debug/libsim_bridge.dylib"):
            with self.subTest(name=name), fixture() as f:
                _ = (f.capture.root / name).write_text("changed")
                with self.assertRaises(GateContractError):
                    _ = validate_outcome(f.plan, f.result)
        with fixture() as f:
            path = f.capture.capture / "command.log"
            _ = path.write_text(path.read_text() + "SCRIPT ERROR: synthetic\n")
            with self.assertRaises(GateContractError):
                _ = validate_outcome(f.reseal(), f.result)


if __name__ == "__main__":
    _ = unittest.main()
