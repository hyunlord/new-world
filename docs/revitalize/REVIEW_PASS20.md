# Pass 20 independent review

Two rounds at most (brief 0.3). Each round is a fresh `general-purpose` subagent given only the brief, the diff and the outputs; it changed no repository file. Its report is reproduced as written, followed by the dispositions.

## Round 1 (13:07–13:23 KST)


Reviewer: fresh general-purpose subagent. I had not seen the work before this review. Inputs: the brief (`docs/revitalize/PASS20_GOAL.md`), `git diff 5cb16639..HEAD` (HEAD went from `27d1688d` to `b2c77217` during the review), the committed docs and outputs, and the raw data in `/Users/rexxa/work/p20-data/`. I changed no repository file and ran no cargo or probe. `.mac_quiet.lock` was absent throughout. Re-runs went to a scratch directory.

**Summary: no BLOCKER, 1 MAJOR (already disclosed by the run), 6 MINOR, 8 NOTE.** Every committed verdict reproduces exactly from the raw data. Identity holds on the Part 1 build. The final-build identity (T5) and the gate were still running, so I could not review them.

---

### 1. Identity: does anything change a scene without v25?

**How I checked.** I read every Rust hunk reachable when `SocietyRules::relations` is `None`:

- **`agent_decision.rs`.**
  - The `porters` map, `forage_spots`, `avoid_of` and the dispute-day roll-over are built only under `if relations_rule.is_some()`. Otherwise they are empty.
  - `walking` chains an empty iterator. The builder row gains `&& !porters.contains_key` on an empty map.
  - The three porter arms are guarded by `porters.contains_key`, so they never match.
  - The quarrel block sits inside `if let Some(rule) = relations_rule`.
- **`bush_eligible` rewrite.** The radius test is the same expression through a closure. `avoid.iter().all(..)` on `&[]` is `true`. `bush_memo` is per tick and keyed by the home tile, so the avoid list (a function of the village) cannot go stale within a tick.
- **`movement.rs`.** `walker = founding.is_some() || porter.is_some()` reduces to the old `founding` tests. Adding `Option<&PorterTrip>` to the query does not change archetype matching or iteration order.
- **Other systems.**
  - `fire.rs`, `stockpile_deposit.rs` and `fission.rs` are each gated on `relations.is_some()`.
  - In `fission.rs`, an empty `porters` set leaves both ranking filters and the household member list unchanged (same iterator order).
  - `RelationsSystem` is registered at priority 149 in `register_fire_systems`, after `FissionSystem` (148), and returns before touching state when there is no rule.
- **Engine state.**
  - `SimResources::settlement_relations` is a new empty field.
  - The new `Tallies` fields are not emitted by `tallies_to_dict`, which builds its output field by field and was unchanged.
  - The probe adds a `pass20` key only when the rule is present.
  - `SocietyRules::relations` has `#[serde(default)]`. The summary's `rules` object holds only `firewood`, `keyed_draws` and `noise_replicate`, so no key is added.

**Pins.**
- `harness_revitalize_pass20_frozen.rs` was committed alone in `915e15f7` (09:52:25), before any other Rust change. The other commit at that timestamp, `39018fcc`, contains only tools.
- The pin file is unchanged since then (`git diff 915e15f7..HEAD` on it is empty) and contains no struct literal.
- The `relations.is_none()` asserts live in a separate Part 1 test, `harness_relations_rule_only_in_game_v25`.

**Identity runs.** I ran `validate_identity_p20.sh docs/revitalize/probe/p20-0_base docs/revitalize/probe/p20-1_identity p20-data/p1id p20-data/base` (the 4-argument version from `b2c77217`). Exit 0:
- I1, I3–I8: 0 differences.
- Chronicles: 32/32 files equal by content for every label.
- I2: 20 array hashes equal.
- DETERMINISM PASS 7/7.

Before that commit I had reproduced the same result independently: the files differ only in `git_sha` and in the `kinds` vocabulary (17 names become 22).

- **NOTE N1 — chronicle identity is by content, not by bytes.** Brief 4.1 asks for "`--chronicle-out` of I7/I8 identical". The files are not byte-identical in any scene. `sim-test/src/probe/chronicle_out.rs:65` writes `ChronicleKind::ALL` names (now 22) into every file, and `git_sha` is the run label. `b2c77217` records this deviation in DECISIONS and compares by content. The deviation is honest, and only that header changes for scenes without v25. The REPORT should say "identical in content (kinds header extended)", not "identical".
- **NOTE N2 — the validator hides its failure reason.** `validate_identity_p20.sh` captures the Python output with `line=$(…)`, but `sys.exit("msg")` writes the message to stderr. A FAIL line therefore prints with an empty reason. Cosmetic.
- **T5 (final build): not reviewable.** `docs/revitalize/probe/p20-3_final/` is untracked and `p20-data/finalid` was still at I1.

### 2. Trip accounting

**How I checked.** I read `relations.rs` `progress`, `depart`, `turn_home` and `book_lost`, and the death helper `survival/mod.rs` `despawn_agent`. That helper despawns the entity, so the inventory goes with it and nothing redeposits it.

Every path books withdrawn = delivered + returned + lost:

| Path | Code | What is booked |
|---|---|---|
| Departure | `depart` L651–664 | `withdrawn` is what the store actually gave; overflow is restored and excluded from `amount` |
| Delivery | L313–348 | `delivered += unload(remaining)` |
| Deadline or dissolved receiver | L349–362 | `failed`, stage set to `Return`, entry cause 10+ |
| Home | L366–378 | `returned += unload(remaining)`; any shortfall goes to `book_lost` |
| Death | L279–292 | Porter is absent from the `(Agent, Position, PorterTrip)` query; `lost = remaining` |
| Dissolved sender | L301–308 | Load removed from the porter's hands and booked `lost` |

The identity therefore holds by construction. Only `StockpileDepositSystem` and `RelationsSystem` remove from an `Inventory`, and the deposit system skips porters.

