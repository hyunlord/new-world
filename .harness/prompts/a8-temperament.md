# A-8: Temperament→Behavior Pipeline — Connect TCI to Cognition + Shift Rules

## Section 1: Implementation Intent

### Why does this exist?

The Temperament component (`sim-core/src/temperament.rs`) exists with full TCI 4-axis data (NS/HA/RD/P), PRS weight derivation from HEXACO, and bias matrix support. The `steering.rs` system already reads temperament for movement weights. **But temperament has zero influence on cognition (action selection), no shift rules execute at runtime, no SimBridge getter exposes it to UI, and no harness test verifies the pipeline works.**

This means an agent with NS=0.9 (highly exploratory) and an agent with NS=0.1 (rigid/cautious) make **identical action decisions** — only their movement weights differ. The entire psychology stack from the 38D gene core through TCI is architecturally present but functionally disconnected from behavior.

### What this solves

After this feature:
1. **High-NS agents explore more, take risks** — cognition system biases action scores by temperament
2. **Traumatic events shift temperament** — `check_shift_rules()` actually runs on events
3. **UI shows TCI axes** — entity_detail_panel displays temperament archetype + 4 axis bars
4. **Harness verifies** — `harness_temperament_biases_behavior` confirms NS/HA affect action selection measurably

### Academic basis

Cloninger, Svrakic & Przybeck (1993) — TCI 4-axis maps to specific neurotransmitter systems and predicts concrete behavioral tendencies:
- High NS (dopamine) → exploratory approach, impulsive, rapid switching
- High HA (serotonin) → cautious, pessimistic, passive avoidance
- High RD (noradrenaline) → social reward seeking, attachment, conformity
- High P → industrious, perseverant despite frustration

---

## Section 2: What to Build

### Part A: Cognition bias (Rust — sim-systems)

**File: `rust/crates/sim-systems/src/runtime/cognition.rs`**

Add temperament to the action scoring query. Currently cognition queries `(&Personality, &Needs, &Emotion, ...)` — add `Option<&Temperament>` to the query.

Add a `temperament_action_bias()` function that returns a score modifier for each action type based on TCI axes:

```rust
fn temperament_action_bias(temperament: &Temperament, action: &ActionType) -> f64 {
    let t = &temperament.expressed;
    match action {
        // NS: explore, forage far, approach strangers
        ActionType::Explore => 0.3 * (t.ns - 0.5),
        ActionType::Forage => 0.15 * (t.ns - 0.5),

        // HA: flee, avoid danger, stay near settlement
        ActionType::Flee => 0.3 * (t.ha - 0.5),
        ActionType::Rest => 0.15 * (t.ha - 0.5),

        // RD: socialize, share, help
        ActionType::Socialize => 0.3 * (t.rd - 0.5),
        ActionType::Share => 0.2 * (t.rd - 0.5),

        // P: continue current task, build, craft
        ActionType::Build => 0.2 * (t.p - 0.5),
        ActionType::Craft => 0.15 * (t.p - 0.5),
        ActionType::GatherStone => 0.1 * (t.p - 0.5),

        _ => 0.0,
    }
}
```

The bias is centered at 0.5 (neutral) — agents with axis > 0.5 get positive bias, < 0.5 get negative bias. Max effect is ±0.15 per action, which nudges but doesn't dominate needs-based decisions.

**Integration point**: In the action scoring function where action candidates are evaluated, add:

```rust
let temperament_bias = temperament_opt
    .map(|t| temperament_action_bias(t, &action_type))
    .unwrap_or(0.0);
score += temperament_bias;
```

Find the exact location by searching for where `ActionType` candidates are scored/weighted in cognition.rs. If cognition.rs doesn't have a clear scoring function, check `psychology.rs` lines around `TODO(v3.1)` — the action selection may be in the psychology system.

### Part B: Shift rules execution (Rust — sim-systems)

**File: `rust/crates/sim-systems/src/runtime/psychology.rs`**

The `check_shift_rules()` method on Temperament is a stub that only logs. Implement actual shift execution:

1. Find where `SimEvent` / `SimEventType` events are processed in psychology.rs
2. When a shift-triggering event occurs (e.g., `FamilyDeathWitnessed`, `BattleSurvived`, `StarvationRecovered`), call `temperament.apply_shift()` with appropriate deltas
3. Log the shift to CausalLog

For initial implementation, hardcode 3 shift triggers (later these will come from RON data):

```rust
const TEMPERAMENT_SHIFTS: &[(&str, f64, f64, f64, f64)] = &[
    // (event_key, ns_delta, ha_delta, rd_delta, p_delta)
    ("family_death_witnessed", -0.10, 0.15, -0.05, 0.0),    // trauma → cautious
    ("battle_survived",        0.05, -0.05, 0.0,   0.10),    // confidence → bold + persistent
    ("starvation_recovered",   0.0,   0.05,  0.0,  -0.05),   // scarcity anxiety
];
```

### Part C: SimBridge getter (Rust — sim-bridge)

**File: `rust/crates/sim-bridge/src/lib.rs`**

Add temperament data to entity detail. In the `runtime_get_entity_detail()` function, add TCI axes to the returned dictionary:

```rust
if let Ok(temperament) = world.get::<&Temperament>(entity) {
    dict.set("temperament_ns", temperament.expressed.ns);
    dict.set("temperament_ha", temperament.expressed.ha);
    dict.set("temperament_rd", temperament.expressed.rd);
    dict.set("temperament_p", temperament.expressed.p);
    dict.set("temperament_archetype", temperament.archetype_label_key().to_string());
    dict.set("temperament_awakened", temperament.awakened);
}
```

