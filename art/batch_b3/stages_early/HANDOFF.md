# B3 early construction / damage handoff

16 families are fully audited across 96 slots. 60 generations cover 54 physical state slots; six weak states (11.1%) received a second try. Maximum actual attempts: two. No pending generation or review remains.

| Outcome | Slots |
|---|---:|
| New numeric + visual candidates | 27 |
| Rejected / unavailable | 27 |
| Existing A2 built / damaged reuse | 18 |
| Portable-object construction N/A | 8 |
| Shared blueprint recipe, root owned | 16 |

`coverage.json` is the integration authority: rows have `family`, `state`, `status`, `path` or `source_commit` + `source_path`, SHA256, and all attempts/reasons. `manifest.json` includes every full raw/normalized numeric record, exact prompt, seed-null disclosure, reference fingerprints and normalization operations. The 27 PNG/JSON pairs live in `art/style05/struct/b3/stages_early/`. No runtime bindings changed. `art_import.draft.json` is intentionally a draft with source commit for root to fill after committing.

The A2 reuse pin is `227529c26307bc3a854eb7bf819e88d02af39046`. Built sources are exact per-family A2 sprites. Existing pen-fence and windbreak damage variants are reused. Grinding stone, loom, hearth and trap are placed complete; their foundation/half-built slots are N/A. A2 fallen megalith is 2x1 and was not aliased to the upright 1x1 family; the new 1x1 flat-fragment ruin is selected instead.

All four `boards/early_{1..4}_1x.png` and nearest-neighbour `_2x.png` boards include every attempt beside the exact A2 built source. Historical labels report numeric results, not final adoption. Bread foundation/ruin try1 are too narrow; megalith ruin try1 is upright. Ritual foundation try1 passes numeric checks but loses the original five-upright-plus-altar layout/width and is withheld after independent root review. Root confirmed score90/PASS for the other27 final selections. Frozen metric failures are retained; none was repaired through thresholds, repainting, palette replacement or alpha cleanup.

Verification: `verification.json` records fresh hash checks for all60 preserved raw images, normalized images and references, plus all27 exported PNGs;96 unique coverage rows; raw transparency minimum0; unclipped opaque border. Six raw images peak at alpha254 and54 at255; existing frozen numeric alpha checks pass where reported. Calibration hash remains `7218ac77979065ec6398c77cb6156bce5bd523979ed48732945f2ab642a5d5fd`.

External evidence: `/Users/rexxa/work/art-batch/B3/stages_early/` contains records, original tool arguments/results, raw outputs, frozen processing results, finalization script and matching boards. Regeneration is unnecessary. Verify portable manifest hashes with Python hashlib against `manifest.json` record fields `raw_preserved/raw_sha256`, `normalized/normalized_sha256`, and each `references[].path/sha256`; verify exported rows from `coverage.json` against their `sha256`.

Not checked: runtime import/placement and historical separate model-judge scores. `judge_v2.model_status` remains MISSING and full judge-v2 pass count remains zero;27 are numeric+human-visual candidates, not fabricated model passes. The27 unavailable slots remain integration gaps.

Root integration review normalized the27 file proposals to role/id/source/dest, added assets.json geometry adapter rows and bindings.draft.json symbolic mappings. This changes draft packaging only; all selected PNGs and numeric records are unchanged.
