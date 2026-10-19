# Retry census

Snapshot: 2026-10-19T14:01:19.431878+00:00. Supersedes the earlier bounded report; its complete JSON is preserved in `historical_snapshots`. No runtime checks were rerun for this audit.

**Global three-strike compliance is not certified.** Every executed target check counts, including offline/debug checks. Acquisition campaigns, their subprocesses, offline comparisons, and pre-Godot admission refusals are recorded separately. Pending identity/final gates and the Stage-B denominator ambiguity remain explicit.

| Target | Completed executed checks/acquisitions | Outcomes | Status |
|---|---:|---|---|
| real-input-p13 | 2 | 1: FAIL; 2: PASS | PASS |
| H10-road-porter | 3 | 1: VISUAL_FAIL; 2: FAIL; 3: FAIL | TERMINAL_FAIL |
| H10-relations | 1 | 1: PASS | CAPTURE_PASS |
| H10-chronicle | 1 | 1: PASS | CAPTURE_PASS |
| Life-estranged | 3 | 1: FAIL; 2: FAIL; 3: PASS | CAPTURE_PASS |
| Stage-B-kind0-supplemental-acquisition | 3 | 1: ACQUISITION_FAIL; 2: ACQUISITION_FAIL; 3: CAPTURE_CONTRACT_PASS | TERMINAL_SHORTAGE |
| Stage-B-kind2-supplemental-acquisition | 3 | 1: ACQUISITION_FAIL; 2: ACQUISITION_FAIL; 3: CAPTURE_CONTRACT_PASS | TERMINAL_SHORTAGE |
| Stage-B-kind4-supplemental-acquisition | 3 | 1: ACQUISITION_FAIL; 2: CAPTURE_CONTRACT_PASS; 3: CAPTURE_CONTRACT_PASS | INVENTORY_FILLED |
| Stage-B-kind5-supplemental-acquisition | 3 | 1: CAPTURE_CONTRACT_PASS; 2: CAPTURE_CONTRACT_PASS; 3: CAPTURE_CONTRACT_PASS | INVENTORY_FILLED |
| Stage-B-kind6-supplemental-acquisition | 3 | 1: CAPTURE_CONTRACT_PASS; 2: CAPTURE_CONTRACT_PASS; 3: CAPTURE_CONTRACT_PASS | INVENTORY_FILLED |
| write-skip-W500 | 2 | 1: COLLECTOR_FAIL; 2: ADMISSION_REFUSED; 3: SPEED_PASS | VALID_MEASUREMENT_PASS_JOINT_ACCEPTANCE_FAIL |
| write-skip-W200 | 1 | 1: SPEED_FAIL | VALID_MEASUREMENT_THRESHOLD_FAIL |
| walker-cargo-tests | 2 | 1: COMPILE_FAIL; 2: PASS | PASS |
| walker-bridge | 2 | 1: COVERAGE_FAIL; 2: PASS | PASS |
| walker-numeric | 1 | 1: PASS | PASS |
| walker-fixture | 1 | 1: PASS | PASS |
| walker-identity | 2 | 1: FAIL; 2: PASS | PASS; merged identity pending |
| Stage-B-initial-generic-matrix | 6 | 1: CAPTURE_CONTRACT_PASS; 2: CAPTURE_CONTRACT_PASS; 3: CAPTURE_CONTRACT_PASS; 4: CAPTURE_CONTRACT_PASS; 5: CAPTURE_CONTRACT_PASS; 6: CAPTURE_CONTRACT_PASS | ACQUIRED |
| walker-speed-W200 | 1 | 1: PASS | VALID_SPEED_PASS_IDENTITY_SEPARATE |
| walker-speed-W500 | 1 | 1: PASS | VALID_SPEED_PASS_IDENTITY_SEPARATE |
| walker-speed-comparator | 3 | 1: FAIL; 2: PASS; 3: PASS | PASS |

## Reconciliation

**Stage B: 21 runtime acquisitions = 6 initial generic captures + 15 targeted supplements.** The initial matrix contains three Z4 sources and three Z1 views. Stage-B selection uses the three Z4 sources plus ten successful supplements: 13 sources, or 26 PNG/JSON files. Five supplements failed acquisition. Each missing kinds 0/2/4/5/6 received three supplements; final eligible inventory remains [1,2,1,2,2,2,2]. Kinds 0/2 remain terminal shortages.

