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
- Revised again after calibration round 1 (seed 6 lost 6 people to the cold on day 18–23 with the soak script, also
  under M3 rules — so neither knowledge nor tool wear): the 0-1 search from the whole camp region put the crossing
  wherever the river was narrowest, far from the hearth; the script's stockpile "3 east of the hearth" then lay across
  the river, reachable only by a long detour, so firewood never reached the stock and the hearth went out. The crossing
  is now the river part of the shortest way **from the hearth** (Dijkstra, land 1, river 3), so the ford lies on the
  camp's own way to the forest. Seeds 0–7 × 36 days with the soak script: 0 deaths, 352–752 firewood split
  (`$DATA/cal/fix2`).
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
  `Maintain 2` hafted axes **after** the firewood bill (first placed before it: with a storehouse, a `Maintain` bill
  counts only the stock under its roof, so the axe bill never filled and the bench never split firewood — seed 6's
  bench made 23 axes and 0 firewood), so blades are consumed all along.
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

## D7 — States of a technology: "at risk" means one holder

- Context: goal 3.4 defines AtRisk as 1–2 practitioners and the event "when practitioners drop to 1", while §13 and F1
  expect the start to show "⚠ 기술 위험 2" with doorway and hide scraping red ("1명만 앎") — but hafting, timber walls,
  instruction, tracking, packed floors and digging also start with two holders, so the literal rule would show 8.
- Choice: `TechStatus` from the holders (at threshold) and the practitioners (adult holders who used it, or gained it,
  within 6 days): 0 holders → Lost if it was ever held, else Unknown; exactly 1 holder → AtRisk; ≥ 3 practitioners →
  Stable; otherwise Known. `KnowledgeAtRisk` is logged when a node moves from Known or Stable into AtRisk (naming the
  one holder); `KnowledgeLost` when it moves into Lost (naming the last named holder). The HUD badge counts AtRisk.
- Reason: the warning names one person ("문: Fox만 앎"), which is exactly the one-holder case; a two-holder node is
  "known but not stable" (yellow on the tech map).

## D8 — Teaching without instruction

- Context: goal 3.3 requires the teacher to know `instruction` (seeded to 2 of 10 founders), and F3 expects the doorway
  order from its single holder to complete by day 8 on ≥ 28/32 seeds. With independent seeding the doorway holder
  knows instruction in about 2 worlds of 10, so the core loop (the one holder teaches) would be impossible in most.
- Choice: any practitioner may teach; `instruction` is the full-fidelity channel: fidelity 950 ‰ with it, 700 ‰
  without (`KnowledgeRule::fidelity_untrained`), × 1,100 ‰ at ≥ 800 ‰ proficiency. A session gives
  round(15 × fidelity / 1000) ‰: 14 with instruction (15 sessions to 200 ‰), 11 without (19 sessions).
- The K dialog lists every practitioner as a teacher and marks those with instruction.

## D9 — Seeding order, difficulty classes, fine blades

