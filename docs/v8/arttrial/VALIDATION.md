# Validation evidence

## Historical caf25579 code checkpoint

- `python3 -m unittest discover -s art/pipeline -t . -p 'test_*.py'`:65 tests,11.582s,PASS (final rerun).
- `uvx ruff check art/pipeline`:PASS.
- `uvx basedpyright --pythonpath /opt/homebrew/bin/python3 art/pipeline`:0errors,0warnings,0notes.
- `git diff --check`:PASS.
- External timestamped record: `/Users/rexxa/work/arttrial-data/preflight/p2-pipeline-checkpoint.json`.

Test discovery requires `-t .` for package-relative imports. The typechecker must use the system Python containing Pillow/NumPy; its isolated uvx environment alone cannot resolve those libraries. Earlier invocation mistakes are retained in the checkpoint, not classified as code failures or suppressed diagnostics.

## Independent review

Read-only agent `/root/contract_audit` reviewed compose/single_frame/frames/boards and their tests, including family-specific tint/body-context behavior, registration, dimensions and crop geometry.25focused tests passed. No blocking code defect found. The reviewer identified one stale Round1 status sentence in `BLIND_RESULTS.md`; it was corrected to point to completed Round2.

All26blind input image hashes (10first scene+10second scene+6variation) matched frozen records. The reviewer confirmed21complete/3incomplete board inputs and24distinct rendered pixelsets. A final regeneration from the packaged383asset atlas also matched all6frozen variation input images exactly. This does not turn the incomplete board or failed reading scores into a pass.

## Baseline and final Rust gates

Baseline `cargo test --workspace && cargo clippy --workspace -- -D warnings`:PASS, evidence `/Users/rexxa/work/arttrial-data/preflight/rust-baseline-result.json`.

Post-work gate:PASS; session98271 exited0. `cargo test --workspace && cargo clippy --workspace -- -D warnings` completed, final clippy0.16s. No Rust files changed. Evidence: `/Users/rexxa/work/arttrial-data/preflight/postwork-rust-gate.json`.

## Visual and coverage gates

Scene round1:3/10FAIL. Scene round2:3/10FAIL (generic-carrying leniency4/10 alsoFAIL). Variation:10/24FAIL. Strict24complete board gate:FAIL; coverage board displays omissions explicitly. One scene revision round exhausted; no additional reviewer reroll.

Final registered assets:383/383 mechanical PASS; all383packed atlas rectangles and anchors exactly match their sources. Final scene, A/B and ten blind crops retained all12frozen file hashes. Family distributions and P90 outliers are in `REFERENCE_DISTRIBUTION.md`;58self references and63byte-identical references are explicitly disclosed. The pre-publication71-file scope totaled2,181,153bytes, maximum579,406bytes; both limits pass. The final committed budget is recorded in `PUBLICATION.md`. Neither test success, palette membership, color-distance scores nor atlas packing establish visual acceptance or runtime/device performance.

## External accounting tools

The cost reconciler lives outside the shipped pipeline. Its final lint/no-excuse checks pass; basedpyright reports0errors and407warnings from dynamic producer JSON. Those warnings are retained, not suppressed or represented as a clean typecheck. The shipped `art/pipeline` has0errors/0warnings/0notes. Accounting invariants, source hashes and independent interval-union checks provide the ledger evidence.


## Continuation verification

The continuation adds21frames to the preserved383-asset package. All404assets pass mechanical checks, and every packed region, box and anchor exactly matches its registered source. All383historical source hashes,12scene artifacts and6sample recompositions remain unchanged. The board has22complete and2incomplete requests, with24distinct rendered images. `CONTINUATION_VERIFICATION.json` and `CONTINUATION_VISUAL_QA.json` record the final checks and byte-bound parent visual observation.

Current pipeline rerun:65tests PASS in10.765s; offline Ruff PASS; basedpyright0errors,0warnings,0notes. Evidence: external `continuation_accounting/pipeline_verification.json`. Pipeline code and Rust code were unchanged by this continuation; no new runtime or performance claim is made.

The additive integration helper passes3regression tests, Ruff and strict basedpyright. The isolated continuation accounting helper passes its tests, Ruff and structural checks, with0type errors and200warnings from dynamic producer data. These warnings are retained; the shipped pipeline's clean result is separate. Parent independently checked all140continuation raw/original byte pairs,142call records,21selected records and zero current in-flight calls. Historical unresolved batch slots and unknown durations remain disclosed in `COST.md`.

Scene3/10and variation10/24remainFAIL. Wardrobe256/612and complete-board22/24remainFAIL. No additional reviewer round, sample replacement or relaxed threshold was used.
