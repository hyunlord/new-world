# M-CORE shadow comparison

**Latest status: round 4 shadow stage PASS — zero unexplained categories, zero residual port defects. The completed round-4 48-day calibration candidate passes outcome and reason-coverage checks; policy is frozen for held-out evaluation, while full M-CORE acceptance remains pending.** Rounds 1–3 below retain their source-bound history, including round-3 candidate acceptance; none substitutes for the corrected round-4 candidate.

## Round 1 — FAIL, not accepted

The ≥97% port-behaviour agreement bar fails in both steward modes. Recorded fingerprint identity and coverage/accounting pass independently; neither makes this policy acceptable. Category diagnoses below are provisional and fixes require a new, separately frozen round. No held-out seeds were accessed. Round 1 consumes one of the maximum four calibration rounds; this document does not launch or imply a rerun.

### Frozen provenance and identity scope

- Independent audit: `/Users/rexxa/work/core-data/shadow-round-1/round1-independent-audit.json`; SHA-256 `2e5719931a7f312fbc987f9746cb566b4924ee7ea0800bc726f42ddfad450635`.
- Source HEAD: `891c6cae7a2427b35b8f4524788c9861660cac48`.
- Policy SHA-256: `f983bd195f2babbffa66662cb7c9513c09868fa5f704d5cde46a47f2fe9dcf9e`; policy revision `15345750137061056887` (not a cryptographic identifier).
- Freeze manifest SHA-256: `ce2046c1c9b80096c1d862b88655947aaf57bbc1fb663514df568b6ef7013473`.
- Started `2026-10-26T15:32:49Z`; seeds 0–7, 20 days, colony, story/neighbours ON, no player orders, steward OFF and ON. Exact artifact hashes and process exits are in the independent audit and `round-start.json`.

**Identity PASS scope:** compare observed feature-ON, observation-disabled feature-ON control and feature-OFF control using existing initial/terminal agent/tallies/settlement and colony fingerprints plus daily colony fingerprints. Both modes have zero identity mismatches. This is not exhaustive serialized-state equality, a new save-migration claim, or the final baseline-isolation gate.

### Arithmetic

Agreement uses unique person-ticks and the final prediction after same-tick replay. Evaluation totals include replay and must not replace that denominator. Only actual admitted routine-entry comparisons where both choices are registered routine behaviours are the declared draw replacement; job skips remain outside the port denominator. A rest-duration draw replacement removes no extra port person-ticks ([CORE_DECISIONS](CORE_DECISIONS.md)).

| Steward | Person-ticks | Evaluations | Port denominator | Behaviour matches / denominator | Band matches / denominator | Both matches / denominator | Result |
|---|---:|---:|---:|---|---|---|---|
| OFF | 2,231,893 | 2,231,943 | 2,038,246 | 1,861,596/2,038,246 = 91.333235% | 2,034,536/2,038,246 = 99.817981% | 1,860,088/2,038,246 = 91.259249% | FAIL |
| ON | 2,304,000 | 2,304,037 | 1,691,103 | 1,595,848/1,691,103 = 94.367286% | 1,685,625/1,691,103 = 99.676069% | 1,594,025/1,691,103 = 94.259486% | FAIL |

Missing predictions, uncovered arms, pending settlement, observer errors and unmatched predictions are all zero in both modes. Replay adds 50 evaluations OFF and 37 ON. Declared routine-entry replacement person-ticks are 9,784 OFF and 8,418 ON; not all are disagreements. Total recorded disagreement occurrences are 185,859 OFF and 103,710 ON, including band-only differences and declared routine draws. Do not subtract this total from port behaviour matches.

### Agreement by old arm

Each entry is behaviour or band matches / port person-ticks; zero denominators stay explicit. Counts are unique person-ticks, not the Part 1 inclusive timer scope entries.

| Old arm | OFF behaviour / port | OFF band / port | ON behaviour / port | ON band / port |
|---|---:|---:|---:|---:|
| `activity_seeking` | 132,094 / 132,094 | 132,094 / 132,094 | 106,788 / 106,788 | 106,788 / 106,788 |
| `consume_activity` | 444,914 / 444,914 | 444,914 / 444,914 | 236,178 / 236,178 | 236,178 / 236,178 |
| `consuming` | 791,081 / 791,081 | 791,081 / 791,081 | 726,682 / 726,682 | 726,682 / 726,682 |
| `consuming_gather` | 1,602 / 1,602 | 1,602 / 1,602 | 1,123 / 1,123 | 1,123 / 1,123 |
| `consuming_social` | 7,434 / 7,434 | 7,434 / 7,434 | 5,524 / 5,524 | 5,524 / 5,524 |
| `draft_hold` | 0 / 0 | 0 / 0 | 4,975 / 4,975 | 4,975 / 4,975 |
| `flee_hold` | 22,903 / 22,903 | 22,903 / 22,903 | 4,337 / 4,337 | 4,337 / 4,337 |
| `forage` | 292 / 292 | 292 / 292 | 201 / 210 | 201 / 210 |
| `gather` | 1,447 / 1,447 | 1,447 / 1,447 | 929 / 929 | 929 / 929 |
| `job_skip` | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `memory_commit` | 328 / 328 | 328 / 328 | 422 / 422 | 422 / 422 |
| `migration_fallback` | 228,179 / 402,627 | 402,627 / 402,627 | 140,742 / 232,342 | 232,342 / 232,342 |
| `natural_commit` | 11,036 / 11,469 | 10,087 / 11,469 | 11,596 / 11,952 | 10,260 / 11,952 |
| `routine_entry` | 0 / 1,769 | 0 / 1,769 | 0 / 3,290 | 0 / 3,290 |
| `routine_started` | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `seeking` | 103,968 / 103,968 | 103,968 / 103,968 | 304,432 / 304,432 | 304,432 / 304,432 |
| `seeking_gather` | 43,398 / 43,398 | 43,397 / 43,398 | 36,015 / 36,015 | 36,015 / 36,015 |
| `seeking_social` | 72,920 / 72,920 | 72,362 / 72,920 | 15,904 / 15,904 | 15,417 / 15,904 |

