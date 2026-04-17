#!/usr/bin/env python3
"""Interactive controller for WorldSim harness visual verification.

Connects to Godot's TCP command server and executes test scenarios.
Protocol: newline-delimited JSON over TCP (port 9223).
No external dependencies — uses only Python stdlib.

Scenario step parsing rules (explicit > implicit):
- Numbered steps beginning with "Set zoom to Z<N>" drive zoom.
- Numbered steps beginning with "Wait <N> ticks" / "Wait <N> frames" drive timing.
- Numbered steps containing "Screenshot:" AND a quoted label capture a labeled screenshot.
- Numbered steps containing "Click on an agent" / "Click a different agent"
  query live agent snapshots and click a REAL agent's pixel (different agent
  on the second call). The selected entity id is captured after the click
  via `get_selected_entity` and logged.
- Numbered steps containing "Click empty space" click a corner pixel known to
  be empty of agents / buildings, so the detail panel is dismissed.
- Numbered steps containing "personality tab" OR "temperament tab" call
  `click_tab` on the detail panel directly (tab index 3).

**Any numbered step that does not match a known pattern causes the scenario
to FAIL.** Silent skipping hides broken scenarios (past regression).
"""

import argparse
import json
import os
import re
import socket
import sys

# Stable mapping from panel title to tab index (matches v5 _build_tab_bar()).
TAB_INDEX_OVERVIEW = 0
TAB_INDEX_NEEDS = 1
TAB_INDEX_EMOTION = 2
TAB_INDEX_PERSONALITY = 3

# Shared across scenarios so Scenario 3 can pick a different agent than Scenario 1.
_selection_history: list = []
# Parallel to _selection_history: world position of each selected agent at
# time of click, used for spatial isolation checks in later scenarios.
_selection_positions: list = []


def send_command(sock: socket.socket, cmd: dict) -> dict:
    """Send a JSON command and receive the response (newline-delimited)."""
    msg = json.dumps(cmd) + "\n"
    sock.sendall(msg.encode("utf-8"))
    buf = b""
    while b"\n" not in buf:
        chunk = sock.recv(8192)
        if not chunk:
            raise ConnectionError("Server disconnected")
        buf += chunk
    line = buf.split(b"\n")[0]
    return json.loads(line.decode("utf-8"))


def parse_scenarios(text: str) -> list:
    """Parse interactive scenarios from markdown format."""
    scenarios = []
    current = None

    for line in text.split("\n"):
        if line.startswith("### Scenario"):
            if current:
                scenarios.append(current)
            name = line.replace("### ", "").strip()
            current = {"name": name, "steps": [], "expected": ""}
        elif current is not None and line.startswith("Expected:"):
            current["expected"] = line.replace("Expected:", "").strip()
        elif current is not None and re.match(r"^\d+\.", line.strip()):
            current["steps"].append(line.strip())

    if current:
        scenarios.append(current)

    return scenarios


def _parse_screenshot_label(step: str) -> str | None:
    """Extract screenshot label from e.g. `3. Screenshot: "panel_opened"`."""
    # Match Screenshot: "label"  (either quote style)
    m = re.search(r'[Ss]creenshot\s*:\s*["\']([A-Za-z0-9_\-]+)["\']', step)
    if m:
        return m.group(1)
    # Fallback: Screenshot label without quotes, single token
    m = re.search(r"[Ss]creenshot\s*:\s*([A-Za-z0-9_\-]+)", step)
    if m:
        return m.group(1)
    return None


def _center_of_viewport(state: dict) -> tuple:
    vp = state.get("viewport_size", [1152, 648])
    return (vp[0] / 2.0, vp[1] / 2.0)


def _empty_space_click_coords(state: dict) -> tuple:
    """Return a pixel likely to be empty (upper-right quadrant just below HUD)."""
    vp = state.get("viewport_size", [1152, 648])
    # HUD sidebar is on the right; empty space is well inside the viewport
    # but to the left of the sidebar.  Use 12% in from the left, 30% down.
    return (vp[0] * 0.12, vp[1] * 0.30)


