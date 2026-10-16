# Art Pass 9 independent review — round 1, preparation

Status: ROUND 1 INDEPENDENT REVIEW COMPLETE — ACCEPT EVIDENCE WITH PERFORMANCE FAILURES RETAINED. No confirmed production defect or evidence discrepancy requiring correction found. F1/F4/F5 PASS; F2/F3 FAIL; F6/F7 REPORT_ONLY. This is not full performance acceptance. Publication and documentation-only commit steps remain the leader's responsibility. Earlier pending statements below are dated progress checkpoints, superseded by the final sections.

## Independence and scope

Reviewed working source based on HEAD `034b8b1300efd344390a4187c7a4aa3842d71f78`, including uncommitted Part 1 implementation. Did not read GOAL sections 3–7, the original attachment, MIGRATION_PLAN, planner rationale, or parent history. Read READ_AUDIT and FFI_INVENTORY as permitted evidence, then checked relevant source independently. No builds, Godot sessions, foreign process operations, or code edits were performed. This document is the reviewer's only write path.

One graft repo-map call was used before source lookup; it reported approximately 1,857,527 tokens saved. GDScript inspection used direct source reads because the returned graph covered Rust/Python.

## Source findings

No prioritized production correction is requested at this checkpoint.

- Merge `49bc0b7c87159340b8024d6b9327bc394dc6daa0` has parents `fe185a23` and `da8e45c7`. Its scripts, scenes, and project.godot match original art `56059e0f`; the Rust delta against `da8e45c7` is confined to world_node.rs and static-source tests. The bridge delta is 404 added lines, zero deleted lines, preserving upstream implementation while adding art presentation getters.
- Current changes relative to the merge remain within the permitted source areas; unrelated .gitignore/.ignore changes are excluded. Rust implementation changes are confined to the causal getter/helper/tests in world_node.rs. `git diff --check` completed without errors.
- `scripts/ui/sim_reads.gd:22–40,65–81,103–109` validates sibling ownership and preserves uncached, mutable fake-world pass-through. `:83–125` caches ordinary getters by process frame and terrain by world lifetime. `:154–165` invalidates after mutations and immediately clears terrain for both reset methods; the connected deferred signal independently clears it again.
- `scenes/main.tscn` adds SimReads immediately after WorldSim, preserving reset connection order. No production protected direct getter/mutator call was found by the inspected searches; the static boundary test also scans game/test/support scripts, including dynamic dispatch.
- `sim_reads.gd:180–188` freezes nested dictionaries and ordinary arrays. Packed arrays remain governed by caller ownership. The HUD duplicates its compatibility dictionary (`hud_status_panel.gd:115`), and GodHandEffects replaces retained state on reset (`god_hand_effects.gd:65`). Source audit and the 600-frame consumer evidence did not expose packed-array caller writes.
- Terrain lifetime proof agrees with independent field-use search: writes occur in initialization/worldgen, including terrain.rs:127's swap, and fixtures. Runtime settlement/influence paths read terrain; no runtime terrain writer was found.
- `world_node.rs:1277` onward collects events in tile-input/ring order with first-seen ID deduplication. AgentRenderer preserves per-kind ingestion order and adds reset/overflow rearming (`agent_renderer.gd:326–401`). Synthetic tests exercise both cue types, ordering, reset, and overflow.

## Verified evidence and resolved limitations

- `gates/p1/reader-attempt1.log`: zero failures.
- `gates/p1/consumers-attempt1.log`: W0 and W500 each exercised for 600 frames, both zooms, open inspector/GodHand, enabled layers, and reset after retained cache; both report pass.
- `gates/p1/causal-attempt1.log`: 600 ticks, 8 recall ingestions, 0 combat ingestions, zero failures. Natural combat equivalence was not observed in that run; combat coverage is synthetic, not a claimed natural event sample.
- The earlier owner warnings in `gates/p1/equivalence-attempt2.log` are historical and **resolved**, not an open finding. Independently read `gates/p1-full/test_sim_reads_equivalence_p9.log`: 461 assertions, zero failures, no warnings. Its results.tsv row records exit 0 and 65 seconds. Current source unsets/restores owner around reader removal/reinsertion.
- At this checkpoint, `gates/p1-full-attempt2/results.tsv` contained 61 completed rows and provenance still said `running`. It is not classified as full PASS. Final exact-source/final-publication evidence remains separate.

