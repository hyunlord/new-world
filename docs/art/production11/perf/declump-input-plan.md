# Conditional declump input memoization safety sketch

2026-10-16, read-only source inspection. **Selected for one candidate experiment at2026-10-16T14:10Z.** Verified P0 section capture shows declump6676.5us, the largest bucket above1ms (pose2550us next), with total24923us. Implementation and acceptance remain pending. No predicted speedup. Key comparison/copy costs and animated footprint changes may erase any benefit; measure actual hit/miss counts and paired release performance if selected.

## Pure computation boundary

`camp_seats.gd:161–213` static `declump` reads its arguments and allocates a new result. Its transitive helpers are `_walking` (128), `_covered` (215), `_declump_allowed` (226), `_spatial_cell_size` (299) and `SpatialIndex` construction/insertion/query/overlap (306–365). They do not read simulation tick, time, camera, nodes, or global mutable state. SpatialIndex is local to each invocation. `agent_renderer.gd:671` is the sole production call, after live seating, spreading, target construction, and every footprint/CarryLoad merge (632–670).

This is memoization of a pure function's **complete current inputs**, not the rejected tick-key cache. It must remain correct when inputs change within one tick and may reuse only when exact inputs match across ticks. Do not replace any upstream/downstream work, alter static `declump`, or reuse offsets based on tick/agent count/visibility alone.

## Exact conservative key

| Input | Required captured value / reason |
|---|---|
| Ordered rows | For every row in original order: `int(id)`, effective `bool(visible,false)`, effective `_walking(row)`, effective `bool(activity_place_known,false)`. `_walking` means `bool(moving,true)` OR `str(activity, fallback pose)` in walk/run/carry/carry_wood. Preserve defaults and fallback exactly. Retaining row order avoids assuming duplicate-ID/tie sorting behavior; production unique IDs do not authorize collapsing rows. |
| All targets | Deep value snapshot of the entire dictionary, including missing/present keys, all Vector2 components and unused entries. Visible filtering requires target membership; descending y/id ordering, nearest-front squared distances and chosen side depend on exact targets. Do not quantize or hash-only compare. |
| All rects | Entire dictionary of exact Rect2 position and size, including **rectangles for rows filtered out of ordered**. `_spatial_cell_size` scans every value, so omitted/invisible/non-row rectangles can affect cell partitioning. Frame size, anchor, flips and visible CarryLoad already merge into these rects upstream. |
| Terrain | Exact width, height, and independent PackedByteArray content snapshot for `terrain`. Bounds/indexing and water value4 determine all eight-pixel shift attempts. Capture cells even when the same Dictionary/PackedByteArray instance is reused. Other terrain fields are not read. |
| Blocked | Entire dictionary snapshot conservatively; implementation reads membership only. Preserve changes to keys between calls even at same tick. Values technically do not affect this function; retaining them is a safe extra miss, not a reason to weaken equality. |
| Spatial mode | `use_spatial_hash` exact boolean. Keep hash and brute-force modes distinct. |

`_seated` is unused by this function. It need not enter the key; seating effects must already be reflected in full `targets`. Row pose/activity need not both be copied if their exact effective walking boolean is captured; keeping full rows instead is conservative but likely causes extra misses and allocations. Row tile/ipos/stationary time/meal do not reach declump after target construction. Camera/canvas/delta are not inputs here; walker offsets and easing downstream continue every frame.

Use exact deep value equality, not approximate vectors, rounded coordinates, JSON serialization, object identity, checksum alone or tick generations. Cache one last input/result pair; bounded memory and no global cache shared between renderer instances. Nonfinite inputs should bypass caching conservatively rather than establish unusual NaN equivalence.

## Aliasing and lifecycle

Snapshot before exposing a cached result, independently of caller-owned rows/dictionaries/packed cells. `duplicate(true)` handles nested Arrays/Dictionaries but explicitly duplicate PackedByteArray terrain storage as well; do not rely on undocumented reference assumptions. Rows contain sprites and extra objects in the renderer: capture only scalar projection above, not node references or a deep copy of entire scene rows. Vector2/Rect2 are value types. Keep independent cached output; return a duplicate so caller mutation of a previous result cannot corrupt the next cache hit. An empty dictionary is a valid cached output, so use an explicit valid flag rather than `is_empty()` as the cache state.

On renderer world reset, clear input/result/valid state even though full-input equality should remain sufficient. On enable/disable changes, clear it too; disabled path always directly invokes original static declump and does not populate cache. Do not reuse P10 `allocation-cache` switch: a new P11 switch must toggle **only this candidate**, keeping accepted allocation/seat-input/spatial features identical in control and candidate. Default/reset counters must distinguish misses, hits and disabled calls without entering performance samples unless explicitly diagnostic. Preserve reset's existing CampSeats recreation at renderer382–396.

## Consumers and operations that must remain live

Immediately after declump, renderer reads offsets to form targets, then stateful easing/rounded positions, `seated`, `seat`, `declump_offset`, `drawn_target` metadata and walker lateral relief (674–715). These have per-frame dependencies and cannot be skipped on a hit. Retain complete rows, footprint/CarryLoad calculation, final positions, metadata, animation, pose, frame selection and every appearance update. No offscreen skip or pixel mask is involved. Static function result contains only integer-ID→Vector2 offsets; front arrays/spatial bins must not escape into the cache.

## Verification if selected

Existing `scripts/test/art/test_declump.gd` covers the 0.30 threshold, water and blocked feet, zero/all-blocked offsets, eight-pixel limit, nearest front, equal-y ID order, final shifted front targets, renderer metadata and d24 fixtures. `test_spatial_hash_p10.gd` compares hashed/brute-force results with `camp_seats_reference_p10.gd`, including fractional thresholds, negatives/boundaries, duplicate contributions and varied mutation histories. Keep these reference sources unchanged. `test_camp_seat_cache_p10.gd` and `test_camp_seats.gd` protect upstream seating but do not prove this new cache.

Add one focused mutation-history test that calls cached wrapper and untouched `declump` after every step, checking exact dictionaries and expected hit/miss transitions:

1. First call miss; identical values in newly allocated containers hit; mutate returned dictionary then identical call still returns pristine reference result; empty-output input can hit.
2. Sequentially mutate target x/y, visible, moving, activity fallback/pose, place-known, row order/duplicate ID, target/rect membership, and rect origin/size. Include an invisible/non-row rectangle enlarged enough to change spatial cell size; assert invalidation even if result happens to be equal.
3. Mutate the **same** blocked dictionary and packed terrain array; width/height/cell4 changes; reverse every mutation and compare again. This specifically catches aliased key snapshots and same-tick stale results.
4. Switch spatial mode, disable/re-enable candidate, and reset; verify no stale hit and original controls unchanged. Daily activity/CarryLoad footprint transitions should be represented by changed rects and place-known/targets.
5. Drive stateful renderer twice with identical declump inputs but changing delta/animation context to prove downstream easing, metadata and walkers continue. Do not assert that all render frames must hit.

Offline tests establish only functional boundaries. Acceptance still requires exact-clock six-framing A/A′/B comparison (P5 exact zero), W0/W200 600-frame identity, W500 frozen/2×240-frame pan identity, and three alternating release pairs with immutable workload/build provenance. A failed attempt remains rejected with raw evidence; timer evidence precedes any implementation decision.
