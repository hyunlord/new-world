# M5 decisions

Numbered decisions of the M5 pass (`docs/v8/m5/M5_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m5-data/` (`$DATA`), outside the repository. The goal says "never ask the
owner": every open question is decided here.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/main` = `08f59813` (M4 report; gate green at `941410fc`); `v8/wip` at `24ea244d`
  (the goal commit on top of `08f59813`), checked with `git rev-parse` against `origin/*` at the start
  (2026-10-23 10:08 KST).
- Choice: base worktree `../new-world-m5-base` detached at `08f59813` with the main clone's `rust/Cargo.lock`;
  `$DATA` = `/Users/rexxa/work/m5-data/`; an owned `caffeinate -i` for the session; `CARGO_BUILD_JOBS=8`, tests
  `--test-threads=8`, probe `--workers 6`. Start 10:08 KST; the 22-hour target is 08:08 and the 26-hour hard stop
  12:08 on 2026-10-24. Base data (`$DATA/base/`): release binaries of `08f59813` (`bin/`), the identity capture
  (`identity/`, with the base `perf.sh` run in `identity/perf/`), the soak (`soak/`) — `$DATA/base_capture.sh`.
- The host is shared (an art session may run ART-PROD2 in parallel); timings are labelled with their load.
- Evidence: `$DATA/TIMELOG.txt`, `$DATA/base_capture.log`.

## D2 — Art record correction (goal §5.2)

- Context: M4's records (ROADMAP ART-TRIAL5 row and "다음" line, HISTORY, ADR 0018) say people art is parked after
  ART-TRIAL5. ART-PROD1 (`v8/art-prod1`, `eb428ce2`, `docs/v8/artprod1/PROD1_REPORT.md` there) has since produced
  whole-figure sheets by route I: 13 of 18 adult looks and 0 of 18 child looks accepted (130 frames); the children
  failed on the sleep frame (canvas normalisation and sleep width), not on reading; the trial's verdict was FAIL
  (S1, R1, P1) and route I was not adopted as the standard. ART-PROD2 runs on the same branch to fix the sleep rule
  and adopt.
- Choice: the records say people art is on route I (whole-figure sheets), pending ART-PROD2; "사람 그림 보류" is
  withdrawn. ART-PROD2's final state is read from `docs/v8/artprod2/` on `v8/art-prod1` in Part 7 and recorded then.
  This pass does not touch `art/**`, `docs/v8/art*/**` or merge any art branch.

## D3 — The ledger rule, second fix (goal §5.3)

- Context: M4 appended its own rows (pass "M4") at Part 5, so the Part 6 gate's `bench_ledger.py check` compared
  M4 with M4 (M4 D23 then compared M3 → M4 by hand).
- Choice: `check --pass NAME [--base COMMIT]` compares the candidate with the latest quiet rows whose pass label
  differs from NAME and whose commit is not a strict descendant of COMMIT; `append` needs `--gate-log PATH` (a
  `gate.txt` whose head matches the run's commit and whose status is 0) or `--re-measure TEXT` (an older pass that
  already passed its gate); the gate's `bench` step only checks (`BENCH_PASS`, `BENCH_BASE`), and a pass appends its
  rows in the report step, after its gate. Tests in `tools/v8/test_bench_ledger.py`.
- Built by a subagent in an isolated worktree (files listed in its report `$DATA/part0_tooling_report.md`), reviewed
  and brought in by the lead.
- **M4 re-checked against M3** (goal 5.3): quiet session 12:01–12:06 KST (1-minute load 2.4–3.6 before every
  repeat; `$DATA/bench_session.sh`, `$DATA/bench/`): M3 `bf16de16`, M4 `941410fc` and M4 with the allocator counter,
  interleaved, 3 rounds of `bench --repeats 1`. True M3 → M4 tick mean: n200 ×1.220, n1000 ×1.054, n2000 ×1.034,
  n200_script ×1.157, n200_side64 ×1.013; p99 n1000 ×1.064 (M4's ×1.106 is not reproduced in a quiet interleaved
  session); RSS per agent 2,021 → 2,175 B (×1.076). The counter costs ×1.000–×1.022 on the tick mean.
- **The n1000 p99 look** (per system at N = 1,000, medians of the three rounds, `$DATA/bench/p99_look.txt`):
  `AgentMovementSystem` is the system: +22 µs a tick (88 → 110) and its worst tick 2,268 → 7,221 µs — path
  planning across M4's fords (the far forest became reachable, so A* searches cross the river); the decision
  system +64 µs (the gather selectors, M4 D23). Nothing else moved more than 5 µs.
- Ledger rows appended with `--re-measure`: pass "base M3" and pass "M4 re-measure" (with alloc/agent 1,871 B) — M5's
  comparison base; `check --pass M5` compares with "M4 re-measure" and decides memory on alloc/agent.

## D4 — The allocator byte counter (goal §4, §5.4)

- Choice: `sim_test::alloc::Counting`, a `GlobalAlloc` that forwards to `System` and counts live bytes; installed as
  the global allocator of `worldsim_perf` only (never the game, the bridge or the probe). The goal names
  `sim_core::alloc`; `sim-core` is `#![forbid(unsafe_code)]` and a `GlobalAlloc` needs `unsafe`, so the counter lives
  in the measurement crate instead of weakening the simulation crate's guarantee to `deny` (the subagent's first
  version did that). `worldsim_perf bench` records
  live bytes before and after the build and after the run; `alloc_per_agent` = (n2000 − n200 after the build) / 1,800
  and `alloc_per_tile` likewise. The ledger gains the column `alloc/agent (B)` after `RSS/agent (B)`; from M5 on the
  regression rule's memory figure is the allocator number when both rows have it (RSS stays as information).
