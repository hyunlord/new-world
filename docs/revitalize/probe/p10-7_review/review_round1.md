# Pass 10 — independent review, round 1

Reviewer: fresh-context read-only `general-purpose` subagent, 2026-10-12. Scope: `git diff 6aa4dba0..HEAD -- rust/ tools/revitalize/ tools/worldmodel/` (HEAD `341e4453`), PASS10_GOAL.md sections 4, 6, 7, 8, 10, 11 and 12, METRICS/DECISIONS/TEST_LEDGER `## Pass 10`, and the committed pilot, fork and H-target outputs. Raw data under `/Users/rexxa/work/worldsim-wm-data/p10/` and `/Users/rexxa/work/p10bin/` was read but not modified. Nothing was built, run under cargo or killed.

Independent recomputations (read-only python):
- The eight `WorldVariety` draws, reimplemented from `rng.rs` (splitmix chain, subject 0, tick 0, replicate 0), match `seed_info.world_variety` for all 192 probe seeds (round-2 10-day, held-out 10-day and 120-day) and `meta.world_variety` for all 112 pilot `game_v16` episodes.
- H2: every seed passes on round-2 and held-out, for both v16 scenes. The smallest margin over the per-seed floor is 7.2, in held-out v16.
- H3: share, median, maximum share, severity means and starvation figures are identical to METRICS. In every 120-day run the exposure deaths inside the seed-winter windows equal the total exposure deaths (76, 58, 164, 131).
- H6: the means and climate shares over 128 seeds match METRICS. There are no deadwood shortfalls and no berry swaps.
- H7: matches `diversity.json`. Wood fails.
- H8: recomputed from 256 raw fork episodes. rot_wood 0.5625 / +1.180, rain 0 / −0.594, cold_snap 0.0625 / +0.156, gift_wood 0 / −0.625. These match `fork_metrics.json`, and G0a is 224 / 224.
- H10: the ratios from the timing summaries match (1.027, 1.179, 1.060, 1.173).

## Checks

