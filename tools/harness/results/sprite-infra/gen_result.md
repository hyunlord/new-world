---
feature: sprite-infra
code_attempt: 3
---

## Files Changed
- rust/crates/sim-core/src/effect.rs: Removed out-of-scope `EffectStat::Meaning` variant (reviewer issue #6) — sprite-infra scope only requires Comfort.
- rust/crates/sim-systems/src/runtime/effect_apply.rs: Removed the `EffectStat::Meaning → NeedType::Meaning` mapping to match the removed variant; Comfort mapping preserved exactly.
- rust/crates/sim-core/src/config.rs: Tightened `ROOM_EFFECT_RITUAL_COMFORT_AMOUNT` doc comment — no longer references the removed `EffectStat::Meaning` name.
- rust/crates/sim-test/src/main.rs:
  - C3/C4 discriminator: `harness_sprite_infra_furniture_vote_majority_and_determinism` now exercises the exact plan shape — 2 totems + 1 storage_pit → Ritual (C3) and 1 totem + 2 storage_pits → Storage AND != Ritual (C4). storage_pit (not hearth) per plan v3.
  - G1/G2: Rust-mirror tests replaced with `harness_sprite_infra_g{1,2}_pick_variant_*` that execute the real GDScript picker via `godot --headless --script scripts/test/harness_sprite_infra_picker.gd`. Godot binary located via `GODOT_BIN` env, then the canonical developer path, then `$PATH`. Shared `OnceLock` caches the harness output so a single Godot invocation serves all three G assertions.
  - G3: Source-grep test replaced with an execution path that invokes the real `_load_furniture_texture` + `_tile_furniture_icon` on the production renderer (the same functions exercised by the draw path). Asserts totem emoji 🗿 and hearth emoji 🔥 after confirming `_load_furniture_texture` returns null (fallback path taken).
  - H1: Inline `serde_json::json!([…])` replaced with a committed fixture file read from disk via `std::fs::read_to_string` + `serde_json::from_str` — same I/O primitive used by `runtime_load_ws2`, through the authoritative `sim_core::room::Room` deserializer.
  - **[attempt-3 fix]** `sprite_infra_run_gdscript_picker_harness`: replaced unbounded `.output()` with `spawn()` + background reader threads + 60-second hard deadline. Passes `--user-data-dir /tmp/godot_harness_<timestamp>` to Godot for a writable user-data path. On timeout/`try_wait` error: kills the child, joins the reader threads (pipe closure unblocks them), and returns structured `[G1] FAIL: …` / `[G2] FAIL: …` / `[G3] FAIL: …` lines so each assertion fails with a clear message rather than hanging the test runner indefinitely.
- rust/crates/sim-test/fixtures/legacy_rooms_pre_ritual.json (NEW): committed legacy-save fixture — 5 Room entries using only pre-feature RoomRole variants (Shelter/Hearth/Storage/Crafting/Unknown).
- scripts/ui/renderers/building_renderer.gd: `_pick_variant_for_entity` / `_pick_variant_for_tile` now return 0-based indices in `[0, variant_count-1]` per plan G1 threshold. `building_variant_path` / `furniture_variant_path` perform the +1 conversion to the on-disk 1-based filename convention, so no call-site drift. `posmod` keeps negative entity_ids non-negative.
- scripts/test/harness_sprite_infra_picker.gd (NEW): Headless GDScript harness — loads the real `building_renderer.gd`, runs the picker 10x for determinism (G1), 100x for distribution (G2), and exercises `_load_furniture_texture`+`_tile_furniture_icon` on totem/hearth (G3). Prints `[Gx] PASS/FAIL` lines + `[SUMMARY] ALL_PASS` on success; exits 0/1 accordingly.

## Observed Values (seed 42, 20 agents)
- C3 (2 totems + 1 storage_pit): observed role = RoomRole::Ritual
- C4 (1 totem + 2 storage_pits): observed role = RoomRole::Storage
- G1 (entity_id=42, variant_count=5): value=2 across 10 deterministic calls (in [0, 4]).
- G2 (100 entity_ids, variant_count=5): 5 unique indices produced.
- G3: _load_furniture_texture("totem", ...) = null, _tile_furniture_icon("totem") = "🗿"; _load_furniture_texture("hearth", ...) = null, _tile_furniture_icon("hearth") = "🔥".
- H1: 5/5 rooms loaded from committed JSON fixture; 0 Ritual-role rooms post-deserialisation.
- E1/A11 Ritual rooms after 4380 ticks: 0
- E2/A12 Shelter rooms after 4380 ticks: 4
- E3/A13 Complete buildings after 4380 ticks: 5
- D3 Comfort delta: 0.02 (EFFECT_DAMPING_FACTOR = 0.0, so undamped)
- F2 Spiritual source-side sample > 0.0 with adjacent-side < 10% of source through stone wall

## Threshold Compliance
- A1 (cairn + gathering_marker manual_role strings): plan=exact equality, observed=match, PASS
- A2 (totem/hearth role_contribution strings): plan=exact equality, observed=match, PASS
- A3 (shelter optional_components contains hearth): plan=membership, observed=present, PASS
- A4 alias = A2 (hearth role_contribution string equality): PASS
- A5 alias = A3 (shelter.optional_components hearth membership): PASS
- B1 (room_role_locale_key(Ritual) = "ROOM_ROLE_RITUAL", prefix-guarded): plan=ROOM_ROLE_* prefix + non-empty + not-debug-fallback, observed=ROOM_ROLE_RITUAL, PASS
- B2 (5 en locale keys non-empty and != key): plan=0 violations, observed=0, PASS
- B3 (5 ko locale keys non-empty + non-key + contains Hangul): plan=0 violations, observed=0, PASS
- C1 (enclosed totem-only room → Ritual): plan=RoomRole::Ritual, observed=Ritual, PASS
- C2 (enclosed hearth-only room → Hearth, not Ritual): plan=Hearth AND != Ritual, observed=Hearth, PASS
- C3 (2 totems + 1 storage_pit → Ritual): plan=Ritual, observed=Ritual, PASS
- C4 (1 totem + 2 storage_pits → Storage AND != Ritual): plan=Storage AND != Ritual, observed=Storage, PASS
- C5 (empty enclosed room != Ritual): plan=!= Ritual, observed=Shelter, PASS
- C6 (non-enclosed with totem → Unknown, not Ritual): plan=Unknown AND != Ritual, observed=Unknown, PASS
- C7 (totem removal demotes off Ritual): plan=!= Ritual after remove, observed=Shelter, PASS
- D1 (1 Comfort AddStat per Ritual-room agent per cycle): plan=1, observed=1, PASS
- D2 (Comfort amount == 0.02 literal): plan=0.02, observed=0.02, PASS
- D3 (Needs.Comfort delta within 1e-6 of expected): plan=[damped 0.02 ± 1e-6], observed=0.02 with factor=0.0, PASS
- D4 (non-enclosed totem room → 0 Comfort effects): plan=0, observed=0, PASS
- D5 (empty enclosed room → 0 Comfort effects): plan=0, observed=0, PASS
- D6 (Comfort sign strictly positive): plan=>0.0, observed=0.02, PASS
- E1/A11 (baseline Ritual rooms == 0): plan=0, observed=0, PASS
- E2/A12 (baseline Shelter rooms ≥ 1): plan=≥1, observed=4, PASS
- E3/A13 (baseline complete buildings ≥ 3): plan=≥3, observed=5, PASS
- E4/A15 (ChannelId::count() == 10): plan=10, observed=10, PASS
- F1/A15 (Spiritual sample at totem tile > 0.0 via production path): plan=>0.0, observed=positive, PASS
- F2 (stone-wall shielding: adjacent < 10% of source): plan=adjacent < 0.1*source, observed=within, PASS
- G1 (picker deterministic + value in [0, 4] via real GDScript): plan=[0, 4] + deterministic, observed=value=2 repeated 10x, PASS
- G2 (≥ 3 unique indices over 100 ids via real GDScript): plan=≥3, observed=5, PASS
- G3 (emoji fallback preserved for totem/hearth via render-entry exercise): plan=🗿/🔥 and _load_furniture_texture null, observed=match, PASS
- H1 (legacy save deserialises via production pipeline + 0 Ritual rooms): plan=clean load + count match + 0 Ritual, observed=5/5 rooms + 0 Ritual, PASS

## Gate Result
- cargo test: PASS (253 passed, 0 failed — `cargo test --workspace` terminated in 689.34s)
- cargo test -p sim-test harness_: PASS (246 passed, 0 failed, terminated in 689.15s)
- cargo test -p sim-test harness_sprite_infra_g: PASS (3/3 G tests passed, terminated in 5.34s — no hang)
- clippy: PASS (`cargo clippy -p sim-test -- -D warnings` exits 0)
- harness (sprite-infra): PASS (30/30 `harness_sprite_infra_*` tests passing)

## Notes
- All five reviewer items from attempt 2 addressed, plus the requested scope cleanup:
  1. C3/C4 now use the exact plan shape — 2 totems + 1 storage_pit → Ritual; 1 totem + 2 storage_pits → Storage (≠ Ritual). The storage_pit counterpoint shuts both the "Ritual always wins when totem present" and "Ritual hard-coded lowest priority" bypass paths in one test.
  2. `_pick_variant_for_entity` is now 0-based per G1's `[0, 4]` threshold. The +1 to the on-disk 1-based filename happens exclusively inside `building_variant_path`/`furniture_variant_path`, keeping the picker boundary clean and callers uniform. `_get_variant_count` still enumerates `1.png, 2.png, …` (unchanged on-disk convention).
  3. G1/G2 now run the real `_pick_variant_for_entity` inside a headless Godot process. A single `OnceLock` caches the harness stdout; the three G tests grep for their `[Gx] PASS` lines independently so each reports its own outcome.
  4. G3 exercises the production render fallback: the GDScript harness calls `_load_furniture_texture("totem"/"hearth", …)` expecting null (no sprite shipped) then asserts `_tile_furniture_icon` returns the exact emoji literal. This replaces the source-grep check with a real render-path exercise.
  5. H1 now reads `rust/crates/sim-test/fixtures/legacy_rooms_pre_ritual.json` from disk and loads through `sim_core::room::Room::deserialize`, mirroring the production `runtime_load_ws2` path of "read from disk → serde parse". The JSON fixture is committed so the test is auditable in code review.
  6. `EffectStat::Meaning` variant and its `NeedType::Meaning` mapping removed from `effect.rs` / `effect_apply.rs`. `ROOM_EFFECT_RITUAL_COMFORT_AMOUNT`'s doc comment no longer references the removed variant. `NeedType::Meaning` and other pre-existing items are untouched — sprite-infra Feature 1 remains Comfort-only per spec.

- **[attempt-3 subprocess fix]** The previous attempt-3 code used `.output()` to run Godot for G1/G2/G3. That call has no timeout and would hang indefinitely if Godot crashed on startup (e.g. due to a locked or missing user-data directory). Fixed by:
  - Switching to `.spawn()` with `Stdio::piped()` stdout/stderr.
  - Draining stdout/stderr on two background threads; killing the child closes its pipes and unblocks the threads.
  - Polling via `child.try_wait()` in a 250ms loop; killing the child and returning structured `[Gx] FAIL: timed out` lines if the 60-second deadline passes. This converts any hang or startup crash into a normal test failure with full stderr diagnostics rather than a deadlocked test runner.
  - Passing `--user-data-dir /tmp/godot_harness_<millis>` so Godot always has a writable user-data path, regardless of the build environment's default Godot data directory.
  - Confirmed: G1/G2/G3 now complete in **5.34 s** (Godot found at the canonical developer path, ran the picker harness successfully).

- Godot binary resolution: `GODOT_BIN` env var → the canonical dev path recorded in the project MEMORY (`/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`) → `which godot` on `$PATH`. If none are available, G1/G2/G3 panic with an informative message rather than silently passing — the reviewer contract says the tests MUST exercise the real picker.

- One subtle compliance detail worth flagging: `harness_sprite_infra_sprite_path_resolver_non_empty` (A14) still performs a source-level grep on `building_renderer.gd` to pin the four static path resolvers' signatures + loader call sites + exact prefix strings. This was untouched because A14 was NOT flagged by the reviewer and is the contract that proves "mirror and production agree on path format" — the mirror is now only used by A14 (the integer-path sanity checks), while G1/G2/G3 exercise the picker/fallback through the real renderer via Godot. If the Challenger re-reviews and wants A14 also moved to headless-only, I'm happy to do that in a follow-up — but I held back per scope discipline.

- `EFFECT_DAMPING_FACTOR` is 0.0 in current config, so D3's expected delta equals the raw 0.02 with no scaling. The test computes `expected = 0.02 * (1.0 - config::EFFECT_DAMPING_FACTOR)` so it stays correct if the factor is ever tuned.
