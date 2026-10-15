# Part 1 spatial implementation notes

Planning only; inspected at `b0d3ab6c683b3fb03b1b988853d6c0099939c1cf`. No production edits or runtime verification in this task. Preserve GOAL 5.2c semantics and prove six-framing identity plus the running check before accepting implementation.

## Exact inputs and footprint

`agent_renderer.gd:545` builds blocked tiles, seats, tile spread, targets and merged local footprints, then declumps, eases and rounds drawing positions, and finally applies walker relief. These are distinct stages. Easing and rounding must continue every frame even when the declump result is reused.

Use the exact existing footprint calculation (`agent_renderer.gd:569`): frame size and anchor from the current chosen animation frame; horizontal AND vertical flips adjust the origin; visible `CarryLoad.get_rect()` translated by its local position is merged into that rectangle. Do not substitute opaque-pixel bounds, atlas dimensions, body-only dimensions, or add FounderBundle to the footprint: those change today's meaning. Cache a footprint only by all of its actual inputs, including CarryLoad rect/position/visibility and flips, or reconstruct the cheap rectangle each frame.

Determine the largest actual footprint from the resulting `rects` for that invocation: maximum of both width and height across the merged rectangles. Set the cell size to at least `ceil(max_extent + 8.0)` world units. Do not assume one atlas frame is largest across all animations and loads. Insert each world rectangle into **every** cell its bounds span, using `floor` for negative coordinates. Inclusive upper-bound cells are harmless extra candidates; missing boundary cells are not. Since queries enumerate all covered cells, walker offsets at unusually small zoom remain correct even when their world displacement exceeds eight pixels. Query the shifted rectangle itself for each retry, not just the unshifted rectangle's cells.

## Declump: coverage locality does not imply nearest-front locality

`camp_seats.gd:128` sorts visible rows by target Y descending, then agent ID descending. Each processed row appends its **shifted** rectangle and shifted feet target to fronts. Walkers remain occluders, even though they receive no declump shift. Keep `_walking` at `camp_seats.gd:95` unchanged, including paused `activity=walk/run/carry/carry_wood`.

Give fronts stable ordinal indexes in this exact append order. The hash maps cells to ordinal indexes; each query deduplicates indexes and sorts them ascending before summing intersections. Keep Rect2 intersection, area division, clamp, 0.30 threshold, strict comparisons and retry order unchanged. Nonintersecting rectangles contribute zero; dropping them is safe for ordinary finite geometry. Do not aggregate cell totals or sum in bucket order: fractional sums near 0.30 are order-sensitive.

**Keep the nearest-front search over ALL fronts**, with strict `<` and first-in-front-order tie breaking. A nearest feet target can have no overlapping rectangle, while a farther feet target with a larger/offset footprint causes the coverage. Restricting nearest search to overlap candidates changes preferred shift side. Start with the unchanged global nearest scan only when initial coverage reaches 0.30; this is a safe bounded optimization even if that residual scan costs time. Optimize nearest separately only with an exact global nearest search preserving ties.

Insert each final shifted front immediately after choosing its offset. Retain zero-offset result entries for nonwalkers, absence of entries for walkers, and the existing 1..8 magnitudes / preferred then opposite side / first coverage below threshold termination.

## Walkers: mutable spatial index with original summation order

`camp_seats.gd:199` sorts all visible valid rows by ID ascending. Its `others` list includes every other row in that order. Earlier walkers use their already accepted offsets; later walkers and nonwalkers use original rounded positions. A static index of original positions silently breaks this sequential dependency.

Build the index from all rounded rectangles. For the current walker, exclude its own ordinal in queries. Query all candidate rectangles for each tested shift; return candidates in ascending original row ordinal. After accepting its offset, remove its old cell entries and insert the shifted rectangle before processing the next walker. Retain absent-result versus zero-result behavior (nonwalkers, zero direction and zero initial overlap return no entry; attempted overlap returns an entry even when offset is zero).