- Seeding (goal 3.1 leaves "who" open): nodes in id order; for a `Count(n)` node the adult founders are ranked by a
  keyed draw (`STREAM_KNOWLEDGE`, subject = lineage key, slot = node id), those already practising every prerequisite
  first, and the first n get it at 500 + a keyed draw below 201 ‰ (slot 0x100 + id). So hafters knap, the door-maker
  builds log walls, the hide scraper knaps. `All` nodes go to every founder at 600 ‰. For N founders a `Count(n)` is
  scaled to round(n · N / 10), at least 1 (the benchmark's 1,000 founders).
- Difficulty (the goal table gives none for discoverable nodes): timber walls 1, clay digging 1, clay preparation 1,
  pressure flaking 2, stone grinding 2, stone walling 2, pot forming 1 (2 until calibration round 4, D18), pot firing 2
  — odds of 1 in 40 or 1 in 90 per check (6 checks a day per candidate).
- `knap_fine_blade`: 60 work ticks (the goal gives none), `quality_add` +15 ("quality +15 floor" read as +15 to the
  craft's quality before the knowledge cap).

## D10 — Storage of `Knowledge`

- Choice: a sorted `Vec<KnowledgeEntry>` (12 B an entry: last use `u32`, proficiency and observed `u16`, id and source
  `u8`) and the cached `u32` practitioner bitset, instead of the goal's `SmallVec<[_; 12]>`: no new dependency
  (`arrayvec` is fixed-capacity, and a person may come to hold all 24). A founder holds ≈ 8–12 entries: 32 B inline +
  96–144 B on the heap. Measured in Part 5 (bytes per person).
- Addendum (calibration rounds 5–6, D18): measured 161–212 B a person with 12-byte entries in a `Vec`, over the
  160 B plan. Now a boxed slice (`Box<[KnowledgeEntry]>`, 16 B header, rebuilt on the rare insert or removal) of
  6-byte entries: last use as a day `u16`, proficiency `u16`, one byte with the id (low 5 bits) and the source (high 3),
  one byte with the watched part (‰ / 5). Measured 121–159 B (calibration) and 136–159 B (held-out seeds 700–731).

## D11 — What is gated (gating completeness)

- Gated (`knowledge::gates()`): every recipe (`knap_blade` flint knapping, `haft_axe` hafting, `split_firewood`
  firewood splitting, `knap_fine_blade` pressure flaking, `grind_axe` stone grinding, `form_pot` pot forming,
  `fire_pot` pot firing); building every blueprint (hut hut-building, log wall timber walls, door doorway, packed floor,
  bedroll bedding, stone wall stone walling, knapping spot flint knapping, wood bench firewood splitting, drying rack
  hide scraping, grinding stone stone grinding, pottery bench pot forming); the designations quarry (stone quarrying),
  dig (digging) and clay pit (clay digging).
- Universal (open to everyone): delivering materials to a site, supplying a workshop shelf, hauling, restocking,
  clearing a site, chopping, gathering deadwood, deconstructing, and all survival work. Fire keeping, foraging and
  water finding are everyday lore: no job is gated by them (gating food or water would kill), they are stamped used by
  daily life (D13).
- A bill no living adult can craft is not runnable (`BillSystem`), so another bill of the same workshop runs (a wood
  bench without a hafter still splits firewood). A site waits built-ready until a practitioner comes.
- "Use" is stamped when a gated job completes (crafts, quarry, dig and clay batches) or, for buildings, when the build
  work starts (a V7 hut is finished by `ConstructionSystem`, after which the site is gone).

## D12 — The era at the start is not 0 %

- Context: goal 3.4/§13 expect "구석기 0 %" at the start, but the table seeds timber walls (a Neolithic marker, weight 2)
  to 2 founders (it gates the log walls M3 already builds).
- Choice: the table wins over the sentence; the start reads 2 practitioners of 10 adults × weight 2 / 14 ≈ 28 ‰ ("구석기
  2 %"), and falls to 0 when nobody has built a log wall for 6 days. Tests assert the start value computed from the
  table, not 0.

## D13 — Everyday lore and the evening telling

- Oral telling runs at 20:30 (`oral_minute` 1230; inside the 18:00–21:00 evening-gather window, before night rest).
  The gathered: within 12 tiles of a hearth (the camp) or in the hall (D14).
- Fire keeping is stamped used by every gathered holder each evening; foraging when the person is seen seeking or
  eating food (sampled at each forgetting step), water finding when seen drinking. Storytelling is used by a teller.
- A listener "lacks" a node when below its threshold, so a forgotten node can be heard again.
- Oral learning is not logged in the chronicle (it would flood it); the knowledge tab shows it.

## D14 — The evening telling: who gathers, rehearsal, telling is use (calibration before Part 2's tests)

- Measured on the soak (seeds 0–3, 20 days; `$DATA/scan/`): with the gathered read as "within 6 tiles of the hearth"
  only 3.4 of 10 people were there at 20:30 (busy people work into the evening; the soak's walled house lies 8–12 tiles
  east across a ford), and every variant lost nodes within 20 days:
  - no rehearsal: hut building on all 4 seeds at day 18.7 — everyone held it at 600 ‰, no hut was ordered, all faded
    on the same day, and talk could not help since nobody "lacked" it;
  - rehearsal only when the teller had nothing new: radius 6 / 10 / 12 / 16 at 20:30, 18:30 or 19:30 still lost
    3–7 nodes (hut building, hide scraping, instruction, tracking, clay digging);
  - rehearsal by every pair: radius 10 lost instruction, radius 12 hide scraping;
  - plus "telling is use" for the teller: radius 10 and 12 lost nothing.
- Choice: the gathered are the people within 12 tiles (16 since calibration round 5, D18) of a hearth or in the hall; every pair
  rehearses one node both practise (keyed), stamping it used for both; the teller is stamped as using what it tells.
  Shared knowledge lives while people talk; a node held by one person spreads by talk or dies with them or fades if
  they never tell it — the at-risk drama stays.
- Same measurement: adult knappers on day 12: 7, 8, 8, 5 (F2 needs 6); seeded entries forgotten by day 20: 1, 1, 0, 1;
  `Knowledge` 144–155 B a person (after D10's 8-byte entries; 182–226 B before).

## D15 — The hearth as the pot-firing workshop

- Context: goal 3.5 — `fire_pot` is made "at the hearth (the hearth becomes a workshop target for this recipe only)".
- Choice: under the knowledge rule the colony worldgen adds a `Workshop` of kind `BlueprintKind::HearthFire` (1×1 at
  the camp hearth's tile, worked from the tile south of it, never placeable, `placeable()` false) with the next
  workshop id. It occupies no tile (the hearth stays the settlement's building), so rooms never count it as a workshop
  and a hall stays a hall. Its only recipe is `fire_pot`; bills are added to it like any workshop's.

## D16 — Clay banks

- Choice (goal 3.5: "density such that ≥ 3 banks lie within 24 tiles of the camp in ≥ 30/32 seeds"): under the
  knowledge rule, one clay bank per 80 river-bank tiles on the free bank tiles (shuffled with `STREAM_CLAY`, slot 0),
  then, while fewer than 3 lie within 24 tiles of the camp, more on the free bank tiles within that distance (slot 1).
  A bank holds 18 clay = 6 digs of 3 (`DigClay` jobs, `WorkType::Mine`, 60 ticks at level 5, skill Mining). The bank
  material is `clay` (no stats file: nothing reads its hardness). Quarry marks skip clay banks; `DesignateClay` marks
  them.
- Measured: PENDING (banks within 24 tiles on seeds 0–31).

## D17 — `DebugKill` (F10's fixture)

- A recorded command (`Command::DebugKill { agent }`) that kills a person through the normal despawn path (recorded as
  old age, so the death reaches the chronicle, the settlement and the job layer exactly as a death does). The probe
  script parser accepts `debug_kill`; the game bridge has no `cmd_debug_kill` (the bridge API list does not change for
  it), so a player cannot issue it.

## D18 — Calibration (goal §10: seeds 0–7, at most 8 rounds; the M4-U script; `docs/v8/m4/m4_score.py`)

- The scorer's reading of every bar ("by day N" = reached at the end of some day ≤ N; F3 = the order closes done; F5's
  stall = a once-known gate the colony had an order for with no living adult practitioner; F8's means) is registered in
  the scorer's docstring before round 1 and was not changed except F3's source of truth (round 2, below).
- The M4-U script (`tools/v8/m4_u_script.json`) = the soak script + the day-2 teach order (`order_teach`,
  `single_holder`: the holder, those with instruction first, to the lowest-`AgentId` adult without it) + clay marks on
  every bank within 24 tiles once someone practises clay digging + a pottery bench, `Make 6 form_pot` and
  `Make 6 fire_pot` (at the hearth fire) once someone practises pot forming + a grinding stone and `Make 4 grind_axe`
  once someone practises stone grinding. Quarry marks (1 boulder, 2 outcrops) renew every 3 days from round 2 (the
  goal's "quarry" in the economy; without it flint ran out by day 10).
- Rounds (each on `$DATA/cal/rN`, 36 days, seeds 0–7; bars scaled to 8 seeds):
  1. as committed at `276923de`: F1 F5 F9 pass; F2 5, F3 1, F4 5, F6 1, F7 1, F8 0, F11 7 (seed 6: 6 cold deaths).
     Causes: F11 the soak's axe bill order and the ford placement (D5, D6); F3 lessons needed both partners free in the
     same tick, and the probe counted only session completions; F8 flint ran out.
  2. script quarry renewals; lessons one-sided with a 240-tick wait (D8 addendum below); F3 read from the order's
     completion tick: F3 8, F4 7, F5 7, F6 5, F11 8 pass; F2 5, F7 1, F8 0.
  3. the oral rule as the goal states it (each candidate rolled, the first that takes is learned; D14): F2 4, F7 2.
  4. oral entries fade 1 ‰ an hour (was 2), pot forming difficulty 1, stone axes 20 chops (a sensitivity test): F7 6
     passes; F2 5; F8 0 — blades still stop around day 21 with 20-chop axes, so durability is not the lever (D20); the
     goal's 40 is restored.
  5. gathering radius 16 (was 12), axes 40: F2 7, F4 8 pass; F6 4; memory 161–212 B a person (over the plan).
  6. 6-byte entries in a boxed slice (D10): memory 121–159 B; F6 4 (conditions held 33–35 days on the failing seeds).
  7. discoverers include the most curious 20 % (D19): every bar passes except F8 (F1 8, F2 8, F3 8, F4 8, F5 8, F6 6,
     F7 8, F9 8, F11 8; F8 0).
- Frozen after round 7 for the held-out evaluation (seeds 700–731).

## D19 — Who discovers: novelty seeking ≥ 600 or the most curious fifth

- Context: goal 3.3: "candidates = adults with `Temperament.ns ≥ 600` (innovation potential; top ≈ 20 %)". On the
  calibration seeds the founders at ≥ 600 were 5, 4, 1, 2, 4, 1, 0, 6: seed 6 could discover nothing, and seeds 2 and 5
  depended on one person (round 6: stone grinding's conditions held 33 days on seeds 3, 5, 6 with no discovery).
- Choice: a candidate has novelty seeking ≥ 600 **or** is among the most curious `discovery_top_permille` (200) of the
  adults by novelty seeking (ties by `AgentId`, at least one) — 2 of 10 adults at minimum, the goal's "top ≈ 20 %".

## D20 — F8 (skill growth) is not reached: the economy saturates

- Measured on every round: the top knapper reaches level 5–7 by day 36, but no blade is made after about day 21 on
  most seeds, so "the blade quality mean of days 25–36" does not exist. Causes: (1) every adult ends up holding an axe
  (10 axes in circulation; 200–270 chops in 36 days is 20–27 per axe), so few axes break (0–7 per world); (2) with a
  storehouse, `Maintain` bills count only the stock under its roof (M3 ADR 0023), so axes and blades lying outside
  are made in excess early (12–57 axes) and then nothing more is needed; (3) nothing but hafting consumes blades.
- Halving the stone axe's durability (round 4: 20 chops, 3–7 breaks a world) did not restore late blades either, so
  the goal's 40 stays; F8 is reported as failed with this cause. Options for the owner: tools go back to a stockpile
  between jobs, the `Maintain` count includes held tools, or a second blade consumer (hide scraping, M5).

## D8 addendum — one-sided lessons (calibration round 2)

- A lesson needed teacher and learner free in the same tick; in a busy colony that took up to 32 days. Now either
  partner who is free takes its lesson job, walks to the meeting tile and waits there up to `WAIT_TICKS` (240) for the
  other; a session that loses its learner (a need) goes back to waiting. Round 2: the doorway order completed by
  sessions on days 4.2–6.3 on all 8 seeds (15–19 sessions).

## D21 — The UI (Part 4) and the M3 UI test

- Built by a subagent in an isolated worktree against `276923de` (GDScript, locale, sprites only; no Rust), brought
  into the main tree unchanged and re-checked there: strict check 33/33, smoke 0/0/0/0, `tools/v8/m4_ui_test.gd` 32
  checks 0 failures, `m2_ui_test.gd` 0 failures, `test_locale_trf.gd` pass. Keys: T tech map, K teach dialog, F7 then
  2 clay pit (F7 alone still enters quarry mode, M2's behaviour), build menu `0` pottery bench and `-` grinding stone,
  the inspector's 지식 tab, a read-only 토기 솜씨 column in the priority table. Source words: 전승 (seeded), 구전 (oral),
  관찰 (observed), 발견 (discovered), 도제 (apprenticed). 147 new locale keys (`docs/v8/m4/M4_LOCALE.md`).
- Found on the way: the hearth's fire-pot workshop hid the hearth sprite from tick 1 (the building layer drew the
  workshop over it) — the hearth is drawn again and a click opens its workshop panel; the chronicle panel had
  silently dropped kind 23 (RoomFormed) — fixed with 24–28; `m2_ui_test` assumed the first workshop was the knapping
  spot (now the hearth fire) and 6 skills — fixed.
- `tools/v8/m3_ui_test.gd` (M3's record, not a gate step) now fails one check, `house_room`: its house is built of
  stone walls, which M4 gates behind stone walling (a Neolithic node nobody knows at the start), so the 20 wall sites
  wait. Kept as M3's record; `m4_ui_test.gd` covers the build menu with gating.

## D22 — F10 found a gating hole: the V7 construction arm built doors nobody knew

- The first F10 fixture run (`tools/v8/m4_f10_script.json`, seeds 0–7, build `84f336a5`) lost the doorway on
  8/8 seeds 120 ticks after the kill (the aggregate's next interval), but seed 5 ended with no door order waiting:
  both of its door sites (the house's, placed at tick 360, and the storehouse's, placed at the kill) were finished
  at ticks ~2,200–2,880 with no living practitioner.
- Cause, traced tick by tick: an adult with no job standing on a delivered site took the V7 decision system's
  fourth cascade step ("a co-located active ConstructionSite exists"), entered `Consuming{ConstructionSite}`, and
  `ConstructionSystem` advanced the site. A door site is the walkway through its wall ring, so people stand on it.
  The job layer's gate (`assign.rs`, Build) was right; this second builder never asked.
- Fix: `knowledge::open_to_any_builder` — the decision system's site snapshot leaves out every colony site whose
  blueprint knowledge gates (M4 rule only), so such a site is built only by a practitioner's `Build` job.
  `decision/system.rs` +4 −1 (the M4 seam budget is +20). Baseline has no colony sites (identity unaffected, checked).
  Test `a_door_site_is_never_built_by_a_passer_by_without_the_doorway` (colony seed 5: kill the door maker, deliver
  a door site, keep an adult idle on it for 720 ticks → progress 0; then that adult practises the doorway → the door
  is built) fails without the fix (progress moves) and passes with it.
- The held-out result of record is the pre-fix run (`84f336a5`); the fixed build is measured again on the same
  held-out seeds and both are reported (M4_EVALUATION.md). The fix was not tuned on those seeds.

## D23 — F12: what the benchmark found (memory fixed; the fords' cost at N = 200 justified)

- Ledger session 03:51–03:54 KST (quiet, 1-min load 3.5–4.4; M3 `bf16de16` and the candidate `e7deb890`
  interleaved, 3 rounds). Knowledge systems together 0.7 µs/tick at N = 200, 3.3 at N = 1,000, 7.5 at N = 2,000
  (budget 150 at N = 1,000); `JobAssignSystem` 121.7 → 122.5 µs at N = 1,000 (+0.9; budget +40). C3 ceilings 4/4
  PASS. Tick mean against the same-session M3: n1000 ×1.030, n2000 ×1.038, n200_side64 ×1.036, but n200 ×1.235 and
  n200_script ×1.171; RSS per agent 2,048 → 2,412 B (×1.18).
- **Memory, fixed.** Without seeding (a diagnostic build) RSS per agent matched M3 (2,094 vs 2,112 B, medians of 3),
  so the +330 B was not the knowledge data (≈ 150 B) but where it was put: `seed_founders` inserted `Knowledge` into
  every founder after all were spawned, moving all of them to a new archetype; hecs keeps the old archetype's columns
  allocated. Founders now get an empty `Knowledge` in the per-founder spawn sequence (`worldgen/camp.rs`, colony
  knowledge rule only) and seeding replaces it in place: 2,166 vs M3 2,121 B (+2 %). The colony is byte-identical (the
  calibration run, seeds 0–7 × 36 days, has the same `summary.json`); the baseline does not take the branch (identity).
- **Tick at N = 200, justified.** Building `worldsim_perf` at every Part's commit and timing n200 interleaved:
  M3 313 µs, Part 0 (`06c965aa`, fords and tool wear) 392, Parts 1–2 (`276923de`, knowledge) 398, Part 3 386, the
  candidate 386 — the whole step is Part 0. A diagnostic build of the candidate without fords runs n200 at ×0.981 and
  n200_script at ×0.995 of M3. With the fords the bench world's forest across the river is reachable (4 ford tiles), so
  people forage there: the V7 decision system's gather selectors (`gather_tile_exists`, `nearest_gather_tile`, a scan
  of all ≈ 2,050 food tiles per call) and bush choice run more often (samples: gather 154 → 257, `choose_bush`
  0 → 20). The cost is the world's new reach, not knowledge code, and shrinks with N (×1.03 at 1,000, ×1.04 at 2,000).
  A radius-box scan was tried and was slower (hash lookups over a 17 × 17 box cost more than iterating 2,050 entries;
  decision 276 → 320 µs), so it was reverted. An index over food tiles is M-PERF work (the V7 selectors are not
  touched by M4). The CHANGELOG states the justification the gate's `bench` step quotes.