The prospective protocol specifies three Z4 scenes and two eligible holders per activity; it does not unambiguously define whether each generic scene also consumes a retry for every missing kind. If treated that way, supplemented kinds have six acquisition exposures each, not three. This census does not retroactively redefine the target or assert blanket compliance. All acquisitions remain visible; successful capture contracts are distinct from successful reader/recognition checks.

**W500 write-skip:** attempt 1 executed and failed collector validation; attempt 2 was denied before Godot; attempt 3 executed and passed with ratio 0.947285393220096. Thus three admission slots, two actual collectors. W200 has one valid session with ratio 0.9986985164993292: threshold FAIL. Write-skip remains default OFF.

**Walker:** bridge coverage FAIL1 (22/24 nonempty), PASS2 (24/24). Identity comparator1 FAIL retains 924 differing control pixels; acquisition 2 now passes the compound comparator; merged-build identity remains pending. Runtime speed sessions are one per world; latest ratios are W200 0.64922376483374 and W500 0.9446215390142153. Offline speed comparison executes three times: calibrationFAIL1, corrected invocationPASS2, strengthened W500 guardrailPASS3. These are three checks of the same runtime data, not extra runtime measurements.

**H10 and Life:** road has three failures, including runtime 0/visualFAIL attempt 1; relations/peace success does not repair H10. Estranged failures1/2 and success3 remain preserved. Cargo compileFAIL1 counts as failed verification invocation, although tests did not execute; attempt 2 passes 57 tests. Numeric/fixture each retain their single observed pass.

Admission waiting is **3600 seconds**, from `perf/attempts.json` SHA256 `6a74b477fbcd05943eb62c75db859194cda4f1238670ea67adb8ef3f706c4bda`. Its `admitted=true` means policy-spacing permission; W500 slot2 has invalid runtime probe/exit 75 and no Godot. Current six slots correspond to five runtime measurement launches.

## Legend acknowledgment

No original timestamped legend acknowledgment was located in the searched saved evidence/OMX outputs. TASKS contains the contemporaneous narrative that a fresh fork-none reader acknowledged seven names before crop delivery. The original message timestamp remains **NOT_AVAILABLE**; reader answer-freeze timestamps are not substituted. No reader rerun or fabricated attestation was created.

## Census boundaries

The JSON inventories all current external top-level logs and baseline gate row logs with hashes, preserves prior failures, and includes actual launch tables for identity campaigns. Ancillary format/strict/tool/build logs are retained without inventing invocation counts or terminal verdicts. It is not a claim that every recursive subprocess or later final-gate check has been exhaustively classified. Future terminal checks must extend this census before final reporting. No hidden reader answers were inspected.

See [retries.json](retries.json), [H10 report](h10/REPORT.md), and [write-skip report](perf/write-skip/REPORT.md).

## Part3 refresh — 2026-10-19T15:00:52.563637+00:00

Identity attempt2 is PASS; attempt1 failure remains. Part3 gate1 is 118/120 (cargo_test and meal_ring failed). Gate2 is 120/120 by exit codes but **not clean acceptance**: agent_drawn_positions and cold_presentation emit missing-bridge ERROR. Both corrected fixtures pass individual actual-bridge retests with zero failures/no ERROR. Targeted dispatch 4 tests and meal_ring also pass.

Gate3 is active at `gates/p3-attempt3`; no completed row existed in its results.tsv at this snapshot. It is not PASS. Merged-build identity, Part4 and final gate remain pending. JSON contains every 120-row set from p0, p3 and p3-attempt2 plus cross-gate invocation lists. Campaign counts and row counts must not be summed as different executions. Real-input-p13 now has 4 completed invocations (baselineFAIL, targetedPASS, two Part3 PASS). The two missing-bridge fixtures each have two error-producing Part3 runs and a targeted clean retest; upcoming gate3 remains counted when it actually executes. No blanket retry compliance claim is made.

## Part3 terminal update — 2026-10-19T15:28:15.298609+00:00

Gate3 is now **CLEAN PASS: 120/120, zero ERROR**, session 30784 terminal 0; root audit 15:13:04.508215Z. Earlier gate1FAIL and gate2log-errorFAIL remain. Cross-gate row census includes all 120 gate3 rows. Real-input-p13 now has 5 completed invocations: baseline failure, targeted pass and three Part3 passes.

Required merged identity repeat is **PASS**, session 53070 terminal 0:28 launches from 15:13:29Z through 15:26:34Z, observed terminal 15:26:57.646379Z. All six RGBA framings and five running worlds are exact. This is an additional required post-merge compound check after historical identity2PASS; **no timing rerun**. Part4 and final gates remain pending. Earlier in-progress prose above is retained as historical snapshot.
