# GOAL: WorldSim — Revitalize Pass 21: feuds that end and help that arrives (`game_v26`: estranged villages really keep apart, conflict that can be repaired within a season, aid that happens, exchange that can happen)

Repository `hyunlord/new-world`. Continue the simulation line: `codex/revitalize-2026-10` = `6e1446b4d6af41e13e2b221b17609751510f046f` (Pass 20, gate green), `codex/revitalize-2026-10-wip` = `6bb092d59cf8fa37481e4303994ffb6891e60a31`. Draft PR #510.
This run is executed by **Claude Code**, unattended, on the Mac, in `/Users/rexxa/work/new-world-pass4`. **The owner launches it only after Art Pass 12 has finished**, so the Mac is yours. The quiet-lock protocol (1.2) stays in force in case another run starts. No DGX.

Pass 20's mechanics are exact: 71 porters delivered, the ledger balanced, identity held, and the cost was +2.65 %. Its **balance** is wrong, and the planner's brief is the cause.
1. **Estranged villages never actually separate.** The avoid filter reached the three forage-choice sites the brief named but not the **opportunistic Gather arm**, which takes any food within 8 tiles, nor its post-pass target. 426 of 511 quarrels happened after estrangement.
2. **Conflict cannot heal.**
   - A quarrel costs −110 (dispute −10 + quarrel −100); a delivery gains +40.
   - Every forager pair counts once per day, so a village pair can lose hundreds a day (19509: 300 → −987 in under 49 days).
   - Drift is 1 % a day, so recovering from −1,000 to −150 takes 114 days.
   - Result: no reconciliation in 32 worlds; 14 of 32 pairs estranged at day 120.
3. **Aid is rare and exchange is impossible.**
   - The stock target keeps every village at 1.5–6.5 food days, so a rich and a short village rarely coexist.
   - 176 of the 214 qualifying pair-days were blocked by estrangement.
   - The tuned aid thresholds (3,000 / 2,000 ‰) equal the exchange's food thresholds, and aid is tried first, so exchange never runs.
   - T1 0.474 < 0.6.

Pass 21 adds `game_v26` = `game_v25` with a revised `RelationsRule`:
- the avoid filter at every food-target selector;
- per-village-pair daily caps on what disputes and quarrels cost;
- smaller losses and faster drift, chosen by a pre-registered grid;
- an aid keep line below the giver threshold;
- exchange tried before aid.

No new mechanics, no fights. The live game moves to `game_v26`.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 6bb092d5
cp ~/Downloads/claude_goal_worldsim_pass21.md docs/revitalize/PASS21_GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS21_GOAL.md>
```
The clone's hooks are neutralized as before (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag. Leave the local `.gitignore` edit and `.ignore` uncommitted. Stage explicit paths only.

**This document is `docs/revitalize/PASS21_GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part and after every compaction or pause. The owner drives the run with `/goal`: the section 11 report, the U table, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
Same as Pass 20's 0.2:
- no `install_hooks.sh`;
- never `lead/main` or `main`;
- no `harness_pipeline.sh` (4.4 is the gate);
- built-in `general-purpose` subagents only;
- a blocking hook other than `/goal` → record, push, report.

Pass 20's files are the reference for every procedure cited here: `PASS20_GOAL.md`, `REPORT_PASS20.md`, `REVIEW_PASS20.md`, `DECISIONS.md` Pass 20, `tools/revitalize/*_p20*`, `run_watched_p20.sh`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions go in `docs/revitalize/DECISIONS.md` under `## Pass 21`, with a RUNLOG table.
2. **Do not end your turn** between Parts or while a measurement or probe batch runs.
3. **Parts:** 0 = section 5; 1 = section 6; 2 = section 7; 3 = section 8. Push after every Part.
4. **Review loops:** two rounds maximum.
5. **On failure:** work around, note it, continue. A bar missed is reported as missed.
6. **Time rule:** past **12 hours** of working time (pauses excluded and recorded), finish the current step, push, go to Part 3. If Part 1 is not pushed by hour 6, drop 6.5 (exchange first) and run the grid (7.2) with 4 configurations only (the first four of its list).
7. **Drop order:** exchange first (6.5) → the grid's last 8 configurations → the comparison run on 19400–19431 (7.5). **Never drop** Parts 0 and 3, identity, the gather avoid (6.2), the caps (6.3), the diagnostic (7.1), the grid's first 4 configurations, the evaluation on 19600–19631, or U1–U7.
8. **Mac sleep:** record the pause and continue.

