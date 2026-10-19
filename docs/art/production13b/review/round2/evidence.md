# Art Pass 13b independent review — ROUND2 evidence lane

**Status: FINAL ROUND2 COMPLETE; terminal closure below supersedes historical PENDING notes. Technical publication eligible under GOAL §8.2; overall acceptance remains partial, not all-pass.** Reviewed frozen source `40aa7a463d740b776248828211251187319c6af6`. Read production13b/GOAL.md fully; reviewed prior round findings/remediation and original evidence. No runtime, Godot, builds, tests, probes, source edits, commits or delegation. Only this external report is written.

## Current findings and disposition

- **No newly established P1/P2 defect in the completed Parts0–3 evidence inspected.** This does not close the overall gate: H8 remains NOT_VERIFIED (only12/14 eligible crops), H10 remains FAIL (road porter3/3 failed), and Part4 remains pending; final technical gate completed below. These failed/incomplete mandatory acceptance targets block an overall-green claim. Technical publication eligibility is governed separately by GOAL §8.2: a clean final technical gate can permit the branch move while these H verdicts remain failed/incomplete. Do not rerun exhausted visual targets.
- Prior round P2 stale retry census is substantially remediated by `docs/art/production13b/retries.md` and JSON, including terminal Part3 updates. Final Part4/final-gate invocations still must be appended before closure. Stage-B denominator ambiguity is now explicitly disclosed rather than declared compliant. No blanket global three-strike certification is supportable.
- Prior round P3 legend-first provenance limitation remains honestly disclosed: original timestamped acknowledgment NOT_AVAILABLE. TASKS narrative supports the intended order; freeze hashes establish answer integrity but cannot independently prove the earlier legend acknowledgment. This is not evidence of leakage and is not repaired by inventing a rerun.

## Independently inspected evidence

### Font / glyph

`agent_inspector_panel.gd:166–167` sets normal/bold24. Read all four all-pairs JSON arrays under `glyph/attempt1`:10442 comparisons (4186+1035+4186+1035), minima EN regular3, KO regular1, EN bold6, KO bold2 design pixels. Report separately records measured24px differences12/4/24/8;24 is2×12. Machine distinctness passes, not a guarantee of error-free name reading. No atlas rerender was performed.

### Merged technical gate / real input

Independently read `gates/p3-attempt3/results.tsv`:120 rows, all exit0; searched its top-level logs and found zero ERROR. Verified all17 original PNG SHA256 values and passed states in `real-input.J6t6BN/capture/result.json`; windowed=true, input route `Input.parse_input_event`, errors empty. Personally viewed original `12_tooltip_after_delay.png`, `13_tooltip_mouse_out.png`, `14_tooltip_no_activity.png`, `09_language_korean.png`: “Hubo — Resting by the fire” appears in the show frame and is absent in both hiding frames; Korean inspector is visibly rendered. The gate's bridge hash is explicitly a separate test-build identity, not claimed to be the measured release bridge. No runtime was relaunched.

### Candidate identity / paired speed

Read merged `walker-identity-merged/comparison-attempt1/walker-identity.json`: compound PASS, errors empty, RGBA/running reference PASS; verified all38 hashed input artifacts against actual files. It records six still framings and five running/pan schedules, expected4560 frame stamps plus10 final stamps, and18 still capture stamps. This is required post-merge replay, not another timing run.

The accepted walker speed evidence retains three alternating pairs each: W2000.6472445182/0.6466932538/0.6541020268, pooled0.6492237648; W5000.9499485291/0.9436013447/0.9386296695, pooled0.9446215390. Both3/3 faster, with W200≤0.97 and W500≤1.00. Read `walker/performance/comparison-attempt3/walker-pairs.md` and the merged compound record together; its older standalone “identity required” text is not an overall current rejection. Raw medians were not independently recomputed in this round.

Write skip correctly remains OFF: W200 pooled0.9986985165 fails0.97 even though2/3 pairs faster; W500 attempt3 pooled0.9472853932 and3/3 faster. `perf/write-skip/REPORT.md` preserves failed collector1 and pre-Godot denial2. No speed rerun to seek a better outcome is justified.

### Provenance qualification

Read external `after/identity-source-equivalence.md`: full manifests are NOT identical.655 untracked paths differ:183 generated `.gd.uid`,471 import UID fields, and ignored measurement Cargo.lock. All471 compiled imported resources and tracked runtime sources are equal. Measured library is root release copy SHA256 `30ed97544d8fab39146c612c0cf87cd4b59245f427bfba5896fe188e9c6ad042`, also used by merged identity; the differing measure lockfile did not build it. Only this qualified source/resource/loaded-library equivalence is supportable, not complete-manifest identity. Detailed differencesJSON and stable local measurement snapshots must remain with final evidence.

