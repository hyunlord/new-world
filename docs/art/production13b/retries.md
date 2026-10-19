# Retry ledger

Evidence snapshot: 2026-10-19T13:28:17.877697+00:00. Docs-only audit; no checks rerun.

GOAL §0.5 counts every executed check, including debug runs, and stops a failing target after three. §0.8 separately limits admission to six slots per item, at least 30 minutes apart, with three hours total wait. An admission refusal before Godot is not an executed check.

| Target | Observed executions | Outcomes | Current status |
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
| write-skip-W500 | 1 | 1: COLLECTOR_FAIL; 2: ADMISSION_REFUSED (not executed) | NOT_VERIFIED |
| write-skip-W200 | 1 | 1: SPEED_FAIL | VALID_MEASUREMENT_THRESHOLD_FAIL |
| walker-cargo-tests | 2 | 1: COMPILE_FAIL; 2: PASS | PASS |
| walker-bridge | 2 | 1: COVERAGE_FAIL; 2: PASS | PASS |
| walker-numeric | 1 | 1: PASS | PASS |
| walker-fixture | 1 | 1: PASS | PASS |
| walker-identity | Unknown / pending | Terminal evidence not observed | COMPLETION_NOT_OBSERVED |

H10 road attempt 1 returned runtime 0 but failed visual review: a foreign mint SitByFire badge covers the porter glyph in EN/KO. Attempts 2/3 fail the strict unobscured-candidate requirement. Preserve all three; H10 is FAIL despite successful relations and peace captures. Estranged succeeded on its third execution at 51870; its first two failures remain counted.

W500 has **one executed failed collector and one refused runtime admission**, across two policy slots. The ledger’s `admitted=true` is policy-spacing permission. Attempt2 has load 23.0029 (limit<8), exit 75, and no collector execution under `session_validity_p13.py:149`. W200 has one valid measurement execution containing three pairs; its ratio 0.9986985 fails the 0.97 speed threshold. The admission ledger records 1800 seconds waiting.

Stage B has three supplemental acquisitions for each of kinds 0, 2, 4, 5, 6. Runtime/capture-contract success is not proof of two eligible distinct holders. Final inventory is [1,2,1,2,2,2,2]; kinds 0/2 stop with one eligible crop each. These are supplemental-run counts; the initial matrix is not silently folded into or erased from a purported global per-kind count.

Walker cargo attempt 1 failed compilation before tests; attempt 2 passed 57 tests. Actual bridge attempt 1 lacked nonempty coverage (22/24), then attempt 2 covered 24/24. Numeric 560 cases and fixture 4000 records each passed their single observed run. These do not establish the unfinished identity gate.

Every completed evidence file referenced in [retries.json](retries.json) has its path, byte length, and SHA256. Active identity outputs are intentionally unhashed. See [H10 report](h10/REPORT.md) and [Life report](life-capture-report/REPORT.md) for visual/capture provenance.

Limits: this is a bounded ledger, not a global run census. Blind/fact reading is separate and no reader answers were inspected. No source files or capture artifacts were modified.
