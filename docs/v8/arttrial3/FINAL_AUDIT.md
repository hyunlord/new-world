# Final bounded audit — structure, production provenance and costs

2026-10-22. Read-only audit of the frozen route-A assets, checker, generation/derivation processing and cost arithmetic. Only this document was written. Character-lab code/runtime, which was being edited concurrently, was deliberately excluded. This is **not a full ART-TRIAL3 completion gate**.

## Result and required retained limitations

No additional pixel-processing or arithmetic defect was found within this bounded audit. The actual route remains **NOT_COMPLETE**, not viable or fully accepted: `fur_cloak_S` is absent in adult/child/elder after the three-generation cap. The checker correctly reports `passed:false`, `subset_passed:true`, `status:NOT_COMPLETE`. The only failed result is `minimum_set`, with those three missing-variant findings. An accepted-layer statistic of87/87 must not replace complete-inventory acceptance.

Sleep coverage is three bare bodies, with no tested top/hair/carry attributes during sleep. Head geometry and hand-mask declarations remain structural evidence rather than semantic anatomy recognition. Mask provenance hashes do not prove correct anatomical region annotation. GIF readability, live runtime/FPS and final report delivery were outside this audit and cannot be inferred from it.

## Current-state checks independently rerun

- Manifest87 layers:29 generated adults and58 zero-call child/elder derivatives.
- Actual `check_layer` calls: **87/87 pass**. Actual `fixed_upper` calls: all pass.
- Recomputed exhaustive combinations: **603/603 pass**, comprising600 upright Cartesian products and3 `bare_sleep_only` bodies. Per age: S40 + E80 + N80 + sleep1 =201. The missing S fur top is not invented or silently substituted.
- Every `CHECKS.json` input hash matches current manifest, sidecar, image and mask bytes; zero absent/mismatched inputs.
- Independently called provenance audit for all87 sidecars: **87/87 without metadata/hash findings**. This covers current accepted files, not a claim about semantic mask quality.
- `python3 -m pytest -q art/pipeline3/test_structure.py`: **22 passed in1.23s**, including the diagnostic-subset acceptance regression and sleep geometry/coverage tests.

## Pixel replay, not merely operation-log inspection

All29 generated adult RGBA images were independently reconstructed in memory from the retained raw PNGs: magenta key, full-canvas Lanczos, alpha<16 cleanup, optional declared exact torso mask-box extraction, declared nearopaque top-alpha cleanup, and exact direction-idle head/torso copyback on animation frames. The reconstructed pixels were compared against the current accepted PNGs. **29/29 are pixel-identical; zero differing pixels.** No body subtraction, unlogged RGB repaint, contour fitting or hidden alternate artwork was required to reproduce them.

All58 derived RGBA images were likewise reconstructed in memory from their recorded adult sources: documented age band transforms, direction-aware elder head shift, exact top-torso mapping, documented sleep scale/translation, alpha cleanup, and derived-idle head/torso copyback. **58/58 are pixel-identical; zero differing pixels.** The derivation module's band-resize helper was reused; the transform orchestration and final comparison were independently executed without calling its file-writing production entrypoint. This proves agreement with the inspected recipe, not independent visual naturalness of the age transforms.

Masks were hash-audited and structurally checked, and the code implementing categorical nearest-label completion was inspected. Mask polygons were not independently redrawn against anatomy in this audit. Source reference and prompt hashes were checked by the provenance audit. Top rectangle extraction is explicitly logged and permitted by goal§3.4; whether the resulting rectangles look natural belongs to visual assessment.

## Attempt ceilings and cost arithmetic

The external raw directory contains **52** `*/try*/raw.png` records across30 requested adult layers. The maximum per-layer record count is**3**, every try suffix is≤3, and the cost ledger lists the same52 records. Accepted originals are29; derived files do not increase that denominator. Thus actual submissions/accepted generated layer is**52/29 =1.793103448275862**, not selected-attempt indices or52/87. The capped S fur failure and unused submitted candidates remain in the denominator.29 accepted results out of52 submitted calls is55.77%, above the route-level50% stop threshold even under this conservative per-call denominator; the top family alone is not a separate route.

All52 calls have recorded timing fields. Independently summed elapsed fields equal**2252.416s**; cost documentation separately reports direct tool-call intervals**1922.523s**. These are overlapping recorded intervals, not elapsed production time or labor. Recorded cost range per accepted original is1.105–1.294min. Credits remain `UNKNOWN_NOT_EXPOSED`; labor remains `NOT_MEASURED`.

Projection arithmetic is correct:4×3×4×6×2×3×6 =**10,368 compositions**. The stated sharing scenarios give136 and196 unique layers and measured-interval projections**150.266–253.720min**. They are hypothetical reuse scenarios with different item-direction assumptions, not a measured production delivery bound. Missing route completeness and unmeasured review/integration/labor costs remain explicit exclusions.

## Size snapshot and audit boundary

At inspection, `art/` size excluding local `.godot` state was79,805,886bytes against77,993,992bytes in the pinned base tree: approximately**1.81MB growth**, well below40MB. The largest PNG in `route_a` was5,199bytes, below1.5MB. No GIF existed at the instant of this size snapshot; the concurrently running lab lane must verify its eventual GIF≤2MB and final total-growth caps. This snapshot is not final runtime-artifact verification.

No commits, generation calls, asset changes or checker edits were made by this audit.

## Follow-up: frozen lab and recorded evidence audit

Read-only inspection of `art/proto/CharacterLab.gd`, `capture_lab.py`, foreground crowd records, final-v2 GIFs, blind review results, style results and verdict. No Godot process was launched by this audit. The full final-v3 grid sweep was still running and is not certified here.