## Independent A/A′ control recomputation

Read only `/Users/rexxa/work/art-production9-evidence/identity/attempt2/{A,Aprime}`. Used an independent Pillow RGBA difference, maximum over all four channels, and histogram count of nonzero pixels. No masking or generated comparison artifacts were used. All three pairs are 1920×1080, and their JSON sidecars match exactly.

| Framing | Differing pixels | Maximum channel difference | Identical A/A′ PNG SHA-256 |
|---|---:|---:|---|
| P1 | 0 | 0 | `9a75583e2c099bc5085dafd97fae1a1d9d2ced33673b6fd2fdc3df90a1a215e6` |
| P2 | 0 | 0 | `b97eca15b04dd38be8057ebfe2f964250625f006a7434a621f05db31d8c21dfb` |
| P3 | 0 | 0 | `69b4ab9ca4364196b274f8aaf7f688ffe0e0047f8e1f07f71710b77aa8a21459` |

These establish only the available baseline controls. They do not establish A/B equivalence or visual approval.

## Pending before final verdict

Independent block-median recomputation for each performance phase; F1–F7 verdict audit; quiet-lock/hold provenance; completed final publication gates bound to actual source; final allowed-path audit. These remain round 1 work when artifacts arrive. No complete independent review is claimed yet.

## Round 1 artifact continuation — six-framing identity

The implementation owner supplied source commit `a779e9270a54579e1e0c30c46fa3913100a7f784` and the completed `identity/attempt2/{A,Aprime,B,comparison}` evidence under `/Users/rexxa/work/art-production9-evidence`. The prior running-gate observation above is historical: the reviewer subsequently read all 70 results.tsv rows in p1-full-attempt2 and found zero nonzero exit statuses. This integration gate is separate from final publication checks.

Independently opened the full A/P1.png and B/P1.png pair, then the full A/P4.png and B/P4.png pair. P1 preserves the close village scene, river edge, residents, shelters/hearth/storage, top HUD, lower-left feed, and lower-right GodHand controls. P4 preserves the wide view with three settlement labels, terrain footprint and grey outside-map area, distributed residents/vegetation, and HUD/panels. No visible A/B change was observed in either pair. This is preservation approval for the optimization; pre-existing crowding and grey exterior in P4 are present in both images and are not newly introduced defects.

Read comparison/comparison.json and independently recomputed all six pairs using RGBA differences, including alpha, with no masks. Every image is 1920×1080. P1, P2, P3, P4, P5, and P6 each have A/A′ = 0 differing pixels / 0 maximum channel difference and A/B = 0 differing pixels / 0 maximum channel difference. These independent results agree with all six official comparison rows. F5 pixel identity is supported as PASS for these captured states. Timing/performance conclusions are not inferred from image equality.

No additional graft calls were needed; cumulative call count remains one and reported savings remain approximately 1,857,527 tokens.

## Round 1 continuation — baseline timing and derived caches

Reviewed baseline.json and the four original raw session files named in its sources list. Independently calculated the median of `process_ms + render_cpu_ms` from each first `frozen_Z4_0` block's 300 counted samples, excluding its 30 discarded warmup samples. Results agree with stored summaries to floating-point rounding:

| World | Independently recomputed first-block median (ms) |
|---|---:|
| W0 | 10.198000000059604 |
| W500 | 68.07550000619888 |
| W1 | 15.985999997496606 |
| W200 | 35.17900000566244 |