- Reason: RSS per agent varied by up to 1 MB between runs of one binary (M4 D23: bimodal macOS allocator placement);
  live allocated bytes are a property of the program, not of the allocator's placement.

## D5 — M4 owner decisions recorded (goal §5.5)

- The teach dialog (K) auto-suggests the learner: the adult with the highest novelty seeking (`Temperament.ns`), then
  the youngest adult, among those who lack the node; the player may change it. (UI work in Part 5.)
- A re-learned knowledge keeps its original source's forgetting floor (M4 D24 F4 reversed by the owner): an entry
  whose proficiency rises through another channel keeps the source it was first held by when that source's floor is
  higher (seeded and discovered knowledge keep their floor). Implemented in Part 4 with a test.
- Forgetting speed unchanged pending the owner's playtest; pot firing unchanged.

## D6 — Where the M5 state lives

- Animals are not ECS entities: they live in `ColonyState.wildlife` (a `BTreeMap` by animal id, at most 16 at once).
  Reason: queries over `Position` without `Agent` exist in the command checks (`commands.rs`, people on a tile block a
  placement), the bridge's space reads and the probe; a beast entity would be counted as a person there. Nearest-person
  searches for wolves use the agents' spatial index; deer searches scan ≤ 12 animals.
- The storyteller's state is `ColonyState.story`; fights (person or animal against person or animal) are
  `ColonyState.fights`, resolved by `CombatSystem` beside its V7 pairs; person-pair relations are `ColonyState.bonds`;
  every one of them is saved with `ColonyState` (format v5).
- Chronicle causes are a side table of the chronicle (`Chronicle::causes`, entry index → `CauseRef`), not a field of
  `ChronicleEntry`: the probe's chronicle files carry the entry size and the approximate bytes, and identity compares
  them for the baseline; the side table is empty there.

## D7 — The storyteller's schedule (goal 3.1)

