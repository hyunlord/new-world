# F3 population correction evidence

Original frozen evaluation is preserved in commit `e353f18f5b523b4198fd785b44c5ad962b15576d` and `probe/p17-b_eval/original_registered/`. Two pre-result notices disclose the post-launch correction. Simulation producer remains47b72440; the Rust diff is empty.

Test-only application first:9 tests,9 expected assertion failures, no setup/import errors. Production application afterward:9 focused tests PASS;45 Pass17 evaluator tests PASS; external report-role fixtures PASS. Original RED output is retained. `source_check.json` verifies syntax, four source hashes and AST identity of all10 unchanged numeric/kin definitions. Only population selection, optional reference fields and serialization change.

Full siblings include all recorded known ordered-parent matches, independent of residence or coexistence. First shared non-null-settlement context is optional metadata and remains null if absent. Controls require first shared non-null-settlement observation, excluding known direct/shared-parent kin, without any age-category restriction. No thresholds or bootstrap arithmetic change. Missing unknown founder ancestry is never invented.

The tests preserve existing numeric/kin/null-control assertions. One old assertion excluding unassigned siblings is intentionally corrected and documented; it was the erroneous population rule. New cases cover different settlements, nonconcurrent lives, unassigned/no-hourly sibling context, unknown/reversed parents, founder controls and first-observation timing. Test seeds remain below1000.

Offline corrected results receive separate source and input hash receipts after the corrected source is committed. The original tuning report remains in `p17-b_tuning/r1/`; the new analysis uses `r1-f3-corrected/`. This is a methodological correction, not a second tuning round or unchanged preregistration. Full technical gate and fresh second review remain separate requirements.