### Stage B / contrast

Inspected source `prepare_blind_crops_p13b.py:191–217`: exclusions written before crop creation; preserved ordered initial sources and supplement time constraints. Independently verified12 crop hashes and legend-reader answer hash against the freeze. Terminal inventory[1,2,1,2,2,2,2] yields12/14. Legend diagnostic10/12 with Repair0/1; without legend9/12 with Repair0/1,Craft0/2. These cannot satisfy full≥12/14 plus no kind0/2. Personally viewed raw native anonymous crop02; body and two overlapping badge regions are visible, illustrating why a real crop cannot be replaced with a clean icon atlas. Full recognition PASS is not claimed. Contrast remains partial12/21 cells; no unmeasured cell is credited and proxy adjacent backgrounds must remain identified.

### H10 actual screenshots

Personally viewed retained `h10/porter_road_z4_ko.png`, `relations_z1_en.png`, `chronicle_ko_page_000.png`. Road has overlapping bodies/badges around the designated subject; it does not clearly establish the required carrier glyph plus pose. Z1 visibly contains green/orange solid relation lines and red dashes. KO chronicle visibly says villages1/2 made peace at day86. Report's H10 FAIL is correct; peace/relations cannot compensate for porter3/3 failures. These are debug capture evidence, not release performance. This round did not re-view all six language variants or re-adjudicate every sidecar.

### Fact / name split

Verified all161 original PNG hashes from the three private maps119/6/36 and all three frozen answer hashes. Read four-slice aggregate and name confusion ledger. Independently verifiable non-name correct counts665/595/117/624 have zero recorded wrong; screen-only127/4/14/68, abstentions and motion inferences remain excluded. Names1383 occurrences =1323 exact+2 partial+56 wrong+2 alternatives. Heterogeneous fact units are not pooled; name errors are not silently removed from their own denominator. This review checks integrity and reporting consistency, not a second full blind adjudication of every claim.

### Retries / allowed paths

Current completed census: road3 terminalFAIL; estranged3 with finalPASS; real-input-p13 five invocations (one failure, targeted pass, three Part3 passes); walker identity historical1FAIL/2PASS plus one required mergedPASS; bridge2, cargo2, numeric1, fixture1; speed runtime one per world, offline comparator3. Full Part3 gate1FAIL, gate2 exit-green/log-errorFAIL, gate3 cleanPASS are all retained. Write-skip W500 uses three admission slots but only two collectors; W200 one. Stage B21 acquisitions =6 initial generic+15 supplements. The prospective target definition is ambiguous: counting initial Z4 scenes against each kind gives six exposures per supplemented kind. No global compliance assertion is made.

Reviewed committed changed paths and18 subjects from `8c1b18be` through frozen40aa7a46: all subjects use `art-p13b(pN):`; paths remain production13b docs, scripts, art_pipeline tools, art-owned Rust wrappers/kernel and fixtures. No earlier evidence/owner feedback/ignore files or simulation-owned paths appear in that committed diff. Semantic Rust review belongs to the parallel code lane.

## Part4 closure still pending

At assignment, W0/W500 attempt1 were denied before Godot (load>8); these are admission attempts, not valid after samples or performance failures. Other targets ongoing. No current incomplete after result is accepted. Await terminal matrix/timers, exact attempt census, before/after table with calibration caveats, final gate and frozen-source checks. Final metrics and P1/P2 closure will be appended to this SAME ROUND2 report after leader supplies terminal evidence.

## Same-round final-gate update

Independently inspected `docs/art/production13b/gates/final/results.tsv`:120 rows, all exit0; zero ERROR in top-level logs. Leader reports terminal process gone and frozen40aa7a46 source unchanged. Technical gate PASS is separate from overall H8/H10 acceptance. Under GOAL §8.2 technical publication is eligible on this clean gate; this review does not label the whole art gate green or claim a branch move occurred. Part4 attempt2 is still active; no final after metrics accepted yet. Leader reports union admission wait5886.58s including useful work in cooldown; terminal ledger still to inspect.

## ROUND2 terminal closure — supersedes PENDING above

**FINAL ROUND2 COMPLETE. No outstanding P1/P2 reporting defect found in the inspected terminal evidence. Overall result remains PARTIAL / NOT_VERIFIED with explicit failed targets; this is not an all-pass verdict.** The earlier pending paragraphs are historical within this same review round. No third review or extra runtime was launched.

