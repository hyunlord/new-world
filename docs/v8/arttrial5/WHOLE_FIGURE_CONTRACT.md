# Whole figure preregistration — 2026-10-23

Frozen before inspecting any Part 1 generated sheet. See `art/character/TEMPLATE_V3.md`.
Alpha structure uses >=128, the conventional half-opacity coverage boundary,
not a fitted threshold. Pixel storage is never thresholded: all RGBA values survive
crop and Lanczos scaling. Alpha >0 determines crop/edge extent. Structural
alpha>=128 determines the contact baseline; its absence fails.

Exactly five 8-connected alpha128 components, ordered left to right, are required.
If fewer exist and a component crosses a nominal equal-spacing boundary, search
that boundary's +/-10% sheet-width/5 window for the narrowest occupied alpha
column; split between columns, retaining every pixel, record the cut. No isolated
component is discarded. More than five components fails. Components must occupy
disjoint horizontal intervals after cuts; a detached prop fails rather than being
silently assigned. These are structural figure candidates, not semantic proof.

Uniform scale = 56 (adult) or 40 (child) / idle alpha128 bbox height. All panels
use this same factor and Lanczos; rounding each extent is recorded. Lowest structural
bottom pixel y88, sleep y56. Canvas 64x96, sleep96x64. No output clipping allowed.

Coverage = count(alpha>=128)/canvas area. Upright adult [0.08,0.55], child
[0.04,0.40]; sleep adult [0.04,0.50], child [0.02,0.35]. Broad bounds are geometric
sanity limits for 56/40px figures on a 6144px canvas, allowing wide baked props;
they do not certify anatomy. Single 8-connected structural component, no enclosed
4-connected background hole >12px². Alpha>0 may not touch canvas border. At least
one alpha128 pixel; structural support must exist. Contact is pixel geometry only,
not proof of hands gripping a tool. Limb information is NOT_MEASURED without
semantic masks; reviewers assess activity and anatomy. Sleep must be wider than tall.

Every attempted sheet and rejection keeps diagnostics. Whole sheet acceptance
requires all five structural checks plus frozen judge-v2 numeric pass. Model/style
and semantic review remain distinct required evidence. No repair, island deletion,
quantization, morphology, independent per-pose scale, or hidden threshold changes.

## Pre-batch format correction

Before this executor inspected any Part 1 output, the lead supplied Part 0 source
format evidence: image-tool RGBA outputs have alpha extrema(0,254), never255.
The initial exact255 baseline would reject this format independently of anatomy.
Baseline measurement therefore uses the already frozen alpha>=128 structural
boundary; the 128 threshold, stored RGBA, coverage limits and hole limits are
unchanged. This supersedes initial exact255 wording; it is not a post-result fit.

## Version 2 measurement correction after first six sheets

Original v1 rejection records remain under external `production/slices/` and
`production/processed/`. The first six reveal that raw connected-component count
confounds a whole person with isolated source-alpha dust (observed 1–3 pixels).
V2 declares dust as an alpha128 component strictly smaller than 0.01% of raw
canvas area: `<157.2864 px²` for1536x1024. Exactly five **major** components are
required (with the existing touching-component split rule). Six substantial
figures or substantial detached props fail. Dust is assigned by the rectangular
panel boundaries, never erased; every source RGBA pixel inside a crop is retained.
Raw count, major count, threshold, each fragment area/bbox/destination panel and
split columns are logged. Scale uses the idle major component's height so remote
dust cannot shrink the figure. Final connectivity, alpha threshold, holes, edge
and coverage gates remain unchanged. Lanczos may naturally attenuate tiny dust;
there is no additional filtering or pixel repair.

The frozen person judge's full-sheet result is preserved and remains in the
original acceptance conjunction. Native cropped single-person judge results are
added as **supplementary** measurements to expose the five-miniature-person vs
single-reference comparison confound; they do not replace the original gate.
Reprocessing writes `slices-v2`, `processed-v2`, `panel-judges-v2` and
`processing_summary_v2.json`; no v1 diagnostic files are overwritten. This is a
post-observation measurement correction, explicitly distinct from preregistration.

## Version 3: lossless connected-component ownership

The lead's independent contract review established that “connected components of
alpha → five figures” authorizes automated whole-figure extraction. V2's demand
for disjoint rectangular columns was an additional restriction, not a goal gate.
V3 replaces only that extraction restriction. It retains alpha128 major/dust
classification, dust area threshold, exactly-five count, touching split rule,
scale, all final checks and the frozen full-sheet numeric acceptance conjunction.

Ownership is fixed before v3 measurement: each major CC retains every original
RGBA pixel and coordinate. All remaining alpha>0 pixels belong to the closest
major CC by Euclidean distance transform; exact ties choose the leftmost major
(component order by minimum x, stable source component order). Each opaque dust
CC remains intact, assigned by minimum distance from any of its pixels to a
major, with the same leftmost tie. No pixel is painted, recoloured or repaired.
Zero-alpha RGB is not visible evidence; all nonzero-alpha pixels must reconstruct
exactly once with byte-identical RGBA in source coordinates before any resizing.
Per-panel crop rectangles may overlap, but owned pixel sets cannot overlap.

Log source RGBA hash, uint8 row-major ownership-map SHA256 (0 transparent,
1..5 left-to-right), ambiguous/tied visible pixel count, dust destinations and
ties, and reconstruction/foreign-major invariants. Native panels retain exact
original RGBA values. Output Lanczos and checker are unchanged. V1/v2 evidence
remains immutable; v3 outputs use `processed-v3`, `slices-v3`, `panel-judges-v3`
and `processing_summary_v3.json`. Supplementary native judge never replaces
historical raw-sheet scores or their role in acceptance.

## Version 4: post-Lanczos zero-alpha padding

An audit of ten v3 extent rejections found six where post-Lanczos alpha-zero
padding, rather than visible output extent, caused rejection. V4 crops only the
alpha>0 bounding rectangle **after** the same resize; no alpha>0 pixel is removed
or changed. The full resized size, trim rectangle, placement, original-resized
coordinate to canvas translation and retained visible count are logged. Trimmed
visible extent is centered horizontally and its lowest alpha128 pixel registered
to the unchanged baseline. Actual visible overflow still rejects. Historical v3
outputs remain under their original paths; v4 uses versioned output directories.
Sleep-axis checks continue to compare actual alpha128 silhouette width/height,
not canvas/buffer dimensions. No scale, alpha, coverage, hole or numeric threshold
changes accompany this normalization correction.

## Provenance and atomic-publication hardening (algorithm remains v4)

Numeric acceptance now verifies the exact raw SHA and person family, pins the
existing frozen calibration SHA256, and recomputes numeric metrics/checks against
that calibration. Existing judge JSON has no calibration field: no historical
field is invented; fresh bindings are separate provenance evidence. Model calls
are not involved, and every numerical threshold is unchanged.

Reentrant processing audits cached result/dispatch/raw/original/prompt/calibration
identities, diagnostic age/facing/attempt/algorithm/judge, normalized-image hashes
and fresh checks, and accepted delivery files. Historical records are never
rewritten on mismatch; timestamped `cache-audits-v4` reports expose failures and
the current summary/exit status includes identity errors. Historical diagnostics
without an explicit age use their recorded look-age prefix plus all available
frame-sidecar ages; a raw-size failure never reached the image algorithm.

Publication copies all ten artifacts into a hidden sibling staging directory,
validates hashes and every figure, then renames the complete directory atomically.
A copy/validation failure leaves no public partial sheet and retains diagnostics.
