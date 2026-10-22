# TRIAL3B Part 0 — template amendments and A recheck

2026-10-22 owner amendment. **Part 0 checker evidence complete; route A under
amended rules: 39/87 PASS, 48/87 FAIL.** This is a diagnostic recheck, not A
repair, visual approval, or route B acceptance.

## Definitions and implementation

`art/character/TEMPLATE.md` records exact scale, alpha, bbox, scanline and
applicability semantics. `art/pipeline3/anatomy.py` implements `contact_ok`
and `limb_ok`; `check_layer.py` invokes both. Contact shadow centre is (32,88).
Opaque lower edge must be86..88 and one contiguous overlap run with the
ellipse must be >=3 game pixels. Separate pixels across a gap cannot count.

Hands >=6px, feet >=8px longest bbox dimension, limb transverse width >=4px,
head occupied bbox >=70% head-box area. A uses fixed projected measurement
windows, not new sidecars. Measurements cannot prove that an overlapping
side-view silhouette truly represents a separate hand; visual semantics remain
an independent review requirement. Optional renderer anatomical masks are
pixel-validated. Sleep contact is N/A; sleep without anatomical limb evidence
fails `limb_ok` with NOT_VERIFIED, never silently passes hidden limb sizes.

## A results (unchanged sources)

| Measure | Result |
|---|---:|
| Total layers | 87 |
| Passed | 39 |
| Failed | 48 |
| Layers failing `limb_ok` | 48 |
| Layers failing `contact_ok` | 22 |
| Original A files byte-verified unchanged | 265 |

Failures overlap: contact failures are among the48 failing bodies. All39
non-body layers pass their applicable rules; body results include45 upright
projected measurements and3 sleep bodies whose unmeasured limbs fail closed.
No body is rejected simply for lacking an upright anatomical sidecar. Original
A `CHECKS.json`, `CHECKS.md`, images, masks, metadata and manifest are preserved.
Historical87/87 results remain historical, not amended acceptance.

Raw evidence directory:
`/Users/rexxa/work/arttrial3-data/b/part0-amendment/`

- `CHECKS.json`, `CHECKS.md`: full amended per-layer findings; checker exit1.
- `failures.png`: all48 rejected layers, labels and diagnostic previews.
- `a-before.json`, `a-after.json`: identical SHA256 maps of265 original files.
- `summary.json`: recomputed counts and unchanged flag.
- `tests-red.txt`: new tests initially fail because amendment module is absent.
- `tests-green.txt`:30 tests pass, including existing22 and new8 amendment cases.

Verification command:
`python3 -m pytest art/pipeline3/test_anatomy.py art/pipeline3/test_structure.py -q`

Synthetic cases cover accepted segmented body,4px hovering, missing visible
hands, narrow hand/foot, thin limb, undersized head, isolated foot pixels whose
bbox spans a gap, and missing sleep anatomical evidence. Historical sleep test
fixtures now supply explicit pixel segmentation; no acceptance assertion was
removed. `git diff --check` passes. These checks establish checker behaviour,
not art readability, style, frame performance or full game validation.
