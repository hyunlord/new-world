#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/art_pipeline/run_activity_captures_p10.py CHECKOUT NEWDIR POPULOUS_SEED SELECTION_EVIDENCE
"""Capture six live activity frames after Part 2 integration; never stress worlds."""
from __future__ import annotations

import hashlib
import json
import math
import signal
import os
from pathlib import Path
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from typing import Final, TextIO, TypeAlias
from collections.abc import Mapping

QUIET: Final = Path("/Users/rexxa/work/.mac_quiet.lock")
Json: TypeAlias = None | bool | int | float | str | list["Json"] | dict[str, "Json"]
SCRIPT: Final = "scripts/test/art/capture_activities_p10.gd"


@dataclass(frozen=True, slots=True)
class Capture:
    seed: int
    day: int
    hour: int
    zoom: int

    @property
    def name(self) -> str:
        return f"s{self.seed}_d{self.day}_{self.hour:02d}00_z{self.zoom}"


def cases(populous_seed: int) -> tuple[Capture, ...]:
    """Require the actual later probe winner; there is deliberately no default."""
    if not 20261011 <= populous_seed <= 20261027:
        raise InvalidInput("populous seed must be within the GOAL 6.5 probe cohort")
    return tuple(Capture(seed, day, hour, zoom)
                 for seed, day, hour in [(20261010, 30, 10), (20261010, 30, 15),
                                         (populous_seed, 60, 10)] for zoom in [4, 1])


class InvalidInput(ValueError):
    """The requested launch lacks required explicit evidence or configuration."""


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def utc() -> str:
    return datetime.now(timezone.utc).isoformat()


def source_digest(checkout: Path) -> str:
    """Bind all renderer scripts, scenes, shaders, atlas and active bridge libraries."""
    paths = [p for folder in ["scripts", "scenes", "shaders"]
             for p in (checkout / folder).rglob("*") if p.is_file()
             and p.suffix in {".gd", ".tscn", ".gdshader"}]
    paths += [checkout / "sim_bridge.gdextension"]
    paths += list((checkout / "rust/target/debug").glob("libsim_bridge.*"))
    paths += [checkout / "project.godot", checkout / "assets/art/atlas/world_atlas.json",
              checkout / "rust/crates/sim-bridge/src/ffi/world_node.rs"]
    return hashlib.sha256("\n".join(f"{p.relative_to(checkout)} {sha256(p)}"
                                   for p in sorted(set(paths))).encode()).hexdigest()


def launch_command(checkout: Path, godot: Path, seconds: int) -> list[str]:
    """Window wrapper bounds the quiet-lock recheck and the Godot invocation."""
    return ["bash", "tools/art_pipeline/with_window_lock.sh", str(seconds),
            "bash", "-c", 'while [[ -d "$1" ]]; do date -u; echo "waiting for quiet lock $1"; sleep 300; done; shift; exec "$@"',
            "activity-launch", str(QUIET), str(godot), "--path", str(checkout),
            "--fixed-fps", "60", "--resolution", "1920x1080", "--script", "res://" + SCRIPT]


def remaining(deadline: float) -> float:
    """Reject missing/expired deadlines before waiting or launching."""
    seconds = deadline - time.time()
    if not math.isfinite(seconds) or seconds <= 0:
        raise InvalidInput("P10_DEADLINE_EPOCH must remain in the future")
    return seconds


def wait_for_quiet(deadline: float, log_path: Path, name: str) -> None:
    while QUIET.is_dir():
        left = remaining(deadline)
        with log_path.open("a") as log:
            log.write(f"{utc()} {name} waiting for {QUIET}\n")
        time.sleep(min(300.0, left))
    remaining(deadline)


