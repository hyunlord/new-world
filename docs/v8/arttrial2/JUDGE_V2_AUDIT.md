# Independent TRIAL2b numeric-judge audit

Status: **corrections required before final C2/acceptance claims**. This is a
read-only audit of the numeric implementation and its current evidence; no judge
code or thresholds were modified. Snapshot hashes and independently recomputed
results: `/Users/rexxa/work/arttrial2-data/trial2b/judge-independent-audit.json`.
Concurrent work may supersede this snapshot; findings require explicit disposition.

## Findings

1. **Mixed raw/working cohort is mislabeled as all raw.**
   `art/pipeline2/rescore_v2.py:38` selects `_raw.png` only for non-ground assets.
   All six ground paths remain `ground/<name>_try1/candidate.png`, 512×512 working
   derivatives, whereas the originals are 1254×1254. The report labels all 38
   “raw generations” and its limitations claim native raw inputs. This is false
   for six assets and weakens a common-resolution comparison. I independently
   evaluated all six original ground PNGs: all still pass numeric v2. Fixing
   identity need not change the pass count, but the files/hashes and model inputs
   must consistently identify the selected raw images. The other 32 raw hashes
   match their original generation sidecars (`raw_sha256` or `source_sha256`).

2. **Calibration failure is not enforced at the acceptance boundary.**
   `rescore_v2.py:62` derives `calibration_status` only from negative controls,
   ignoring `calibration.leave_one_out`. `judge.py:242` permits acceptance when
   numeric/model scores pass even if a supplied calibration has failed folds.
   The CLI calibration path does return failure for failed folds, but the rescore
   path calls `calibrate()` directly and loses that gate. Current evidence really
   has 12/12 passing folds, so this is a concrete fail-open code path rather than
   a claim that today's folds failed. Require both calibration conditions in the
   producer/acceptance path; test a failed-fold calibration explicitly.

3. **Current calibration prose describes a different implementation.**
   The TRIAL2b section of `JUDGE_CALIBRATION.md` states 10 crops, seven checks,
   19 numeric passes, working-only scoring and failed negative calibration.
   Current code has 12 crops, eight checks including stroke q90, raw non-ground
   scoring, 30 numeric passes and 3/3 person-control rejections. Floors/region-size
   prose also differs. Keep the older result as a clearly dated intermediate
   record and write the actual final formulas, crop revisions and chronology;
   do not silently rewrite the failed intermediate measurement as never having
   occurred. The source comment “frozen before candidate scoring” does not alone
   prove threshold preregistration; preserve dated intermediate outputs.

4. **Existing building model results are not current-input results.**
   `trial2b/model-review/buildings.json` records small working candidates and old
   reference-crop hashes. Current numeric manifest uses revised native crops;
   `raw-inputs.json` prepares different raw candidates and frozen reference files.
   Prepared inputs are not completed model judgments. Do not join old building
   scorecards to new raw-reference identities as if the model saw those images.
   At audit time the committed rescore correctly remains `accepted: 0` and labels
   historical scores diagnostic. Re-run or preserve that non-acceptance state.

5. **Plant C2 includes selected training/reference assets.**
   The oak/birch reference crops come from the same first-attempt assets included
   in the 38-asset cohort. This is explicitly requested planner selection, not
   fabricated provenance, but it is optimistic in-sample evaluation. Full-object
   candidate versus internal crop is not literal byte duplication; still, their
   marks/colors establish their own family's acceptance ranges. Report the
   required cohort without changing its denominator, alongside the sensitivity:
   numeric 30/38 = 78.95%; excluding the two reference assets, 28/36 = 77.78%.
   This is numeric-only, not a combined accepted first-try rate.

## Verified properties and limits

- Recomputed `calibrate()` exactly equals the saved `JUDGE_V2_CALIBRATION.json`:
  12/12 folds pass. Each fold excludes the held crop by name and uses only its
  family. This is genuine leave-one-crop-out, not leave-one-source-out. Person
  upper/full substantially overlap; four ground crops share one image; the two
  props share one image. Their correlation limits generalization claims.
- All 12 source hashes, derivative hashes and exact RGBA crop pixels match the
  manifest. There was no hidden resampling in these recorded crop files.
- All 38 current scored-file hashes match their rescore records. Non-ground raw
  lineage additionally matches the generation metadata. Ground working identity
  also matches its old cohort; the issue is calling that identity raw.
- Foreground alpha/magenta masking, aspect-preserving 256px local measures and
  1024px stroke measures are applied by one common implementation. This avoids
  comparing raw arrays at arbitrary sizes, but does not make all visual content
  interchangeable: a 650px interior tree patch normalized to 1024 and a whole
  canopy normalized to 1024 represent different motif scales. Actual game-size
  stroke/readability remains a separate test.
- Stroke width uses a dark-ridge distance transform near Canny edges, rather than
  distance-transforming the one-pixel Canny output. Wash flatness is variance in
  connected edge-bounded interiors. Global Lab histograms are absent.
- The three existing wrong-style images depict people with substantial natural
  background, whereas the new person anchor is a magenta cutout with a different
  camera/proportion rule. All three fail the person-family numeric range, as
  recorded. The negative test is therefore person-specific and content-confounded;
  it is not independent proof that each environment-family judge rejects wrong
  style. As a diagnostic only, I applied them to all family ranges: painterly and
  photo pass all four environment ranges; woodcut fails all five. These cross-family
  applications are **not valid production labels**, but make broad specificity
  claims unjustified. Do not require unrelated person controls to become valid
  tree controls by arbitrary relabeling or fit thresholds to force those results.
- API scores are validated as four integers 1–5, pass requires all ≥4, and missing
  or legacy model evidence fails closed. `model_family_verified` is a caller-supplied
  assertion rather than a provenance verifier; final assembly must check actual
  candidate/reference/card hashes against completed review records.

## Required disposition

Resolve findings 1–4 in the owning lane, retain finding 5 and the specificity/
correlation limits beside final C2, and re-audit exact final evidence before
promotion. This audit does not certify uncompleted model calls or production
acceptance, and makes no recommendation to loosen thresholds for acceptance.

## Fix disposition (same pass, after owning-agent update)

Findings 1 and 2 are resolved in code: rescore now maps ground to the six original
`ground/<name>_try1.png` files, and calibration status includes all LOO folds.
Certification additionally requires nonempty folds, fold count equal to anchor
count, and all passing. I inspected these paths and ran the current judge tests:
nine tests pass, including rejection of a calibration containing a failed fold.
The independent raw-ground check above already established the unchanged 6/6
numeric result. Findings 3/4 still require final evidence/prose reconciliation;
finding 5 and the correlated/control-specificity limitations remain reporting
requirements, not defects to erase by adjusting thresholds.

## Final evidence disposition

Findings3/4 are resolved: JUDGE_CALIBRATION.md now preserves intermediate chronology and describes the final12crop/eight-measure implementation; ENV_MODEL_REVIEW_V2.json contains38actualraw reviews with exactcurrentfamilyrefs. Independent final audit verified all35promoted PNG/raw/working/calibration/acceptance/modelcard/modelmanifest/reference hash joins. The final first30/38and after35/38are combinedresults. Finding5(training/reference overlap), correlatedLOO andpersoncontrolconfounds remain limitations. Finalcalmground numeric0/6versusmodel6/6, andpersonparts numeric2/40versusmodel40/40 further restrict claims of content-insensitivity. No threshold retuning followed these findings.
