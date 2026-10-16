# P5 independent evidence/provenance review — round 1

Reviewed 2026-10-16, working HEAD `6a3a2ee0865a39a5333bd08e9b2f230a226094b2`. Read GOAL in full and repository AGENTS; GOAL's explicit branch and no-harness override governs. Read-only audit: no Godot, cargo, gate, checkout, source edits, or children. Only this external review file was written. Scope is completed P0 matrix plus completed P3 references, not a candidate or final release verdict. Parts 3–5 are ongoing.

## Findings, severity ordered

No blocking evidence-integrity defect found in the completed matrix or reference artifacts examined.

1. **Low — merge identifier wording is imprecise.** `docs/art/production11/CHRONICLE_CONTRACT.md:3` says “after merge ecf512d8…”. That SHA is the one-parent contract/provenance commit, not the merge. Actual merge is `5cdf233409f11afdfde1134565730e21e8807a39`, with parents `cdf69c0e85a5876c821a8bf22b18089bf740c581` and `5cb16639b2334800aee9b907a7f3b26f0c657aaf`. State “inspected at ecf512d8 after merge 5cdf2334” in final provenance to avoid a misleading merge citation. Verified using native git parent records, not narrative summaries.
2. **Low — baseline heading describes optional W1000 as missing without its optional qualifier.** `docs/art/production11/perf/baseline.md:5` says “Missing worlds: W1000.” Raw aggregate correctly lists it in `optional_worlds_not_measured` and all four required worlds exist. Change the report wording to “Optional W1000 not measured” when preparing final tables; this is not a failed mandatory matrix.

## Independent raw recomputation

Inputs: `/Users/rexxa/work/art-production11-evidence/perf/baseline-aggregate.json` and each `baseline-{W0,W500,W1,W200}-attempt1.json`.

- Each aggregate session is exactly equal as a JSON value to its corresponding raw session; no merged or omitted session data.
- Independently sorted every block's raw sample arrays and recomputed median, nearest-rank p90/p99 and max for primary, process, render CPU, and ticks. Independently recomputed the same metrics for all pooled all-frame/tick-carrying groups. **Zero numeric mismatches** (tolerance 1e-9).
- Independently recomputed every ablation cost as the median of 300 adjacent all-on raw samples minus the median of the 150 off samples. **Zero mismatches**. Negative costs remain retained.
- Raw sample counts: W0 10,950; W500 10,950; W1 3,600; W200 3,600.

| World | Frozen Z4 median | Frozen Z4 p99 | Frozen Z1 median | Baseline interpretation only |
|---|---:|---:|---:|---|
| W0 | 2.8750000047 | 3.7549999952 | 2.8395000005 | H1 baseline meets bar |
| W1 | 5.8210000029 | 6.7640000058 | 5.9700000067 | context |
| W200 | 16.2799999996 | 17.8139999927 | 16.5115000056 | H2 baseline exceeds both bars |
| W500 | 28.9180000091 | 30.6749999950 | 28.8774999985 | H3 baseline exceeds bar |

Each frozen entry has 900 samples (3 × 300). W500 AgentRenderer ablation is 24.8000000025 ms; W0 is 1.5224999993 ms. These are before values, not final outcomes.

All 24 running blocks have the prescribed freshly restored start tick and population: W0 tick40560/pop29, W1 tick126994/pop66, W200 tick1440/pop200, W500 tick1440/pop500. The source reset sequence (`scripts/test/art/perf_frames_p11.gd:150–184`) independently confirms setup, speed0 advance, camera reframe and `_last_tick` restoration before each block. No negative ticks_run in samples or discarded warmups. Every stored population flag equals end < 0.9 × start. W500 2× is 500→392 in all three blocks, 4× is 500→363 in all three. **Baseline H4 has 0/3 eligible 2× blocks and must remain NOT_VERIFIED**, regardless of its displayed timing. Other worlds' running blocks maintain population.

## Release provenance and quiet record

All four raw sessions record release, checkout `/Users/rexxa/work/art-p11-measure`, baseline/build commit `b94bfef736f24f6f9e0567c39bb5d7ce5950916f`, and equal release/debug SHA256 `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`. Independently hashing both actual files during this review returned that same SHA; measure HEAD remained b94bfef7. Recorded collector SHA `e397d14f52a53405955c809dfc65ac5859feb5d48373e3ad559da0f85f853335` independently matches the committed collector blob at b94bfef7. The collector's release/hash refusal is at `perf_frames_p11.gd:71–84`; the runner also checks provenance before accepting sessions. No committed `sim_bridge.gdextension` diff from cdf69c0e to reviewed HEAD.

