# Mapped assertion audit result

**PASS — 467/467 mapped mutations killed** against `0189eead423c0496bda04d93b2e2da57a3efea27`. No survivors, invalid runs or invalid baselines. All 336 ledger rows are accounted for: 331 have killed predicate targets, and five change diagnostic wording only.

Execution used the detached `/Users/rexxa/work/art-production-audit` worktree, its own `rust/target`, and eight Cargo jobs. The audit worktree is clean after execution. Production source/test comparison: no drift.

The broader lexical inventory contains 758 candidates. The authorized assertion-focused run executed all 467 candidates mapped to changed assertions; the 291 additional inventory candidates were not run. Consequently `summary.json` deliberately reports full-inventory `audit_complete: false`, while `mapped-summary.json` and per-row `assertion_accounting_complete` report the completed requested scope. `inventory.json` is a discovery artifact, not execution evidence.

## Evidence

- `execution-scope.json`: exact target SHA and all executed candidate IDs.
- `results.jsonl`: 467 results, source hashes, baseline and mutant log paths.
- `assertion-accounting.json`: all 336 ledger rows and target outcomes.
- `mapped-summary.json`: scope-aware final result.
- `summary.json`: complete inventory denominator retained honestly.

## Diagnostic-only rows

| Ledger line | Test function | Written reason |
|---|---|---|
| 318 | `harness_p14_delta_a9_notification_feed_constants_present` | This row changes only assertion diagnostic wording; the adjacent ledger row for the same function changes and targets the actual predicate. Diagnostic wording does not inspect runtime source and has no independent source mutant. |
| 325 | `harness_p14_epsilon_a12_main_tscn_load_steps_equals_12` | This row changes only assertion diagnostic wording; the adjacent ledger row for the same function changes and targets the actual predicate. Diagnostic wording does not inspect runtime source and has no independent source mutant. |
| 327 | `harness_zoom_a17_main_tscn_load_steps_equals_12` | This row changes only assertion diagnostic wording; the adjacent ledger row for the same function changes and targets the actual predicate. Diagnostic wording does not inspect runtime source and has no independent source mutant. |
| 331 | `harness_needs_a5_renderer_and_bridge_contract` | This row changes only assertion diagnostic wording; the adjacent ledger row for the same function changes and targets the actual predicate. Diagnostic wording does not inspect runtime source and has no independent source mutant. |
| 333 | `harness_needs_a5_renderer_and_bridge_contract` | This row changes only assertion diagnostic wording; the adjacent ledger row for the same function changes and targets the actual predicate. Diagnostic wording does not inspect runtime source and has no independent source mutant. |

No survivor waivers were used. These are source-inspection mutation results; they do not establish visual or simulation acceptance.
