# Pass 10 — independent review, round 2

Reviewer: fresh-context read-only `general-purpose` subagent, 2026-10-12, HEAD `50d027b7`. Scope: `git diff 341e4453..HEAD`, which contains round 1's fixes (`00c65bfa`) and the unstaging commit (`50d027b7`); PASS10_GOAL.md sections 4, 6, 7, 8, 10, 11, 12 and 13; METRICS/DECISIONS `## Pass 10`; the committed `h_targets.json`, pilot and fork outputs; and the untracked draft `REPORT_PASS10.md`. Nothing was built and no cargo command was run. The only command executed was `python3 -m unittest test_evaluate_pass10` (7 tests, OK, no files left behind) plus read-only python and `shasum`.

## Independent recomputations

- **H5 escape.** For all 8 committed `h_targets.json`, I recomputed `direction` / `median_of` / `no_worse` on every population clause. Escapes happen only on Pass 6 Y9:
  - round 1 v16: `idle_1200` 0.631 vs 0.634, `work_1200` 0.333 vs 0.308;
  - round 1 v16n: `idle_1200` 0.619 vs 0.634, `work_1200` 0.30769 vs 0.30769 (an exact tie);
  - round 2 v16 and v16n, and both timing runs: `idle_1200` 0.621 / 0.620 vs 0.634;
  - held-out v16: `idle_1200` 0.6168 vs 0.6190;
  - held-out v16n: `work_1200` 0.344 vs 0.308.

  Every one of these escapes is correct: v16 is no worse in the stated direction. None is wrong, and no failing clause was refused.
- **Regenerated `h_targets.json`.** Against `341e4453`, every value is identical except the `requirement` text and the 23 added `escaped_as_no_worse_than_v15` keys, so no verdict changed. This confirms DECISIONS item 3.
- **H1 receipts.** The `p10-7_final_*` summaries (legacy, v8, v9–v14 × 10/120 days) equal `p9-6_final_*` on every shared leaf. The only differences are `schema_version` and timing (`ms_per_agent_tick`, `wall_seconds`), and `determinism_checked` is true in all 14 runs.
- **Provenance.** I computed the sha256 of the binaries:
  - `p10bin/probe_frozen` = `p10bin/probe_p3r2` = `rust/target/release/worldsim_probe` (built 18:28) = `c10bd376…`;
  - `p10bin/wm_frozen` = `rust/target/release/worldsim_wm` = `282037ec…`.

  Both match `p10-5_pilot/commands.md` and `p10-6_forks/commands.md`.
- **The 13 scratch runs (M2).** I read `/Users/rexxa/work/p10bin/scratch/out/{A..M}/summary.json`. The exposure knobs, the share of seed-winters with a death, the maximum share and the starvation/dehydration figures match METRICS's table on every row. The utilization and weighting columns cannot be checked from the outputs, because the env overrides are not echoed into the `rules` block. They are consistent with E→H (starvation 8 → 0) and I→J.
- **H2, H3, H4, H6.** The METRICS numbers for round 1, round 2 and held-out match the JSON:
  - H3 shares, maxima and severity means;
  - H4 deaths 58 + 76 + 131 + 164 = 429, none outside a cold window;
  - H2 smallest margin 7.2 and smallest retention 0.80, both in held-out v16;
  - H6 means and climate shares.
- **H7 and pilot deaths.** They match `diversity.json`: v16 64.67 + 2.32 = 66.99, v16n 70.74 + 2.00 = 72.74. The attribution key is renamed and its numbers are unchanged.
- **H8 / G0a.** They match `fork_metrics.json`: rot_wood 0.5625 / +1.180, rain 0 / −0.594, cold_snap 0.0625 / +0.156, gift_wood −0.625; G0a 224 / 224 and 224 / 224.
- **Staging fix.** `00c65bfa` added 165 `verification/**.log` files and `50d027b7` deletes exactly 165, plus one DECISIONS line. No other unintended file was added: the other 29 added files are `p10-7_final_*` summaries and `review_round1.md`.

## Round-1 findings: status