**Data.** Over the 32 `game_v25` evaluation worlds: every ended trip balances, no trip is open at day 120 (`in_flight_units` 0), and tallies equal the ledger. As DECISIONS states, the turn-back, death and dissolved-sender paths are exercised only by `harness_relations_turn_back_*` and `harness_relations_loads_are_lost_*`; they never occurred in play.

### 3. Porter arms and exemptions

**Arms (`agent_decision.rs`).**
- The Idle arm (L1554) comes immediately after the founding Idle arm and before the generic Idle arm, which holds `routine.can_enter` / `start` (L2175–2184), forage, builders, meals and social.
- The Seeking{Agent} arm (L2305) comes before the generic Seeking arm, where the social-chase timeout lives (`social_chase_since`, L2422–2447), so the timeout cannot idle a porter.
- The post-pass arm (L3058) comes before the generic Agent post-pass arm and sets `SeekTarget` to the hearth of the porter's current stage.
- In the Idle and Seeking arms an actionable need pre-empts the walk while the marker stays. Meals are not exempt.

**Activity holders.** `depart` sets them Idle and removes `SeekTarget`. The next `RoutineTick::new` (`routine/mod.rs` L90–108) ends the activity as interrupted. `harness_relations_candidates_are_free_fed_empty_handed_adults` covers this.

**Exemptions.**

| Exemption | Where | Checked |
|---|---|---|
| Deposit | `stockpile_deposit.rs` L106 skips any entity with `PorterTrip` | `harness_relations_trip_delivers_returns_and_is_not_redeposited` asserts the camp store stays at 80 over 11 ticks at home |
| Haul-home and tether | `movement.rs` L343, L422 via `walker` | Code read |
| Local-food catchment | `walking` set | Code read |
| Cold | `fire.rs` L149–154, inside `cold_night`, mirroring founders | Code read |
| Builders | L883 | Code read |
| Fission | `fission.rs` L517–527 / 649 / 662 / 674: excluded from both rankings and from the household and couple member list | Code read |

A porter can never receive `Founding`: candidates exclude `Founding` holders and fission excludes `PorterTrip` holders. No porter changed village in the data (T7 `trips_with_village_change` 0).

### 4. Dispute snapshot and quarrel draw

**Snapshot (L1005–1027).**
- Built before the decision loop: `Consuming{GatherFood}` at the agent's position, or `Seeking{GatherFood}` with a target at the target tile.
- Keyed by the forager's village (`member_of`), sorted by AgentId per tile.
- The rival is the first row with another village and another id (L2771).

**Draw and pair handling.**
- The pair must have a relation entry, and `disputed.insert((min, max))` limits each forager pair to once per display day. `dispute_day` is `(tick + clock_offset) / ticks_per_day` and the set is cleared on roll-over.
- Draw: `keyed_below(world_seed, STREAM_RELATIONS, lineage key of the lower AgentId, tick, 0, noise_replicate_for(tick), 1000)`.
- `p = clamp((ns + rns − ha − rha + 1000) / 4, 0, 500)` in i64, giving 250 without temperament.
- The decision loop is AgentId-ordered under `game_v25` (routine enabled).

**On a quarrel.** The picker becomes Idle with `continue`. Nothing follows the match inside the loop body, so the `continue` skips no bookkeeping. The rival goes into `quarrelled`, and its pickup on the same tick becomes Idle with no pickup (L2761). The `Quarrel` entry is recorded after the loop (L2976).

**Tests.** `harness_relations_dispute_costs_relation_once_per_forager_pair_and_day` and `…quarrel_is_a_keyed_draw_that_stops_both_pickups` cover both outcomes, both pickups, the 250 default and the chronicle fields.

- **NOTE N3 — correlated draws.** When one forager is the lower-id party of two disputes in the same tick, both draws share (subject, tick, slot 0) and come out identical. This is what the brief specifies.
- **NOTE N4 — disputes need a real pickup.** A dispute is evaluated only when `want > 0 && food_here`, i.e. an actual pickup would happen. This is recorded in DECISIONS.

### 5. Drift, clamp and hysteresis

- `decide` L407–428 runs drift, then `add`, which clamps to ±1,000 with `saturating_add`.
- Estrangement is then evaluated: estranged at ≤ −300, reconciled at ≥ −150.
- These are the only writers of `estranged` (repo grep), and they run only at `decision_minute`.
- Quarrel and delivery changes clamp through `RelationState::add`.

**Data check.** Pair (3, 4) of 19430 sits at −1000 at day 120 but is not estranged. The chronicle shows its quarrels running from tick 171337 to 172097. It was at −179 when the day-119 decision ran at tick 171360, so the result is consistent with evaluation only at the decision.

**Integration test.** −290, one forced quarrel → −400, then drift +7 → −393 → `VillagesEstranged` at −393, no trips, and the avoid filter applies. This is brief 6.9's case.

### 6. No new `TargetKind`; public items

- `agent_state.rs` is unchanged: no new `TargetKind` variant.
- **MINOR m4 — the DECISIONS deviation list is incomplete.** It records `TripRecord`, `PairDay`, `CAUSE_*`, `RelationState::add` and `SettlementRelations::{key, get, estranged}`. It omits:
  - the pub helpers outside brief §2's "food days, wood nights and base relation": `in_reach`, and arguably `need_milli`, `food_keep`, `wood_keep` and `burn_night` (`relations.rs:67–119`);
  - `PorterKind::{resource, as_u8}` and `PorterStage::as_u8`;
  - the six pub fields of `SettlementRelations` (`active`, `credited`, `dispute_day`, `disputed`, `trips`, `days`).

  None of these is harmful; the list just needs the missing entries.

### 7. Chronicle completeness and T7 evidence