### Disagreements and example ticks

All recorded categories are retained, including those below 0.1% of a mode’s person-ticks. The 0.1% review thresholds are 2,231.893 OFF and 2,304 ON (at least 2,232 / 2,304 occurrences). Thus the large migration-fallback mismatch requires review in both modes, and routine-entry failure also exceeds the threshold ON. Smaller failures remain visible and must not be silently counted as intended. Examples below are actual archived records; facts/score/full top-level world data remain in each mode’s observed `mind_shadow.json`.

| Category (old arm: behaviour:band → predicted behaviour:band) | OFF count (% all person-ticks) | ON count (% all person-ticks) | Actual example: mode, seed, tick, agent, predicted node | Diagnosis / disposition |
|---|---:|---:|---|---|
| `forage:forage:work->wander:idle` | 0 (0.000000%) | 9 (0.000391%) | ON, 0, 1368, 7, `wander` | Actionable eligibility/order facts miss legacy action. Inspect stored example and holder/threshold inputs; FIX PENDING, remains port. |
| `migration_fallback:wander:idle->rest:idle` | 174,448 (7.816145%) | 91,600 (3.975694%) | OFF, 0, 347, 4, `rest_between_activities` | Predicted rest_between_activities competes with legacy wander during routine rest. Redundant-node removal proposed; FIX PENDING rerun, remains port. |
| `natural_commit:drink:need->drink:critical` | 944 (0.042296%) | 1,058 (0.045920%) | OFF, 0, 1114, 1, `camp_critical_water_idle` | Same behaviour, differing critical-band attribution at a boundary/exit. Band mapping/policy diagnosis PENDING; not silently accepted as intended. |
| `natural_commit:drink:need->wander:idle` | 0 (0.000000%) | 3 (0.000130%) | ON, 0, 352, 9, `wander` | Actionable eligibility/order facts miss legacy action. Inspect stored example and holder/threshold inputs; FIX PENDING, remains port. |
| `natural_commit:eat:need->eat:critical` | 5 (0.000224%) | 278 (0.012066%) | OFF, 0, 19440, 8, `critical_thirst_idle_hunger` | Same behaviour, differing critical-band attribution at a boundary/exit. Band mapping/policy diagnosis PENDING; not silently accepted as intended. |
| `natural_commit:socialise:need->wander:idle` | 433 (0.019401%) | 353 (0.015321%) | OFF, 0, 1587, 9, `wander` | Social eligibility/peer facts disagree with successful legacy choice. Provider/order diagnosis and FIX PENDING; no intended exemption. |
| `routine_entry:no_choice:unmapped->wander:idle` | 1,769 (0.079260%) | 3,290 (0.142795%) | OFF, 0, 4563, 1, `wander` | Legacy routine attempt ends with no choice/place; prediction falls through to wander. Explicit no-place lifecycle being prepared; FIX PENDING rerun, not a draw replacement. |
| `routine_started:routine_craft:leisure->routine_explore:leisure` | 286 (0.012814%) | 260 (0.011285%) | OFF, 0, 233, 3, `enter_routine_explore` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_craft:leisure->routine_hearth_circle:leisure` | 513 (0.022985%) | 447 (0.019401%) | OFF, 0, 104, 8, `enter_routine_hearth_circle` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_craft:leisure->routine_repair:leisure` | 370 (0.016578%) | 309 (0.013411%) | OFF, 0, 203, 7, `enter_routine_repair` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_craft:leisure->routine_tend_fire:leisure` | 494 (0.022134%) | 432 (0.018750%) | OFF, 0, 203, 2, `enter_routine_tend_fire` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_explore:leisure->routine_craft:leisure` | 307 (0.013755%) | 247 (0.010720%) | OFF, 0, 1859, 8, `enter_routine_craft` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_explore:leisure->routine_hearth_circle:leisure` | 332 (0.014875%) | 336 (0.014583%) | OFF, 0, 203, 4, `enter_routine_hearth_circle` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_explore:leisure->routine_repair:leisure` | 199 (0.008916%) | 179 (0.007769%) | OFF, 0, 194, 9, `enter_routine_repair` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_explore:leisure->routine_tend_fire:leisure` | 312 (0.013979%) | 273 (0.011849%) | OFF, 0, 281, 6, `enter_routine_tend_fire` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_hearth_circle:leisure->routine_craft:leisure` | 496 (0.022223%) | 420 (0.018229%) | OFF, 0, 1424, 1, `enter_routine_craft` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_hearth_circle:leisure->routine_explore:leisure` | 338 (0.015144%) | 257 (0.011155%) | OFF, 0, 449, 5, `enter_routine_explore` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_hearth_circle:leisure->routine_repair:leisure` | 407 (0.018236%) | 319 (0.013845%) | OFF, 0, 1625, 1, `enter_routine_repair` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_hearth_circle:leisure->routine_tend_fire:leisure` | 525 (0.023523%) | 473 (0.020530%) | OFF, 0, 320, 6, `enter_routine_tend_fire` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_repair:leisure->routine_craft:leisure` | 355 (0.015906%) | 329 (0.014280%) | OFF, 0, 1850, 3, `enter_routine_craft` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_repair:leisure->routine_explore:leisure` | 242 (0.010843%) | 155 (0.006727%) | OFF, 0, 1521, 5, `enter_routine_explore` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_repair:leisure->routine_hearth_circle:leisure` | 406 (0.018191%) | 335 (0.014540%) | OFF, 0, 317, 9, `enter_routine_hearth_circle` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_repair:leisure->routine_tend_fire:leisure` | 392 (0.017564%) | 325 (0.014106%) | OFF, 0, 110, 6, `enter_routine_tend_fire` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_tend_fire:leisure->routine_craft:leisure` | 468 (0.020969%) | 425 (0.018446%) | OFF, 0, 329, 7, `enter_routine_craft` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_tend_fire:leisure->routine_explore:leisure` | 311 (0.013934%) | 281 (0.012196%) | OFF, 0, 1705, 5, `enter_routine_explore` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_tend_fire:leisure->routine_hearth_circle:leisure` | 566 (0.025360%) | 521 (0.022613%) | OFF, 0, 1406, 5, `enter_routine_hearth_circle` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `routine_started:routine_tend_fire:leisure->routine_repair:leisure` | 382 (0.017116%) | 309 (0.013411%) | OFF, 0, 386, 4, `enter_routine_repair` | Declared routine-entry lottery replacement; retain category under the stated inventory rule. Not evidence of overall acceptance. |
| `seeking_gather:gather:work->gather:critical` | 1 (0.000045%) | 0 (0.000000%) | OFF, 5, 8126, 3, `finish_gather_critical_thirst` | Same behaviour, differing critical-band attribution at a boundary/exit. Band mapping/policy diagnosis PENDING; not silently accepted as intended. |
| `seeking_social:socialise:need->socialise:critical` | 558 (0.025001%) | 487 (0.021137%) | OFF, 0, 1426, 9, `finish_social_need_thirst` | Same behaviour, differing critical-band attribution at a boundary/exit. Band mapping/policy diagnosis PENDING; not silently accepted as intended. |