| Finding | Status | Evidence |
|---|---|---|
| M1 capacity weighting also raises the store target | **Confirmed (documented)** | `rules.rs:145-152` doc comment; METRICS frozen-knob line; draft report L220. The inline comment `rules.rs:424-425` still says "the birth gate" only (see L3 below). |
| M2 exploration understated | **Confirmed** | The METRICS table of 13 runs matches the scratch outputs; the strike line was reworded; the DECISIONS line was corrected in place. |
| L1 H5 escape has no direction check | **Implemented, but the rule is wrong for some clauses** | See M1 below. It is correct for every clause that actually escaped. |
| L2 attribution label | **Confirmed** | `diversity_p10.py:191-192`, `diversity.json`, `diversity.md:167`. The caveat is still missing in one DECISIONS line and one draft-report line (L2 below). |
| L3 replicate-independence test | **Confirmed** | `harness_revitalize_pass10_variety.rs:93-108` checks all 7 uniform slots plus the climate roll against `keyed_u64(…, slot, 0)`, with slot order equal to `variety.rs:95-108`. It fails if any slot, the replicate or the range formula changes. The "rebuilt" block (`:125-128`) is still a determinism tautology, but its comment is now accurate. |
| L4 encoding of `Exposure` | **Confirmed** | `event.rs:974-981` pins `as_str` "exposure", `as_u8` 4 and `OldAge` 3. |
| L5 CLI test | **Confirmed** | `test_evaluate_pass10.py:161-164` asserts exit 1 and H3 false. |
| L6 Y3 margin excludes exposure | **Confirmed (documented)** | `evaluate_pass10.py:36-40`. |
| L7 H1 receipt wording | **Confirmed** | METRICS round-2 paragraph; recomputed above. |
| L8 frozen hash recorded late | **Confirmed (recorded)** | DECISIONS item 10. |
| L9 two ambiguous sentences | **Confirmed** | METRICS Part 6 paragraph. |
| L10 provenance | **Confirmed** | sha256 recomputed above. |
| L11 11.5 third set beyond the allowed step | **Partly** | Recorded only in DECISIONS item 13. The METRICS 11.5 line and the original DECISIONS Part 5 line are unchanged (L4 below). |
| L12 worldgen sweep test | **Confirmed** | `harness_revitalize_pass10_variety.rs:162-182`. Its assertion is stricter than "no panic" (L5 below). |
| L13 duplicated assertion | **Confirmed** | `probe/args.rs:141`. |
| Staging mistake | **Confirmed** | 165 added / 165 removed; the files stay on disk. |

## Section 13 checks, fresh pass

| # | Check | Result |
|---|---|---|
| 1 | v8–v15 invariance | **PASS.** `exposure` is applied only under the rule (`fire.rs:154-156`, `survival/mod.rs:163,191-196`). `utilization` 0.78 is set only in `game_v16_neutral` (`rules.rs:384-403`). `capacity_weighted` is set only in `game_v16_with` (`rules.rs:423-431`) and is false in the frozen literals (`rules.rs:590,648`). Variety is used only by the v16 constructors. All 14 Part 7 reproductions match; the v15 guards and Part 0 match. |
| 2 | Variety draws use replicate 0 | **PASS.** `variety.rs:50-52` and `:94` hard-code subject, tick and replicate 0; `draw(seed)` takes nothing else. The new L3 test pins this. |
| 3 | No order dependence | **PASS.** `fire.rs:128-150` sorts agents by AgentId; `expose` books deaths after the loop (`fire.rs:163-203`). `weighted_source_count` sorts before summing (`food_economy.rs:121-135`). |
| 4 | Exposure deaths only from `Cold` | **PASS.** `exposed` ⊆ `cold` (`fire.rs:136-147`). H4: 429 / 429 deaths lie inside a cold window. |
| 5 | Heal block | **PASS.** `survival/mod.rs:191-196` is gated by `heal_block.is_some_and(...)` and uses `Cold` or `blocks_healing`. |
| 6 | `DeathReason::Exposure` counted | **PASS** within scope: `survival/mod.rs:100`, `event.rs:87,106`, `world_node.rs:2775`, `probe/deaths.rs:261,330`, `pass10.rs:20`, `summary.rs:56,242`, `export.rs:155`, `fork_metrics.py:142,180`, `diversity_p10.py:119-121,180`. These sums exclude exposure by design and are disclosed: `hud_status_panel.gd:234` (merge note 2b), `bench_v0/v1` (goal 6.3), `evaluate_pass4.total_deaths` (L6), and `fingerprint.rs` (frozen by 4.3). |
| 7 | Pilot/fork provenance; overstatement | **PASS.** The hashes match. METRICS and DECISIONS report H7 and H8 as **FAIL** and H5 for `game_v16` as "reported only". The overstatements I found are minor (L1, L2, L5 below). |

## High

None.

## Medium

