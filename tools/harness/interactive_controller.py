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


def _empty_space_click_coords(
    state: dict, agents: list = None, buildings: list = None
) -> tuple:
    """Return a pixel likely to be empty (no agent, no building, no settlement).

    If ``agents`` or ``buildings`` are supplied, pick a viewport location whose
    world-tile does NOT overlap any building footprint (padded by 1 tile to
    cover `entity_renderer._handle_click`'s 3x3 search) and is at least 4 tiles
    from any alive agent. Falls back to the original (12%, 30%) heuristic
    corner if none of the candidate points are clear.
    """
    vp = state.get("viewport_size", [1152, 648])
    camera_pos = state.get("camera_pos", [2048.0, 2048.0])
    zoom = float(state.get("camera_zoom", 1.0)) or 1.0
    cam_x, cam_y = float(camera_pos[0]), float(camera_pos[1])
    agents = agents or []
    buildings = buildings or []

    # Candidate screen coords: a grid across the viewport interior,
    # sorted so we prefer corners (furthest from the central agent cluster).
    candidates = []
    for px_frac in (0.10, 0.08, 0.12, 0.20, 0.28, 0.36, 0.44):
        for py_frac in (0.30, 0.22, 0.18, 0.42, 0.50, 0.62, 0.74):
            candidates.append((vp[0] * px_frac, vp[1] * py_frac))

    def _ok(px: float, py: float) -> bool:
        # Viewport inverse transform: world = camera + (screen - vp/2)/zoom.
        wx = cam_x + (px - vp[0] / 2.0) / zoom
        wy = cam_y + (py - vp[1] / 2.0) / zoom
        tx = int(wx // 16)
        ty = int(wy // 16)
        # Avoid any building footprint expanded by 1 tile (matches the click
        # handler's 3x3 building search).
        for b in buildings:
            bx = int(b.get("tile_x", 0))
            by = int(b.get("tile_y", 0))
            bw = int(b.get("width", 1))
            bh = int(b.get("height", 1))
            if (bx - 1) <= tx <= (bx + bw) and (by - 1) <= ty <= (by + bh):
                return False
        # Avoid pixels within 4 tiles (64 px) of any agent world pos.
        for a in agents:
            awx = float(a.get("world_x", 0.0))
            awy = float(a.get("world_y", 0.0))
            if (wx - awx) ** 2 + (wy - awy) ** 2 < (64.0 * 64.0):
                return False
        return True

    for (px, py) in candidates:
        if _ok(px, py):
            return (px, py)
    # Last-resort fallback (legacy behaviour).
    return (vp[0] * 0.12, vp[1] * 0.30)


def _near_building(wx: float, wy: float, buildings: list, pad_tiles: int = 2) -> bool:
    """Return True if (wx, wy) is within ``pad_tiles`` of any building footprint.

    ``entity_renderer._handle_click`` scans a 3x3 tile area centred on the
    clicked tile for a building before checking entities. A building anywhere
    inside that 3x3 area steals the selection, leaving ``selected_entity_id``
    at -1. We therefore treat an agent as "unclickable" whenever its own tile
    is within 2 tiles of a building footprint, which guarantees the 3x3 search
    centred on the agent's tile will not find one.
    """
    if not buildings:
        return False
    tx = int(wx // 16)
    ty = int(wy // 16)
    for b in buildings:
        bx = int(b.get("tile_x", 0))
        by = int(b.get("tile_y", 0))
        bw = int(b.get("width", 1))
        bh = int(b.get("height", 1))
        if (bx - pad_tiles) <= tx <= (bx + bw + pad_tiles - 1) and (
            by - pad_tiles
        ) <= ty <= (by + bh + pad_tiles - 1):
            return True
    return False


def _choose_agent_near_center(
    agents: list,
    vp_size: tuple,
    avoid_ids: set,
    avoid_agents: list = None,
    buildings: list = None,
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

    If `buildings` is provided, candidates whose tile is within 2 tiles of
    any building footprint are rejected: ``entity_renderer._handle_click``
    checks a 3x3 tile region for a building BEFORE checking entities, so a
    nearby building will steal the selection. This mirrors the prior
    regression where clicking agent#18's pixel selected a building instead.
    """
    cx, cy = vp_size[0] / 2.0, vp_size[1] / 2.0
    # Allow some margin so we don't pick an agent under the HUD sidebar.
    x_min, x_max = 40.0, vp_size[0] * 0.70
    y_min, y_max = 40.0, vp_size[1] - 80.0
    ISOLATION_PIXELS = 80.0  # 5 world-tiles @ 16 px/tile; > 3-tile click radius
    ISOLATION_PIXELS_SQ = ISOLATION_PIXELS * ISOLATION_PIXELS
    avoid_agents = avoid_agents or []
    buildings = buildings or []

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
        # Reject agents adjacent to a building — the click would land on the
        # building instead (see _near_building doc).
        if _near_building(wx, wy, buildings):
            continue
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
    agents: list,
    vp_size: tuple,
    avoid_ids: set,
    avoid_agents: list,
    buildings: list = None,
) -> dict | None:
    """Relaxed variant of `_choose_agent_near_center`: enforces the
    avoided-agent spatial separation AND building avoidance but drops the
    candidate-vs-candidate isolation check. Used as a fall-back when every
    candidate has a nearby neighbour (common at high population densities)."""
    cx, cy = vp_size[0] / 2.0, vp_size[1] / 2.0
    x_min, x_max = 40.0, vp_size[0] * 0.70
    y_min, y_max = 40.0, vp_size[1] - 80.0
    ISOLATION_PIXELS_SQ = 80.0 * 80.0
    buildings = buildings or []

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
        if _near_building(wx, wy, buildings):
            continue
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





def _pick_target(
    agents: list,
    vp_size: tuple,
    avoid_ids: set,
    avoid_positions: list,
    buildings: list,
    must_be_different: bool,
    result: dict,
) -> dict | None:
    """Cascade through the three filter tightness levels and return the
    best candidate, or None if no agent satisfies any tier."""
    target = _choose_agent_near_center(
        agents, vp_size, avoid_ids, avoid_positions, buildings
    )
    if target is None and not must_be_different and _selection_history:
        # Second attempt: allow re-selecting an already-selected agent.
        target = _choose_agent_near_center(agents, vp_size, set(), [], buildings)
    if target is None and must_be_different:
        # Second fall-back attempt: drop the candidate-vs-candidate isolation
        # requirement while keeping the ID + avoided-agent spatial check.
        result["steps_log"].append(
            "retry: relaxing candidate-vs-candidate isolation check"
        )
        target = _choose_agent_near_center_loose(
            agents, vp_size, avoid_ids, avoid_positions, buildings
        )
    if target is None and must_be_different:
        # Third fall-back: also drop the building-adjacency filter.
        result["steps_log"].append(
            "retry: dropping building-adjacency filter (may produce click-steal)"
        )
        target = _choose_agent_near_center_loose(
            agents, vp_size, avoid_ids, avoid_positions, []
        )
    return target


def _click_and_read_selection(
    sock: socket.socket, target: dict, result: dict
) -> dict:
    """Issue the click command for the given target, wait briefly, and
    return the HUD's `get_selected_entity` response.  Both events are
    appended to `result["steps_log"]` for evidence traceability."""
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
    # The Godot side now invokes `_handle_click` synchronously from the
    # `click` command handler, so the selection is already settled by the
    # time this call returns. We keep a tiny wait for backwards
    # compatibility with older Godot builds that fall back to the
    # push_input path.
    send_command(sock, {"action": "wait_frames", "count": 2})
    sel = send_command(sock, {"action": "get_selected_entity"})
    result["steps_log"].append(
        f"after-click selection: entity_id={sel.get('entity_id')}"
        f" building_id={sel.get('selected_building_id', -1)}"
        f" settlement_id={sel.get('selected_settlement_id', -1)}"
        f" name='{sel.get('name')}'"
        f" panel_visible={sel.get('panel_visible')}"
        f" TCI(NS={sel.get('tci_ns'):.3f},HA={sel.get('tci_ha'):.3f},"
        f"RD={sel.get('tci_rd'):.3f},P={sel.get('tci_p'):.3f})"
        f" label={sel.get('temperament_label_key')}"
    )
    return sel


def _perform_agent_click(
    sock: socket.socket, result: dict, must_be_different: bool
) -> dict | None:
    """Query alive agents, pick a target, click its pixel, and record the
    resulting selection.  Returns the target agent dict or None on failure.

    If `must_be_different` is True, the target must differ from any id already
    in `_selection_history`; otherwise any valid agent near center is fine.

    The routine retries up to `MAX_ATTEMPTS` times when a click lands on no
    entity (possible under heavy renderer load) or selects an entity that
    violates the distinct-id constraint. Each retry widens the `avoid_ids`
    set so we don't repeatedly click the same failing candidate.
    """
    MAX_ATTEMPTS = 4

    tried_target_ids: set[int] = set()

    for attempt in range(1, MAX_ATTEMPTS + 1):
        state = send_command(sock, {"action": "get_state"})
        vp_size = tuple(state.get("viewport_size", [1152, 648]))
        agents_resp = send_command(sock, {"action": "get_agents"})
        agents = agents_resp.get("agents", [])
        buildings: list = []
        try:
            b_resp = send_command(sock, {"action": "get_buildings"})
            if isinstance(b_resp, dict):
                b_list = b_resp.get("buildings", [])
                if isinstance(b_list, list):
                    buildings = b_list
        except Exception as exc:
            # Graceful degrade on older servers that lack `get_buildings`.
            result["steps_log"].append(
                f"get_buildings unavailable ({exc}); continuing without building filter"
            )
        result["steps_log"].append(
            f"attempt {attempt}: queried {len(agents)} alive agents,"
            f" {len(buildings)} buildings (vp={vp_size})"
        )
        avoid_ids: set[int] = set(tried_target_ids)
        if must_be_different:
            avoid_ids |= set(_selection_history)
        avoid_positions = list(_selection_positions) if must_be_different else []
        target = _pick_target(
            agents,
            vp_size,
            avoid_ids,
            avoid_positions,
            buildings,
            must_be_different,
            result,
        )
        if target is None:
            result["result"] = "FAIL"
            result["detail"] = (
                f"attempt {attempt}: no valid agent within viewport click region"
                f" (avoided={sorted(avoid_ids)}, total_alive={len(agents)},"
                f" buildings={len(buildings)})"
            )
            # No candidate left — stop retrying.
            return None

        tried_target_ids.add(int(target["id"]))
        sel = _click_and_read_selection(sock, target, result)
        sel_id = int(sel.get("entity_id", -1))
        sel_building = int(sel.get("selected_building_id", -1))
        sel_settlement = int(sel.get("selected_settlement_id", -1))

        # Successful selection path — record evidence and return.
        panel_visible = bool(sel.get("panel_visible", False))
        distinct_ok = (not must_be_different) or (sel_id not in _selection_history)
        if sel_id >= 0 and panel_visible and distinct_ok:
            result.setdefault("tci_samples", []).append(
                {
                    "target_entity_id": int(target["id"]),
                    "selected_entity_id": sel_id,
                    "name": sel.get("name", ""),
                    "tci_ns": sel.get("tci_ns"),
                    "tci_ha": sel.get("tci_ha"),
                    "tci_rd": sel.get("tci_rd"),
                    "tci_p": sel.get("tci_p"),
                    "temperament_label_key": sel.get("temperament_label_key"),
                    "panel_visible": sel.get("panel_visible"),
                }
            )
            _selection_history.append(sel_id)
            _selection_positions.append(
                {
                    "world_x": float(target.get("world_x", 0.0)),
                    "world_y": float(target.get("world_y", 0.0)),
                }
            )
            result["result"] = "PASS"
            result["detail"] = ""
            return target

        # Failure path — log reason, decide whether to retry.
        if sel_id < 0:
            stolen_by = ""
            if sel_building >= 0:
                stolen_by = f" (stolen by building id={sel_building})"
            elif sel_settlement >= 0:
                stolen_by = f" (stolen by settlement id={sel_settlement})"
            reason = (
                f"click at agent#{target['id']} pixel did not select any"
                f" entity{stolen_by}"
            )
        elif must_be_different and sel_id in _selection_history:
            reason = (
                f"selection {sel_id} equals a previously-selected agent"
                f" (history={_selection_history})"
            )
        elif not panel_visible:
            reason = f"entity {sel_id} selected but detail panel not visible"
        else:
            reason = "unknown selection failure"

        result["steps_log"].append(f"attempt {attempt} FAIL: {reason}")
        result["result"] = "FAIL"
        result["detail"] = reason

        # Emit a persistent tci_sample ONLY for the final attempt so the
        # evidence records the last attempted selection (non-PASS path).
        if attempt == MAX_ATTEMPTS:
            result.setdefault("tci_samples", []).append(
                {
                    "target_entity_id": int(target["id"]),
                    "selected_entity_id": sel_id,
                    "name": sel.get("name", ""),
                    "tci_ns": sel.get("tci_ns"),
                    "tci_ha": sel.get("tci_ha"),
                    "tci_rd": sel.get("tci_rd"),
                    "tci_p": sel.get("tci_p"),
                    "temperament_label_key": sel.get("temperament_label_key"),
                    "panel_visible": sel.get("panel_visible"),
                }
            )
            return None

    return None


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
            # Fetch alive agents + building footprints so the empty-space
            # pick avoids landing on either.  A click on a building leaves
            # `_selected_building_id >= 0` which keeps the detail panel open
            # with stale agent data — exactly the regression we hunt here.
            try:
                a_resp = send_command(sock, {"action": "get_agents"})
                agents = a_resp.get("agents", []) if isinstance(a_resp, dict) else []
            except Exception:
                agents = []
            try:
                b_resp = send_command(sock, {"action": "get_buildings"})
                buildings = (
                    b_resp.get("buildings", []) if isinstance(b_resp, dict) else []
                )
            except Exception:
                buildings = []
            x, y = _empty_space_click_coords(state, agents=agents, buildings=buildings)
            resp = send_command(sock, {"action": "click", "x": x, "y": y})
            result["steps_log"].append(
                f"click empty-space ({x:.1f}, {y:.1f}): {resp}"
            )
            # Log selection afterward to confirm deselect.
            send_command(sock, {"action": "wait_frames", "count": 2})
            sel = send_command(sock, {"action": "get_selected_entity"})
            result["steps_log"].append(
                f"after empty click: entity_id={sel.get('entity_id')}"
                f" building_id={sel.get('selected_building_id', -1)}"
                f" settlement_id={sel.get('selected_settlement_id', -1)}"
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
