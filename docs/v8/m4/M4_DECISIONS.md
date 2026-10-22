# M4 decisions

Numbered decisions of the M4 pass (`docs/v8/m4/M4_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m4-data/` (`$DATA`), outside the repository.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/main` = `v8/wip` = `5da079d3` (checked with `git ls-remote` at the start,
  2026-10-23 00:33 KST).
- Choice: base worktree `../new-world-m4-base` detached at `5da079d3`, with the main clone's `rust/Cargo.lock`
  copied in. Data in `$DATA` = `/Users/rexxa/work/m4-data/`. An owned `caffeinate -i` runs for the session.
  `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`. Start 00:33 KST on 2026-10-23; the 20-hour
  target is 20:33 and the 24-hour hard stop 00:33 on 2026-10-24.
- The host is shared: an art session may run ART-TRIAL5 in parallel (goal header); the 1-minute load was 3.5 at
  the start and 13–14 eight minutes later. Timings are labelled with the load they were taken under.
- Evidence: `$DATA/TIMELOG.txt`.

## D2 — The performance plan (goal §4, written before any knowledge code)

- Budget (pre-registered): at N = 1,000 with the soak script, all knowledge systems together (oral,
  observation, decay, discovery, aggregate) ≤ 150 µs per tick mean; `JobAssignSystem` growth from gating
  ≤ +40 µs per tick; ≤ 160 B per person added (the `Knowledge` component); the whole tick stays within the C3
  ceilings and within +10 % of the quiet M3 ledger rows (Part 0 ledger rule).
- Cadence: decay every 60 ticks, observation every 30 (only people at a gated job, neighbours through the
  agent positions of that pass), discovery every 240, aggregate every 240, oral once a day at the evening
  gathering. Gating reads a cached practitioner bitset (`Knowledge::practice`, `u32`, bit `i` = knowledge `i` at or
  above its threshold), refreshed by every path that changes a proficiency (decay, oral, teaching, discovery,
  use); the gate of each recipe, blueprint and designation is a precomputed `u32` mask (`knowledge::gates()`).
- Measurement: `worldsim_perf` gets a `knowledge` line (the five systems' µs per tick summed) in its per-system
  table; `bench` and `nav-bench` run after Part 2 and after Part 3.
- If over budget, in this order, each step's effect recorded: (1) the cached bitset (planned from the start);
  (2) observation every 60; (3) aggregate every 480.

## D4 — The M3 owner decisions, recorded (goal 3.6)

- Room roles: automatic with the player's manual lock (`SetRoomRole`) is the default (ADR 0023 as built).
- Roof: the enclosure is the roof until further notice (ADR 0023; no roof item).
- Rivers get fords; the default seed stays (D5).
- F7 (skill growth) is measured again in M4, with tool wear as the blade sink (D6; M4 F8).

## D5 — Fords

- Context: goal 3.6 — the default world's near forest lies across the river with no ford and no refusal reason.
- Choice: a `ford` column on the tile grid (empty until set; `TileGrid::is_ford`); `nav::tile_cost` gives a ford river
  tile `COST_BANK` = 30 (the reserved shallow-water cost). The colony worldgen (`worldgen::colony::fords`, after the
  camp is opened and before founders are moved) finds the largest forest region (the nav region with the most forest
  tiles); when it is not the camp's region, up to two crossings become fords, each the 4-connected way with the fewest
  river tiles inside the box of 40 tiles around the camp (0-1 BFS; mountain and undug hill never crossed; the second
  crossing more than 8 tiles from the first). Ford tiles stay river (no items, still water to drink).
- Revised after the first measurement: "the largest forest region" read as the nav region with the most forest tiles
  anywhere gave 0 fords on every seed — the main landmass is always the camp's region — while the default seed's near
  forest (a 547-tile patch 8 tiles from the camp) stayed across the river. The target is now the cut-off region (not
  the camp's) with the most forest tiles **within 40 tiles of the camp**, at least 100 of them; both crossings go to it.
- Marks no one from the camp can walk to are refused: chop, quarry and dig marks on targets outside the hearth's
  region are skipped, and a command whose every valid target is unreachable is rejected `Unreachable` (bridge reason
  `unreachable`, toast "닿을 수 없음: 강 건너"). Under M1/M2 rules (no space rule) nothing is refused.
- Measured (`colony_nav.rs::fords_join_the_camp_to_the_largest_cut_off_forest_within_40_tiles`): seeds 0–7 and the
  default seed 20,261,010 each get two crossings of 2 tiles (4 ford tiles); afterwards the largest cut-off forest
  within 40 tiles holds 0–7 tiles (it held 400–1,100 before, e.g. the default seed's 547-tile patch at 8 tiles).
  `a_mark_across_the_river_is_refused_as_unreachable`: a chop mark on the nearest cut-off tree is rejected
  `Unreachable` on every seed that has one.

## D6 — Tool wear

- Context: goal 3.5 — StoneAxe durability 40 chops; at 0 the axe breaks (consumed, chronicle `ToolBroke`).
- Choice: durability is the held tool's `Tool::uses` against the kind's limit (`EconomyRule::axe_uses`, 40 under
  `EconomyRule::M4`), not a per-item field: items in stacks merge by spec, and a tool is worn only while held. A tool
  dropped at a death and picked up again starts fresh (stacks keep no wear). `EconomyRule::tools_break` (M4) uses the
  broken tool up (ledger consumed, `ColonyTallies::tools_broken`, chronicle `ToolBroke` 24: subject the holder, cause
  the item code, value the uses); M2/M3 rules keep the M2 behaviour (a stone back).
- The soak script (`tools/v8/soak_script.json`, renamed from `m3_soak_script.json`; gate step name kept) adds
  `Maintain 2` hafted axes after the `Make 4` bill, so blades are consumed all along.
- Evidence: `colony_skills.rs::under_m4_a_worn_out_axe_breaks_after_40_chops_and_is_used_up`.

## D3 — The ledger rule and the quiet M3 re-measure (goal 3.6)

- Context: the M3 ledger rows (`23be0178`, and the base M2 rows `f0ba4347`) were appended from a contended session
  (1-minute load 18.6–24.7), so the gate's quiet run at `bf16de16` compared against inflated numbers.
- Choice: `bench_ledger.py`: a run with any repeat started at a 1-minute load ≥ 8 is contended; `append` refuses it
  without `--note` and marks every row's note "contended, not a base: …"; `check` compares with the latest pass whose
  rows are not so marked. The ten M3-session rows are re-labelled "contended, not a base" and stay in the file.
  Tests: `tools/v8/test_bench_ledger.py` (a contended run needs a note and is never a base; a ledger of contended rows
  only has no base).
- Quiet re-measure: `$DATA/bench_session.sh`, base `f0ba4347` (the M2 simulation of `f6a89e72`, M3 base worktree
  binary) and candidate M3 `bf16de16` (= `5da079d3`'s simulation, M4 base worktree binary), interleaved, 3 rounds of
  `bench --repeats 1`, 00:43–00:47 KST on 2026-10-23; every repeat started at a 1-minute load of 4.2–6.8 (admitted).
  Appended as passes "base M2" and "M3".
- The true M2 → M3 change (tick mean / p99): n200 ×0.873 / ×0.945, n1000 ×1.053 / ×1.225, n2000 ×1.024 / ×1.197,
  n200_script ×0.889 / ×0.916, n200_side64 ×1.156 / ×1.319; RSS per agent 1,720 → 1,993 B; RSS per tile 96.5 → 89.5 B;
  M3 ceilings 4/4 PASS. So M3 did exceed +10 % on the 64² scene's mean and on three p99s, which the contended session
  had hidden; recorded, not re-litigated (M3 is closed). The M4 gate compares with these M3 rows.
- Evidence: `$DATA/bench/RUN.txt`, `$DATA/bench/{base,cand}_bench.json`.
