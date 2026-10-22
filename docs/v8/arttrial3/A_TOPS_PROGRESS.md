# Route A top production record

Five of six requested adult tops pass both layer checks and every available adult
combination (40 combinations each, 200/200 total at verification). `fur_cloak_S`
is capped at three generations and rejected. No fourth attempt was generated.
Style/readability acceptance is not implied. No boards were created by this lane.

## Promoted sources

| Top | Selected attempt | Submitted generations | Processing |
|---|---:|---:|---|
| hide_wrap_S | 2 | 3 | Overlay-only, key, Lanczos, nearopaque alpha cleanup |
| fur_cloak_S | none | 3 | All rejected: extent or dressed-source skin contamination |
| hide_wrap_E | 3 | 3 | Dressed fallback, exact torso mask-box, alpha cleanup |
| fur_cloak_E | 3 | 3 | Dressed fallback, exact torso mask-box, alpha cleanup |
| hide_wrap_N | 2 | 3 | Dressed fallback, exact torso mask-box, alpha cleanup |
| fur_cloak_N | 2 | 2 | Dressed fallback, exact torso mask-box |

17 submissions / 5 accepted = 3.4 submissions per accepted top including capped
failures and the unused third N-hide attempt already in flight. Selected attempt
indices must not replace the actual 17-submission cost denominator.

## Processing and limits

- Raw canvas preserved at1024x1536; chroma key then whole-canvas Lanczos to64x96.
- Overlay-only failed to obey coordinates even when the guide was the primary
  edit target; no coordinate refitting or contour forcing was used.
- Permitted dressed fallback hides both arms under an oversized poncho. Exact
  half-open mask-box [18,50,46,70] is extracted AFTER downscale, without body
  subtraction. Visual inspection verifies cloth-only contents. No hole filling
  was needed. These exact rectangles may look visibly artificial; no style claim.
- Lanczos ringing caused alpha249..254 over visible body despite layer coverage
  passing. Logged alpha cleanup sets existing248..254 to255 with RGB unchanged,
  never fills transparent/open regions or alters silhouette. Root directed this
  allowed cleanup; original pre-cleanup images/checks retained. The zero/partial
  corner pixels left on S-hide/E-hide have no body underneath; actual set checks
  pass all five body frames, two hairs, axe/no-axe and carry/no-carry.
- S-fur dressed attempt2 still contains visible redrawn arm skin inside the
  torso. It was not extracted/promoted merely because geometry could pass.
- Every original, actual prompt, reference SHA256, start timestamp and elapsed
  wall time is under `/Users/rexxa/work/arttrial3-data/route_a/top_*/try*/`.
  `generation.json` preserves tool output hint and default source path.
- `candidate/check` records precede final nearopaque cleanup; `accepted.png`,
  `accepted.json`, `accepted_check.json` preserve promoted result per source.
  `top_set_checks.json` records all actual adult combinations.
- Tool credits are NOT_MEASURED. Elapsed wall seconds include tool waiting and,
  on some sequential records, the short post-call observation interval.

## Attempt outcomes before final alpha cleanup / extraction selection

| Layer | Attempt | Wall time | Initial/fallback checker rules |
|---|---|---:|---|
| top_fur_cloak_E | try1 | 18.6s | coverage,overlay_extent |
| top_fur_cloak_E | try2 | 38.0s | PASS |
| top_fur_cloak_E | try3 | 40.6s | PASS |
| top_fur_cloak_N | try1 | 73.7s | coverage,overlay_extent |
| top_fur_cloak_N | try2 | 93.5s | PASS |
| top_fur_cloak_S | try1 | 21.4s | overlay_extent |
| top_fur_cloak_S | try2 | 24.3s | coverage,overlay_extent |
| top_fur_cloak_S | try3 | 55.9s | overlay_extent |
| top_hide_wrap_E | try1 | 45.4s | coverage,overlay_extent |
| top_hide_wrap_E | try2 | 21.0s | coverage,overlay_extent |
| top_hide_wrap_E | try3 | 53.8s | PASS |
| top_hide_wrap_N | try1 | 54.0s | coverage,overlay_extent |
| top_hide_wrap_N | try2 | 72.2s | PASS |
| top_hide_wrap_N | try3 | 81.7s | overlay_extent |
| top_hide_wrap_S | try1 | 25.7s | coverage,overlay_extent |
| top_hide_wrap_S | try2 | 23.4s | coverage |
| top_hide_wrap_S | try3 | 22.0s | coverage |

Summed submitted-call observation time: 765.1s (12.75min); parallel calls overlap. This is not additive human labor or wall-clock throughput.