### Part D: UI display (GDScript)

**File: `scripts/ui/panels/entity_detail_panel.gd`**

In the "성격" (personality) tab or a new "기질" tab, display:
- Archetype label: `Locale.ltr(temperament_archetype)` → "다혈질" / "담즙질" / "점액질" / "우울질"
- 4 axis bars: NS / HA / RD / P, each 0.0-1.0 with colored fill
- "Awakened" badge if `temperament_awakened == true`

---

## Section 3: How to Implement

### Cognition integration — finding the right injection point

Search cognition.rs and psychology.rs for where actions are selected. Look for:
- `ActionType` enum usage in scoring/weighting
- `match` blocks on action types
- Functions that return an action choice
- The `TODO(v3.1)` comment in psychology.rs line 2

The temperament bias should be added as a **score modifier** after needs-based scoring but before final selection.

### ECS query pattern

```rust
for (entity, (behavior, needs, emotion, personality, temperament_opt)) in
    world.query::<(&mut Behavior, &Needs, &Emotion, &Personality, Option<&Temperament>)>().iter()
{
    if let Some(temperament) = temperament_opt {
        score += temperament_action_bias(temperament, &candidate_action);
    }
}
```

### Shift rule integration — event matching

Search psychology.rs for where events like death, battle, starvation are handled. The shift should happen **after** the event's emotion/stress effects but **before** behavior re-evaluation.

---

## Section 4: Dispatch Plan

| # | Ticket | File | Language | Mode | Depends On |
|---|--------|------|----------|:----:|:----------:|
| T1 | Temperament action bias function | sim-systems/src/runtime/cognition.rs or psychology.rs | Rust | DISPATCH | — |
| T2 | Shift rule execution (3 hardcoded triggers) | sim-systems/src/runtime/psychology.rs | Rust | DISPATCH | — |
| T3 | SimBridge TCI getter | sim-bridge/src/lib.rs | Rust | DIRECT | T1 |
| T4 | UI temperament display | entity_detail_panel.gd | GDScript | DISPATCH | T3 |
| T5 | Localization keys | localization/en.json + ko.json | — | DISPATCH | — |
| T6 | Harness test | sim-test/src/main.rs | Rust | DISPATCH | T1 |

---

## Section 5: Localization Checklist

| Key | JSON file | en value | ko value |
|-----|-----------|----------|----------|
| TEMPERAMENT_SANGUINE | both | Sanguine | 다혈질 |
| TEMPERAMENT_CHOLERIC | both | Choleric | 담즙질 |
| TEMPERAMENT_MELANCHOLIC | both | Melancholic | 우울질 |
| TEMPERAMENT_PHLEGMATIC | both | Phlegmatic | 점액질 |
| TEMPERAMENT_NS | both | Novelty Seeking | 새로움 추구 |
| TEMPERAMENT_HA | both | Harm Avoidance | 위험 회피 |
| TEMPERAMENT_RD | both | Reward Dependence | 보상 의존 |
| TEMPERAMENT_P | both | Persistence | 인내력 |
| TEMPERAMENT_AWAKENED | both | Awakened | 각성됨 |

---

## Section 6: Verification & Harness

### Gate command

```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

### Harness test (MANDATORY)

**Test name**: `harness_temperament_biases_behavior`

```rust
#[test]
fn harness_temperament_biases_behavior() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(2000);

    let world = engine.world();

    let mut high_ns_explore_count = 0u32;
    let mut low_ns_explore_count = 0u32;
    let mut high_ns_total = 0u32;
    let mut low_ns_total = 0u32;

    for (_entity, (behavior, temperament)) in
        world.query::<(&Behavior, &Temperament)>().iter()
    {
        let ns = temperament.expressed.ns;
        let is_exploring = behavior.current_action == ActionType::Explore
            || behavior.current_action == ActionType::Forage;

        if ns >= 0.7 {
            high_ns_total += 1;
            if is_exploring { high_ns_explore_count += 1; }
        } else if ns <= 0.3 {
            low_ns_total += 1;
            if is_exploring { low_ns_explore_count += 1; }
        }
    }

    // Type A: All temperament axes must be in [0, 1]
    for (_entity, temperament) in world.query::<&Temperament>().iter() {
        assert!(temperament.expressed.ns >= 0.0 && temperament.expressed.ns <= 1.0);
        assert!(temperament.expressed.ha >= 0.0 && temperament.expressed.ha <= 1.0);
        assert!(temperament.expressed.rd >= 0.0 && temperament.expressed.rd <= 1.0);
        assert!(temperament.expressed.p >= 0.0 && temperament.expressed.p <= 1.0);
    }

    // Type C (empirical seed-42): High-NS agents explore >= low-NS rate
    if high_ns_total > 0 && low_ns_total > 0 {
        let high_rate = high_ns_explore_count as f64 / high_ns_total as f64;
        let low_rate = low_ns_explore_count as f64 / low_ns_total as f64;
        assert!(
            high_rate >= low_rate,
            "High-NS agents should explore at least as much as low-NS: high={:.2} low={:.2}",
            high_rate, low_rate
        );
    }
}
```

---

## Section 7: In-Game Verification

1. 에이전트 선택 → entity_detail에서 기질 탭에 4축 바 + 아키타입 라벨 표시 확인
2. NS 높은 에이전트가 탐험/채집 행동을 더 자주 하는지 콘솔 로그 확인
3. 가족 사망 이벤트 발생 시 HA 상승 로그 확인
4. FPS 영향 없음, 콘솔 에러 0건
