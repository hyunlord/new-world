# M3 decisions

Numbered decisions of the M3 pass (`docs/v8/m3/M3_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m3-data/` (`$DATA`), outside the repository.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/wip` = `f0ba4347`, `v8/main` = `4874c9dc` (checked with `git ls-remote` at the
  start, 2026-10-22 17:05 KST; `v8/art-trial2` = `2433a612`, `v8/art-trial3` = `b2a06115` on the remote).
- Choice: base worktree `../new-world-m3-base` detached at `f0ba4347`, with the main clone's `rust/Cargo.lock`
  copied in. Data in `$DATA` = `/Users/rexxa/work/m3-data/`. An owned `caffeinate -i` runs for the session.
  `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`. Start 17:05 KST on 2026-10-22; the 18-hour
  target is 11:05 and the 22-hour hard stop 15:05 on 2026-10-23.
- The host was shared and loaded at the start (1-minute load 20.9, 17 sessions); timings are labelled with the
  load they were taken under (M2 D23).
- Evidence: `$DATA/TIMELOG.txt`.

## D2 — The performance plan (goal §4, written before any pathfinding code)

- Budget (pre-registered, goal 3.1): at N = 1,000 with the M3 soak script (walls and rooms present),
  `AgentMovementSystem` + pathfinding ≤ 0.6 ms per tick mean; the whole tick stays under the C3 ceilings of
  `docs/perf/PERF_LEDGER.md`.
- Expected costs at N = 1,000: ≤ 200 path requests per 100 ticks; A* on 192² with walls ≤ 2,000 nodes expanded on a
  typical request; memo hit rate ≥ 60 % on hauling routes.
- Measurement: `worldsim_perf nav-bench` (`nav_bench`) reports path requests, mean nodes expanded per search,
  memo hits and the movement + nav µs per tick. It runs after Part 1 and after Part 3.
- If movement + nav exceeds 0.6 ms, apply in this order and record each step's effect: (1) path reuse for the same
  `(from_chunk, to)`; (2) the flow-field cache (6.4); (3) a cap on re-plans per tick (round-robin).

## D3 — Skill tie-break: rounds of (priority level, work type)

- Context: goal 3.4 — "within one priority level, `JobAssignSystem` orders agents by the work type's skill level
  descending before `AgentId`". M2 (D14) served each person's level-k work types in turn, person by person, so there
  was no single "work type" to order people by inside a level.
- Choice: under the economy the assignment runs one round per (level 1 … 4, work type in the tie-break order); a round
  serves the people whose priority for that work type is that level, ordered by their skill for it (highest first),
  then `AgentId`. Skill for a work type: craft = the better of knapping and woodworking, build = building, quarry =
  quarrying, chop = chopping, split = woodworking, haul = none (`assign::skill_for`). Without the economy (M1 rules)
  nothing changes.
- Effect on M2's behaviour: within a level, every person's craft work is now offered before anyone's haul work (work
  type before person); before, the first person in `AgentId` order took any of their level-k types before the next
  person tried any. The M2 colony runs move, and the M2 fixtures that pin colony outcomes are re-captured where they
  changed (D6). The baseline is untouched.
- Evidence: `colony_skills.rs::within_a_priority_level_the_more_skilled_person_takes_the_job` (fails with the sort
  removed: checked).

## D4 — Quality words: the band table decides

- Context: goal 3.4 gives five bands (0–19 조악, 20–39 보통, 40–59 좋음, 60–79 훌륭, 80–100 걸작) and the example
  "부싯돌 날 · 좋음 (62)", which the bands put in 훌륭.
- Choice: the band table decides (62 → "훌륭 (62)"); the example's number is read as a typo. `EconomyText.quality`
  formats "{word} ({value})" (`UI_QUALITY_FMT`, `QUALITY_BAND_0…4`); the stack line and the inspector's tool line use
  it. Test: `tools/v8/test_locale_trf.gd` (band edges 0, 19, 20, 59, 62, 79, 80, 100 and a stack line).

## D5 — The economy soak fixture

- Context: goal 3.4 — 20 days, chop marks renewed every 2 days (15 trees nearest the camp not yet marked), quarry 3
  nodes, workshops, bills `Make 20` blades, `Make 4` axes, `Maintain 40` firewood, `Maintain 10` blades; plus M3's
  build orders; gate step `soak` on 4 seeds asserting ledger, reservations, 0 deaths and determinism.
- Choice: `tools/v8/m3_soak_script.json` — the M2-U layout (6×6 stockpile 3 east of the hearth, knapping spot and
  wood bench south of it) with those bills; quarry marks on the nearest boulder and the 2 nearest flint outcrops (the
  blades need flint, the bench needs stone); no work priorities (the goal lists none). The probe gained `"every_days"`
  (an entry applied again every n days, resolved anew) and the resolver `nearest_unmarked_trees`. M3's build orders
  join the script in Parts 2–3. `tools/v8/soak.sh <out>` runs seeds 0..4, 20 days, `--check-determinism` and fails on
  any ledger or reservation violation tick, any death, or an idle colony (fewer than 2 workshops, no craft, no tree
  felled). Gate step 26.
- First run (Part 0 tree, `$DATA/try/soak0`): SOAK PASS; 72–77 crafts, 150 trees felled, 0 deaths, 10 alive per seed;
  4 seeds × 20 days in 12 s.

## D6 — Part 0 records: the art state, the M2 fixtures, the base data

- Art (goal 5.2; a subagent read the art branches with `git show origin/<branch>:<path>`, report
  `$DATA/logs/art_record_report.md`): ADR 0018's status, ROADMAP rows ART-TRIAL2b / ART-TRIAL3 / ART-TRIAL3b, HISTORY
  lines and the archive index's art-branch rows now cite `2433a612` (TRIAL2b), `352f4101` (TRIAL3) and `b2a06115`
  (TRIAL3b). Three readings differ from goal §2 and the branches' own reports win: TRIAL3's recommendation is the 2D
  artist only (route B is the pre-registered alternative, unjudged); "87/87" is 87 accepted layers passing the
  checker (22 checker tests), route A's set is incomplete and its playback readability not verified (verdict
  PARTIAL); TRIAL2b also failed people readability (C5, G1) and part yield (C7) with seasons open, so 05 is a
  conditional candidate. ART-TRIAL3b is not running: it **stopped** at its Part 0 tool check (`higgsfield account
  status` exit 4, "No workspace selected", no generation), recorded as 중단.
- M2 fixtures (goal 5.4): with the skill tie-break alone the whole workspace suite passed (1,854 tests, 0 failed,
  `$DATA/logs/p0_tests.log`), so no M2 fixture needed re-capturing; the tie-break changes M2 colony runs (D3) but
  no test pinned the moved values.
- Base data (goal 5.5, `$DATA/base/`): `bin/` release probe and perf at `f0ba4347`; `identity/` the identity capture
  (17:05–17:18); `m2u/` the base colony with the M2-U script, seeds 0–7, 20 days; `soak_first_run/` the soak
  fixture's first run on the Part 0 tree (the base probe cannot read `every_days`). The benchmark ledger base line is
  measured with the candidate in one interleaved quiet session (M2 D17/D23), so it is taken when Part 3 is done.
