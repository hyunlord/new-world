---
feature: next-c-stage1-phase2-determinism
code_attempt: 1
---

## Files Changed
- `rust/crates/sim-test/tests/harness_phase2_determinism.rs`: 13-test determinism harness (test-only commit — no production code modified)

## Observed Values (seed 42, 20 agents — N/A for this harness)
These tests construct bare `SimEngine::new(32, 32, MaterialRegistry::new())` and call systems
directly; seed/agent_count do not apply. Observed values per plan assertion:

- A1 FIFO Warmth regions: `[(4,4,6,6),(8,8,12,12),(17,12,23,18),(24,24,26,26)]` — exact match
- A2 all 4 STAMPED channels byte-identical: 3 regions each, all channels equal within and across runs
- A3 clamp+OOB: queue_len=0, 3 valid regions `[(2,2,8,8),(0,0,31,31),(31,31,31,31)]` — exact match
- A4 IUS clear+swap: all channel current buffers == 0 after 30 ticks with seeded pending
- A5 viz digest: tick=12, warmth_total=100, danger_peak=200 — exact match
- B1 propagate_bfs: N=5 instances byte-identical, at least one non-zero tile
- B2 propagate_shadowcast: N=5 byte-identical, at least one lit tile
- B3 propagate_noise: N=5 byte-identical, at least one non-zero tile
- B4 stamp_social_aggregate LOD filter: Simplified/Dormant tiles=0, Full/Medium tiles=1 at center
- B5 blocking cache lookup: all keys return 0.0f32 (0x00000000 bit pattern), N=5 identical
- B6 EC-2 source self-blocking: source tile=200 despite wall material placed on it
- B7 EC-3 Additive Warmth midpoint: warmth_mid=150 (75+75), Light midpoint≤100 (Max)
- B8 propagate_danger: N=5 byte-identical, at least one non-zero tile

## Threshold Compliance
- A1 (queue_fifo_drain_order): plan=`[(4,4,6,6),(8,8,12,12),(17,12,23,18),(24,24,26,26)]`, observed=exact match, PASS
- A2 (dirty_regions_content_byte_identical): plan=3 regions per channel, all channels equal, observed=PASS
- A3 (bss_clamp_oob_deterministic): plan=`[(2,2,8,8),(0,0,31,31),(31,31,31,31)]`, observed=exact match, PASS
- A4 (ius_clear_swap_idempotent_30_ticks): plan=all bytes==0, observed=all bytes==0, PASS
- A5 (viz_digest_tick_boundary_6): plan=tick==12,warmth_total==100,danger_peak==200, observed=exact match, PASS
- B1 (propagate_bfs_byte_identical): plan=N=5 identical + any>0, observed=PASS
- B2 (propagate_shadowcast_byte_identical): plan=N=5 identical + any>0, observed=PASS
- B3 (propagate_noise_byte_identical): plan=N=5 identical + any>0, observed=PASS
- B4 (stamp_social_aggregate_lod_filter): plan=Simplified/Dormant=0, Full/Medium=1, observed=PASS
- B5 (blocking_cache_lookup_byte_identical): plan=0.0f32 bit pattern, N=5 identical, observed=PASS
- B6 (ec2_source_self_blocking_exempt): plan=source==200, observed=200, PASS
- B7 (ec3_multi_source_additive_vs_max): plan=warmth_mid==150, Light≤100, observed=PASS
- B8 (propagate_danger_byte_identical): plan=N=5 identical + any>0, observed=PASS

## Gate Result
- cargo test: PASS (257 passed, 0 failed across all workspace crates)
- clippy: PASS (clean, no warnings)
- harness: PASS (13/13 passed)

## Notes
- This is a **test-only commit**. No production code was modified (`sim-core`, `sim-systems`,
  `sim-engine`, `sim-bridge` sources are unchanged).
- The test file `harness_phase2_determinism.rs` was pre-written as part of plan generation and
  existed at the start of this code attempt. All 13 tests went GREEN immediately against the
  existing production code — confirming the plan's R1 rationale: the tested surfaces (queue drain,
  dirty_regions, IUS clear+swap, viz digest, primitive direct paths) are fully functional today.
- B6 (EC-2): as documented in the plan, `MaterialBlockingCache::empty()` means 0.0 blocking
  everywhere so the assert `== 200` passes trivially. The deeper EC-2 invariant (wall material
  with non-zero blocking coefficient on source tile still exempt) is covered by the unit test
  `test_bfs_source_self_blocking_exempt` in `propagate.rs` which uses a registry-backed cache.
- No threshold discrepancies observed. All assertions match plan thresholds exactly.