def _choose_agent_near_center(
    agents: list, vp_size: tuple, avoid_ids: set, avoid_agents: list = None
) -> dict | None:
    """Pick the alive agent whose screen coords are closest to viewport center,
    skipping any id in `avoid_ids` and any agent rendered outside the viewport.

    If `avoid_agents` is provided, each candidate must ALSO be spatially
    isolated from every avoided agent: the click radius is 3 world-tiles
    (48 pixels at 1.0 zoom, renderer.gd `best_dist = 3.0`), so a candidate
    whose world position is within ~4 tiles of any avoided agent risks
    having the click snap onto the avoided agent instead. The isolation
    check enforces a generous 5-tile (80-pixel) minimum separation in
    world-pixel space.

    Additionally, the candidate must be isolated from ALL OTHER candidates
    in the returned agent list — otherwise the click resolves onto whichever
    of the cluster is closest to the cursor and we cannot predict which.
    """
    cx, cy = vp_size[0] / 2.0, vp_size[1] / 2.0
    # Allow some margin so we don't pick an agent under the HUD sidebar.
    x_min, x_max = 40.0, vp_size[0] * 0.70
    y_min, y_max = 40.0, vp_size[1] - 80.0
    ISOLATION_PIXELS = 80.0  # 5 world-tiles @ 16 px/tile; > 3-tile click radius
    ISOLATION_PIXELS_SQ = ISOLATION_PIXELS * ISOLATION_PIXELS
    avoid_agents = avoid_agents or []

    # Build a fast lookup of (world_x, world_y) for every alive candidate so
    # we can enforce candidate-vs-candidate isolation too.
    all_positions = [
        (float(a.get("world_x", 0.0)), float(a.get("world_y", 0.0)), int(a.get("id", -1)))
        for a in agents
    ]

    best = None
    best_dist = float("inf")
    for a in agents:
        aid = int(a.get("id", -1))
        if aid < 0 or aid in avoid_ids:
            continue
        sx = float(a.get("screen_x", -1))
        sy = float(a.get("screen_y", -1))
        if not (x_min <= sx <= x_max and y_min <= sy <= y_max):
            continue
        wx = float(a.get("world_x", 0.0))
        wy = float(a.get("world_y", 0.0))
        # Isolate from previously-clicked agents (defeats ID-mismatch-but-
        # cluster-overlap failure: clicking near agent X could snap onto
        # agent Y = the prior selection because it happens to be closer in
        # world space).
        too_close_to_avoided = False
        for avoid in avoid_agents:
            awx = float(avoid.get("world_x", 0.0))
            awy = float(avoid.get("world_y", 0.0))
            dsq = (wx - awx) ** 2 + (wy - awy) ** 2
            if dsq < ISOLATION_PIXELS_SQ:
                too_close_to_avoided = True
                break
        if too_close_to_avoided:
            continue
        # Candidate-vs-candidate isolation: the click must unambiguously
        # resolve to this agent, so no OTHER alive candidate may sit within
        # one click-radius (~3 tiles = 48 px).
        conflicts = 0
        for (owx, owy, oid) in all_positions:
            if oid == aid:
                continue
            dsq = (wx - owx) ** 2 + (wy - owy) ** 2
            if dsq < (48.0 * 48.0):
                conflicts += 1
                if conflicts > 0:
                    break
        if conflicts > 0:
            continue
        d = (sx - cx) ** 2 + (sy - cy) ** 2
        if d < best_dist:
            best_dist = d
            best = a
    return best


def _choose_agent_near_center_loose(
    agents: list, vp_size: tuple, avoid_ids: set, avoid_agents: list
) -> dict | None:
    """Relaxed variant of `_choose_agent_near_center`: enforces the
    avoided-agent spatial separation but drops the candidate-vs-candidate
    isolation check. Used as a fall-back when every candidate has a nearby
    neighbour (common at high population densities)."""
    cx, cy = vp_size[0] / 2.0, vp_size[1] / 2.0
    x_min, x_max = 40.0, vp_size[0] * 0.70
    y_min, y_max = 40.0, vp_size[1] - 80.0
    ISOLATION_PIXELS_SQ = 80.0 * 80.0

    best = None
    best_dist = float("inf")
    for a in agents:
        aid = int(a.get("id", -1))
        if aid < 0 or aid in avoid_ids:
            continue
        sx = float(a.get("screen_x", -1))
        sy = float(a.get("screen_y", -1))
        if not (x_min <= sx <= x_max and y_min <= sy <= y_max):
            continue
        wx = float(a.get("world_x", 0.0))
        wy = float(a.get("world_y", 0.0))
        too_close_to_avoided = False
        for avoid in avoid_agents:
            awx = float(avoid.get("world_x", 0.0))
            awy = float(avoid.get("world_y", 0.0))
            dsq = (wx - awx) ** 2 + (wy - awy) ** 2
            if dsq < ISOLATION_PIXELS_SQ:
                too_close_to_avoided = True
                break
        if too_close_to_avoided:
            continue
        d = (sx - cx) ** 2 + (sy - cy) ** 2
        if d < best_dist:
            best_dist = d
            best = a
    return best


