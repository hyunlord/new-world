# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 generic_whole_gate_stall_capture.py SESSION SHA ARM OUTPUT
"""Capture newH stall using the original bounded wrapper and real child journal."""

import json
import os
import sys
from pathlib import Path

from gate_stall_child import (
    HASH_ENV,
    PLAN_ENV,
    ChildPlan,
    bind_chain,
    capture_wrapper,
    close_capture,
)
from generic_gate import document, require
from generic_process import atomic_json, file_hash
from generic_stall import artifact_inventory, assess_timing
from generic_whole_gate_contract import current_product_inputs, hashes, load_session
from m6a_r1_stall_capture import required_artifacts
from mux2_gate_inputs import BRIDGE, Json, Node
from mux2_gate_lock import FD_ENV, lane
from mux2_gate_stall import validate_stall_capture

CHAIN = (
    "capture-log-closed.json",
    "native-start.json",
    "native-terminal.json",
    "wrapper-start.json",
    "wrapper-terminal.json",
)


def strict_result(raw: str, arm: str) -> bool:
    """BASE is a complete reference observation; only candidate has acceptance bars."""
    require(arm in ("base", "candidate"), "/arm", "unknown stall arm")
    if arm == "candidate":
        return assess_timing(raw).strict_pass
    return all(
        row.strict_pass
        for row in validate_stall_capture(document(raw).value, strict=False)
    )


def source_binding(path: Path, digest: str, arm: str) -> tuple[Node, Node]:
    session = load_session(path, digest)
    require(
        "materialization" in session.mapping(),
        "/session",
        "product materialization required",
    )
    require(arm in ("base", "candidate"), "/stall", "unknown stall arm")
    bridge = current_product_inputs(session, arm, "bridge")
    return session, bridge


def plan_inputs(session: Node, bridge: Node, arm: str = "candidate") -> dict[Path, str]:
    root = Path(session.field("arms").field(arm).field("root").text())
    godot = Path(session.field("environment").field("GODOT_BIN").text())
    inputs = {
        root / name: sha for name, sha in hashes(bridge.field("source_inputs")).items()
    }
    inputs.update(
        {
            Path(name): sha
            for name, sha in hashes(session.field("external_inputs")).items()
        }
    )
    binary = Path(bridge.field("executable").text())
    require(binary == root / BRIDGE, "/bridge", "unexpected bridge selection")
    inputs[binary] = bridge.field("executable_sha256").text()
    for key in ("python", "bash"):
        tool = session.field(key)
        inputs[Path(tool.field("path").text())] = tool.field("sha256").text()
    require(godot in inputs, "/godot", "Godot binding missing")
    for path, sha in inputs.items():
        require(file_hash(path) == sha, str(path), "input changed")
    return inputs


def make_plan(
    session: Node, bridge: Node, output: Path, arm: str = "candidate"
) -> ChildPlan:
    root = Path(session.field("arms").field(arm).field("root").text())
    expected = Path(session.field("output").text()) / arm / "mux2-stall"
    require(
        output == expected and output.is_absolute() and output.resolve() == output,
        "/output",
        "foreign or noncanonical stall output",
    )
    require(not output.exists(), "/output", "fresh output required")
    godot = Path(session.field("environment").field("GODOT_BIN").text())
    inputs = plan_inputs(session, bridge, arm)
    capture, evidence = output / "capture", output / "chain"
    argv = (
        str(godot),
        "--path",
        str(root),
        "--resolution",
        "1920x1080",
        "--always-on-top",
        "--windowed",
        "--script",
        "res://tools/v8/mux2_stall_test.gd",
        "--",
        str(capture / "run"),
    )
    output.mkdir(parents=True)
    evidence.mkdir()
    plan = output / "child-plan.json"
    atomic_json(
        plan,
        {
            "schema": "stall_bounded_child_v1",
            "protocol": "NONE",
            "root": str(root),
            "capture": str(capture),
            "evidence": str(evidence),
            "argv": list(argv),
            "inputs": {str(path): sha for path, sha in sorted(inputs.items())},
        },
    )
    return ChildPlan(
        plan,
        file_hash(plan),
        root,
        capture,
        evidence,
        argv,
        tuple(sorted(inputs.items())),
    )


