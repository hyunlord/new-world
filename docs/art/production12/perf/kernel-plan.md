# Part 1a declump implementation and evidence plan

Status: source-reviewed implementation plan, not a precision, identity, or speed PASS. No cargo or Godot was launched for this document. The original attachment and GOAL sections 5.2a/5.3 are authoritative. Source inspected: `scripts/ui/camp_seats.gd:128,161–239,299–365`, `scripts/ui/agent_renderer.gd:640–680`, existing declump/spatial tests, bridge wrapper conventions and crate manifests. `godot = "0.4"` is the actual workspace dependency (the ffi module comment mentioning 0.5 is stale); no dependency or manifest changes are necessary.

## Scope and sequence

1. Preserve `CampSeats.declump` and its helpers byte-for-byte. Add the pure kernel in the single permitted `ffi/art_kernels.rs`, its one `pub mod art_kernels;` declaration, and the packed wrapper in `world_node.rs`.
2. Put packing/unpacking in an art GDScript adapter reused by production and tests. Replace only the renderer call site with a switch: original by default until all acceptance gates pass; explicit `P12_DECLUMP_KERNEL=1` enables the candidate. No simulation or walker-offset changes.
3. First capture the real reference fixtures without enabling the candidate. Validate counts, worlds, modes, complete output keys, hashes, and build identity. Then replay in Rust and through the actual Godot wrapper.
4. Run existing declump/spatial suites through both paths. Only after numeric equivalence run the full 5.3 identity and alternating speed gates. Keep the switch off if rejected, unfinished, or unverified.

## Packed wrapper ABI

Proposed `#[func] art_declump(&self, ...) -> VarDictionary`, using arguments in this order:

| Argument | Godot type | Contract |
|---|---|---|
| row_ids | PackedInt64Array | Original input row order, not dictionary order or sorted |
| visible, walking, place_known | Three PackedByteArray | One 0/1 byte per row |
| target_ids | PackedInt64Array | All targets in dictionary iteration order |
| target_xy | PackedFloat32Array | x,y pairs, length 2 × target count |
| rect_ids | PackedInt64Array | All rectangles, including ids excluded from ordered rows |
| rect_xywh | PackedFloat32Array | position x,y and size x,y, length 4 × rect count |
| width, height | i64 | Original terrain dimensions |
| terrain | PackedByteArray | Full terrain bytes, including short arrays if supplied |
| blocked_xy | PackedInt32Array | x,y pairs for blocked dictionary keys; values irrelevant |
| use_spatial_hash | bool | Original mode |

Return `{ids: PackedInt64Array, offsets: PackedFloat32Array}`; offsets are interleaved x,y and ids follow result dictionary insertion order. Output must distinguish an absent key from a present zero offset. Place-known and walking rows produce no result entry but still occlude later rows. Duplicate row ids remain separate front entries; dictionary result updates retain the first key insertion position and the latest value. Target/rect dictionaries have unique keys by construction.

GDScript computes walking exactly as `_walking`: `bool(row.get("moving", true)) or str(row.get("activity", row.get("pose", ""))) in ["walk", "run", "carry", "carry_wood"]`. Visible/place-known defaults are false. `_seated` is unused by the reference and is not part of this ABI. Packing does not filter targets/rectangles and does not change source containers.

The wrapper validates column lengths, converts to native borrowed slices or small native structs, calls the pure kernel, and packs its output. It must not read `self.engine`, hecs, or a snapshot. A malformed ABI produces an explicit error; it must not silently drop rows and report success. The pure kernel contains no Godot types or runtime calls, so unit tests run without a Godot process. Native structs and `std::collections` suffice; serde is not needed for fixture decoding.

## Numeric contract and operation order

Record `OS.has_feature("double") == false` from the actual capture executable before claiming any equivalence. Scalar GDScript `float` is f64, while Vector2/Rect2 storage and arithmetic are f32. Do not replace this mixed precision with uniform f64. Do not use `mul_add`, vectorized reductions, fast-math, or algebraic reassociation.

