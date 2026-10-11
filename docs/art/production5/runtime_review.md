# Runtime review — Art Pass 5

Reviewed 2026-10-11, HEAD `0ce263372e52461f4fd7c4ad3281beaa5ce04fcb` plus concurrent working-tree changes, against `c6d88d62`. Read GOAL sections 6, 9, 10 and 11. Scope: runtime presentation, shader/variant changes, relevant tests and localization. Simulation merge excluded. No implementation edits or windowed Godot runs.

## Finding

**P2 — Actual seat coordinates are never attached to capture entities.** At `scripts/ui/agent_renderer.gd:518-521`, the renderer records `seated`, `drawn_target`, `spread_offset` and `declump_offset`, but never records `seat`. `scripts/test/art/capture_metadata.gd:89-93` includes `seat` only when node metadata already contains it, so this newly added serializer branch is never reached for real villagers. GOAL 11.1 requires each villager's seat. `drawn_position` includes smoothing and shiver; the boolean alone does not provide the explicit persistent seat record. Set seat metadata from the actual seat assignment and remove it when the assignment ends. Add a renderer-to-capture regression covering both states. The current metadata test passes because it sets fixture metadata directly and does not check a seat coordinate.

No other concrete correctness defect established in the reviewed runtime changes. This finding was sent to the leader before the report; subsequent edits may resolve it and require a follow-up check.

## Follow-up verdict — PASS within review scope

The P2 finding above is resolved. Re-inspection confirms `agent_renderer.gd:519-522` now sets `seat` from the actual assignment and removes stale metadata when the villager becomes unseated. The original finding is preserved above as the review record.

Independently reran `test_agent_drawn_positions.gd` after the fix with the same 35-second bounded headless command: exit 0, `agent drawn positions: 0 failures`. Its new assertion verifies that a real assigned seat reaches sprite metadata and matches `drawn_target`. Also reran `test_capture_metadata.gd`: exit 0, `capture metadata failures=0`. Seat removal was inspected in code; the added assertion tests the assigned state, not the later removal transition. No implementation files were edited by this reviewer.

## Verification

Godot 4.6 executable: `/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.

All ten headless scripts passed with exit code 0: `test_declump`, `test_cold_presentation`, `test_camp_seats`, `test_hud_wood`, `test_inspector_cold`, `test_art_hash`, `test_variant_codes`, `test_elder_variant`, `test_capture_metadata`, `test_agent_click_drawn`.

The nine scripts after declump used `bash tools/art_pipeline/run_bounded.sh 35 ... --headless --path . --script scripts/test/art/<test>.gd`. Direct execution of the wrapper initially returned permission denied; invoking it with bash succeeded. Declump completed directly in under one second.

Observed: 41-seat separation/persistence test passed; cold presentation reported zero failures; click integration reported zero failures; 3,000 identities yielded tone counts `[723,772,755,750]` and hair counts `[595,620,575,601,609]`; all 28 tone/hair encoding pairs passed.

Not verified here: rendered shader pixels, puff visibility in actual captures, visual natural-colour correctness, full art capture campaign, frame-time comparison, Rust gates, or final post-integration tree. Headless presentation tests do not establish visual acceptance.
