# PROD1 round0 visual QA Pass A

Verdict: diagnostic viewer works; minor inherited control finding remains. Production character certification FAIL/NOT_ELIGIBLE independently of viewer quality.

## Scope and evidence
Read-only review of actual HTML/CSS/JavaScript, prod1_diagnostic/prod1_browser render and capture code, and DESIGN.md. Viewed real Chrome screenshots at375/768/1280 (lab/browser/overview/*.png), inspected specimens01/05 and actual panels. Chrome155.0.8059.85 captured24GIFs ×24samples =576 browser screenshots on target8Hz cadence for3seconds each. No page errors. Timing samples are recorded; maximum screenshot duration0.060seconds. Packet contains48real-playback contact sheets, no answer key. This is expressly DIAGNOSTIC_REJECTED_SET_NOT_CERTIFIED; formal_R1_eligible=false.

## Findings
1. LOW, inherited: surrounding-ground control changes the body class and figure CSS background but produces no visible specimen change. Each figure is entirely covered by an opaque composed GIF and paper caption. The frozen idle specimen figure screenshots before/after forest selection are byte-identical (functional/ground_check.json and ground_before/after.png). Both required meadow and forest scene rows are already present in every GIF, so this does not omit the required ground conditions. Suggested follow-up: expose a visible surround area or remove/clarify this redundant control; do not alter raster art to simulate a change.
2. REQUIRED ART LIMITATION, not a viewer bug:16of120 fixed source frames are missing, explicitly marked MISSING; specimen08 is absent in both facings. Rejected frames remain rejected, and diagnostic labels remain visible. This artifact cannot satisfy the certified12-person lab or formal R1 gate.

## Passed functional checks
- Actual DOM controls and figure/caption components, CSS custom-property palette, spacing and type tokens match DESIGN.md. Raster specimen GIFs are the specifically requested artifact; UI itself is not a screenshot substitute.
- Source-scale selection swaps1x224px and2x448px source widths without interpolation or fit-to-viewport scaling.
- At375/768/1280,12figures load and document scrollWidth equals viewport width. At375and2x,448px source is retained within local horizontal figure scrolling; no page overflow.
- Grid checkbox hides outlines; keyboard Tab advances from zoom to ground; visible focus/hover styles are present and exercised. Manifest HTTP500 is visibly reported as Viewer unavailable, no silent fake fallback.
- Grounds remain continuous behind alpha-composited figures; no black/white alpha rectangles observed. Sleep canvas width112 is preserved; no horizontal sleep clipping observed. Footline alignment and contact shadows are consistent with recorded production geometry.
- All labels/footer are visible. No CJK content exists on this viewer; CJK-glyph QA is N/A.

## Evidence files
- lab/browser/playback.json: actual sample times, hashes, native bounds, Chrome version, eligibility.
- lab/browser/functional/checks.json: responsive widths, decoded sizes, control/focus/error behavior.
- lab/browser/functional/ground_check.json: frozen-figure control comparison and mobile2x bounds.
- lab/provenance.json: selected attempts,104available normalized frames,16MISSING markers, exact rejected sidecars/summaries and hashes.
- certified-lab-attempt.log: accepted-only builder refused demographic quota failure before certified evidence creation.

## Objective reference evidence consumed
`visual_diff/diff.json` uses command `image-diff`; dimensionsMatch=true. Reference and actual are both224×248; totalPixels55552, diffPixels0, diffRatio0, similarityScore100, alphaChannelIntact=true, hotspots=[]; summary:100/100 similarity;0/55552pixels differ. Inspected actual.png: native idle figure/grounds/diagnostic footer render intact. This proves exact rendering for that one idle specimen/frame only, not readability, missing-frame recovery, animation fidelity across every frame, or certified eligibility. The control finding above remains despite the100score. PassB remains an independent requirement. No repository code changed during this review.
