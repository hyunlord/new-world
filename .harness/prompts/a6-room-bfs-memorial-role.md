# Feature: a6-room-bfs-memorial-role

## Summary

Add `RoomRole::Memorial` variant to complete the A-6 Room BFS pre-req.

A-6 Room BFS is 95% already implemented (detect_rooms, assign_room_ids,
role voting, 3 RoomRole effects working in production — Shelter/Hearth/
Ritual). The only gap was RoomRole::Memorial, which caused cairn-containing
rooms to misclassify (landmark → fallback Shelter via majority_role).

This feature adds the variant + a single passive Meaning effect. No new
infrastructure — same pattern as existing roles.

## Changes Made

### rust/crates/sim-core/src/room.rs
- Added `RoomRole::Memorial` variant (6 → 7 roles)

### rust/crates/sim-core/src/config.rs
- Added `ROOM_EFFECT_MEMORIAL_MEANING_AMOUNT: f64 = 0.0001`
  (400× smaller than Mourn burst, models passive long-term accumulation)

### rust/crates/sim-systems/src/runtime/influence.rs
- `assign_room_roles_from_buildings`: added `"cairn" => Some("landmark")` to
  building-type role match
- `majority_role`: added `Some("landmark") | Some("memorial") => Memorial`
- `apply_room_effects`: added `Memorial` arm emitting `EffectPrimitive::AddStat {
  stat: Meaning, amount: ROOM_EFFECT_MEMORIAL_MEANING_AMOUNT }` with source
  kind `"memorial_meaning"`, routed through EffectQueue (A-3) → causal_log (A-4)

### rust/crates/sim-bridge/src/tile_info.rs
- Added `RoomRole::Memorial => "ROOM_ROLE_MEMORIAL"` to locale key match

### localization/{en,ko}/ui.json
- `ROOM_ROLE_MEMORIAL`: "Memorial" / "추모소"
- `CAUSE_MEMORIAL_MEANING`: "Memorial meaning" / "추모의 의미"

### rust/crates/sim-test/src/main.rs
- Updated `harness_wall_click_info_a16_room_role_locale_key` to include
  `RoomRole::Memorial` in the exhaustive variant array
- Added 4 harness tests:
  - `harness_a6_memorial_role_exists` (A1, Type A): variant distinct from
    all 6 existing variants
  - `harness_a6_landmark_maps_to_memorial` (A2, Type A): enclosed room with
    completed cairn receives `RoomRole::Memorial`
  - `harness_a6_memorial_room_boosts_meaning` (A3, Type A): apply_room_effects
    enqueues exactly 1 Meaning AddStat; EffectApplySystem flush yields positive
    Meaning delta matching ROOM_EFFECT_MEMORIAL_MEANING_AMOUNT
  - `harness_a6_memorial_room_causal_log_entry` (A4, Type A): causal_log records
    kind="memorial_meaning" for recipient agent

## Effect distinction

| Source | Type | Meaning delta | Frequency |
|--------|------|---------------|-----------|
| Mourn action (memorial-system-v1) | burst | +0.04 | once per Mourn completion |
| Memorial room (this feature) | passive | +0.0001 | every tick while inside |

Both can co-occur on the same agent in the same tick; causal_log distinguishes
them via `kind`: `"mourn"` vs `"memorial_meaning"`.

## Integration with A-3/A-4/A-6/A-8

```
Agent enters Memorial room
  → A-6 Room BFS: detects RoomRole::Memorial (this feature)
  → A-3 EffectQueue: enqueues entry
  → A-3 EffectApplySystem: applies + pushes to causal_log
  → A-4 CausalLog: records kind="memorial_meaning"
  → A-4 UI: Entity Inspector "최근 인과" displays Memorial entries
  → A-8 Temperament: agents with high Persistence stay longer → more accumulation
```

## Scope

- No BFS algorithm changes
- No shelter quality refactor
- No new landmark buildings (cairn is the only trigger)
- No Memorial-specific UI visual effects
- Meaning amount 0.0001 is a placeholder (tuning deferred to Phase 3)

## Verification

- cargo check: clean
- cargo clippy: clean
- 4 A-6 harness tests: PASS
- 12 A-4 harness tests: PASS (regression check)
- Full sim-test suite: 315 passed, 0 failed (311 prior + 4 new)
- Exhaustiveness check: all `match role`/`match *role` sites updated
  (influence.rs, tile_info.rs, A16 test)
