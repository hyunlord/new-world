# Sprite variants: implementation and evidence

Status: all 680 masks, 2040 baked frames and 204 frozen transforms are installed and mechanically verified. Semantic reviews are complete with retained uncertainties; the literal skin-coverage gates fail and the formal hair-silhouette gate remains NOT_VERIFIABLE. This record distinguishes mechanical preservation, source annotation review, semantic mask review, and literal numeric gates. No owner adoption is claimed.

## Frozen plan and data contract

`art/character/FACE_VARIANTS.json` and `FACE_VARIANTS.md` were fixed before portrait generation. The accepted Route I inventory is **34 unique looks: 16 adults and 18 children**, producing 204 unique variants. The 36 logical look combinations resolve two adult male long-tied looks to their accepted bun fallbacks. This is the inventory discrepancy recorded as D3; the goal's prose count is not silently substituted for actual source files.

Each accepted look has six variants. Adults cover all three skin targets, four hair targets, two older variants including H4, and all three beard types for men; children cover three skin and three hair targets without grey hair or beards. The target RGB entries were added to `palette_05.json` without replacing its existing judging palette.

`art/compositor/face_variants.py` validates this plan and implements the documented unsigned 32-bit multiplicative person hash. Adults aged 50 or more select among their own older variants; child looks retain their child variant pool. `FacePlan.read()`, `for_look()`, `choose_variant()` and `baked_frame()` are the shared compositor/portrait contract. The default original frames remain independently available.

Production asset paths:

- Masks: `art/character/variant_masks/<look>/<S|E>/<pose>_<skin|hair>.png`.
- Bakes: `art/character/route_i/<look>/<v1..v6>/<S|E>/<pose>.png`.
- Transform records: `art/character/variant_transforms.json`; one skin and one hair Lab delta per look/variant, reused across all ten frames.
- Source geometry: `art/character/variant_anatomy.json` plus four source-envelope reports under this directory.

## Detection and preservation

The detector uses eight fixed semantic CIELAB seeds and eight bounded Lloyd updates. Spatial supports come from template/body geometry and source-only anatomy annotations. Source-authored garment and prop polygons take precedence over both classifiers; explicit face/neck skin supports take precedence over brown hair labels. Actual adult beard/sideburn supports are separate from scalp/back-fall supports. Explicit ear/cheek holes override beard exclusion as well as hair selection. Local skin-shadow recovery is spatially constrained; class 5 highlight recovery additionally requires the independent full face box and source-declared skin support.

The immutable outline policy is L≤18 or alpha<128. Only selected RGB bytes are transformed. Alpha, protected outline, all unmasked pixels, frame dimensions and original source files are preserved. Hair coverage uses a source-defined reference including protected ink; the old label2 intersection remains a separately named classifier diagnostic, not a formal silhouette gate.

`sprite_sources.py` validates all 340 actual look/facing/pose keys, rejects duplicates or unexpected keys, validates original source hashes, and consumes `garment_exclusions`, `prop_exclusions`, `hair_envelopes`, `facial_hair_envelopes`, `hair_exclusions` and `skin_supports`. The inventory is 34 S/idle + 144 adult nonidle + 162 child nonidle. Every new diagnostic snapshots exact source-report bytes before processing.

## Evidence revisions and retained failures

Raw evidence is external at `/Users/rexxa/work/artpolish2-data/masks/`.

| Evidence | Result and limitation |
|---|---|
| Initial fixed geometry | 9/34 face boxes, 0/34 both forearms; 203/340 classifier hair proxy passes. Preserved in `initial_manifest.json`. |
| Independent source anatomy review |102 regions: 69 PASS, 30 FAIL, 3 UNCERTAIN; all 33 proposed corrections applied without shrinking boxes to chase thresholds. |
| Initial fixed 40 semantic review | 8 PASS, 27 FAIL, 5 UNCERTAIN; zero true-silhouette gates verified. Raw 80 masks and 40 panels preserved in `semantic_initial_snapshot/`. |
| Initial non-owner 26 | 20 contamination FAIL, 5 PASS, 1 UNCERTAIN; all 26 had obvious skin omissions. |
| Initial frozen owner 8 |Contamination clearances at exact hashes only; incomplete skin and literal coverage failures retained. Full original owner assets archived in `frozen_owner8_v1/`. |
| Diagnostic v2 |Substantial anatomy recovery; exact owner input bytes recovered and hash-matched. Exact consumed non-owner input serialization could not be recovered: v2 is explicitly **NOT_FULLY_REPLAYABLE**. |
| Diagnostic v3/v4 |Exact inputs preserved. Neck/jaw shadows selected as hair, then source face/neck priority and explicit beard support corrected the classifier boundary. V4 still omitted some actual foreheads and inner arms. |
| Diagnostic v5 |Actual forehead/ear/cheek recovery succeeded, but broad class 5 eligibility introduced six confirmed fur/hem pixels in owner masks. Failed revision retained. |
| Diagnostic v6 |All34 S/idle masks independently clear for contamination and obvious completeness. Literal face29/34 and both-forearms27/34 remain below the required all-look threshold. Semantic clearance does not waive those failures. |
| `frames_diagnostic_v1` |All 340 frames and 2040 diagnostic tints preserve protected bytes. Fresh independent review found nonidle carry-log spill and side-view arm omission; these initial failures are retained. |
| `frames_diagnostic_v2` |All original contamination witnesses cleared. Fixed 40: 31 contamination PASS and 9 UNCERTAIN; formal hair 40 NOT_VERIFIABLE (6 invalid references, 34 uncertain). Corpus: 28 looks PASS, 2 FAIL for newly exposed arm omissions, 4 UNCERTAIN. All 272 frozen idle masks/bakes matched exactly. V2 remains immutable, including its known stale display caption. |
| `frames_diagnostic_v3` |Final integrated revision. Only the two remaining arm masks and their 12 bakes changed. Independent corpus review found zero confirmed remaining failures and retained four uncertainties. Fixed 40 retains 31 contamination PASS, 9 UNCERTAIN and 40 formal-hair NOT_VERIFIABLE. All 2040 baked frames reproduce with frozen transforms and preserve protected bytes. |