`docs/art/production11/perf/holds.md:15–78` supplies four complete matching holds: durations 109/499/259/96 seconds, all exit0, entry loads 3.59/2.89/3.77/2.66, empty foreign-process checks immediately before acquisition. Inter-hold gaps 3602/2101/2702 seconds exceed 600. All below the 3600-second hold cap. Reviewed six-capture Aprime and running-A launch intervals begin after the preceding hold ends; no overlap with these matrix holds. This verifies the recorded admission/launch evidence, not uninterrupted OS process surveillance throughout each hold. Section timer hold was still pending during review; P0 remains incomplete until timers are retained.

## Merge and chronicle contract

Git verifies the actual two-parent merge above and simulation launch head. `rust/crates/sim-bridge/src/ffi/chronicle.rs` and `docs/revitalize/CHRONICLE.md` remain byte-identical to the simulation parent at reviewed HEAD. All five methods are registered at `world_node.rs:552–588`. Source checks confirm packed i32 kind/cause, i64 remaining columns, found booleans, person sentinel fields, surviving person table, signed-ID translation, and paging max1..512/start clamp/next-total. Kind numbering 0..16 and DepartCause0..4 agree with `sim-core/src/chronicle.rs:18–132`. The documented additional found field, Joined/Left/SettlementDissolved causes, missing partner field, and absent leader roles for Settled/DepartureCancelled are appropriately distinguished from the planned shorthand.

## P3 references — not candidate acceptance

Source directory `/Users/rexxa/work/art-production11-evidence/p3-runtime/six-attempt2`:

- Independently decoded all six A and Aprime PNGs as RGBA and counted unequal pixels: **P1–P6 each 0**. Entire paired JSON values also equal, including complete manual/automatic render-event sequences. P5 is exactly reproducible; no water masking or pixel tolerance was needed for this control.
- Provenance pins Part2 final `f0d04f8e5bb698385b4751f0e0ea9f401ebef5ac` and debug bridge `2bcac5929643387e37e3bb724ffc7e447d47e44508f2f1794b2e7e561240889d`. The recorded automatic draw events are not silently discarded; their equality was checked along with manual events.

Source directory `/Users/rexxa/work/art-production11-evidence/p3-runtime/running-A`:

- Independently hashed W0, W200, W500_FROZEN, W500_2X JSONL dumps: each matches its metadata SHA.
- Actual line counts 600/600/240/240; frame numbers contiguous1..N; each raw tick equals its metadata tick array. All launch exits0 and provenance unchanged.
- Pan begins at hearth−(1200,0), camera_start (−184,500), then every raw frame position exactly matches start+(10×index,0). Frozen stays tick1440; 2× advances240 ticks to1680. Both use fixed60 and viewport-plus64 scope. W0/W200 running references advance300 ticks over600 frames.
- **No candidate B comparison exists in this review scope.** These reference successes do not establish H5 or any optimization acceptance. Candidate must still pass six B comparisons, W0/W200 running comparison and both pan comparisons.

## Art10 procedure and H truthfulness

Art10 after is **planned only / NOT_MEASURED**. Runner `run_perf_matrix_p11.sh:24–53` restricts historical mode to cdf69c0e/phase after, selects its p10 collector and p10 validator, W0/W500, and forwards the p10 provenance fields while using the p11 quiet lock. GOAL4.6 still requires the actual fresh debug build SHA and unchanged collector verification, W500 section timers, and separate historical debug tables. No Art11 release number can fill those cells. Use a separate raw evidence directory for historical sessions because phase-after filenames are shared with final P11 runs; `P11_OUTPUT_PREFIX` only changes the generated summary prefix at line207.

The current report draft `/Users/rexxa/work/art-production11-evidence/p5-report-prep/REPORT.md:55–76` correctly labels final release cells pending, H4 baseline NOT_VERIFIED, H7 NOT_VERIFIED and H8 FAIL, and Art10 NOT_MEASURED. H5's A/Aprime text is now stale-pending and can be updated to reference-repeat PASS while keeping overall H5 pending. I did not independently rescore H7/H8; their underlying blind/life review lanes are outside this evidence slice. No final FAIL is inferred merely from the ongoing Parts3–5 work.

## Remaining evidence to review at completion

P0 section timers; any accepted optimization's alternating pairs and candidate identity; frozen after matrix with flags and hashes; final H1–H8 and before/after section/ablation tables; optional Art10 actual procedure or explicit drop; final gate, branch publication and cleanup. Their present absence is an in-progress boundary, not a new defect or final pass/fail.
