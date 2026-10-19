# Independent review — round 2 of 2 (FINAL)

Final reviewed source: `02f8721d04494dedc176a1ef34f1601338d89374`. Review verdict: no actionable production-code or evidence-integrity defect found; full technical gate PASS82/82. Overall Art Pass10 performance acceptance remains NOT_VERIFIED and activity readability is limited (blind6/14). H8 inspector evidence is supported with signal-click/pointer limitations below. This is the final update of round2, not a third review. No GOAL.md/original attachment sections 3–7 read. No Godot, cargo, heavy tests, production edits or recursive agents.

The following interim entries are preserved chronologically; their pending statuses are superseded only by the explicit final evidence section at the end.

## Interim conclusions

No new actionable production defect identified. Round1 source/provenance results remain applicable to their exact historical sources, not an assertion that the final activity implementation is pixel-identical to the pre-activity renderer.

The capture validator correction in `tools/art_pipeline/run_activity_captures_p10.py:151` permits 131 samples through frame130. The preserved first-attempt sidecar records state-change frame10, capture frame130, 131 samples, both observed deltas 0.0166666666666667, fixed_delta_valid=true, complete=true, no capture errors. Existing `_process` counts all callbacks, including frame0; the correction matches this inclusive boundary. New test explicitly accepts131 and rejects132. Fixed FPS, delta tolerance1e-9, requested instant, exact frames10/130, and minimum120 samples remain enforced. This is a bounded sample-count correction, not relaxation of pixel identity or simulation/pose thresholds. First attempt still records capture_contract_pass=false and its original validation failure.

Second cohort runner binds HEAD0c044, capture script SHA256d4c35c0cd50471ff16307ea14a8db67f25c40177c441ad2fe0b4b63d02d9b9c7 and bridge SHA256c757571424054d7f916103a87773b63c069c94a26779ff6bb22465d9fcf12e3f. It records all six requested cases and explicit selection evidence. Runner captures the working-tree patch; Python validator is currently uncommitted and should be committed with its regression test after cohort freeze, retaining its patch binding.

Reviewed installed activity tests assert concrete pose/body/view matrices, cache parity and copy safety, arrival seat/declump/walker exclusions, same-tile spread and activity icons. Existing test count/logs in committed focused evidence are historical (47 Python tests); newly reported48 result awaits its evidence location. No rerun launched by reviewer.

## Pending final inputs

- Actual six-capture terminal results, independent pose-oracle results and visual/blind records.
- Integrated Part3 source, 14-key EN/KO and registry preservation, focused inspector/runtime and visual evidence.
- Final performance matrix, source/build provenance, quiet-window records, full gate and final allowed-path diff.
- Final evidence should retain the tick candidate as REJECTED after three P5 failures and never claim its speedup as installed production behavior.

## Update: committed Part2 evidence and Part3 source reviewed

Reviewed Part3 `5fa4b3178c894f0ea0131c42f66d1ba4d1908dad`. No actionable code defect identified. Focused inspector source hashes all match installed files; archived logs record personal63 and activity85 checks with0 failures, locale exit0. Independently compared merge baseline compiled EN/KO: exactly14 additions, zero existing string changes, zero existing registry-ID changes. Eight activity IDs5226–5233 and six inspector IDs5234–5239 are contiguous and preserve prior IDs. Full diff remains within allowed path scope; Rust only the existing static-read test.

Oracle precision v2 is justified for this fixed schedule: `reset_world` queues world_reset in the deferred callback queue; capture changes state after settled frame10; renderer reset sets animation0 and `_process(0)`/refresh adds no time; frames11–130 each add1/60. Repeating120 IEEE754 additions yields1.9999999999999978, whereas decimal JSON reports2.0. The oracle reconstructs from strict frame10/130 and fixed60 inputs, not observed pose/frame; serialized time must agree within1e-12. It then compares one exact expected pose, not both adjacent phases. Tests distinguish120 additions(run) from121(hop), reject altered schedule and inconsistent serialized time. Recorded17 oracle tests pass; original first-five mismatch report remains archived. This is arithmetic reconstruction for a fixed capture schedule, not a general oracle for arbitrary frame timing.

Verified Part2 ZIP SHA256c8ac836f5f19fcdb6baa57ab8202febdada1f6076f9611f051c394df8958eb3b, CRC and member hashes; six oracle reports contain150 observations and0 mismatches. Verified frozen answers SHA25649cea5aa790d771f30d2f42dded4af31b78dab2b8b129e220c3d02ca15883c01. Freeze02:37:05 precedes recorded reveal02:37:17. Archived crop key agrees with score truth; recomputed score6/14. This is poor recognition evidence with no invented pass threshold; technical pose consistency must not be presented as visual readability. The documented icon/position/occlusion limits remain explicit.

The before/after sample-boundary regression logs are now retained under verified-captures; after log records48 Python tests passing. No reviewer runtime tests launched.

Still pending: actual EN/KO parent screenshots and click evidence, final performance/quiet qualification, final full gate. Do not infer these from focused tests or historic accepted Part1 identities.

## Update: actual parent inspector evidence (b8b452d9)

Reviewed actual-parent launch, source bindings, capture script, parent sidecar and visual-review.md. Launch exited0 with no validation errors; recorded before/after source bindings match. All indexed parent artifact hashes verified. Sidecar records the live selected parent5, partner24 and child30, four verified signal-triggered selections across EN/KO, and six620px-panel captures. Click harness checks target is an actual live family member, emitted selection resolves to the correct entity bits, and returned detail matches that agent. This supports handler/identity behavior; pointer hit testing and real mouse-wheel interaction are explicitly untested.