**M1 — The stricter H5 escape compares the wrong statistic for two population clauses and reads the direction from descriptive text. The new unit test encodes one of these errors. No committed verdict is affected.**

- **Evidence.**
  - `evaluate_pass10.py:287-315`: `direction()` searches the whole requirement string for `>=`/`<=`, and `median_of()` takes `observed["median"]`, or else the median of the first nested dict.
  - It is applied to every key with the prefixes in `H5_POPULATION_PREFIXES` (`:62-64`), to every Y9 key, to X3 (b)–(e) and to X4 (`:270-279`).
- **Wrong escape possible:**
  - `Y4_every_seed`. The clause is `seeds_q["min"] >= .40` (`evaluate_pass6.py:242`), but the escape compares **medians**. A v16 run with a better median and a worse minimum (more failing seeds) would escape.
  - `Y5_autumn_wood`. The clause is `autumn.median >= 1.5 × summer.median` (`evaluate_pass6.py:253-255`, observed `{"autumn": …, "summer": …}`). `median_of` returns the **autumn** median only and ignores summer. A run whose autumn median rose while its ratio got worse would escape. The result also depends on dict order: if `summer` came first, the summer medians would be compared with a "+1" sign.
  - `test_evaluate_pass10.py:146-148` asserts exactly this autumn-only behaviour as correct (the nested case: autumn 2.0 / summer 1.0 vs autumn 1.5 with no summer → True).
- **Direction misread, saved by accident:** `X3_d_two_or_more`'s text is "share of seed-winters with **>= 2** dark nights in [0.10, 0.50]". `direction()` returns +1 because of the descriptive "≥ 2", although the bound is two-sided. It refuses only because the observed block (`{"share", "n"}`) has no median.
- **Refused although the direction is readable:** `X3_b` (the mode share ≤ 0.75; observed `{harsh: {mode_share}, normal: {mode_share}}`) can never escape. This is conservative, but the docstring's "no escape when the direction cannot be read" does not describe it: here the direction can be read and the statistic is missing.
- **Integration not tested.** The tests call `no_worse()` and `direction()` directly. Nothing exercises `population()` (`:255-262`). Reverting it to `mine["passed"] or not theirs["passed"]` would keep the suite green.
- **Current impact.** None. Every escape in the committed files is a Y9 median clause, where the rule is correct, and Y4/Y5/X3 pass for both scenes in every run. The defect matters if `evaluate_pass10.py` is reused, for example on Part 7's final runs or in Pass 11.
- **Fix.**
  - Replace text parsing with an explicit per-clause table of `(statistic, sign)`:
    - Y4 `every_seed` → `observed["min"]`, +1;
    - Y5 `autumn_wood` → `autumn.median − 1.5 × summer.median`, +1;
    - X3 b → `−max(mode_share)`;
    - X3 c, d, e → no escape (two-sided or compound), stated explicitly;
    - X4, Y4 `store_share`/`winter`, Y5 `autumn_work`, Y6, Y9, Y10, Y11 medians → `median` with their sign.
  - Raise `EvidenceError` for any population clause missing from the table, so that a new clause cannot inherit a guess.
  - Change the nested test to expect `False` for an autumn-only comparison, and add one test that calls `h5`/`population` with both clauses failing and v16 worse (expected: fail).
  - Regenerate the 8 `h_targets.json`; I expect no verdict change.

## Low

**L1 — The list of escaped H5 clauses is incomplete, and one METRICS sentence overstates v16's H5.**

- **Evidence.**
  - Footnote ¹ (METRICS Part 4) names two escapes. The committed files contain more:
    - round 1, both scenes: `idle_1200` and `work_1200`; v16n `work_1200` is an exact tie, 0.30769 = 0.30769;
    - round 2 v16: `idle_1200` 0.621 vs 0.634;
    - held-out v16: `idle_1200` 0.6168 vs 0.6190.
  - METRICS: "H5 for v16 … every clause passes except Pass 6 Y9 `work_1500`". In held-out v16, `idle_1200` (0.617 against ≤ 0.60) fails its own bound and passes only through the escape.
  - Draft report L128 says "이 규칙으로 통과한 조항은 둘이다", which is true only for the judged v16n tuning and held-out runs.
- **Fix.** Add one line per run listing the escaped clauses (or give the count from `escaped_as_no_worse_than_v15`, e.g. "round 1: 2 per scene, one an exact tie"). Change the v16 sentence to "every clause passes or escapes as no worse than v15 (Y9 `idle_1200`), except Y9 `work_1500`".

