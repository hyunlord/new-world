# Pass 18 independent review

Two rounds at most (brief 0.3/9.2). Each round is a fresh built-in `general-purpose` subagent given only `PASS18_GOAL.md`, the diff from `8cbb697c` and the outputs (prompt: checks A–H — identity and pins; `subprofile` off; each 7.2/7.3 change and its proof; the R5/F8 procedure; `game_v23` and the elder-follow path; G verdicts; allowed paths; quiet-lock record). Reviewers were told not to run cargo, worldsim binaries or Godot while the Mac was being timed.

## Round 1 (2026-10-16 03:34–03:49 KST; official performance result and gate still pending)

Verbatim review: below, after the dispositions.

| id | severity | finding | disposition |
|---|---|---|---|
| A1 | MAJOR | I1–I6 had not been run on the final (Part 3) build | already running at review time (`probe/p18-4_final/`, binaries `cmp`-equal to the `57c47981` release build); G5 is judged on it |
| D5 | MINOR | official R5 margin is inside hold-to-hold noise; report all four stage-a runs | done in REPORT_PASS18.md and PERF_BUDGET.md |
| H1 | MINOR | shared-Mac record missed Art Pass 10's 01:38–01:41 hold (15 s overlap with my I6) | added to DECISIONS.md |
| C8 | MINOR | "pins for every candidate" too strong (C1 only stacked) | wording corrected in DECISIONS.md |
| E5 | MINOR | G3 top-up detection necessary but not sufficient | accepted limitation of the registered rule; no case occurred |
| E6 | MINOR | tests missing for the top-up split, `Some(&[])`, seed skip by an elder | three tests added (test code only) |
| F3 | NOTE | evaluator committed after results | committed script reproduces the tuning and evaluation reports byte-for-byte |
| D6 | NOTE | official hold also ran attribution and stage c | stated in DECISIONS.md and the re-judgement |
| E7 | NOTE | unranked partner not checked for `Elder` | latent assumption, documented |
| E8 | NOTE | tuning ran on an earlier build | family code identical; noted |
| B4 | NOTE | 7.2f's "reuse in the Idle cascade" done differently | deviation named in DECISIONS.md |
| A5 | NOTE | I6 has no Pass 17 reference | as recorded |

### Round 1 review text


Reviewer: fresh subagent. I modified no repository file. Scope: the brief `docs/revitalize/PASS18_GOAL.md`, `git diff 8cbb697c..e5e37acb` (branch `p18-part3`), the uncommitted working-tree edits (DECISIONS/ROUTINE/FAMILY, `probe/p18-3_eval/`, `probe/p18-3_tuning/`, untracked `tools/revitalize/{evaluate_pass18.py,family_runs_p18.sh,hold_p18_final.sh,hold_p18_part2.sh}`), and raw data in `/Users/rexxa/work/p18-data/`. I ran no cargo, no worldsim binaries and no Godot. My only checks were small python3 scripts over JSON, `git`, `stat` and `find`.

**Snapshot note.** Two commits landed while I was reviewing: `fec33d5d` (official R5/F8) and `7c91d4d8` (evaluation, G6, and the four tools committed). The brief for this round says the official quiet-lock result is **pending**, so I did not judge it. Where I looked at those commits at all, I say so (see D5 and the addendum).

#### Summary of findings