Independently opened EN temperament and KO family images: four distinct bar fills and localized labels fit; living partner/child rows and disabled absent-parent rows are readable; no raw keys, tofu, overlap or horizontal clipping observed. Lower disabled text has reduced contrast. Existing dedicated visual review inspected all six parent images separately; temperament/family lower views share the same scroll position and are not independent scroll-extents. Exact temperament numbers are sidecar data, not printed UI values. No new actionable defect found.

H8 inspector evidence is supported within this scope: source/static/focused tests, actual bilingual parent snapshots, scroll placement and live family signal routing. Final performance/quiet/full-gate evidence remains pending; this review does not convert pending measurement into a pass.

## Update: Part4 cutoff/quiet evidence (208eb522)

Reviewed final-unmeasured supervisor source, gate events, runner logs, verdict and quiet holds. Gate closes at1792119780.08078 (03:03:00UTC original deadline), only own waiting wrapper/sleep descendants receive TERM, no MEASUREMENT_STARTED event exists. Matrix runner exits1; waiting wrapper143 is a cutoff result, not a measured performance failure. Before/after production/build/tool bindings match. All14 indexed local and ZIP member hashes match; ZIP CRC passes.

Quiet log checks at02:43/02:48/02:53/02:58 record foreign worldsim_probe/cargo/rustc and loads34.89/125.44/92.02/62.52. There is WAIT_BUSY but no final HOLD_START. These records support zero final measurement launches and no accepted after world. The explicit NOT_VERIFIED verdict for H1–H4, H6 and after-section timers is appropriate. Historical candidate speedups cannot replace final-matrix measurements, and the rejected tick optimization remains rejected.

No threshold relaxation or fabricated after measurement found. Full Part5 gate at208eb522 is still running and remains PENDING in this review.

## Update: final gate attempt1 and static correction (02f8721d)

Attempt1 is preserved under gates/final and final-supervisor-attempt1:82 rows,81 exit0, cargo_test exit101, supervisor terminal1, unchanged source. Cargo failure is exactly the old TARGET_LABELS length8 assertion encountering9. Bridge detail mapping in world_node.rs1997–2008 explicitly maps Agent→5, GatherFood→6, GatherWood→7 and Activity→8. Correction changes expected length to9, retains exact5/6/7 assertions and adds exact8 activity-place key. This restores domain coverage without weakening assertions or modifying production behavior.

Rebuild changes dylib SHA from c7575714… to85d2b0c3…; supervisor records this separately from stable source. Earlier capture evidence remains bound to its original c757 build, not retroactively rebound to the gate build. Full diff through02f8721d remains in allowed paths, with exactly two Rust sim-test static-test files changed and no Rust production changes.

Attempt2 remains PENDING until terminal evidence is available. A focused passing static check does not substitute for the full rerun.


## Final evidence and disposition

Independently read final-attempt2 raw results and external final-gate-supervisor-attempt2 records. Terminal gate.exit is0; result statusPASS, valid=true, errors=[], source_stable=true, changed_source_paths=[]. Results contain exactly82 unique expected row names, all exit0. All83 artifact SHA256 entries match actual files. Before/after HEAD is02f8721d04494dedc176a1ef34f1601338d89374; complete source and gate hash dictionaries match. This supports the full technical gate at that exact source, including cargo tests, clippy, bridge build, strict/static/focused tests and smoke. No new tests were launched by reviewer.

Gate rebuild changed dylib85d2b0c3… to6b76bbc3bb25d4b9de088f191b8337efbb57e9b7a6bdd9b45e6b3fc5f7405c85. This is explicitly recorded, not conflated with the c757 build used for historical identity/activity/parent captures. No claim is made that those images were captured with the final rebuilt binary.

Final disposition: no actionable source defect or assertion/threshold weakening found. Static target-domain correction is supported by actual FFI semantics; previous failed full gate remains preserved. Committed scope through reviewed HEAD remains authorized paths, exactly14 new locale keys with all prior strings and IDs preserved, and Rust changes limited to two static sim-test files. Reviewer touched only external round1/round2 reports. Workspace .gitignore/.ignore changes were pre-existing and excluded from reviewed committed scope.

Remaining acceptance limits are substantive and must remain in the delivery:

- H1–H4, H6 and after-section timers NOT_VERIFIED: no final measurement launched before cutoff. Candidate improvements are historical candidate evidence, not a final performance matrix.
- Tick cache remains REJECTED after three P5 identity failures; no tick-cache performance claim applies to installed production.
- Activity oracle150 observations/0 mismatches verifies its exact pose contract. Blind identification6/14 and crowded/occluded overview sprites limit visual readability; no pass threshold invented.
- Inspector actual EN/KO layout and family/temperament evidence support H8. Four family clicks were actual button signals; pointer hit testing and mouse-wheel input were not tested. Exact temperament numerals are not displayed, although bar values are recorded.
- Pose oracle does not independently certify arrived world-position rules, icon visibility or pixel occlusion; focused behavior tests and bounded visual observations are separate evidence.
- No further review round requested or performed. Later documentation-only packaging can cite these exact source/gate bindings, but source changes would exceed this final review's coverage.