def bounded_launch(command: list[str], checkout: Path, environment: Mapping[str, str],
                   stream: TextIO, deadline: float, limit: int) -> int:
    """Bound wrapper lock waits and descendants, terminating only our process group."""
    timeout = min(remaining(deadline), float(limit))
    process = subprocess.Popen(command, cwd=checkout, env=environment, stdout=stream,
                               stderr=subprocess.STDOUT, start_new_session=True)
    try:
        return process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            return process.wait()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        return 124


def record(value: Json) -> Mapping[str, Json]:
    if not isinstance(value, dict):
        raise InvalidInput("expected JSON object")
    return value


def integer(value: Json, minimum: int = 0) -> int:
    if type(value) is not int or value < minimum:
        raise InvalidInput("expected bounded integer")
    return value


def vector(value: Json, length: int) -> list[float]:
    if not isinstance(value, list) or len(value) != length:
        raise InvalidInput("invalid vector length")
    if any(type(v) not in (int, float) or not math.isfinite(v) for v in value):
        raise InvalidInput("nonfinite/non-numeric vector")
    return [float(v) for v in value]


def validate_capture(payload: Json, case: Capture) -> None:
    """Independent structural/state check; intentionally not an expected-pose oracle."""
    data = record(payload)
    expected: Mapping[str, Json] = {"schema_version": 1, "seed": case.seed, "display_day": case.day,
        "hour": case.hour, "zoom": case.zoom, "state_change_frame": 10, "capture_frame": 130,
        "fixed_fps": 60, "viewport": [1920, 1080], "errors": [], "complete": True,
        "pose_oracle_checked": False, "fixed_delta_valid": True, "delta_tolerance": 1e-9}
    for key, value in expected.items():
        if data.get(key) != value or type(data.get(key)) is not type(value):
            raise InvalidInput("capture field mismatch: " + key)
    if integer(data.get("delta_samples"), 120) > 130:
        raise InvalidInput("too many fixed-delta samples for capture frame")
    low, high = vector([data.get("minimum_delta"), data.get("maximum_delta")], 2)
    if low > high or abs(low - 1 / 60) > 1e-9 or abs(high - 1 / 60) > 1e-9:
        raise InvalidInput("observed process deltas differ from fixed FPS")
    info, clock = record(data.get("world_info")), record(data.get("clock"))
    tpd, offset = integer(info.get("ticks_per_day"), 1), integer(info.get("clock_offset_ticks"))
    tick = (case.day - 1) * tpd + case.hour * tpd // 24 - offset
    if (tpd, offset) != (1440, 480) or integer(info.get("seed")) != case.seed:
        raise InvalidInput("live calendar/seed differs from baseline")
    if integer(data.get("tick")) != tick or integer(clock.get("tick")) != tick or integer(clock.get("day_index")) != case.day - 1 or clock.get("time_of_day") != case.hour:
        raise InvalidInput("capture does not match requested calendar instant")
    raw = record(data.get("raw_activity_snapshot"))
    holders = data.get("activity_holders")
    count = integer(data.get("holder_count"), 1)
    if integer(data.get("population"), 1) < count or not isinstance(holders, list) or len(holders) != count:
        raise InvalidInput("holder/population count mismatch")
    columns = [raw.get(k) for k in ("ids", "kinds", "place_x", "place_y", "until_tick")]
    if any(not isinstance(column, list) or len(column) != count for column in columns):
        raise InvalidInput("raw activity arrays differ in length")
    for column_index, column in enumerate(columns):
        for raw_value in column:
            integer(raw_value, -1 if column_index == 4 else 0)
    vector(data.get("camera"), 2)
    agent_snapshot = record(data.get("raw_agent_snapshot"))
    entities, ids, states = (agent_snapshot.get(k) for k in ("ids", "agent_ids", "states"))
    population = integer(data.get("population"), 1)
    if any(not isinstance(values, list) or len(values) != population for values in (entities, ids, states)):
        raise InvalidInput("raw agent columns differ from population")
    live = {integer(e, 1): (integer(a), integer(state)) for e, a, state in zip(entities, ids, states)}
    if len(live) != population or len(set(ids)) != population:
        raise InvalidInput("duplicate raw agent identity")
    seen_entities: set[int] = set()
    seen_agents: set[int] = set()
    for index, value in enumerate(holders):
        row = record(value)
        entity, agent = integer(row.get("entity_bits"), 1), integer(row.get("agent_id"))
        kind, until = integer(row.get("kind")), integer(row.get("until_tick"), -1)
        place = vector(row.get("place"), 2)
        if live.get(entity) != (agent, integer(row.get("state_tag"))):
            raise InvalidInput("activity holder differs from live agent snapshot")
        if kind > 6 or entity in seen_entities or agent in seen_agents:
            raise InvalidInput("duplicate ID or invalid kind")
        seen_entities.add(entity)
        seen_agents.add(agent)
        if [entity, kind, *place, until] != [column[index] for column in columns]:
            raise InvalidInput("holder rows do not match raw activity arrays")
        if row.get("renderer_present") is not True or row.get("missing_metadata") != []:
            raise InvalidInput("renderer absent or incomplete")
        meta = record(row.get("metadata"))
        for key, expected_value in {"p10_activity_schema": 1, "activity_kind": kind,
                "activity_place_known": until >= 0, "entity_bits": entity, "art_id": agent}.items():
            if meta.get(key) != expected_value or type(meta.get(key)) is not type(expected_value):
                raise InvalidInput("metadata identity mismatch: " + key)
        for key in ("activity", "pose", "body", "facing", "sprite_id", "cold_pose", "cold_base_pose", "activity_frame_view", "activity_frame_activity"):
            if not isinstance(meta.get(key), str) or not meta[key]:
                raise InvalidInput("missing final render metadata: " + key)
        overrides = meta.get("activity_overrides")
        if not isinstance(overrides, list) or any(v not in ("pickup", "meal", "cold", "morning_chill") for v in overrides):
            raise InvalidInput("invalid explicit override list")
        if meta["body"] not in ("adult_a", "adult_b", "child") or meta["facing"] not in ("N", "S", "E", "W") or meta["activity_frame_view"] not in ("N", "S", "E"):
            raise InvalidInput("invalid body/view metadata")
        if integer(row.get("state_tag")) > 4:
            raise InvalidInput("unknown agent state tag")
        rect = vector(row.get("viewport_rect"), 4)
        if rect[2] <= 0 or rect[3] <= 0:
            raise InvalidInput("invalid body viewport rectangle")
        for key in ("visible", "visible_in_tree", "intersects_viewport", "fully_inside_viewport", "flip_h", "region_enabled"):
            if type(row.get(key)) is not bool:
                raise InvalidInput("missing visibility/texture flag: " + key)
        x, y, width, height = rect
        if row["intersects_viewport"] != (x < 1920 and y < 1080 and x + width > 0 and y + height > 0) or row["fully_inside_viewport"] != (x >= 0 and y >= 0 and x + width <= 1920 and y + height <= 1080):
            raise InvalidInput("viewport visibility flags differ from actual rectangle")
        vector(row.get("region_rect"), 4)
        if not isinstance(row.get("texture_path"), str):
            raise InvalidInput("missing texture path")