Keep screen-space direction normalization and perpendicular construction, parity-based side, integer rounding of screen shifts, length <= 3, orthogonality tolerance 0.001, inverse-basis conversion to world units, terrain checks and strict improvement `overlap < best - 0.00001` exactly. Use the complete `WorldObjects.get_global_transform_with_canvas()` transform currently supplied, never camera zoom alone. Do not transform the world rectangles into screen space as part of this optimization.

## Safe cache keys and invalidation

Use a digest only as a fast reject; exact canonical input equality after matching digests avoids treating hash collisions as equality. Keep independent declump and walker caches. Do not use a single target-position cache for both.

- **Declump:** canonical agent ID set/order, presence of target/rect, target position, complete local rectangle, visibility and exact `_walking` result; full canvas transform as required by GOAL; terrain and blocked-set version/content. Tick, mutation and world-reset epochs may conservatively invalidate terrain/blocked state. Reuse offsets only while every input remains equal.
- **Walker:** all of the above using **current eased, rounded pre-lateral positions** instead of desired targets, plus each walking direction. Even if targets stay constant, easing can change rounded positions; recompute in that case. Include complete canvas transform, terrain/blocked state and presence flags. Sequential accepted offsets are deterministic outputs of these inputs, rebuilt within a recomputation.
- **Seats:** ordered hearth list, blocked set, claimant membership and each claimant's inputs to `_eligible` (visible, exact walking result, stationary threshold boolean `>= 0.3`, meal, pose, tile), plus `ipos` for nearest candidate distance and the 64px acceptance limit. The claimant set must include currently unassigned eligible agents; position or eligibility changes can free seats or change their candidate choice. Subthreshold elapsed seconds themselves need not invalidate, but crossing 0.3 does. Preserve existing assignments and their history; a changed key permits the current incremental update, not wholesale reassignment. Reset clears state and all keys.

A seat occupancy spatial index may replace `_free`'s scan because its result is existential, but keep strict distance `< 16*16` (distance exactly 16 is free), candidate ordering and tie behavior. Rebuild/remove occupancy as assignments are retired, and insert each new assignment immediately. Do not change candidate ordering or choose a globally optimal seating arrangement.

## Semantic locks before optimization

Existing relevant fixtures: `test_camp_seats.gd`, `test_declump.gd`, `test_meal_ring.gd`, `test_agent_drawn_positions.gd`, `test_capture_metadata.gd`. Read their expected values before changing any pin. Preserve an independent straightforward reference implementation in a focused test fixture for differential coverage, rather than deriving expected answers through the optimized helper.

Add focused cases for:

1. Coverage >= 0.30 from a large distant footprint while the nearest feet target is a nonoverlapping front; nearest-distance ties; equal Y sorting; hidden/nonwalking/paused-walking combinations.
2. Fractional intersection areas on either side of 0.30; a front spanning several cells counted once; negative coordinates, touching edges, zero area; all sixteen declump retries; terrain and blocked-tile rejection.
3. Multiple overlapping walkers where earlier offsets change a later walker's answer; three distinct cardinal/diagonal directions; tiny, nonuniform and rotated canvas bases; zero direction; exact overlap-improvement tolerance.
4. Same targets but changing rounded eased positions, changing walking direction, CarryLoad visibility/position, animation size/anchor, flip H/V, visibility, terrain, blocked set or canvas. Each must invalidate the appropriate cache. Identical input must reuse the exact output shape.
5. Seat threshold 0.299 -> 0.3; eligibility changes without ID changes; claimant removal/new claimant; changed `ipos` of an unassigned eligible claimant; hearth/blocked changes; exact 16px occupancy; reset and rebuild preserving original deterministic outcomes.

Run these only when the parent authorizes Part 1 execution and the quiet-lock protocol permits it. Differential fixtures supplement, never replace, W0/W200 600-frame equality and six-framing pixel identity.
