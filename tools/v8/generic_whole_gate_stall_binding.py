# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_stall
"""Authenticate actual newH stage41 and admit only after the global conjunction."""

import re
from pathlib import Path

from gate_stall_child import HASH_ENV, PLAN_ENV, ChildPlan, bind_chain
from generic_gate import boundary, document, integer, require
from generic_process import file_hash
from generic_whole_gate_contract import hashes
from m6a_r1_stall_capture import producer_sources, required_artifacts
from mux2_gate_inputs import BRIDGE, Json, Node

from generic_whole_gate_perf import Anchors
from generic_whole_gate_stall import OtherStep
from generic_whole_gate_stall_capture import (
    CHAIN,
    plan_inputs,
    source_binding,
    strict_result,
)


def conjunction(others: tuple[OtherStep, ...], safety: bool) -> None:
    require(
        type(safety) is bool and safety, "/stall", "world safety incomplete or failed"
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


@boundary
def evaluate_stage(
    session: Node,
    stage: Node,
    other_steps: tuple[OtherStep, ...],
    world_safety_passed: bool,
) -> dict[str, Json]:
    """A/C own session and stage journal admission; this verifies the retained native chain."""
    stage.field("step").equals(41)
    arm = stage.field("arm").text()
    require(arm in ("base", "candidate"), "/arm", "unknown stall arm")
    if arm == "candidate":
        conjunction(other_steps, world_safety_passed)
    stage.field("state").equals("RETURNED")
    root = Path(session.field("arms").field(arm).field("root").text())
    output = Path(session.field("output").text()) / arm / "mux2-stall"
    anchors = Anchors(
        tuple(hashes(stage.field("observer_artifacts")).items()),
        tuple(
            (name, integer(Node(mode)))
            for name, mode in stage.field("observer_modes").mapping().items()
        ),
    )
    anchors.subtree(output)
    capture = output / "capture"
    receipt = document(anchors.read(capture / "receipt.json"))
    receipt.field("schema").equals("whole_stall_capture.v1")
    receipt.field("integrity").equals("VERIFIED")
    receipt.field("errors").equals([])
    receipt.field("cleanup_errors").equals([])
    receipt.field("capture_complete").equals(True)
    receipt.field("arm").equals(arm)
    session_path = Path(receipt.field("session").text())
    session_digest = receipt.field("session_sha256").text()
    actual_session, bridge = source_binding(session_path, session_digest, arm)
    require(
        actual_session.value == session.value, "/session", "foreign capture session"
    )
    receipt.field("bridge_view").equals(bridge.value)
    expected = [
        "python3",
        "tools/v8/generic_whole_gate_stall_capture.py",
        str(session_path),
        session_digest,
        arm,
        str(output),
    ]
    stage.field("argv").equals([value for value in expected])
    stage.field("cwd").equals(str(root))
    code = integer(receipt.field("native_exit"))
    require(code in (0, 1), "/exit", "unsettled or unexpected native outcome")
    stage.field("native_exit").equals(code)
    process_path = Path(stage.field("process_receipt").text())
    require(
        file_hash(process_path) == stage.field("process_receipt_sha256").text(),
        "/process",
        "changed process",
    )
    process = document(anchors.read(process_path))
    process.field("argv").equals(
        [
            session.field("bash").field("path").text(),
            "-c",
            'exec "$@" 2>&1',
            "whole-step",
            *expected,
        ]
    )
    for key in ("pid", "cwd", "environment", "native_exit"):
        process.field(key).equals(stage.field(key).value)
    process.field("status").equals("PASS" if code == 0 else "FAIL")
    process.field("errors").equals([] if code == 0 else ["native exit 1"])
    for channel in ("stdout", "stderr"):
        require(
            file_hash(process_path.parent / (channel + ".log"))
            == process.field(channel + "_sha256").text(),
            "/process",
            "changed aggregate log",
        )
    plan_path = output / "child-plan.json"
    plan = document(anchors.read(plan_path))
    plan.field("schema").equals("stall_bounded_child_v1")
    plan.field("protocol").equals("NONE")
    plan.field("root").equals(str(root))
    plan.field("capture").equals(str(capture))
    plan.field("evidence").equals(str(output / "chain"))
    godot = session.field("environment").field("GODOT_BIN").text()
    argv = (
        godot,
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
    plan.field("argv").equals(list(argv))
    receipt.field("plan_sha256").equals(file_hash(plan_path))
    child = ChildPlan(
        plan_path,
        file_hash(plan_path),
        root,
        capture,
        output / "chain",
        argv,
        tuple((Path(name), sha) for name, sha in hashes(plan.field("inputs")).items()),
    )
    require(
        dict(child.inputs) == plan_inputs(session, bridge, arm),
        "/inputs",
        "foreign source or tool inputs",
    )
    chain = tuple(hashes(receipt.field("chain_files")).items())
    require(set(dict(chain)) == set(CHAIN), "/chain", "incomplete chain")
    environment = hashes(receipt.field("environment"))
    supplied = hashes(stage.field("environment"))
    _ = supplied.pop("OLDPWD", None)
    supplied["PWD"] = str(root)
    supplied["SHLVL"] = str(int(supplied.get("SHLVL", "0")))
    expected_environment = supplied | {
        PLAN_ENV: str(child.path),
        HASH_ENV: child.digest,
        "WORLDSIM_TEST_SETTINGS_DIR": str(capture / "settings"),
        "WORLDSIM_MUX2_PROFILE": "1",
    }
    require(
        environment == expected_environment,
        "/environment",
        "foreign capture environment",
    )
    pid = bind_chain(child, environment, code, chain)
    receipt.field("native_pid").equals(pid)
    wrapper = document(anchors.read(child.evidence / "wrapper-start.json"))
    wrapper.field("capture_pid").equals(integer(process.field("pid")))
    required_artifacts(capture)
    raw = anchors.read(capture / "run/stall-report.json")
    report = document(raw)
    strict = strict_result(raw, arm)
    require(
        code == (0 if strict else 1),
        "/exit",
        "strict result/native exit differ",
    )
    report.field("environment").field("settings_dir").equals(str(capture / "settings"))
    report.field("bridge").field("sha256").equals(file_hash(root / BRIDGE))
    sources = hashes(report.field("source_sha256"))
    require(
        set(sources) == producer_sources(root),
        "/source",
        "producer source inventory differs",
    )
    for name, sha in sources.items():
        require(file_hash(root / name) == sha, name, "producer source changed")
    command = anchors.read(capture / "command.log")
    first = command.splitlines()[0]
    document('{"argv":' + first + "}").field("argv").equals(
        ["bash", str(root / "tools/v8/run_bounded.sh"), "960", *argv]
    )
    terminal = re.findall(
        r"^MUX2_STALL_DONE complete=true pass=(true|false) errors=\[\] evidence=(.+)$",
        command,
        re.MULTILINE,
    )
    require(
        terminal
        == [("true" if code == 0 else "false", str(capture / "run/stall-report.json"))],
        "/capture",
        "invalid terminal",
    )
    require(
        "SCRIPT ERROR" not in command
        and not re.search(r"^ERROR:", command, re.MULTILINE),
        "/capture",
        "engine errors",
    )
    artifacts = {
        name: sha
        for name, sha in anchors.files
        if Path(name).is_relative_to(capture) and Path(name) != capture / "receipt.json"
    }
    receipt.field("artifacts").equals({name: sha for name, sha in artifacts.items()})
    anchors.subtree(output)
    final_session, final_bridge = source_binding(session_path, session_digest, arm)
    require(
        final_session.value == session.value and final_bridge.value == bridge.value,
        "/source",
        "source changed during audit",
    )
    require(
        bind_chain(child, environment, code, chain) == pid,
        "/chain",
        "chain changed during audit",
    )
    return {
        "status": "PASS",
        "stall_status": "REFERENCE_COMPLETE"
        if arm == "base"
        else ("STRICT_PASS" if code == 0 else "WAIVED"),
        "native_exit": code,
        "native_pid": pid,
        "capture_complete": True,
        "reference_stage_exits": [{"arm": arm, "step": 41, "native_exit": 1}]
        if arm == "base" and code == 1
        else [],
        "waived_stage_exits": []
        if code == 0 or arm == "base"
        else [{"arm": arm, "step": 41, "native_exit": 1}],
        "publication_authority": False,
    }
