#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/art_pipeline/run_alternating_p13.py CHECKOUT CHANGE_ID NEW_OUTPUT_DIR [WORLD]
"""One quiet hold, one Godot launch and one frozen world per candidate experiment."""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

import summarize_perf_p13 as perf
from validate_alternating_p13 import validate

Record = dict[str, perf.Json]


def provenance(checkout: Path) -> Record:
    """Hash actual source files, including uncommitted candidate edits, and verified release bridge."""
    if os.environ.get("P13_BUILD_PROFILE") != "release":
        raise perf.EvidenceError("Set P13_BUILD_PROFILE=release")
    debug_sha = hashlib.sha256((checkout / "rust/target/debug/libsim_bridge.dylib").read_bytes()).hexdigest()
    release_sha = hashlib.sha256((checkout / "rust/target/release/libsim_bridge.dylib").read_bytes()).hexdigest()
    if debug_sha != release_sha:
        raise perf.EvidenceError("Release/debug dylib SHA mismatch")
    files = {checkout / "project.godot"}
    for directory in ("scripts", "scenes", "shaders"):
        files.update(path for path in (checkout / directory).rglob("*") if path.suffix in {".gd", ".tscn", ".gdshader"})
    return {"build_profile": "release", "release_sha256": release_sha, "checkout": str(checkout), "head": subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True).strip(), "build_sha256": hashlib.sha256((checkout / "rust/target/debug/libsim_bridge.dylib").read_bytes()).hexdigest(), "sources": {str(path.relative_to(checkout)): hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(files)}}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checkout", type=Path)
    parser.add_argument("change", choices=("declump-kernel", "write-skip"))
    parser.add_argument("output", type=Path)
    parser.add_argument("world", nargs="?", default="W500", choices=("W0", "W1", "W200", "W500"))
    parser.add_argument("--kernel-state", choices=("on", "off"))
    args = parser.parse_args()
    checkout, change, output = args.checkout.resolve(), args.change, args.output.resolve()
    world = args.world
    if change == "write-skip" and args.kernel_state is None:
        parser.error("write-skip requires --kernel-state on|off")
    renderer = (checkout / "scripts/ui/agent_renderer.gd").read_text()
    methods = ("set_perf_candidate_p13", "get_perf_candidate_p13", "get_perf_candidate_p12") if change == "write-skip" else ("set_perf_candidate_p12", "get_perf_candidate_p12")
    for method in methods:
        if f"func {method}(" not in renderer:
            raise perf.EvidenceError(f"Missing temporary renderer integration {method}; no hold or Godot launch started")
    before = provenance(checkout)
    output.mkdir(parents=True, exist_ok=False)
    root = Path(__file__).resolve().parents[2]
    environment = dict(os.environ, P13_VALIDITY_OUT=str(output / "frames.json.validity.json"), P13_PAIR_CHANGE=change, P13_PAIR_WORLD=world,
                       P13_PAIR_OUT=str(output / "frames.json"), P13_PAIR_HEAD=str(before["head"]))
    if change == "write-skip":
        environment.update(P13_PAIR_KERNEL_STATE=args.kernel_state,
                           P12_DECLUMP_KERNEL="1" if args.kernel_state == "on" else "0", P13_WRITE_SKIP="0")
    command = ["bash", str(root / "tools/art_pipeline/with_quiet_lock_p13.sh"), f"pair-{change}-{world}", "3600",
               environment.get("GODOT_BIN", "/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot"),
               "--path", str(checkout), "--resolution", "1920x1080", "--script", "res://scripts/test/art/perf_alternating_frames_p13.gd"]
    launch: Record = {"change_id": change, "world": world, "before": before,
                      "started": datetime.datetime.now(datetime.timezone.utc).isoformat(), "command": command}
    if change == "write-skip":
        launch["kernel_enabled"] = args.kernel_state == "on"
    (output / "launch.json").write_text(json.dumps(launch, indent=2) + "\n")
    with (output / "session.log").open("w") as stream:
        completed = subprocess.run(command, env=environment, stdout=stream, stderr=subprocess.STDOUT, check=False)
    (output / "session.exit").write_text(str(completed.returncode) + "\n")
    launch.update({"exit_code": completed.returncode, "finished": datetime.datetime.now(datetime.timezone.utc).isoformat(), "after": provenance(checkout)})
    (output / "launch.json").write_text(json.dumps(launch, indent=2) + "\n")
    result = validate(output)
    (output / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
