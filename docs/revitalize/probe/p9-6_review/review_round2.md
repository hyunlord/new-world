# Pass 9 independent review, round 2 (fixes in `ba278357`)

I worked read-only: no builds and no cargo runs. I ran `test_fork_metrics.StatisticsTests` (9 tests, all OK) and the numpy edge cases below.

**Verdict:** no High findings. One Medium finding, about process. The rest are Low.

## Findings

### R2-1 (Medium, process): no gate has run on the fixed code

- **Evidence:**
  - The gate file committed with the fixes, `docs/revitalize/verification/p9-5/gate.txt:1`, says `head 05bb3ecd`, which is the pre-fix commit.
  - The DECISIONS entry for round 1 does not mention a re-run.
- **What has no recorded run:**
  - the rewritten `harness_v15_newborns_carry_their_first_parents_key`;
  - the new `arguments_validate` lines (`export.rs:832-836`);
  - the two new Python tests.
- **Reading the code:** I believe everything compiles and passes. `Lineage` is `Copy`; `0..if … {400} else {40} {` parses; `fork("1440")` satisfies every rule in `parse`.
- **Fix:** run the full gate on `ba278357` (or later) and commit its `gate.txt` before the report.

### R2-2 (Low): the M1 closure is much larger than needed and its comment undercounts it

- **Where:** `harness_revitalize_pass9_rng.rs:79-92`.
- **The mismatch:**
  - The comment says "founder -> child -> grandchild".
  - The loop actually builds four levels: 24 founders, then 9,600 children, 384,000 grandchildren and 15,360,000 great-grandchildren.
  - That is about 15.75 M `BTreeSet` inserts plus a 246 MB `Vec`, roughly 0.5 GB peak, in the debug and release gate runs alongside other tests.
- **How much is needed:**
  - The first parent is the lowest-`AgentId` member (`settlement_system.rs:766-775`), so almost every birth in a village chains through one founder.
  - A 28-day run has about 22–26 births.
  - A newborn becomes a first parent only after every lower-id member of its settlement has gone.
  - The bounds of 400 / 40 are therefore ample, and two levels (about 394 k keys) already cover any plausible chain.
- **Fix:** use `0..2`, or build the closure by expanding only the keys that actually occur. Align the comment with the code.

### R2-3 (Low): the L4 floor is calibrated against the values' level, not their spread, and the noisy-seed counts ignore it

- **Where:** `fork_metrics.py:55`. The floor is `1e-12·mean(y²)`.
- **Why it is too loose:** rounding residue is about `(2⁻⁵²·|y|)²`, roughly `5e-32·y²`, so `1e-12` is 20 orders of magnitude looser than needed.
- **Failure it allows:** at level 1e5 with a single ±1 replicate difference (32 seeds × 8 replicates), the real V_u is 0.0078. The floor is 0.01, so `vrf_of` returns `(0, 0, None)`. The target would then be excluded from G0b as "no agent noise".
  - At level 1e4 this does not happen.
  - Current targets stay far below that level (cold member-hours in the low thousands), so there is no effect today.
- **Inconsistency with the counts:** `seeds_*_noisy` (`:90`) still use the exact test `np.std > 0`.
  - A constant fractional row counts as noisy: 3–4 of 6 rows in the L4 test matrix, and 16 of 32 for R = 7 or 10.
  - These counts feed `vrf_seeds_both_noisy`.
- **Fix:** use a per-seed tolerance (e.g. `ptp(row) <= 1e-9·max(1, |row|max)`), applied the same way in `vrf_of` and in the noisy counts.

### R2-4 (Low): the bootstrap exclusion is not visible in the outputs

- **Where:** the module docstring (`fork_metrics.py:19-24`) and the prose above the `fork_metrics.md` table. Neither says that quiet resamples are dropped.
- **What a reader sees:** rain dark nights shows `[1, 1]` after excluding 96 of 2,000 resamples (4.8 %). gift1 is in the same position.
- **Round 1's estimate was too low:** it said at most 1.3e-3. Analytically, (29/32)^32 = 4.3 % for 3 noisy seeds.
- **The gate is safe:** dropping +∞ draws can only lower `lo`, so G0b can only become stricter.
- **Fix:** say "percentiles over the non-quiet resamples (count in `bootstrap_quiet_resamples`)" in the docstring and the markdown, or show the count in the table.

### R2-5 (Low): G0a can still pass with nothing compared

- **Where:** `fork_metrics.py:253-254`.
- **The problem:** with no `--placebos` or no `--unforked`, the counts are 0 = 0, so G0a passes vacuously. L6 closed the partial-directory case only.
- **Fix:** require `placebo_runs > 0` and `replicate0_runs == len(seeds)·len(ids)`.

### R2-6 (Low): the L1 and L6 fixes have no unit test

