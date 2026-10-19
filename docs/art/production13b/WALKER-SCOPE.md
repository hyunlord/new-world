# Optional walker kernel: prerequisite scope

Status: READ_ONLY_INVESTIGATION; implementation, runtime semantics probes, fixtures, identity and speed NOT_VERIFIED. Source inspection at HEAD `8900e1c69f8ba3aa8775d6f1af154e622bcb87df`; concurrent work may advance HEAD. No Godot, cargo or tests were run for this investigation. Only this document was written.

## Contract and decision boundary

`GOAL.md` 4.4 requires a pure `walker_offsets` in `art_kernels.rs`, one `#[func] art_walker_offsets` bridge wrapper, and `P13B_WALKER_KERNEL`. The existing GDScript functions remain unchanged as oracle and off-switch. At least 2,000 actual renderer calls must cover W200 running, W-multi running and W500 frozen and replay bit-for-bit with `cargo test -p sim-bridge`. Identity and admitted alternating release measurements are separate gates: W200 running 2× must improve under inherited acceptance rules and W500 frozen pooled median must be ≤ 1.00. This candidate is first in the drop order; never-drop work and Part deadlines take priority. No recommendation here constitutes acceptance or a reason to delay those deadlines.

## Exact source closure

- Entry: `scripts/ui/camp_seats.gd:237`, `walker_offsets`.
- `_walking` at 128: `moving` defaults true; activity falls back to pose, matching `walk`, `run`, `carry`, `carry_wood`.
- `_declump_allowed` at 226: subtract `(456,42)`, divide by 16, vector-round, convert to tile; reject blocked/out-of-bounds/missing terrain/code 4.
- `_walker_overlap` at 291: summed intersection area, no coverage normalization or clamp.
- `_spatial_cell_size` at 299: all rectangle dictionary values, not just eligible rows; `max(8, ceil(max absolute dimension + 8))`.
- `SpatialIndex` at 305: constructor, `cells`, `insert`, `_add_cells`, `move_agent`, `query`, `overlap`. This is a movable index, unlike the current Rust declump index.
- Builtin numeric closure: `Transform2D.affine_inverse`, `basis_xform`, `Vector2.normalized`, `is_zero_approx`, unary negation, component arithmetic, `round`, `length`, `dot`, `Rect2.abs`, `end`, `intersection`, `get_area`, float `floor`/`ceil`/`abs`, and positive modulo parity.
- Caller: `scripts/ui/agent_renderer.gd:830`, after easing and sprite-position rounding. Inputs are `agents`, `rounded`, merged sprite/carry-load `rects`, terrain, blocked tiles, current global canvas transform and `_spatial_hash_enabled`. Capturing declump targets before easing is not a walker fixture.

Seat assignment, spread and declump precede the walker but are input producers, not functions in its pure computation closure. Their implementations need not change.

## Order, omission and ties

1. Filter to visible rows with position and rectangle entries, then sort ascending integer agent ID. Preserve rows rather than deduplicating IDs. Duplicate-ID equal-comparator ordering is not established by this inspection; either prove real IDs unique and validate that boundary, or verify Godot sorting for duplicates before claiming a general exact implementation.
2. Insert every filtered row into the spatial index in that order, including stationary/place-known rows. They remain obstacles. Skip producing offsets for place-known/nonwalking/approximately zero-direction rows.
3. In brute mode, construct other rectangles in sorted-row order, excluding every matching ID. Apply previously produced offsets to earlier rows. In spatial mode exclude all matching agent IDs and sum deduplicated ordinals in ascending insertion order.
4. Compute baseline overlap. If ≤ 0, omit the result key. Otherwise examine magnitudes 1, 2, 3, each with parity-selected side first, then opposite side (positive side for even ID). Round screen shift before checking length ≤ 3 and absolute direction dot ≤ 0.001. Transform that rounded shift using inverse basis, then test terrain.
5. Accept only `overlap < best - 0.00001`; equal or near-equal candidates keep the earlier choice. There is no early exit on zero overlap. Once baseline was positive, write a result entry even when offset remains zero. Absence and explicit zero are different outputs.
6. Move all index ordinals for this agent after its final choice; removal/addition changes bucket membership but not ordinal summation. Cells include both endpoint cells, visit y then x and deduplicate by ordinal. Negative rectangle size is normalized for cell lookup; unsupported negative-size intersection must not silently acquire invented semantics.

## Precision and remaining numeric prerequisites

The earlier pinned evidence is `docs/art/production12/perf/declump-kernel/verification.md`, not a file at the production12 root. It documents Godot 4.6 single precision, vector half-away-from-zero rounding with signed zero, f32 rectangle area/distance before widening, and zero touching intersection. `art_kernels.rs:507` has the corresponding numeric test. Those historical observations are not new verification for this candidate.

Vector2/Rect2 and Transform2D storage and builtin vector intermediates must follow f32 real_t; GDScript `float` variables and scalar comparisons/accumulated areas use f64. Preserve each left-associated vector addition/multiplication rounding before widening. In particular, `position + shift + local_rect.position`, `perpendicular * float(magnitude) * sign_value`, vector dot/length, normalized division and inverse determinant arithmetic must match actual Godot operations; avoid f64-only reconstruction, reassociation and fused multiply-add substitutions. Cell division widens components into f64 because `cell_size` is a GDScript float. Overlap sums individually f32-rounded rectangle areas in f64.