**L2 — The exposure attribution still lacks the utilization caveat in two places.**

- **Evidence.**
  - DECISIONS Part 5 pilot line: "Exposure barely moves any η² (|Δ| ≤ 0.041)".
  - Draft report L298: "노출이 더한 η²는 |Δ| ≤ 0.04".
  - In both, the v15 → v16n difference also includes utilization 0.78. METRICS and draft L184 state this correctly.
- **Fix.** Add "(+ utilization 0.78)" in both places.

**L3 — Stale comments next to the frozen knobs.**

- **Evidence.**
  - `rules.rs:411-412`: "`WorldVariety::NEUTRAL` gives `game_v16_neutral()` exactly". It does not: `capacity_weighted` is true, and the 7.2 test clears the flag first (`harness_revitalize_pass10_variety.rs:24-31`). Only the digest is identical.
  - `rules.rs:424-425`: "the birth gate weighs bushes by their capacity". It omits the store-target effect that M1 of round 1 documented.
  - DECISIONS Part 1's H5 line still states the original rule ("passes when it passes or when v15 fails it as well") with no pointer to item 3.
- **Fix.** Reword the comments to "…gives `game_v16_neutral()` except `capacity_weighted` (no effect where every bush holds 4; digest-identical)" and "…the supply estimate behind the birth gate and the winter-prep store target". Append "(tightened in Part 7, item 3)" to the DECISIONS line.
- **Note.** Doc-comment edits in `rules.rs` can shift line numbers. Round 1's edit left the binaries byte-identical (verified above), but re-check the hash if you edit this again.

**L4 — L11 is recorded only in DECISIONS' handling list, and "no allowed change reaches 0.30" is stated as fact.**

- **Evidence.**
  - The METRICS Part 5 11.5 line and the original DECISIONS Part 5 line still read "every range widened by its step plus climate weights [383, 234, 383]". The middle weight moved −100, against section 8's ±50.
  - "No allowed change reaches 0.30" generalizes from three tested sets. η² is not monotone in range width, so the beyond-step set is not a strict upper bound.
  - The draft report L192 does say "허용 폭 이상", but L193 repeats "어떤 허용 조정도".
- **Fix.** In METRICS, add "(the third set exceeds the climate step, −100 vs ±50; an over-wide check, not an allowed set)". Write "none of the three tested sets, the widest beyond the allowed step, reached 0.30" instead of "no allowed change".

**L5 — The worldgen sweep test asserts more than "worldgen accepts the world", and the report draft words it as a universal claim.**

- **Evidence.**
  - `harness_revitalize_pass10_variety.rs:178` fails on `deadwood_spots_placed != v.deadwood_spots`. A shortfall is a legitimate outcome that H6 counts (goal 7.1(a), `camp.rs` places fewer silently). The test would turn a legal, reported outcome into a red gate if ranges or terrain change.
  - Draft L66: "월드젠은 모든 범위를 받아들인다" rests on 2,400 sampled seeds.
- **Fix.** Either keep the strict assertion and say in its comment that a shortfall is treated as a regression on purpose, or relax it to "no panic, all bushes placed, founders = draw" and report shortfalls separately. In the draft, write "샘플 2,400 시드(0–399, 1000–2999)에서 패닉·실패 없음".

**L6 (nit) — The probe's reported `food_supply_per_day` ignores the capacity weighting.**

- **Evidence.**
  - `sim-test/src/probe/long_run.rs:56` uses the unweighted `source_count`. In `game_v16` capacity-3 or capacity-5 worlds, the reported supply therefore differs from the estimate the birth gate and the store target actually use (`food_economy.rs:139-150`).
  - It is reporting only, and `long_run.rs` is near frozen code (4.3 freezes `trace`).
- **Fix.** Do not change the code. Add one METRICS sentence: "probe `food_supply_per_day` counts bushes unweighted; the game_v16 gate uses min(cap, 4)/4".

## What I did not find

- No wrong escape and no wrong refusal in any committed H5 verdict. No verdict changed between `341e4453` and HEAD.
- No v8–v15 behaviour change, no replicate-dependent variety draw, no order-dependent loop, no exposure death outside `Cold`, and no heal-block error.
- No mismatch between committed numbers and the JSON or raw outputs for H1 (Part 7 receipts), H2, H3, H4, H6, H7, H8, H10 or the 13-run table.
- METRICS and DECISIONS report H7 and H8 as FAIL and H5 for game_v16 as reported only. The only overstatements are L1, L2 and L5.
