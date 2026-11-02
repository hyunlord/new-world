"""Local pristine stall outcome consistency; never whole-chain or safety admission."""
import re
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path, PurePosixPath
from typing import Literal

from generic_gate import GateContractError, boundary, require
from generic_process import (
    ProcessResult,
    ProcessSpec,
    check_digest,
    environment_for,
    file_hash,
    real_path,
)
from generic_stall import (
    CaptureBinding,
    StallVerdict,
    assess_timing,
    authenticate_capture,
    parse,
)
from mux2_gate_inputs import Json


@dataclass(frozen=True, slots=True)
class StallOutcomePlan:
    invocation_id: str
    protocol: Literal["NONE"]
    source_sha: str
    capture_root: Path
    native_spec: ProcessSpec
    capture_receipt_sha256: str
    native_receipt_sha256: str
    captured_files: tuple[tuple[str, str], ...]


@dataclass(frozen=True, slots=True)
class StallOutcome:
    verdict: StallVerdict
    native_exit: Literal[0, 1]
    native_pid: int
    capture_receipt_sha256: str
    native_receipt_sha256: str
    artifacts: tuple[tuple[str, str], ...]
    capture_wrapper_complete: bool
    safety: Literal["PENDING"] = field(default="PENDING", init=False)
    publication_authority: Literal[False] = field(default=False, init=False)


def _inventory(root: Path) -> tuple[tuple[str, str], ...]:
    real_path(root, directory=True)
    files: list[tuple[str, str]] = []
    for path in sorted(root.rglob("*")):
        real_path(path, directory=path.is_dir())
        if path.is_file():
            files.append((path.relative_to(root).as_posix(), file_hash(path)))
    return tuple(files)


def _manifest(plan: StallOutcomePlan) -> tuple[tuple[str, str], ...]:
    names: set[str] = set()
    for name, digest in plan.captured_files:
        path = PurePosixPath(name)
        require(bool(name) and name != "." and not path.is_absolute() and ".." not in path.parts
                and path.as_posix() == name and name not in names, "/manifest", "unsafe or duplicate path")
        check_digest(digest, name)
        names.add(name)
    require("receipt.json" in names, "/manifest", "capture receipt missing")
    actual = _inventory(plan.capture_root)
    require(actual == tuple(sorted(plan.captured_files)), "/manifest", "capture inventory changed")
    require(dict(actual)["receipt.json"] == plan.capture_receipt_sha256, "/manifest", "receipt hash differs")
    return actual


def _bind_native(plan: StallOutcomePlan, result: ProcessResult, expected_exit: Literal[0, 1]) -> int:
    """Mirror bind_process checks, admitting only the recomputed stall exit1 diagnostic."""
    spec = plan.native_spec
    require(result.receipt == spec.output_root / "process.json", "/process", "foreign receipt")
    require(file_hash(result.receipt) == plan.native_receipt_sha256, "/process", "receipt changed")
    require(type(result.pid) is int and result.pid > 0, "/process", "missing native PID")
    pid = result.pid
    if pid is None:
        raise GateContractError("CONTRACT_REJECTED", "/process", "missing native PID")
    status = "PASS" if expected_exit == 0 else "FAIL"
    require(type(result.native_exit) is int and result.native_exit == expected_exit and result.status == status,
            "/process", "native result disagrees with timing")
    process = parse(result.receipt.read_text())
    expected: dict[str, Json] = {
        "schema_version": 1, "status": status, "native_exit": expected_exit, "pid": pid,
        "argv": list(spec.argv), "cwd": str(spec.cwd), "executable": str(spec.executable),
        "expected_executable_sha256": spec.executable_sha256, "input_root": str(spec.input_root),
        "output_root": str(spec.output_root), "fresh_directories": list(spec.fresh_directories),
        "environment": dict(environment_for(spec)),
        "environment_policy": "fixed_defaults_plus_explicit_only_no_inheritance",
        "expected_inputs": dict(spec.inputs), "before_inputs": dict(spec.inputs), "after_inputs": dict(spec.inputs),
        "expected_external_inputs": {str(path): digest for path, digest in spec.external_inputs},
        "before_external_inputs": {str(path): digest for path, digest in spec.external_inputs},
        "after_external_inputs": {str(path): digest for path, digest in spec.external_inputs},
        "before_executable_sha256": spec.executable_sha256, "after_executable_sha256": spec.executable_sha256,
        "errors": [] if expected_exit == 0 else ["native exit 1"],
        "build_provenance_verified": False,
    }
    for key, value in expected.items():
        process.field(key).equals(value)
    # The existing 960-second bash wrapper owns timeout; no replacement direct-child policy.
    require(spec.timeout_seconds is None and "timeout_seconds" not in process.mapping()
            and "timed_out" not in process.mapping(), "/process", "altered timeout policy")
    started = datetime.fromisoformat(process.field("started_at").text())
    ended = datetime.fromisoformat(process.field("ended_at").text())
    require(started.tzinfo is not None and ended.tzinfo is not None and ended > started,
            "/process", "missing or reversed completion timestamps")
    duration = process.field("duration_ns").value
    require(type(duration) is int and duration > 0, "/process", "invalid elapsed duration")
    for name in ("stdout", "stderr"):
        process.field(name + "_sha256").equals(file_hash(spec.output_root / (name + ".log")))
    check_digest(spec.executable_sha256, "/executable")
    require(file_hash(spec.executable) == spec.executable_sha256, "/process", "executable changed")
    require(len(dict(spec.inputs)) == len(spec.inputs), "/inputs", "duplicate inputs")
    for name, digest in spec.inputs:
        path = PurePosixPath(name)
        require(bool(name) and name != "." and not path.is_absolute() and ".." not in path.parts
                and path.as_posix() == name, "/inputs", "unsafe input")
        check_digest(digest, name)
        require(file_hash(spec.input_root / name) == digest, "/inputs", "input changed")
    require(len(dict(spec.external_inputs)) == len(spec.external_inputs), "/inputs", "duplicate external inputs")
    for path, digest in spec.external_inputs:
        check_digest(digest, str(path))
        require(not path.is_relative_to(spec.input_root) and not path.is_relative_to(spec.output_root)
                and not path.is_relative_to(plan.capture_root), "/inputs", "external input overlaps evidence/product")
        require(file_hash(path) == digest, "/inputs", "external input changed")
    return pid


