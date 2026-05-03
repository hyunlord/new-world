# TPS Performance Optimization: 13 TPS → 20 TPS

Prompt saved for harness audit trail. See git history for implementation details.

## Results

### Profiling (Phase 1)
- PerfTracker extended with SystemStats cumulative tracking + report()
- 65 systems profiled across 20/100/200 agent counts
- story_sifter_system identified as O(n²) bottleneck (82% of time at 200 agents)
- territory_system identified as #2 (O(bands × entities) leader lookup)

### Optimizations Applied (Phase 2)
1. story_sifter tick_interval: 10 → 20 (notification cadence, non-critical)
2. sift_first_occurrence: O(total_events²) → O(total_events) via HashSet pre-computation
3. sift_relationship_triangle: O(n × r²) capped to 50 entities max scan
4. territory Step 4: O(bands × entities) → O(entities + bands) via position pre-lookup
5. SimEventType: added Hash derive for HashSet usage

### Performance Change (200 agents, 1000 ticks)
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Avg tick | 8.39ms | 4.55ms | -46% |
| TPS | 119 | 220 | +85% |
| story_sifter total | 9171ms | 3201ms | -65% |
| Total system time | 11207ms | 5239ms | -53% |