def _perform_agent_click(
    sock: socket.socket, result: dict, must_be_different: bool
) -> dict | None:
    """Query alive agents, pick a target, click its pixel, and record the
    resulting selection.  Returns the target agent dict or None on failure.

    If `must_be_different` is True, the target must differ from any id already
    in `_selection_history`; otherwise any valid agent near center is fine.
    """
    state = send_command(sock, {"action": "get_state"})
    vp_size = tuple(state.get("viewport_size", [1152, 648]))
    agents_resp = send_command(sock, {"action": "get_agents"})
    agents = agents_resp.get("agents", [])
    result["steps_log"].append(
        f"queried {len(agents)} alive agents (vp={vp_size})"
    )
    avoid_ids = set(_selection_history) if must_be_different else set()
    avoid_positions = list(_selection_positions) if must_be_different else []
    target = _choose_agent_near_center(
        agents, vp_size, avoid_ids, avoid_positions
    )
    if target is None and not must_be_different and _selection_history:
        # Second attempt: allow re-selecting an already-selected agent
        target = _choose_agent_near_center(agents, vp_size, set(), [])
    if target is None and must_be_different:
        # Second fall-back attempt: drop the candidate-vs-candidate isolation
        # requirement while keeping the ID + avoided-agent spatial check. At
        # very high populations every agent may have a near-neighbour, but we
        # still MUST be far from any previously-selected agent to satisfy the
        # plan's distinct-id requirement.
        result["steps_log"].append(
            "retry: relaxing candidate-vs-candidate isolation check"
        )
        target = _choose_agent_near_center_loose(
            agents, vp_size, avoid_ids, avoid_positions
        )
    if target is None:
        result["result"] = "FAIL"
        result["detail"] = (
            "no valid agent within viewport click region "
            f"(avoided={sorted(avoid_ids)}, total_alive={len(agents)})"
        )
        return None

    click_resp = send_command(
        sock,
        {
            "action": "click",
            "x": float(target["screen_x"]),
            "y": float(target["screen_y"]),
        },
    )
    result["steps_log"].append(
        f"click target=agent#{target['id']} at ({target['screen_x']:.1f},"
        f" {target['screen_y']:.1f}) world=({target['world_x']:.1f},"
        f" {target['world_y']:.1f}) resp={click_resp}"
    )

    # Give the UI one frame to react, then read the HUD selection back.
    send_command(sock, {"action": "wait_frames", "count": 2})
    sel = send_command(sock, {"action": "get_selected_entity"})
    result["steps_log"].append(
        f"after-click selection: entity_id={sel.get('entity_id')}"
        f" name='{sel.get('name')}'"
        f" panel_visible={sel.get('panel_visible')}"
        f" TCI(NS={sel.get('tci_ns'):.3f},HA={sel.get('tci_ha'):.3f},"
        f"RD={sel.get('tci_rd'):.3f},P={sel.get('tci_p'):.3f})"
        f" label={sel.get('temperament_label_key')}"
    )
    result.setdefault("tci_samples", []).append(
        {
            "target_entity_id": int(target["id"]),
            "selected_entity_id": int(sel.get("entity_id", -1)),
            "name": sel.get("name", ""),
            "tci_ns": sel.get("tci_ns"),
            "tci_ha": sel.get("tci_ha"),
            "tci_rd": sel.get("tci_rd"),
            "tci_p": sel.get("tci_p"),
            "temperament_label_key": sel.get("temperament_label_key"),
            "panel_visible": sel.get("panel_visible"),
        }
    )

    sel_id = int(sel.get("entity_id", -1))
    if sel_id < 0:
        result["result"] = "FAIL"
        result["detail"] = (
            f"click at agent#{target['id']} pixel did not select any entity"
        )
        return None
    if must_be_different and sel_id in _selection_history:
        result["result"] = "FAIL"
        result["detail"] = (
            f"selection {sel_id} equals a previously-selected agent"
            f" (history={_selection_history})"
        )
        return None
    if not sel.get("panel_visible"):
        result["result"] = "FAIL"
        result["detail"] = (
            f"entity {sel_id} selected but detail panel not visible"
        )
        return None
    _selection_history.append(sel_id)
    _selection_positions.append(
        {
            "world_x": float(target.get("world_x", 0.0)),
            "world_y": float(target.get("world_y", 0.0)),
        }
    )
    return target


