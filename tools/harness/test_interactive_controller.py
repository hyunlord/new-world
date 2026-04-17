#!/usr/bin/env python3
"""Unit tests for interactive_controller.py defect fixes.
Run with: python3 tools/harness/test_interactive_controller.py
No pytest dependency — uses plain assertions and exit code.

Tests the three defects identified by the Evaluator in the 8th pipeline run:
  Defect 1: Unrecognized steps must fail the scenario (not silently skip).
  Defect 2: Agent selection uses real agent positions (not viewport center).
  Defect 3: "Different agent" selection excludes previously-clicked agents.
"""
import sys
import os

sys.path.insert(0, os.path.dirname(__file__))
from interactive_controller import (  # noqa: E402
    _choose_agent_near_center,
    _choose_agent_near_center_loose,
    execute_step,
)


def _make_agent(aid, sx, sy, wx=0.0, wy=0.0):
    """Create a minimal agent dict matching get_agents response format."""
    return {
        "id": aid,
        "screen_x": sx,
        "screen_y": sy,
        "world_x": wx,
        "world_y": wy,
    }


def test_defect1_unrecognized_step_raises():
    """Defect 1: Unrecognized steps must raise RuntimeError, not silently skip."""
    result = {"steps_log": [], "result": "PASS", "detail": "", "tci_samples": []}
    try:
        execute_step(None, "99. Do something completely unknown", result)
        assert False, "Should have raised RuntimeError"
    except RuntimeError as e:
        assert "unrecognized" in str(e).lower(), f"Error should mention 'unrecognized': {e}"
    print("  Confirmed: unrecognized step raises RuntimeError")


def test_defect2_selects_nearest_to_center():
    """Defect 2: Agent selection picks the agent closest to viewport center."""
    vp = (1000, 600)
    center_x, center_y = 500, 300
    agents = [
        _make_agent(1, 100, 100, 100, 100),  # far from center
        _make_agent(2, 490, 290, 200, 200),  # near center
        _make_agent(3, 400, 250, 300, 300),  # mid distance
    ]
    best = _choose_agent_near_center(agents, vp, set())
    assert best is not None, "Should find an agent"
    assert best["id"] == 2, f"Should pick agent closest to center (id=2), got {best['id']}"


def test_defect2_skips_offscreen_agents():
    """Defect 2: Agents outside viewport bounds are not selectable."""
    vp = (1000, 600)
    agents = [
        _make_agent(1, -50, 300, 0, 0),   # off-screen left
        _make_agent(2, 1100, 300, 0, 0),   # off-screen right (but also beyond 0.70 cutoff)
        _make_agent(3, 500, -10, 0, 0),    # off-screen top
    ]
    best = _choose_agent_near_center(agents, vp, set())
    assert best is None, "No agent should be selectable (all off-screen)"


def test_defect3_avoid_ids_excludes_previous():
    """Defect 3: avoid_ids parameter excludes previously-clicked agents."""
    vp = (1000, 600)
    agents = [
        _make_agent(1, 490, 290, 100, 100),  # closest to center
        _make_agent(2, 480, 280, 200, 200),  # second closest
        _make_agent(3, 300, 200, 500, 500),  # farther (well-separated in world space)
    ]
    # First: pick without avoidance
    best1 = _choose_agent_near_center(agents, vp, set())
    assert best1 is not None and best1["id"] == 1, "First pick should be agent 1 (closest)"

    # Second: avoid agent 1, should pick something else
    # Note: agent 2 may be too close to agent 1 in world space (isolation check),
    # so the loose version is the fallback
    best2 = _choose_agent_near_center(agents, vp, {1}, [{"world_x": 100, "world_y": 100}])
    if best2 is None:
        # Strict isolation may reject agent 2 because it's within 80px of agent 1
        # Try the loose fallback
        best2 = _choose_agent_near_center_loose(
            agents, vp, {1}, [{"world_x": 100, "world_y": 100}]
        )
    assert best2 is not None, "Should find a different agent after excluding id=1"
    assert best2["id"] != 1, f"Should not re-select agent 1, got {best2['id']}"


def test_defect3_no_valid_agent_returns_none():
    """Defect 3: If all agents are excluded, returns None (triggers FAIL)."""
    vp = (1000, 600)
    agents = [
        _make_agent(1, 490, 290, 100, 100),
    ]
    best = _choose_agent_near_center(agents, vp, {1})
    assert best is None, "Should return None when only agent is excluded"


def test_negative_id_skipped():
    """Agent with id < 0 is never selected."""
    vp = (1000, 600)
    agents = [
        _make_agent(-1, 500, 300, 0, 0),
    ]
    best = _choose_agent_near_center(agents, vp, set())
    assert best is None, "Agent with negative id should be skipped"


def test_empty_agents_returns_none():
    """Empty agent list returns None."""
    vp = (1000, 600)
    best = _choose_agent_near_center([], vp, set())
    assert best is None, "Empty agent list should return None"


if __name__ == "__main__":
    tests = [v for k, v in list(globals().items()) if k.startswith("test_") and callable(v)]
    failed = 0
    for t in tests:
        try:
            t()
            print(f"PASS: {t.__name__}")
        except AssertionError as e:
            print(f"FAIL: {t.__name__}: {e}")
            failed += 1
        except Exception as e:
            print(f"ERROR: {t.__name__}: {type(e).__name__}: {e}")
            failed += 1
    print(f"\n{len(tests) - failed}/{len(tests)} passed")
    sys.exit(0 if failed == 0 else 1)