These are individual first-block medians, not pooled-condition values. All four sessions record debug build, `res://rust/target/debug/libsim_bridge.dylib`, forced drawing, no contention at admission, 1920×1080, VSync disabled, uncapped FPS, and restored camera/layers/VSync/probes. The four required worlds are present; W1000/W2000 remain explicitly unmeasured. GPU samples report unavailable and must not be interpreted as measured zero GPU cost.

Read with_quiet_lock_p9.sh and holds.md. Recorded baseline holds lasted 437/934/331/155 seconds for W0/W500/W1/W200, respectively, with admission loads below 4, empty competing-process lists, and exit 0. The ledger preserves cooldowns and waits. The separate p3-initial-W500 run is explicitly `contended=true` after waiting 3847 seconds, rather than being mislabeled quiet; the later p3-agent-W500 hold records `contended=false`. `contended=false` describes admission checks, not continuous proof that no external workload appeared during a hold: W0/W500 exit loads rose to 13.05/12.98. Final comparison should retain this environmental evidence. Completed final-after holds remain pending.

Reviewed the agent cache delta associated with `9f4a7eab541363f6e06869d63a0061f37e6b378e` and current building sleeper-count cache. No production correction identified. Agent base appearance depends only on ID, is made read-only, and elder adjustment duplicates before editing (`agent_activity.gd:76–82`); retirement/reset bounds and clears cache state. Talk deltas depend on ordered coordinates and index; coordinate copies detect same-frame packed-array mutation and preserve tie order. Per-tile contexts are limited to one process invocation. Building sleeper-count cache tracks every input used by `shelter_cold.gd:9–28`, including key presence, Variant type and packed-array content, plus world identity. It deep-copies keys and resets cache state on cold reset; frost fading and breath animation still execute each frame.

Regression source covers coordinate ordering/length/in-place edits, appearance-stage changes, retirement/reset, and all sleeper input key mutation/removal/type transitions/world replacement. The building regression log records PASS with 56 checks. These are targeted regression evidence, not a substitute for the final full gate after both optimizations.

Independently hash-checked every source_sha256 entry in `identity/p3-agent-attempt1/provenance.json` and `identity/p3-building-attempt1/provenance.json` against current files: all match. Independently recomputed baseline A versus each new B at full RGBA resolution for all six framings: both candidate sets have zero differing pixels and zero maximum channel difference in every framing. The supplementary provenance files provide source binding; the generic B manifest's empty git_sha alone would not. Thus identity evidence covers the currently reviewed agent/building cache implementations. Final after timings, F1–F7 synthesis and final full/publication gate remain pending.

## Round 1 continuation — NatureRenderer derived opacity cache

Reviewed NatureRenderer's current delta and nature_canopy.gd's complete dependency calculation. No production correction identified. CANOPY.alpha depends on head-bucket contents, tree position, and get_rect geometry; the cache compares those exact inputs. Head buckets are freshly allocated every invocation, so retained cache keys do not alias a mutable FFI snapshot. Agent movement/visibility changes clear cached geometry; tree position, offsets, regions and texture dimensions are covered by position/get_rect comparisons. Kind changes and removed sprites retire entries, and refresh/reconciliation clear them. Season tint is recomputed every invocation, including cache hits; only redundant identical modulate writes are skipped.

Read the regression source and `gates/p3-nature/final-cache-test.log`, which records 20 fresh-reference color comparisons passing, together with cache identity reuse/retirement/refresh assertions in the source. Coverage includes direct same-frame movement and visibility changes, tree geometry and kind changes, season change, insertion/removal and reset. This focused evidence does not replace the pending final full gate.

Independently checked all four current-file SHA-256 values in `identity/p3-nature-attempt1/provenance.json` (agent renderer, building renderer, nature renderer and nature regression): all match. Read the official comparison and independently recomputed all six baseline A versus nature candidate B RGBA pairs without masks. Each pair is 1920×1080 with zero differing pixels and zero maximum channel difference. This establishes exact captured-state preservation for the source-bound three-cache candidate. Final after measurements, F1–F7 synthesis and final publication gates remain pending. This remains the same round 1 review; graft count remains one.

