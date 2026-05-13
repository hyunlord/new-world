# T7.10.E — Spiritual k drift fix (governance integrity, IUS spec align)

> Lane: `--quick` (sim-systems IUS constant + sim-test assertion ranges + minimal GDScript renderer comment)
> Scope: SINGLE constant correction. Spiritual decay constant in
> `runtime/influence/update.rs` IUS drifted from the Phase 0 spec
> (`channel.rs:69` documents `k = 0.08`). Production used `k = 0.10`
> (`SPIRITUAL_DECAY_PER_STEP = 0.904_837`). This commit aligns the IUS
> to the documented Phase 0 spec.
> Governance: v3.3.16. Restores **spec-as-source-of-truth** invariant —
> `channel.rs` doc is the repo-internal Phase 0 authority; runtime code
> must mirror it. The T7.10.F audit detected this drift via grep sweep
> across all 6 stamped channels.

---

## Section 1 — Implementation Intent

T7.10.E landed Spiritual BFS exponential-decay wiring on 2026-05-13 with
`SPIRITUAL_DECAY_PER_STEP = 0.904_837` (k=0.10). Subsequent audit during
T7.10.F discovered that `channel.rs:69` Phase 0 spec documents
`Decay: exponential, k = 0.08. Aggregation: Max.` for `Spiritual = 6`.
All other 5 stamped channels (Warmth, Light, Noise, Danger, Beauty)
match their `channel.rs` spec exactly. Spiritual was the sole drift.

This commit corrects the drift:

1. **IUS constant** — `runtime/influence/update.rs` `SPIRITUAL_DECAY_PER_STEP`
   changes from `0.904_837` (exp(-0.10)) to `0.923_116` (exp(-0.08)).
2. **Harness assertions** — `harness_t7_10_e_spiritual_bfs_wiring.rs`
   ranges for E2/E3/E5 shift to k=0.08 chain math; E4 strict monotone
   invariant unchanged (still holds at gentler decay).
3. **Regression-guard messages** — A/B/C/D/F harnesses + phase2_substantial
   update single-line `k=0.10` references in their error messages
   to `k=0.08` so future drift detection points to the current spec.
4. **Renderer comment** — `world_renderer.gd` line 13 SPACE-cycle comment
   updates from `k=0.10` to `k=0.08`.

**No production logic flow changes** — only the decay constant value
moves to match the Phase 0 spec. `propagate_bfs` primitive itself,
IUS dispatch shell, and Phase 2 channel-wiring topology are unchanged.

**Why this matters**:
- Restores **spec = source of truth** invariant (governance integrity).
- Closes the audit gap the T7.10.F grep sweep exposed.
- Prevents future channel additions from copying the stale k=0.10 number
  as if it were canonical.
- Spiritual ritual influence carries further than thermal heat
  (k=0.08 < k=0.15), matching the Phase 0 design intent.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| Phase 0 spec for Spiritual decay | `rust/crates/sim-core/src/influence/channel.rs:69` | `k = 0.08` |
| Pre-fix IUS constant | `rust/crates/sim-systems/src/runtime/influence/update.rs:173` (pre-fix) | `0.904_837` (exp(-0.10)) |
| Post-fix IUS constant | same line (post-fix) | `0.923_116` (exp(-0.08)) |
| BFS primitive used | `rust/crates/sim-systems/src/runtime/influence/propagate.rs:75` | `propagate_bfs` (unchanged) |
| Other stamped channels' decay | `channel.rs:33-75` | Warmth k=0.15, Light shadowcast, Noise α=15, Danger α=5, Beauty k=0.12 (all unchanged) |
| Initial intensity at source | T7.10.E spec | 200 |
| Chain math k=0.08 at d=1 | `floor(200 * 0.923116)` | 184 |
| Chain math k=0.08 at d=2 | `floor(184 * 0.923116)` | 170 |
| Chain math k=0.08 at d=15 | `floor(200 * 0.923116^15)` | ≈60 |

---

## Section 3 — Files touched

**Production (1 file):**
- `rust/crates/sim-systems/src/runtime/influence/update.rs`
  - Constant `SPIRITUAL_DECAY_PER_STEP`: `0.904_837` → `0.923_116`
  - Module/function docs: 3 occurrences of `k=0.10` → `k=0.08` with
    channel.rs:69 reference

