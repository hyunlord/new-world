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
  `Hardiness` permille, i.e. the lowest cold tolerance, then `AgentId`) who wears something colder gets it; when that
  person is busy the top waits for them. Children are not dressed (children take no jobs); recorded as a limit.
- A worn top is counted as held by the ledger (`ledger_of` adds worn tops to `held`); at a death it is left on the
  ground like a held tool. Tops never wear out in M5.