- **L1:** the regenerated JSON is internally consistent: `after_{1,6,24,72}h` equals `median_moved_share_by_row[1,6,24,72]` for all six variants. But no test pins the tick lookup (`:293-296`).
- **L6:** no test covers the missing-run branch (`:247-248`).

### R2-7 (Low, documentation): L7 is stated as latent but cannot occur

- **Why it cannot occur:** `completed` gets at most one entity per position. `progress_ops` comes only from `sites_by_pos`, a map keyed by position (`construction_system.rs:104-115`, `:147-150`). A tie in the sort at `:201` is impossible by construction.
- **What remains latent:** only O8 itself (`RNG_INVENTORY.md:50`, which site wins at `:111-113`). That is a different line from the one DECISIONS cites as "the inventory's O8".
- **Side note:** the map is `std::collections::HashMap` with `RandomState`, so any reachable tie would break run-to-run determinism, not only cross-arm comparisons.
- **Fix:** reword DECISIONS item 8.

## Task 1: the fixes

**M1: correct.**
- *Would it catch the fallback?* A fallback key is `keyed_u64(seed, NEWBORN, sid, tick, u32::MAX, 0)`. It cannot be in the closure, whose slots are below 400 and whose subjects are lineage keys. Any newborn with that key that is alive on day 28 fails the assertion, and `!newborns.is_empty()` stops the test passing when there are no newborns.
- *Are the bounds enough?* Yes (see R2-2). About 24 births, almost all from one founder, versus 400 per founder and 40 per later parent.
- *Residual gaps:*
  - The test cannot tell which parent was used, so a regression that picked a non-lowest member would still pass.
  - Newborns that die before day 28 are not checked.
  - A parent whose `births` is not persisted is caught indirectly, by the distinct-keys assertion.

**L1: correct.**
- The fork experiment uses `every` = 60.
- 29,400, 31,920, 33,720 and 35,160 are all multiples of 60.
- With 72 h added, the latest is 39,480, which lies inside the run (cold_snap has 255 rows after 35,160).
- Rows are taken before that tick's interventions (`export.rs:309-321`), so the `start + 60h` row comes 60h ticks after the intervention.
- Windfall at 1 h is now 0.405.

**L3: correct.**
- `export.rs:627` rejects 0 and anything at or beyond `days·1440`.
- No tool or test uses fork tick 0 (`test_export_determinism` uses 1440; Rust tests use 1440).
- `record()` called directly with a fork at tick 0 is still unchecked (Low).

**L4: correct for the stated case.**
- My re-run: a constant fractional column now gives V = 0 for R = 3, 5, 6, 7 and 10. A noisy factual plus a constant shift gives `inf`.
- See R2-3 for what remains.

**L5: correct.**
- The test passes: 62 of 200 resamples are quiet and `lo` = ∞.
- The committed changes match DECISIONS: only rain's dark-night `hi` goes from ∞ to 1.
- See R2-4 for the missing documentation.

**L6: correct.**
- A missing run now becomes `{"identical": False, "missing_unforked_run": True}`, goes into `failures` and fails G0a.
- The committed JSON has 224 of 224 runs and no failures.

## Task 2: L2 and L7 documentation

- **L2:** accurate.
  - No key contains an `AgentId`. AgentIds are monotonic, so matched agents change relative order only when their birth order differs.
  - "Documented … in the report" points to `REPORT_PASS9.md`, which does not exist yet. Make sure it is included there.
- **L7:** see R2-7.

## Task 3: new or missed issues

**Introduced by the fixes:**
- The JSON schema now has the added `bootstrap_quiet_resamples` key.
- `failures` entries for missing runs have a different shape.
- `vrf_lo` and `vrf_hi` can be `null` even when `vrf` is set, if every resample is quiet.
- No consumer reads `fork_metrics.json` (grep finds no other reader), so nothing breaks.

**Checked and clean:**
- **Keys:** movement keys `(seed, stream, lineage key, tick, 0, r)` and lifespan keys `(seed, LIFESPAN, key, 0, slot, r_b)`. The newborn's tick enters only `born_tick`, not the key (`aging.rs:89-91`).
- **Founders:** they use the worldgen replicate 0.
- **Sequential draws in v15:** the only one is the no-`Lineage` movement fallback, which is unreachable. `MovementRng` is otherwise used only at worldgen and for the firewood and severity hashes, keyed on seed and year.
- **Fork semantics:** `replicate_at(t)` uses `t >= tick`; `apply_noise_fork(0)` is a no-op; the recorder applies fork, then `apply_due`, then `tick`.
- **VRF pairing:** `np.roll(-1)` pairs replicate k with k+1 within a seed.
- **v8–v14 gating:** the construction sort is gated on `shelter_planning || keyed_draws`.

## Confirmed fixes

- M1 (bounds sufficient; see R2-2)
- L1
- L3
- L4 (see R2-3)
- L5 (see R2-4)
- L6
- L2 documented accurately (pending REPORT_PASS9)
- L7 documented but overstated (R2-7)