Owner v6 review: `MASK_OWNER_V6_REVIEW.json`. Non-owner review history and v6 hash binding: `IDLE_MASK_REVIEW.json`. Fixed40 and full-corpus fresh reviewers own their separate verdict records. Their reports, not this implementation's class labels or numeric counts, determine semantic findings.

`frames_diagnostic_v1/audit40.json` names exactly 40 frames: the first accepted adult female, adult male, child female and child male look, each across both facings and all five poses. Every entry records original/report/envelope/mask/reference/panel hashes. Per-frame `audit.png` shows actual source, selected masks, independent hair reference and a diagnostic tint; per-look `contact.png` shows all ten frames.

## Anchor calibration

Diagnostic revisions held the previous Lab transforms fixed to isolate mask changes. `calibration_v1` compares those old anchors against means measured from the independently reviewed v6 S/idle masks. Across 204 variants, average mean-target ΔE76 changes from 2.229 to 0.265 for skin and 1.724 to 0.062 for hair. Skin maximum changes from 9.875 to 3.362. Out-of-gamut selected-pixel counts summed over 204 variants increase from 1426 to 2738 for skin and 28 to 85 for hair; gamut clipping is reported, not hidden.

All 34 native and 2× six-variant old/new sheets are stored with exact source/mask hashes and per-variant mean errors/clipping counts. `release_v1` stages a complete 680-mask/2040-bake package externally from those corrected anchors; it is not permission to install the package. The leader authorized exact v6 masks and calibrated anchors. `variant_reference_freeze.json` SHA256 `9cd4f807a1b42b1b021e143e85b357b1b3e058c6c2cf037b55b30b068d44d89a` now protects all 34 sources, 68 idle masks, 204 idle bakes, the transform record and both final idle source-envelope reports. The complete previous 680 masks and 2040 bakes are preserved under `final_reference_v1/previous/`, with `previous_files.json` hashes. Existing 30 portrait try1 input snapshots were preserved by the portrait lane.

## Verification and replay

Implementation file groups:

- Plan/selector: `sprite_plan.py`, `art/compositor/face_variants.py` and its tests.
- Source geometry/segmentation: `sprite_annotations.py`, `sprite_landmarks.py`, `sprite_regions.py`, `sprite_sources.py`, `sprite_masks.py`.
- Colour and installation: `sprite_colour.py`, `sprite_records.py`, `sprite_freeze.py`, `sprite_reference_apply.py`, `sprite_integrate.py`, `sprite_build.py`.
- Reproducible evidence: `sprite_source_panels.py`, `sprite_idle_preview.py`, `sprite_audit.py`, `sprite_diagnostic.py`, `sprite_frame_diagnostic.py`, `sprite_calibration.py`, `sprite_release.py`.
- Regressions: `test_sprite_variants.py`, `test_sprite_integrate.py`. Sprite-prefixed modules are under `art/pipeline_polish2/`.

The legacy builder's independent mask/anchor recomputation was removed. Shared typed records now live in one module, eliminating import cycles between evidence and installation code. Originals, IDs, source frame geometry and frozen final references are preserved.

Meaningful regressions cover plan axes and fallbacks, elder selection, deterministic masks, protected outline, all original-manifest hashes, all 2040 live baked-frame reproduction/preservation, freeze drift rejection, known neck-shadow rejection, ear/beard precedence, all 340 source imports, and actual wood whose Lab colours otherwise select 39 prop pixels. A regression preserves the recovered forehead highlight while rejecting all six observed class 5 fur/hem failures.

