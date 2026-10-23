# ART-POLISH2 sprite pipeline independent code review

Review state: **NO OPEN CODE FINDINGS after independently verified corrections**. This is a code review of the active working tree based on `a7a71aaab3a2fda5af4bc2206b48ff1f1109e8b9`; nonidle source repairs and final rebaking are still in progress. No source image, production asset, or implementation file was modified by this reviewer. This status does not declare the asset pass complete.

Scope: `POLISH2_GOAL.md` §3.2, `SPRITE_VARIANTS_IMPLEMENTATION.md`, all `art/pipeline_polish2/sprite_*.py`, `art/compositor/face_variants.py`, and their two focused test modules. Graft file lookup preceded source inspection. The review concerns preservation, the common ten-frame Lab transform, source/freeze/review provenance, deterministic selection, and truthful numeric-versus-semantic evidence. It does not substitute for the independent visual reviewers or owner acceptance.

## Findings corrected during review

### HIGH, RESOLVED — integration accepted an absent freeze as an unprotected corpus

Original locations: `art/pipeline_polish2/sprite_integrate.py:44`, `:100`, `:110`; `art/pipeline_polish2/sprite_freeze.py:47`. Corrected validation: `art/pipeline_polish2/sprite_integrate.py:43` and `:79`.

`FreezeGuard.read()` deliberately returns an empty guard when `variant_reference_freeze.json` is absent. The new final integration entrypoint accepts that guard without requiring the 34 frozen looks and their 204 idle references. Its only checks for immutable candidate equality and skipped writes depend on membership in `guard.hashes`; with an empty guard every S/idle mask and bake becomes writable. The final message still claims that all frozen references remained unchanged.

Required correction: reject a missing or incomplete final freeze before any mutation. Bind the expected accepted-look inventory, original S/idle sources, both masks, six variant references, and frozen transform record. Preserve `FreezeGuard` bootstrap behavior elsewhere only if it is still needed. An absent/incomplete freeze regression should assert rejection before asset writes.

This is a missing-precondition finding, **not evidence of current reference drift**: the current live guard validates 309 protected files.

Resolution verified: the integration now reads the freeze file unconditionally and validates all 34 look keys, six exact variant paths per look, exact original/skin/hair paths, and protection of every expected image and the transform record before any writes. Focused regressions reject an empty inventory and an empty protection map. An additional independent in-memory mutation of a skin-mask path to the hair-mask path is rejected with `Frozen source/mask paths disagree with their look`.

### MEDIUM, RESOLVED — review files were recorded without binding to the installed diagnostic

Original location: `art/pipeline_polish2/sprite_integrate.py:58`. Corrected validation: `art/pipeline_polish2/sprite_integrate.py:63` and `:101`.

The `--review` loop accepts any readable file and records only its path and SHA256. It does not inspect a diagnostic identifier, summary digest, or reviewed frame/mask/reference digests. Therefore an earlier review of different bytes, including the initial `MASK40_SEMANTIC_REVIEW.json`, satisfies the same argument contract as a review of the candidate being installed. Candidate-to-summary hashes are checked later, but no edge connects those candidate bytes to the review artifact.

Required correction: validate the review's exact diagnostic/version/hash binding before mutation, and retain the original verdict/status in the evidence chain. A mismatched earlier review must be rejected. **Do not require semantic PASS**: an honestly failed or uncertain reviewed candidate can still be installed for requested boards/reporting under the leader's authorization; this check proves provenance and does not waive or rewrite any gate.

Both findings were reported directly to the sprite implementation agent and leader. The leader confirmed that exact binding and a mandatory final freeze are required, while preserving FAIL/UNCERTAIN publication evidence.

Resolution verified: `validate_review()` requires the exact candidate directory and diagnostic summary SHA256. The new test accepts a correctly bound `FAIL` verdict, rejects changed summary bytes, and rejects another candidate directory. The review artifact itself remains hash-bound in the installed manifest; verdicts are not rewritten.

