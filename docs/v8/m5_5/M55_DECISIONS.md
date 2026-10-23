# M5.5 decisions

Numbered decisions of the M5.5 pass (`docs/v8/m5_5/M55_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m55-data/` (`$DATA`), outside the repository. The goal says "never ask the
owner": every open question is decided here.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/main` = `cb58a801` (M5 report; gate green at `ef1bf3b7`); `v8/wip` at the same commit,
  checked with `git rev-parse` against `origin/*` at the start (2026-10-23 22:44 KST; the local branch ref
  `v8/main` is stale at `4874c9dc` and is not used — `v8/main` moves on the remote by explicit refspec).
- Choice: base worktree `../new-world-m55-base` detached at `cb58a801` with the main clone's `rust/Cargo.lock`;
  `$DATA` = `/Users/rexxa/work/m55-data/`; an owned `caffeinate -i` for the session; `CARGO_BUILD_JOBS=8`, tests
  `--test-threads=8`, probe `--workers 6`. Start 22:45 KST; the 22-hour target is 2026-10-24 20:45 and the 26-hour
  hard stop 2026-10-25 00:45. Base data (`$DATA/base/`): release binaries of `cb58a801` (`bin/`), the identity
  capture (`identity/`, with the base `perf.sh` run in `identity/perf/`), the soak (`soak/`) — `$DATA/base_capture.sh`.
- The host is shared (the art session runs ART-STRUCT in parallel); timings are labelled with their load.
- Evidence: `$DATA/TIMELOG.txt`, `$DATA/base_capture.log`.

## D2 — Art record (goal §5.2)

- Read from `origin/v8/art-env1` (`429a1800`, `docs/v8/artenv1/ENV1_REPORT.md`): ART-ENV1 ended **PARTIAL** — item
  style 12/13, deer readable / wolves not (neither adopted), built pieces 13/23 accepted with seams 246/256 failing,
  scene readability failed, FPS passed. The owner's verdict (goal §2): the structures look unnatural (mixed
  projection). ART-STRUCT runs on `v8/art-struct`, which is not on the remote yet (its data folder
  `/Users/rexxa/work/artstruct-data` exists). Route I is the adopted people standard (ART-PROD3, 34/36).
- Choice: ROADMAP gets ART-ENV1 (끝·PARTIAL) and ART-STRUCT (진행 중) rows; HISTORY and ADR 0018 get the record. No
  art branch is merged and nothing under `art/**` or `docs/v8/art*/**` is touched. ART-STRUCT's state is read again
  in Part 6 for ROADMAP's "다음" line (goal §11: M-ART2 when the owner approves ART-STRUCT, else M6 prep).

## D3 — ADR 0016 amendment: the memory instrument and the RSS outer bound (goal §3.4, §5.3)

- Context: M5 D19 moved the C3 per-agent and per-tile memory ceilings from RSS to the allocator counter, and did it by
  stopping a running gate. The change was sound (RSS on macOS read 83.0–117.5 B per tile for one binary across
  sessions; the allocator read 63.1 B every time) but the contract did not say it, and fragmentation is real memory
  that the allocator counter does not see.
- Choice: ADR 0016 gains an amendment: the ceilings judge the allocator counter when the run has it; **RSS keeps an
  outer bound** — `worldsim_perf bench` runs, after its timed repeats, 5 build-only processes each of N = 200,
  N = 2,000 and the 64² scene (`bench-one --build-only REPEAT`: build, read RSS and the allocator, exit; about 0.1 s
  each), pairs them by repeat, and judges the median of the 5 per-repeat slopes against 5,120 B per agent and 128 B
  per tile (`rss_outer` in `bench.json`; ceilings `rss_per_agent_le_5120`, `rss_per_tile_le_128`). Build-only runs
  never enter the timed medians (`summarise` splits them by the `build_only` flag, so `bench-summarise` of an
  interleaved session handles them too). Without build-only runs the bound is `n/a`, which `bench_ledger.py check`
  fails like any ceiling that is not PASS (absence is a failure). `bench_ledger.py check` prints the bound every run
  and the gate summary carries the line (`rss outer`).
- The rule "never change an instrument while a gate runs" is written into ADR 0016's amendment and the pitfalls skill
  (Part 6): a gate that finds an instrument wrong is stopped, the finding recorded, the instrument changed in its own
  commit, and the gate restarted from step 1.