Commands used:

```sh
python3 -m pytest art/compositor/test_face_variants.py art/pipeline_polish2/test_sprite_variants.py -q
uvx ruff check art/pipeline_polish2/sprite_*.py art/pipeline_polish2/test_sprite_variants.py art/compositor/face_variants.py art/compositor/test_face_variants.py
uvx basedpyright --pythonpath /opt/homebrew/bin/python3 art/pipeline_polish2/sprite_*.py art/pipeline_polish2/test_sprite_variants.py art/compositor/face_variants.py art/compositor/test_face_variants.py
python3 -m art.pipeline_polish2.sprite_diagnostic --revision diagnostic_v6
python3 -m art.pipeline_polish2.sprite_frame_diagnostic --revision frames_diagnostic_v1
python3 -m art.pipeline_polish2.sprite_calibration --diagnostic diagnostic_v6 --revision calibration_v1
python3 -m art.pipeline_polish2.sprite_release --revision release_v1
```

Every revision directory is immutable; a replay must choose a new name. Replaying old diagnostics requires their snapshotted code and report inputs, not whichever live report revision now exists. After final v3 integration, all 19 tests pass, including reproduction of all 2040 baked frames. Ruff passes and basedpyright reports zero errors and zero warnings. The earlier S/idle-only transition and its 1836 stale nonidle bakes ended; those temporary failed checks remain in the code-review history. Lint and strict type verification are run on this lane's files; unresolved warnings are not classified green. No game/runtime performance or owner visual adoption is inferred from these offline checks.

## Remaining limitations and handoff

Sprite implementation and final verification are complete. No confirmed corpus defects remain after the bounded repairs, but four corpus uncertainties and nine fixed 40 contamination uncertainties remain visible in the independent reports. The literal 60% face/forearm and 50% actual-hair gates retain their original thresholds and failed values. Portrait generation/blind testing, boards, wall/material work and publication belong to other lanes.

## Final integration preconditions

The legacy detector-based mutation entrypoint was removed. `sprite_build.py` now delegates to `sprite_integrate.py`, which requires exactly 340 unique frame keys, a complete 34-look/204-reference freeze, unchanged protected transforms, current source-report hashes, and independent review files bound to the exact diagnostic directory and summary hash. Failed or uncertain review verdicts remain intact; provenance validation does not manufacture semantic acceptance. Every candidate mask/bake is hash-checked and reproduced before any write. Frozen source/mask/reference bytes are skipped and verified unchanged. Successful completion requires all installed files to match after writing.

The immutable `frames_diagnostic_v2` introduced the source authors' nonidle repairs; v3 repaired the final two observed arm omissions. Its hair reference is an independent original-colour candidate with explicit Lab L≤52, R−G<55 and alpha≥16 parameters, constrained by source scalp/backfall/beard geometry. This reference includes visible protected ink without reading selected masks or Lloyd labels, but still requires independent completeness/face-exclusion review and is not ground truth.

## Final installation receipt

Exact-bound reviews: `MASK40_FINAL_V3_REVIEW.json` SHA256 `cbe31f642f1f5f63a3247c98a5a02cbeccec8506ca5b583fea77e2f5c396514b`; `MASK_CORPUS_V3_REVIEW.json` SHA256 `f9f95f95079bd8d6a7b9541f65f59dfb0f035eaf0a411948bd1dd44a85097690`.

```sh
python3 -m art.pipeline_polish2.sprite_build --diagnostic frames_diagnostic_v3 --review docs/v8/artpolish2/MASK40_FINAL_V3_REVIEW.json --review docs/v8/artpolish2/MASK_CORPUS_V3_REVIEW.json
```

Final raw checks are in `/Users/rexxa/work/artpolish2-data/sprite-final-verification/`: `checks.json`, `assets.json`, `pytest.txt`, `ruff.txt`, `basedpyright.txt`. All 680 accepted original PNG/JSON files match their original manifests. All 340 hair masks satisfy the recorded head-plus-four containment check. The independent-colour hair reference numerically reaches 50% in 251/340 frames, explicitly diagnostic only; it is not formal silhouette acceptance.

| Installed record | SHA256 | Bytes |
|---|---|---:|
| `variant_masks/MANIFEST.json` | `0339da11489774f64f49a0714df424c481a3a30d4084440361cddcdb6df72694` |1207330|
| `variant_transforms.json` | `3e7344a5a368581f7384be029ca85f61ca30245620a4c2e21e02d64d0546cc33` |188138|
| `variant_reference_freeze.json` | `9cd4f807a1b42b1b021e143e85b357b1b3e058c6c2cf037b55b30b068d44d89a` |68641|

No source sprite geometry or original pixels were changed. No commits were made by this lane.