def run(checkout: Path, output: Path, populous_seed: int, selection: Path) -> int:
    """Preserve failed launches and stop on any capture/metadata invariant failure."""
    deadline = float(os.environ.get("P10_DEADLINE_EPOCH", "0"))
    remaining(deadline)
    matrix = cases(populous_seed)
    godot = Path(os.environ.get("GODOT_BIN", "/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot"))
    seconds = int(os.environ.get("P10_CAPTURE_LIMIT", "1800"))
    if not 1 <= seconds <= 3600 or not os.access(godot, os.X_OK):
        raise InvalidInput("requires executable GODOT_BIN and 1..3600 seconds limit")
    if not selection.is_file() or selection.stat().st_size == 0:
        raise InvalidInput("requires nonempty actual populous-seed selection evidence")
    for relative in [SCRIPT, "project.godot", "tools/art_pipeline/with_window_lock.sh",
                     "tools/art_pipeline/run_bounded.sh"]:
        if not (checkout / relative).is_file():
            raise InvalidInput("missing required file: " + relative)
    output.mkdir(parents=False, exist_ok=False)
    (output / "selection-evidence").write_bytes(selection.read_bytes())
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=checkout, text=True).strip()
    diff = subprocess.check_output(["git", "diff", "HEAD", "--"], cwd=checkout)
    (output / "working-tree.patch").write_bytes(diff)
    (output / "git-status.txt").write_bytes(subprocess.check_output(
        ["git", "status", "--short"], cwd=checkout))
    digest = source_digest(checkout)
    (output / "runner.json").write_text(json.dumps({
        "checkout": str(checkout), "head": head, "started": utc(), "source_digest": digest,
        "capture_script_sha256": sha256(checkout / SCRIPT),
        "bridge_sha256": sha256(checkout / "rust/target/debug/libsim_bridge.dylib"),
        "bridge_source_sha256": sha256(checkout / "rust/crates/sim-bridge/src/ffi/world_node.rs"),
        "selection_path": str(selection), "selection_sha256": sha256(selection),
        "selection_claim": "caller-supplied actual probe winner; not inferred by this runner",
        "cases": [asdict(case) for case in matrix], "limit_seconds": seconds,
        "quiet_lock": str(QUIET), "deadline_epoch": deadline, "pose_oracle_checked": False}, indent=2) + "\n")
    for case in matrix:
        wait_for_quiet(deadline, output / "quiet-waits.log", case.name)
        directory = output / case.name
        directory.mkdir()
        if source_digest(checkout) != digest:
            raise InvalidInput("capture source changed during matrix; preserve run and restart in newdir")
        environment = os.environ.copy()
        environment.update(P10_ACTIVITY_OUT=str(directory), P10_ACTIVITY_SEED=str(case.seed),
                           P10_ACTIVITY_DAY=str(case.day), P10_ACTIVITY_HOUR=str(case.hour),
                           P10_ACTIVITY_ZOOM=str(case.zoom))
        started = utc()
        with (directory / "launch.log").open("w") as log:
            code = bounded_launch(launch_command(checkout, godot, seconds), checkout, environment, log, deadline, seconds)
        text = (directory / "launch.log").read_text()
        sidecar = directory / "capture.json"
        errors: list[str] = []
        try:
            validate_capture(json.loads(sidecar.read_text()), case)
        except (OSError, ValueError, TypeError, KeyError, IndexError) as error:
            errors.append(str(error))
        passed = (code == 0 and not errors and (directory / "capture.png").is_file()
                  and "SCRIPT ERROR" not in text and "ERROR:" not in text
                  and source_digest(checkout) == digest)
        (directory / "launch.json").write_text(json.dumps({
            "started": started, "finished": utc(), "exit_code": code, "validation_errors": errors, "deadline_epoch": deadline,
            "capture_contract_pass": passed, "pose_oracle_checked": False,
            "artifacts": {p.name: sha256(p) for p in [sidecar, directory / "capture.png"] if p.is_file()}}, indent=2) + "\n")
        if not passed:
            return 1
    return 0


def main(arguments: list[str]) -> int:
    if len(arguments) != 4:
        print(__doc__ + "\nUsage: run_activity_captures_p10.py CHECKOUT NEWDIR POPULOUS_SEED SELECTION_EVIDENCE", file=sys.stderr)
        return 2
    try:
        return run(Path(arguments[0]).resolve(), Path(arguments[1]).absolute(),
                   int(arguments[2]), Path(arguments[3]).resolve())
    except (InvalidInput, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Activity capture refused/failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