def execute_step(sock: socket.socket, step: str, result: dict) -> None:
    """Execute a single numbered step.  Raises RuntimeError on unrecognized
    steps so the scenario fails rather than silently skipping."""
    step_lower = step.lower()

    # --- Zoom commands -----------------------------------------------------
    zoom_match = re.search(r"(?:set\s+)?zoom\s+(?:to\s+)?z(\d+)", step_lower)
    if zoom_match is None:
        # Also accept "zoom level 3.0" or "zoom 3.0"
        zoom_match = re.search(r"zoom\s+(?:level\s+)?(\d+(?:\.\d+)?)", step_lower)
    if zoom_match and "click" not in step_lower:
        token = zoom_match.group(1)
        # Map ZN → concrete zoom multiplier (matches WorldSim camera stages).
        z_map = {"1": 5.0, "2": 3.0, "3": 1.5, "4": 0.75, "5": 0.4}
        if token in z_map and "z" in zoom_match.group(0).lower():
            level = z_map[token]
        else:
            level = float(token)
        resp = send_command(sock, {"action": "zoom", "level": level})
        result["steps_log"].append(f"zoom {level} (from token '{token}'): {resp}")
        return

    # --- Wait commands (must come before click so "wait X frames" wins) ----
    if re.search(r"\bwait\b", step_lower) or "대기" in step_lower:
        # Extract the count AFTER the "wait" keyword so we don't accidentally
        # pick up the leading step number (e.g. "2. Wait 200 ticks").
        num_match = re.search(r"(?:wait|대기)\s+(\d+)", step_lower)
        count = int(num_match.group(1)) if num_match else 5
        if "tick" in step_lower:
            resp = send_command(sock, {"action": "wait_ticks", "count": count})
        else:
            resp = send_command(sock, {"action": "wait_frames", "count": count})
        result["steps_log"].append(f"wait {count}: {resp}")
        return

    # --- Labeled screenshot ------------------------------------------------
    label = _parse_screenshot_label(step)
    if label is not None:
        resp = send_command(sock, {"action": "screenshot", "label": label})
        result["steps_log"].append(f"screenshot '{label}': {resp}")
        return

    # --- Tab navigation ----------------------------------------------------
    if ("personality tab" in step_lower
            or "temperament tab" in step_lower
            or ("personality" in step_lower and "tab" in step_lower)
            or ("navigate" in step_lower and "personality" in step_lower)):
        resp = send_command(
            sock, {"action": "click_tab", "tab_index": TAB_INDEX_PERSONALITY}
        )
        result["steps_log"].append(f"click_tab personality (idx=3): {resp}")
        if not resp.get("ok", False):
            raise RuntimeError("click_tab did not succeed — panel or tab missing")
        return

    # --- Click commands ----------------------------------------------------
    if "click" in step_lower or "클릭" in step_lower:
        # Explicit coordinates take precedence.
        coord_match = re.search(r"\((\d+(?:\.\d+)?),\s*(\d+(?:\.\d+)?)\)", step)
        if coord_match:
            x, y = float(coord_match.group(1)), float(coord_match.group(2))
            resp = send_command(sock, {"action": "click", "x": x, "y": y})
            result["steps_log"].append(f"click ({x}, {y}): {resp}")
            return

        # "Click empty space" — close any detail panel.
        if ("empty space" in step_lower or "empty area" in step_lower
                or "close panel" in step_lower):
            state = send_command(sock, {"action": "get_state"})
            x, y = _empty_space_click_coords(state)
            resp = send_command(sock, {"action": "click", "x": x, "y": y})
            result["steps_log"].append(
                f"click empty-space ({x:.1f}, {y:.1f}): {resp}"
            )
            # Log selection afterward to confirm deselect.
            send_command(sock, {"action": "wait_frames", "count": 2})
            sel = send_command(sock, {"action": "get_selected_entity"})
            result["steps_log"].append(
                f"after empty click: entity_id={sel.get('entity_id')}"
                f" panel_visible={sel.get('panel_visible')}"
            )
            return

        # "Click a different agent" — explicit diff-selection path.
        if "different" in step_lower:
            _perform_agent_click(sock, result, must_be_different=True)
            return

        # "Click on an agent" / generic agent click.
        if "agent" in step_lower:
            _perform_agent_click(sock, result, must_be_different=False)
            return

        # Plain "click" with no context → fail loudly.
        raise RuntimeError(f"ambiguous click step: {step!r}")

    # --- Review / screenshot-review steps ---------------------------------
    if ("review all screenshots" in step_lower
            or "review screenshots" in step_lower
            or step_lower.startswith("review ")):
        # Scenario 4 is a meta-review step. Re-capture a final summary
        # screenshot for the VLM so it can cross-check locale keys.
        resp = send_command(
            sock, {"action": "screenshot", "label": "review_final"}
        )
        result["steps_log"].append(f"review screenshot: {resp}")
        return

    # Verify/check commands — take a verification screenshot with safe name.
    if any(kw in step_lower for kw in ["verify", "check", "확인", "검증"]):
        label_safe = re.sub(r"[^A-Za-z0-9_]+", "_", step)[:40]
        resp = send_command(sock, {"action": "screenshot", "label": f"verify_{label_safe}"})
        result["steps_log"].append(f"verify screenshot: {resp}")
        return

    # --- Anything else is a FAILURE ---------------------------------------
    raise RuntimeError(f"unrecognized step: {step!r}")


