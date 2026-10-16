# P5 evidence review — round 2 (OPEN for terminal metrics)

Initial snapshot: 2026-10-16 after17:10UTC resume; WIP068a97ad, frozen measurement build5ee5e1ee98df24a16756360b8b125d582f68fdd0. This is the second and final review round; later terminal evidence will be appended here, not opened as a third round. Read-only audit; only this external report is owned. No runtime/gate/build/checkout/source changes or child agents.

## Findings, severity ordered

1. **Medium documentation correction — historical BEFORE is incorrectly erased.** `docs/art/production11/perf/art10_after.md:7–9` marks the Before cells NOT_MEASURED, explaining “original final matrix had0 sessions”. The final/after matrix was unmeasured, but `docs/art/production10/perf/baseline.md`, `baseline.json`, and baseline-W0/W500 files preserve measured debug baseline data. Restore historical baseline before values with contended/profile provenance, or explicitly label these cells “original final matrix” rather than “Before”. Keep optional new Art10 after NOT_MEASURED; no rerun is requested.

No blocking defect found so far in candidate identity, archive integrity, source-equivalence, or completed W0 raw metrics. Remaining after worlds and terminal report are pending, not failed by absence.

## Candidate independently recomputed

Source: external p3-runtime/candidate1, six-attempt2 reference, candidate1-comparison, p3-pair-declump-input-cache and p3-candidate-package.

- Decoded six A/B PNGs as RGBA: P1–P6 each exactly0 unequal pixels. Full draw_clock objects equal for each A/B pairing; reference A/Aprime exact equality was independently established in R1.
- Parsed every JSONL row: W0/W200 running A/B arrays equal in entirety,600 frames each. Frozen and2× W500 pan arrays equal in entirety,240 frames each. This checks actual property dumps, not just comparator PASS strings. Both pan paths and the reference schedule were checked in R1.
- Candidate identity PASS belongs to candidate `1dc641b5cd161cf9683349f367194935e8213b3f`, not a new frozen-build runtime capture.
- Six alternating blocks are control/candidate/control/candidate/control/candidate with150 samples each. Independently summing raw process+render values gives pooled control median62.858500009775156ms, candidate63.947ms, ratio0.9829780913846646. All450 samples per mode are retained. Session contended=true (hold entry load111.32), so these are contended comparison values. Slower candidate is correctly REJECTED and not adopted; identity PASS does not imply speed acceptance.
- Recomputed SHA256 for all170 manifest paths: zero disk mismatches. Every manifest file exists inside raw-evidence-v2.zip with matching bytes/hash; archive has172 entries and CRC test passes. Separate reference archive128 entries also passes CRC. Historical first failed fixture/red-test evidence remains in manifest.

## Frozen-source equivalence

Independently ran git diff name-only from Part2 reference f0d04f8e to frozen5ee5e1ee across scripts/scenes/shaders/assets/localization/project.godot/data/rust/sim_bridge.gdextension. Exactly10 changed paths match source-equivalence.json: five Rust static-source tests and five capture/probe/clock harness files. No production path differs. This substantiates production-source byte equivalence after reverting the rejected optimization, with the qualification in the artifact preserved: source equivalence is not a fresh capture at frozen5ee. A final H5 claim must explain candidate runtime PASS and frozen source-equivalence separately rather than pretending those are identical provenance.

## Completed W0 after

Input: perf-after/after-W0-attempt1.json. Independently recomputed every block median/p90/p99/max for primary/process/render CPU/ticks; zero mismatches at1e-9 tolerance. Source has frozen5ee build, release profile and equal debug/release SHA7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c. Retained exit file0. Hold16:26:56–16:29:34,158s, contended=true, entry load9.89, exit load16.67.

| W0 frozen | Samples | Median | p90 | p99 | Max |
|---|---:|---:|---:|---:|---:|
| Z4 |900|3.3704999997|3.6129999969|4.2609999928|4.5729999995|
| Z1 |900|3.4605000058|3.7070000033|4.4509999881|4.6080000043|

H1 meets6ms threshold on this measured contended session; annotate contended rather than silently presenting it as quiet. Other after worlds are not yet available in this snapshot, and no missing-world result is manufactured.

## Abort/resume provenance

W500 attempt1 interrupted receipt records status INTERRUPTED_WHILE_WAITING, measurement_started=false, exit_code=null. Quiet ledger contains its wait checks but no HOLD_START; preserving no synthetic exit is correct. Existing W0 result is retained and resumed runner reuses it. p4-resume.sh preserves original P11_DEADLINE_EPOCH1792180200 (2026-10-16 19:50UTC), selected worlds W0/W500/W200/W1, and a distinct W500 attempt2. It does not reset the12hour contract. Final report must retain interrupted receipt and distinguish abort from failed performance.