### Crowd measurement

`lab-foreground/crowd/runtime.json` records **4,734 frame_post_draw callbacks /60.009588s =78.8873937944716FPS**, independently recomputed. This is an actual-render callback count, not the process-loop count. All61 visibility samples report `can_draw:true`; hidden_process_frames=0. None of61 samples reports keyboard focus. Therefore the evidence supports a drawable always-on-top window, not a claim of continuously focused input. Frame-time intervals sum to60.004054s, consistent with the wall measurement plus completion overhead.

The runtime records unchanged source during the run. Its script SHA256 `95e5caf6b0aeaf6902cd62d7b9b3bb0f350da69c25827afe527e58aa2557a758` matches the current inspected `CharacterLab.gd`. Every runtime asset hash also matches current bytes. Viewport and requested viewport both equal1312×1088. Code draws exactly200 upright actors, each switched to walk_a/walk_b, and conservative full64×96 bounds plus ±3px motion fit all200 inside the viewport. The population-bounds record agrees; the capture was visually inspected and shows the complete20×10 grid with camp props above it.

`p1_eligible:false`, dataset NOT_COMPLETE, and VERDICT's `INELIGIBLE_INCOMPLETE_SET` correctly separate the observed numeric FPS bar from complete-route P1 acceptance. This is a standalone render harness; no simulation performance claim follows. `CROWD_A.json` is a compact summary omitting raw frame_times_us, not a byte-identical copy of the external raw runtime record.

### GIF encoding, review binding and score arithmetic

All24 final-v2 GIFs match both their private answer-key SHA256 and corresponding anonymized blind-review GIF bytes. Every file has a1000ms loop. Moving sequences contain8 frames with alternating120/130ms durations, averaging8fps; identical sleep frames are losslessly coalesced into a single1000ms frame. Largest GIF is260,157bytes, below2MB. Frame capture is offline phase sampling; it is not claimed to have run live at8fps during capture.

Both reviewers explicitly record that actual timed playback was unavailable. Recounting their individual answers reproduces reviewer1's9/12 activities and32/36 attributes at each zoom, and reviewer2's10/12 and34/36 at each zoom. Reviewer1 misses the2× activity threshold. `READABILITY.md`, `READABILITY_RESULTS.json`, and VERDICT correctly keep R1 **NOT_VERIFIED**, despite some proxy thresholds passing. Bare sleep answers score actual none/none/bald; they provide no sleeping wardrobe differentiation evidence.

The six style score tuples independently average **2.5416666666666665/5**. Numeric judge0/6 is reported separately. The verdict does not call either route viable and applies the artist-commission recommendation without claiming an owner decision.

### Specific provenance follow-up still pending at this snapshot

Final-v2 GIF runtime records identify script SHA256 `0322aeaa1c6a5c060478cb453a6b6b1092ae03cd6a7d16bd141243b40cbdbe7a`, different from the final foreground/grid source. No exact source archive under arttrial3-data or committed CharacterLab history existed at inspection. The parent has identified this gap and is running a final-source GIF recapture after the grid sweep, then comparing all24 resulting SHA256 values to the already blind-reviewed files. This audit does **not** pre-certify that pending rerun or claim old GIFs came from the current source. Closure requires the new runtime source hashes and byte-comparison results; if GIF bytes differ, earlier blind responses do not automatically bind to the replacements.

At this follow-up snapshot `art/` growth excluding local `.godot` state was7,551,860bytes, below40MB. The final exhaustive grid artifacts were not yet all copied, so the final delivery size cap still needs its final snapshot.

## Final-source provenance and page-sweep closure

The previously pending final-source GIF provenance gap is now **resolved for the delivered GIF bytes**. Independently compared all24 files under `gifs-final-source/gifs` to final-v2: **24/24 byte-identical**, and those prior files already matched the blind-review copies. All24 new capture runtime records identify unchanged script SHA256 `95e5caf6b0aeaf6902cd62d7b9b3bb0f350da69c25827afe527e58aa2557a758`, matching the current source. This binds the exact reviewed GIF bytes to a verified final-source recapture; it does not retroactively fabricate the missing old0322 source archive or upgrade proxy readability to timed-playback acceptance.

`source-final/manifest.json` archives the final CharacterLab.gd, character_lab.tscn, project.godot and capture_lab.py. Independently verified every recorded SHA256 against **both current source and archived copy**, all matching.

The completed `lab-final-v3/boards/pages.json` contains **78 pages**:8 pages at1× and31 at2× for each of meadow and forest_floor. For each of the four ground/zoom conditions, concatenated recorded indices equal exactly0..602, with603 unique entries and no omissions or duplicates. Every board's bytes and SHA256 match its page record. All78 runtime records agree with the page indices/draw counts, have matching requested/actual viewport sizes, positive actual rendered-frame counts, non-headless rendering, unchanged matching final script source and explicit NOT_COMPLETE dataset status. `docs/v8/arttrial3/LAB_PAGES.json` is byte-identical to the external pages ledger.

The full external page set totals **43,259,260bytes**; it is deliberately outside the repository. Its largest page is929,500bytes. Current repository representative boards and GIFs remain within the individual limits: no PNG over1,500,000bytes or GIF over2,000,000bytes under art/character or art/proto. The repository GIF copies found match the final-source recaptures. Final inspected art-tree growth excluding local `.godot` state is **8,978,340bytes**, below40MB. These are current-state size measurements, not a promise that later files cannot increase them.

No new acceptance promotion follows: complete route inventory remains missing S fur cloak; R1 remains NOT_VERIFIED timed playback; the measured crowd numeric bar remains separate from full eligible P1. No Godot process or code/assets were changed by this closure audit; only this document was appended.