| Reference operation | Required evaluation |
|---|---|
| Target y read for sorting | f32 component widened to f64; descending y, then descending signed i64 id |
| `target + local_rect.position` | Each component added in f32 |
| `target + shift + local_rect.position` | First f32 target+shift, then a separate f32 +position |
| Final front rect | First f32 target+offset, then f32 +position; store shifted target separately |
| `rect.get_area()` | f32 width × height, then widen product to f64 |
| Intersection | f32 ends `position + size`; f32 max positions/min ends and subtraction; empty intersection has zero area. Match Godot empty/touch behavior, not an unverified Rust geometry library |
| Overlap accumulation | Each intersection area is f32 product widened to f64, then accumulated sequentially in f64 |
| Coverage | f64 accumulated overlap / widened f32 area, capped using f64 `minf(..., 1.0)`; area <= 0 returns 0 |
| Nearest front | f32 component differences, individual f32 squares, f32 sum; widen distance result to f64. Compare strictly `<`, never `<=`; scan all fronts |
| Side | Component comparisons; x tie chooses +1 for even id, -1 for odd, matching positive modulo |
| Shift | f64 direction × magnitude, narrow into f32 Vector2 x; y is +0.0 |
| Allowed tile | f32 `(point - Vector2(456,42))`, f32 division by 16, Vector2 rounding in f32, then integer conversion |
| Cell extent | Read every rect size component as f64; f64 abs/max, f64 `ceil(extent + 8.0)`, f64 max with 8.0 |
| Spatial bounds | `rect.abs()` uses f32 position + min(size,0), and f32 abs(size). `bounds.end` is f32 position + size |
| Cell coordinates | Widen each component to f64, divide by f64 cell size, floor to integer |

The original source does not define the implementation of Godot built-ins. Before locking arithmetic helpers, add a small Godot probe that exports exact f32 bits for distance, area/intersection, negative-size abs, touching edges, and rounding at negative and positive half tiles. Check against the installed engine or matching C++ source, rather than treating the Rust binding's convenience implementation as proof. In particular, prove half-tie behavior and signed zeros. This is a prerequisite verification, not a reason to launch a separate dependency project.

Maintain branch/order details verbatim:

- Filter visible rows with both target and rect, then sort as above. Inputs with identical id/y compare equal; preserve observed duplicate-row behavior.
- Stationary and place-unknown rows always get a result key, even if offset is zero. Moving/place-known fronts are inserted too.
- Test initial coverage against f64 0.30; scan all fronts for nearest, including fronts outside spatial query buckets.
- Recompute best coverage as the reference does; magnitude order 1 through 8, direction order `[side, -side]`.
- Skip forbidden shifts; update best and offset only for strictly smaller coverage. Stop at first allowed coverage strictly below 0.30, after the best update.
- The allowed check first rejects blocked keys, then bounds, then short terrain length, then terrain code 4. Integer width/height/index arithmetic is signed; never cast negative coordinates to usize before checking.
- Each processed row inserts its shifted rectangle before the next row.

## Both overlap modes

Brute mode sums every front intersection in insertion order. Spatial mode inserts ordinal indices into all inclusive cells (y outer loop, x inner loop); query collects unique ordinals then sorts ascending before summation. Never accumulate bucket iteration order or count a rectangle twice. Keep the nearest-front scan global in either mode. HashMap is acceptable for cell lookup because hash iteration must not control output; BTreeMap is also sufficient. Do not use parallelism.

The cell size considers all rect dictionary values, not just eligible rows. Inclusive last cells at rectangle endpoints are required. Zero area stops coverage before querying. Walker-specific `move_agent` and exclusion support are outside this kernel: they remain unchanged in GDScript.

## Dependency-free binary fixture

Use a little-endian versioned binary file under `rust/crates/sim-bridge/tests/fixtures/art_p12/declump_v1.bin`, and a JSON metadata manifest alongside it. Production input floats are all f32 component values, so store their exact IEEE u32 bit patterns, never decimal JSON. Use Godot PackedByteArray `encode_float`/`decode_u32` or equivalent verified binary primitive; Rust decodes with `u32::from_le_bytes` then `f32::from_bits`. Capture any scalar f64 probe values separately as IEEE u64. Do not use Variant serialization or add a JSON/RON decoder dependency.

File layout:

