# Pass 15 — independent review

Two rounds at most (PASS15_GOAL.md 10.1), each by a fresh `general-purpose` subagent, read-only (no cargo, no builds, no benchmark binaries — timed batches could start at any moment; only `git`, `grep`, `sed` and short standard-library `python3` checks).

## Round 1 (2026-10-15 00:51–00:58 KST, HEAD `09fed98d`)

Checklist (all PASS):

| # | item | result |
|---|---|---|
| 1 | `tick` refactor: old statements in `tick_inner` in the same order; `Instant` only under a profile; no new field in an existing public struct | PASS |
| 2 | I1–I3 base vs final (and base vs Pass 13) | PASS — 0 / 155,815 and 0 / 105,187 differences; 20 of 20 sha256 equal |
| 3 | no existing scene / probe / recorder / GDScript calls the stress code | PASS |
| 4 | overhead number | PASS — −0.22 %, as every document states |
| 5 | one run recomputed from its sidecar (`stress_stress_v19_s15100_n500_side64_fed`) | PASS — mean 0.30475709 ms, p99.9 (rank 2,818 of 2,820) 0.660 ms, max 0.800 ms, steady sum 859.415 ms: all equal the JSON |
| 6 | one slope recomputed (`SettlementSystem`, stress fed) | PASS — b 0.976656 [0.96690, 0.98641], t(0.975, 38) = 2.02439; equal to `scaling.json` to all digits |
| 7 | P verdicts follow 7.5; Part 5 trigger follows section 9 | PASS — pooled p99.9 at n = 500 recomputed from the 8 sidecars (0.634 ms over 22,560 ticks) |
| 8 | allowed paths; `Cargo.lock` and `rules.rs` unchanged | PASS |
| 9 | other (gate script = Pass 13's steps + discoveries + quick + paths; perf.rs bucketing, gift timing, slow-tick logic; audit and inventory line spot-checks; budget arithmetic) | PASS, with the findings below |

Findings (0 High, 2 Medium, 7 Low) and dispositions:

| id | finding | disposition |
|---|---|---|
| M1 | Only the natural sweep started below the < 4 load limit; fed (13.41), repeatability (9.85), raw (9.85) and map (7.02) started above it inside the one-gate Part 3 batch yet carry `contended: false`; DECISIONS understated this. | Fixed: DECISIONS corrected; every sweep's start load listed in `scaling.md` ("Timed batches") and METRICS; P2's fed n = 500 runs and the three repeats rerun with one gate per sweep (`p15-2_stress_recheck/`, `p15-2_stress_repeat/`) and compared — recheck (gated, load 2.94): median mean 0.2971 ms (−2.1 %), pooled p99.9 0.608 ms, identical fingerprints on 8 of 8 seeds, P2 PASS; gated repeats CV 1.8 % (now the reported repeatability) |
| M2 | `scaling.json` could not be regenerated from committed files: without sidecars P2 silently became FAIL; ~360 sidecars lay untracked in the repository tree. | Fixed: a missing sidecar now makes P2 "not computable" (never FAIL; `perf_p15.py`, test updated); the eight n = 500 fed sidecars (8 × 11.5 KB) are committed as a deliberate exception; every other sidecar moved to `/Users/rexxa/work/p15-data/sidecars/` and read through `--sidecars`. |
| L1 | The rule's reference slopes came from 8 seeds × n 100–2,000, not the rule's own design (1 seed × n 100/500/1,000). | Fixed: `perf_p15.py` computes the reference slopes on the rule's design (seed 15100, n 100/500/1,000) and PERF_BUDGET section 2 quotes those. |
| L2 | P3's art figure omits two per-frame getters. | Fixed: labelled "partial (2 art-only getters untimed)" in the P3 row of PERF_BUDGET, METRICS and `scaling.md`. |
| L3 | Two "—" cells in PERF_BUDGET 1.2 for values below 1 %. | Fixed: 0.9 % and 0.1 % filled in. |
| L4 | Repeatability inputs not in the repository; the sweep's own run is a fourth repetition. | Fixed: the gated repeats are committed under `p15-2_stress_repeat/r{1,2,3}/`; the fourth repetition is mentioned. |
| L5 | `_betacf` guarded the old `c`. | Fixed (Numerical Recipes form); tests unchanged and green. |
| L6 | RUNLOG times overlap. | Fixed: the overlap is stated in the RUNLOG. |
| L7 | The overhead check always runs plain before profiled. | Noted (DECISIONS): −0.22 % is inside noise; later passes may use an ABBA order. Not changed (the measurement already ran). |

## Round 2 (2026-10-15 01:46–01:52 KST, HEAD `da8e45c7`, while the gate ran)

Verified by recomputation: the recheck (median mean 0.29714 ms, pooled p99.9 0.608 ms over 22,560 ticks, max 0.823 ms, 8 of 8 fingerprints equal), the gated repeats (CV 0.017610), the batch start loads against `batches.log`, regeneration of `scaling.json` from committed files only (P2 PASS), the rule-design reference slopes, the supplementary medians, maxima, crossings (B1 4,347, B2 3,601, all-sizes fit 4,484) and p99.9/mean ratios, eight PERF_BUDGET sections against `scaling.json`, the source lines of PERF_BUDGET section 4, and the allowed paths (every run JSON carries `git_sha` 8f052142; no `rust/` change after it).

Findings (0 High, 1 Medium, 7 Low) and dispositions — all fixed in the text (one small script fix):

| id | finding | disposition |
|---|---|---|
| M1' | Three headlines still said the whole Part 3 batch was "not contended" (DECISIONS Part 3 bullet and RUNLOG row 3, METRICS Part 3 heading). | Fixed: each now names the gate load and the four sweeps that started above the limit. |
| L-a | PERF_BUDGET said the repeats were "each batch gated"; they ran as one gated batch. | Fixed ("three runs in one gated batch, start load 3.57"). |
| L-b | The fourth repetition was not mentioned anywhere. | Fixed: the sweep's seed-15100 run (0.3048 ms) and the recheck's (0.2970 ms) are named as a fourth and fifth repetition (PERF_BUDGET 1.1, report). |
| L-c | The P2 row showed the contended sweep without a note. | Fixed in PERF_BUDGET, METRICS and the report: "sweep started at load 13.4; gated recheck 0.297 / 0.608 ms, also PASS". |
| L-d | "Both budgets hold to ≈ 3,000–3,600" rests on the pooled p99.9; per world, B2 already fails at 3,000 (worst 8.32 ms); the supplementary sidecars were outside the repository. | Fixed: both readings stated, ≈ 3,000 called the conservative figure; the 32 supplementary sidecars are committed. |
| L-e | The rule's slope step had no command; `perf_p15.py` crashed with an empty natural directory. | Fixed: the command and the `scaling.json` key are in PERF_BUDGET 2(b); the P1 row now renders "no natural runs" instead of crashing (checked on the recheck directory). |
| L-f | 2(a)'s reference came from 2-day 8-seed runs, not the rule's `quick` run. | Fixed: stated, with the one `quick` reading (0.306 ms at load ~12) and that the same-day before/after pair decides. |
| L-g | Section 4 skipped `StockpileConsumeSystem` (0.023 > movement's 0.017). | Fixed: the list's selection rule (audited systems only) is stated and the skipped system named. |
| L-h | `AgentMovementSystem` natural 0.75 % printed as 0.8 %; table 1.2 is not a strict top 10. | Fixed (0.7 %; pointer to the full top 10 in `scaling.md`). |

Two review rounds reached; no further round.
