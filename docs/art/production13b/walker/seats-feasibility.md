# Seats kernel feasibility before Part1 deadline

READ_ONLY feasibility, not implementation or acceptance. Requested decision window:13:39Z to17:08Z,209minutes nominal; remaining walker acceptance and mandatory work consume that same serial window. No product/source/HEAD changes, Godot, builds or tests were run.

## Recommendation

Technically feasible within permitted paths as an explicit state-transition kernel, but NOT a dependable end-to-end accepted deliverable within the remaining209minutes. A short pure-Rust port is not the whole task: persistent assignment/cache state, a new trig/pow numeric boundary, history-aware real-call fixtures and another full identity/measurement cycle remain. GOAL4.4.2 explicitly permits execution only after4.4.1 is accepted and time remains; walker is currently pending per leader. No seats implementation should start on this report's authority.

If walker acceptance finishes promptly and never-drop work has a reserved window, a bounded preparation/implementation attempt is possible. Do not declare feasibility solely from available lines of code or pure replay. If valid speed admission or state/float identity cannot finish before the Part1 limit, retain the failed/unverified evidence and leave seats off. No allowed-path blocker forces a simulation change or new dependency.

## Actual closure and state

`CampSeats.update` is `scripts/ui/camp_seats.gd:34`, not a stateless replacement for the walker:

- Mutable persistent `_hearths`, `_blocked`, `_candidates`, `_assignments`, `_input_cache_enabled`, `_input_cache_valid`, `_input_cache_rows`, `_input_cache_result`.
- `set_input_cache_enabled`/`_invalidate_input_cache`/`clear` at14–32; changing cache flag invalidates cache without discarding assignments. `clear` resets both.
- `_rebuild` at84, `_free`112, `_eligible`118, `_walking`128, `_tile_before`131.
- `spread_offsets` at134 is stateless given raw rows and just-produced seats, but has an independent raw-row ordering contract and >=5-person trig path.
- Renderer `_apply_drawn_positions` prepares the actual blocked set before calling update/spread at793/794. Keep this terrain/blocked producer unchanged. Renderer replaces `_camp_seats` with a fresh object on reset at532 and configures its input cache at148; a candidate adapter must observe equivalent lifecycle resets.

Minimum pure boundary: `seat_step(current_inputs, previous_state) -> {next_state, ordered_seats, ordered_spread}` inside art_kernels.rs. A thin `#[func] art_seats` wrapper accepts/returns packed values; a GDScript adapter owns the returned state between calls. This avoids adding persistent fields to WorldSimNode outside the allowed wrapper-only surface. Do not store simulation state or add engine methods. Packed prior state can be a typed collection of columns or versioned opaque bytes with explicit validation; serialization/packing overhead is an important performance risk.

The allowed files are art_kernels.rs, new art wrapper/imports in world_node.rs, fixtures, new art GDScript adapter/capture/test files, minimal renderer switch/reset wiring and production13b evidence. Existing camp_seats.gd and historical frozen reference stay byte-identical. New wrapper/state support can coexist with walker, but measurement/capture/build phases cannot run concurrently with its quiet holds or source freezes.

## Exact semantic requirements

Update sorts hearths y then x, rebuilds only if sorted hearth array or blocked Dictionary differs, then canonicalizes rows by integer ID with LAST raw row winning for duplicates. Unique IDs are sorted ascending for new assignments. Existing assignments retain insertion order, are removed only when missing/ineligible or invalidated by environment rebuild, and are never greedily reassigned just because interpolated position changes. A later lower-ID arrival cannot displace an older assignment.

Candidates are generated for each sorted hearth with j=-4..4 then i=-7..7. Odd-row offset uses positive modulo even for negative j. Filter ellipse with f64 `pow(offset.x/104,2)+pow(offset.y/72,2)>1`, f32 vector length<16, rounded blocked/hearth tile tests. Final order is j, i, hearth(y,x). Candidate membership during rebuild compares full seat dictionaries (position,hearth,i,j), not just position or a newly numbered slot. Preserve this across hearth insertion/removal/reordering.

Assignment eligibility: not place-known, visible, not walking, stationary_seconds not less than0.3, meal or idle/sit/hop/crossed, Chebyshev tile distance<=6. Nearest free candidate must have f32 squared position distance widened to f64 <=4096. Exactly equal distance retains the earlier candidate; exactly256 separation is allowed because `_free` rejects only <256. Persistent assignment dictionary iteration determines output ordering, which matters to bit-exact fixture identity even where dictionary equality tests ignore order.

Cache key is sorted canonical rows of id,visible,walking,place-known,`not(stationary<0.3)`,meal,exact pose string,tile,ipos. Stationary elapsed time above threshold is intentionally collapsed; row-order changes do not miss. Preserve pose strings in cache identity even when multiple poses share eligibility. The cache returns a copy: caller modifications cannot alter cached result/state. Vector equality follows Godot numeric equality (signed zeros compare equal), not raw-byte equality. For valid nonfinite inputs the current oracle has special comparisons (NaN stationary is not <0.3); either retain semantics or make valid-input constraints explicit and verified. Do not accidentally replace `not(x<0.3)` with `x>=0.3` without addressing NaN.

