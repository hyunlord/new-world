# Count-guard: replace nested cargo with static source count (Generator-stall root fix)

## Section 1: Implementation Intent

**Why:** `harness_p8_beta_a26_test_count_regression_guard`
(`rust/crates/sim-test/tests/harness_p8_beta_memory_system.rs`) spawns a NESTED
`cargo test --workspace --lib --bins --tests -- --list` to count tests. A prior fix
excluded doctests (rustdoc cost) — but the nested `cargo … --list` is STILL ~13 min when
run COLD and NESTED inside the outer gate's `cargo test --workspace` (build-lock contention
while enumerating ~150 test binaries). Because this guard runs inside every `cargo test
--workspace` — including the Generator's self-gate — it is a primary contributor to the
Generator 900/1800s-timeout "stall". The nested-cargo-in-cargo pattern is the root.

**Approach:** The guard's real job is "detect silent test REMOVAL". A `#[test]` function
removed from the workspace is, by definition, a `#[test]` attribute gone from a source
file — so counting `#[test]` attributes by walking the source tree is a MORE DIRECT measure
than enumerating compiled binaries, and it spawns NO subprocess (file reads only, ~tens of
ms even cold). This eliminates the nested cargo entirely.

**Measured:** line-anchored `#[test]` count across `rust/crates/*/{src,tests}` = **1637**
(sim-core 279, sim-engine 11, sim-systems 95, sim-test 1252, sim-bridge 0), essentially the
same as the old cargo `--list` count (~1630) — so the method swap does not move the number.

**Tradeoff:** The static count misses macro-GENERATED tests (none significant here) and
counts by attribute, not by compiled symbol. For a silent-removal tripwire that is the
correct, faster signal. Floor is retightened from the absurdly-loose 804 to ~1600 so it
actually catches removal of a few dozen tests while tolerating minor refactor churn.

## Section 2: What to Build

**File:** `rust/crates/sim-test/tests/harness_p8_beta_memory_system.rs` — rewrite ONLY the
`harness_p8_beta_a26_test_count_regression_guard` test + its baseline const(s). Touch
nothing else in the file.

1. Replace the `BASELINE_TEST_COUNT` (787) + `MIN_NEW_TESTS` (17) consts with a single:
   ```rust
   /// Static `#[test]` floor (silent-removal tripwire). Current workspace count
   /// is ~1637; this floor tolerates minor refactor churn while catching the
   /// silent removal of dozens of tests. Retightened from the old loose 804.
   const STATIC_TEST_FLOOR: u32 = 1600;
   ```
2. Rewrite the test body to:
   - Compute the workspace root from `CARGO_MANIFEST_DIR` (sim-test → crates → rust).
   - Recursively walk every `crates/<crate>/src` and `crates/<crate>/tests` directory
     (a small `fn walk(dir, &mut Vec<PathBuf>)` using `std::fs::read_dir`; recurse into
     subdirs; collect `*.rs` files). No external crate (no `walkdir`).
   - For each `.rs` file, read to string, count lines where
     `line.trim_start().starts_with("#[test]")` (line-anchored → ignores `#[test]` inside
     a string/inline). Sum across all files.
   - `assert!(count >= STATIC_TEST_FLOOR, "…silent test removal: {count} < floor {STATIC_TEST_FLOOR}…")`.
   - NO `std::process::Command`, NO `cargo`, NO `--list` anywhere in the test.
   - Keep the `#[test]` attribute + function name `harness_p8_beta_a26_test_count_regression_guard`
     (the name is referenced by streak/audit tooling; do not rename).

**Scope boundary — DO NOT:** touch any other test in the file; change other harnesses;
modify `tools/harness/*`; change the Generator timeout; touch sim-core/systems/engine/bridge.

## Section 3: How to Implement

1. Edit the consts (replace the two with `STATIC_TEST_FLOOR`).
2. Rewrite the test:
   ```rust
   fn collect_rs(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
       let Ok(entries) = std::fs::read_dir(dir) else { return; };
       for e in entries.flatten() {
           let p = e.path();
           if p.is_dir() { collect_rs(&p, out); }
           else if p.extension().is_some_and(|x| x == "rs") { out.push(p); }
       }
   }
   ```
   Workspace root = `Path::new(env!("CARGO_MANIFEST_DIR")).parent().parent()` (sim-test →
   crates → rust). Walk `rust_root.join("crates")`. Count `#[test]` lines. Assert ≥ floor.
3. Verify the count locally is ~1637 and `1637 >= 1600`.

**Feature harness** `rust/crates/sim-test/tests/harness_count_guard_static.rs` (new) —
verifies the rewrite's properties without re-running the slow path:
- A1 (Type A — no nested cargo): read the source of `harness_p8_beta_memory_system.rs`;
  assert the `a26` region contains NO `Command::new("cargo")`, NO `"--list"`, NO
  `std::process::Command`. (The nested-cargo anti-pattern is gone.)
- A2 (Type A — static count sound): independently walk `crates/*/{src,tests}` and count
  line-anchored `#[test]`; assert the count `>= STATIC_TEST_FLOOR` (1600) AND `> 1500`
  (sanity: the walk found the tree, not an empty dir).
- A3 (Type D — floor is meaningfully tight): assert `STATIC_TEST_FLOOR` is in `[1500, actual_count]`
  — i.e. below the current count (so removals trip it) but not absurdly loose like 804.
  Read the floor from source; compare to the independent count.
- A4 (Type A — fast / no subprocess): the independent walk completes (implicitly fast; assert
  it returns in the test without spawning a process — i.e. the count is from file reads).
- A5 (Type A — name preserved): the `a26` function name
  `harness_p8_beta_a26_test_count_regression_guard` still exists in the file.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | rewrite a26 (static walk) + const | 🔴 DIRECT | — (locked-test edit, authorized by this task) |
| T2 | `harness_count_guard_static.rs` meta-verify | 🟢 DISPATCH | T1 |

DIRECT for T1: editing the locked count-guard test itself — this task explicitly authorizes it.

## Section 5: Localization Checklist

No new localization keys. (Pure test-infra change; no user-visible text.)

## Section 6: Verification & Notion

```bash
cd rust && cargo test -p sim-test --test harness_p8_beta_memory_system harness_p8_beta_a26 -- --nocapture
# a26 must now finish in well under a second (no subprocess), even cold.
cd rust && cargo test -p sim-test --test harness_count_guard_static -- --nocapture
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
```

Expected: a26 passes near-instantly (no nested cargo); meta-harness A1–A5 pass; workspace
green; clippy clean. The whole `cargo test --workspace` no longer pays the ~13 min
count-guard nested-cargo cost → the Generator self-gate (and Step 0, and the pipeline gate)
become fast for all future features, especially sim-core (foundation) changes.

Notion: harness infra — count-guard nested-cargo eliminated (static source count); Generator
stall root-fixed; Direction-2 slice 2-2 (pickup behavior) is next and should run without the
eval-only recovery dance.
