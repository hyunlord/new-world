# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_stall
"""Existing real stall-chain/capture joins followed by the whole-gate conjunction."""

from dataclasses import dataclass
from typing import Literal

from gate_stall_child import ChildPlan, bind_chain
from generic_gate import require
from generic_stall import (
    CaptureBinding,
    StallEvidence,
    assess_timing,
    authenticate_capture,
    authenticate_derived_capture,
)
from generic_stall_build import BuildEvidenceReference

from generic_whole_gate_perf import Anchors


@dataclass(frozen=True, slots=True)
class StallRequest:
    source_sha: str
    child: ChildPlan
    wrapper_environment: tuple[tuple[str, str], ...]
    chain_files: tuple[tuple[str, str], ...]
    anchors: Anchors
    build_reference: BuildEvidenceReference | None = None


@dataclass(frozen=True, slots=True)
class StallEvidenceJoin:
    capture: StallEvidence
    native_pid: int
    native_exit: int
    capture_complete: bool
    request: StallRequest


@dataclass(frozen=True, slots=True)
class OtherStep:
    number: int
    status: Literal["PASS", "FAIL", "INCOMPLETE", "UNKNOWN", "NOT_RUN"]


@dataclass(frozen=True, slots=True)
class StallAdmission:
    status: Literal["STRICT_PASS", "WAIVED"]
    native_exit: int
    native_pid: int
    publication_authority: Literal[False] = False


def authenticate_stall(request: StallRequest) -> StallEvidenceJoin:
    """Require actual native chain plus capture; never construct child process results."""
    child = request.child
    request.anchors.subtree(child.capture)
    request.anchors.subtree(child.evidence)
    raw = request.anchors.read(child.capture / "run/stall-report.json")
    timing = assess_timing(raw)
    code = 0 if timing.strict_pass else 1
    pid = bind_chain(
        child, dict(request.wrapper_environment), code, request.chain_files
    )
    receipt = child.capture / "receipt.json"
    digest = dict(request.anchors.files).get(str(receipt))
    require(digest is not None, "/capture", "missing original capture receipt anchor")
    assert digest is not None
    argv = ("bash", str(child.root / "tools/v8/run_bounded.sh"), "960", *child.argv)
    binding = CaptureBinding(request.source_sha, argv, code, digest)
    if request.build_reference is None:
        capture = authenticate_capture(child.root, child.capture, binding)
    else:
        capture = authenticate_derived_capture(
            child.root, child.capture, binding, request.build_reference
        ).capture
    require(
        capture.verdict == timing,
        "/capture",
        "timing changed during source authentication",
    )
    request.anchors.subtree(child.capture)
    request.anchors.subtree(child.evidence)
    require(
        bind_chain(child, dict(request.wrapper_environment), code, request.chain_files)
        == pid,
        "/chain",
        "native identity changed",
    )
    # assess_timing validates the native complete field even when old strict wrapper says false.
    return StallEvidenceJoin(capture, pid, code, True, request)


def admit_stall(
    evidence: StallEvidenceJoin,
    others: tuple[OtherStep, ...],
    world_safety_passed: bool,
) -> StallAdmission:
    """Grant a local waiver only after every other original step and world safety pass."""
    require(
        type(world_safety_passed) is bool and world_safety_passed,
        "/stall",
        "world safety incomplete or failed",
    )
    require(
        tuple(sorted(row.number for row in others))
        == tuple(n for n in range(1, 52) if n != 41),
        "/stall",
        "missing, duplicate or foreign other step",
    )
    require(
        all(row.status == "PASS" for row in others),
        "/stall",
        "all other steps must pass",
    )
    fresh = authenticate_stall(evidence.request)
    require(
        fresh == evidence and fresh.capture_complete,
        "/stall",
        "capture incomplete or changed",
    )
    return StallAdmission(
        "STRICT_PASS" if fresh.capture.verdict.strict_pass else "WAIVED",
        fresh.native_exit,
        fresh.native_pid,
    )