- "Days 2–5 of a season" = 0-based days 1–4 of the colony's 6-day season. The storyteller runs at the first tick of
  each display day (08:00 with the colony's clock offset), so a pressure begins at 08:00 of its day and an `end_day`
  ends it at 08:00 (a 3-day cold snap covers three nights).
- The goal's kind weights are read per season: winter ColdSnap 600 / Beasts 400; summer Drought 600 / Beasts 400;
  spring Beasts 700 / no major 300; autumn Beasts 700 / Illness 300 (Illness is an autumn-winter pressure, so the
  spring's 300 is a quiet season). Autumn and winter also plan one minor Illness (not when Illness is the major).
- F1 (32/32 worlds see a cold snap and beasts in two years) cannot hold with independent draws (P(no snap in two
  winters) = 0.16). A kind is **due** when it has not come for a while: ColdSnap when none began in the last 24 days
  (a winter missed; only from the second winter on), Beasts when none in the last 18 days (three seasons). A due kind
  replaces the season's draw where its season and mercy allow (ColdSnap only in winter; in winter a due ColdSnap wins
  over a due Beasts).
- Mercy and gaps move the drawn day forward inside the season, never back; a day that no longer fits drops the
  major (`Storyteller::skipped`). "Winter's last days" for Beasts = the last two days. "Not before day 12" excludes
  the first spring and summer from every major (both seasons end by day 11).
- Illness: the onset picks one healthy adult (keyed, `STREAM_STORY`, slot 5); the spread is checked each morning for
  the night before: a person whose bed (`bed_of`) lies in the same roofed bedroom as a sick person's bed falls ill at
  200 ‰ (keyed per person and day). The illness pressure ends when nobody is sick any more.
- Costs: deaths of any cause while active; items lost = the extra firewood a snap burned (lit hours only) and the
  forage units a drought did not regrow (units per pulse × sources); work lost = sick person-ticks (counted daily)
  and drafted person-ticks during beasts.
- Evidence: `colony_story.rs` (schedule on seeds 0–7 × 48 days with mercy and gaps; F1's schedule half on seeds 0–7
  and the held-out 800–831, registered before any evaluation run).

## D8 — Animals, hunting and drafting: the numbers the goal leaves open

- Fights: every resolved hit is `hit_ticks` = 20 ticks apart for each attacker (the goal gives damage per resolved
  combat, not a rate). A person a wolf bites strikes back with what they hold unless drafted (then the draft's orders
  decide). A person–animal fight uses `colony.fights` (`Fighter::Person | Animal`); V7's `combat_pairs` stay
  person–person and untouched.
- Deer: the first goal draft (flee 4 steps of 2 tiles, then rest 10 steps while still wandering) made hunting all but
  impossible: a hunt's attack takes 60+ ticks and a deer that rested 30 ticks bolted before it ended (trace: one
  attack in three days). Now a deer rests **in place** for 40 steps (120 ticks) after a flight and wanders only
  within 16 tiles of where it was born (fleeing deer drifted out of the 96-tile job radius). A kill then takes about
  1,600 ticks for a level-0 hunter with a spear (`colony_wildlife.rs`).
- Hunting: hit chance 400 + 20 × level ‰ (to 800 at 20); a spear doubles it, capped at 950 ‰. A hunter without a
  tool fetches a spear, else a flint blade (a knife) from a stockpile within `axe_reach` (24). Elders do not hunt
  (default priority 0, like quarrying and mining).
- Drafting: the drafted state is `Seeking { Job }` (walking) or `Consuming { Job }` (holding) without a `Job`; the
  decision seam puts every drafted entity into its "may not yield" set, so `decision/system.rs` is unchanged (+0
  lines; budget +20). `JobAssignSystem` never assigns them (it takes `Idle` and leisure only). Critical need = the
  agent rule's `critical_preemption` (75).
- Meat: "eaten like foraged food" = meat on a stockpile cell or by a hearth goes into the settlement's food at 2 units
  a piece every hour (`meat_eaten` counts it there); the larder then feeds people as V7's stockpile does. Spoilage
  works on lots by kill day (a lot's meat spoils from the ground stacks in tile order; meat in someone's hands waits
  for the next day).
- Added while testing (`colony_wildlife.rs`) and calibrating (D14 has the rounds):
  - Wolves spread their targets: each step a wolf picks the nearest person no other wolf of the step has picked, else
    the nearest (round 0: two or three wolves on one drafted fighter killed them).
  - A hunter picks a wounded deer first, then the nearest (`hunt::pick`: `(hp, distance, y, x, id)`), so a hunt that
    lost its first target finishes the herd's wounded deer instead of starting over.
  - `tracking` is used (practice and mood note) once per attack, not once per hunt.
  - A hunter holding a flint blade swaps it for a spear on the way when one is stocked within reach (`hunting_fetch`).
  - `WorkType::Hunt` comes right after Build in the work order (craft, build, hunt, quarry, mine, chop, split,
    haul): placed after Mine it rarely ran, since quarrying and mining always had work (round 2: hunts were rare
    on every world). The tests pinned to the order were updated in the same commit.
  - Deer live as a herd: the first deer are placed within `deer_home_tiles` (16) of one keyed centre, and a new deer
    is born near the home of a living one, so a hunt zone over the herd keeps its deer (round 3: random respawns
    over the whole forest emptied the zone). The centre lies within `deer_herd_max_tiles` (48) of the hearth when
    the forest allows (round 4: seed 6's herd sat in a map corner 110 tiles out, beyond the 96-tile job radius, and
    no deer was ever hunted).
  - Arming a drafted person takes a spear or blade from a stockpile without a reservation (the drafted person holds
    no job, and the reservation invariant allows reservations only to job holders; seed 5 of round 0 failed F10).

