# Part 4 tooling

The catalog maps 80 physical fill cells to existing target slots. It adds nothing to `production_sheets.SHEETS`.

Commands from the repository root:

```sh
python3 tools/art_pipeline/key_plan_p5.py
python3 tools/art_pipeline/key_references_p5.py docs/art/production5/masks/references.json
python3 tools/art_pipeline/key_references_p5.py approve docs/art/production5/references/manifest.json REVIEW_JSON
python3 tools/art_pipeline/key_candidates_p5.py V_a6 /Users/rexxa/work/art-production5-raw/V_a6/attempt_1.png 1 docs/art/production5/references/approved_manifest.json
```

A fresh reference review has `verdicts` rows with `body`, `verdict`, `key_sha256`, `natural_rgba_sha256`, and `observation`. The approval command binds that review to both images, preserves the pending manifest, and refuses to overwrite an earlier approved manifest. A rejected reference cannot start key conversion. Adult_b references additionally require a Part 3 `template_v5` approved idle mask.

For explicitly non-key adult fallback generations, omit the teal prompt addition and use the natural 8x identity image; pass `--natural` to the converter. Those candidates use the approved positional template masks and remain pending independent review. A template alignment failure becomes `template_hair_unavailable`. Child sheets need no key or hair mask.

The converter retains raw crops outside git beside the raw sheet. It preserves the initial fill conversion ledger, re-fits against the extended palette, protects original key texels during despeckle, and freezes both pre-substitution and final delta E. Ranking uses only pre-substitution delta E. The natural sprite and exact native/outlined masks are separate frozen artifacts. No script in this lane installs or approves candidate frames.

Verification performed before generation: the three executable regression scenarios in `test_key_hair_p5.py` passed; compileall passed for the six modules; the programming no-excuse audit found no violations. An isolated temporary 16-cell synthetic child sheet traversed `fill_v4.process`, converter, ledger, and four review boards with zero conversion exceptions. This smoke is tooling evidence only, not generated art, acceptance or image-call evidence. Ruff and basedpyright were not present on PATH.

A second isolated synthetic smoke traversed the actual key path with the approved reference: sixteen records, four boards, sixteen free of key errors, zero conversion exceptions. First-cell pre-substitution delta E was 2.5089009308045664, post-substitution 6.708909509026543; this confirms intentional teal replacement is excluded from ranking. These synthetic idle repetitions do not establish correct animation poses or any art acceptance.

Run `key_targets_p5.py` only after Part 3 to freeze slot-level eligibility. After the first real key sheet review, run `key_checkpoint_p5.py CANDIDATES_JSON REVIEWS_JSON`; it freezes the literal 12/16 key-clean and 8/16 approved stop decision. Candidate runs also emit `review_index.json` for the Part 1 selector's fingerprint binding.

## First real conversion guard correction

The shared `guarded_candidate` began preserving hard errors from a record. The fill stage initially seeds `conversion_failure`; the successful key refit had not cleared that initial guard before evaluation, so the old marker survived. The converter now clears the old guard and conversion error after successful refitting, retaining freshly computed key errors. A fresh isolated 16-cell key conversion verifies no stale `conversion_failure` remains.

`key_reguard_p5.py` creates `candidates_reguarded.json` and `review_index_reguarded.json` without replacing the original ledger or any image. The first real sheet correction records original candidate fingerprints and verifies 112 unchanged image files. It changes technical guards only: two frame passes, eight key-error-free cells. Existing image review remains independent; no verdict is created or altered by the correction.

Adult_b uses the separate `V_b5_natural.txt` / `V_b6_natural.txt` prompts, natural reference image, and `--natural`, because the approved individual idle-mask proposal was not installed after its group animation rejection. The frozen key prompts remain unchanged.
