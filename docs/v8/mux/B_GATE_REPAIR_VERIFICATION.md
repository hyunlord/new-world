# B gate repair verification — focused PASS, full rerun pending

Full gate 1 at clean `60f3371691b398e16a9d3675ea3e36b058559616` **FAILED: 31/36 steps passed**, exit 1. Preserve `/Users/rexxa/work/mux-data/gate-stage-b-1/receipt.json` and every original log. All file hashes listed by that receipt independently match current archived bytes. Failed steps: test_debug, test_release, python_unittest, m1_tests, knowledge. Five failed steps do not mean five independent runtime defects: the axe failure repeats in knowledge, and three Rust failures recur across profiles.

| Test/setup defect | Original evidence | Test-only repair |
|---|---|---|
| Material table cardinality | colony_items.rs98: actual7/expected5 | Compare exact IDs/names/integerized stats with strictly parsed authored physical definitions; retain every placement assertion. |
| Synthetic tree lacks B5 identity | colony_knowledge.rs1091: tools_broken0/expected1 | Use existing plant fixture helper; retain uses119→break120 and Chronicle assertion. |
| Historical art-style names | engine_space_ffi.rs331: log/LogWall | Assert authored log/drystone/slab_door/packed_earth names, retaining procedural DugFloor, material/category and numeric style-code checks. |
| Obsolete bridge manifest | Python live API comparison used M5.5 list | Point exact set equality to current bridge_api_mux.txt; retain historical list and removed-name tests. |
| Locale harness omitted catalog initialization | Nine TRF failures rendered unknown labels | Explicit bootstrap_catalog plus affirmative receipt/nonempty matching UI revision before existing locale assertions. |

The diff changes exactly three Rust integration test files and two test tools. It changes no production Rust/GDScript, authored product data, baseline, threshold or expected physical behavior. The corrections restore valid setup/current authored contracts; original failures remain part of the record.

Focused repair evidence, reviewed from `stage-b/b5/gate1-repair-focused/`:

- fmt check exit0.
- Exact Rust regressions: items1, axe1, space1 PASS; existing tests otherwise filtered, not claimed rerun.
- Python discovery: 133 tests PASS (`python.log`:21–23).
- Explicit sim-bridge build exit0 before Locale execution.
- Locale `TRF_TEST_PASS`, exit0; single-file Locale strict check exit0.
- Workspace/all-target clippy with `-D warnings` exit0.

These receipts establish focused repair validation, not a new full gate PASS. LSP retry still failed with Transport closed (ISS284); do not substitute a successful LSP claim. ISS305/306/308/309/310 are the associated repair records; their scoped repairs are fixed in docs/ISSUES.md; full-stage acceptance remains pending. ISS307 remains separately reserved for the external C-wrapper correction; do not reuse it for these five test repairs.

### Source binding

Independent hashing of the original 2,123-path B5 freeze finds exactly five differences, all the test paths below; the other **2,118 paths match exactly**. This is a comparison against the unchanged original freeze, not permission to overwrite it. The test-code diff lists exactly these five paths; accompanying documentation records this verification. Old OFF/ON execution remains bound to its original full-source freeze and binary receipts; unchanged production inputs support carrying forward product evidence, but it is not literal execution of the newly repaired test-source tree. No new OFF/ON run or binary identity is claimed here.

| Path | Original SHA256 | Repaired SHA256 |
|---|---|---|
| `rust/crates/sim-test/tests/colony_items.rs` | `739fe67ce94d454fb20e07146296d3e1153c580b1dd37c781cb63f063e0703a2` | `80f6fbff658c83380b88e6c7fd279bf9a934647812697a6250a7d97a63b2a2c6` |
| `rust/crates/sim-test/tests/colony_knowledge.rs` | `b0bf75785b66b5e616acf0004fdc43d40ce3922a9a0f7317bcb756e19592d299` | `c861514cadadcd2ada2396a7a4db6c70db0b1fa46e0ef0036ad05127e9af2145` |
| `rust/crates/sim-test/tests/engine_space_ffi.rs` | `bcde222cb1c91f8db9620725c06ce4c04b08f68de5abbc0f5fef27f96e4a7cd8` | `fba1e41c2f07e0fda352c4d6fa27451bced1936117895fefcda1ea23e277ba0d` |
| `tools/v8/test_bridge_api.py` | `083c345fff95c4b87c7fc0a474fcf99dc904ba89ab1ac408ec441b2f9bccadec` | `8d3ca840f3045bfcc6ea2e78179de80df0078a1757f5685d17df55a4c947f37e` |
| `tools/v8/test_locale_trf.gd` | `e4ab874bf26b7b21e65a699f50363118980763cf3ef60fa4f13aafd387320f04` | `5df1a635a14c7fba345371fb459e69b2c69c302d00e0c7aafa1c5b5f54f424cd` |

A later completed full-gate receipt must be recorded separately; never overwrite gate1. The independent documentation audit used read-only inspection and hashing only.