- Header: 8-byte magic `P12DCL01`, u32 version=1, u32 call count.
- Each call: u32 payload byte length; u8 world enum (W0=0,W1=1,W200=2,W500=3,W-multi=4); u8 running flag; u8 spatial flag; u8 reserved=0; i64 seed; i64 tick; u64 observed render frame; i64 width; i64 height.
- u32 row count, then each row `{i64 id,u8 visible,u8 walking,u8 place_known,u8 reserved=0}`.
- u32 target count, then each target `{i64 id,u32 x_bits,u32 y_bits}`.
- u32 rect count, then each rect `{i64 id,u32 x_bits,u32 y_bits,u32 w_bits,u32 h_bits}`.
- u32 terrain byte count, followed by bytes; u32 blocked count, then `{i32 x,i32 y}` pairs.
- u32 expected-result count, then each expected `{i64 id,u32 dx_bits,u32 dy_bits}` in original dictionary key order.

Rust test decoder uses bounded slice reads with informative errors, rejects trailing bytes/version mismatches/count mismatches, and asserts output ids plus every component `to_bits()`. Payload lengths allow exact framing and truncation detection. Encode original GDScript outputs before any candidate call. Do not regenerate expected outputs from Rust. A separate malformed-fixture check can live in the same module; no extra Rust source file is needed.

## At least 2,000 real-call capture plan

Collect 200 actual renderer declump invocations for each of 5 worlds × 2 conditions (frozen and running 2×), totaling 2,000; do not fabricate samples or duplicate one call to meet the count. Use exact worlds/ticks in `perf/worlds.md`, fresh reset per block, and record tick/frame/population per call. Frozen calls may naturally match; retain them and report unique input counts honestly. The source frame must be a real `_apply_drawn_positions` call after footprints/targets have been built, not a hand-generated arrangement.

Capture must be enabled only by an evidence environment flag and run outside measurement quiet holds. Buffer/stream files outside performance runs. For each captured input, compute original spatial and brute outputs and store two records (4,000 records derived from 2,000 real calls), with paired provenance. Keep distinct metadata counts `real_calls=2000`, `records=4000`, and per-world/condition/mode counts. Cross-mode equality may be checked but each replay is compared to its own original mode's expected bits.

Record capture engine version, false double feature, baseline source SHA, dylib SHA/profile, source/script hashes, seed/tick/frame, population, fixture SHA256, and exact command in the manifest. The Rust test asserts at least 2,000 distinct provenance calls, complete 5×2 coverage, and both mode records per call in addition to bit equality. Preserve raw logs and any failed captures; no acceptance claim from a fixture count alone.

## Tests and acceptance integration

- Kernel unit tests live inside `art_kernels.rs` under cfg(test), including `include_bytes!("../../tests/fixtures/art_p12/declump_v1.bin")` relative to this source file's directory (verify the path when implementing), decoded and replayed with std only. Existing Cargo manifest already exposes rlib and cdylib.
- `test_declump.gd`: route each declump assertion through a helper that invokes the reference and candidate adapter, compares presence/keys and packed float bits, and returns the candidate result. Keep existing semantic assertions intact, including seating, water, nearest ties, renderer integration, and D24 fixtures.
- `test_spatial_hash_p10.gd`: add adapter checks in `compare_case` for both flags. Preserve existing SpatialIndex and walker assertions; they must not become tests of an unrelated replacement.
- The renderer test instantiates Renderer without attaching it to the tree. Do not depend on autoload tree lookup that breaks this case: provide a test-owned actual WorldSimNode bridge to the adapter or explicit injectable adapter bridge; normal production uses its existing bridge. Do not fake kernel output in the test.
- Run those Godot suites with the kernel enabled, plus `cargo test -p sim-bridge` and clippy through the allowed gate. The bridge/runtime differential proves packing as well as the pure algorithm.
- Acceptance still requires all six exact-clock framings including P5; W0/W200 running 600-frame dumps; W500 frozen/2× 240-frame pans; and three alternating 150-frame pairs for both W500 frozen Z4 and W200 running 2×. Valid-session pooled candidate/reference median <=0.97 and at least 2/3 faster pairs in each condition. No cargo or capture processes during a measurement hold.
- A numeric PASS is not a frame-budget PASS. If any identity condition fails, debug the first mismatch; never weaken comparison. If speed misses on valid holds, reject and leave reference in production. If holds never valid in three attempts, mark NOT_VERIFIED (contended). Preserve the Part 1 hour-6 stop and rejection evidence.

No new localization keys. No simulation changes, Cargo changes, commits, or runtime processes were made by this planning subtask.