- **Kinds.** 17–21 are appended. `ALL` has 22 entries, each `as_str` name is unique, and the stability test asserts 17–21 and `ALL.len()` = 22. `CHRONICLE.md` rows 17–21 plus the note at L44 are present.
- **Where each kind is written.**
  - Departures: per porter, via `record` after `decide`'s loops.
  - Deliveries and turn-backs: at the event.
  - Quarrels: after the decision loop.
  - Estranged / reconciled: pushed directly. They name no person, so `record` would add nothing.
- **T7 evidence (re-run).** In every world, each tally equals its chronicle count. Each departure entry matches one trip record on (tick, porter, villages, cause, load). Unbalanced trips 0, village changes 0. C1–C4 PASS, and C4 has 32/32 byte-identical files across runs A and B. The bridge marshals kinds as ints, so new kinds cannot break the marshalling.
- **NOTE N5 — no GDScript chronicle consumer in this checkout.** I could not check whether the art line's chronicle window tolerates kinds ≥ 17. The owner's §10 check covers this.

### 8. T verdicts, rule-table timing, tuning levers

**Re-runs into scratch.** `evaluate_pass20.py`, `evaluate_pass20_chronicle.py` (A + twin B), `evaluate_perf_p20.py` (`perf/t6`) and `diag_p20.py` (`diag/probe`) all produced output **byte-identical** to the committed `eval.md`, `chronicle.md`, `t6.md` and `diag.md`. The JSON files are equal after key sorting. `tuning.md` differs only in its title line.

**Independent recomputation from the raw `pass20.days`:**
- 2,642 in-reach pair-days.
- 214 met the food-aid condition; 176 of those were estranged.
- 38 were not estranged: 36 sent food and 2 sent nothing. 2 further trips were wood aid.
- The exchange condition never held.
- Trip totals by kind match.

**Seed choices.** The bin_p2 thresholds are the tuned ones: there are aid-food rows with giver < 4,000 ‰ or receiver ≥ 1,500 ‰, and none below 3,000 / at or above 2,000. The T6 natural seeds (the four lowest with ≥ 2 villages at day 60: 19400, 19401, 19404, 19407) are correct.

**Rule-table timing.**

| Time (KST) | Event |
|---|---|
| 11:09–11:13 | Diagnostic |
| 11:16–11:19 | Tuning run |
| 11:54:25 | Tuning commit `31474119` |
| 11:54:47 | Rule table `3562a9d4` |
| 11:55:03 | `bin_p2` built (sha256s match `eval_binary.sha256` / `binary.sha256`) |
| 11:55:39 | Evaluation start (`p2/times.txt`, `runlog.txt`) |

Order is correct.

**Levers.** I diffed `p20-data/tune/src` against `bfc77f2c`. The only rules change is `aid_giver_days_permille` 4000 → 3000 and `aid_receiver_days_permille` 1500 → 2000, both brief levers. The only other differing tracked file is `sim-core/CLAUDE.md`, which is documentation.

**Verdicts as committed and reproduced:**

| T | Verdict | Value |
|---|---|---|
| T1 | FAIL | 9/19 worlds deliver = 0.474; failed 0/71 |
| T2a | FAIL | +1.053 [0.000, 3.105]; 26 vs 6 deaths |
| T2b | reported | −0.444, 9/9 matched |
| T3 | PASS | 0.684 / 0.684 / 0.438 |
| T4 | PASS | — |
| T6 | PASS | stress +2.65 % / +2.05 %; natural ×1.045 / ×1.037 |
| T7 | PASS | — |
| T5 | pending | — |

- **NOTE N6 — T3's estranged share depends on when the run ends.** 19430 (3, 4) at −1000 counts as not estranged. Counted as estranged, the share would be 15/32 = 0.469, still ≤ 0.5.

### 9. Quiet-lock record

`quiet_lock_runlog.txt`:
- Request 12:13:48.
- PAUSE of own groups 21979 (identity I6) and 44082 (`game_v24` evaluation).
- Settled at load 3.48 after 106 s.
- HOLD 12:15:33–12:31:05 (932 s), `contended=0`.

The watcher logs in `p2/RUNLOG.txt` and `p1id/RUNLOG.txt` show both groups stopped from 12:13:48/49 to CONT at 12:31:09. `provenance.json` binary hashes match `bin_p2` and `bin_base`. The DECISIONS record is consistent.

### 10. Statements not supported by the data

- **MINOR m1 — I8 claim made before I8 ran.** DECISIONS L1194 ("The `game_v24` side is evaluated on the same build (I8 shows the build leaves `game_v24` unchanged)") was committed in `1f42dcee` at 13:04:11. The Part 1 I8 run ended only at 13:12:10 (`p1id/RUNLOG.txt`).
  - That I8 ran on `bin_p1`, not on `bin_p2`, the binary that produced the `game_v24` evaluation side.
  - It is now supported for `bin_p1` (`b2c77217`), and for `bin_p2` only by reasoning: the binaries differ in two `V25_RELATIONS` constants.
  - Relatedly, Part 2's evaluation started (11:55) before Part 1's identity check had finished (13:12). Brief 6.9 puts identity before the Part 1 push. The timing is recorded in L1202 but not called out as an ordering deviation.
- **MINOR m2 — "bit-identical" overstates the identity result.** RELATIONS.md L3 says "Every earlier scene … is bit-identical (identity I1–I8, all pins)".
  - At `27d1688d` neither the Part 1 I8 nor any final identity existed.
  - The chronicle files are equal in content only (N1).
  - Suggested wording: "identical (summaries, I2, pins; chronicles in content)", with T5's final result cited.
- **MINOR m3 — "until time heals it" is contradicted by the data.** RELATIONS.md "What the player sees" says estranged villages "keep to their own side of the land between them, until time heals it".
  - 0 reconciliations in 32 worlds.
  - 426 of 511 quarrels came after estrangement, which is the Gather-arm leak of M1, so they do not keep to their side.
  - From −1000 the drift needs 114 days to reach −150 (computed with the integer rule), longer than a 120-day world.

  The "Measured" section of the same file states the truth. The player-facing bullet should match it.