---

# ⚠️ 1. MACHINE
## 1.1 The Mac
- **Work only in** `/Users/rexxa/work/new-world-pass4` and one base worktree `../new-world-p21-base` (5.2). Never touch other checkouts or processes you did not start. No DGX.
- **Parallelism:** `CARGO_BUILD_JOBS=6`, tests `--test-threads=6`, probes `--workers 4`.
- **Data:** `/Users/rexxa/work/p21-data/`.

## 1.2 Sharing the Mac
As Pass 20's 1.2: the tools are copied to `_p21` names (owner string, environment names and data path renamed).

---

# 2. GIT
- **Branches:** commits go to `codex/revitalize-2026-10-wip`. `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose gate (4.4) is green. Run `git ls-remote` after every push. Messages are `revitalize(p21-N): …`.
- **May change:** `rust/**`, `tools/revitalize/**`, `docs/revitalize/**`.
- **Must not change:**
  - `scripts/**`, `scenes/**`, `assets/**`, `localization/**`;
  - `docs/art/**`, `tools/art_pipeline/**`;
  - `tools/worldmodel/**`, `docs/worldmodel/**`;
  - `rust/crates/sim-test/src/worldmodel/**` (the recorder: no schema change).
  - Pass 20's tools are not edited: copy them to `_p21` names.
- **New public items allowed:**
  - the `RelationsRule` fields of 6.1 and `V26_RELATIONS`;
  - `WorldRules::game_v26`, `WorldGenConfig::game_v26`, `game_v26_scene` and the probe / perf wiring (Pass 20's 3.4 list, `game_v25` → `game_v26`);
  - named `Tallies` fields (6.6);
  - the extra `avoid` parameter on the gather selectors (6.2).
  - **Not allowed:** no new `TargetKind`, no new chronicle kind, no new component, no new RNG stream. The bridge getters are unchanged except the live-game scene. `game_v25` and older scenes are bit-identical.

---

# 3. CONTEXT (planner's survey at `6bb092d5`, paths under `rust/crates/`; verify each fact)

## 3.1 The relations code
- **The rule:** `RelationsRule` and `V25_RELATIONS` (`sim-core/src/rules.rs` ≈ L961–985: giver 3000, keep 3000, receiver 2000, trade 3000/2000/4/2, drift 10, delivery 40, dispute 10, quarrel 100, estrange −300, reconcile −150, porter hunger 30); `game_v25` ≈ L988–991.
- **`RelationsSystem`** (`sim-systems/src/runtime/settlement/relations.rs`, priority 149):
  - `decide` (≈ L404) applies drift, then hysteresis (≈ L407–428), then the measures (≈ L440–456), then the ordered pair rows `aid_food` / `aid_wood` / `exchange` (≈ L476–487);
  - the trips (≈ L491–525): aid first; exchange only `if row.exchange && !used.contains(&(r, g))`.
  - Amount = min(10 × slots, stock − keep, 10 × candidates). With keep = 3,000 ‰ and giver ≥ 3,000 ‰, the surplus is often 0–10 units.
- **Disputes** (`sim-systems/src/runtime/decision/agent_decision.rs` ≈ L2754–2810):
  - every forager pair is counted once per day (`relations.disputed`, cleared on a new display day ≈ L1042–1044);
  - the costs are `dispute_loss` and, on a keyed quarrel draw, `quarrel_loss`;
  - `quarrelled` makes the rival's pickup a no-op; Quarrel entries are written after the loop (≈ L2975).
- **The avoid list:** `avoid_of` / `avoid_for` (≈ L1004–1048) feed `choose_bush` / `bush_eligible` / `bush_exists` (≈ L98–149) at the exit predicate (≈ L1411), the forage arm (≈ L2066) and the retarget (≈ L3251–3260).
- **Gather selectors that ignore it:**
  - `nearest_gather_tile` (≈ L644; radius `GATHER_SCAN_RADIUS` 8, ≈ L564; catchment filter) and `gather_tile_exists` (≈ L666), memoized per tick by `GatherMemo` keyed by (tile, catchment) (≈ L156–158);
  - callers: the Gather-arm trigger (≈ L2153), its exit/memo use (≈ L1385–1495), the post-pass seek attach (≈ L3080) and the post-pass `SeekTarget` match (≈ L3162).
  - Grep for every caller and every other function that can give an agent a `TargetKind::GatherFood` target, and list them in DECISIONS.
  - **Eating paths** (`nearest_local_food`, the Food target) do not cause disputes and stay unchanged.

## 3.2 Pass 20's numbers (`docs/revitalize/probe/p20-2_eval/`, `REPORT_PASS20.md`)
| quantity | value |
|---|---|
| multi-village worlds | 19 of 32 |
| worlds with ≥ 1 delivery | 9 |
| trips / porters, all delivered | 38 / 71 |
| exchange trips | 0 |
| disputes / quarrels | 1,834 / 511 (426 of the quarrels after estrangement) |
| estrangements / reconciliations | 14 / 0 |
| estranged pairs at day 120 | 14 of 32 |
| daughter non-old-age deaths, v25 vs v24 | 26 vs 6 (19414: 24 vs 5) |

- **Diagnostic seeds 19500–19515:** food days of givers 1,444–6,759 ‰; receivers never below 1,444 ‰.
- **Gather-avoid experiment (temporary build):** post-estrangement quarrels 52/73/94/74 → 1/21/7/31 in four worlds; one pair reconciled.

## 3.3 The planner's model of the relation balance (to be checked by 7.1, not trusted)
- **The model:** a village pair with λ disputes per day, quarrel share 0.28, the daily caps of 6.3 (3 disputes, 1 quarrel), losses 5 / 40 and drift 3 %. While estranged, contact falls to 5 % (the gather avoid).
- **Equilibrium:** the pair settles where drift balances the expected daily loss: v* = base − loss / 0.03.
- **Simulated over 120 days from base 300:**

  | λ (disputes per day) | estrangements per pair | estrangements reconciled | estranged at day 120 |
  |---|---|---|---|
  | 0.5 | ≈ 0 | — | ≈ 0 |
  | 1 | 0.3 | 78 % | 7 % |
  | 2 | 3.4 (cycles of feud and reconciliation) | 87 % | 44 % |

- **The same model with v25's rules:** λ = 0.3 → 91 % estranged at day 120 and 0.3 % reconciled.
- **Recovery** from −1,000 to −150 at 3 %: ≈ 35 days (≈ 23 days from −600).

---

# 4. RULES
## 4.1 Identity
- **Checks:** as Pass 20's 4.1, with **I9 = `game_v25`** (copy `identity_p20.sh` → `identity_p21.sh`).
- **What must match** for I1–I9 between base (`6e1446b4`) and candidate:
  - probe summaries (`--seeds 0..32`, 120 days), identical except timing;
  - the recorder I2;
  - all pins;
  - `--chronicle-out` by content.
- **New pins:** 5.1.
- **Three identity sets:** base, Part 1 build, final.

## 4.2 Seeds
- **Seed sets:**
  - diagnostic and grid: **19500–19515**, **120 days** (Pass 20 used 60; reconciliation needs the season);
  - evaluation: **19600–19631**, 120 days, fresh;
  - comparison (reported only): 19400–19431, where Pass 20 evaluated.
- **Unused check:** `grep -rnE -- '--seeds +196[0-9]{2}' docs/revitalize tools/revitalize` and `find docs/revitalize -name '*_s196[0-9][0-9][._]*'` must find nothing. Otherwise shift by +100 and record.

## 4.3 Arithmetic and draws
- Integers and permille only; no forbidden math.
- No new draws: the existing keyed quarrel draw only, skipped when the quarrel cap is reached. A keyed draw consumes no stream state, so skipping is deterministic.
- Maps are iterated in key order.

## 4.4 The gate
`tools/revitalize/gate_pass21.sh` = `gate_pass20.sh`'s 15 steps, with `allowed_paths` checked against `6bb092d5` and the watcher on. Run the final gate on a commit with no edits during the run (Pass 20 review B).

## 4.5 Performance
Pass 20's `PERF_BUDGET.md` rules against **`game_v24`**:
- stress five-repeat;
- natural measurement on the 4 lowest evaluation seeds with ≥ 2 settlements at day 60;
- `game_v25` reported beside.

---

# 5. PART 0 — Pins and base
5.1 **Pins.**
- File: `rust/crates/sim-test/tests/harness_revitalize_pass21_frozen.rs`, scene `game_v25`, seeds 0–4.
- Fingerprints: fingerprint and `state_digest` at 1,440, 40,320 and 86,400 ticks; births, splits and `settlement_relations` digest. One `#[test]` per seed.
- Rule asserts: field-by-field asserts of `WorldRules::game_v25(s)`, including every `V25_RELATIONS` field (no struct literal).
- Commit alone, before any other Rust change.

5.2 **Base.**
- Worktree `../new-world-p21-base` at `6e1446b4`; release build.
- Run I1–I9 into `docs/revitalize/probe/p21-0_base/`. I1–I8 must equal Pass 20's final record.
- Push.

---

# 6. PART 1 — `game_v26`

## 6.1 The rule
**New `RelationsRule` fields.** `V25_RELATIONS` takes the "v25" values, which reproduce Pass 20 exactly:

| field | v25 | v26 | meaning |
|---|---|---|---|
| `avoid_gather` (bool) | false | true | 6.2 |
| `dispute_cap_per_pair_day` (u32) | 0 = no cap | 3 | 6.3 |
| `quarrel_cap_per_pair_day` (u32) | 0 = no cap | 1 | 6.3 |
| `exchange_first` (bool) | false | true | 6.5 |

**`V26_RELATIONS`** = `V25_RELATIONS` with the v26 column and the **grid defaults** (configuration G1 of 7.2):

| field | v26 default (G1) |
|---|---|
| `dispute_loss` | 5 |
| `quarrel_loss` | 40 |
| `relation_drift_permille` | 30 |
| `aid_giver_days_permille` | 3000 |
| `aid_keep_days_permille` | **2000** (below the giver threshold, so a giver always has a surplus) |
| `aid_receiver_days_permille` | 2000 |

Every other field is as v25. `V26_RELATIONS` is replaced by the configuration that 7.2 selects, and frozen.

**Wiring:**
- `game_v26(seed)` = `game_v25(seed)` with `society.relations = Some(V26_RELATIONS)`; wire per Pass 20's 3.4.
- The live game (`init`, `reset_world`) builds `game_v26`; `reset_world_stress` stays `game_v19`.
- Update the test `live_game_is_game_v25` (`world_node.rs` ≈ L2955) to v26.

## 6.2 Estranged villages keep apart at every food-target selector (`avoid_gather`)
When the rule is set with `avoid_gather`, `nearest_gather_tile` and `gather_tile_exists` take the member's avoid list and skip a tile whose Chebyshev distance to an avoided hearth is smaller than to the own hearth. This is the same test as `bush_eligible`.
- **Callers:** every caller from 3.1 passes `avoid_for(sid)`, or the empty list for a non-member or when the flag is off.
- **The memo:** `GatherMemo`'s key gains the avoid list's identity (the member's settlement id, or a hash of the sorted list). Prove by a test that two members of different villages at one tile get different answers when one is estranged.
- **Unit tests:**
  - every selector skips the avoided tile;
  - each is unchanged with an empty list;
  - the trigger and the post-pass choose the same tile (the existing radius-confounder rule).

## 6.3 Daily caps per village pair
- **Ledgers:** `SettlementRelations` gains per-day counters per pair: disputes costed and quarrels. They are cleared with `disputed` on a new display day.
- **The dispute cap:** a dispute is still tallied (`forage_disputes`), but its `dispute_loss` applies only while the pair's costed disputes today < `dispute_cap_per_pair_day`.
- **The quarrel cap:** the quarrel draw is made only while the pair's quarrels today < `quarrel_cap_per_pair_day`. Otherwise the dispute is not a quarrel and the pickup proceeds.
- **No cap:** 0 means no cap (v25).
- **New tallies:** `disputes_uncosted`, `quarrels_capped` (disputes that would have drawn), `quarrels_while_estranged`.

## 6.4 Aid amounts
No code change; the keep line moves (6.1). The amount formula is unchanged.

## 6.5 Exchange first (`exchange_first`)
- **Order:** for each ordered pair (g, r) in id order, evaluate exchange **before** aid.
- **If exchange applies** (both legs depart, using both ordered pairs' slots), skip aid for (g, r) and (r, g) that day.
- **Otherwise:** aid as v25.
- The conditions stay v25's `trade_*` values.
- **Unit test:** a pair meeting both the aid and the exchange conditions performs the exchange under v26 and the aid under v25.

## 6.6 Probe and evaluation output
New probe-summary keys appear only when `relations.is_some()` and `dispute_cap_per_pair_day > 0`:
- the 6.3 tallies;
- per pair: days estranged, estrangement and reconciliation ticks.

Pass 20's keys are unchanged for `game_v25`.

## 6.7 Checks of Part 1
- **Identity:** I1–I9 identical on the Part 1 build; a `game_v26` determinism twin.
- **Unit tests:** 6.2–6.5, plus drift at 30 ‰ (the integer stall at gap ≤ 33 documented).
- **Integration test:** two villages at relation −290 with daily contact. With v25 the relation stays below −300 after estrangement. With v26 the contact falls under avoid and the pair reconciles within 40 days.
- Push.

---

# 7. PART 2 — Diagnostic, grid, evaluation

## 7.1 Diagnostic (mandatory, first)
Run on 19500–19515, 120 days, `game_v25` and `game_v26` (G1). Per world and pair:
- disputes per pair-day before and while estranged (the real λ, to check 3.3's model);
- quarrels;
- estrangements, reconciliations, days estranged;
- the relation trajectory (daily 08:00 values);
- qualifying aid and exchange pair-days, trips;
- daughter non-old-age deaths.

Output: `docs/revitalize/probe/p21-2_diag/diag.md`. State whether 3.3's model holds: λ distribution and reconciliation share.

## 7.2 The grid (pre-registered here; run after 7.1, before the rule table)
Twelve configurations G1 … G12 on 19500–19515, 120 days, all with the v26 flags and caps of 6.1, in this order:
- quarrel_loss ∈ {40, 60} (outer);
- drift ∈ {30, 20} ‰ (middle);
- (aid giver, aid receiver) ∈ {(3000, 2000), (2500, 2000), (3000, 2500)} (inner), keep 2,000 ‰ always.

So G1 = (40, 30, 3000/2000), G2 = (40, 30, 2500/2000), …, G12 = (60, 20, 3000/2500).

**Selection rule:**
- Among the configurations meeting **all** of these, choose the one with the highest share of multi-village worlds with ≥ 1 delivered trip; ties go to the earlier configuration:
  - estranged share of pairs at day 120 ≤ 0.25;
  - quarrels in ≥ 0.15 of multi-village worlds;
  - if ≥ 3 estrangements occurred, ≥ 0.5 of them reconciled by day 120.
- If none qualifies, choose the configuration with the most delivering worlds among those with estranged share ≤ 0.5, and record it.

Freeze the choice in `V26_RELATIONS` (one commit, values in DECISIONS). There is no other tuning.

## 7.3 Rule table
Committed to METRICS.md under `## Pass 21` after 7.2 and before 7.4. It compares `game_v26` against `game_v25` and `game_v24` on seeds 19600–19631, 120 days, paired, with Pass 17's definitions and bootstrap. A "multi-village world" is one with ≥ 2 settlements alive at once in `game_v26`.

| # | rule |
|---|---|
| U1 | **aid happens:** ≥ **0.6** of multi-village worlds have ≥ 1 delivered trip; failed-trip share ≤ 0.1. Deliveries, food and wood moved, causes reported. |
| U2 | **exchange can happen:** ≥ 1 exchange trip over the 32 worlds (if 6.5 was dropped: NOT_RUN). Exchange pair-days reported. |
| U3 | **conflict that heals:** disputes in ≥ 0.3 and quarrels in ≥ 0.15 of multi-village worlds; estranged share of pairs alive at day 120 ≤ **0.25**; quarrels while estranged ≤ **0.2** of all quarrels; if ≥ 3 estrangements, ≥ **0.5** reconciled by day 120 (else reported). |
| U4 | **health:** Pass 17's R2 rules and F1/F5 within ± 0.05 for `game_v26` − `game_v24`. Daughter non-old-age deaths per multi-village world `game_v26` − `game_v24` ≤ 0 (point estimate, interval reported) and the same against `game_v25`. Daughters alive at day 118 ≥ `game_v24`'s. |
| U5 | **identity:** I1–I9 and all pins identical on the final build. |
| U6 | **performance against `game_v24`** (4.5): stress mean ≤ +15 %, p99.9 ≤ +35 %, ≤ 4.0 ms, slope < 1.7. Natural: mean ratio ≤ 1.15, p99.9 ratio ≤ 1.35. |
| U7 | **accounting and chronicle:** Pass 20's T7 on `game_v26`, plus tallies = chronicle for reconciliations, and the 6.3 tallies consistent (costed + uncosted = disputes). |

## 7.4 Evaluation
- **Runs:** `game_v26`, `game_v25` and `game_v24` on 19600–19631, 120 days, `--agent-trace --chronicle-out`.
- **Outputs:** `docs/revitalize/probe/p21-2_eval/eval.md` and `eval.json`; U6 goes to `docs/revitalize/perf/p21-2/`.

## 7.5 Comparison (reported, droppable)
`game_v26` on 19400–19431 against Pass 20's records of `game_v25` and `game_v24` there: U1 and U3's quantities side by side. Push.

---

# 8. PART 3 — Docs, review, gate, report
1. **Docs:**
   - `RELATIONS.md` gains a Pass 21 section: what changed, why each number, the grid result, the measured λ against 3.3.
   - `PERF_BUDGET.md` gets the Pass 21 numbers.
   - `CHRONICLE.md`: kinds unchanged; note that reconciliations now occur.
2. **Independent review** (fresh `general-purpose` subagent, two rounds max). Scope:
   - identity I1–I9 and pins;
   - that nothing changes `game_v25` or older;
   - every food-target selector and the memo key (6.2);
   - the caps and their ledgers;
   - exchange first, with slots;
   - the keep line;
   - the grid order and selection, applied exactly;
   - the U verdicts;
   - the quiet-lock record.

   Findings go in `docs/revitalize/REVIEW_PASS21.md`.
3. **Gate** (4.4, on an unedited commit) → `docs/revitalize/verification/p21_gate/`. Then fast-forward `codex/revitalize-2026-10`, add a PR #510 section, and remove `../new-world-p21-base`.
4. **Report:** `docs/revitalize/REPORT_PASS21.md` in Korean (section 11 format), and the same text in the final message.

---

# 9. Invariants (tests)
- Pass 20's 9 invariants hold on `game_v26`.
- Costed disputes per pair per day ≤ the cap; quarrels per pair per day ≤ the cap.
- With the flags off and caps 0, `game_v26`'s code path equals `game_v25`'s. The Part 1 twin compares `game_v25` before and after the change.

---

# 10. 인게임 확인사항 (오너용)
- 게임이 `game_v26`로 시작한다. 마을이 둘 이상인 세계에서 짐꾼이 Pass 20보다 자주 보이는지(아침 8시쯤 다른 마을로 가는 사람).
- 사이가 틀어진 두 마을의 채집꾼이 서로의 화덕 쪽 덤불에 가지 않는지(줌아웃해서 두 마을 사이 경계에 사람이 몰리지 않는지).
- 한두 달(게임 일 30~40일) 지나면 연대기(C 키)에 "화해함"이 나오는지. 다툼은 있지만 대부분의 마을 쌍이 끝까지 원수로 남지는 않는지.
- 콘솔 에러가 없고 2배속에서 끊김이 없는지.

---

# 11. 구현 완료 보고 (REPORT_PASS21.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
### 구현 방법
(모든 채집 목표 선택기의 회피와 메모 키, 마을 쌍 하루 상한과 장부, 손실·이동·유보선, 교환 우선, 격자와 선택 규칙, 진단의 실제 λ와 계획자 모형 대조)
### 기능 설명
### 변경된 파일 목록
(커밋별)
### 측정 결과
진단(7.1) 요약(실제 λ 분포, 소원 전·중 분쟁률), 격자 12구성 표와 선택, U1~U7 표(값과 구간), game_v25 대비·game_v24 대비, 세계당 배달·교환·실패, 분쟁·말다툼(상한에 걸린 수, 소원 중 말다툼 비율)·소원·화해·소원 일수, 딸 마을 사망과 생존, 건강 표, 성능(스트레스·자연), 19400–19431 비교
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
소원한 마을이 이제 정말 떨어져 지내는가? 다툼이 한 계절 안에 풀리는가? 원조가 이제 흔한가, 그리고 사람을 살리는가? 교환이 일어나는가? 다음(v2: 싸움·습격, 또는 다른 것)은 무엇인가?
```
The final message also includes:
- every commit hash with its Part;
- both branch heads with `git ls-remote` confirmation;
- the PR #510 update;
- the time per Part.
