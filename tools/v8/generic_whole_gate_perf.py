# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_perf
"""Source-qualified shell performance outputs; no independent child-wait claim."""

import json
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from generic_gate import document, integer, require
from generic_perf_observations import PerfObservation, Plan, paired_safety, parse_native
from generic_process import file_hash
from mux2_gate_inputs import Json


@dataclass(frozen=True, slots=True)
class Anchors:
    files: tuple[tuple[str, str], ...]
    modes: tuple[tuple[str, int], ...]

    def bytes(self, path: Path) -> bytes:
        """Read only bytes frozen by the actual stage owner before callbacks."""
        expected = dict(self.files).get(str(path))
        require(
            expected is not None and path.is_absolute() and not path.is_symlink(),
            str(path),
            "unbound artifact",
        )
        require(
            stat.S_ISREG(path.lstat().st_mode) and file_hash(path) == expected,
            str(path),
            "artifact changed",
        )
        return path.read_bytes()

    def read(self, path: Path) -> str:
        """Decode an anchored text artifact."""
        return self.bytes(path).decode()

    def subtree(self, root: Path) -> None:
        """Reject changed membership, modes, links and bytes under the selected output."""
        paths = (root, *root.rglob("*"))
        actual = {str(path): path.lstat().st_mode for path in paths}
        expected = {
            name: mode for name, mode in self.modes if Path(name).is_relative_to(root)
        }
        require(
            actual == expected and bool(expected),
            str(root),
            "output modes or namespace changed",
        )
        for path in paths:
            require(not path.is_symlink(), str(path), "output link")
            require(
                path.is_dir() or stat.S_ISREG(path.lstat().st_mode),
                str(path),
                "special output",
            )
            if path.is_file():
                _ = self.bytes(path)


@dataclass(frozen=True, slots=True)
class SelectedRun:
    name: str
    raw: Path
    kind: Literal["perf", "nav"]
    ticks: int
    seed: int
    agents: int | None
    workload: str


@dataclass(frozen=True, slots=True)
class PerfEvidence:
    step: int
    arm: str
    worlds: tuple[tuple[str, PerfObservation], ...]
    anchors: Anchors
    attribution: Literal["SOURCE_QUALIFIED_AGGREGATE_WITH_OBSERVER_PID"] = (
        "SOURCE_QUALIFIED_AGGREGATE_WITH_OBSERVER_PID"
    )


def selected_runs(step: int, output: Path, anchors: Anchors) -> tuple[SelectedRun, ...]:
    """Select exact source-owned natural/quick/stress or default nav invocations."""
    if step == 27:
        return (
            SelectedRun(
                "nav",
                output / "nav_bench/nav_bench.json",
                "nav",
                4320,
                15100,
                1000,
                json.dumps({"script": "soak_script.json"}),
            ),
        )
    require(step == 12, "/step", "unsupported performance family")
    result: list[SelectedRun] = []
    selections = [
        (f"natural_r{repeat}", "natural", (19600, 19603, 19605, 19606), 172800, None)
        for repeat in range(1, 4)
    ] + [
        ("quick", "quick", (15100,), 1440, 500),
        ("stress_v26", "stress", (15100,), 1440, 500),
    ]
    for directory, mode, seeds, ticks, agents in selections:
        root = output / "perf" / directory
        anchors.subtree(root)
        entries = document(anchors.read(root / "runs.json")).field("runs").array()
        require(
            tuple(integer(row.field("seed")) for row in entries) == seeds,
            str(root),
            "run index seed/order coverage differs",
        )
        names: set[str] = set()
        for row in entries:
            row.field("status").equals("ok")
            name = row.field("file").text()
            require(
                Path(name).name == name
                and name.endswith(".json")
                and name not in names,
                str(root),
                "unsafe or duplicate raw file",
            )
            names.add(name)
            seed = integer(row.field("seed"))
            workload = {
                "mode": mode,
                "scene": "game_v26" if mode == "natural" else "stress_baseline",
                "side": 64,
                "feed": "none" if mode == "natural" else "fed",
            }
            result.append(
                SelectedRun(
                    f"{directory}-s{seed}",
                    root / name,
                    "perf",
                    ticks,
                    seed,
                    agents,
                    json.dumps(workload),
                )
            )
        require(
            {p.name for p in root.glob("*.json")} == names | {"runs.json"},
            str(root),
            "unregistered raw report",
        )
        require(
            {p.name for p in root.glob("*.tick_us.bin")}
            == {Path(n).with_suffix(".tick_us.bin").name for n in names},
            str(root),
            "timing sample coverage differs",
        )
        require(
            {p.name for p in (root / "gate_observations").iterdir()}
            == {n + ".safety.json" for n in names},
            str(root),
            "sidecar coverage differs",
        )
    return tuple(result)


def collect_performance(
    step: int, arm: str, output: Path, anchors: Anchors
) -> PerfEvidence:
    """Consume actual source-qualified output after C authenticates the shell stage.

    A/C must bind the exact step recipe, product source/build and original anchors.
    A sidecar PID is native observer attribution, not an independently observed wait.
    """
    require(arm in {"base", "candidate"}, "/arm", "invalid arm")
    runs = selected_runs(step, output, anchors)
    if step == 27:
        anchors.subtree(output / "nav_bench")
    worlds: list[tuple[str, PerfObservation]] = []
    for run in runs:
        raw = anchors.read(run.raw)
        sidecar = run.raw.parent / "gate_observations" / (run.raw.name + ".safety.json")
        encoded = anchors.read(sidecar)
        pid = integer(document(encoded).field("producer_pid"))
        plan = Plan(
            run.kind, run.ticks, run.seed, run.agents, str(run.raw), pid, run.workload
        )
        observation = parse_native(encoded, raw, plan)
        report = document(raw)
        report.field("build_profile").equals("release")
        if run.kind == "perf":
            report.field("warmup").field("ticks").equals(60)
            report.field("tick_summary").field("ticks").equals(run.ticks - 60)
            binary = run.raw.with_suffix(".tick_us.bin")
            require(
                dict(anchors.files).get(str(binary)) == file_hash(binary)
                and binary.stat().st_size == run.ticks * 4,
                str(binary),
                "incomplete raw timings",
            )
        worlds.append((run.name, observation))
    for run in runs:
        anchors.subtree(run.raw.parent)
    return PerfEvidence(step, arm, tuple(worlds), anchors)


def paired_performance(base: PerfEvidence, candidate: PerfEvidence) -> tuple[Json, ...]:
    """Apply the existing ordinary safety rule to the same exact invocation set."""
    require(
        base.step == candidate.step
        and base.arm == "base"
        and candidate.arm == "candidate",
        "/pair",
        "foreign performance arm",
    )
    require(
        tuple(name for name, _ in base.worlds)
        == tuple(name for name, _ in candidate.worlds),
        "/pair",
        "performance occurrence coverage differs",
    )
    return tuple(
        paired_safety(left, right)
        for (_, left), (_, right) in zip(base.worlds, candidate.worlds, strict=True)
    )
