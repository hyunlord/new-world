# ART-TRIAL execution ledger

Contract: `TRIAL_GOAL.md`, base `3183e77dfbe6373f1c3d3e0d08a1121fb546b694`.
Started 2026-10-21 12:21:38 UTC (21:21:38 KST), verified from the session log. Target 10 hours; hard stop 14 hours.
Only `art/**`, `docs/v8/arttrial/**`, `docs/images/arttrial_*.png` enter commits.
Raw images, rejected candidates, logs and working files: `/Users/rexxa/work/arttrial-data/`.

## Sequential milestones

| Part | Deliverable and proof | State |
|---|---|---|
| 0 | English STYLE with Korean summary; reproducible 48-colour palette; all eight pipeline steps; three synthetic fixtures, negative checks; commit/push/ref equality | VERIFIED_LOCAL; publication pending |
| 1 | Two candidates per generated environment asset; selection/reject ledger; references; ground seam tests and permitted fallback; packed sheet | PENDING |
| 2 | Four bodies × two directions × four upright frames plus south sleep; generated-on-body hair/beard, tops and tools, subtraction extraction; PERSON_SPEC | PENDING |
| 3 | Native-scale 1920×1080 scene, code and mock-up comparison; 24 labelled variations; optional night; anonymous test crops kept external | PENDING |
| 4 | Fresh context blind test ≥8/10, variation ≥20/24, all-assets consistency, pairwise uniqueness, full cost ledger; preserve failures and at most one revision | PENDING |
| 5 | Korean report in §9's exact headings, decision brief, changed files by commit, final remote verification | PENDING |

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

## Current checkpoint

- Palette: 48 exact colours; two rebuilds byte-match the saved JSON/PNG; one sourced skin and three declared trial skins.
- Part 0 exact foundation: 33 tests PASS (6.067 seconds), Ruff PASS, basedpyright zero errors / zero warnings, no-excuse rules PASS. Independent audit found no remaining functional §3 blocker. Later-part modules are excluded from this exact foundation gate.
- Real two-asset registry CLI: 2/2 mechanical pass, measured self-references, 8,031-byte atlas under external `registry-smoke/`. This is a subset probe, not the final environment atlas.
- Cached `uvx --offline ruff` and `uvx --offline basedpyright --pythonpath /opt/homebrew/bin/python3` work without adding project dependencies. Foundation warnings were resolved with typed JSON validation and NumPy reductions, not suppressions or relaxed settings. Later modules receive their own gates.
- Tree and wood asset generators own separate external subdirectories and selection ledgers. Parent must independently review and merge their entries.
- People pilot: first sheet rejected for repeated walk phases; raw02 corrects them. Parent inspected direction-specific registered output: 26–27-pixel adult silhouettes, nine distinct poses, accepted provisionally as the family reference. Other three bodies are in generation.
- Two generated short-hair variants failed strict subtraction because they erase body pixels. Both are retained as rejected geometry drift; neither is an accepted paper-doll layer. A bounded alternative attempt is running without relaxing extraction checks.
- Environment: ten additional tree candidates and eleven wood candidates are preserved externally. Selected tree foliage/bark receives a documented material-subpalette pass before final approval. Ground and remaining props are in production.