Reviewed corrected SHA256: `sprite_integrate.py` = `73f06c85d8bcf35b6adc72758879f6d0fe2d1766372f63da9b939f8d24a50ed3`; `test_sprite_integrate.py` = `6c14b916ad8074243431bbc686539a09f9bf3e592a01df8d6d9bf387db290777`.

## Verified behavior and retained limitations

- `python3 -m pytest art/compositor/test_face_variants.py art/pipeline_polish2/test_sprite_variants.py -q`: **15 passed, 1 failed** at `test_sprite_variants.py:102`. The failure is the acknowledged intermediate state after final calibrated S/idle installation: all **204/204 S/idle** bakes reproduce from the live transforms, while **0/1836 nonidle** bakes reproduce until rebaking. First mismatch: `adult_F_fur_cloak_bun/S/walk_a/v1`, 210 pixels. This is not a final green-suite claim or a new unexplained defect.
- After the integration fixes, `python3 -m pytest art/pipeline_polish2/test_sprite_integrate.py art/compositor/test_face_variants.py art/pipeline_polish2/test_sprite_variants.py -q -k 'not every_baked_frame'`: **17 passed, 1 explicitly deselected**. The known transitional all-bake reproduction failure was not reclassified as a pass. `python3 -m art.pipeline_polish2.sprite_build --help` succeeds through the new entrypoint.
- A separate full-corpus read-only loop checked **2040/2040 current bakes** for dimensions, alpha, all unmasked bytes, outline bytes, and disjoint/protected masks: **zero violations**. Thus the pending common-transform rebake is distinguishable from geometric/alpha/outline corruption.
- Fresh detection using the current source envelopes and refinement flags reproduces all **34/34 frozen S/idle skin/hair mask pairs** exactly. Fresh literal gates remain **29/34 face**, **27/34 both forearms**, and **27/34 source-hair proxy**. The last value is explicitly not true-hair formal acceptance.
- The focused suite verifies original Route I files against accepted per-look manifests, Lab round-trip/selective RGB changes, deterministic masks, outline exclusion, child/adult selection and fallback rules, all 340 source-envelope keys, explicit garment/prop priority, ear/beard priority, and the known forehead-versus-fur regression. Alpha/unmasked preservation and common-transform reproduction are separately asserted.
- `shifted()` writes only masked RGB. Source-derived outline exclusion is `L <= 18` or `alpha < 128`; the new integration checks both mask disjointness and protected selection before comparing all candidate pixels to deterministic reproduction with the frozen look/variant deltas.
- The replacement integration checks the exact 340 frame-key set, current plan and transform snapshot hashes, source-report hashes, source PNG hashes, mask/bake hashes, and frozen candidate equality before copying. It does not recompute anchors.
- Literal face/forearm denominators are complete recorded boxes; the immutable-pixel ceiling is a separate diagnostic. The classifier hair metric is explicitly diagnostic; the source-hair ratio remains `AWAITING_SEMANTIC_AUDIT`. No code path inspected promotes those counts into a verified true-hair gate.

## Resolved during review

The previous `sprite_build.main()` bypassed source envelopes and refinement flags. A read-only reproduction for `adult_F_fur_cloak_long_tied/S/carry` selected **11 pixels inside the declared prop exclusion**, compared with **0** through the envelope-aware detector. The implementation agent replaced that mutation entrypoint with delegation to `sprite_integrate.main()`, eliminating this runnable legacy regeneration path. The historical result remains here as evidence; it is not an additional open finding.

## Completion boundary

The new installer has not been invoked by this reviewer. Full nonidle repair/review/rebake and a fresh all-2040 reproduction pass remain pending at this report revision; the two code findings are resolved. No lint/type-check, game/runtime/performance, portrait identity, true-hair semantic gate, or owner adoption claim is inferred from this bounded review. The leader owns final pass disposition and publication. The reviewer changed only this report and made no production simplifications.