def execute_scenario(sock: socket.socket, scenario: dict) -> dict:
    """Execute a single test scenario and return results.

    Unrecognized steps cause the scenario to fail (this used to be a silent
    skip — see prior harness bug where Scenario 1 steps were ignored)."""
    name = scenario["name"]
    result = {
        "name": name,
        "steps_log": [],
        "result": "PASS",
        "detail": "",
        "tci_samples": [],
    }

    for step in scenario["steps"]:
        try:
            execute_step(sock, step, result)
        except RuntimeError as exc:
            result["result"] = "FAIL"
            result["detail"] = str(exc)
            result["steps_log"].append(f"FAIL: {exc}")
            break
        except Exception as exc:
            result["result"] = "FAIL"
            result["detail"] = f"exception: {exc}"
            result["steps_log"].append(f"EXC: {exc}")
            break

        # Short-circuit if a step already marked the scenario failed
        # (e.g. _perform_agent_click set FAIL without raising).
        if result["result"] == "FAIL":
            break

    return result


def _compute_cross_scenario_tci_delta(all_results: list) -> dict:
    """Extract TCI samples across scenarios, compute max axis delta."""
    samples = []
    for r in all_results:
        for s in r.get("tci_samples", []):
            # Only use samples that actually opened the panel on a valid id
            if int(s.get("selected_entity_id", -1)) < 0:
                continue
            ns = s.get("tci_ns")
            ha = s.get("tci_ha")
            rd = s.get("tci_rd")
            p = s.get("tci_p")
            if None in (ns, ha, rd, p):
                continue
            if any(float(v) < 0 for v in (ns, ha, rd, p)):
                continue
            samples.append(s)
    if len(samples) < 2:
        return {
            "available": False,
            "sample_count": len(samples),
            "max_axis_delta_pp": 0.0,
            "threshold_pp": 10.0,
            "threshold_met": False,
            "pair": [],
        }
    # Use first two samples with different selected_entity_id; else just the
    # first pair.
    pair = None
    for i in range(len(samples)):
        for j in range(i + 1, len(samples)):
            if samples[i]["selected_entity_id"] != samples[j]["selected_entity_id"]:
                pair = (samples[i], samples[j])
                break
        if pair:
            break
    if pair is None:
        pair = (samples[0], samples[1])
    a, b = pair
    deltas_pp = {
        "NS": abs(float(a["tci_ns"]) - float(b["tci_ns"])) * 100.0,
        "HA": abs(float(a["tci_ha"]) - float(b["tci_ha"])) * 100.0,
        "RD": abs(float(a["tci_rd"]) - float(b["tci_rd"])) * 100.0,
        "P": abs(float(a["tci_p"]) - float(b["tci_p"])) * 100.0,
    }
    max_pp = max(deltas_pp.values())
    return {
        "available": True,
        "sample_count": len(samples),
        "max_axis_delta_pp": max_pp,
        "per_axis_pp": deltas_pp,
        "threshold_pp": 10.0,
        "threshold_met": max_pp >= 10.0,
        "pair": [
            {
                "entity_id": int(a["selected_entity_id"]),
                "tci": {
                    "NS": a["tci_ns"],
                    "HA": a["tci_ha"],
                    "RD": a["tci_rd"],
                    "P": a["tci_p"],
                },
                "label": a.get("temperament_label_key"),
            },
            {
                "entity_id": int(b["selected_entity_id"]),
                "tci": {
                    "NS": b["tci_ns"],
                    "HA": b["tci_ha"],
                    "RD": b["tci_rd"],
                    "P": b["tci_p"],
                },
                "label": b.get("temperament_label_key"),
            },
        ],
    }


