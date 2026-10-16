#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/art_pipeline/run_alternating_p11.py CHECKOUT CHANGE_ID NEW_OUTPUT_DIR [WORLD]
"""One quiet hold, one Godot launch and one frozen world per candidate experiment."""
from __future__ import annotations

import datetime
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

import summarize_perf_p11 as perf
from validate_alternating_p11 import validate

Record = dict[str, perf.Json]


def provenance(checkout: Path) -> Record:
    """Hash actual source files, including uncommitted candidate edits, and verified release bridge."""
    if os.environ.get("P11_BUILD_PROFILE") != "release":
        raise perf.EvidenceError("Set P11_BUILD_PROFILE=release")
    debug_sha = hashlib.sha256((checkout / "rust/target/debug/libsim_bridge.dylib").read_bytes()).hexdigest()
    release_sha = hashlib.sha256((checkout / "rust/target/release/libsim_bridge.dylib").read_bytes()).hexdigest()
    if debug_sha != release_sha:
        raise perf.EvidenceError("Release/debug dylib SHA mismatch")
    files = {checkout / "project.godot"}
    for directory in ("scripts", "scenes", "shaders"):
        files.update(path for path in (checkout / directory).rglob("*") if path.suffix in {".gd", ".tscn", ".gdshader"})
    return {"build_profile": "release", "release_sha256": release_sha, "checkout": str(checkout), "head": subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True).strip(), "build_sha256": hashlib.sha256((checkout / "rust/target/debug/libsim_bridge.dylib").read_bytes()).hexdigest(), "sources": {str(path.relative_to(checkout)): hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(files)}}


def main() -> int:
    if len(sys.argv) not in {4, 5}:
        raise perf.EvidenceError("Usage: run_alternating_p11.py CHECKOUT CHANGE_ID NEW_OUTPUT_DIR [WORLD=W500]")
    checkout, change, output = Path(sys.argv[1]).resolve(), sys.argv[2], Path(sys.argv[3]).resolve()
    world = sys.argv[4] if len(sys.argv) == 5 else "W500"
    if not change or world not in {"W0", "W1", "W200", "W500"}:
        raise perf.EvidenceError("Requires nonempty candidate ID and W0/W1/W200/W500")
    renderer = (checkout / "scripts/ui/agent_renderer.gd").read_text()
    for method in ("set_perf_candidate_p11", "get_perf_candidate_p11"):
        if f"func {method}(" not in renderer:
            raise perf.EvidenceError(f"Missing temporary renderer integration {method}; no hold or Godot launch started")
    before = provenance(checkout)
    output.mkdir(parents=True, exist_ok=False)
    root = Path(__file__).resolve().parents[2]
    environment = dict(os.environ, P11_PAIR_CHANGE=change, P11_PAIR_WORLD=world,
                       P11_PAIR_OUT=str(output / "frames.json"), P11_PAIR_HEAD=str(before["head"]))
    command = ["bash", str(root / "tools/art_pipeline/with_quiet_lock_p11.sh"), f"pair-{change}-{world}", "3600",
               environment.get("GODOT_BIN", "/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot"),
               "--path", str(checkout), "--resolution", "1920x1080", "--script", "res://scripts/test/art/perf_alternating_frames_p11.gd"]
    launch: Record = {"change_id": change, "world": world, "before": before,
                      "started": datetime.datetime.now(datetime.timezone.utc).isoformat(), "command": command}
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
