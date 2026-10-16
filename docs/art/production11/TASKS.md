# Art Pass 11 execution ledger

Start: 2026-10-16T07:50:00Z (Asia/Seoul 16:50).
Part 4 freeze deadline: 2026-10-16T16:50:00Z. Overall cutoff: 2026-10-16T19:50:00Z.

| Part | Outcome | Start UTC | End UTC | Status |
|---|---|---|---|---|
| 0 | Merge, delivered contract, tools, gate, release baseline | 07:50 | 14:09:58 | COMPLETE; optional Art10 after dropped |
| 1 | Names, cached reads, life and records, chronicle, tests and captures | 10:17 (approx.) | 10:47 (approx.) | COMPLETE; H8 FAIL (44/45 correct) |
| 2 | Seven icons, village summary, blind test | 10:35 | 12:36 | COMPLETE_WITH_LIMITATIONS; H7 NOT_VERIFIED, C12/14 shortage, D round2 PASS |
| 3 | Timed optimization with identity and pan proof | 12:37 | 15:25:03 | COMPLETE; candidate REJECTED; no optimization adopted |
| 4 | Frozen release matrix and H1–H8 | 15:25:03 | | IN_PROGRESS; frozen5ee5e1ee |
| 5 | Independent review, final gate, report, publication, cleanup | | | PENDING |

## Initial provenance
- Art base and both remote art branches: cdf69c0e85a5876c821a8bf22b18089bf740c581.
- Simulation launch head: 5cb16639b2334800aee9b907a7f3b26f0c657aaf.
- Merge-tree: b5f091b962df586fa2f6cf8b92a040507eda1ae1; no conflicts.
- Local .gitignore and .ignore excluded from staging throughout.
- CARGO_BUILD_JOBS=6 and CARGO_PROFILE_TEST_OPT_LEVEL=2 for gate.
- 2026-10-16T10:01Z: Initial cargo_test completed exit0. Gate reports 7,695 wall seconds while its 1,800-second subprocess timeout did not fire; observed UTC jumped roughly two hours during the run. Treat this as host suspension/time discontinuity, not a quiet performance session. No timing claims derived from gate duration.
- P0 tool commits fe794805 and e68b851b pushed; ls-remote WIP=e68b851b26869671446b9542b72b077f2c01a1ef, publication remains cdf69c0e85a5876c821a8bf22b18089bf740c581.
- Measure worktree release bridge built, copied to debug load path; both SHA256=7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c. Initial import startup reported missing imported fonts; post-import confirmation clean. Four P11 collectors parse successfully.
- Timing correction: initial Part1 start was hand-entered10:22 ahead of the observed clock; corrected to approximate10:17. Verified code commit01fa24dd timestamp10:20:15Z; first capture launched10:20:28Z.
- 2026-10-16T13:14:42Z: All four required baseline worlds completed on b94bfef7, release/debug SHA equal. W200 hold96s, load2.66→2.89, unflagged200→200 in all six running blocks. P0 section timers remain queued; do not move measure checkout until they finish.
- 2026-10-16T13:10:41Z: First complete six-framing reference A finished, all six launches exit0 and clock metadata valid. P5 preliminary independent repeat has equal complete render event sequence and zero RGBA differences. Full A′ and running/pan reference are still in progress; no H5 pass claimed.
- 2026-10-16T13:24Z: Python unittest discovery completed392 tests in19.078s, exit0. This is a focused pre-freeze tool gate, not the full final gate.

- 2026-10-16T13:35:52Z: Reference capture sequence completed: six A/Aprime exact complete draw-event sequences and RGBA zero differences; running-A W0/W200600 frames and W500 frozen/2x240 frames pass structure/provenance validation. Reference evidence only; candidate H5 remains NOT_MEASURED.

- 2026-10-16T14:09:58Z: P0 W500 section capture valid300frames,14s quiet hold load3.24→3.78, uncontended, exit0. Largest section declump6676.5us, total24923us. Full baseline aggregate preserved byte-identical externally and in verified40-file ZIP; removed only redundant generated72.8MB working-tree JSON.

- 2026-10-16T14:14:52Z: Largest-timer declump-input-cache candidate implemented after written complete-input plan. Initial intentional RED retained; first green test fixture lacked a shadow for reset and was rejected despite exit0, then only fixture cleanup corrected. Strict3files/6FFI and six regression scripts now pass with no SCRIPT ERROR. Static section generator preserves ordinary statements. Independent source/scope review found no defect. Candidate speed/identity remain pending.

- 2026-10-16T14:42Z: Candidate1dc641b5 B capture completed with unchanged source/build/HEAD. All six full render-event sequences match A/A′ and all six RGBA comparisons have zero differing pixels/max-channel zero, including unmasked P5. W0/W200600-frame and W500 frozen/2x240-frame pan property comparisons all PASS. Alternating release pairs still queued; candidate acceptance remains pending speed evidence.

- 2026-10-16T15:24Z: Part3 declump-input-cache REJECTED. Three alternating150-frame pairs:39.1445→37.8680,63.8485→65.8330,64.7850→65.9935ms; pooled450 median62.8585000098→63.947ms (0.9829780914x). Hold15:20:34–15:21:51Z contended=true, load111.32→51.64. Structural validation and all identity checks PASS do not establish faster. Restore both production files byte-for-byte to d74f740 (same runtime as Part2 final f0d04f8e); remove candidate-only test after preserving committed source/evidence. No Part3 optimization accepted; stop further optimization to freeze and complete Part4.

- Freeze receipt: {"frozen_commit": "5ee5e1ee98df24a16756360b8b125d582f68fdd0", "freeze_utc": "2026-10-16T15:25:03.021297+00:00", "reference_commit": "f0d04f8e5bb698385b4751f0e0ea9f401ebef5ac", "runtime_paths": ["scripts/ui", "scripts/core", "scripts/rendering", "scenes", "shaders", "assets", "localization", "project.godot", "data", "rust"], "runtime_diff_empty": false, "candidate_rejected": "1dc641b5cd161cf9683349f367194935e8213b3f", "deadline_epoch": 1792180200, "world_order": ["W0", "W500", "W200", "W1"]}

- Freeze exact receipt15:25:03Z; preceding15:24:41 was the decision clock. Product sources equal Part2f0d after excluding five static-source Rust test repins, whose exact diff is retained. Part4 runtime remains5ee5e1ee although evidence/docs commits advance WIP.

- 2026-10-16T17:11Z: Turn interruption terminated W500 attempt1 while waiting (no hold or measurement); preserved empty log and explicit interrupted receipt, no invented exit status. Confirmed old runner absent and removed only its empty runner mutex. Resumed matrix from valid W0; next W500 attempt2, original deadline unchanged.