def main():
    parser = argparse.ArgumentParser(description="WorldSim interactive test controller")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=9223)
    parser.add_argument("--evidence-dir", required=True)
    parser.add_argument("--scenarios", required=True, help="Path to scenarios markdown file")
    args = parser.parse_args()

    # Read and parse scenarios
    with open(args.scenarios) as f:
        scenario_text = f.read()

    scenarios = parse_scenarios(scenario_text)
    if not scenarios:
        print("No scenarios found in input file")
        sys.exit(1)

    print(f"Parsed {len(scenarios)} scenario(s)")

    # Connect to Godot command server
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(20.0)
    try:
        sock.connect((args.host, args.port))
    except ConnectionRefusedError:
        print(f"ERROR: Could not connect to {args.host}:{args.port}")
        sys.exit(1)
    print(f"Connected to {args.host}:{args.port}")

    # Get initial state
    state = send_command(sock, {"action": "get_state"})
    print(f"Initial state: {state}")

    # Execute scenarios
    all_results = []
    overall_pass = True
    for scenario in scenarios:
        print(f"\n--- Executing: {scenario['name']} ---")
        try:
            result = execute_scenario(sock, scenario)
        except Exception as e:
            result = {
                "name": scenario["name"],
                "steps_log": [f"ERROR: {e}"],
                "result": "FAIL",
                "detail": str(e),
                "tci_samples": [],
            }
        all_results.append(result)
        if result["result"] != "PASS":
            overall_pass = False
        print(f"Result: {result['result']} ({result.get('detail', '')})")
        for log_line in result["steps_log"]:
            print(f"  {log_line}")

    cross = _compute_cross_scenario_tci_delta(all_results)

    # Write human-readable results
    output_path = os.path.join(args.evidence_dir, "interactive_results.txt")
    with open(output_path, "w") as f:
        for r in all_results:
            f.write(f"SCENARIO: {r['name']}\n")
            f.write(f"RESULT: {r['result']}\n")
            f.write(f"DETAIL: {r.get('detail', '')}\n")
            f.write("STEPS:\n")
            for log_line in r["steps_log"]:
                f.write(f"  - {log_line}\n")
            if r.get("tci_samples"):
                f.write("TCI_SAMPLES:\n")
                for s in r["tci_samples"]:
                    f.write(f"  - {json.dumps(s)}\n")
            f.write("\n")
        f.write("CROSS_SCENARIO_TCI_DELTA:\n")
        f.write(f"  {json.dumps(cross, indent=2)}\n")
        f.write(f"OVERALL: {'PASS' if overall_pass else 'FAIL'}\n")

    # Also write a structured JSON summary for programmatic consumers
    json_path = os.path.join(args.evidence_dir, "interactive_results.json")
    with open(json_path, "w") as f:
        json.dump(
            {
                "scenarios": all_results,
                "cross_scenario_tci_delta": cross,
                "overall_pass": overall_pass,
            },
            f,
            indent=2,
        )

    print(f"\nResults written to: {output_path}")
    print(f"JSON summary: {json_path}")
    print(f"Cross-scenario max TCI delta: {cross.get('max_axis_delta_pp', 0.0):.2f}pp"
          f" (threshold 10.0pp, met={cross.get('threshold_met', False)})")
    print(f"OVERALL: {'PASS' if overall_pass else 'FAIL'}")

    # Tell Godot to quit
    try:
        send_command(sock, {"action": "quit"})
    except Exception:
        pass
    sock.close()

    if not overall_pass:
        sys.exit(2)


if __name__ == "__main__":
    main()
