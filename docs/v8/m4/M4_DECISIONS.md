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
  pressure flaking 2, stone grinding 2, stone walling 2, pot forming 2, pot firing 2 — the chain of four pottery
  discoveries and F6's 20/32 by day 30 need odds of about 1 in 90 per check (6 checks a day per candidate).
- `knap_fine_blade`: 60 work ticks (the goal gives none), `quality_add` +15 ("quality +15 floor" read as +15 to the
  craft's quality before the knowledge cap).

## D10 — Storage of `Knowledge`

- Choice: a sorted `Vec<KnowledgeEntry>` (12 B an entry: last use `u32`, proficiency and observed `u16`, id and source
  `u8`) and the cached `u32` practitioner bitset, instead of the goal's `SmallVec<[_; 12]>`: no new dependency
  (`arrayvec` is fixed-capacity, and a person may come to hold all 24). A founder holds ≈ 8–12 entries: 32 B inline +
  96–144 B on the heap. Measured in Part 5 (bytes per person).

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
- Choice: the gathered are the people within 12 tiles of a hearth (6.9 of 10 a night) or in the hall; every pair
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