The declump probe does NOT establish affine-inverse implementation, transform multiplication order, normalization/sqrt/zero thresholds, or vector dot precision for walker inputs. Before implementation acceptance, inspect the exact installed Godot version's source and record actual-runtime bit probes for rotation, nonuniform scale, tiny direction, zero/signed zero, rounding boundaries and inverse basis. Do not assume zoom-only transforms: `test_spatial_hash_p10.gd:95` already covers identity, tiny/nonuniform scale, and rotation. Singular transforms need an explicit supported-input policy matching the oracle, not an unverified fallback.

## Existing architecture and capture mechanism

`rust/crates/sim-bridge/src/ffi/art_kernels.rs` currently has packed borrowed `DeclumpInput`, ordered IDs plus f32 output pairs, `Rect`/intersection arithmetic, a terrain eligibility helper, validation and an insert/query-only spatial index. These are reusable patterns, but the new walker needs stored rectangles, agent-to-ordinal lists and remove/reinsert movement. Avoid changing accepted declump results while extending shared internals; a separate walker index within the same art-owned file is a bounded alternative if sharing would threaten that contract.

`world_node.rs:279` shows the thin packed-array `art_declump` wrapper and error dictionary; `scripts/ui/art_declump_p12.gd` shows ordered packing, resolved flags, actual bridge method checks, returned-column validation and byte-exact output comparison. New input columns additionally need per-row walking-direction f32 pairs and exact canvas transform components. Compute the inverse in Rust to reproduce the specified full pure function; sending only a GDScript-precomputed inverse would leave part of the requested closure outside it. Preserve independent positions/rect dictionaries and all entries affecting cell size.

`scripts/test/art/declump_fixture_recorder_p13.gd` records through an actual renderer sink at `agent_renderer.gd:802`; it invokes the original oracle in both brute/spatial modes, serializes little-endian exact f32 bits and result insertion order, retains seed/tick/render-frame/population, rejects duplicate invocation per observed frame, hashes sources/dylib before and after and writes provenance JSON. `capture_declump_fixture_p13.gd` sets up real worlds, zoom 4, tick progression checks and 200 calls per ten world/condition blocks. Its existing binary contains no canvas or walking directions and cannot establish walker identity.

A changed capture must get a `_p13b` name and a separate magic/schema/output file. Place its new sink immediately before the walker call after rounding; keep the old declump recorder unchanged. Capture ≥ 2,000 distinct real calls across the three required conditions, with a preregistered per-condition allocation; recording two oracle modes yields ≥ 4,000 replay records but still only 2,000 calls. Preserve native input/output bit patterns, distinguish empty from zero entries and bind every record to frame/tick/source/library provenance. Retain the existing rule that fixture collection occurs outside measurement quiet holds and use `with_window_lock.sh` for all windowed runs. Frozen repeated inputs may be real separate frames, but manufactured duplication is not evidence.

## Minimal permitted prospective implementation surface

- `rust/crates/sim-bridge/src/ffi/art_kernels.rs`: pure function, types, required helpers, replay/numeric tests.
- `rust/crates/sim-bridge/src/ffi/world_node.rs`: only art wrapper and corresponding art-kernel imports; no engine/simulation edits.
- A new fixture directory beneath `rust/crates/sim-bridge/tests/fixtures/` and manifest.
- New `scripts/ui/art_walker_p13b.gd` adapter plus minimal `agent_renderer.gd` switch/call/sink wiring. Leave `camp_seats.gd` unchanged.
- New `_p13b` capture/recorder/adapter verification scripts under `scripts/test/art/`; only necessary measurement-tool copies under `tools/art_pipeline/`. Keep prior tool/evidence originals.
- New evidence and decision records under `docs/art/production13b/**` and designated external evidence directory.

No dependencies, simulation changes, generic refactor, reference modifications or seats kernel are prerequisite to this walker slice.

## Realistic execution prerequisites and risks

Obtain an available window/quiet-lock schedule from the leader before any capture or measurement; no concurrent build/capture may contaminate admitted pairs. Budget a bridge rebuild/import, numeric prerequisite capture, ≥ 2,000 real-call fixture capture and Rust replay, actual bridge adapter checks, all six framings/running dumps/pan identity, and three valid alternating pairs on each workload. Existing structural walker tests (`test_meal_ring.gd`, `test_spatial_hash_p10.gd`, `test_activity_installed_p10.gd`) cover useful behaviors but do not replace fixture bits or full identity. The section generator anchors the `var lateral:` statement, so integration must preserve or explicitly verify instrumentation generation.

The main implementation risks are inverse/normalize numeric fidelity, f64/f32 boundaries, duplicate-row and omission semantics, mutable spatial ordering, wrong pre-easing capture location, packing overhead erasing the kernel speedup, and inability to obtain admitted measurements before deadlines. A pure Rust microbenchmark or fixture pass alone cannot establish the end-to-end benefit. If prerequisites cannot fit while preserving never-drop checks, record this optional target as not built/not verified and continue the mandated work; do not enable it speculatively.