**Harness (7 files):**
- `rust/crates/sim-test/tests/harness_t7_10_e_spiritual_bfs_wiring.rs`
  - Header: drift fix note + k=0.08
  - E2 range: `[179,183]` → `[182,186]` (target 184)
  - E3 range: `[161,165]` → `[168,172]` (target 170)
  - E5 range: `[40,50]` → `[55,65]` (target 60)
  - E4 doc: decay multiplier 0.923116, error msg refs "Spiritual k=0.08"
  - Stale `k=0.10` references kept in error messages as drift-detection hints
- `rust/crates/sim-test/tests/harness_t7_10_a_warmth_wiring.rs` (line 206)
- `rust/crates/sim-test/tests/harness_t7_10_b_light_shadowcast_wiring.rs` (line 247)
- `rust/crates/sim-test/tests/harness_t7_10_c_noise_linear_wiring.rs` (line 283)
- `rust/crates/sim-test/tests/harness_t7_10_d_danger_linear_wiring.rs` (lines 284, 296)
- `rust/crates/sim-test/tests/harness_t7_10_f_beauty_bfs_wiring.rs` (F2/F3/F5 discriminator math: 163→170, 45→60)
- `rust/crates/sim-test/tests/harness_phase2_substantial.rs` (line 907)

**Renderer (1 file):**
- `scripts/ui/world_renderer.gd` (line 13 comment k=0.10 → k=0.08)

---

## Section 4 — Test plan (RED → GREEN)

E-series tests rerun with new ranges:

- **E1** (channel registration): unchanged — still asserts Spiritual in
  registered-channel mask.
- **E2** (d=1 chain): NEW range `[182,186]` target 184. Discriminates
  against Warmth k=0.15 (~172) and stale k=0.10 (~180).
- **E3** (d=2 chain): NEW range `[168,172]` target 170. Discriminates
  against linear-α=5 (~190) and stale k=0.10 (~163).
- **E4** (strict monotone d=0..=14): unchanged invariant; verified k=0.08
  inter-step gap at d=14→15 = 65→60 ≈ 5 unit drop ≥ 1, strict `>` safe.
- **E5** (boundary d=15): NEW range `[55,65]` target 60. Discriminates
  against Warmth k=0.15 (~21) and stale k=0.10 (~45).

All 6 channel wiring tests must remain CLEAN (A/B/C/D/E/F). Phase 2
substantial harness must remain CLEAN.

---

## Section 5 — Acceptance criteria

1. `cargo build --workspace` clean
2. `cargo test --workspace` all green (no new failures, baseline
   tolerance per `.harness/baseline/known_failures.txt` if any)
3. `cargo clippy --workspace --all-targets -- -D warnings` clean
4. T7.10.A-F + phase2_substantial harness all CLEAN
5. No code-quality regressions
6. VLM analysis: PASS or WARNING (stone-age sim) acceptable
7. Pipeline score ≥ 90 (CLAUDE.md §7.1)

---

## Section 6 — Risk register

| Risk | Mitigation |
|------|------------|
| Other downstream code reads `SPIRITUAL_DECAY_PER_STEP` and depends on 0.904_837 | grep sweep: only `update.rs` references this constant. No external dependents. |
| Strict monotone invariant E4 fails at gentler decay | Verified: 200 * 0.923116^14 ≈ 65, 200 * 0.923116^15 ≈ 60, gap = 5 ≥ 1, safe. |
| f32 vs f64 rounding pushes a sample one unit outside the new range | Ranges have ±2 / ±2 / ±5 tolerances mirroring original harness style. |
| GDScript renderer breaks (no scene reload before VLM) | Only a comment line changes — no behavior. |

---

## Section 7 — Out of scope

- Other channel constant audits (already verified clean during F's grep sweep)
- Beauty k=0.12 → any other value (Phase 0 spec, no drift)
- New channel additions
- Visual renderer cycle order change
- Phase 0 spec doc updates (channel.rs:69 IS the spec; code mirrors it)
- Phase 3 sprite/agent rendering work (separate ticket)