## Deadline correction source/test audit

Current with_quiet_lock_p11.sh retains deadline checks during wait, before mkdir and immediately after ownership; an admitted hold no longer truncates its own limit to task-deadline remaining time. Each admitted session still has max3600s wrapper/watchdog. run_agent_sections_p11.py:145 allows remaining+3600+10 for the outer parent, permitting one already-admitted hold and cleanup. Matrix guards reject fresh attempts and split fragments after cutoff. Resume script has no outer hard-cut wrapper that would negate the admitted-session tail.

This matches GOAL0.6's finish-current-step interpretation and retains original admission cutoff. It does not allow a queued whole four-world matrix to count as a single admitted step, nor authorize a new section hold after cutoff. On terminal cutoff, missing worlds/timers must remain explicitly unmeasured.

Read test_deadline_p11.py and retained deadline-tests-transcript.md: five synthetic cases cover expired admission, foreign-lock preservation, admitted child finishing after cutoff, hold-cap termination, and AST verification of section outer timeout. The transcript records two expected RED failures then five GREEN tests. I did not rerun them because this audit authorizes no runtime launches. The tests isolate temporary lock/tree with fake load/process probes; no actual Mac lock or Godot. Coverage does not explicitly exercise post-mkdir clock race, cooldown expiry or multiworld/split cutoff orchestration; those paths were source-reviewed, not claimed dynamically tested.

## Pending supplement within this same round

Remaining after raw sessions and any section timers, aggregate arithmetic/H4 eligibility, latest hold chronology, corrected historical-before table, final Korean report distinctions and final gate/publication evidence. Keep this review OPEN until root provides terminal artifacts.

### Same-round correction verification

Historical-before finding addressed by the assigned documentation owner. Independently reopened historical `docs/art/production10/perf/baseline.json` and recomputed process+render medians: W0Z4 4.284500003457069, W0Z1 4.414999997675419, W500Z4 51.387499997138974, W500Z1 51.674499999999995, each900 samples. These agree with the corrected P11 historical table. After remains NOT_MEASURED/dropped; original historical evidence is not edited. Finding1 resolved for that table, pending final report integration.

### Same-round W500 terminal supplement

Independently read `perf-after/after-W500-attempt2.json` and terminal `.exit=0`. Every block median/p90/p99/max for primary/process/render/ticks and every adjacent-on/off ablation cost recomputed from raw samples: zero mismatches (1e-9). Frozen build5ee5e1ee, release, equal loaded/release hashes, contended=false. Hold17:45:48–17:54:19, entry load3.62→exit5.51; after-exit load does not retroactively change admission contended flag.

| W500 frozen | Frames | Median | p90 | p99 | Max |
|---|---:|---:|---:|---:|---:|
| Z4 |900|28.9335000030|29.9629999964|31.0020000114|31.7939999862|
| Z1 |900|28.7549999975|29.5289999866|30.2449999890|31.1889999878|

H3 FAIL:28.9335>16.67. All three2× blocks freshly start tick1440/pop500 and end392; all three4× end363. Flags are correct by end<90%start. No negative tick deltas, including discarded samples. **H4 NOT_VERIFIED with0/3 eligible2× blocks**, never PASS/FAIL from their timing. W500 AgentRenderer after cost independently24.7665000017ms versus baseline24.8000000025ms; no acceptance bar applies to H6. Interrupted attempt1 stays excluded and retained. W200/W1/after-section timer still pending; R2 remains OPEN.

### Same-round W200 terminal supplement

Raw `perf-after/after-W200-attempt1.json`, terminal exit0: independently recomputed all block median/p90/p99/max for primary/process/render/ticks, zero mismatches. Frozen5ee release, loaded/release hashes equal, contended=false. Hold18:04:19–18:05:56, load3.91→5.48. Six fresh running setups start tick1440/pop200, end200, all unflagged.

| W200 frozen | Frames | Median | p90 | p99 | Max |
|---|---:|---:|---:|---:|---:|
| Z4 |900|16.3175000050|16.8840000029|17.5150000043|17.9130000037|
| Z1 |900|16.4459999955|16.9759999967|17.6459999955|18.2850000029|

H2 FAIL on both required bounds: median>12 and p99>16.67. W1/after-section terminal evidence remains pending; same R2 stays OPEN.

### Same-round completed matrix / W1 supplement