| # | Check | Result |
|---|---|---|
| 1 | v8–v15 invariance | **PASS.** Every new behaviour is behind a rule field that is None or false in v8–v15. `exposure`: `rules.rs` default and frozen literals, `fire.rs:154-156` calls `expose` only under the rule, `survival/mod.rs:163,193-195` sets the heal block only under the rule. `capacity_weighted`: false in `game_v9`/`game_v10` (`rules.rs:586,644`) and set only in `game_v16_with` (`rules.rs:~424`). Utilization 0.78 is set only in `game_v16_neutral`. Variety is used only by `WorldGenConfig::game_v16*` and `WorldRules::game_v16*`. The v15 literal and pins are in `harness_revitalize_pass10_frozen.rs`. The worldgen changes are measurement-only writes (`camp.rs:270-273`, `terrain.rs:77-83`), and the inert test plus the 4.6 guards confirm them. |
| 2 | Variety draws use replicate 0 | **PASS.** `variety.rs:50` (`keyed_below(seed, STREAM_WORLD_VARIETY, 0, 0, slot, 0, n)`) and `variety.rs:94` (climate, slot 6) hard-code replicate 0. `draw` takes only `seed`. My reimplementation reproduces all recorded draws. |
| 3 | No iteration-order dependence | **PASS.** FireSystem: `agents` is sorted by AgentId (`fire.rs:130`), `exposed` is built in that order, and deaths are booked in that order (`fire.rs:163-203`). The heal block is per-agent with no cross-agent effect. The capacity-weighted sum iterates a HashMap but sorts it first and sums small integers, so it is exact (`food_economy.rs:123-136`). Probe and recorder: `BTreeSet`/`BTreeMap`, plus a `min` reduction that does not depend on order (`pass10.rs:171-178`, `export.rs` `hps`). |
| 4 | Exposure deaths only from `Cold`, in AgentId order | **PASS.** `exposed` holds exactly the agents in `cold` (those inserted or already marked, `fire.rs:136-146`). `expose` damages only these agents and despawns after the loop in AgentId order (`fire.rs:163-203`). The probe confirms every exposure death lies in a cold window (H4) and inside the seed-winter windows. |
| 5 | Healing block | **PASS.** `exposure_clock.rs:107-109`: `tick < last_cold_tick + 60 + 60*h`. `survival/mod.rs:193-195` blocks healing while `Cold` or while the clock blocks, only under the rule. `harness_exposure_blocks_healing` checks +60, +120 and +179 (no heal) and +180 (heal). |
| 6 | `DeathReason::Exposure` is counted everywhere | **PASS.** Counted in `despawn_agent` (`survival/mod.rs:100`), `as_str`/`as_u8` (`event.rs:87,106`), the FFI tallies (`world_node.rs:2775`), the census and `cross_check` (`deaths.rs:261-262,314,330`), the row key (`pass10.rs:165`), Demography (`summary.rs:241-242`), recorder global col 35 (`export.rs:155`, `schema.rs:64`), `fork_metrics --deaths-with-exposure` (`fork_metrics.py:33-34,53-54`) and the diversity death report (`diversity_p10.py:119-122,179-182`). Caveats are L4 and L6. Out of scope (merge notes 2b and 2c): the HUD and the death colour. |
| 7 | Pilot on the frozen commit; fresh seeds | **PASS.** All 336 pilot `meta.json` files carry `git_sha` 58119582, schema 4, every 60 and 120 days. The fork and unforked episodes (1,400) also carry 58119582. `wm_frozen` was built at 17:07, after the commit at 17:06:49. No production Rust changed in `58119582..HEAD` (only a test file, `fork_metrics.py` and a Python test). Seeds 20000–20111 and 21000–21111 appear in no Pass 10 probe summary (tuning 0–63, held-out 64–127/32–95, scratch 5000–5031 and 30000–30111, sweep 1000–2999). The disclosed test-only use (DECISIONS L611: a tick-0 world of 20000 and a 1,440-tick run of 21000 inside a test, pass/fail only, never committed; the `git log -p` of `rust/` shows no such seed) is judged harmless: no metric was seen and nothing was tuned from it. |
| 8 | `calcore_wm11.py` matches its source | **PASS.** Running `diff` against `git show ecf8c316:tools/worldmodel/wm1/calcore.py` shows only the one added header line. |
| 9 | H verdicts follow section 10 | **PASS** (recomputed above). The documented interpretations are reasonable and disclosed (DECISIONS L603, L615, L617, L624): H5 "no new failure" for population clauses (see L1), utilization in both v16 rulesets (the attribution caveat is in METRICS, missing in `diversity.md`, see L2), capacity weighting capped at 1 (justified by the scratch result that uncapped weighting starves capacity-5 worlds; but see M1), and 11.5 not used (justified; see L11). |
| 10 | Other defects | Findings below. No High findings. |

## High

None.

## Medium

**M1 — `capacity_weighted` also changes the winter store target, but it is described as a birth-gate-only fix.**
- **Evidence.** `supply_per_day` applies the weighting (`food_economy.rs:139-150`). It feeds `annual_mean_supply`, which the birth gate uses, and also `winter_supply_per_day` (`food_economy.rs:53-55`). `stock_target` reads `winter_supply_per_day` for the summer and autumn winter-prep target (`food_economy.rs:84-95`, line 92).
- **Effect.** In `game_v16` worlds with capacity-3 bushes, the weighting lowers the estimated winter supply by ¼. That raises the store target and changes foraging/hoarding behaviour, not only births.
- **What the docs say.** DECISIONS L615 ("the birth gate's supply estimate counts each bush …"), METRICS L2022/2040 and the `rules.rs:145-149` doc comment ("in the supply estimate") never mention the store target.
- **Why it matters.** Goal 7.1 allowed only "fixing the estimate for v16" for the birth gate. Neutral and capacity ≥ 4 worlds are unaffected (the weight is 1).
- **Fix.** Do not change the code: the frozen results depend on it. Document in DECISIONS, METRICS (frozen knobs) and the `SustainRule::capacity_weighted` doc comment that the weighting feeds both the birth gate and the winter-prep store target. Say in the report that the knob has this second effect in capacity-3 worlds.

