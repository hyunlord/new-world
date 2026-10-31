#!/usr/bin/env python3
"""M-READ gate verifiers: interleaved bench, core normalised comparison,
and the stall waiver. Called by gate.sh when READ_GATE_CONFIG is set.

Usage:
    python3 read_gate_checks.py bench <config_json> <log_dir>
    python3 read_gate_checks.py core_fixture <config_json> <log_dir>
    python3 read_gate_checks.py stall_waiver <stall_log>
"""
import json
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def load_config(path):
    """Load and validate the READ gate config."""
    config = json.loads(Path(path).read_text())
    assert config.get("schema_version") == 1, "bad schema_version"
    for key in ("base_probe", "base_perf", "base_probe_sha256", "base_perf_sha256"):
        assert key in config and config[key], f"missing {key}"
    return config


def verify_binary(path, expected_sha256):
    """Check that a binary exists and matches its pinned sha256."""
    import hashlib
    p = Path(path)
    assert p.exists(), f"binary not found: {path}"
    actual = hashlib.sha256(p.read_bytes()).hexdigest()
    assert actual == expected_sha256, f"sha256 mismatch for {path}: {actual} != {expected_sha256}"


def wait_for_load(threshold=8, timeout=2700):
    """Wait until 1-min load average is below threshold."""
    start = time.time()
    while time.time() - start < timeout:
        load1 = os.getloadavg()[0]
        if load1 < threshold:
            print(f"admission: quiet (1-min load {load1:.2f} < {threshold})")
            return load1
        time.sleep(30)
    raise RuntimeError(f"load did not drop below {threshold} in {timeout}s")


# ── Interleaved bench ───────────────────────────────────────────────────