### Next evidence required

Finish explicit category diagnoses, retain the frozen round-1 artifacts, and run a separately logged calibration round after source/policy freeze. Only a rerun can establish a fix. Recompute unique person-tick denominators and compare both band and behaviour, with ≥97% port behaviour agreement and zero unexplained categories before acceptance. Production switch, held-outs, why coverage, final baseline isolation, native UI acceptance and paired performance remain separate pending work. No current source edits rewrite this failed round’s results.

## Round 2 — threshold PASS, NOT ACCEPTED

The numerical ≥97% port-behaviour bar passes in both modes, and the independent audit reports fingerprint identity and complete accounting PASS. **This round is not accepted:** twelve known ON port defects remain (nine forage and three drink choices incorrectly become wander for Idle people retaining a Job). They remain mismatches in the counts and are not intended replacements. [CORE_DECISIONS D17](CORE_DECISIONS.md#d17--keep-job-presence-separate-from-current-job-ownership) requires correction and round 3. Round 1 above remains the original failed historical record.

### Provenance and arithmetic

- Audit: `/Users/rexxa/work/core-data/shadow-round-2/round2-independent-audit.json`; SHA-256 `8dcdd9a8c193dc56f54f5d59f82fce890e7522c557f2de8db7da1f6cc83c2bd8`.
- Started `2026-10-26T15:51:52Z`; source HEAD `891c6cae7a2427b35b8f4524788c9861660cac48` plus freeze manifest SHA-256 `7f78ea27cefe9d209a343dcfc4122b2d84db7e14bc0f76ad69a659f42221ca85`. The manifest binds the working source/binary; the shared HEAD alone does not distinguish rounds.
- Frozen policy SHA-256 `36fa1b148a5eae61eb30961eb316a8810474b26b660f5d9e743ebff3ffceec05`. Seeds 0–7 × 20 days, steward OFF/ON; no held-out access.
- Identity covers the existing five initial/terminal/daily fingerprint fields against same-binary control and all 20 daily colony fingerprints against feature-OFF. It does not establish exhaustive serialized-state identity. Coverage has zero errors, unmatched predictions, uncovered arms, missing predictions or pending settlement.
- Unique person-ticks, final replay prediction and the original port denominator are retained. Behaviour misses remain 1,769 OFF and 3,302 ON. No-place differences are still included; only admitted routine-entry draw comparisons retain their narrow replacement classification.

| Steward | Person-ticks | Evaluations | Port denominator | Behaviour matches / denominator | Band matches / denominator | Both matches / denominator | Disposition |
|---|---:|---:|---:|---|---|---|---|
| OFF | 2,231,893 | 2,231,943 | 2,038,246 | 2,036,477/2,038,246 = 99.913210% | 2,035,527/2,038,246 = 99.866601% | 2,035,527/2,038,246 = 99.866601% | Threshold PASS; not accepted |
| ON | 2,304,000 | 2,304,037 | 1,691,103 | 1,687,801/1,691,103 = 99.804743% | 1,686,465/1,691,103 = 99.725741% | 1,686,465/1,691,103 = 99.725741% | Threshold PASS; not accepted |

The strict category checker exited 0 in each mode. This is a limited checker result, not authorization to accept the known defects or switch production. The category totals are 10,420 OFF (24 categories) and 11,270 ON (25 categories); they include band-only differences and declared draws, so they are not behaviour-miss totals.

### Round-2 agreement by old arm

| Old arm | OFF behaviour / port | OFF band / port | ON behaviour / port | ON band / port |
|---|---:|---:|---:|---:|
| `activity_seeking` | 132,094 / 132,094 | 132,094 / 132,094 | 106,788 / 106,788 | 106,788 / 106,788 |
| `consume_activity` | 444,914 / 444,914 | 444,914 / 444,914 | 236,178 / 236,178 | 236,178 / 236,178 |
| `consuming` | 791,081 / 791,081 | 791,081 / 791,081 | 726,682 / 726,682 | 726,682 / 726,682 |
| `consuming_gather` | 1,602 / 1,602 | 1,602 / 1,602 | 1,123 / 1,123 | 1,123 / 1,123 |
| `consuming_social` | 7,434 / 7,434 | 7,434 / 7,434 | 5,524 / 5,524 | 5,524 / 5,524 |
| `draft_hold` | 0 / 0 | 0 / 0 | 4,975 / 4,975 | 4,975 / 4,975 |
| `flee_hold` | 22,903 / 22,903 | 22,903 / 22,903 | 4,337 / 4,337 | 4,337 / 4,337 |
| `forage` | 292 / 292 | 292 / 292 | 201 / 210 | 201 / 210 |
| `gather` | 1,447 / 1,447 | 1,447 / 1,447 | 929 / 929 | 929 / 929 |
| `job_skip` | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `memory_commit` | 328 / 328 | 328 / 328 | 422 / 422 | 422 / 422 |
| `migration_fallback` | 402,627 / 402,627 | 402,627 / 402,627 | 232,342 / 232,342 | 232,342 / 232,342 |
| `natural_commit` | 11,469 / 11,469 | 10,520 / 11,469 | 11,949 / 11,952 | 10,613 / 11,952 |
| `routine_entry` | 0 / 1,769 | 0 / 1,769 | 0 / 3,290 | 0 / 3,290 |
| `routine_started` | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `seeking` | 103,968 / 103,968 | 103,968 / 103,968 | 304,432 / 304,432 | 304,432 / 304,432 |
| `seeking_gather` | 43,398 / 43,398 | 43,397 / 43,398 | 36,015 / 36,015 | 36,015 / 36,015 |
| `seeking_social` | 72,920 / 72,920 | 72,920 / 72,920 | 15,904 / 15,904 | 15,904 / 15,904 |

### Round-2 categories and disposition

All raw categories are retained below; `unreviewed` is the archived category suffix, not an instruction to rewrite the raw report after review. Complete reviewed examples remain hash-bound in the audit.

| Raw category | OFF | ON | Actual example: mode, seed, tick, agent, node | Review |
|---|---:|---:|---|---|
| `forage:forage:work->wander:idle:unreviewed` | 0 | 9 | ON, 0, 1368, 7, `wander` | D17 known port defect: Job presence confused with ownership. FIX REQUIRES RERUN; no exemption. |
| `natural_commit:drink:need->drink:critical:unreviewed` | 944 | 1,058 | OFF, 0, 1114, 1, `camp_critical_water_idle` | D15 explicit critical-trigger band semantics; same behaviour, zero port denominator exclusions. |
| `natural_commit:drink:need->wander:idle:unreviewed` | 0 | 3 | ON, 0, 352, 9, `wander` | D17 known port defect: Job presence confused with ownership. FIX REQUIRES RERUN; no exemption. |
| `natural_commit:eat:need->eat:critical:unreviewed` | 5 | 278 | OFF, 0, 19440, 8, `critical_thirst_idle_hunger` | D15 explicit critical-trigger band semantics; same behaviour, zero port denominator exclusions. |
| `routine_entry:no_choice:unmapped->routine_no_place:leisure:unreviewed` | 1,769 | 3,290 | OFF, 0, 4563, 1, `routine_no_place` | D16 explicit no-place lifecycle versus legacy no_choice label. Keep full port denominator and raw mismatch; diagnostic draw only is replaced. |
| `routine_started:routine_craft:leisure->routine_explore:leisure:declared_routine_draw` | 286 | 260 | OFF, 0, 233, 3, `enter_routine_explore` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_craft:leisure->routine_hearth_circle:leisure:declared_routine_draw` | 513 | 447 | OFF, 0, 104, 8, `enter_routine_hearth_circle` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_craft:leisure->routine_repair:leisure:declared_routine_draw` | 370 | 309 | OFF, 0, 203, 7, `enter_routine_repair` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_craft:leisure->routine_tend_fire:leisure:declared_routine_draw` | 494 | 432 | OFF, 0, 203, 2, `enter_routine_tend_fire` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_explore:leisure->routine_craft:leisure:declared_routine_draw` | 307 | 247 | OFF, 0, 1859, 8, `enter_routine_craft` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_explore:leisure->routine_hearth_circle:leisure:declared_routine_draw` | 332 | 336 | OFF, 0, 203, 4, `enter_routine_hearth_circle` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_explore:leisure->routine_repair:leisure:declared_routine_draw` | 199 | 179 | OFF, 0, 194, 9, `enter_routine_repair` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_explore:leisure->routine_tend_fire:leisure:declared_routine_draw` | 312 | 273 | OFF, 0, 281, 6, `enter_routine_tend_fire` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_hearth_circle:leisure->routine_craft:leisure:declared_routine_draw` | 496 | 420 | OFF, 0, 1424, 1, `enter_routine_craft` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_hearth_circle:leisure->routine_explore:leisure:declared_routine_draw` | 338 | 257 | OFF, 0, 449, 5, `enter_routine_explore` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_hearth_circle:leisure->routine_repair:leisure:declared_routine_draw` | 407 | 319 | OFF, 0, 1625, 1, `enter_routine_repair` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_hearth_circle:leisure->routine_tend_fire:leisure:declared_routine_draw` | 525 | 473 | OFF, 0, 320, 6, `enter_routine_tend_fire` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_repair:leisure->routine_craft:leisure:declared_routine_draw` | 355 | 329 | OFF, 0, 1850, 3, `enter_routine_craft` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_repair:leisure->routine_explore:leisure:declared_routine_draw` | 242 | 155 | OFF, 0, 1521, 5, `enter_routine_explore` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_repair:leisure->routine_hearth_circle:leisure:declared_routine_draw` | 406 | 335 | OFF, 0, 317, 9, `enter_routine_hearth_circle` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_repair:leisure->routine_tend_fire:leisure:declared_routine_draw` | 392 | 325 | OFF, 0, 110, 6, `enter_routine_tend_fire` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_tend_fire:leisure->routine_craft:leisure:declared_routine_draw` | 468 | 425 | OFF, 0, 329, 7, `enter_routine_craft` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_tend_fire:leisure->routine_explore:leisure:declared_routine_draw` | 311 | 281 | OFF, 0, 1705, 5, `enter_routine_explore` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_tend_fire:leisure->routine_hearth_circle:leisure:declared_routine_draw` | 566 | 521 | OFF, 0, 1406, 5, `enter_routine_hearth_circle` | Declared admitted routine lottery replacement, narrow existing scope. |
| `routine_started:routine_tend_fire:leisure->routine_repair:leisure:declared_routine_draw` | 382 | 309 | OFF, 0, 386, 4, `enter_routine_repair` | Declared admitted routine lottery replacement, narrow existing scope. |
| `seeking_gather:gather:work->gather:critical:unreviewed` | 1 | 0 | OFF, 5, 8126, 3, `finish_gather_critical_thirst` | D15 explicit critical-trigger band semantics; same behaviour, zero port denominator exclusions. |

### D17 corrective boundary and pending round 3

Thirty-eight idle need, camp and survival-work predicates distinguish current job ownership from mere Job-component presence. Routine admission continues to require no Job. Consuming construction-site work is job-owned regardless of site occupancy; occupancy remains a separate post-choice restoration condition. Release bookkeeping must snapshot actual held job states so an external transition to Idle does not cancel a retained job. These are explicit port fixes, not new exclusions or retrospective relabeling of the twelve failures.

Round 3 is running separately. No partial round-3 artifacts were read for this update and no result is inferred. A completed independent audit, corrected-regression verification and the standing switch/held-out/why/performance gates remain required. This round consumes the second of at most four calibration rounds.

## Round 3 — shadow bar PASS, accepted for the shadow stage

Both steward modes exceed 97% port-behaviour agreement, with zero unexplained categories and zero residual port defects in the reviewed audit. The twelve ON Idle+Job defects from round 2 are absent; counts improve by exactly twelve ON behaviour and band matches. This accepts the shadow stage only. The full M-CORE goal is not complete: the completed 48-day calibration candidate result is recorded separately below; no production-switch, held-out, native-UI or performance acceptance is inferred.

### Frozen evidence

- Independent audit `/Users/rexxa/work/core-data/shadow-round-3/round3-independent-audit.json`: SHA-256 `e4e6b1af637b705e9594cb734242c43f0fc9a87cbc2fb458b45af2dd8449204c`.
- Policy SHA-256 **`7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8`**.
- Freeze manifest SHA-256 `020fcbbe2f410472319f969cb6e03a85deac19c229f0013959dbd93388f9821d`; source HEAD `891c6cae7a2427b35b8f4524788c9861660cac48` plus frozen working source/binary evidence.
- Started `2026-10-26T16:06:30Z`; calibration seeds 0–7 × 20 days, OFF/ON; no held-out access. This is round 3 of the maximum 4 calibration rounds.
- OFF reviewed categories: `/Users/rexxa/work/core-data/shadow-round-3/off/calibration-review.json`; SHA-256 `804f0ec58d9bf409af05e66e83c493afab83d3df80cf83ad6b6753679d04d678`; strict checker exit 0.
- ON reviewed categories: `/Users/rexxa/work/core-data/shadow-round-3/on/calibration-review.json`; SHA-256 `b3998173aa230f7af90b0f88aeef06f3f6e3601b5efef1b8957e7b43cc622c9b`; strict checker exit 0.

The independent freeze check passes. Existing initial/terminal/daily fingerprint identity and complete coverage/accounting pass in both modes: zero observer errors, unmatched predictions, uncovered arms, missing predictions or pending settlement. All 20 daily colony fingerprints match feature-OFF. This preserves the limited fingerprint-contract claim, not exhaustive serialized-state equality.

### Round-3 arithmetic

| Steward | Person-ticks | Evaluations | Port denominator | Behaviour matches / denominator | Band matches / denominator | Both matches / denominator | Shadow result |
|---|---:|---:|---:|---|---|---|---|
| OFF | 2,231,893 | 2,231,943 | 2,038,246 | 2,036,477/2,038,246 = 99.913210% | 2,035,527/2,038,246 = 99.866601% | 2,035,527/2,038,246 = 99.866601% | PASS |
| ON | 2,304,000 | 2,304,037 | 1,691,103 | 1,687,813/1,691,103 = 99.805452% | 1,686,477/1,691,103 = 99.726451% | 1,686,477/1,691,103 = 99.726451% | PASS |

Denominators are unchanged from prior rounds: unique person-ticks with final replay prediction. The remaining 1,769 OFF / 3,290 ON behaviour misses are the explicitly reviewed no-place label difference; they remain port mismatches, not excluded equivalences. D15 critical-context differences remain visible as band differences, with zero behaviour exclusions. Actual admitted routine lottery replacements remain 9,784 OFF / 8,418 ON person-ticks.

### Round-3 agreement by old arm

| Old arm | OFF behaviour / port | OFF band / port | ON behaviour / port | ON band / port |
|---|---:|---:|---:|---:|
| `activity_seeking` | 132,094 / 132,094 | 132,094 / 132,094 | 106,788 / 106,788 | 106,788 / 106,788 |
| `consume_activity` | 444,914 / 444,914 | 444,914 / 444,914 | 236,178 / 236,178 | 236,178 / 236,178 |
| `consuming` | 791,081 / 791,081 | 791,081 / 791,081 | 726,682 / 726,682 | 726,682 / 726,682 |
| `consuming_gather` | 1,602 / 1,602 | 1,602 / 1,602 | 1,123 / 1,123 | 1,123 / 1,123 |
| `consuming_social` | 7,434 / 7,434 | 7,434 / 7,434 | 5,524 / 5,524 | 5,524 / 5,524 |
| `draft_hold` | 0 / 0 | 0 / 0 | 4,975 / 4,975 | 4,975 / 4,975 |
| `flee_hold` | 22,903 / 22,903 | 22,903 / 22,903 | 4,337 / 4,337 | 4,337 / 4,337 |
| `forage` | 292 / 292 | 292 / 292 | 210 / 210 | 210 / 210 |
| `gather` | 1,447 / 1,447 | 1,447 / 1,447 | 929 / 929 | 929 / 929 |
| `job_skip` | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `memory_commit` | 328 / 328 | 328 / 328 | 422 / 422 | 422 / 422 |
| `migration_fallback` | 402,627 / 402,627 | 402,627 / 402,627 | 232,342 / 232,342 | 232,342 / 232,342 |
| `natural_commit` | 11,469 / 11,469 | 10,520 / 11,469 | 11,952 / 11,952 | 10,616 / 11,952 |
| `routine_entry` | 0 / 1,769 | 0 / 1,769 | 0 / 3,290 | 0 / 3,290 |
| `routine_started` | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `seeking` | 103,968 / 103,968 | 103,968 / 103,968 | 304,432 / 304,432 | 304,432 / 304,432 |
| `seeking_gather` | 43,398 / 43,398 | 43,397 / 43,398 | 36,015 / 36,015 | 36,015 / 36,015 |
| `seeking_social` | 72,920 / 72,920 | 72,920 / 72,920 | 15,904 / 15,904 | 15,904 / 15,904 |

### Reviewed category accounting

| Category family | OFF occurrences | ON occurrences | Accepted explanation and scope |
|---|---:|---:|---|
| Admitted routine draw differences | 7,701 | 6,632 | Declared routine lottery replacement only, both sides registered activities. |
| No-place: no_choice → routine_no_place | 1,769 | 3,290 | D16 explicit denial lifecycle; all remain in the port denominator as mismatches. |
| Natural critical-context band differences | 949 | 1,336 | D15 triggering urgency; identical dispatched need behaviour, zero behaviour exclusions. |
| Gathering critical-exit band difference | 1 | 0 | D15 critical exit context; identical gather dispatch/completion boundary. |
| Known Idle+Job port defects | 0 | 0 | Prior nine forage and three drink failures absent; D17 port fix, no relabeling. |
| Total recorded disagreements | 10,420 | 11,258 | Includes band-only and draw categories; not a behaviour-miss total. |

Every raw category is reviewed individually in the hash-bound calibration-review files (24 OFF, 23 ON), with exact counts and retained seed/tick/agent examples. No categories were merged away when summing across worlds. The two strict checks exit 0; unexplained categories and residual port defects are both zero. Earlier failed or unaccepted rounds and their diagnoses remain above as history, not overwritten by this stage result.

## Round-3 full-runtime calibration candidate — PASS, not held-out acceptance

The `mind_candidate` build executed the full mind runtime on calibration seeds 0–7 for 48 days, steward/defence ON and no player script. This is a training-world outcome check after shadow acceptance; it is not a held-out evaluation or default production-switch claim. `outcomes.json` explicitly records `phase: calibration`, `status: VALID`, `passed: true`, `acceptance_claim: false`.

| Outcome | Passed / denominator | Required | Result |
|---|---:|---:|---|
| Day-1 stockpile | 8/8 | 8 | PASS |
| Walled beds for all by day 10 | 8/8 | 8 | PASS |
| Zero cold deaths | 8/8 | 8 | PASS |
| Alive at day 48 | 8/8 | 8 | PASS |
| At most one wolf death | 8/8 | 8 | PASS |
| Exact item ledger | 8/8 | 8 | PASS |
| No firewood depletion during an observed cold snap | 5/5 exposed | 4 (= exposed − 1) | PASS |

Cold-snap exposure is 5/8 worlds; seeds 0, 2 and 4 were unexposed and count as neither successes nor failures. `probe.log` reports DETERMINISM PASS. Actual reason coverage totals are **135,368/135,368** first choices plus registered behaviour changes and **2,984/2,984** stricter critical choice transitions, with **0 failures**. The critical denominator includes first/node/behaviour/restart choices ending in the critical band; it is not just critical behaviour switches. These calibration coverage results do not satisfy the separate held-out requirement by substitution.

Evidence under `/Users/rexxa/work/core-data/shadow-round-3/candidate/`: `outcomes.json`, `parent-receipt.json`, `probe/summary.json`, `probe.log`, `probe.exit`. Outcome SHA-256 `54cdbfa0f7b26bda3c2e606937247b59d833f2a8d4009d90496320b3d163de28`; summary SHA-256 `70cccdc7e6a00703cba8ea04ef9210e6d13de6e1d94ec5c95c10bea21dc7e68b`; the outcome receipt also binds scorer and validator hashes. Candidate binary SHA-256 `d9616079b288e4acc1b1bfa493477e8ac75603d1903e824f50b31e1ba4f0f97b`.

Policy frozen for held-outs at `2026-10-26T16:12:57Z`: `/Users/rexxa/work/core-data/policy-freeze-round-3.json`, SHA-256 `edd0c751c67c053032743864f9355a35d92a013a3fba8502cb3f302aae0e4b09`. Policy bytes remain `7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8`. No tuning on held-out seeds 1232–1263 is permitted. Those worlds, paired BASE/new timing, final isolation, native UI, story-yield comparison and the full gate remain outside this calibration acceptance.


## Round 4 — shadow PASS; full-runtime calibration candidate completed below

The fourth and final admitted calibration round rechecks the D18 mind-only gather quarrel RNG provenance correction. Policy bytes are unchanged from round 3: `7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8`. The rare-branch effect regression and its RED/GREEN history remain recorded in [D18](CORE_DECISIONS.md#d18--replace-only-the-mind-gather-quarrel-random-key). Matching shadow results do not establish exposure or equivalence of that rare effect; no gather person-ticks are excluded.

### Frozen evidence and checks

- Started `2026-10-26T16:31:01Z`; source HEAD `891c6cae7a2427b35b8f4524788c9861660cac48`; seeds 0–7 × 20 days, both steward modes, unchanged story/neighbours ON and no player orders.
- Freeze manifest SHA-256: `ad4bf65d0bf1747f8e2572c9d630df48b36c2a14c7df1d80017aa243a1f80002`. The independent audit ran the strict frozen-source/binary check before reading round-4 result files; log: `/Users/rexxa/work/core-data/shadow-round-4/audit-freeze-check.log`.
- Independent audit: `/Users/rexxa/work/core-data/shadow-round-4/round4-independent-audit.json`; SHA-256 `fb444661fff91ca8495434306f9d044aa0628e6843b74024078b394ee30daa4f`. Its Markdown companion, all exact input paths/hashes, per-world counts and every retained category example remain alongside it.
- OFF category review SHA-256: `6699948932d3fef1eaf8bee56dc53c14589245cc3e636e3d1b316619cf7332a2`; ON: `fa6268166635b8d3b5b63bd488d4f117f6d22a98ea50c90cfd8399cbf022061f`. Files are each mode's `calibration-review.json`; both strict `core_gate_checks.sh shadow` checks exit 0, with logs retained as `calibration-review-check.log`.

All six capture commands exit 0. Identity matches all five initial/terminal/daily fingerprint fields against same-binary observation-disabled controls, and all 20 daily colony fingerprints against feature-OFF for every seed. This is the existing fingerprint scope, not exhaustive serialized-state equality. Coverage, final-replay and by-arm accounting pass; observer errors, unmatched predictions, missing predictions, uncovered arms and pending settlement are all zero.

| Steward | Person-ticks | Evaluations | Behaviour matches / port | Band matches / port | Result |
|---|---:|---:|---|---|---|
| OFF | 2,231,893 | 2,231,943 | 2,036,477/2,038,246 = 99.913210% | 2,035,527/2,038,246 = 99.866601% | PASS |
| ON | 2,304,000 | 2,304,037 | 1,687,813/1,691,103 = 99.805452% | 1,686,477/1,691,103 = 99.726451% | PASS |

### Category review and remaining boundary

The summed categories equal round 3: 24 OFF categories / 10,420 occurrences and 23 ON categories / 11,258 occurrences. This includes declared routine-entry draw differences (7,701 OFF / 6,632 ON), D15 natural critical-context band differences (949 / 1,336), the single OFF gathering critical-exit band difference, and D16 no-place mismatches (1,769 / 3,290). Every raw category remains individually reviewed; maps are summed per key rather than overwritten across worlds.

D16 `no_choice → routine_no_place` discrepancies remain in the full port denominator as behaviour and band mismatches. The prior D17 Idle+Job forage/drink defect categories are absent: zero residual port defects and zero unexplained categories. There is no blanket routine or gather exclusion.

At the shadow audit, the separate 48-day full-runtime candidate on calibration seeds 0–7 was pending. It has now completed, as independently audited below; the completed round-3 candidate remains historical evidence, not a substitute. No held-out seeds were accessed by this audit, and the independent auditor launched no simulations or builds. The production switch, held-out evaluation, paired performance, story-yield comparison, final fixture and full gate remain separate acceptance steps.


## Round-4 full-runtime calibration candidate — PASS; policy frozen

The corrected-source `mind_candidate` run completed at `2026-10-26T16:33:50Z`, exit 0, with DETERMINISM PASS. Independent verification checks exact seeds 0–7 × 48 days, steward/defence ON, story/neighbours ON and no player orders. Recomputed calibration outcomes are identical to the recorded outcome artifact: all seven bars pass. Six rows are 8/8; cold-snap firewood is 5/5 exposed, required 4, with three unexposed worlds counting as neither success nor failure.

Actual reason coverage is **135,368/135,368** first choices plus behaviour switches, **141,853/141,853** node/restart choices, and **2,984/2,984** stricter critical choices. All **277,221/277,221** expected records are covered, with zero failures over **552,960** observed mind ticks. These are calibration results, not the separate held-out coverage requirement.

- Independent audit: `/Users/rexxa/work/core-data/shadow-round-4/candidate/why-calibration-audit.json`; SHA-256 `5f790a727f481248bea9ef834a013c1285e00c4ed67228442b4271b971909370`. Markdown companion retains the coverage summary; JSON includes every world, verified references and outcome rows.
- Outcome SHA-256 `59030b9dade34ce2af8fe857dfee1bcffe7c56f7b992c410f1173115946d642e`; summary SHA-256 `70cccdc7e6a00703cba8ea04ef9210e6d13de6e1d94ec5c95c10bea21dc7e68b`; parent-receipt SHA-256 `6e70a132f711ba8a629ac6ed817a4b5041098a63b4e42df66369371edfb95606`.
- Candidate executable SHA-256 `13ac2e8f306ed2fab68e466336408c249530bf234da7587dfb8f47f4b9fea1ed`; frozen Rust archive SHA-256 `029f058369396efc1f33fa2577706b8a664fd7e51905c5349de965b2c8cce1d2`. The audit hashes the actual executable and verifies all **518** frozen source files inside the archive against the freeze manifest. Build causality remains parent attestation; no compilation was independently reproduced.

Policy freeze: `/Users/rexxa/work/core-data/policy-freeze-round-4.json`, timestamp `2026-10-26T16:35:29Z`, SHA-256 `8306d91c24ee200f2fb308c332ecbbb93f6a90c14c18b5ca2a9e89b22a97675d`. It binds the completed parent receipt, round-4 shadow audit, outcomes, independent why audit, freeze manifest and Rust archive. Policy bytes remain `7a6c90ac89db9cfb08acac6a8e14c91d1a1690d9fa40c3798c52323ead43eae8`; D18 changes only the documented mind gather RNG provenance. Identical round-3/4 gameplay summaries do not prove that rare branch was exposed; its actual-effect regression remains the direct mechanism evidence.

All four calibration rounds and prior freeze receipts remain preserved. No held-out seeds were accessed in this audit, and **no policy tuning on held-out seeds 1232–1263 is permitted**. The calibration budget is exhausted. Production-switch identity, held-out outcomes/why, paired performance, story-yield, native UI and the final gate remain separate acceptance steps.