## Round 1 continuation — partial final-after evidence

Independently read after-W0-attempt1.json, after-W500-attempt1.json and after-W1-attempt1.json in `/Users/rexxa/work/art-production9-evidence/perf-after`. Recomputed first-block `frozen_Z4_0` medians from process_ms plus render_cpu_ms (300 counted samples, 30 warmup samples excluded): W0 **4.048499995529651 ms**, W500 **51.437499997198586 ms**, W1 **9.039999995827674 ms**. These are block medians; W500's three-block pooled median is separately **51.5325 ms**, with nearest-rank p99 **53.021 ms** (rounded).

Checked every counted sample in those three sessions for matching ffi_counter_frame/process_frame and closed inspector/influence-overlay context. No mismatched frame or open context was found. All 900 W500 frozen Z4 samples contain exactly 11 total recorded getter calls, at most one per getter, and zero terrain calls after warmup. All three sessions record debug library/build, valid=true, empty errors, contended=false and restored controls. These are measured facts, not yet a final F1–F7 verdict.

Independently hashed all 94 entries in gates/final-source-manifest.json against current file bytes: all match. The manifest binds source commit `46193a0301d335fc04dcfd6460e59cc0f1370d08`; it is not itself gate acceptance. W200, A30, assembled final comparison and final full/publication checks are still pending at this checkpoint. Same round 1, no additional graft call or runtime launch.

## Round 1 continuation — final gate and binary provenance

Final source gate is now independently verified: `gates/final/results.tsv` has 73 rows, every exit status zero; session.exit is 0. Summing successful Rust test-result counts in cargo_test.log gives 2146. Results SHA-256 independently recomputes to `a0504297823f2272dd96f9155d390f42176d4fe64a1979e545f7f5b3859171a5`, matching provenance. Rechecked all 94 source-manifest entries against current bytes with zero mismatches. Gate source is `46193a0301d335fc04dcfd6460e59cc0f1370d08`; completion recorded at 2026-10-15 15:06:57 +09:00. Later documentation commits are covered through source-manifest equality, not an assertion that their future commit hashes were executed.

Both inspected bridge build logs (p1-full-attempt2 and final) explicitly show `dev` profile `[unoptimized + debuginfo]`. The final library is independently measured as 12,521,344 bytes with SHA-256 `b2ba6ef135c366862b5d016357d42353c5feb2493f39e9150c059ca974bae706`. Provenance preserves the pre-gate hash `69c336daf1d952bcb0ca40bf5faad3edf5f437611c000c4a4a7a1e6675a6239d`. Identical committed source, file size and profile do not prove byte-identical binaries or explain the difference. The byte-difference cause remains unestablished.

After W0/W500/W1 timings were captured with the earlier library hash; subsequent W200/A30 are scheduled with the post-gate hash. Keep both hashes explicitly associated with their sessions. Source/profile continuity supports comparing these as the stated build configuration, but the matrix is not one byte-identical binary run. Six final B captures against the new library and assembled F comparisons remain pending; earlier image equality is not silently transferred to the new binary. No reviewer runtime/build was launched.

## Round 1 continuation — F1–F6 threshold audit

Read the required-world after aggregate and preliminary comparison-pre-a30.json. Independently recomputed pooled primary-CPU statistics directly as process_ms plus render_cpu_ms; p99 uses nearest rank. Each row below uses 900 counted frames, and F3 filters running Z4 2× frames to ticks_run > 0 (all 900 selected frames carry ticks).

| Rule | Before median (ms) | After median (ms) | After p99 (ms) | Audited outcome |
|---|---:|---:|---:|---|
| F1 W0 frozen Z4 | 10.214000 | 3.968000 | 4.761000 | PASS: median ≤ 8 |
| F2 W500 frozen Z4 | 67.579500 | 51.532500 | 53.021000 | FAIL: median > 12 and p99 > 16.67 |
| F3 W500 running Z4 2×, tick-only | 68.298000 | 36.133500 | 168.294000 | FAIL: median > 16.67 |