- Evidence: `bench.rs` test `the_rss_outer_bound_is_the_median_of_the_paired_build_only_repeats` (five repeats with an
  outlier: the median is 5,120 B per agent exactly — PASS at the bound — and 129 B per tile — MISS; without
  build-only runs: `n/a`). Earlier sessions' RSS per tile (M5 gate `ef1bf3b7`: (9,296 − 5,504) KiB × 1,024 / 32,768 =
  118.5 B) sit under the 128 B bound.

## D4 — M5 owner decisions recorded (goal §5.4)

The owner's decisions on the M5 report (goal §2 and §5.4), each implemented in Part 1 and recorded in an ADR
amendment:
- **No pressure overlap** (ADR 0027): illness never starts during a major or within 2 days after one; a major waits
  while 2 or more are sick.
- **The undrafted flee** (ADR 0028): drafting must matter; an undrafted person near a wolf runs to a roofed room or the
  hearth and defends at half damage.
- **The hunter carries the meat** (ADR 0028): the kill chains a haul of the meat to a stockpile.
- **Knife = 4 uses** (ADR 0029), **hide wrap = 1 cured hide** (ADR 0029).
- **Mood and quarrels may be tuned** within the goal's limits (§3.1.6; ADR 0030): pressure modifiers up to ×2,
  breakdown threshold to 150, tantrum threshold to 300, the quarrel formula's constants.
- **Deer art yes, wolf art pending**: the art line may install ART-ENV1's deer; the wolf stays on the code-drawn
  placeholder until a wolf passes review (no art work in this pass).

## D5 — V7 migration code is not used for migrants (goal §2)

- Context: goal §2 asks to inspect `components/settlement_migrant.rs` and `runtime/settlement/` (V7 migration and
  fission) before reviving them.
- Finding: `SettlementMigrant` is a marker for a non-member walking to a simulated settlement through the decision
  system's `Seeking { Agent(member) }` arm and joining by proximity; fission splits a simulated settlement. Both
  assume both ends are simulated villages, and the colony runs with fission off (ADR 0014).
- Choice: migrants are new colony code (`runtime/neighbours/`): they are spawned as colony people at a map edge and
  walk to the hearth with the job layer's walking state (as drafted people do, the decision seam skips them while
  they wait); accepting a wave is a command. No colony code depends on the V7 migration path.

## D6 — The ledger rule for this pass

- The gate's bench compares with the latest quiet rows of another pass (`check --pass M5.5`): M5's rows appended at
  its report (`55c638c0`, pass "M5"). M5.5 appends its rows in its report step with `--gate-log`.

## D7 — Calibration and held-out seeds (goal §0, §10)

- Calibration on seeds 0–7 only, at most 8 rounds, each logged here (D-numbered rounds table) with what changed.
- Held-out seeds 900–931, grepped before use: no earlier pass used them (M3 400–431 and 600–631, M4 700–731, M5
  800–831). They are measured once at the end (Part 5) and once after the review fixes; never used to tune.

## D8 — Rule versions and the save format

- `ColonyRule::M5_5` = M5 with `StoryRule::M5_5` (no stacked pressure), `WildlifeRule::M5_5` (flight, the hunter's
  haul, a 4-use knife) and `MoodRule::M5_5` (the calibrated mood); `WorldRules::colony` uses it. The M5 constants stay
  as they were, so tests that pin M5 behaviour on an explicit `…::M5` rule keep their meaning (`colony_hides.rs`'s
  8-use knife). Every new rule field has `#[serde(default)]` with the M5 value.
- The recipe change (a hide wrap from 1 cured hide) lives in `recipes.ron` and is not versioned (recipes never were).
- `sim_systems::save::FORMAT_VERSION` 5 → 6 (`Fleeing`, the new tallies and wildlife fields, the chronicle's need
  samples; Part 2 and 3 add neighbours, migrants and caravans under the same version).

## D9 — No stacked pressure, as built (goal §3.1.1, ADR 0027 amendment)

- A planned pressure that may not begin today waits **a day at a time** (`story::held_back`): an illness while a major
  is active or until `stack_gap_days` = 2 after one ended (end day exclusive: a major ending on day E lets an illness
  begin on E + 2), a major while `major_wait_sick` = 2 or more people are sick. An illness may wait until its season's
  last day; a major until its window's last day (day 4 of the season; for beasts in winter the winter-end mercy). Past
  that it is dropped (`skipped`). Waiting only moves a pressure later, so mercy and gap rules stay true.