Independent verification of `docs/art/production11/perf/after.json`: every aggregate session exactly equals its accepted raw JSON (W0attempt1, W500attempt2, W200attempt1, W1attempt1). Recomputed all pooled all-frame/tick-carrying median/p90/p99/max across every world for primary/process/render/ticks: zero mismatches. Interrupted W500attempt1 is not substituted or counted.

W1's individual block statistics independently match all raw samples. Frozen5ee release profile, equal dylib hashes, contended=true. Six running blocks freshly start126994/pop66, end66, all unflagged. Hold ends19:10:44,286s, load7.33→5.71, so report this as contended. W1 is contextual and has no dedicated H threshold.

| W1 frozen | Frames | Median | p90 | p99 | Max |
|---|---:|---:|---:|---:|---:|
| Z4 |900|6.7769999998|7.7479999990|9.2220000067|10.2509999881|
| Z1 |900|6.5744999985|7.6819999976|9.0910000024|9.8619999785|

All four required after worlds now have valid terminal raw data. Optional W1000 remains unmeasured. After-section timer admission is still queued against original19:50 cutoff; final report/package and section terminal status remain to audit in this same OPEN round.

### Same-round deadline terminal / complete hold ledger

After-section session is terminal: `p4-sections-W500-attempt1/session.exit=124`, outer `p4-sections-driver.exit=1`, summary valid=false with empty section_statistics_usec, and no frames.json. The exact quiet ledger entry is19:50:00 TASK_DEADLINE for pid3770; this pid never acquired HOLD_START. Therefore section after is **NOT_MEASURED (admission deadline while waiting)**, not a measured slow result or failed performance threshold. Keep failed session receipt/summary; do not synthesize section numbers from unchanged production source or baseline. This does not invalidate the four completed matrix sessions.

Independently parsed all10 completed HOLD_START/HOLD_END pairs. Durations109/499/259/96/14/77/158/511/97/286s are all≤3600. Every subsequent inter-hold gap≥600s; exactly600s before after-W200, others3302/4236/3905/4574/3602s in the later sequence. No HOLD_START exists for interrupted W500attempt1 or deadline-expired after sections. Contended admission flags are retained for pair, afterW0 and afterW1; other completed holds uncontended. W200 duration is97s by epoch arithmetic (18:04:19→18:05:56), correcting any96s narrative shorthand. Earlier baselineW200 was96s; do not conflate them.

Final archive was not yet present at this check; report/package integration remains OPEN within R2.

## R2 closure — metrics, package and report

Status: **CLOSED for assigned evidence/provenance and metrics/document review; no blocking findings remain in this slice.** Final gate, exact published heads, PR update and cleanup remain root-owned PENDING work and are not certified by this review. No third review round was opened.

Independently verified installed `docs/art/production11/perf/final-evidence/raw-evidence.zip`:19,876,781bytes, SHA256 `e8deeca2f63761e364704a5a27d26f4254bf18166e829fc708218ef675862c40`; CRC clean;57 ZIP entries containing55 manifested payloads. Every payload's bytes/size/SHA matches manifest and every extant external source file. Manifest SHA `60e7653e022de1ec0800eb0c6949aa73d510c8fb9851f795360cfad0561b4507` matches manifest.sha256. Both aggregate schemas, accepted raw sessions, interrupted W500 receipt and failed section terminal evidence are retained. Plain after.json was removed during this audit; archived after aggregate hash is `eec3447b19883fabc4d11e57d6ac071343eb28ee5e7a4f0a8826522fee20a883`, and its accepted sessions had already been independently compared exactly with raw files before removal.

Parsed all16 condition rows in installed REPORT.md and compared all before/after median/p90/p99/max strings with archived aggregate values at4decimals: **zero mismatches**. H1PASS, H2/H3FAIL, H4NOT_VERIFIED0/3 and H6report-only agree with independent computations. Historical-before correction is integrated; optional Art10 after stays unmeasured. Contended afterW0/W1 and candidate results are explicit; ratios are not called optimization gains. H5 correctly separates rejected-candidate runtime PASS from reverted frozen-source equivalence and states no fresh5eeB capture. Section after is unmeasured, not fabricated zero. H7/H8 limitations and TIMEBOXED_PARTIAL remain visible; their separate review-lane scores were not re-scored in this slice.

REPORT currently leaves final gate, branches/ls-remote, PR and cleanup PENDING rather than claiming premature success. Root must replace those slots with actual terminal evidence and update commit lists after publication. Such pending delivery receipts do not change the completed numeric review. The report's package-verification-pending prose can now be replaced with the verified archive hash/size above. Earlier process-surveillance limitations remain as stated in quiet-audit R1; successful archive validation does not strengthen them.
