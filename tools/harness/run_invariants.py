#!/usr/bin/env python3
"""
Drives the Godot headless harness via WebSocket to verify assertion 7.
Starts Godot headless, connects, advances ticks, runs all 7 invariants, prints results.
"""
import asyncio
import json
import subprocess
import sys
import time
import os

GODOT_BIN = "/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot"
PROJECT_PATH = "/Users/rexxa/github/new-world-wt/lead"
PORT = 9877
TICK_COUNT = 500


async def run_harness():
    sys.path.insert(0, "/Users/rexxa/github/Godot-Rust-MCP/src")
    from godot_ws import GodotWS

    # Start Godot headless
    godot_proc = subprocess.Popen(
        [GODOT_BIN, "--headless", "--path", PROJECT_PATH],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    print(f"[harness] Godot PID={godot_proc.pid}, waiting for WS server...")

    ws = GodotWS(port=PORT)
    connected = await ws.connect_with_retry(timeout=30.0)
    if not connected:
        godot_proc.terminate()
        print("[harness] ERROR: Could not connect to Godot harness WS server")
        sys.exit(1)

    print("[harness] Connected to WS harness")

    # Ping to confirm
    pong = await ws.send("ping", {})
    print(f"[harness] ping → {pong}")

    # Advance ticks
    print(f"[harness] Advancing {TICK_COUNT} simulation ticks...")
    tick_result = await ws.send("tick", {"n": TICK_COUNT})
    print(f"[harness] tick result: {json.dumps(tick_result, indent=2)}")

    # Snapshot to confirm alive count
    snap = await ws.send("snapshot", {})
    print(f"[harness] snapshot: tick={snap.get('tick')}, alive={snap.get('alive')}")

    # Run all 7 invariants
    print("[harness] Running all invariants...")
    inv_result = await ws.send("invariant", {})
    print(f"[harness] invariant result:\n{json.dumps(inv_result, indent=2)}")

    # Summary
    total = inv_result.get("total", 0)
    passed = inv_result.get("passed", 0)
    failed = inv_result.get("failed", 0)
    print(f"\n[harness] === INVARIANT SUMMARY ===")
    print(f"[harness] Total: {total}, Passed: {passed}, Failed: {failed}")
    for r in inv_result.get("results", []):
        status = "PASS" if r.get("passed") else "FAIL"
        print(f"[harness]   {r['name']}: {status}")

    await ws.close()
    godot_proc.terminate()
    try:
        godot_proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        godot_proc.kill()

    if failed == 0:
        print("[harness] ASSERTION 7: PASS — all 7 invariants passed")
        sys.exit(0)
    else:
        print(f"[harness] ASSERTION 7: FAIL — {failed} invariants failed")
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(run_harness())
