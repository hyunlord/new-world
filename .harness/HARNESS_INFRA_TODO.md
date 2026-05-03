# Harness Infrastructure Stabilization (Follow-up)

## Tracked Issues

### 1. Godot headless test infrastructure (from Feature 1 — sprite-infra)
- `scripts/test/harness_sprite_infra_picker.gd` exists but not wired to sim-test
- Needs sim-test integration without subprocess hang
- Root cause: Godot headless mode doesn't terminate cleanly after script execution

### 2. Codex MCP hang during Regression Guard (from Feature 2 — sprite-assets-round1)
- Pipeline attempt 2 hung at Regression Guard step (20:32:43, no response)
- Codex MCP server did not respond, requiring process kill
- Need: hard timeout (10 min) for Codex steps + fallback to cached Regression result

### 3. VLM Analysis capturing stop hook text instead of screenshots
- Both attempts: VLM output was stop hook text, not actual visual analysis
- Pipeline fallback appended VISUAL_WARNING automatically
- GDScript-generated visual checklist (Assertions 7-10) was VISUAL_OK — real evidence exists
- Need: VLM step isolation from hook environment

### 4. HARNESS_SKIP used 2 consecutive features — pattern concern
- Feature 1 (sprite-infra): G1/G2/G3 Godot hang → HARNESS_SKIP
- Feature 2 (sprite-assets-round1): Codex MCP hang → HARNESS_SKIP
- Both are infrastructure issues, not code regressions
- Budget: next 2 features must PASS harness without SKIP to restore confidence

## Acceptance Criteria

- [ ] Codex MCP step has 10-minute hard timeout before SKIP + WARN
- [ ] Godot headless subprocess wrapper with SIGKILL after 5 minutes
- [ ] VLM step runs in isolated environment (no stop hook interference)
- [ ] Pipeline telemetry tracks SKIP usage rate (alert at 3/5 features)
- [ ] Godot headless test wrapper properly signals exit

## Priority

Medium — harness reliability affects developer confidence but not game correctness.
Address before Feature 5 (tile grid rendering) which needs reliable visual verify.