def bench(config_path, log_dir):
    """3-round interleaved bench, BASE vs HEAD, medians compared."""
    config = load_config(config_path)
    verify_binary(config["base_perf"], config["base_perf_sha256"])
    head_perf = str(ROOT / "rust/target/release/worldsim_perf")
    assert Path(head_perf).exists(), f"HEAD perf not built: {head_perf}"
    out = Path(log_dir)
    out.mkdir(parents=True, exist_ok=True)

    base_bin = config["base_perf"]
    script = str(ROOT / "tools/v8/bench_script.json")
    wait_for_load()

    # 3 rounds, interleaved
    for r in range(1, 4):
        for label, binary in [("base", base_bin), ("new", head_perf)]:
            load1 = os.getloadavg()[0]
            print(f"round {r} {label} load={load1:.1f}")
            run_dir = out / f"{label}-r{r}"
            subprocess.check_call(
                [binary, "bench", "--out", str(run_dir), "--commands", script, "--repeats", "1"],
                cwd=str(ROOT),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )

    # Compute medians and compare
    scenes = {}
    for r in range(1, 4):
        for label in ("base", "new"):
            run_dir = out / f"{label}-r{r}" / "bench.json"
            data = json.loads(run_dir.read_text())
            for scene in data["scenes"]:
                name = scene["label"]
                scenes.setdefault(name, {"base_mean": [], "new_mean": [], "base_p99": [], "new_p99": []})
                scenes[name][f"{label}_mean"].append(scene["tick_mean_us"])
                scenes[name][f"{label}_p99"].append(scene["tick_p99_us"])

    failures = []
    print(f"{'scene':<20} {'base':>10} {'new':>10} {'ratio':>8} {'verdict':>8}")
    print("-" * 60)
    for name in sorted(scenes):
        bm = sorted(scenes[name]["base_mean"])
        nm = sorted(scenes[name]["new_mean"])
        bp = sorted(scenes[name]["base_p99"])
        np_ = sorted(scenes[name]["new_p99"])
        b_med = bm[len(bm) // 2]
        n_med = nm[len(nm) // 2]
        ratio_mean = n_med / b_med if b_med else 0
        b_p99 = bp[len(bp) // 2]
        n_p99 = np_[len(np_) // 2]
        ratio_p99 = n_p99 / b_p99 if b_p99 else 0
        ok_mean = ratio_mean <= 1.10
        ok_p99 = ratio_p99 <= 1.10
        verdict = "ok" if ok_mean and ok_p99 else "FAIL"
        print(f"{name:<20} {b_med:>10.1f} {n_med:>10.1f} {ratio_mean:>8.3f} {verdict:>8}")
        if not ok_mean:
            failures.append(f"{name} tick mean x{ratio_mean:.3f}")
        if not ok_p99:
            failures.append(f"{name} p99 x{ratio_p99:.3f}")

    # Also check C3 ceilings from the last HEAD run
    last_head = json.loads((out / "new-r3/bench.json").read_text())
    ceilings = last_head.get("ceilings", {})
    for ceiling, passed in ceilings.items():
        if not passed:
            failures.append(f"ceiling {ceiling} MISS")
        print(f"ceiling {ceiling}: {'PASS' if passed else 'MISS'}")

    if failures:
        print(f"READ_BENCH FAIL: {', '.join(failures)}")
        sys.exit(1)
    print("READ_BENCH PASS")


# ── Core fixture normalised comparison ──────────────────────────────────

def strip_timing_and_read_keys(obj):
    """Remove timing keys (as identity.sh does) AND the read-only presentation keys."""
    TIMING_KEYS = {"wall_seconds", "ms_per_agent_tick", "timing", "git_sha"}
    READ_KEYS = {"why_not", "diagnosis_cache", "requests_cache"}
    ALL_EXCLUDED = TIMING_KEYS | READ_KEYS
    if isinstance(obj, dict):
        return {k: strip_timing_and_read_keys(v) for k, v in obj.items() if k not in ALL_EXCLUDED}
    if isinstance(obj, list):
        return [strip_timing_and_read_keys(item) for item in obj]
    return obj


def core_fixture(config_path, log_dir):
    """Run baseline and colony worlds at BASE and HEAD, compare normalised."""
    config = load_config(config_path)
    verify_binary(config["base_probe"], config["base_probe_sha256"])
    head_probe = str(ROOT / "rust/target/release/worldsim_probe")
    assert Path(head_probe).exists(), f"HEAD probe not built: {head_probe}"
    out = Path(log_dir)
    out.mkdir(parents=True, exist_ok=True)
    base_probe = config["base_probe"]

    fixtures = [
        ("baseline", ["--scene", "game_v26", "--seeds", "0..4", "--days", "10", "--workers", "4"]),
        ("colony_u", ["--scene", "colony", "--seeds", "0..4", "--days", "10", "--workers", "4",
                       "--commands", "tools/v8/u_script.json", "--check-determinism"]),
        ("colony_soak", ["--scene", "colony", "--seeds", "0..2", "--days", "20", "--workers", "2",
                          "--commands", "tools/v8/soak_script.json"]),
    ]

    for name, args in fixtures:
        for label, binary in [("base", base_probe), ("head", head_probe)]:
            run_dir = out / f"{label}_{name}"
            print(f"running {label} {name}...")
            subprocess.check_call(
                [binary] + args + ["--out", str(run_dir)],
                cwd=str(ROOT),
                stdout=(out / f"{label}_{name}.log").open("w"),
                stderr=subprocess.STDOUT,
            )

    # Compare each fixture, stripping timing and read-only keys
    for name, _ in fixtures:
        base_summary = json.loads((out / f"base_{name}/summary.json").read_text())
        head_summary = json.loads((out / f"head_{name}/summary.json").read_text())
        a_clean = strip_timing_and_read_keys(base_summary)
        b_clean = strip_timing_and_read_keys(head_summary)
        if a_clean == b_clean:
            print(f"CORE_FIXTURE {name}: MATCH (timing and read keys excluded)")
        else:
            a_str = json.dumps(a_clean, sort_keys=True, indent=2).splitlines()
            b_str = json.dumps(b_clean, sort_keys=True, indent=2).splitlines()
            diffs = 0
            for i, (la, lb) in enumerate(zip(a_str, b_str)):
                if la != lb:
                    diffs += 1
                    if diffs <= 10:
                        print(f"  line {i+1}:")
                        print(f"    A: {la}")
                        print(f"    B: {lb}")
            print(f"CORE_FIXTURE FAIL on {name} ({diffs} differences)")
            sys.exit(1)

    # Check colony determinism
    determ_log = out / "head_colony_u.log"
    if determ_log.exists() and "DETERMINISM PASS" not in determ_log.read_text():
        print("CORE_FIXTURE FAIL: colony determinism")
        sys.exit(1)

    print("READ_CORE_FIXTURE PASS")


# ── Stall waiver ────────────────────────────────────────────────────────

def stall_waiver(stall_report_path):
    """Verify the M-UX2.1/M6a stall waiver on a stall-report.json.
    - Capture complete
    - At each zoom, ≤2 post-warmup frames > 50 ms and none > 60 ms
    """
    path = Path(stall_report_path)
    assert path.exists(), f"stall report not found: {stall_report_path}"
    report = json.loads(path.read_text())
    assert report.get("complete"), "stall capture not complete"

    waiver_used = False
    for arm in report.get("arms", []):
        zoom = arm.get("zoom", "?")
        assert arm.get("capture_complete"), f"zoom {zoom}: capture not complete"
        # Post-warmup frames: those after render_warmup.complete is true.
        samples = arm.get("samples", [])
        post_warmup = False
        pw_frames = []
        for s in samples:
            warmup = s.get("telemetry", {}).get("buildings", {}).get("render_warmup", {})
            if warmup.get("complete", False):
                post_warmup = True
            if post_warmup:
                pw_frames.append(s.get("interval_ms", 0))
        over_50 = [round(f, 3) for f in pw_frames if f > 50.0]
        over_60 = [round(f, 3) for f in pw_frames if f > 60.0]
        if over_60:
            print(f"STALL FAIL: zoom {zoom} has {len(over_60)} post-warmup frames > 60 ms: {over_60}")
            sys.exit(1)
        if len(over_50) > 2:
            print(f"STALL FAIL: zoom {zoom} has {len(over_50)} post-warmup frames > 50 ms (max 2): {over_50}")
            sys.exit(1)
        if over_50:
            waiver_used = True
            print(f"MUX2_STALL_WAIVER_USED zoom={zoom} frames_over_50ms={over_50}")
        else:
            print(f"STALL zoom={zoom}: {len(pw_frames)} post-warmup frames, max {max(pw_frames):.1f} ms")

    if not waiver_used:
        print("STALL PASS (no waiver needed)")
    else:
        print("STALL PASS (waiver used)")


# ── Bench ratio rule (for unit tests) ──────────────────────────────────

def check_bench_ratio(base_median, new_median, limit=1.10):
    """Return (pass, ratio)."""
    ratio = new_median / base_median if base_median else 0
    return ratio <= limit, ratio


def check_stall_frames(frames, max_over_50=2, max_ms=60.0):
    """Return (pass, over_50_count, max_frame)."""
    over_50 = [f for f in frames if f > 50.0]
    over_60 = [f for f in frames if f > max_ms]
    ok = len(over_50) <= max_over_50 and len(over_60) == 0
    return ok, len(over_50), max(frames) if frames else 0


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    cmd = sys.argv[1]
    if cmd == "bench":
        bench(sys.argv[2], sys.argv[3])
    elif cmd == "core_fixture":
        core_fixture(sys.argv[2], sys.argv[3])
    elif cmd == "stall_waiver":
        stall_waiver(sys.argv[2])
    else:
        print(f"unknown command: {cmd}", file=sys.stderr)
        sys.exit(2)
