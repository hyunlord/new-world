# Part 5 security / scope review — final code round 2, incremental cache

Baseline HEAD: `0635f950204b67d186017e3c07196e019ba03d37`.
Reviewed working-tree files and SHA-256:

- `scripts/ui/camp_seats.gd`: `fc4a1a6cb08fcacaa6faa59ed7dfef234ff6009fcf4dd0201839628db20ea97a`
- `scripts/ui/agent_renderer.gd`: `872ed4fe75713726718117e5bafb144de034435947ce3eae773b0cad352a1924`
- `scripts/test/art/test_declump_cache_p11.gd`: `07759d426eb5b59c773a9b2358033742040e022ce3e18a8335226f791a1a0d78`

Read `docs/art/production11/perf/declump-input-plan.md`, original static declump and transitive helpers, renderer call context, and mutation-history test. No runtime, Godot, cargo, gates, child agents, source edits or process actions. Only this external report written.

**PASS — incremental code/scope review. No concrete severity finding identified.**
**NOT_VERIFIED — executed tests, exact pixel identity, cache effectiveness, performance acceptance, final gate.** This result permits testing the candidate; it does not establish acceptance or justify claiming a speedup. The plan cites actual largest measured P0 declump cost of 6676.5 us; this reviewer did not independently rerun that measurement.

## Evidence

1. **Complete effective inputs:** `camp_seats.gd:51–58` records ordered scalar rows, full targets and rectangles, terrain dimensions/content, blocked membership dictionary, and spatial switch. Static `declump` at line 217 uses id/visible/moving/effective activity-or-pose/place-known only from rows; `_walking` at line 183 supplies exactly the projected walking predicate. `_declump_allowed` at line 282 uses captured dimensions/cells/blocked. `_spatial_cell_size` at line 355 scans all rectangles, including those not in rows; the key retains them. `_seated` is unused. No clock, node, tick, random state or external mutable state enters the pure computation.
2. **Aliasing:** lines 63–64 deep-copy container inputs and explicitly duplicate packed cells. Projected rows contain values, not sprite/node references. Lines 61 and 67 return independent output dictionaries; offsets are Vector2 values, so a shallow output copy suffices. Empty outputs use a separate valid flag. Nonfinite targets/rectangles clear and bypass the cache (lines 43–50).
3. **Lifecycle and controls:** lines 22–25 clear on enable-state change; lines 82–84 clear during reset. `agent_renderer.gd:78–86` introduces a P11-only named control and leaves P10 control implementation intact. Renderer world reset recreates CampSeats and reapplies P11 enable state at lines 408–409. Disabled calls directly execute original static declump and do not populate cache.
4. **Live upstream/downstream:** only the declump call is replaced at `agent_renderer.gd:684`. Seating, spread, current sprite/CarryLoad footprint construction still happen before it; easing, rounded positions, all metadata, selection outline and walker relief still happen afterward. Static declump and SpatialIndex bodies have no diff from baseline.
5. **Test scope:** the new test compares cached results to untouched static declump through input mutations, fresh equal containers, returned-output mutation, missing targets/rects, duplicate/reordered rows, terrain/blocked mutations, full footprint changes, spatial toggling, disable/re-enable, reset and empty output. Its renderer exercise checks easing changes and metadata/walker refresh on a cache hit while preserving P10 control. These are inspected assertions, not a claim of executed PASS. Nonfinite fallback is not explicitly exercised in this test, a small coverage limitation rather than an identified product defect.
6. **Allowed paths:** all three files fall under GOAL 2.1 `scripts/**`. No simulation-owned Rust, chronicle getter, gdextension, earlier evidence, or excluded ignore-file change is part of this patch. Local `.gitignore` and `.ignore` remain separate user state. Targeted `git diff --check -- scripts/ui/camp_seats.gd scripts/ui/agent_renderer.gd` returned no diagnostics.

Remaining risk: exact dictionary comparison/copy, full terrain snapshots and per-frame key construction may outweigh saved declump work or miss because current footprints change. Only the prescribed release pairs and identity evidence can settle that. Default enabled state must not be interpreted as an accepted candidate before those gates finish. Round 1's separate runner finding is outside this three-file incremental review; this report neither reopens nor clears it.
