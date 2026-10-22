# Route A east body frames

Five E body layers are structurally accepted, with no code/checker changes.
Current final evidence: `/Users/rexxa/work/arttrial3-data/route_a/E_final_checks.json`
(5 per-layer passes and 5 exact fixed-upper passes).

| Frame | Generation attempts | Outcome |
|---|---:|---|
| idle | 3 (inherited) | Existing try3 processed and mask corrected; PASS |
| walk_a | 1 | PASS |
| walk_b | 1 | PASS |
| work_a | 3 | Try1/2 head-alignment failure from raised arms; try3 PASS |
| work_b | 1 | PASS |

All new calls used built-in `image_gen.imagegen` edit mode with inspected E idle
raw, adult_E guide, person_v2 and scene anchors. Each sidecar retains complete
prompt, actual reference hashes, elapsed generation seconds and raw hash.
Idle try3's references were completed from its original call provenance:
E idle try1 raw, adult_E guide, person_v2 and scene.

Processing is the existing full-canvas chroma-key/Lanczos/alpha-cleanup recipe.
Head [20,32,44,50] and torso [18,50,46,70] are copied exactly from idle into
all new frames, as explicitly permitted. No fitting or repositioning occurred.
Base loincloth masks use region5 within the logged hand-coded polygon;
remaining visible pixels use skin1. No colour quantization or outline repair.

Raw failed attempts and checks are retained under
`/Users/rexxa/work/arttrial3-data/route_a/body_E_work_a/try1` and `try2`.
No failed image was promoted into accepted layers. Work poses had to use
restrained hip-level movement to preserve the head/torso geometry contract.
Final walk pair differs at418 pixels and work pair at398 pixels, but this is
not readability evidence: the moving-GIF review still must decide activity.
Final processed work frames were visually inspected at native64x96 size.