F4 PASS agrees with the previously independently verified 900 frozen W500 samples: all total 11 actual counted getter calls, each getter at most once, zero warm terrain calls, aligned frames and closed inspector/overlay context. The ceiling is 20, and the reported baseline counts remain unmeasured rather than invented. F5 PASS is now independently re-established against the post-gate binary evidence: all 94 source hashes and all six recorded B PNG hashes match; all six A/B full RGBA comparisons again have zero changed pixels and zero maximum channel difference. The bound binary hash is b2ba6ef135c366862b5d016357d42353c5feb2493f39e9150c059ca974bae706.

F6 is REPORT_ONLY, with no qualifying measured population before or after. Independently recomputed W200 frozen Z4 median/p99 as 35.199000/37.522000 ms before and 29.690500/32.834000 ms after; W500 also misses the 12/16.67 rule. Actual frozen populations are 200 and 500. W1000/W2000 remain unmeasured, so no extrapolated capacity claim is justified. The after W200 session is explicitly contended: its hold waited 3658 seconds including gate/identity pause, admitted at load 14.90 with another worldsim probe present, and ended at load 6.92. Preserve this caveat alongside its measured result.

Independently listed the committed merge-base-to-final-source delta (`49bc0b7c` → `46193a03`): all 676 changed paths are within permitted script/scene/tool/production9-doc/TEST_LEDGER paths or narrow Rust bridge/static-test areas. No unexpected path was found. This does not include unrelated uncommitted ignore files. The preliminary F1–F6 outcomes match the inspected threshold implementation and recomputed evidence. F7 remains NOT_VERIFIED pending actual A30; the whole independent review is not yet complete.

## Final round 1 verdict — A30 and completion

Read both A30 raw frame_times_soc_layers.json files, their session metadata and exit records, and the final perf/comparison.json. Both raw runs are valid with empty errors, exactly twelve blocks of 50 frames, and 300 counted frames for each layer state. Independently recomputed medians from process_ms plus render_cpu_ms:

| A30 state | Before (ms) | After (ms) | Before/after speed-up |
|---|---:|---:|---:|
| Layers ON | 17.767999986171723 | 10.807500000715255 | 1.64404349× |
| Layers OFF | 15.96550000357628 | 9.759000007271766 | 1.63597705× |

These agree with the final F7 report. F7 is REPORT_ONLY, not PASS. Both session exits are 0; metadata records debug profile, non-fixed-FPS measurement and the respective baseline/final source commits. The after A30 hold ran 16:23:38–16:29:05 +09:00 with contended=true, wait 3798 seconds and load 6.59→7.82. This contention limits causal attribution of timing differences and remains part of the accepted evidence, not a hidden quiet-run claim.

Rechecked all 94 current source hashes once more with zero mismatches. The completed 73/73 final gate and 2146 Rust tests, source-bound post-gate six-framing RGBA identity, independently recomputed baseline/after blocks and pooled thresholds, allowed-path audit, and hold/binary provenance collectively support the recorded result. No outstanding reviewer correction request remains. F2/F3 performance targets were not achieved, F6 has no qualifying measured population, W1000/W2000 are unmeasured, after W200/A30 were contended, and the timing matrix spans two same-source/profile library hashes whose byte difference is unexplained. None of these limitations is waived or converted to PASS.

Round 1 is complete with the final classifications F1 PASS, F2 FAIL, F3 FAIL, F4 PASS, F5 PASS, F6 REPORT_ONLY, F7 REPORT_ONLY. No second review round was needed because no correction finding was raised. Approval here concerns correctness and faithful evidence reporting, not a claim that every performance target passed. Leader publication/doc-only finalization is outside this read-only review slice. Blind-review boundaries remained intact, and this reviewer changed only REVIEW.md. Graft total remains one call, approximately 1,857,527 reported tokens saved.