**M2 — Tuning search is understated: 13 scratch configurations ran outside the three judged rounds, and METRICS lists only some of them.**
- **Evidence.**
  - `/Users/rexxa/work/p10bin/scratch/out/{A..M}/summary.json`, seeds 5000–5031 × 120 days, env-override probe.
  - 11 runs before round 1 (A–K, 16:35–16:45), with exposure sets [5,4,6]/2, [6,4,7]/2, [5,3,5]/1, [7,5,8]/2, [10,4,5]/2, [12,4,5]/2, [9,4,6]/1 plus capacity-scale and v16_neutral variants (H, I, J, K).
  - 2 more runs before round 2 (L, M, 16:57).
- **What the docs say.** DECISIONS L614 says "eight knob sets". METRICS L2022 lists 7 outcomes and omits [5,3,5]/1 (share 0.008) and [7,5,8]/2 (share 0.156, max share 0.55). METRICS L2040 states "Strikes used: 2 of 3".
- **Why it matters.** Goal 4.4/8 limits tuning to at most three rounds judged on tuning seeds. The exploration is disclosed and used no judged or pilot seeds, and held-out H2–H6 pass on the frozen commit, which bounds the overfitting risk. Even so, "2 of 3" reads as far less search than took place.
- **Fix.** In METRICS, add a table of all 13 scratch runs (knobs, scene, share, max share, starvation). Change the strike line to "2 of 3 judged rounds, after 13 disclosed scratch configurations on seeds 5000–5031". Repeat this in REPORT_PASS10 (자율 결정 / 정직한 판단).

## Low

**L1 — H5's "no new failure" escape has no direction check and no unit test.**
- `evaluate_pass10.py:252-258` passes a population clause whenever v15 also fails it, however much worse v16 is.
- v16_neutral's H5 PASS relies on this escape twice:
  - tuning `Y9_idle_1200`: v16n median 0.620, v15 0.634, limit ≤ 0.60;
  - held-out `Y9_work_1200`: v16n 0.344, v15 0.308, limit ≥ 0.35.
- In both cases v16n is better than v15, so the verdict stands under a stricter rule.
- **Fix:** require `mine` to be no worse than `theirs` when escaping. Add a unit test. Footnote the escaped clauses in the METRICS H5 cells (round 2 and held-out).

**L2 — The pilot's "exposure" attribution label omits the birth-gate change.**
- `diversity_p10.py:191` labels the v15 → v16_neutral difference "exposure", and so does `p10-5_pilot/diversity.md:165-167`. That difference also contains utilization 0.81 → 0.78.
- METRICS states the caveat; the committed deliverable does not.
- **Fix:** relabel it "exposure (+ utilization 0.78)" or add a note line in `markdown()`.

**L3 — `harness_variety_independent_of_replicate` cannot test what its comment claims.**
- The "rebuilt" block (`harness_revitalize_pass10_variety.rs:111-117`) builds a fresh world with no fork in force, so "a world built while the replicate in force is 3" is not exercised.
- The keyed-formula check covers slot 0 only (`:92`).
- The structural guarantee holds (check 2).
- **Fix:** assert all eight slots (and the climate roll) against `keyed_u64(seed, STREAM_WORLD_VARIETY, 0, 0, i, 0)`, and assert that replicate 3 would give a different raw value for at least one slot. Drop the misleading comment.

**L4 — No test pins the encoding of `DeathReason::Exposure`.**
- `event.rs:968` `death_reason_as_str` and the `as_u8` tests cover only the old four variants. `as_u8` is documented as "a locked contract" for the GDScript colour switch.
- **Fix:** add `assert_eq!(DeathReason::Exposure.as_str(), "exposure")` and `assert_eq!(DeathReason::Exposure.as_u8(), 4)`.

**L5 — A CLI test cannot fail on the verdict.**
- `test_evaluate_pass10.py:147`: `self.assertIn(main(...), (0, 1))`.
- **Fix:** assert the exit code the synthetic rounds must produce, or assert H3/H5 explicitly.