- `simulate_schedule` applies the illness half (it has no people, so no major waits there).
- Evidence (`colony_story.rs`): seeds 0–7 × 48 days, 14 illnesses under M5.5 and none during or within 2 days after a
  major; M5's schedule had stacked 15 illnesses on a major. A drought planned with two people sick waits and begins the
  day after one recovers; under M5 it began at once.

## D10 — Flight, as built (goal §3.1.2, ADR 0028 amendment)

- `FleeSystem` (123, every 3 ticks, `WildlifeRule::flee`): wolves that **hunt or bite** (a retreating pack is no threat)
  are the threats; the people within `flee_tiles` = 8 of one come from the agents' spatial index (chunk rings around
  each of ≤ 4 wolves). In `AgentId` order: drafted people and people at a critical hunger or thirst (the draft's
  `critical_preemption`) are left alone; someone standing in a roofed room is safe (wolves look only outside) and stays;
  anyone else **begins a flight** — the job is cancelled with `CancelReason::Needs` (flight is a survival choice, the
  reason the decision seam already uses for flight), the activity removed, `Fleeing { until = now + 30, from, to, since }`
  added and the person walks (`Seeking { Job }` + `SeekTarget`, as a drafted person walks).
- The refuge (`flee::refuge`): the nearest tile `(distance, y, x)` of a roofed room the person can reach; else the
  hearth; else the tile 8 steps directly away from the wolf. On arrival they hold (`Consuming { Job }`). While a threat
  is within 8 the flight is renewed each run; otherwise it ends at `until` (the person is `Idle`).
- The decision seam (`jobs::seam::holders`) puts fleeing people among those that may not yield (+7 lines in
  `seam.rs`); **`decision/system.rs` is unchanged (+0 lines; budget +20)**. Drafting a fleeing person ends the flight.
- Defence: a fleeing person's blows at a wolf are `flee_defence_permille` = 500 of the weapon's
  (`hunt::defence_damage`, used where the bitten strike back and at every resolved hit). Wolf strength is unchanged.
- F2c's measure (`probe/m55.rs`, registered here before any held-out run): an **episode** begins when an undrafted
  adult outside a roofed room comes within 8 of a hunting or biting wolf (and was not within 8 the tick before); it
  **moved away** when within 30 ticks the person stands farther from the wolf's tile at the episode's start than
  where they stood then, or inside a roofed room. Pooled over the worlds: moved away / episodes.
- Evidence (`colony_wildlife.rs`): the flight begins at the first run (≤ 3 ticks), the person runs for the roofed room,
  farther from the wolf and closer to the room after 30 ticks, without a job; a drafted person beside them does not flee;
  with no roofed room the refuge is the hearth; a fleeing spear-holder hits for 6 instead of 12.

## D11 — The hunter carries the meat home, as built (goal §3.1.3, ADR 0028 amendment)

