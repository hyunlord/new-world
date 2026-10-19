# Declump kernel verification

Status: candidate disabled by default; numeric/runtime prerequisites and real-fixture replay verified, exact identity PASS; speed NOT_VERIFIED (contended), not accepted.

- Original `camp_seats.gd` SHA256 remains `f0bc0b1f23985a1f104e0607068ef8b8f7ae617166dd7c4e0835235c05b46298`.
- Actual Godot 4.6 stable is single precision (`double=false`). Clean numeric probe exited 0 without errors. Vector rounding is half away from zero and preserves signed zero. Rect area and distance operations round in f32 before widening; touching rectangles have zero intersection. Exact observed bits are covered by the pure Rust numeric test.
- The first exploratory probe deliberately included unsupported negative-size intersection and emitted the engine's negative-size error. Its raw JSON/log are retained; it is not a clean test. The clean probe tests negative-size `abs()` separately and constructs positive/negative zero from runtime bit patterns, avoiding GDScript literal pooling.
- Test-first RED: actual Godot adapter test exited 1 because the adapter did not exist. After implementation it exited 0 with zero failures. Existing declump semantic tests pass unchanged in meaning through both reference and actual bridge, and spatial suite passes 594 comparisons.
- Independent read-only source review found no concrete defects. This does not substitute for real-fixture replay, full identity or valid speed evidence.
- Section instrumentation regenerated successfully against the candidate renderer, preserving ordinary statements. Its unique declump marker remains valid; no marker change was necessary.

Build: main checkout, `CARGO_BUILD_JOBS=6 CARGO_PROFILE_DEV_OPT_LEVEL=2 cargo build -p sim-bridge`, followed by actual Godot headless import, both exit 0. This is the same identity profile as the reference, not performance evidence.

Raw working evidence: `/Users/rexxa/work/art-production12-evidence/kernel/`. Performance acceptance still requires both workloads and valid admission snapshots. No frame-budget improvement is claimed here.

## Real fixture

Capture completed with 2,000 distinct observed renderer calls, 4,000 records (both overlap modes), 200 calls in every world/condition, errors empty. All five worlds and frozen/running conditions are present. Frozen calls remain real observations, not manufactured duplication; 3,764 distinct input/output payloads were observed. Full binary is 57,406,240 bytes; its SHA256 is `f69fb677667ad16849a0590a5bfeecf4d3171e2f69f356d7b1f866883548fd58`. The adjacent JSON retains every observed tick/frame/population and source/dylib hashes. Capture used an uncommitted candidate on parent c8f092db; source hashes and loaded-library hash, not parent HEAD alone, identify it.

`CARGO_BUILD_JOBS=6 CARGO_PROFILE_TEST_OPT_LEVEL=2 cargo test -p sim-bridge`: 52 passed, zero failed, including all 4,000 bit-exact fixture records and numeric observations. Strict GDScript check passes. First clippy attempt found only the modern `is_multiple_of` spelling; the one-line correction preserves parity semantics, and its failed log is retained.

## Identity and speed verdict

Six candidate framings match reference A/A-prime with zero differing RGBA pixels. Running W0/W200 (600 frames each) and W500 frozen/running pan (240 frames each) pass the strict comparator. The first running comparison rejected collector SHA mismatch, although raw rows were byte-identical; its failure is retained. Re-capturing the reference with the same collector resolved the provenance mismatch without weakening the comparator. See pixel-comparison.json, running-comparison.json and running-collector-mismatch.json.

Both speed workloads exhausted three admission holds; all six snapshots are invalid and no Godot measurement was admitted. W200 attempt3 had load5.11 but foreign make-meme Python PID39003 at100% CPU in both snapshots. The outer lock log's contended=false describes its preliminary load/process check only; the saved terminal validity snapshot is authoritative. Speed is NOT_VERIFIED (contended), never REJECTED or PASS. No performance improvement is claimed; P12_DECLUMP_KERNEL remains default-off.

The first admission poll of W500 attempt3 saw our short UI-test process1313; it had exited before the actual hold. No own auxiliary heavy process ran during a hold. Full admissions, runner failures and provenance are retained in speed-admissions/.