**L6 — The reported Y3 births−deaths margin ignores exposure deaths.**
- `evaluate_pass4.total_deaths` (`evaluate_pass4.py:168-169`) sums four causes. It feeds the reported Y3 margin (`evaluate_pass8.py:202`, `evaluate_pass6.py:217`), so v16 reports overstate the margin.
- It is reported-only and disclosed in the `evaluate_pass10.py:36-37` docstring.
- **Fix:** add `deaths_exposure` when the key is present (it does not change any judged clause), or mark the v16 Y3 margin as "exposure excluded" in the outputs.

**L7 — The H1 receipt is worded ahead of the evidence.**
- METRICS L2038 says "H1 here takes the legacy pin and the v8–v15 reproductions as receipts (Part 0 and the guards …)".
- Round-2 and held-out `h_targets.json` pass `--v8-v15-repro PASS`. However, Part 0 and the guards reproduced only v13–v15 and legacy; v8–v12 are Part 7 runs (`p10-7_final_*`, in progress).
- **Fix:** say "v13–v15 + legacy reproduced; v8–v12 pinned by tests, probe reproduction in Part 7". Re-evaluate H1 with the Part 7 receipts.

**L8 — The frozen commit hash was recorded after Part 4 ran.**
- Goal 4.4: Parts 4–5 run on a frozen commit "recorded in METRICS.md before they start". The knobs were recorded in `58119582` itself, but the hash first appears in `ed2d4b48` (17:22:02), after the held-out runs (17:07–17:22). The pilot started at 17:22:11, after the record.
- **Fix:** state this in DECISIONS (process only; the commit is unambiguous).

**L9 — The METRICS H8 prose and the v15 comparison need qualifiers.**
- METRICS L2108: "rain −0.59 exposure deaths (births −0.39)". The parenthetical is ambiguous; it presumably means the births effect.
- The "dark-night effect differs from v15" column (L2099) compares different worlds, since game_v16 seed 12000 ≠ game_v15 seed 12000 under world variety. It also compares replicate means over R = 4 against R = 8, so "differs" is close to automatic (gift1's 1/32 is the exception).
- **Fix:** write "(births effect −0.39)" and add one sentence on the confound.

**L10 — Pilot and fork provenance is operator-asserted.**
- `meta.git_sha` is the `WORLDSIM_GIT_SHA` environment variable set in `p5_pilot.sh` / `p6_forks.sh`, not a build-embedded SHA.
- The indirect evidence agrees (binary mtime after the commit; no simulation Rust changed since).
- **Fix (future):** embed the build SHA via `env!`/`build.rs`, or record `shasum` of `wm_frozen` in `commands.md`.

**L11 — The 11.5 scratch exploration went outside the allowed step.**
- DECISIONS L624 / METRICS L2084 use climate weights [383, 234, 383]. The middle entry moves −100, outside section 8's ±50 per entry. The range recordings on 30000–30111 were also made before any declaration.
- Nothing was adopted, and the larger-than-allowed change still leaves wood η² at 0.111, so the conclusion ("no allowed change reaches 0.30") is if anything strengthened.
- **Fix:** say explicitly that the third set exceeded the allowed step and served only as an upper-bound check.

**L12 — The live game can still panic in worldgen.**
- `terrain.rs:111-118` `expect(...)` and `terrain.rs:127` `panic!` predate this pass. `reset_world(seed)` now builds `game_v16` worlds with forest down to 220‰ and up to 15 bushes, which makes the repair harder.
- The 2,000-seed sweep (METRICS, Part 4 worldgen counts) found no panic, but no test guards it.
- **Fix (optional):** add a worldgen-only sweep test (for example 0..2,000 `game_v16` tick-0 worlds) or note the residual risk.

**L13 (nit) — A duplicated assertion.**
- `probe/args.rs:141-142` repeats the same `game_v16_neutral` assertion twice.

## What I did not find

- No v8–v15 behaviour change, no unkeyed or replicate-dependent variety draw, no order-dependent loop, no exposure death outside `Cold`, no healing-block off-by-one, no missed death counter in the probe, recorder or FFI.
- No mismatch between the committed numbers and the raw data for H2, H3, H6, H7, H8 or H10.