Blocked Dictionary comparison includes values, not only keys; production currently supplies true-valued membership entries. A packed membership set is sufficient for free/blocked decisions, but does not exactly represent arbitrary value-only change detection unless the adapter establishes the true-valued input boundary. Hearth duplicates remain actual array entries. Avoid opportunistic deduplication of candidate rows.

Spread does NOT reuse canonicalized update rows. It iterates original rows, excludes walking rows unless place-known and excludes seated IDs, groups by first-seen tile insertion order, sorts group IDs, and emits per-group ranks. No visibility filter is applied here. Duplicate raw IDs can occur in group lists and repeated result assignments overwrite in their original dictionary slot. Sizes1/2/3/4 use exact fixed vectors; >=5 uses `Vector2.from_angle(TAU * float(rank) / float(size)) * 7`.

## Numeric prerequisites beyond Pass12

Pass12 `docs/art/production12/perf/declump-kernel/verification.md` pins single-precision vectors, half-away-from-zero round/signed zero and f32 distance/area before widening. It does not pin f64 `pow`, GDScript TAU arithmetic and f32 `Vector2.from_angle` sin/cos behavior for all spread ranks.

Upstream Godot4.6 `vector2.h` declares from_angle(real_t), and vector2.cpp returns cos/sin of that real_t. Thus TAU/rank/count expression is f64 first, then converted to f32 for angle, then f32 trig and vector multiply7. A Rust f64 sin/cos or converting rank too early is wrong. Exact platform libc/optimizer behavior needs actual Godot bit probes and Rust comparison; do not silently replace pow(x,2) by x*x merely because algebra agrees.

Probe all finite ellipse i/j values and candidate ordering across representative positive/negative hearth tiles, distance ties/boundaries, and spread ranks for every observed group size (plus threshold4→5). Fixture seeds may not naturally exercise large same-tile groups; synthetic numeric probes supplement, never replace real calls. Angle rounding and sin/cos are a larger remaining uncertainty than fixed seat offsets.

## Fixture boundary and minimum tests

Use a NEW sink immediately around actual `_camp_seats.update`/spread, after the existing blocked preparation. Record each real call once, including reset/clear/cache-toggle sequence boundaries, previous state, full ordered inputs, next state and ordered seats/spread exact f32 bits. Do not invoke update twice on the same live oracle to manufacture before/after observations. Options are before/after snapshots of its state, or a separate persistent oracle replay receiving the same history and verified against actual outputs.

At minimum follow4.4's same-method actual>=2000calls from W200running/W-multirunning/W500frozen, retaining frame/tick/population/source/library hashes. Crucially replay whole consecutive histories from reset, not independent calls with the answer's state supplied on each step: independently seeded pre-state can mask a broken state transition. Captured warmup calls/state must be included if recording begins after assignment history already exists. Compare next state and ordered outputs; both cached/uncached paths require targeted behavioral tests even if production uses cached path.

Reuse existing requirements in `test_camp_seats.gd` and `test_camp_seat_cache_p10.gd`: persistence, lower-ID arrivals, removal, moving/invisible/place-known release, blocked/hearth changes, cache invalidation/copy isolation, row reorder/last duplicate, stationary0.299/0.3, six-tile and64px bounds, >=16px spacing, exact ties, spread1–4 and trig>=5, clear/reset and180-step histories. Existing cache frozen ReferenceSeats is historical and omits activity_place_known eligibility; CURRENT CampSeats remains the oracle and arrival-specific tests must cover that difference. Existing approximate >=5spread tests do not establish bit equality.

Fixture replay belongs in `cargo test -p sim-bridge`, with required capture files and no silent skip. Actual wrapper/adapter checks and full existing gate still required. Independent state resets for every OFF/ON performance block are mandatory; toggling a flag mid-world between stale independent seat histories could create identity differences or a misleading speed comparison. Either transfer exact state explicitly or reinitialize identical worlds before each candidate block, matching the established alternating protocol.

## Time estimate and decision milestones

These are engineering estimates, not measured completion times:

| Work | Plausible elapsed budget absent contention |
|---|---:|
| State schema, Rust update/rebuild/cache/spread, wrapper/adapter/reset wiring |75–120min|
| Numeric probes and exact-bit corrections, compile/clippy/basic history tests |25–50min|
| Real-call recorder/schema/replay +2000-call capture/provenance |25–45min|
| Six framing control/reference/candidate + running/pan identity |25–50min|
| Two workloads, three valid alternating pairs each + independent review |20–40min plus admission waits|
| Total serial-equivalent work |170–305min plus remaining walker/mandatory queue|

Parallel offline preparation can reduce elapsed coding time, but build/window/quiet-locked verification remains serial and a rejected admission costs at least30minutes before another attempt. One numeric or history correction can invalidate dependent captures and consume the three-strike budget. The lower-bound170min leaves39min of the nominal209min for all remaining walker acceptance/mandatory work and no meaningful contention margin. This is why a full accepted rollout is not dependable before17:08Z.

If leader elects to attempt after walker acceptance: first fix the state schema and preregister fixtures/workloads; establish numeric exactness before full fixture/identity capture; require a checkpoint with working wrapper+history replay and sufficient remaining time for all captures/pairs. Drop the optional candidate rather than enable unmeasured behavior or substitute microbenchmarks. No implementation, pass, speedup or acceptance is claimed by this report.