def capture(path: Path, digest: str, arm: str, output: Path) -> int:
    """Return actual native 0/1; unfinished or unsafe capture raises, never becomes waived."""
    require(FD_ENV in os.environ, "/lane", "caller-owned inherited lane required")
    require(
        not any(key.startswith("CORE_P7_") for key in os.environ),
        "/protocol",
        "P7 forbidden",
    )
    session, bridge = source_binding(path, digest, arm)
    child = make_plan(session, bridge, output, arm)
    child.capture.mkdir()
    (child.capture / "settings").mkdir()
    receipt: dict[str, Json] = {
        "schema": "whole_stall_capture.v1",
        "session": str(path),
        "session_sha256": digest,
        "arm": arm,
        "bridge_view": bridge.value,
        "plan_sha256": child.digest,
        "integrity": "INCOMPLETE",
        "native_exit": None,
        "capture_complete": False,
        "artifacts": {},
        "chain_files": {},
        "environment": {},
        "errors": [],
        "cleanup_errors": [],
    }
    endpoint = child.capture / "receipt.json"
    atomic_json(output / "capture-started.json", receipt)
    code = -1
    try:
        with lane() as descriptor:
            environment = dict(
                os.environ,
                **{
                    FD_ENV: str(descriptor),
                    PLAN_ENV: str(child.path),
                    HASH_ENV: child.digest,
                    "WORLDSIM_TEST_SETTINGS_DIR": str(child.capture / "settings"),
                    "WORLDSIM_MUX2_PROFILE": "1",
                },
            )
            receipt["environment"] = dict(environment)
            command = [
                "bash",
                str(child.root / "tools/v8/run_bounded.sh"),
                "960",
                *child.argv,
            ]
            launched = False
            try:
                with (child.capture / "command.log").open("w") as log:
                    _ = log.write(json.dumps(command) + "\n")
                    log.flush()
                    launched = True
                    code = capture_wrapper(
                        command, child.root, environment, log.fileno(), descriptor
                    )
            finally:
                if launched:
                    primary = sys.exception()
                    try:
                        close_capture(environment, child.capture / "command.log")
                    except BaseException as error:
                        receipt["cleanup_errors"] = [
                            type(error).__name__ + ": " + str(error)
                        ]
                        if primary is None:
                            raise
            receipt["native_exit"] = code
            files = tuple((name, file_hash(child.evidence / name)) for name in CHAIN)
            pid = bind_chain(child, environment, code, files)
            # Original bytes are anchored before the first timing oracle.
            artifacts = artifact_inventory(child.capture)
            required_artifacts(child.capture)
            raw = (child.capture / "run/stall-report.json").read_text()
            strict = strict_result(raw, arm)
            require(
                code == (0 if strict else 1),
                "/exit",
                "strict result/native return differ",
            )
            end_session, end_bridge = source_binding(path, digest, arm)
            require(
                end_session.value == session.value and end_bridge.value == bridge.value,
                "/source",
                "session or bridge changed",
            )
            require(
                artifact_inventory(child.capture) == artifacts,
                "/capture",
                "capture changed in validation",
            )
            require(
                bind_chain(child, environment, code, files) == pid,
                "/chain",
                "native identity changed",
            )
            receipt.update(
                {
                    "integrity": "VERIFIED",
                    "capture_complete": True,
                    "artifacts": artifacts,
                    "chain_files": dict(files),
                    "native_pid": pid,
                }
            )
    except BaseException as error:
        receipt["errors"] = [type(error).__name__ + ": " + str(error)]
        raise
    finally:
        atomic_json(endpoint, receipt)
    return code


if __name__ == "__main__":
    require(len(sys.argv) == 5, "/arguments", "SESSION SHA ARM OUTPUT required")
    raise SystemExit(
        capture(Path(sys.argv[1]), sys.argv[2], sys.argv[3], Path(sys.argv[4]))
    )