- **NOTE N7 — 38 pair-days vs 38 trips.** DECISIONS L1201 says "36 of the 38 became food trips (the other 2 trips of the 38 were wood aid)", which conflates the 38 non-estranged food-aid pair-days with the 38 trips. Data: 38 such pair-days give 36 food trips and 2 with no trip; separately there were 2 wood-aid trips, 38 trips in all. RELATIONS.md's wording is correct but hard to follow.
- Checked and supported:
  - "49 days from −400 to −150": computed 49.
  - "2.4 units per adult": growth 0.05 × 1,440 / 30.
  - "39 tiles": 420 × 3/4 / 8 = 39.4.
  - 1,834 disputes and 511 quarrels: summed from the table.
  - "19401 has an identical fingerprint in both rules": `9b69e9efeaad7ddd` in both natural files.
  - "`RelationsSystem` 0.02–0.03 %": in `t6.md`.

---

### Findings by severity

### MAJOR

**M1 — estrangement does not separate the foragers.** The brief's intent (6.6: "each avoids bushes nearer the other's hearth") is not achieved in play. The opportunistic Gather arm, which uses `nearest_gather_tile` (any food within 8 tiles inside the own catchment) and its post-pass re-resolution, does not consult `avoid`.
- **Evidence:** 426 of 511 quarrels happened after the pair was estranged, and no pair ever reconciled. 176 of 214 food-aid opportunities were blocked by estrangement, which plausibly feeds T1's miss and T2a's 19414 deaths. The run's own uncommitted experiment (`p20-data/gather_exp`) cuts the post-estrangement quarrels from 52 / 73 / 94 / 74 to 1 / 21 / 7 / 31.
- **Procedure was correct.** The brief names exactly three call sites, the implementation covers them (verified: exit predicate L1408, forage arm L2066, retarget L3251 all pass `avoid_for(sid)`), and the run disclosed the leak in DECISIONS and RELATIONS.md without tuning after registration.
- It remains the main behavioural defect of `game_v25`. The REPORT must list it under 확인된 제한사항 / 정직한 판단 as the reason "conflict rules" in the worlds where it starts (12 of the 13 worlds with disputes end with an estranged pair).

### MINOR
- **m1** — DECISIONS L1194 claimed I8 evidence before I8 had run, and on a different binary; Part 2 began before Part 1 identity finished (§10).
- **m2** — RELATIONS.md L3 "bit-identical … I1–I8" was written before any I8 existed, and the chronicles are equal in content only (§10).
- **m3** — RELATIONS.md "What the player sees", "until time heals it" / "keep to their own side", is contradicted by the data (§10).
- **m4** — DECISIONS' list of public items outside brief §2 is incomplete: `in_reach` and the other pub helpers, `PorterKind` / `PorterStage` methods, the `SettlementRelations` fields (§6).
- **m5 — exchange is unreachable in `game_v25`.** After the tuning round, the exchange food condition equals the kin-aid food condition and is tried later with the same amount. DECISIONS and RELATIONS.md say so. In effect the tuning round carried out the brief's first drop item ("exchange") without calling it a drop. The REPORT should present exchange as "implemented and unit-tested, unreachable in play", not as a delivered feature.
- **m6 — the Rust determinism twin exercises no relations code.** `harness_relations_game_v25_is_deterministic` (seed 0, 8 days) never has a second village; seed 0 has one settlement until day 30 per I8's `settlements_alive_daily`. Determinism of the relations state rests on the probe twin (`--check-determinism`, which compares `settlement_relations` every tick; DETERMINISM PASS in all three evaluation logs). Using a multi-village seed or more days would make the Rust test meaningful.

### NOTE
- **N1** — chronicle-out identity is by content (`kinds` header and `git_sha`); recorded in `b2c77217`.
- **N2** — the validator's FAIL line prints an empty reason (stderr is not captured).
- **N3** — quarrel draws are correlated when one lower-id forager is the subject of two disputes in one tick (as specified).
- **N4** — a dispute requires an actual pickup (`want > 0 && food_here`); recorded.
- **N5** — no GDScript chronicle consumer in this checkout; kinds ≥ 17 in the art line are covered by the owner's §10 check.
- **N6** — T3's day-120 snapshot counts −1000 pairs that fell after the last decision as not estranged (still passes at 0.469).
- **N7** — the "38 / 38" wording in DECISIONS L1201.
- **N8 — logs and bridge quirks.**
  - `SettlementRelations::{trips, days}` are measurement-only logs that grow without bound in the live game. They are small: at most 12 pair-days a day plus a few trips.
  - `get_porter_snapshot().amount` keeps the departed amount after delivery (stage `Return`); the art line should not draw it as a carried load.
  - Brief §9's "relations exactly for living pairs, values in range" is not asserted over the evaluation worlds. I checked it: all 32 worlds have `pairs_end` = n(n−1)/2 for the villages alive at the end, with values in range.

### Not reviewable yet
- T5 (final-build I1–I8 and pins): `p20-3_final` is in progress.
- The gate (4.4): running in `rust/target`.

### Round 1 dispositions

