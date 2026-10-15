# Pass 17 independent review

Two fresh reviewers were given the unchanged brief, source diff and output artifacts, without the implementation conversation or another reviewer's conclusions. These are the only two independent review rounds. Focused implementation checks, raw-data audit helpers and report assembly checks are supporting evidence, not additional review rounds.

## Review scope and current status

| Round | Reviewed source | Result | Evidence |
|---|---|---|---|
| 1 | `43f22e81f120a755de33712b02a62469c67ab840`, runtime/evaluator producer `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21` | Two P2 F3 population defects; original metrics independently reproduced | [Full review](verification/p17-review/round1.md), [evidence](verification/p17-review/round1.json) |
| 2 | `51e1629a059ada0aa6ea9c8434b348d7587cc9aa`; runtime remains47b72440 | Corrected source and trace arithmetic verified; no new actionable source defect; terminal C full gate PASS | [Full review and same-round C supplement](verification/p17-review/round2.md), [evidence](verification/p17-review/round2.json). The pre-gate snapshot remains separately preserved. |

Both stages retain measured acceptance FAIL: Stage A R5 exceeds relative timing budgets; Stage B F1=527/1064 is below0.50. The original tuning F5=9/80 FAIL also remains. Technical gate success cannot override any of these results.

## Findings and dispositions

| Finding | Evidence and scope | Treatment |
|---|---|---|
| F3-1 / P2: extra control-stage filter | Tuning18100 has2316 eligible unrelated co-resident pairs;489 were excluded solely because their stage category was absent among siblings. Notice10:53:13.607460UTC, before held-out metric output. | Preserved original registration/results, removed only the extra filter, added regression coverage and replayed unchanged raw data. |
| F3-2 / P2: sibling co-residence restriction | Tuning contains1790 recorded full-sibling pairs;40 were excluded,31 concurrently alive in separate settlements and9 without shared hourly presence. Notice11:08:18UTC, before held-out metric output. | Include every known ordered-parent full sibling regardless of residence/coexistence. Shared-snapshot context remains null when absent; ancestry is never inferred. |
| R5 valid measured failure | Mean+33.0401456%, p99.9+339.8637138% versus+15%/+35% bars. | Preserve FAIL; do not rerun for a favorable measurement or change thresholds. |
| F1 valid measured failure | Held-out527/1064=0.4953007519 versus0.50. | Preserve FAIL and original familiarity300; no evaluation-seed tuning. |
| Post-launch methodology correction | The original registration disclosed two extra filters, but they were not in literal§7.7. | Report original/corrected results and exact timestamps/source separately. Do not describe the corrected analysis as unchanged preregistration. |

Original analysis was committed as `e353f18f5b523b4198fd785b44c5ad962b15576d` before correction source `51e1629a059ada0aa6ea9c8434b348d7587cc9aa`. Test-only application produced9 expected assertion failures; the correction then passed9 focused and45 total Pass17 evaluator tests. Every non-F3 value and verdict is exactly unchanged. Original evaluation F3=0.10862373982554437; corrected F3=0.10492328730233096,95%CI[0.08279428448915346,0.1260843651055091]. Both pass their original bar. Corrected tuning is a replay, not another tuning round.

## Required checklist

Both reviews inspect the section8.2 obligations: I1–I5 and fixed pins; only the two allowed new fields of existing public structs; exhaustive target matches; activity entry after Gather and before settlement; builder/peer/meal Idle equivalence; no new tally; unchanged birth gates; hand arithmetic; preregistration order and cohort separation; exact verdict rules; allowed paths; and quiet-lock records. Their full tables provide file/line and raw-artifact references.

The second review independently confirms all32 corrected pair-ID sets:7054 siblings,107049 controls and271 siblings without shared context. It reproduces F1,F2,F4,F5 exactly and the F3 point/interval to floating precision. It verifies73 identity evidence hashes,706 corrected evaluation receipt entries and369 corrected tuning entries with zero mismatches. Original report bytes match the archived commit. All four32-world identity summary pairs have zero non-timing differences; all20 recorder hashes and both3-test pin logs agree.

## Independent hand examples

Round1's seed17000/person0 Repair starts323, arrives336 and completes439:103 consuming minutes,13 walking minutes excluded. P=613 gives90+trunc(60*(613-500)/500)=103. Summing raw intervals yields adult completed1364minutes/17episodes and matches the stored trace. Round2 independently uses person7: a33-minute interrupted HearthCircle excludes110 walking minutes; a day-end TendFire completes88minutes. Its adult totals are2336minutes/31completed episodes, with2995 total activity minutes.

Round2's held-out family uses seed18000/mother11/father9. They pair at7920 and have five recorded real-parent children at8640,9360,10800,12240,14400. At9360 parents and Child-stage32/33 share household9 and bed[33,28]. All eight eligible20:00 couple-nights share the shelter. These five births precede day20, so contribute0/0 to post-day20 F2 while remaining in ancestry/all-birth records. Broader family and fission reconstruction is retained separately and is not substituted for this hand example.

## Final technical gate

PASS at canonical C `21d2b478a81d55ac138f0a60b830e06719ec0af3`: all 15 ordered steps exit 0, debug/release each 2,241 passed with 0 failed/ignored, Python 246 executed and worldmodel 25 executed plus 24 explicitly explained skips. The same round2 reviewer independently verified all 18 artifact hashes/sizes and 19 sidecar entries, the empty smoke error log, all 623 allowed changed paths, C source binding and bounded no-RUNLOG lock evidence. Full skip and source-snapshot limitations are in the supplement. C gate ran 11:33:38.913896–12:38:02.001723 UTC; receipt SHA256 `15006bc94791f886cb199ac1c6cfe5b047fe188ce7021602c582e52668e042cb`. This closes the C gate evidence item within round2, not a third review.

The later documentation/evidence publication commit R requires its own full external exact-head gate before publication. It is not certified by C. Its terminal evidence is recorded in `/Users/rexxa/work/p17-data/final_publication_pass17.complete.json`; no future R outcome is claimed in this snapshot. Canonical C logs remain correctly labelled and unchanged.

## Limits retained

No art rendering or owner visual acceptance is established. Actual Godot getter/runtime smoke, complete raw traces and source reviews establish their stated behavior only. LSP transport was unavailable; compiler/Clippy and the full gate provide separate executable checks. Timing failures, the tuning fission miss, the held-out pairing miss, and the post-launch F3 correction are not waived by review.