## D9 — Hunting and hides are gated

- `tracking` gates `Designation::Hunt` (a gate-only variant: hunt marks live in `ColonyState::wildlife`, never on a
  tile), `hafting` also gates `haft_spear`, `hide_scraping` also gates `scrape_hide`, `make_hide_wrap` and
  `make_fur_cloak` (knowledge.ron). The M4 rules have no hunting and no such recipes, so M4 worlds are unchanged.

## D10 — The knife, the dressing pass and what a worn top is

- A knife recipe (`scrape_hide`, the recipe file's `knife: true`) needs one flint blade on the workshop's shelf
  besides its inputs (`Recipe::needs`): the bill supplies it like an input, the craft does not consume it, and each
  craft's start wears it one use (`ColonyState::knife_wear` per workshop); the 8th use takes it off the shelf, used
  up (ledger consumed, chronicle `ToolBroke` naming the crafter). The blade on the shelf is counted as shelved by the
  ledger, so the invariant needs no new term. A crafter keeps the axe in hand.
- Butchering wears a flint blade only when the hunter holds one as the tool; a hunter without a tool fetches a spear
  first, a blade only when no spear is stocked (so most kills give 2 meat once spears exist; F4 counts meat eaten).
- Dressing: hourly, while the order is on; tops on stockpile cells warmest first; the neediest adult (highest
  `Hardiness` permille, i.e. the lowest cold tolerance, then `AgentId`) among the adults free at that pass who wear
  something colder gets it. The first version let the top wait for the neediest person while they were busy; a pass
  runs hourly and a busy person is rarely free at that minute, so wraps lay on the stockpile for days (round 5,
  seed 5: 3 of 6 wraps unworn at day 48); now a busy person is served at a later pass. Children are not dressed
  (children take no jobs); recorded as a limit.
- A worn top is counted as held by the ledger (`ledger_of` adds worn tops to `held`); at a death it is left on the
  ground like a held tool. Tops never wear out in M5.

## D11 — Mood and quarrels: the numbers the goal leaves open

- Kin base: parent and child, siblings (a shared known parent) and partners have base relation `kin_base` = 500, so
  they never quarrel (goal 3.4: kin base ≥ 300). Non-kin pairs have a keyed base in −300..=300 (`chemistry`,
  `STREAM_QUARREL`, the pair's lineage keys); a relation drifts back to its base by `relation_drift` = 10 a day.
  Without a negative base no pair of strangers ever quarrelled on a fed colony (round 0: 0 quarrels on 6 of 8 worlds).
- The night window for the sleep and cold facts is the colony's sleep hours, 20:00–06:00; "last night" is read from
  08:00. A breakdown lasts `breakdown_days` = 1: the person keeps no job for the rest of that day and the next.
- A quarrel's cause is the initiator's largest negative modifier among hunger, cold, sleep, grief and pressure.
  When there is none, or the pair already quarrelled that day, the cause is the relation: cause code
  `RELATION_CAUSE` = 100 with a cause link to the pair's previous quarrel (`Bond::last_quarrel`), so every
  relation quarrel's "why?" chain has at least two links: the quarrel and the relation, then the previous quarrel
  and its cause when there was one. A pair of strangers with a negative base can quarrel over the relation before
  any modifier exists (calibration r7, seed 2: a quarrel at tick 0); its chain is the quarrel and the relation.
- Hostility lives in V7's `relationships` (permille of the 0–1 scale); the M5 bond (`colony.bonds`) holds the
  relation, its day and the last quarrel's chronicle index.

## D12 — The M5-U script, as resolved

`tools/v8/m5_u_script.json` is M4-U's economy (stockpiles, chop, quarry, mine, workshops, knap and axe bills, house,
storehouse, the teach order) with the goal's M5 entries, which the goal gives by day; the script places them by tick
so the colony has the workshop each needs:
- `Maintain 2 StoneAxe` (M4-U's) and `Maintain 6 FlintBlade` (M4-U's knap bill moved from 10 to 6, the goal's count);
- tick 1,800 (day 1, 06:00 after the workshop): `Make 4 Spear` (`haft_spear`, the hafting gate);
- tick 3,240 (day 2): a Hunt zone 33 × 33 (half 16) centred on the home of the deer nearest the hearth
  (`deer_forest` resolver);
- tick 4,680 (day 3): a drying rack, a standing `scrape_hide` bill (make 12) and `Make 6 HideWrap`;
- every day after a pack comes: draft four healthy adults (armed first, then nearest the hearth) and send them at the
  wolves (`draft_party`); when no wolf is left, undraft everyone (`drafted_all`, condition `no_wolves`);
- the dressing order on from the start (the default).
`tools/v8/m5_u_nodraft_script.json` is the same without the draft entries (F3b). The control is the same script with
`--story off` (no storyteller; wildlife, hunting and mood stay on).

## D13 — Scorer registration (`docs/v8/m5/m5_score.py`)

The bars as the scorer reads them are registered in its docstring before the first held-out run; changes after the
calibration rounds started:
- F9a is pooled (Σ births / Σ woman-years over the control's worlds), not per world: a world has 0–4 births in two
  years, so a per-world rate in 0.20–0.30 is noise (M1.5 D1 measured it pooled as well). The bar holds for all
  worlds or none.
- F2a uses the hearth's usable firewood (the settlements' wood plus feedable firewood items), sampled hourly during
  each cold snap, against one day's burn at the snap's rate.
- F3a counts a pack beaten when no wolf of it is left on the map and at least half of it was killed, with no death
  while any pack was on the map (the goal's "kills the pack": a pack that has lost half its number retreats, goal
  3.2, so killing every wolf is not on offer).
- F4's "hides ≥ 6" was missing from the first scorer and was added after calibration, before the first held-out
  run: the `hide` items created (one per deer killed; wolf hides are `wolf_hide`). It changes no calibration verdict
  (r7: hides = deer killed on every seed).

## D14 — Calibration rounds (seeds 0–7, 48 days; ≤ 8 rounds)

Each round: pressure (M5-U), control (`--story off`) and no-draft runs, `$DATA/cal/<round>/`, scored by
`m5_score.py`. Bars scaled to 8 worlds.

| round | change before it | F1 | F2a | F2b | F3a | F3b | F4 | F5 | F6a | F6b | F7 | F8 | F9a pooled | F10 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| r0 | first draft | 8 | 8 | 8 | 2 | 8 | 0 | 0 | 8 | 4 | 1 | 0 | 0.291 | 7 |
| r1 | draft arming without a reservation; wolf target spreading | 8 | 8 | 7 | 7 | 8 | 0 | 0 | 8 | 0 | 1 | 0 | 0.313 | 8 |
| r2 | chemistry ±150 → ±300; relation quarrels linked to the previous quarrel; a hunter swaps a held blade for a spear | 8 | 8 | 8 | 8 | 8 | 0 | 0 | 8 | 0 | 5 | 0 | 0.338 | 8 |
| r3 | Hunt right after Build in the work order | 8 | 8 | 8 | 8 | 8 | 0 | 0 | 8 | 0 | 7 | 0 | 0.364 | 8 |
| r4 | deer as a herd, births near the herd | 8 | 7 | 7 | 7 | 8 | 6 | 5 | 8 | 2 | 8 | 0 | 0.443 | 8 |
| r5 | the herd's centre within 48 tiles of the hearth | 8 | 8 | 8 | 8 | 8 | 8 | 6 | 8 | 1 | 8 | 0 | 0.327 | 8 |
| r6 | dressing serves the neediest free person | 8 | 8 | 7 | 6 | 8 | 8 | 5 | 8 | 3 | 7 | 0 | 0.322 | 8 |
| r7 | a worn top spares cold hours (D15) | 8 | 8 | 7 | 6 | 8 | 8 | 5 | 8 | 3 | 6 | 0 | 0.322 | 8 |

Bars at 8 worlds: F1, F9a, F10 8; F2b, F4, F6a, F7 7; F2a, F3a, F3b, F5, F8 6; F6b 2. Each round's scorer output is
`$DATA/cal/<round>/score.txt` (re-scored with the final scorer), the per-seed appendix `appendix.json`.

Calibration ends with r7 (8 rounds, the goal's limit); the held-out evaluation runs the r7 code. At r7 the bars
below their 8-world scale are F5 (5/8), F7 (6/8), F8 (0/8) and F9a (pooled 0.322 against 0.20–0.30); F2b, F3a are at
the bar. M4 under its own M4-U script, measured the same way: 0.336.

Round 4 seed by seed: seed 6 never hunted (the herd 110 tiles out; fixed for r5 by `deer_herd_max_tiles`); seed 4
lost all ten people to exposure (5 in the control), with meat never reaching the larder — under investigation for r5.
F8 (≥ 10 knives used up by day 48) is 0/8 in every round: a world makes 12–15 knife uses in 48 days (scrape_hide
and butchering), one or two knives' worth at 8 uses a knife; the goal's numbers allow ~2 knives, not 10, without a
larger hide sink than six wraps. Not tuned away (recorded as a limit). F9a: M4's colony under the M4-U script,
measured the same way on seeds 0–7 (`$DATA/cal/m4base`), is 16 births / 47.6 woman-years = 0.336, already above the
0.20–0.30 band at 8 worlds; the held-out 32 decide, and the report compares with M4's rate.

## D15 — What a worn top does against the cold

The goal gives the hide wrap "exposure cold ×700 ‰" (fur cloak ×500 ‰), and F5 measures cold *events* by group. The
first version scaled only the exposure damage; the `Cold` marker comes from the hearth being out, the same for
everyone of the settlement, so clothed and unclothed people had the same cold hours by construction (round 6, seed
3: 8 vs 12 cold hours of 600). Now the factor acts on the cold hour itself: when the fire is out, a wearer is cold
that hour with chance `Top::cold_permille` (one keyed roll an hour, `STREAM_APPAREL`, subject = the lineage key),
and a cold hour bites as for anyone. Cold hours, the mood's "cold last night" and exposure damage all scale by the
factor on average (`colony_hides.rs`: 678 and 481 ‰ of 1,920 cold hours on seed 2). Consequence for F5's second
half (clothed rate ≤ half the unclothed rate): at 700 ‰ the expected ratio is 0.7, so a world passes that half when
its fires stay lit through every winter night (both rates 0) or by chance; this follows from the goal's own factor
and is reported, not tuned.

## D16 — The art line's final state (goal §5.2, §12) and the next milestone

Read from `origin/v8/art-prod1` (not merged; `art/**` and `docs/v8/art*/**` untouched here):
- ART-PROD2 (`docs/v8/artprod2/PROD2_REPORT.md`, base `eb428ce2`): **S1 FAIL, route I not adopted** — 26/36 looks,
  adults 13/18, children 13/18; R1 (two fresh reviewers), P1 (524.8 FPS, 200 people, vsync off) and D1 (re-slice
  reproducible) passed.
- ART-PROD3 (`docs/v8/artprod3/PROD3_REPORT.md`, head `159ef0f9`, 2026-10-23 10:52): **route I adopted** at S1
  34/36 (adults 16/18, children 18/18) under the planner/owner's waiver of the adult 17/18 bar for the two
  `adult_M_*_long_tied` looks (they fall back to the bun of the same age, sex and top); `TEMPLATE_V3` is the
  character standard and `route_i/` with `MANIFEST.json` the asset source; zero new generations; no game code.
- The goal's rule ("다음 → M-ART2 if PROD2 adopted, else M6 prep"): the adoption came one pass later (PROD3), on the
  same route and evidence plus the waiver, so the condition the rule names — route I adopted — holds. ROADMAP's
  다음 is **M-ART2** (route I install, the 64-px camera, environment tiles), with M6 prep after it. ADR 0018 records
  PROD2's failure and PROD3's adoption.

## D17 — Code review fixes (`$DATA/review/code_review.md`)

A fresh reviewer read `git diff 08f59813 6f1713b4 -- rust/crates` and verified its claims on an exported copy. All
seven findings are fixed; each has a test that fails without its fix (checked by reverting it):
- **B1 (blocker)**: `hunt::pick_up_tool` removed whatever reservation the stack had and took any kind. A drafted
  person arriving at an arming stack another worker had reserved since deleted that worker's reservation (the debug
  build's invariant check panics in `JobExecuteSystem`; Godot loads the debug library). Now it takes an `owner` and
  accepted kinds: it releases only its owner's reservation, leaves a stack reserved by someone else, and takes only a
  weapon (the draft) or a spear / flint blade (a hunter). Test
  `a_tool_pick_up_leaves_another_workers_reserved_stack_and_takes_only_its_kinds`.
- **M1 (major)**: an illness had no immunity, so in a bedroom of about eight sleepers it sustained itself (the
  reviewer's probe: active on 61–78 % of days over a year). Who has had it during a pressure is immune for the rest
  of it (`story.touched` holds every onset). With that the pressure ends by itself within people × 3 days, so no cap
  was added. Test `an_illness_in_a_big_bedroom_strikes_each_sleeper_once_and_ends`. The evaluation's bedrooms hold
  four beds and every held-out illness lasted 3 days, so the held-out numbers are not expected to move from this.
- **m1**: a missed attack set the job back to `ToSource` while the state stayed `Consuming`, so the execution system
  cancelled it as `Interrupted` (17 hunt jobs for 15 attacks). A miss now goes back to seeking the deer. Test
  `a_missed_attack_keeps_the_hunt_job` (0 of 7 misses kept the job before).
- **m2**: a fight's damage was fixed when it began; a person's blow now uses the weapon held at that hit (a spear
  that broke mid-fight hits for 4). Quarrel fights keep their fixed damage.
- **m3**: a drafted person holding a non-weapon tool (a knife) never armed; arming now asks `draft::armed`.
- **m4 (C3)**: `wolf_edge` scanned the whole map every 3 ticks while wolves lived; the pack's edge is found once when
  it comes (`Wildlife::pack_edge`) and searched again only when it can no longer be reached.
- **m5**: knife wear was kept per workshop, so a new blade inherited a spilled blade's uses; `BillSystem` drops the
  count once no blade is shelved there. `colony_hides.rs` checks both moments.
These change behaviour, so the held-out seeds are measured once more on the fixed build ("after the reviews" in
`M5_EVALUATION.md`, the final simulation); none was tuned on them.

The numbers review (`$DATA/review/numbers_review.md`) matched every recomputed count and seed list; its findings on
the measurement and the text are applied to `M5_EVALUATION.md` and the probe (the pack window; `m5.rs`).

## D18 — The quarrel check's cost (goal §4, C3; found by the F11 bench)

The first F11 session (`$DATA/bench5`, 14:02–14:06, quiet: 1-minute load 3.4–5.4) put the tick mean within +10 % of
"M4 re-measure" on every scene (largest ×1.043, N = 2,000) and the M5 systems at 91 µs/tick at N = 1,000, but p99
rose ×1.22 (N = 200), ×1.38 (N = 1,000), ×1.63 (N = 2,000), ×1.40 (64²) — the ledger rule fails on p99. The cause is
`QuarrelSystem`: 205 µs a run at N = 200, 2.4 ms at N = 1,000, 8.1 ms at N = 2,000, every 30 ticks (so in the p99), and
80 µs/tick at N = 1,000 against the goal's 40. The benchmark camp is a crowd (at N = 1,000, 279 people on one tile and
84,230 pairs within 2 tiles); gathering every person's neighbours from 3×3-tile cells was O(N²) there.
Choice: each person checks at most `MoodRule::quarrel_partners_max` = 8 partners a run — the nearest later ids
within `quarrel_tiles`, by (distance, id), found ring by ring (own tile, then distance 1, then 2) and stopping at 8;
they are then checked in id order as before. Where 8 or fewer are near, the checked pairs and their order are
exactly the old ones; only crowds change. A run costs 136 µs at N = 200 and 767 µs at N = 1,000 (a timing probe on
the bench world, 60 runs). Test `in_a_crowd_each_person_checks_only_the_nearest_eight_later_ids` (40 people on one
tile: every quarrel spans ≤ 8 id ranks; without the cap some span more). This changes behaviour, so the held-out
seeds are measured once more ("final" in `M5_EVALUATION.md`).
