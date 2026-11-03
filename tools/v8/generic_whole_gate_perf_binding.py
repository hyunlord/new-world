# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_perf
"""Bind whole-stage performance recipes to source-selected outputs before parsing."""

from pathlib import Path
from typing import Final

from generic_gate import boundary, document, integer, require
from generic_process import file_hash
from mux2_gate_inputs import Json, Node

from generic_whole_gate_perf import (
    Anchors,
    PerfEvidence,
    collect_performance,
    paired_performance,
)

PERF_COMMAND: Final = '(cd rust && cargo build --release -p sim-test) && bash tools/v8/perf.sh rust/target/release "$0/perf" base > /dev/null && echo "Fresh arm observations; paired performance acceptance is step25/45"'


@boundary
def authenticate_family(session: Node, stage: Node) -> PerfEvidence:
    """Authenticate the actual outer wait, original recipe and retained output bindings.

    The session itself must be issued/loaded through A's materialization authority.
    Native PID in each sidecar remains observer-reported, not a separate child wait.
    """
    step = integer(stage.field("step"))
    require(step in (12, 27), "/perf", "unsupported performance step")
    arm = stage.field("arm").text()
    require(arm in ("base", "candidate"), "/perf", "unknown arm")
    source = session.field("arms").field(arm)
    root = Path(source.field("root").text())
    output = Path(session.field("output").text()) / arm
    expected = (
        ("bash", "-c", PERF_COMMAND, str(output))
        if step == 12
        else (
            "rust/target/release/worldsim_perf",
            "nav-bench",
            "--out",
            str(output / "nav_bench/nav_bench.json"),
        )
    )
    stage.field("argv").equals(list(expected))
    stage.field("state").equals("RETURNED")
    stage.field("native_exit").equals(0)
    stage.field("cwd").equals(str(root))
    environment = stage.field("environment")
    environment.field("WORLDSIM_GATE_OBSERVER").equals("1")
    receipt = Path(stage.field("process_receipt").text())
    require(
        file_hash(receipt) == stage.field("process_receipt_sha256").text(),
        "/process",
        "changed actual wait",
    )
    native = document(receipt.read_text())
    for key in ("pid", "cwd", "environment", "native_exit"):
        native.field(key).equals(stage.field(key).value)
    native.field("argv").equals(
        [
            session.field("bash").field("path").text(),
            "-c",
            'exec "$@" 2>&1',
            "whole-step",
            *expected,
        ]
    )
    native.field("status").equals("PASS")
    native.field("errors").equals([])
    require(integer(native.field("pid")) > 0, "/process", "missing actual parent PID")
    for channel in ("stdout", "stderr"):
        require(
            file_hash(receipt.parent / (channel + ".log"))
            == native.field(channel + "_sha256").text(),
            "/process",
            "changed aggregate output",
        )
    sources = source.field("source_inputs")
    required = (
        ("tools/v8/perf.sh", "rust/crates/sim-test/src/perf.rs")
        if step == 12
        else ("rust/crates/sim-test/src/nav_bench.rs", "tools/v8/soak_script.json")
    )
    for name in required:
        require(
            file_hash(root / name) == sources.field(name).text(),
            "/source",
            "changed workload source",
        )
    files = tuple(
        (name, Node(digest).text())
        for name, digest in stage.field("observer_artifacts").mapping().items()
    )
    modes = tuple(
        (name, integer(Node(mode)))
        for name, mode in stage.field("observer_modes").mapping().items()
    )
    evidence = collect_performance(step, arm, output, Anchors(files, modes))
    if step == 12:
        for _, world in evidence.worlds:
            raw = document(evidence.anchors.read(Path(world.plan.raw_path)))
            raw.field("git_sha").equals(source.field("source_sha").text())
    return evidence


def evaluate_family(
    session: Node, base_stage: Node, candidate_stage: Node
) -> dict[str, Json]:
    """Compare both actual source-selected performance arms without re-execution."""
    base = authenticate_family(session, base_stage)
    candidate = authenticate_family(session, candidate_stage)
    safety = paired_performance(base, candidate)
    return {
        "status": "PASS",
        "safety": list(safety),
        "coverage": {"base": len(base.worlds), "candidate": len(candidate.worlds)},
        "limits": [
            "SOURCE_QUALIFIED_AGGREGATE_WITH_OBSERVER_PID",
            "No independent native child wait or timing acceptance claimed",
        ],
    }
