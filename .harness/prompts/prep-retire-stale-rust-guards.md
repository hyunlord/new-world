# PREP — Retire stale "zero-Rust" anti-regression guards (Stage 61)

> Governance: Stage 60 (`68d68e85`) → Stage 61 (prep). Lane: `--quick` (test-only).
> Unblocks: S16-α0 + all future Rust backend work. No ENV-BYPASS.

---

## Section 1: Implementation Intent

Four anti-regression guards encode the P12/P13/P14 **GDScript-only-phase** scope promises as **PERMANENT global git-diff assertions**. Each enumerates `git diff origin/lead/main...HEAD` + `git status --porcelain` over the *entire current working tree* and FAILs if any path matches a forbidden Rust-crate prefix (verified against `68d68e85`):

| Guard fn | File | `forbidden_prefixes` |
|---|---|---|
| `harness_p12_alpha_a17_no_rust_crate_source_modified` | `harness_p12_alpha_camera_zoom.rs` | `sim-core/src`, `sim-systems/src`, `sim-engine/src`, `sim-data/src` |
| `harness_p12_beta_a24_no_rust_crate_modifications_in_scope_paths` | `harness_p12_beta_terrain.rs` | `sim-core/`, `sim-systems/`, … |
| `harness_p13_epsilon_a16_no_rust_simulation_crate_modification` | `harness_p13_epsilon_bootstrap_seed.rs` | `sim-core/`, `sim-bridge/`, … |
| `harness_p14_beta_a22_no_rust_crate_modifications` | `harness_p14_beta_resource_building_variety.rs` | `sim-core/`, `sim-systems/`, … |

**Why this is a bug, not a feature.** A guard's real job — "did *this phase* touch Rust crates?" — is a one-time check answerable at the phase's merge commit, and that answer is **permanently recorded in git history**. Re-asserting it against *every future working tree* means **any uncommitted Rust change fails them**. Section 16+ is explicitly Rust simulation work (CLAUDE.md: "ALL simulation logic is Rust"), so these guards make all backend work impossible. S16-α0 is the first Rust feature since they landed — the first to hit the wall.

**Why retiring is a fix, not a bypass.** The guards themselves are defective (a phase-local promise mis-encoded as a permanent global assertion). Forward per-feature scope-creep protection is **already provided** by the pipeline's F-Phase-A scope-semantic guard + the Codex Evaluator, which review every feature's diff against its authorized file list. Removing the stale guards loses no real protection — it removes a false barrier.