Read `part4-terminal-audit.json`, current `docs/art/production13b/perf/after.json`, terminal report-prep comparison/verification/disposition, final aggregation and final archive. The official after summary is `valid:false` and selects exactly W0/W1/W200/W500 valid sessions; it does not accept either W-multi raw capture. W1000 is optional/unmeasured. Raw aggregation deliberately retains seven captures, including three invalid ones; its raw-shape/capture fields do not supersede admission. The initially stale four-world summary tail was corrected during this same round to terminal NOT_VERIFIED and28 probed attempts/56 real endpoints plus1 policy denial.

Independently recomputed pooled primary CPU medians and nearest-rank p99 directly from all four selected raw files (six blocks each,900 samples per frozen/running condition). Results match official summaries:

| World | Frozen median ms | Running2× median ms | Running p99 ms |
|---|---:|---:|---:|
| W0 attempt3 |3.1880000022|5.2619999985|5.8619999965|
| W1 attempt1 |6.1060000035|12.0500000002|13.4150000044|
| W200 attempt1 |11.2804999961|17.0394999996|19.7849999945|
| W500 attempt3 |23.3245000007|41.2404999933|44.7710000119|

All four raw files declare release SHA30ed97544d8fab39146c612c0cf87cd4b59245f427bfba5896fe188e9c6ad042 and their validity sidecars are true. H1 W0 frozen3.188<6 PASS; H2 W1 running p99 13.415<16.67 PASS. H3 W200 running17.0395>16.67 FAIL (before26.145, ratio0.6517307324); H4 W200 frozen11.2805>10 FAIL (before16.2575, ratio0.6938643701); H5 W500 frozen23.3245>16.67 FAIL (before28.4035, ratio0.8211840090). Historical W500 running was population-loss-excluded, so41.2405 receives no invented before ratio. Cross-session ratios remain unnormalized descriptive comparisons with R before2.008996167 versus current2.762010041; paired walker gains are separate stronger candidate evidence. No R normalization rescues failures.

W-multi, required W200/W-multi running timers, optional W500 timer and W200 ablation remain terminal NOT_VERIFIED. Thus final exclusive walker/seats section-cost reductions are not measured; paired kernel ratios cannot substitute for missing H11 after timers.

### Terminal attempts and stopping reason

Independently counted29 audit rows:4 VALID,3 COLLECTED_INVALID,21 PRELAUNCH_REFUSAL,1 POLICY_REFUSAL_WRONG_INHERITED_ITEM. All29 validity files agree with the audit valid flags. There were seven actual collectors: W0 two, W1 one, W200 one, W500 one, W-multi two. W-multi's two collected attempts failed end-load admission; no section/ablation collector launched. They did not exhaust three executed failures. The exported P13_ITEM error produced W-multi file4 denial before probes; file5 after explicit unset is its fourth true admitted opportunity and was refused at load17.5810546875. Neither file is another actual collector.

Independently summed wait intervals to9487.316187858582s and checked admitted event spacing: no interval below1800s. The remaining1312.6838121414185s cannot accommodate the next required1800s cooldown within the10800s cap. The stop is correct as a wait-budget stop, not an artificial three-strike claim. Current retries.md retains the complete terminal paragraph and prior snapshots; final real-input-p13 count is six, including the final technical gate. Broader Stage-B target-definition ambiguity still precludes blanket global three-strike certification.

### Archive and closure

Independently hashed and size-checked all280 records in `perf/after-evidence/SHA256-MANIFEST.json`:280 external sources plus269 repository copies =549 file checks, zero mismatches. Eleven large records remain external-only, explicitly identified. Terminal controls, wrong-item refusal, recovery, invalid originals and earlier report snapshots remain retained. Final source qualification continues to reject complete identity/measure-manifest equality while preserving tracked-source/imported-byte/loaded-bridge equivalence and stable local snapshots.

The final technical gate120/120 zeroERROR was independently checked earlier in this same round. Under GOAL §8.2 it provides technical publication eligibility; this report does not claim the branch was moved or approve an overall-green label. H8 NOT_VERIFIED, H10 FAIL, H3/H4/H5 FAIL and missing Part4 timers remain mandatory disclosures. No remaining code/runtime repair or further measurement is requested by this evidence review. Publication operation, branch-head verification, PR update, worktree removal and final report integration remain leader-owned delivery actions, outside this evidence-lane completion claim.
