#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/art_pipeline/validate_alternating_p11.py NEW_OUTPUT_DIR
"""Validate completed control/candidate experiments; never imply pixel acceptance."""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path

import summarize_perf_p11 as perf

Record = dict[str, perf.Json]


def read(path: Path) -> Record:
    return dict(perf.mapping(json.loads(path.read_text())))


def validate(folder: Path) -> Record:
    """Require terminal evidence, unchanged source/build and six ordered raw blocks."""
    if not all((folder / name).is_file() for name in ("session.exit", "session.log", "launch.json", "frames.json")):
        raise perf.EvidenceError("Missing terminal session evidence")
    if (folder / "session.exit").read_text().strip() != "0":
        raise perf.EvidenceError("Nonzero terminal session status")
    if "ERROR" in (folder / "session.log").read_text():
        raise perf.EvidenceError("Session log contains ERROR")
    launch, data = read(folder / "launch.json"), read(folder / "frames.json")
    before, after = perf.mapping(launch.get("before")), perf.mapping(launch.get("after"))
    if not launch.get("finished") or before != after or launch.get("exit_code") != 0:
        raise perf.EvidenceError("Incomplete launch or source/build changed during session")
    sources = perf.mapping(before.get("sources"))
    if not sources or "scripts/ui/agent_renderer.gd" not in sources:
        raise perf.EvidenceError("Source provenance is absent")
    for digest, length in [(before.get("head"), 40), (before.get("build_sha256"), 64), *[(value, 64) for value in sources.values()]]:
        if not isinstance(digest, str) or len(digest) != length or any(char not in "0123456789abcdef" for char in digest):
            raise perf.EvidenceError("Malformed source/build provenance")
    if data.get("build_commit") != before["head"] or data.get("build_sha256") != before["build_sha256"]:
        raise perf.EvidenceError("Collector/build provenance differs from launch")
    if data.get("world") != launch.get("world") or not data.get("change_id") or data.get("change_id") != launch.get("change_id"):
        raise perf.EvidenceError("World/candidate launch mismatch")
    if data.get("valid") is not True or data.get("errors") != [] or data.get("metric") != "process_ms + render_cpu_ms":
        raise perf.EvidenceError("Invalid measurement or metric")
    initial = perf.mapping(data["initial"])
    if data.get("final") != initial or initial.get("zoom") != [4, 4] or data.get("simulation_speed") != 0 or initial.get("simulation_frozen") is not False:
        raise perf.EvidenceError("World must remain frozen at Z4")
    for key in ("vsync_restored", "layers_restored", "camera_process_restored", "probes_removed"):
        if perf.mapping(data.get("restoration", {})).get(key) is not True:
            raise perf.EvidenceError(f"Unproven restoration: {key}")
    blocks = [perf.mapping(row) for row in perf.sequence(data.get("blocks", []))]
    if len(blocks) != 6:
        raise perf.EvidenceError("Requires three ordered 150-frame control/candidate pairs")
    pooled: dict[str, list[perf.Json]] = {"control": [], "candidate": []}
    last_frame = -1.0
    render_positive = False
    pairs: list[perf.Json] = []
    for index, block in enumerate(blocks):
        enabled = bool(index % 2)
        mode = "candidate" if enabled else "control"
        switch = perf.mapping(block.get("switch_state", {}))
        if block.get("index") != index or block.get("mode") != mode or switch.get("candidate_enabled") is not enabled or switch.get("change_id") != data["change_id"]:
            raise perf.EvidenceError("Control/candidate order or switch readback mismatch")
        samples, discarded = perf.sequence(block.get("samples")), perf.sequence(block.get("discarded"))
        if len(samples) != 150 or len(discarded) != 30:
            raise perf.EvidenceError("Requires 30 discarded and 150 counted frames per block")
        if block.get("max_fps") != 0 or block.get("vsync_disabled") is not True or block.get("camera_process_frozen") is not True or block.get("window_size") != [1920, 1080]:
            raise perf.EvidenceError("Unproven window/vsync/camera/FPS controls")
        for raw in [*discarded, *samples]:
            row = perf.mapping(raw)
            frame = perf.number(row.get("process_frame"))
            if row.get("state") != initial or row.get("paired") is not True or frame <= last_frame or row.get("ticks_run") != 0:
                raise perf.EvidenceError("Frame pairing/order or frozen-world mismatch")
            if row.get("direct_read_context") != {"inspector_visible": False, "influence_channel": -1, "layers_enabled": True}:
                raise perf.EvidenceError("Inspector/overlay/layer controls differ from matrix defaults")
            last_frame = frame
            if any(perf.number(row.get(metric)) < 0 for metric in perf.METRICS):
                raise perf.EvidenceError("Negative frame metric")
            if not math.isclose(perf.number(row["primary_cpu_ms"]), perf.number(row["process_ms"]) + perf.number(row["render_cpu_ms"]), rel_tol=0, abs_tol=1e-9):
                raise perf.EvidenceError("Primary differs from process + render CPU")
            render_positive |= perf.number(row["render_cpu_ms"]) > 0
        pooled[mode].extend(samples)
        pairs.append({"index": index, "mode": mode, "metrics": {metric: dict(perf.summary(samples, metric)) for metric in perf.METRICS}})
    if not render_positive:
        raise perf.EvidenceError("Render CPU unavailable")
    control, candidate = perf.summary(pooled["control"], "primary_cpu_ms"), perf.summary(pooled["candidate"], "primary_cpu_ms")
    return {"valid": True, "world": data["world"], "change_id": data["change_id"], "blocks": pairs, "control": dict(control), "candidate": dict(candidate), "median_speedup": perf.number(control["median"]) / perf.number(candidate["median"]) if perf.number(candidate["median"]) > 0 else None, "pixel_identity": "NOT_VERIFIED", "source": str(folder.resolve())}


def main() -> None:
    result = validate(Path(sys.argv[1]))
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
