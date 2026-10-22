# Derived child and elder layers

`python3 -m art.pipeline3.derive_bodies` derives only structurally passing adult
sources currently in route_a. It introduces zero generation calls. Adult
pixels, sidecars and check thresholds are untouched. Each derivative sidecar
records source image/mask/metadata SHA256 and exact transformation recipe.

## Transform

Upright source y landmarks0,32,50,70,88,96 map to child22,46,60,74,88,94.
Full-band widths are48,48,50,48,48, centered on x32. This maps head and torso
heights exactly to their child template, with narrower body and14px legs.
Elder landmarks map to4,36,54,72,88,96; band width64 is retained. Upper bands
shift one pixel forward: S left1, E/N right1, while torso and legs stay centered.
The shorter torso and shifted upper silhouette provide the template stoop.

RGBA uses Lanczos resize and composition. Masks use nearest sampling; if a
Lanczos edge has alpha but its nearest mask is zero, the nearest visible nonzero
categorical label supplies the region. This changes masks only. Loincloth
region5 remains5, hair2 remains2, and skin1 remains1. Alpha below16 is cleaned
as in the original preparation; no body hole fill, outline forcing or repaint.

Top torso source rectangle maps directly onto the age-specific torso rectangle
using source pixels, while extras use the band transform. Exact transformed-idle
RGBA and masks are copied into the child's or elder's immutable head/torso
rectangles for each upright frame. Pre-copyback derivatives are retained as
`pre_copyback.png` and `pre_copyback_mask.png` outside the repository.

Sleep uniformly scales the full canvas so its source silhouette length becomes
42px(child) or56px(elder), then composes it at center(48,36). Its left-side
head_box receives the same scale/translation. This is an explicitly permitted
age derivation, not an alignment repair on a generated adult candidate.
Grip/carry coordinates receive the same piecewise mapping as their pixels.

## Verification and limits

Current frozen sources: 16 bodies, six hair, two props, five tops(hide_wrap S/E/N and fur_cloak E/N).
All32 derived bodies pass layer geometry, connectivity, holes, masks and fixed
upper checks; twelve hair, four props and ten tops pass. Final counts are
child29/29 and elder29/29. Each age's actual accepted Cartesian combinations
pass201/201 (200 upright plus one bare sleep). These are passing observed
combinations, not complete minimum inventory: fur_cloak_S is missing because
its adult generation exhausted the cap. No derivative invents that source.

Before permitted top alpha cleanup, child hide_wrap_E failed opaque torso
coverage(0.9351<0.95). That evidence remains in
`derived_before_top_alpha_cleanup/child`. The final explicitly allowed alpha
cleanup maps248..254 to255 on tops only, keeping RGB unchanged and leaving all
zero-alpha pixels untouched. Exact counts are logged per sidecar. No zero-alpha
hole fill or broader opacity repair was applied. This removes Lanczos opacity
ringing without changing visible colours; the complete combination checker
then confirms no body-through-top and no enclosed union holes.

`/Users/rexxa/work/arttrial3-data/derived/<age>/checks.json` contains all final
findings; `combinations.json` records the201 actual combinations per age. All derivatives and pre-copyback files remain under that folder;
passing triplets alone are copied into route_a. Initial head-shift trials are
preserved as `derived_setup1` and `derived_setup2`: globally right-shifting the
elder failed S head alignment, globally left-shifting failed E/N. Final
direction-aware one-pixel shift passes without changing the head checker.

Derived images were viewed directly. No readability, style or performance
claim follows from these structural passes. Python compile, existing22
structural tests, and diff whitespace checks passed. Final pixels and sidecars are frozen after the29-source run. Source provenance
hashes identify the adult snapshots used for this derivation.
