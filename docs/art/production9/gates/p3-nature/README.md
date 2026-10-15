# NatureRenderer derived opacity cache

`before.log` records the new behavior regression against the unmodified renderer: exit 0, 20 exact color comparisons to fresh CANOPY.alpha and SEASON.resolve results. After implementation the same cases pass. `final-cache-test.log` additionally verifies unchanged-input cache-entry identity reuse, removed-tree retirement and refresh invalidation.

Every invocation still builds fresh buckets from visible agent sprites and resolves season tint. The cache compares bucket contents exactly, never hashes or ticks. Per-tree entries are keyed by object instance identity and compare live position and get_rect geometry. Only tree-kind sprites retain entries; removal or kind change retires them. Refresh and successful reconciliation clear geometry entries. Identical modulate writes are skipped; other writes remain unchanged.

Behavior cases: repeated unchanged inputs; same-frame agent movement and visibility; tree position, offset, region and texture-size changes; tree/shrub kind changes; season transition; insertion/removal; refresh/reset and explicit reconciliation. All calls are direct capture-style calls without relying on engine-frame advancement.

Final focused checks, all exit 0:

- test_nature_derived_cache_p9: 20 reference comparisons plus cache lifecycle assertions.
- test_canopy_fade: includes actual scene-order entry/exit.
- test_nature_density: failures=0.
- test_nature_clearance.
- gdscript_strict_check: 42 files clean, FFI checks pass.
- git diff --check on changed scripts.

No quiet lock existed at workload launches. Timing owner was notified after completion. No commit, full gate, pixel capture or performance measurement was performed by this lane; those remain leader integration requirements.