@boundary
def validate_outcome(plan: StallOutcomePlan, native_result: ProcessResult) -> StallOutcome:
    """Authenticate pristine NONE captures; all lifecycle/safety and parent joins remain pending."""
    require(plan.protocol == "NONE", "/protocol", "only NONE supported")
    require(re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", plan.invocation_id) is not None,
            "/invocation", "invalid invocation identity")
    require(re.fullmatch(r"[0-9a-f]{40}", plan.source_sha) is not None, "/source", "full SHA required")
    check_digest(plan.capture_receipt_sha256, "/capture_receipt")
    check_digest(plan.native_receipt_sha256, "/native_receipt")
    spec = plan.native_spec
    roots = (spec.input_root, spec.output_root, plan.capture_root)
    for root in roots:
        real_path(root, directory=True)
    require(all(not left.is_relative_to(right) and not right.is_relative_to(left)
                for index, left in enumerate(roots) for right in roots[index + 1:]),
            "/roots", "overlapping product/evidence roots")
    require(spec.cwd == spec.input_root, "/cwd", "native cwd differs from product")
    real_path(spec.executable, directory=False)
    environment = environment_for(spec)
    require(environment.get("WORLDSIM_TEST_SETTINGS_DIR") == str(plan.capture_root / "settings")
            and environment.get("WORLDSIM_MUX2_PROFILE") == "1",
            "/environment", "missing or altered original capture settings/profile")
    require(not any(key.startswith("CORE_P7_") for key in environment), "/protocol", "P7 environment forbidden")
    expected_argv = (str(spec.executable), "--path", str(spec.input_root), "--resolution", "1920x1080",
                     "--always-on-top", "--windowed", "--script", "res://tools/v8/mux2_stall_test.gd",
                     "--", str(plan.capture_root / "run"))
    require(spec.argv == expected_argv, "/argv", "not original direct Godot recipe")
    frozen = _manifest(plan)
    capture_receipt = parse((plan.capture_root / "receipt.json").read_text())
    capture_receipt.field("inputs").equals(dict(spec.inputs))
    capture_receipt.field("godot").equals(str(spec.executable))
    capture_receipt.field("godot_sha256").equals(spec.executable_sha256)
    raw = (plan.capture_root / "run/stall-report.json").read_text()
    require("p7_admission" not in parse(raw).mapping(), "/protocol", "P7 report forbidden")
    verdict = assess_timing(raw)
    expected_exit: Literal[0, 1] = 0 if verdict.strict_pass else 1
    pid = _bind_native(plan, native_result, expected_exit)
    wrapper_argv = ("bash", str(spec.input_root / "tools/v8/run_bounded.sh"), "960", *expected_argv)
    evidence = authenticate_capture(spec.input_root, plan.capture_root,
                                    CaptureBinding(plan.source_sha, wrapper_argv, expected_exit,
                                                   plan.capture_receipt_sha256))
    require(evidence.verdict == verdict, "/capture", "verdict changed during authentication")
    require(_manifest(plan) == frozen, "/capture", "capture changed during authentication")
    _ = _bind_native(plan, native_result, expected_exit)
    return StallOutcome(verdict, expected_exit, pid, plan.capture_receipt_sha256,
                        plan.native_receipt_sha256, frozen, evidence.original_complete)
