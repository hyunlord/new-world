# Section 16-δ — need stagger + acceleration (synchronized burst → continuous flow)

> Governance: Stage 65 (`8f069f57`) → Stage 66. Lane: `--quick` (only `world_node.rs` sim-bridge bootstrap + new sim-test harness). No ENV-BYPASS.
> Acts on the Stage-65 investigation (commit `8f069f57`): the gathering loop works, but all 64 agents are synchronized and slow.

---

## Section 1: Implementation Intent

The Stage-65 investigation (real `cargo test`, not speculation) established: the α0+α+β gathering loop is **correct** (a forced-hungry agent reaches food in 5 ticks and consumes). The "agents move then stop / unclear if they go to resources" symptom is a **tuning** problem, not a bug:

- All 64 bootstrap agents get **identical** initial need values (`0.0`) and **identical** growth rates (Hunger 0.02 / Thirst 0.03 / Sleep 0.01). So they all breach the threshold on the **same tick** → a synchronized burst (measured: **peak 64 simultaneous `Seeking{Water}` at tick 1667**), rush to resources together, consume, and all return to Idle together.
- First breach is slow (Thirst at tick 1667 ≈ **55 s** at 30 TPS), so agents spend ~95 % of the time in aimless Idle Brownian motion.

**δ fix (the investigation's recommendation #1):** give each agent a **deterministic, staggered initial need value** (so they breach at different times → a continuous trickle of agents heading to resources instead of one synchronized burst), and **modestly raise growth rates** (so the first resource trip happens in ~tens of seconds, not ~a minute). Staggering is the primary fix; the rate bump is secondary. **No gathering-loop change** (α0/α/β untouched) — only the bootstrap's initial need values and growth rates.

---

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly two files. Only `bootstrap_spawn_agents`'s per-agent need attachment changes. The gathering loop (movement/decision/SeekTarget), need component code, thresholds, and resource substrate are NOT touched. No new locale keys.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-bridge/src/ffi/world_node.rs` | In `bootstrap_spawn_agents`, replace the identical `Hunger::new(0.0, 0.02)` / `Thirst::new(0.0, 0.03)` / `Sleep::new(0.0, 0.01)` with **per-agent staggered initial values** (deterministic, in `0..=BOOTSTRAP_NEED_STAGGER_MAX` where the cap is `< 50`) and **new growth-rate consts** (Thirst > Hunger > Sleep). Add the supporting `const`s near the existing `BOOTSTRAP_*`. The 12 source-tile seeding and everything else in the fn stays. |
| 2 | `rust/crates/sim-test/tests/harness_s16_delta_need_stagger.rs` | **New** harness, ≥6 assertions (stagger spread, determinism, all-Idle-at-bootstrap preserved, **breach de-synchronization**, rates, loop preserved). |

**Locked values:**
- `const BOOTSTRAP_NEED_STAGGER_MAX: u64 = 45;` — initial need values land in `0..=45`, strictly below the `50` threshold so **every agent is still Idle immediately after bootstrap** (preserves `s16_alpha0:A13` "64 agents all Idle").
- `const BOOTSTRAP_HUNGER_RATE: f32 = 0.05;` · `const BOOTSTRAP_THIRST_RATE: f64 = 0.08;` · `const BOOTSTRAP_SLEEP_RATE: f64 = 0.03;` (preserves Thirst > Hunger > Sleep ordering). Social stays `Social::new(0.0, 0.04)` (unstaggered — `Seeking{Agent}` has no `SeekTarget` so it doesn't drive resource movement; keeping it simple).
- Deterministic derivation: reuse `MovementRng` (the existing public splitmix64) seeded from a **salted** per-agent seed so the agent's own movement RNG stream (`MovementRng::new(seed)`) is **unchanged**.

**Scope boundary — NOT in this ticket:** Idle Brownian tuning (investigation rec #2 — later), any change to movement.rs / agent_decision.rs / SeekTarget / need component structs / thresholds / resource substrate, starvation/death/reproduction, per-kind resource sprites, any sim-core/sim-systems/sim-engine change, new locale keys.

---

## Section 3: How to Implement

In `bootstrap_spawn_agents`, inside the `for j … for i …` loop, after computing `seed`:
```rust
// V7 Section 16-δ — per-agent staggered initial needs (deterministic).
// Derived from a SALTED seed via the existing splitmix64 (MovementRng) so
// the agent's own movement RNG stream (MovementRng::new(seed)) is unchanged.
// Cap < 50 (threshold) → every agent is still Idle right after bootstrap.
let mut need_rng = MovementRng::new(seed ^ BOOTSTRAP_NEED_STAGGER_SALT);
let span = BOOTSTRAP_NEED_STAGGER_MAX + 1; // 0..=MAX
let h0 = (need_rng.next_u64() % span) as f32;
let t0 = (need_rng.next_u64() % span) as f64;
let sl0 = (need_rng.next_u64() % span) as f64;
// …
.insert(entity, (
    MovementRng::new(seed),
    AgentState::Idle,
    Hunger::new(h0, BOOTSTRAP_HUNGER_RATE),
    Thirst::new(t0, BOOTSTRAP_THIRST_RATE),
    Sleep::new(sl0, BOOTSTRAP_SLEEP_RATE),
    Social::new(0.0, 0.04),
    Memory::new(),
))
```
Add consts near `BOOTSTRAP_RNG_BASE`:
```rust
const BOOTSTRAP_NEED_STAGGER_MAX: u64 = 45;
const BOOTSTRAP_NEED_STAGGER_SALT: u64 = 0x5EED_5A66_E0DE_0001; // distinct from movement seed
const BOOTSTRAP_HUNGER_RATE: f32 = 0.05;
const BOOTSTRAP_THIRST_RATE: f64 = 0.08;
const BOOTSTRAP_SLEEP_RATE: f64 = 0.03;
```
Update the existing bootstrap comment (the one mentioning "Default growth rates: Hunger 0.02 …") to describe the new staggered/accelerated values and why the cap is < 50 (A13 preservation). `MovementRng` is already imported.

**Why this works (effect to expect):** with stagger 0..=45 and Thirst rate 0.08, an agent starting Thirst≈45 breaches at `(50-45)/0.08 ≈ 62` ticks (~2 s) while one starting ≈0 breaches at `50/0.08 ≈ 625` ticks (~21 s) → first water trips spread across ~62–625 ticks instead of all at 1667. Continuous flow.

## T2 — harness (`harness_s16_delta_need_stagger.rs`)
Replicate production: `SimEngine::new(64,64,MaterialRegistry::new())` + `register_default_runtime_systems` + `bootstrap_spawn_agents` (all pub). ≥6 assertions:

1. **Stagger spread:** collect all 64 agents' bootstrap `Hunger.value` (and Thirst/Sleep); assert they are **not all equal** (≥ ~10 distinct values) and **all within `0..=45`**.
2. **Determinism:** two independently-bootstrapped engines produce **identical** per-agent need values (seed-derived).
3. **All Idle at bootstrap (A13 preserved):** immediately after bootstrap, all 64 agents are `AgentState::Idle` (because every initial value < 50).
4. **★ Breach de-synchronization (headline):** run ~700 full ticks; record the first tick each agent enters `Seeking`; assert the first-Seek ticks are **spread** (max − min ≥ ~200 ticks AND ≥ ~10 distinct ticks) — i.e. NOT a single synchronized burst. (Contrast: pre-δ all 64 breached on the same tick.)
5. **Accelerated rates:** a bootstrap agent's growth rate matches the new consts (e.g. observe Thirst rising ~0.08/tick, or assert the rate field), and first Seeking happens well before the old tick-1667 (e.g. some agent Seeks within ~700 ticks).
6. **Loop preserved:** the gathering loop still completes — e.g. force/observe an agent through `Seeking → Consuming` (need drops), confirming δ didn't break α0/α/β.

---

## Section 4: Dispatch Plan

| Ticket | Concern | Routing | Depends on |
|--------|---------|---------|-----------|
| T1 | bootstrap stagger + rate consts | 🟢 DISPATCH | — |
| T2 | harness | 🟢 DISPATCH | T1 |

Dispatch 100%. Single-file behavior tweak + its harness.

---

## Section 5: Localization Checklist

**No new localization keys.** Simulation bootstrap tuning only.

---

## Section 6: Verification & Notion

**Gate:** `cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
**Regression (must stay green — A13 + loop):**
```bash
cd rust && cargo test -p sim-test --test harness_s16_alpha0_resource_substrate \
  --test harness_s16_alpha_seektarget --test harness_s16_beta_movement \
  --test harness_s16_gamma_visual --test harness_s16_delta_need_stagger -- --nocapture
```
**Pipeline:** `bash tools/harness/harness_pipeline.sh s16-delta-need-stagger .harness/prompts/s16-delta-need-stagger.md --quick`. (Lane `--quick`: only sim-bridge `.rs` + sim-test. Rust guards retired in Stage 61.)

**Honest disclosure (include in report):** δ changes only the bootstrap's initial need values (now staggered, cap 45 < 50) and growth rates (Thirst 0.08 / Hunger 0.05 / Sleep 0.03). It does NOT touch the gathering loop. The continuous-flow effect (agents heading to resources at staggered times) is confirmed by the breach-de-synchronization harness assertion + a **windowed run** (the authoritative visual check). Idle Brownian is still aimless (investigation rec #2, deferred). Rates are a first tuning pass — may need windowed adjustment.

**Governance chain:** Stage 65 `8f069f57` → Stage 66 (this δ). After APPROVE + commit, a **windowed run** confirms agents now head to resources at staggered times (press `1` for 0.25× per γ). NOT auto-proceeded. Pre-existing `shelter-ring-center-fix-v1` ENV-BYPASS debt remains open.
