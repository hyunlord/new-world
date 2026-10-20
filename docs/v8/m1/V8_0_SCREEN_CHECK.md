# V8-0 on screen (M1 Part 0, goal 5.1)

`tools/v8/verify_v8_0.gd` ran windowed (`--rendering-driver opengl3 --resolution 1920x1080`, under
`caffeinate -u -d`) on the live game's seed (`DEFAULT_GAME_SEED` = 20261010), once against **V8-0**
(`24e16677`, `v8/main`) and once against the **V7 game** (`v7-final-art` = `b1ca032f`), both in the base
worktree `../new-world-m1-base`, back to back on 2026-10-20 (Apple M4 Max). Both runs load the debug
bridge library (the path `sim_bridge.gdextension` names; the game's default), so S8 compares like with like.

**Adaptation for the V7 run:** none to the script. `tools/v8/verify_v8_0.gd` and `tools/v8/v7_ui_deleted.txt`
were copied untracked into the V7 checkout (`tools/` has a `.gdignore` there too); the V7 scene has the
same node paths (`WorldSim`, `Camera2D`, `WorldRenderer`, `AgentRenderer`, `UI/AgentInspectorPanel`,
`UI/CausalPanel`, `UI/ChroniclePanel`) and the inspector, causal and chronicle panels keep the fields the
script reads. The base worktree's `rust/Cargo.lock` is the main clone's.

**Method.** The script loads `res://scenes/main.tscn`, waits 20 frames, freezes the live loop
(`WorldSim.process_mode = DISABLED`) and advances with the tooling call `advance_ticks` to ticks 1,440
and 4,320.
- S1/S2: every node's script path and scene path is compared with the 18 V7 UI files V8-0 deleted
  (`tools/v8/v7_ui_deleted.txt`, from the `docs/v8/V8_0_INVENTORY.md` verdict table, test code excluded),
  at start and at the end.
- S3: the top bar's values are computed from the bridge exactly as `hud_topbar.gd` computes them, and every
  agent's entity id and tile is listed.
- S4/S5: real synthetic clicks (`Input.parse_input_event`, press and release) are sent at the first agent's
  drawn sprite and at an empty tile near the village with causal events.
- S6: the `C` key goes through `Input.parse_input_event` the same way.
- S8: the live loop runs at `set_sim_speed(3.0)` for 120 s wall time; mean FPS = frames / seconds.

| # | check | V7 game (`v7-final-art`) | V8-0 (`24e16677`) | verdict |
|---|---|---|---|---|
| S1 | god hand gone | 3 nodes: `UI/GodHandPanel` (script + scene), `GodHandEffects` | 0 nodes (start and end) | **PASS** |
| S2 | overlays gone | 15 nodes at start (settlement labels, restless meters, village activity summary, relations lines, activity tooltip, status panel, zoom LOD, need bars, seek/death viz, overview, world effects …) | 0 nodes (start and end; 1,288 / 1,332 nodes scanned) | **PASS** |
| S3 | same world | t1440: population 26, shelters 4, food 15, wood 31, day 2, 08:00; t4320: 29, 4, 34, 25, day 4, 08:00 | identical values; agent id + tile lists identical at both ticks (26 and 29 rows; list hashes 2368718544 / 381423601) | **PASS** |
| S4 | person click | inspector opens; "유루게 · #8", "어른 · 나이 7일", "1번 마을" | same; name has the id, age = bridge `ages_days` 7, village = `SettlementSelection.name_for(bridge home 1)` | **PASS** |
| S5 | tile click | causal panel opens on (22, 19): 2 bridge events, 3 rows (title + 2) | same | **PASS** |
| S6 | chronicle | `C` shows it; 6 entries shown of 32 at t4320 | same | **PASS** |
| S7 | console | `SCRIPT ERROR` 0, `ERROR:` 0, `Locale:` 0 | 0, 0, 0 | **PASS** |
| S8 | FPS, 120 s at 3× | pair 1: 102.37 · pair 2: 102.42 | pair 1: 111.00 · pair 2: 109.98 | **PASS** (×1.084 / ×1.074 ≥ 0.95) |

S8 pairs: pair 1 ran V8-0 first (1-min load 4.2 → 3.9), then V7 (load 20.9 → 10.8, right after V7's
import); pair 2 ran V7 first (9.2 → 17.1) and V8-0 second (10.9 → 4.3). The Mac carried foreign load in
both, so the numbers are contended. The ratio held in both orders. Minimum sampled FPS: V7 79 / 83,
V8-0 96 / 90.

Screenshots and raw JSON (not committed): `/Users/rexxa/work/m1-data/v8_0_check/`
(`v8_*.png`, `v7_*.png`, `verify_v8.json`, `verify_v7.json`, run logs) and `…/v8_0_check/pair2/`.
Shots: `hud_t1440`, `hud_t4320`, `s4_inspector`, `s5_causal`, `s6_chronicle`, `s8_end`.

## The two unverified doc answers (goal 5.2)

Round 1 (a fresh `general-purpose` reader given only `README.md`, `docs/**` without `docs/v8/m1/`, and the
skills) answered both questions correctly in substance:
- Q3, add a bridge read: collector → `#[func]` → `SimReads` → test → build → checks → API list.
- Q7, time scale: one tick is one minute, 1,440 ticks a day, 30 ticks per second at 1×, 48 s per day, at most
  5 ticks per frame, the target calendar open.

It listed gaps (raw report: `$DATA/logs/docs_q3_q7_round1.md`):
- Q3: the API-list path for M1; the marshaller's place and visibility; the `ffi/` file list; getters that take
  arguments; `TICK_GETTERS`; when to import; what the strict check compares; the rlib; where the test goes.
- Q7: "game time" wording; the speed keys; pause; the dropped surplus; how day and season count from the
  display clock; ages in days.

The fixes are in ARCHITECTURE §2 and §6 and in the architecture, testing and godot skills
(`$DATA/logs/docs_fix_round1.md`). Part 3 changes the time scale; round 2 re-asks both questions after that
(results in M1_REPORT.md).
