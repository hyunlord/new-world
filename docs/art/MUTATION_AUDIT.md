# Mutation audits

The earlier [Revitalize Pass 2 audit](../revitalize/MUTATION_AUDIT.md) remains unchanged at its original path. Its results and survivor explanations are separate from this production audit.

## Art Production — 2026-10-10

**PASS: 467/467 mapped mutants killed; 0 survived, 0 invalid, 0 baseline-invalid.** Target commit: `0189eead423c0496bda04d93b2e2da57a3efea27` (`art-prod(p7): prepare complete capture and assertion audit evidence`).

All **336 ledger rows** were reconciled individually: **331 rows have killed mutation targets**, and **5 rows only change assertion diagnostic wording**, with written non-mutation reasons below. The run exercised 151 test functions. No survivor waivers were used.

The full lexical inventory has 758 candidates. This authorized run executed every one of the 467 distinct candidates mapped to changed assertions. The other **291 unmapped candidates are outside the changed-assertion execution scope and were not run**. Accordingly, `summary.json` retains `audit_complete: false` for the full inventory, while `mapped-summary.json` records `mapped_audit_complete: true` and per-assertion accounting is complete. The full inventory must not be reported as 758/758 killed.

### Execution and restoration

Execution ran in the detached `/Users/rexxa/work/art-production-audit` worktree, with `CARGO_BUILD_JOBS=8` and its separate `CARGO_TARGET_DIR=/Users/rexxa/work/art-production-audit/rust/target`. Each test function first passed its baseline, then ran each candidate with `--exact --test-threads=1 --nocapture`. Compile errors, zero selected tests and timeouts do not count as kills.

Elapsed time was **7 minutes 33.874 seconds (453.874 seconds)**, measured from the first baseline launch through the final result append, including the initial isolated build: **2026-10-10 12:02:15.405–12:09:49.279 UTC**. Inventory generation and report writing are outside this measured interval.

Every mutation restored its original file before continuing. The audit worktree was clean afterward. At completion, every audited source and test file matched the production checkout byte-for-byte. Later changes to those files require a relevant subset rerun; asset-only changes do not retroactively change this SHA-bound result.

### Evidence and logs

- [Execution scope](production/mutation/execution-scope.json): exact SHA and all 467 candidate IDs.
- [Scope-aware result](production/mutation/mapped-summary.json): completed mapped scope, row counts and restoration status.
- [Raw results](production/mutation/results.jsonl): 467 results with source hashes and exact `baseline_log` / `mutant_log` filenames.
- [Per-assertion accounting](production/mutation/assertion-accounting.json): all 336 rows, replacement lineage, concrete candidate IDs and outcomes.
- [Full-inventory summary](production/mutation/summary.json) and [inventory](production/mutation/inventory.json): preserve the wider denominator.
- [Mapping plan](production/mutation-plan.md): individual ledger-to-target mapping.

All baseline and mutant logs are retained beside `results.jsonl` under `docs/art/production/mutation/`. The first pair is [baseline](production/mutation/831058be474404b34214-ad118b09e74c-1791633735404740000-baseline.log) and [mutant](production/mutation/831058be474404b34214-ad118b09e74c-1791633735404740000-mutant.log); every other exact path is recorded in its result row.

### Diagnostic-only rows

These five rows alter failure messages rather than inspected source predicates. The corresponding actual predicates have separate mapped, killed rows; mutating diagnostic wording in Rust is outside the source-inspection audit and would not test renderer behavior.

| Ledger line | Function | Diagnostic change |
|---|---|---|
| 318 | `harness_p14_delta_a9_notification_feed_constants_present` | must equal 4 |
| 325 | `harness_p14_epsilon_a12_main_tscn_load_steps_equals_12` | load_steps must equal 19 |
| 327 | `harness_zoom_a17_main_tscn_load_steps_equals_12` | load_steps must equal 19 |
| 331 | `harness_needs_a5_renderer_and_bridge_contract` | renderer follows the rendered agent by identity at its native head anchor |
| 333 | `harness_needs_a5_renderer_and_bridge_contract` | renderer draws native warning atlas regions |

This audit verifies the changed source-inspection assertions. It does not establish visual quality, simulation correctness or device acceptance.