- At the kill (`hunt::step` → `haul_home`) the hunt turns into a `Haul` of up to `hunter_meat` = 3 meat (and the meat's
  carry limit, and the cell's room) from the meat's tile to the nearest stockpile cell with room (`(distance from the
  meat, y, x)`, not reserved, reachable from the hunter). The hunt counts as done and the haul as taken; it walks and
  drops as any haul (the job layer's existing arms; the ledger needs nothing new). No cell → the hunt just finishes.
- The goal's "the hunter carries up to 3 meat + 1 hide": a `Carry` holds one kind (M1), so the hunter carries the
  meat — the part that spoils — and the hide waits for a hauler. "Extra stays and gets a normal haul job at haul
  priority + 1": every tile a kill filled (meat, hide, a wolf's hide) is noted in `Wildlife::carcasses`, and the
  `Haul` work type takes those stacks **before any other loose stack** (one step ahead of ordinary hauling, within the
  haul work type; the work priority table is unchanged). A tile leaves the list when nothing of the kill lies there off
  the stockpiles (checked hourly).
- `WildlifeTallies::meat_spoiled_afield` counts meat that spoiled off the stockpile cells (F3's "spoiled before
  reaching a stockpile"); `meat_hauls` counts the chained hauls.
- Evidence (`colony_wildlife.rs`): the hunt becomes a haul of meat at the kill, and within 60 ticks the meat lies on the
  stockpile or is in the larder. Smoke run (seeds 0–1, M5-U script, 48 days): 11–12 hauls for 15 kills each, 0 and 4
  meat spoiled afield of 31 and 37 dropped.

## D12 — One hide a wrap, four uses a knife (goal §3.1.4–5)

- `make_hide_wrap` takes 1 cured hide (`recipes.ron`); `WildlifeRule::M5_5.knife_uses` = 4. `colony_hides.rs`: five
  scrapes with two blades break the first at its 4th use (chronicle `ToolBroke` value 4) and leave the second with one
  use; three wraps use three cured hides. The M5 test with the explicit M5 rule (8 uses) still holds.

## D13 — Quarrel causes that can fail (goal §3.1.7, ADR 0030 amendment)

- A quarrel's cause is still the initiator's largest negative modifier (the relation when there is none), but it is
  now stored as a reference to a concrete record (`mood::cause_record`):
  - hunger, thirst, last night's cold, an outdoor night → a **need sample** taken at the quarrel tick
    (`chronicle::NeedSample { agent, tick, hunger, thirst (tenths), nights, entry }`, kept in the chronicle's new
    side list; `CauseRef::Sample { code, sample }`); `entry` is the cold snap or drought in force, which the "why?"
    chain follows;
  - grief → the death entry (`Died`); sickness, drafting, a pressure → the pressure's `PressureBegan`;
  - the relation → the pair's previous quarrel (`Bond::last_quarrel`), or nothing.
- `mood::check_cause(chronicle, index, 2 days)` resolves a cause only when: a sample is the initiator's, taken within
  the 2 days before the quarrel, and shows the need (hunger or thirst at its seek threshold, the cold or outdoor bit);
  a pressure was in force at some tick of the window (began before the quarrel, not ended before the window); a death
  is the initiator's kin (the dead person's recorded mother or father is the initiator, or the initiator's is the dead,
  or the death names the initiator as partner) within the window; an earlier quarrel names the initiator within the
  window. A modifier with no record (a relation quarrel between strangers with no recent quarrel, a draft without a
  pressure) does not resolve. The bridge shows a sample as its modifier (link kind 1), so the UI contract is unchanged.
- Evidence: unit test `quarrel_causes_resolve_only_to_a_fitting_record` (every failing case fails: another person's
  sample, an old sample, a sample without the need, a pressure ended four days before, a stranger's death, a relation
  without a record); `colony_mood.rs`: a live hunger quarrel stores the initiator's sample at the quarrel tick and it
  resolves. Smoke run (seeds 0–1, M5-U, 48 days): 18 of 23 quarrels resolve — the 5 that do not are relation quarrels
  with no recent record, which calibration addresses (D15).

## D14 — Calibration happens on the M5.5 world (goal §0, §3.1.6)

- The goal's calibration targets (breakdowns, quarrels) are measured with the storyteller on and off over two years;
  migrants change the population and traders the economy, so the rounds run on the M55-U script after Part 3 (the
  script needs caravans and migrant waves). Part 1 ships the knobs (`MoodRule` weights and quarrel constants, M5 values
  by default); the calibrated values land in their own commit with the rounds table here (D15).

## D15 — Calibration rounds (seeds 0–7, 48 days; ≤ 8 rounds)

Recorded with the calibration commit (after Part 3): the M55-U script needs caravans and migrant waves (D14).

## D16 — Visitors are not ECS entities; the Part 1 commit carries Part 2's code switched off

- Waiting migrants and caravans live in `ColonyState::neighbours` (waves and caravans: a group on one tile with a
  path), as animals do (M5 D6), and walk at a person's pace in `VisitorSystem`. Reason: V7's settlement proximity sync
  (`SettlementSystem::sync_membership`) admits every live agent within the proximity radius of the formation tile, so a
  wave spawned as agents would join the colony by standing at the hearth before the player answered; visitors as
  agents would also eat, drink, sleep and appear in every census. A wave becomes agents only when accepted
  (`neighbours::join`).
- Process note: Part 2's data types and systems were written while Part 1's checks ran, in files Part 1 also changes.
  The Part 1 commit carries them **switched off** (`ColonyRule::M5_5.neighbours = None`: every neighbour system
  returns at once, the commands answer `NotYet`); Part 2's commit switches them on. Part 1's evidence: identity PASS
  (`$DATA/p1_identity.log`: summary 228,748 values, chronicle 64 files, raw samples, determinism), the whole workspace
  in release 1,979 passed / 0 failed (`$DATA/p1_test_release.log`) before the Part 2 code; after it, with the rule
  off, the sim-core and sim-systems unit tests and the colony test files pass and clippy is clean.
