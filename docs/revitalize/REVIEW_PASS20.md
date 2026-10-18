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
