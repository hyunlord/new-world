# ART-TRIAL execution ledger

Contract: `TRIAL_GOAL.md`, base `3183e77dfbe6373f1c3d3e0d08a1121fb546b694`.
Started 2026-10-21 12:21:38 UTC (21:21:38 KST), verified from the session log. Target 10 hours; hard stop 14 hours.
Only `art/**`, `docs/v8/arttrial/**`, `docs/images/arttrial_*.png` enter commits.
Raw images, rejected candidates, logs and working files: `/Users/rexxa/work/arttrial-data/`.

## Sequential milestones

| Part | Deliverable and proof | State |
|---|---|---|
| 0 | English STYLE with Korean summary; reproducible 48-colour palette; all eight pipeline steps; three synthetic fixtures, negative checks; commit/push/ref equality | COMPLETE: a799b4bd; remote SHA verified |
| 1 | Two candidates per generated environment asset; selection/reject ledger; references; ground seam tests and permitted fallback; packed sheet | COMPLETE: eaf9442d; 51/51 mechanical PASS; remote SHA verified; visual limits retained |
| 2 | Four bodies × two directions × four upright frames plus south sleep; generated-on-body hair/beard, tops and tools, subtraction extraction; PERSON_SPEC | PUBLISHED821f702e; PARTIAL:235/612wardrobe, spear absent;383submitted assets checked |
| 3 | Native-scale1920×1080 comparison;24labelled requests; anonymous crops | PUBLISHEDa9fec1a9; PARTIAL:21complete/3incomplete |
| 4 | Fresh-context blind/variation, consistency and costs | EXECUTED:scene3/10→3/10FAIL;variation10/24FAIL;383/383mechanical PASS; published0b4454fc |
| 5 | Korean report in §9's exact headings, decision brief, changed files by commit, final remote verification | REPORT WRITTEN; publication is this Part5 commit |

No acceptance is inferred from generation success, metadata, or passing unit tests.
Optional drops follow the contract order; mandatory assets and tests are never dropped.
Each milestone is committed with `v8-arttrial(pN):` intent and Lore trailers, pushed only to `v8/art-trial`, then checked with `git ls-remote`.

## Preflight evidence

- Created `v8/art-trial` at the exact requested base; no remote trial branch existed.
- Preserved the pre-existing `.gitignore` bytes in external `preflight/gitignore.original` and Git stash; restored the same bytes after checkout, checked with `cmp`. Neither ignore file enters the trial commits.
- Rust workspace tests and clippy completed in session 70872 with exit code 0: `cargo test --workspace && cargo clippy --workspace -- -D warnings`. The final clippy phase completed in 11.01 seconds. The long unoptimized test run was allowed to finish; no Rust source was changed.
- Built-in image generator returned a real tree candidate. External raw path: `raw/tree_oak_candidate_01.png`; SHA-256 `8753c845c5a1f7047495f90eedce2be309b2ccdf3e25560cbeee43e8ed2de031`. Its processed 48px output was subsequently selected against a second candidate; see `CANDIDATES.md`.
- Required reference documents and target/contact-sheet images inspected before generation.

## Verification discipline

Synthetic tests exercise each pipeline step, including overflow rejection, off-palette/stray-pixel rejection, channel preservation, hat subtraction and schema compatibility. Real generated assets then pass through the same steps. Record per-asset dimensions, palette/outline/alpha checks and reference ΔE distribution. A fresh reviewer receives images only with neutral identifiers; answers and scoring key are joined only after the answer is frozen. Keep generation prompts, tool outcomes, source hashes, candidate decisions and timings externally, with a compact committed provenance index.

## Final outcome checkpoint

- Foundation and environment were published at a799b4bd and eaf9442d.
- Part2 published821f702e:36bodies+36shadows,43hair+11beard+144tops+37tools+9carry,51environment,16versioned scene revisions;383submitted assets. Mandatory wardrobe remains235/612under D14's explicit interpretation.
- Part3 publisheda9fec1a9: final A/B,24-request board and both labelled reading sheets.21complete/3incomplete;24distinct actual renderings. No substituted inputs.
- Scene round1=3/10FAIL; round2=3/10FAIL; variation=10/24FAIL. One scene revision exhausted. Full responses and frozen hashes preserved.
- Final383/383mechanical PASS; all atlas rectangles/anchors match sources exactly. Final scene/AB/ten crop files retain their frozen hashes.
- Final65Python tests PASS11.582s; Ruff PASS; basedpyright0errors/0warnings/0notes; Rust workspace tests+clippy exit0.
- Part4 published0b4454fcwith remote equality. Part5 contains the final report and this ledger; its exact SHA/remote result is recorded in the final response and external publication evidence. This workflow can publish the failed experiment; it does not waive missing mandatory person outputs or claim that all original acceptance criteria passed.