Separately, `harness_p14_gamma_a23_world_renderer_no_extra_ffi_added` (an FFI **allowlist**, not a global block) must gain `get_resource_snapshot` so the upcoming S16-α0 renderer FFI is authorized — the established pattern (A23 already allowlists each phase's new FFI, e.g. `get_agent_detail`).

---

## Section 2: What to Build  ⚠️ LOCKED SCOPE — test files only ⚠️

**5 files, all under `rust/crates/sim-test/tests/`. NO production code. NO other guards. Preserve every OTHER assertion in each file.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs` | Retire **only** `harness_p12_alpha_a17_no_rust_crate_source_modified`. Preserve all other tests (e.g. Camera2D zoom, D1 STATE_TINTS palette, A18 shader/locale). |
| 2 | `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs` | Retire **only** `harness_p12_beta_a24_no_rust_crate_modifications_in_scope_paths`. Preserve all other tests (e.g. terrain, Camera2D zoom + camera_controller attachment). |
| 3 | `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs` | Retire **only** `harness_p13_epsilon_a16_no_rust_simulation_crate_modification`. Preserve all other tests (e.g. bootstrap seed determinism). |
| 4 | `rust/crates/sim-test/tests/harness_p14_beta_resource_building_variety.rs` | Retire **only** `harness_p14_beta_a22_no_rust_crate_modifications`. Preserve all other tests (e.g. resource/building variety). |
| 5 | `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs` | Extend **only** the `allowed` BTreeSet in `harness_p14_gamma_a23_world_renderer_no_extra_ffi_added` with `"get_resource_snapshot"`. Change nothing else (the `get_agent_detail` positive check stays). |

**Scope boundary — NOT in this prep (do not touch):** S16-α0 production code (separate ticket, already verified, re-run after this), any production crate (`sim-core`/`sim-systems`/`sim-engine`/`sim-bridge`/`sim-data`), any guard not in the table above, F-Phase-A / Evaluator / pipeline logic, `world_renderer.gd`.

---

## Section 3: How to Implement

### Retire (files 1–4)
For each of the four named guard fns, replace its body (the `std::process::Command::new("git")` file enumeration + `forbidden_prefixes` filter + `assert!`) with a documented retirement notice + a trivial `println!`. **Keep the `#[test]` attribute and the fn signature** (stable test registration; in-file rationale). Pattern (apply phase-specific wording to each):
```rust
#[test]
fn harness_p12_alpha_a17_no_rust_crate_source_modified() {
    // RETIRED (S16 prep — Stage 61). This guard mis-encoded P12-α's
    // GDScript-only-phase scope promise as a PERMANENT global git-diff check
    // (forbidden_prefixes over sim-core/sim-systems/sim-engine/sim-data src),
    // so it FAILed on ANY uncommitted Rust change — blocking ALL future Rust
    // backend work (Section 16+). P12-α's actual Rust-untouched state was
    // verified at its merge commit and persists in git history; re-asserting
    // it against every future working tree is a design error. Forward
    // per-feature scope-creep protection is now provided by the pipeline's
    // F-Phase-A scope-semantic guard + the Codex Evaluator. Retiring restores
    // a no-op pass without losing real coverage.
    // See .harness/prompts/prep-retire-stale-rust-guards.md.
    println!("[P12-α A17] RETIRED — stale global Rust-block guard; forward protection via F-Phase-A + Evaluator");
}
```
**Clippy discipline:** `cargo clippy --workspace --all-targets -- -D warnings` MUST stay clean. If removing a guard body orphans a *file-local* helper used ONLY by that guard, remove the helper too (or annotate `#[allow(dead_code)]` with a one-line note). Shared helpers used by other tests in the file (e.g. `project_root`, `read_*_src`, `strip_gd_comments`) MUST stay.

### A23 allowlist (file 5)
In `harness_p14_gamma_a23_world_renderer_no_extra_ffi_added`, add exactly one entry to the `allowed` BTreeSet, right after the `get_agent_detail` line:
```rust
    "get_agent_detail", // new γ FFI
    "get_resource_snapshot", // V7 Section 16-α0 renderer FFI (substrate markers)
```
Do not alter anything else in A23. `world_renderer.gd` is NOT touched by this prep, so the guard's `found` set is unchanged and remains ⊆ the expanded `allowed`.

---

## Section 4: Dispatch Plan

| Ticket | Concern | Routing | Depends on |
|--------|---------|---------|-----------|
| T1 | Retire A17 / A24 / A16 / A22 (files 1–4) | 🟢 DISPATCH | — |
| T2 | Extend A23 allowlist (file 5) | 🟢 DISPATCH | — |

Dispatch 100%. Test-only; no production wiring, no shared-interface coordination.

---

## Section 5: Localization Checklist

**No new localization keys.** Test-only change.

---

## Section 6: Verification & Notion

**Gate:**
```bash
cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
```
Expected: full workspace suite green; the 4 retired guards pass as documented no-ops; A23 passes with the expanded allowlist. **Self-consistency:** this prep modifies ONLY `sim-test/tests/`, which appears in NO guard's `forbidden_prefixes`, so it does not trip any still-active guard during its own gate run.

**Pipeline:**
```bash
bash tools/harness/harness_pipeline.sh prep-retire-stale-rust-guards \
  .harness/prompts/prep-retire-stale-rust-guards.md --quick
```

**Governance chain:** Stage 60 `68d68e85` → Stage 61 (prep). After APPROVE + commit, S16-α0 re-runs against the modernized guards (separate step — NOT auto-proceeded).

**Honest disclosure (include in final report):** this prep changes ONLY test files — it retires 4 stale global Rust-block assertions (a bug fix: phase promise mis-encoded as permanent global, forward protection retained via F-Phase-A + Evaluator) and widens one FFI allowlist by a single entry. Zero production behavior change.