| finding | disposition |
|---|---|
| M1 Gather arm ignores the avoid list | Kept as the evaluated behaviour (the brief names exactly three avoid sites and the rule table is frozen); disclosed in DECISIONS, RELATIONS.md and the report's limitations and judgment with the uncommitted experiment's numbers; the first item for the next pass. |
| m1 I8 cited before it ran; Part 2 began before Part 1 identity finished | DECISIONS line corrected (evidence: Part 1 I8 on `bin_p1`, final I8 on `bin_p2`) and the ordering deviation recorded. |
| m2 "bit-identical" | RELATIONS.md reworded: summaries, recorder arrays and pins identical; chronicle files identical in content (kinds header grows). |
| m3 "until time heals it" | RELATIONS.md's player-facing bullet now says that in the evaluated worlds estrangement did not separate the villages, no pair reconciled, and the drift needs 114 days from −1,000. |
| m4 deviation list incomplete | DECISIONS list completed (fields of `SettlementRelations`, `PorterKind`/`PorterStage` methods, `need_milli`, `food_keep`, `burn_night`, `wood_keep`, `in_reach`). |
| m5 exchange unreachable | The report presents exchange as implemented and tested, unreachable in play after the tuning round (in effect the brief's first drop item). |
| m6 Rust twin exercises no relations code | New test `harness_relations_trips_and_quarrels_are_deterministic`: two runs of the walking porter fixture (two porters out and back) and a forced quarrel, compared field by field (relations, chronicle, tallies, positions); the seed-0 engine twin stays. Tests only. |
| N2 validator hides the reason | `validate_identity_p20.sh` captures the Python message (`2>&1`). |
| N7 "38 / 38" wording | DECISIONS reworded (36 food trips from the 38 pair-days, 2 sent nothing; 2 wood trips). |
| N8 porter snapshot `amount` after delivery; brief 9 pair invariant | RELATIONS.md's bridge note says to draw the load only while `stage` is 0 (or after a turn-back); `evaluate_pass20.py` now reports the brief-9 check (32 of 32 worlds: relations exactly for the living pairs, values in range) and `eval.md`/`eval.json` were regenerated (no verdict changed). |
| N1, N3–N6 | Recorded; no change (N1 already recorded; N3/N4 are the brief's rule; N5 is the owner's §10 check; N6 still passes when counted as estranged, 0.469). |

## Round 2 (14:31–14:43 KST, the last round)


**Reviewer.** A fresh general-purpose subagent. I had not seen the work before this review.

**Inputs.**
- The brief: `PASS20_GOAL.md` §§2, 4, 6–9, 11.
- `REVIEW_PASS20.md`: the round 1 review and its dispositions.
- The diffs `git diff 5cb16639..HEAD` and `git diff b2c77217..HEAD`. HEAD is `45d793e1`.
- The committed outputs.
- The raw data in `/Users/rexxa/work/p20-data/`.
- The gate run in progress in `p20-data/gate1`.
- The report draft, `p20-data/report_draft.md`.

**What I changed or created.**
- No repository file changed and nothing was committed.
- `.mac_quiet.lock` was absent at every check.
- I ran one cargo test: `harness_revitalize_pass20_relations`, with `CARGO_TARGET_DIR=p20-data/target_review`, `CARGO_BUILD_JOBS=4` and `--test-threads=4`. Its log is `p20-data/review2_relations_test.log`.
- I ran no probe. Scratch files went to my scratchpad.

**Summary: no BLOCKER, 2 MAJOR, 3 MINOR, 9 NOTE.**
- Every committed verdict still reproduces from the raw data, and the regenerated `eval.md` / `eval.json` changed no verdict.
- **T5 holds**: I1–I8 summaries are identical, I2 is identical, the chronicles are equal in content, and DETERMINISM PASS ×7. The binaries are the ones behind the evaluation and the perf measurement.
- Two problems are new since round 1:
  - **(A)** The round-1 "fix" to the identity validator broke its chronicle check. It now passes any chronicle difference with exit 0. The T5 result itself is unaffected; see A.
  - **(B)** The first gate run (`gate1`) does not test any single commit.

---

### 1. Round-1 dispositions — were they carried out as stated?

**How I checked.**
- I read `git diff b2c77217..HEAD` for `DECISIONS.md`, `RELATIONS.md`, `evaluate_pass20.py`, `validate_identity_p20.sh` and `harness_revitalize_pass20_relations.rs`, plus the `eval.json` and `eval.md` diffs.
- I re-ran the evaluator into scratch: `evaluate_pass20.py --candidate p2/A/game_v25 … --baseline p2/A/game_v24 …`. The result is that `eval.md` is **byte-identical** to the committed file and `eval.json` is **equal**.
- The `eval.json` diff since `b2c77217` contains only `+ "pairs_invariant": …` lines and one `+ "pairs_invariant_worlds": 32` line. No other value changed, so **the verdicts are unchanged**.
- I ran the Pass 20 harness (debug, review target): **24 passed, 0 failed**. That includes `harness_relations_trips_and_quarrels_are_deterministic`.

| Round-1 item | Carried out? | Evidence |
|---|---|---|
| M1 (Gather arm ignores avoid) | Yes: disclosed, kept | DECISIONS L1198, RELATIONS.md "What the player sees" and *Measured*, draft limitation 1. I verified 426 of 511 quarrels came after estrangement (recounted from `p2/A/chronicle_game_v25`). The `gather_exp` numbers 1 / 21 / 7 / 31 and one reconciliation in 19410 come from `gather_exp/c/*.json`. See MINOR m2 for the seed-selection wording. |
| m1 (I8 cited before it ran; ordering) | Yes | DECISIONS L1194: corrected, and the ordering deviation recorded. |
| m2 ("bit-identical") | Yes | RELATIONS.md L3 now says "identical … chronicle files identical in content (kind list 17 → 22)". |
| m3 ("until time heals it") | Yes in RELATIONS.md L54. **No in the report draft**, see m1 below. | — |
| m4 (public-item list) | Yes | DECISIONS L1175 now lists the `SettlementRelations` fields, `PorterKind` / `PorterStage` methods, `need_milli`, `food_keep`, `burn_night`, `wood_keep` and `in_reach`. Unlisted items I found are covered by brief 6.8 ("pure collectors"): the bridge's `collect_*` and `PorterRow` / `RelationRow`. |
| m5 (exchange unreachable) | Yes | Draft tuning paragraph and limitation 2. |
| m6 (Rust twin exercises no relations code) | Yes | New test at `harness_revitalize_pass20_relations.rs:738–761`. It passes (my run). Its guard is weak, see NOTE n1. |
| N2 (validator hides the reason) | **Done incorrectly**: it introduced a regression | See MAJOR A. |
| N7 ("38 / 38") | Yes | DECISIONS L1200 reworded. |
| N8 (porter `amount`; brief 9) | Yes | RELATIONS.md bridge bullet; `evaluate_pass20.py:148–151, 213, 280`. The check counts pairs rather than matching ids, see NOTE n3. |

### 2. T5: final-build identity and binary provenance

**Re-run.**
```
bash tools/revitalize/validate_identity_p20.sh docs/revitalize/probe/p20-0_base docs/revitalize/probe/p20-3_final /Users/rexxa/work/p20-data/finalid /Users/rexxa/work/p20-data/base
```
- Exit 0.
- I1, I3–I8: `0 differences`.
- Every label: `chronicles: PASS: 32 files equal (… kinds 17 -> 22 names)`.
- `i2: PASS: 20 array hashes equal`.
- `determinism: 7 of 7`.
- The output is identical to the committed `p20-3_final/validation.txt`.

The chronicle PASS lines carry the success text. Python prints that text only after every file compares equal, so the result is valid despite MAJOR A.

**Further checks.**
- The committed summaries equal the raw ones: `cmp finalid/probe/{i1_v19,i8_v24}/summary.json` against `p20-3_final/…` matched.
- The base matches Pass 19's final record: I1, I3–I7 compared with `compare_summaries.py` against `p19-4_final` gave 0 differences each, and `i2_sha256.txt` is identical.
- **Pins.** `gate1/test_debug.log` shows `harness_game_v24_rules_pinned_field_by_field` and `harness_v24_seed0..4_pinned` ok, and all 13 earlier `*_frozen` test binaries ran with no failure. The run reported 2,327 passed and 0 failed on the `27d1688d` tree. Its non-test Rust is identical to `3562a9d4` and HEAD: `git diff 3562a9d4..HEAD -- rust/` touches only the test file.

**Binary provenance.**

| Binary | `bin_p2` hash (current, mtime 11:55) | Also matches |
|---|---|---|
| `worldsim_probe` | `2fe73ce1…` | `p20-3_final/binaries.sha256`, `p20-2_eval/binary.sha256`, `p2/eval_binary.sha256` |
| `worldsim_perf` | `052e11ae…` | `binaries.sha256`, `binary.sha256`, `provenance.json` `builds.final.worldsim_perf_sha256` |
| `worldsim_wm` | `0543a057…` | `binaries.sha256` (not part of eval / perf) |

- `p2/run_eval.sh:6` sets `BIN=/Users/rexxa/work/p20-data/bin_p2`.
- All 37 `perf/p20-2/commands.jsonl` "final" argv entries use `bin_p2`, with `sha_before == sha_after == 052e11ae…`.
- **So the same binaries produced T5, the evaluation and T6.** The one thing the run itself does not record is which binary the T5 run used; see NOTE n6.

### 3. Report draft: every number against the committed outputs

**Checked and correct** (source in brackets).

**T1** [eval.json T1]
- 19 multi-village worlds; 9/19 = 0.474; failed 0/71.
- Trips 38 (36 food, 2 wood); porters 71; food 634, wood 34; causes 67 / 4 / 0.
- Per-world porter deliveries 2 / 8 / 4 (wood, 19410) / 10 / 4 / 10 / 2 / 2 / 29.

**T2** [T2]
- T2a: +1.053 [0.000, 3.105]; 26 vs 6; 19414 24 vs 5; without 19414, 2 vs 1; alive 24 vs 24.
- T2b: −0.444 [−1.333, 0.000], 9/9 matched.
- 19414 [chronicle_19414, both rules]:
  - Daughter 2 estranged at tick 34560 (day 24).
  - v25 deaths: cause 4 ×19 and cause 0 ×5. v24: daughter cause 4 ×5, camp cause 4 ×4.

**T3** [T3]
- 0.684 / 0.684 / 0.4375 (14 of 32).
- Disputes 1,834; quarrels 511 in 13 of 19 worlds; 14 estrangements; 0 reconciliations.
- 426 / 511 after estrangement (my recount).
- 19430 (3, 4) at −1000 not estranged; 15/32 = 0.469 if counted.

**T4** [T4]
- R2: alive +0.6 % [0, 1.9]; deaths +0.53 [0, 1.5]; shelters 1.000; social 1.003; food −1.1 %; wood −0.05 %.
- Health table: 56.84 / 56.50; 1.91 / 1.38; 6.34; 416 / 421; 108.6 / 108.7; 1.741 / 1.735.
- F1 0.681 vs 0.675; F5 0 vs 0.

**T6** [t6.json]
- Stress: 0.3681 → 0.3778 (+2.65 %); p99.9 0.781 → 0.797 (+2.05 %); per-repeat +1.7 … +3.8 %; r4 +43 %, r5 −34 %; slope 0.979 / 0.983.
- Natural: 0.2156 → 0.2252 (×1.045; repeats ×1.050 / 1.045 / 1.048); p99.9 0.468 → 0.486 (×1.037); 0.003739 → 0.003903.
- `RelationsSystem` share 0.022–0.032 %; quick 0.3032 / 0.3071.

**T7** [T7, chronicle.md]
- 0 mismatches, 0 unbalanced, 0 village changes.
- C1–C4 PASS; C4 32/32 byte-identical.

**Diagnostic** [diag.md / json, raw `diag/probe`]
- 6/16 worlds; 348 / 342 pair-days. The 6 out-of-reach pair-days are exactly (2, 3) / (3, 2) of 19511, the two daughters, both of parent 1 per its `fission_log`.
- 1 wood-aid pair-day (19512); 1,444–6,759 ‰.
- 63 disputes / 11 quarrels in 3 of 6 worlds.
- 19509 estranged at tick 69120 (decision day index 48, the 49th day); trajectory 300 / −987.
- T1 out of reach, T3 within reach.

**Tuning** [tuning.md]
- 2 / 6 worlds deliver (0.33 vs 0.17); food aid on 5 pair-days (4 in 19509); disputes, quarrels and estrangement unchanged.

**Other figures.**
- Drift: 49 days from −400 and 114 days from −1,000 (I recomputed both with the integer rule).
- 28-day year: `days_per_season` 7 (`rules.rs:1257`).
- Reach: 39 tiles.
- 14 named `Tallies` fields; 2 bridge unit tests.
- Shared-Mac tools: copies of Pass 19's with only the names, data paths and the owner string changed (diffed).

**Problems in the draft:** m1, m3 and NOTEs n4, n5, n7, n8 below.

### 4. Things round 1 missed

**No finding changes a T verdict.** The identity *claim* stands, but the tool that proves it is currently broken (A). I looked at the following and found them sound:
- The brief-9 invariant by settlement id, not only count: 32/32 worlds.
- The trip ledger.
- The selection of the T6 natural seeds.
- Nothing outside the allowed paths (`git diff --name-only 5cb16639..HEAD`). The `p19_gate` files come from the WIP commit `bbba1dbd`.
- No edit to `sim-test/src/worldmodel/**` or to Pass 19 tools.
- No GDScript chronicle consumer in `scripts/`. A grep for `chronicle` returns nothing, which confirms round 1's N5.

---

### Findings

### MAJOR

**A — `validate_identity_p20.sh` no longer fails on a chronicle difference** (`tools/revitalize/validate_identity_p20.sh:41`, introduced in `0ade8ffd` as the N2 fix).
- **Cause.** `2>&1` was put on its own line after the heredoc terminator `PY`. Inside `$( … )` that line is a separate, empty command with a redirection. Two things follow:
  1. Python's stderr is still not captured.
  2. The substitution's exit status is that of the empty command, which is always 0.
- **Effect.** Any chronicle difference prints `chronicles: PASS: ` with an empty reason, and the script exits 0.
- **Demonstration.**
  - I made a scratch copy of `finalid/chronicle/i1_v19` with one value changed in `chronicle_5.json` (entry 3, field 8 incremented by 1).
  - HEAD's validator printed `i1_v19 chronicles: PASS: ` and exited 0. The reason, `chronicle_5.json: entries differs`, went to stderr.
  - The `b2c77217` version prints `chronicles: FAIL`.
- **T5 is not affected.** Each committed chronicle line in `p20-3_final/validation.txt` (and the `p20-1_identity` one, made with the old script) carries the full success text "32 files equal …". Python prints that text only after every file compared equal; my re-run printed it for all 7 labels.
- **Fix.** Put the redirection on the command:
  ```
  python3 - "$REF_RAW/…" "$RAW/…" 2>&1 <<'PY'
  ```
- **Then:**
  - Re-run the validator.
  - Re-commit `validation.txt`. It should come out identical.
  - Record the regression and its fix in REVIEW_PASS20.md.
- **Report wording.** Until it is fixed, the report must not cite the script's exit status as T5 evidence for the chronicles.

**B — `gate1` does not gate any single commit** (`p20-data/gate1/gate.txt`, `gate_pass20.sh`).
- **What it records.** `gate.txt` says `head 27d1688d…`, started 13:05:40.
- **Debug step on the old tree.** `test_debug` built its test binaries at 13:06, before `0ade8ffd` (the test file's mtime is 13:24:39; the commit is at 13:26). It therefore ran 23 Pass 20 tests, without `harness_relations_trips_and_quarrels_are_deterministic`, and reported `passed 2327`.
- **Release step on the newer tree.** `test_release` compiled the tree checked out at that time and reported `passed 2328`, one more, which is the new test.
- **Later steps.** `clippy`, `python_unittest`, `python_worldmodel` and `allowed_paths` read the working tree and `HEAD` whenever they start. They include the tools changes of `0ade8ffd` and anything committed while the gate runs.
- **Consequence.** The record mixes at least two trees under one head label. The brief §2 / §8.3 rule ("fast-forward to a WIP commit whose gate is green") cannot be satisfied by citing `gate1` for `27d1688d`, nor for the final commit.
- **What I could confirm.** The production Rust is the same in every tree involved, and both test steps are green so far. I also ran the new test myself in debug, and it passes.
- **Required.** Run `gate_pass20.sh` on the final commit, after the validator fix (A) and with no commits during the run. Fast-forward to that commit. If that cannot be done, the report's 게이트 결과 must state the mixed coverage exactly, and must not name a single green commit.

### MINOR

**m1 — The report draft repeats the m3 overstatement** (`report_draft.md:35`, 기능 설명 bullet 2).
- **The claim.** "관계가 −300 아래로 떨어지면 … 각자 자기 쪽 덤불만 쓰며, −150 위로 회복되면 화해한다".
- **What the data shows.**
  - 426 of 511 quarrels came after estrangement.
  - There were 0 reconciliations.
  - From −1,000 the drift alone needs 114 days to reach −150.
- **The thresholds are inclusive**: estranged at ≤ −300, reconciled at ≥ −150.
- **Fix.** Use RELATIONS.md L54's wording: the rule as designed, then "평가 세계에서는 떼어 놓지 못했다 …".

**m2 — "The four most quarrelsome estranged worlds" is wrong** (`DECISIONS.md:1198`; `report_draft.md:108` "가장 다툰 4세계").
- **Post-estrangement quarrels by world** (my recount from `p2/A/chronicle_game_v25`): 19410 94, 19425 74, 19404 73, **19409 61**, 19402 52. By total quarrels, 19409 has 67 against 19402's 56.
- **The mismatch.** The experiment ran on 19402, 19404, 19410 and 19425, so 19409 is skipped.
- **Fix.** Name the seeds, or say "four of the five most quarrelsome". The numbers themselves are correct.

**m3 — Test counts in the draft** (`report_draft.md:9–10`).
- **"Pass 20 하네스 23개" (Part 1).** The Part 1 commit `bfc77f2c` has 22 `#[test]`. `3562a9d4` has 23 (the tuned-aid test was added). HEAD has 24 (the round-1 twin).
- **"assert_eq! 159개" for the field-by-field rule test.**
  - The field-by-field function `assert_v24_fields` holds 154.
  - 159 is the whole file, including the pin `check()` asserts.

### NOTE

- **n1 — The new twin's guard is weak** (`harness_revitalize_pass20_relations.rs:759`).
  - `first.4.pairs[&(1,2)].value < 300` also holds with a dispute alone (290). The quarrel does happen deterministically: p = 500 and `tick_where(…, true, 500)`, so the value is 190.
  - `== 300 − dispute_loss − quarrel_loss` or `tallies.quarrels == 1` would make the assertion's message ("exercises … a quarrel") true by construction.
- **n2 — Final-build identity cited before it existed.** RELATIONS.md L3 ("identical on the Part 1 **and final** builds") and DECISIONS L1194 ("the final identity's I8 on `bin_p2` itself (T5)") were committed in `0ade8ffd` at 13:26:29. T5 ran 13:04–14:30 (I8 14:12–14:30). Both statements are supported since `45d793e1`. This is the m1 pattern again; worth one line in the review record.
- **n3 — The evaluator's brief-9 check counts pairs.** `evaluate_pass20.py:150` compares `len(pairs_end)` with n(n−1)/2 of `settlements_alive_daily[-1]`. I checked it by id as well: the living ids at the end come from chronicle kinds 12 / 14 minus 16, and the pairs match exactly in 32/32 worlds. The claim stands, but the check is weaker than its label.
- **n4 — T2b wording.** Draft L69 says "원조받은 마을 9곳은 … 0.44명 덜 죽었다", but the interval is [−1.33, 0.00], it includes 0, and there are only 9 villages. Say it is not distinguishable from 0. The table row is fine.
- **n5 — 19414 detail.** The draft's "(v24는 노출 5, 캠프 노출 4)" omits that the camp lost 0 in v25. The world's non-old-age total is 24 (v25) vs 9 (v24); stating that avoids implying the camp did worse.
- **n6 — The T5 run does not log its binary path.** `finalid/RUNLOG.txt` and the probe logs omit the `BIN` path, and `identity.out` is empty. Provenance rests on four things:
  - the chronicle label `git_sha` = `3562a9d4`;
  - 22 kind names, which marks a Pass 20 build;
  - `binaries.sha256` = the current `bin_p2` hashes;
  - `bin_p2`'s mtime of 11:55, before the run.

  This is consistent, but not recorded by the run itself.
- **n7 — The natural ratio has two possible definitions.**
  - Ratio of medians: 1.0447 (used).
  - Median of per-repeat ratios: 1.0475.

  The registered text supports both, and both pass.
- **n8 — Delivery counts are porter deliveries.** The draft's "배달 세계: 19400(2) … 19430(29)" counts porter deliveries (sum 71), not trips (38). Label them "짐꾼 배달".
- **n9 — Placeholders still to fill.**
  - T5 should read "요약·I2·핀 동일, 연대기는 내용 동일(종류 목록 17→22)" and cite `p20-3_final/validation.txt`.
  - GATE should follow B.
  - REVIEW should include A and B and their dispositions.

### Round 2 dispositions

| finding | disposition |
|---|---|
| A validator passes any chronicle difference | Fixed: the redirection now sits on the `python3` command (`python3 - … 2>&1 <<'PY'`). Negative test: one changed value in a copy of `chronicle_5.json` now prints `i1_v19 chronicles: FAIL: chronicle_5.json: entries differs` and exits non-zero. Re-running the fixed script on the Part 1 and final identity data reproduces both committed `validation.txt` files byte for byte (exit 0), so T5 and the Part 1 identity stand. |
| B `gate1` gated no single commit | Agreed. `gate1` (13:05–14:43, green 15/15) is kept as a preliminary record (`verification/p20_gate/first_27d1688d/`): its debug step built the `27d1688d` tests (2,327 passed), its release step the tree after `0ade8ffd` (2,328, the round-1 test included), and later steps read the moving working tree; the production Rust was the same in every tree. The final gate runs on the report commit with no commit or edit in the working tree while it runs, and `codex/revitalize-2026-10` moves only to that commit. |
| m1 report repeats "keep to their own side … reconcile" | Report's 기능 설명 now states the rule as designed and that in the evaluated worlds it did not separate the villages and no pair reconciled. |
| m2 "four most quarrelsome" | DECISIONS and the report name the four seeds (four of the five with the most post-estrangement quarrels; 19409 with 61 was not run). |
| m3 test counts | Report: Part 1 had 22 Pass 20 tests (23 after the tuning test, 24 after round 1); the field-by-field function has 154 `assert_eq!` (159 in the whole file). |
| n1 weak twin guard | The test now asserts two trips and the exact post-quarrel value (300 − 10 − 100). |
| n2 final identity cited before it existed | Recorded here: RELATIONS.md L3 and the DECISIONS line were committed at 13:26, before T5 finished (14:30); both are supported since `45d793e1`. |
| n3 brief-9 check counts pairs | `evaluate_pass20.py` now compares the pair ids with the living settlement ids from the chronicle (formed or founded, not dissolved); 32 of 32 worlds; `eval.md`/`eval.json` regenerate byte-identically. |
| n4, n5, n8 report wording | T2b called indistinguishable from 0; 19414's world totals (24 vs 9, camp 0 in v25) given; "짐꾼 배달" labels the per-world delivery counts. |
| n6, n7, n9 | Recorded (T5's binary is identified by its chronicle label, the kind list and `binaries.sha256`; the natural ratio is the ratio of medians, 1.0447, the median of per-repeat ratios 1.0475 also passes; placeholders filled). |