| id | severity | finding |
|---|---|---|
| A1 | **MAJOR** | G5 says "identity on the final build". The I1–I6 suite ran only on the Part 2 build `d3fc22f7`. No I1–I6 run exists for the Part 3 head (`57c47981`/`e5e37acb`, probe sha `f1073afc…`), which is also the live-game build. Only the pins ran on that build. |
| D5 | MINOR (escalates to MAJOR if the report hides it) | The development runs show hold-to-hold noise (±2 points on the mean ratio, and v20's own p99.9 moves 0.53–0.61 ms) that is as large as the remaining margin. One official run near the bars cannot by itself establish a robust pass or fail. The report must show it beside the three development runs. |
| H1 | MINOR | The shared-Mac record misses Art Pass 10's `baseline-W0-attempt1` hold (from 01:38:08 KST, which the art run marked contended). Pass 18's Part 1 I6 probe ran during its first ~15 s. A second overlap with the W200 hold (~10 s, also marked contended by the art run) is recorded. |
| C8 | MINOR | The claim "pins 68/68 for every intermediate candidate" is too strong. C1 (`82a81f0d`) has no pin log of its own; the first is `pins_c2` (C1+C2 stacked). The per-candidate set also leaves out `harness_revitalize_legacy_pin`, which ran once in the full release test on the Part 3 head. |
| E5 | MINOR | G3's "top-up exception" is detected as "the adult departed in a group of exactly `group_min`". That condition is necessary but not sufficient, and the trace reads `Elder` after the departure tick. The rule was registered in that form and no case occurred, so the verdict is unaffected. |
| E6 | MINOR | Unit tests do not cover the one path that may still split an adult–elder couple (the `group_min` top-up). They also do not cover `Some(&[])`, or the seed (as opposed to the fill) skipping a household because of its elder. |
| F3 | NOTE | The registration (`05beff9a`, 02:39:07, confirmed by the reflog) came before the evaluation (02:40:41). However, `evaluate_pass18.py` and the run scripts were committed only at 03:41, after the results existed, and the pre-registration evidence is local only (it was pushed later). |
| D6 | NOTE | The official hold is not byte-for-byte Pass 17's hold. The attribution runs, which include the subprofile build, run first in the same 51 s hold, and stage c follows. The order within each stage (three repeats, old then new) is Pass 17's. |
| E7 | NOTE | `select_departure` keeps any *unranked* mutual partner and does not check `Elder` explicitly. Today the unranked members are only elders (fission.rs L155–157, L567–583), so this is only a latent risk. |
| E8 | NOTE | Tuning ran on `997ed3ab` (Part 2 code at C4). Its family code is byte-identical to the final code (`git diff 997ed3ab 57c47981` touches only the four routine/decision files). The C5–C7 identity, however, was proven on v19–v22 only, not on v23. The tuning decision has no effect on the held-out verdict. |
| B4 | NOTE | Brief 7.2f also asked for the trip results to be reused in the Idle cascade. The change instead uses existence predicates plus a memo. That is behaviour-preserving, but it differs from the letter of the brief and should be named as a deviation. |
| A5 | NOTE | I6 (`game_v22`) has no external Pass 17 reference; base and candidates agree only with each other. DECISIONS records this. |

No BLOCKER was found. For every Part 2 change I checked against the code, I found no behaviour-preservation hole (details in C).

---

#### A. Identity and pins

- **Comparator and coverage.** I re-ran `tools/revitalize/compare_identity_pass17.py` myself. Results:
  - Base → Part 1 (feature-off) and base → final Part 2: I1/I3/I4/I5/I6 all `PASS: 0 differences; only ['ms_per_agent_tick','timing','wall_seconds'] excluded`.
  - Pass 17 references → base: I1/I3/I4 against `p17-0_base` and I5 against `p17-5_b_identity`, all PASS.
  - Sanity check that the comparator can fail: v21 vs v22 reports a difference at `$/days/10/food_tiles_finite_total/p90`.
  - I2 lists: `p17-0_base` = `p18-0_base` = `p18-2_final` (all 20 hashes).
  - Every summary has seeds 0..32, 120 days, `determinism_checked: true` and 32 runs. The flags are `identity_p18.sh:20`, identical to `identity_p17.sh`.
  - The hecs lock is the same in both trees (`hecs v0.10.5`, same `Cargo.lock` sha `775fbb33…`, `p18-0_base/hecs_and_lock.txt`).
- **Rebase claims.** I checked with `git diff --stat <pre> <post> -- rust`. Each pair is identical: `59f99527`/`d3fc22f7`, `4dabb1bb`/`57c47981`, `05beff9a`/`e5e37acb`, `2b6b60bc`/`2b5bdfd3`, `a53803af`/`7e6b0c31`, `f11301ee`/`59c92f70`. The binary hashes are consistent: `bin_p2f/plain/worldsim_probe` = `52451d32…` = `p18-2_final/binaries.sha256`, and `bin_p3f/worldsim_probe` = `f1073afc…` = `p18-3_eval/binary.sha256`.
- **Pins.**
  - `harness_revitalize_pass18_frozen.rs` has one `#[test]` per (scene, seed) for v21/v22 seeds 0–4, each checking fingerprint and `state_digest` at 1,440, 40,320 and 86,400 ticks, plus births and fissions. That is 10 tests.
  - It also has `harness_game_v22_rules_pinned_field_by_field` (155 `assert_eq!`, no struct literal).
  - The leaf added in Part 3 (`elder_partner_follows`) is not in the frozen file. It is asserted for `game_v22(seed)` in `sim-core/src/components/family.rs` (L149 `assert_eq!(v22.society.family, Some(V22_FAMILY))` plus the `const` assert), so every leaf is covered across the two files.
  - Logs: `pins_{c2..c7,c5a,c6a,c6b,p3,p3f}_release.log` are each 68 passed and 0 failed; `pins_p1` is 38/38.
- **A1 (MAJOR).** Brief 4.1 says "Run I5/I6 … again on the final build", and G5 says "I1–I6 and all pins identical on the final build". The final build is the Part 3 head. Part 3 touches code that runs in v17–v22 worlds:
  - `fission.rs` L602–614 (now always passes an `Option`);
  - `households.rs` L84–109 (new filter);
  - `FamilyRule` (new field, which changes the serde/Eq shape of `WorldRules`).

  My reading is that the `None` path is identical: with `partner_of` empty, the filter reduces to `rank.contains_key`, and `any(rank.contains_key)` equals `!is_empty()`. The summary's `rules` block does not contain the family rule. The pins on `bin_p3f` passed. Still, the brief requires the proof, and DECISIONS ("G5/G6 are judged with the final identity") does not say which build. **Fix:** run `identity_p18.sh` (all modes) with the `bin_p3f` binaries, or a fresh release build of `e5e37acb` that `cmp`-equals them. Compare against `p18-0_base` and commit the result as `probe/p18-3_final/`. It is untimed and takes about 75 minutes. Then state G5 on that build.

#### B. `subprofile` off

- **Macro.** `sim-systems/src/lib.rs` L35–53: with the feature off, `scope!($name, $body)` expands to `$body`. With it on, it is `Timer::start(); let value = $body; timer.stop(...)`.
- **Feature-gated sites** (all read-only with respect to simulation state):
  - `lifecycle.rs` L54–57 and L75: thread-local counters only.
  - `note_insert` at L117–119: `world.contains` and `world.get::<&T>`, reads only.
  - `apply_one` (L86–110).
  - `mod.rs` L88 and L119–124 (timers).
  - `agent_decision.rs` L709, L1136, L1175–1177, L2760, L2782, L3120 (timers); the counters in `start`/`place`; `sim-test/src/perf.rs` (log and summary); `subprofile_report.rs`.

  The only thing feature-gated code touches is the `thread_local!` in `subprofile.rs`.
- **Feature declarations.** The feature is declared with no `default` in either Cargo.toml. The sim-test `[[bin]] worldsim_perf` has no `required-features`. `sim-bridge` depends on `sim-systems` without features. The gate (`gate_pass18.sh`) uses no `--all-features`. The subprofile build lives in `rust/target/subprofile`.
- **Feature-off code motion in the final tree.**
  - `let removed = world.remove_one…; let _ = removed;` only extends the lifetime of a `Copy` value.
  - The routine maps are no longer eagerly evaluated (covered in C).
  - The Part 1-only motion (maps and `collect` before the literal) is gone in the final code.

  The feature-off proof is the Part 1 I1–I6 run plus the pins, and DECISIONS records a `cmp`-identical release build after adding scopes. Clean.
- **B4 (NOTE):** see the summary table.

#### C. The 7.2 / 7.3 changes

I checked each claim against the code. Everything below verified, except where noted.

1. **Keys (`Option<&Lineage>`).**
   - Every caller of `can_enter`, `start`, `deadline` and `end(…, Some(key))` passes the *row's own* lineage (`agent_decision.rs` L2060–2075, L2199–2210, L2558–2568).
   - The only other `end` path is `interrupt` → `end(…, None)` (lifecycle L131), where Pass 17 also did not rest.
   - A missing `Lineage` gives `can_enter` = false and a rest key of `id`, the same as Pass 17's `keys.get(&id)` / `.unwrap_or(id)`.
   - An `Option` fetch does not change archetype matching. Nothing in the decision system inserts or removes `Lineage`.
2. **Members.**
   - `member_of` is filled unconditionally under either rule (L905–912 firewood, L938–945 forage), in sorted settlement-id order with `or_insert`. That is exactly `members_of` (`mod.rs` L67–78). The "stock_target" caveat in the brief does not gate the fill.
   - `routine_members_own` is built iff `routine && firewood_rule.is_none() && forage_rule.is_none()` (L1155–1159), which mirrors the fill condition.
   - Nothing in `agent_decision.rs` or `routine/*` mutates `resources.settlements` (grep for `get_mut/insert/remove/entry/values_mut` on settlements and on `member_agents` finds nothing). So building at L905 or L1155 instead of at `RoutineTick::new` (L707) is equivalent. `member_of` has no write after L945.
3. **Entities.**
   - The map is built lazily from `world.query::<&Agent>()` on the first `invite` (lifecycle L146–157). `replay` reads `entities.as_ref()`, and invitations exist only after `invite` has built it.
   - No `spawn` or `despawn` occurs in the decision system (grep matches only tests).
   - The nested query borrows `Agent` immutably, which is compatible with the main query.
4. **Places cache and its inputs.**
   - `collect` (places.rs L63–119) reads `building_registry` (values and `get`), `building_kinds`, `building_owner`, each settlement's `formation_tile` and `member_buildings`, the grid size, and the River/Mountain terrain inside each 29 × 29 scan area.
   - `Inputs::{of,matches}` (L131–193) compares exactly those, including the settlement count.
   - None of these inputs is mutated by the decision system, so evaluating lazily at the first `place()` equals Pass 17's eager evaluation at `RoutineTick::new`.
   - `rewind` before the first `place()` of a tick has no cursors, so the stale cache is never read.
   - `next_to` = "Chebyshev distance 1 to a member of the set", which matches the `distance` const fn and is tested at L370–379.
5. **Cursor in place of `choose`.**
   - Pass 17 took `min_by_key((occ, rank, y, x))`. The rank is constant across tiles, and the lists are built `y`-outer, `x`-inner (ascending), so the result is the first tile with 0 occupants, else the first with 1.
   - Occupancy changes only at `start` (mod.rs L251) and `end` (lifecycle L221). `end` calls `rewind` on every cursor whose list holds the tile (binary search on the `(y, x)` key, L276–281).
   - **Invariants.** `zero` skips only tiles with ≥ 1 occupant. `one` advances only after `zero` has hit the end, when every tile has ≥ 1, so the tiles it skips have ≥ 2. Starts only increase counts, and every decrease rewinds. Cursors are per tick, so replay rounds share them, which is correct because occupancy persists across rounds.
   - Tests: `cursor_matches_choose_under_starts_and_ends` and `short_circuit_choice_equals_the_pass17_minimum`.
6. **Exit test.**
   - `bush_eligible` / `spot_eligible` / `gather_tile_exists` are exactly the filters of `choose_bush` / `choose_spot` / `nearest_gather_tile` (the local and non-local branches both collapse to `is_none_or`).
   - `is_some()` of a `min_by_key` over a filter equals `any` over the same filter.
   - The reorder `need || builder || social || meal || forage() || firewood() || gather()` is safe because the closures are pure (they borrow `claims` and the maps immutably while `social`/`meal` are evaluated).
7. **Trip memos and versions.** I grepped every write in the loop:
   - `claims`: L1955 (+= with a version bump at L1956);
   - `wood_claims`: L1998 (L1999);
   - `food_tiles`: L2585/L2591 and L2618/L2637, both inside a `get_mut` block that bumps at L2586 and L2619;
   - `wood_tiles`: L2670/L2672, inside `take > 0` with a bump at L2665.

   Other writes are pre-loop (L923, L930, L954) or post-pass on other maps (`gather_claims`, `wood_gather_claims`, L2877 and later), after the memos' lifetime. Nothing in the loop decrements claims. The memo keys cover all remaining inputs:
   - bush: `home` (the rule is constant);
   - spot: `(home, urgent)`, because `urgent_rule`;
   - gather: `(pos, local)`, where `local` includes the catchment, plus the constant `GATHER_SCAN_RADIUS`.
8. **`World::exchange` fusion** (lifecycle L45–68). I checked this against hecs 0.10.5 `world.rs`:
   - `exchange` (L781–805) calls `remove_target` (L747–768), which get-or-creates the *same intermediate archetype* as `remove_one`. It then calls `insert_inner` from that intermediate (L601–672): the target archetype is looked up by type set, `RoutineObservation` is a replacement, and the old value is dropped.
   - So archetype *creation order* is unchanged (intermediate, then final).
   - Entity order is also unchanged. Pass 17 appended the entity to the intermediate archetype and immediately swap-removed it as that archetype's last element, so no other entity moves. Both paths swap-remove from the source and append to the final archetype.
   - Activity is read out by `S::get` and not double-dropped.
   - The guard requires the group to be exactly `[Activity(None), Observation, Rest(Some)]` (the sort is stable, so per-id groups are contiguous) and the entity to have `Activity` and `RoutineObservation` and no `ActivityRest`. Any other group falls back to `apply_one` in order.
9. **Occupancy from sorted places** (mod.rs L132–142): it builds the same per-tile counts as the `+= 1` loop.
10. **Member lookup once** (L1288): `member_of` is not mutated in the loop.
11. **C8 (MINOR).** See the table. Smallest fix: reword DECISIONS to "pins per candidate from C2 on (C1 only stacked); the legacy pin covered by the full release test on the Part 3 head".

#### D. R5/F8 re-judgement procedure (official result pending)

- **Same procedure as Pass 17.**
  - `perf_p18.sh` differs from `perf_p17.sh` only in the owner string, the env prefix, stage c and the reuse list (`diff`).
  - `evaluate_perf_pass18.py` differs from Pass 17's only in the owner, stage c and the title.
  - Workload `stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed`, three repeats, old then new, medians, `quick` on base and final, slopes `--agents 100,500,1000 --days 2`.
  - The quiet-lock and wait tools differ only in owner, stamp and prefix.
- **Declared before it ran, and run once.** The "Decision (before any further measurement)" line is in the committed DECISIONS at `cefa3a15` (03:32:52 KST). `quiet.log` shows exactly one official hold, 03:38:06–03:38:57 KST (51.0 s, load 2.65, uncontended), after the final I1–I6 ended at 03:32:23. One caveat: `hold_p18_final.sh`, which defines what that hold ran, was committed only in `7c91d4d8` (03:41).
- **Development runs are reported, not selected.** All three (`perf/p18-2/development/{1,2,3}_*`) are committed with their stage JSONs. I re-read their headline checks:
  - stage a mean +16.6 % / +15.03 % / +16.7 %, p99.9 +49.0 % / +38.0 % / +52.4 %, all FAIL;
  - stage b PASS in all three.

  These match README.md and DECISIONS.
- **D5 (MINOR, reporting).** Those numbers show that hold-to-hold noise (±2 points) is larger than the distance to the 15 % / 35 % bars. Successive builds moved the ratio 1.166 → 1.150 → 1.167 while the code got cheaper. C7, the only change after the third development run, is a hoisted map lookup. Whatever the single official run shows, it should be reported as "official verdict X; the three development runs of near-identical code were …; the margin is inside measured noise". It should not be reported as "routine cost solved". **Fix:** one sentence in REPORT/PERF_BUDGET, plus a table of all four stage-a results side by side.
- **D6 (NOTE).** See the table.

#### E. `game_v23` and elder-follow

- **Rules.**
  - `FamilyRule.elder_partner_follows` (`rules/routine.rs` L104–106) is the only new public field in an existing struct.
  - `V22_FAMILY` = (300, 1200, false). `V23_FAMILY` = (200, 1200, true).
  - `game_v23` = `game_v22` with family replaced (`rules.rs`), tested in `family.rs::v23_changes_only_the_family_rule_of_v22`.
  - `WorldGenConfig::game_v23` = `game_v22`'s.
- **Wiring.**
  - `scenes.rs` `game_v23_scene` (same `draw_v18` variety as v22);
  - probe `args.rs` (`game_v23` now accepted, `game_v24` rejected);
  - `runner.rs` `scene_engine` and `twin_checked`;
  - `perf.rs` `--rules`, `stress_v23`;
  - live `build_game_engine` → v23 (`world_node.rs` L192–193, used by `init` L224 and `reset_world` L268);
  - `reset_world_stress` → `build_stress_engine` → v19 (L184–186, L281);
  - the bridge diff adds no getter keys.
- **`select_departure`** (`households.rs` L84–162, fission.rs L594–614).
  - With `Some(couples)`, a member is kept if it is ranked, or if it is the mutual partner of a ranked member in the same household. `households()` always groups mutual living partners, so `group.contains(partner)` holds for every couple.
  - Groups with no ranked member are dropped (L109), so elder-only groups can no longer sort first in the fill.
  - The seed (L129) and the fill (L141) add whole groups, and the size checks count the kept elders.
  - `min_adults` counts only `rank.row.1`, and elders are not in `rank`.
  - The leader is the lowest-id roster adult, and roster adult = `!child && !elder` (fission.rs L155–157), so an elder can never lead.
  - **The only path that takes an adult without their elder partner is the top-up** (L146–152, single ranked ids). **No path lets an elder leave without their partner at selection time.** Afterwards, the partner's death en route can leave an elder in the group; that is expected.
  - With `None`, the selection is exactly Pass 17's.
- **Trace keys** (`probe/family_events.rs` L81–107): `elders_departed`, `adult_elder_couples`, `adult_elder_couples_split`, `adult_elder_split_pairs`. They are computed from the members, the couples and the `Elder` component. `FamilyEvent::Departed` and `FissionRecord` are unchanged.
- **E5 (MINOR).** `evaluate_pass18.py` `elder_census` flags `group_min_top_up = adult_departed && size == 6`. Seed plus fill can also land on exactly 6, so this over-accepts. The `Elder` set is read after the departure tick, and the fission check runs at 06:00, so ageing in the same tick could reclassify someone. **Fix:** document the limitation in METRICS/eval.md. In a later pass, compute the pre-top-up size in the trace from `ranked` and `households`.
- **E6 (MINOR).** Add tests that cover:
  1. a case where seed and fill stay below `group_min` and the top-up takes an adult with an elder partner (asserting the split is the documented exception);
  2. `Some(&[])` ≡ `None`;
  3. the seed skipping a household because its elder pushes it over `group_max`.
- **E7 and E8 (NOTE).** See the table.

#### F. G1–G6

- **Recomputed from the raw traces.** I used `p18-data/eval/game_v2{3,2}/*_family_fissions.jsonl` and counted completed departures only:
  - v23: 24 fissions, 319 members, 16 elders departed, couples 310, couples split 0, adult–elder couples 54, adult–elder split 0.
  - v22: 24 fissions, 320 members, 0 elders, 220 couples, 10 split, 34 adult–elder couples, 10 adult–elder split. One v22 departure was pending and is excluded.

  This equals `eval.json` `elder_census`. My recomputation of `couples_split` from `couples` × `members` matched every row. v23 has 0 households split at departure (v22 has 12), which fits elders and Child-stage children leaving with their households.
- **Other G values.** G1 F1 = 796/1135 = 0.7013 [0.676, 0.727]. G2 and G4 come from Pass 17's `evaluate_pass17_stage_b` code, which is unchanged. The composition in `evaluate_pass18.py` uses the registered bars (F5 ≤ 0.05; the G4 rows show alive ±10 %, deaths ≤ +1.5, shelters ≥ 0.9, social ≥ 0.9, food and wood ±15 %). I did not re-run the 2,000-sample bootstraps (to keep CPU use low).
- **Registration.** The METRICS registration (`05beff9a`, author and committer time 02:39:07 KST; reflog `HEAD@{02:39:07} commit: register the Pass 18 rule table`) comes before the first held-out probe (`eval/RUNLOG.txt` 17:40:41Z = 02:40:41 KST). The registered G3 exception text is exactly what the script applies. G5 and G6 are outside the script (A1, D).
- **F3 (NOTE).** See the table. Fix: disclose in DECISIONS that the evaluator was frozen at tuning time (~01:19) and committed at 03:41. If a hash of it from evaluation time exists, cite it.

#### G. Allowed paths

- `git diff --name-only 8cbb697c..HEAD` contains nothing outside `rust/`, `tools/revitalize/` and `docs/revitalize/`, and nothing under `rust/crates/sim-test/src/worldmodel/`.
- The working tree adds only `.gitignore` and `.ignore`, both uncommitted as the brief says.
- New `pub` fields: `FamilyRule.elder_partner_follows` (allowed), plus `ScopeTotal.{ns,calls}` in a *new*, feature-gated struct.
- `AgentDecisionSystem` gained one private field, `places: PlacesCache` (Debug + Default), and `new()` = `Self::default()`.
- No bridge getter keys were added. Clean.

#### H. Quiet-lock record

- **Pass 18's own holds** (`perf/p18-2/development/quiet.log` and `p18-data/perf/quiet.log`):

  | KST | length | load | contended |
  |---|---|---|---|
  | 01:42:44 | 51.9 s | 2.89 | no |
  | 02:08:37 | 37.3 s | 2.46 | no |
  | 02:34:14 | 37.6 s | 3.97 | no |
  | 03:38:06 (official) | 51.0 s | 2.65 | no |

  All are ≤ 60 min and ≥ 10 min apart, and the waits are recorded.
- **Art holds are respected.** I listed the files written under `new-world-pass4/rust/target`, `new-world-p18-base/rust/target` and `p18-data` inside the art holds W500 (01:57–02:08) and W1 (02:18–02:34). Nothing appears, so no build, test or probe of Pass 18 ran then. In particular, the C7 pins (`pins_c7_release.log` 02:36:19–02:37:31) and the `bin_p2f`/`bin_p3f` builds (02:38–02:39) all come after the 02:34:52 release.
- **H1 (MINOR).** Art Pass 10's evidence (`/Users/rexxa/work/art-production10-evidence/perf/baseline-W0-attempt1.json`: `started 2026-10-15T16:38:08`, `contended true`; exit 01:41 KST) shows a hold that Pass 18's shared-Mac record does not list. Pass 18 polls only every 5 min (quiet.log 01:37:44 → 01:42:44), so it never saw that hold. Pass 18's Part 1 I6 probe ended at 16:38:23Z, which is ~15 s into the hold, and nothing else of Pass 18's ran in 01:38–01:41 (`find -newermt`). The W200 hold (`started 18:32:14Z`, `contended true`) overlapped Pass 18's I6 by ~10 s, and DECISIONS records that one. Both art holds were taken after the art run's own 60-minute wait, during Pass 18's long untimed probe periods. Pass 18 was allowed to run those, but the effect should be stated. **Fix:** add the W0 hold and the 15 s overlap to the DECISIONS shared-Mac record, and note that two art holds were contended partly by Pass 18's untimed probes.

#### Overall verdict

The Part 2 work looks behaviour-preserving, and the evidence supports it. I traced every input of every cache and memo to its mutation points. I checked the cursor invariants against start, end and replay rounds, and the `exchange` fusion against hecs 0.10.5's source; it preserves archetype creation order and entity order. My own comparator runs reproduce zero non-timing differences for I1–I6 from the Pass 17 references through the base to the final Part 2 build. The `subprofile` feature is off by default, is never required, and touches only a thread-local. The elder-follow selection is correct: no selection path lets an elder leave without their partner, and only the registered top-up can split an adult–elder couple. My recount from the raw traces reproduces G3's census exactly. The allowed-path rules hold.

**One MAJOR gap remains before Part 4 can claim G5:** I1–I6 has not been run on the actual final (Part 3) build. Beyond that:

- D5 makes R5's verdict a measurement-noise question, and the report must say so.
- The shared-Mac record misses one art hold (H1).
- The rest are wording and test-coverage fixes.

No BLOCKER.

#### Addendum (outside round-1 scope, for round 2)

`fec33d5d` and `7c91d4d8` landed during this review. A glance at `PERF_BUDGET.md` (working tree) shows the official stage a at mean +13.9 % and p99.9 +34.3 % against the 15 % and 35 % bars. Both sit inside the ±2-point noise that the three development runs show, so D5 applies directly. PERF_BUDGET already says "the p99.9 margin … is inside run-to-run noise". Round 2 should check that `REPORT_PASS18.md` says the same about the mean and lists all four stage-a runs. I did not review the official artefacts.
