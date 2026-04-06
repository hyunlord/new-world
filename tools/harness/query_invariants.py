#!/usr/bin/env python3
"""
Connect to the already-running Godot headless harness on port 9877 and run invariants.
Does NOT start a new Godot process.
"""
import asyncio
import json
import sys

sys.path.insert(0, "/Users/rexxa/github/Godot-Rust-MCP/src")

PORT = 9877


async def query_invariants():
    from godot_ws import GodotWS

    ws = GodotWS(port=PORT)
    print(f"[query] Connecting to existing Godot harness on port {PORT}...")
    connected = await ws.connect_with_retry(timeout=10.0)
    if not connected:
        print("[query] ERROR: No Godot harness server found on port 9877")
        sys.exit(1)

    print("[query] Connected!")

    # Ping
    pong = await ws.send("ping", {})
    print(f"[query] ping → tick={pong.get('tick')}")

    # Run all 7 invariants on current simulation state
    print("[query] Running all invariants on current state...")
    inv = await ws.send("invariant", {})
    print(f"[query] Raw result:\n{json.dumps(inv, indent=2)}")

    total = inv.get("total", 0)
    passed = inv.get("passed", 0)
    failed = inv.get("failed", 0)
    print(f"\n[query] === INVARIANT SUMMARY ===")
    print(f"[query] Total={total}, Passed={passed}, Failed={failed}")
    for r in inv.get("results", []):
        status = "PASS" if r.get("passed") else "FAIL"
        vcount = r.get("violation_count", 0)
        print(f"[query]   {r['name']}: {status}" + (f" ({vcount} violations)" if not r.get("passed") else ""))

    await ws.close()

    if failed == 0:
        print("\n[query] ASSERTION 7: PASS — all invariants passed")
        sys.exit(0)
    else:
        print(f"\n[query] ASSERTION 7: FAIL — {failed}/{total} invariants failed")
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(query_invariants())
