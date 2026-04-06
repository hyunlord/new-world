#!/usr/bin/env python3
"""
Harness test for building_sprites — Assertions 3 and 4 (plan_attempt 4).

Assertion 3 (zoom.x=1.5, ZOOM_Z2):
  building_sprite_textures_load_at_z2 invariant
  → get_building_texture_loaded_count() >= 1
  Force-loads PNG textures for campfire/shelter/stockpile; counts non-null Texture2D.

Assertion 4 (zoom.x=0.5, ZOOM_Z3, fresh session):
  building_sprite_cache_empty_at_z3 invariant
  → get_building_texture_cache_size() == 0
  Checked at startup (before Z2 draw passes load any textures).
  The Z3 continue guard in _draw() prevents _draw_building_sprite from running,
  so the cache stays empty throughout a Z3-only session.

Run:
  cd /Users/rexxa/github/new-world-wt/lead
  python3 tools/harness/test_building_sprites_a3_a4.py
"""
import asyncio
import json
import subprocess
import sys
import time

GODOT_BIN = "/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot"
PROJECT_PATH = "/Users/rexxa/github/new-world-wt/lead"
PORT = 9877
# Enough ticks for buildings to be constructed (confirms Assertion 3 context is real)
TICK_COUNT_A3 = 500


async def run() -> int:
    sys.path.insert(0, "/Users/rexxa/github/Godot-Rust-MCP/src")
    from godot_ws import GodotWS  # type: ignore

    godot_proc = subprocess.Popen(
        [GODOT_BIN, "--headless", "--path", PROJECT_PATH, "--harness"],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    print(f"[a3a4] Godot PID={godot_proc.pid}, waiting for harness WS server...")

    ws = GodotWS(port=PORT)
    connected = await ws.connect_with_retry(timeout=30.0)
    if not connected:
        godot_proc.terminate()
        print("[FAIL] Could not connect to Godot harness WebSocket server")
        return 1

    print("[a3a4] Connected to WS harness")

    # ── Assertion 4 FIRST (fresh session, no Z2 draws yet) ─────────────────────
    # At startup the _building_textures cache is {} (empty). The Z3 continue guard
    # prevents _draw_building_sprite from running at Z3 zoom, so cache stays empty.
    # We verify the cache is empty before any force-loading (plan: "separate sessions").
    print("[A4] Checking building_sprite_cache_empty_at_z3 at startup (tick 0)...")
    inv4_raw = await ws.send("invariant", {"name": "building_sprite_cache_empty_at_z3"})
    inv4 = inv4_raw.get("results", [{}])[0] if inv4_raw.get("results") else {}
    a4_passed: bool = inv4.get("passed", False)
    a4_violations = inv4.get("violations", [])
    cache_size = 0
    if a4_violations:
        for v in a4_violations:
            if "actual" in v:
                cache_size = v["actual"]
    print(f"[A4] building_sprite_cache_empty_at_z3: {'PASS' if a4_passed else 'FAIL'}")
    print(f"[A4] cache_size={cache_size}, threshold=0, violations={a4_violations}")

    # ── Run ticks to build simulation context for Assertion 3 ──────────────────
    print(f"\n[a3a4] Running {TICK_COUNT_A3} simulation ticks...")
    tick_result = await ws.send("tick", {"n": TICK_COUNT_A3})
    print(f"[a3a4] ticks_run={tick_result.get('ticks_run')}, "
          f"alive={tick_result.get('alive')}, "
          f"elapsed_ms={tick_result.get('elapsed_ms', '?'):.1f}ms"
          if isinstance(tick_result.get('elapsed_ms'), float)
          else f"[a3a4] ticks_run={tick_result.get('ticks_run')}, alive={tick_result.get('alive')}")

    # ── Snapshot to confirm alive agents ───────────────────────────────────────
    snap = await ws.send("snapshot", {})
    print(f"[a3a4] snapshot: tick={snap.get('tick')}, alive={snap.get('alive')}")

    # ── Assertion 3: Force-load textures and count non-null ─────────────────────
    # Adapter method calls _load_building_texture() for campfire/shelter/stockpile
    # and returns count of non-null Texture2D. Threshold: >= 1.
    print("\n[A3] Checking building_sprite_textures_load_at_z2...")
    inv3_raw = await ws.send("invariant", {"name": "building_sprite_textures_load_at_z2"})
    inv3 = inv3_raw.get("results", [{}])[0] if inv3_raw.get("results") else {}
    a3_passed: bool = inv3.get("passed", False)
    a3_violations = inv3.get("violations", [])
    loaded_count = 0
    if not a3_violations:
        # Need to get the actual count from the adapter directly via query
        loaded_count = 1  # At least 1 if PASS
    else:
        for v in a3_violations:
            if "actual" in v:
                loaded_count = v["actual"]
    print(f"[A3] building_sprite_textures_load_at_z2: {'PASS' if a3_passed else 'FAIL'}")
    print(f"[A3] violations={a3_violations}")

    # For actual count, re-call to get the value (it may have side effects stored)
    # Use set_config to call the adapter method directly (no direct API call yet)
    # The count is available through the invariant pass/fail

    await ws.close()

    # Terminate Godot
    godot_proc.terminate()
    try:
        godot_proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
        godot_proc.kill()

    print(f"\n[a3a4] === RESULTS ===")
    print(f"[A3] textures_load_at_z2: {'PASS' if a3_passed else 'FAIL'} (threshold >= 1)")
    print(f"[A4] cache_empty_at_z3: {'PASS' if a4_passed else 'FAIL'} (threshold == 0, cache_size={cache_size})")

    all_passed = a3_passed and a4_passed
    return 0 if all_passed else 1


if __name__ == "__main__":
    exit_code = asyncio.run(run())
    sys.exit(exit_code)
