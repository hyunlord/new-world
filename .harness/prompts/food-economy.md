# Food Economy: 식량 고갈 버그 디버깅 및 수정

## Section 1: Implementation Intent

### 문제
tick 2200부터 settlement stockpile_food=0 영구 고갈, tick 3800에서 인구 감소 시작.
- tick 0: food=11.5
- tick 1400: food=24.0 (최고)
- tick 2200: food=0.4
- tick 2400 이후: food=0 (영구 고갈)
- 인구 31 중 아이 8명, teen 2명

### 근본 원인 분석
식량 수지 = Forage 생산 - childcare 소비 - birth 비용 - crafting 소비

**생산 (Forage):**
- FORAGE_STOCKPILE_YIELD = 2.0 / 24 ticks = 0.083 food/tick per forager
- 성인 ~21명, 비정규 forage 주기 ~114 ticks → 0.0175 food/tick/adult
- 총 생산: 21 × 0.0175 ≈ 0.37 food/tick (최대치, 실제는 더 낮음)

**소비 (Childcare):**
- CHILDCARE_TICK_INTERVAL = 10 (매 10틱)
- Child(8명): threshold 0.75, feed 0.50 → 매 ~130틱 급식 → 8 × 0.50/130 = 0.031
- Teen(2명): threshold 0.70, feed 0.60 → 매 ~160틱 급식 → 2 × 0.60/160 = 0.008
- childcare 합계: ~0.04 food/tick

**소비 (Birth):**
- BIRTH_FOOD_COST = 3.0 per birth
- 11 births over ~2000 ticks = 0.017 food/tick

**소비 (Crafting):**
- 레시피 중 food 소비 항목 존재

**문제:** 인구 증가에 따라 생산보다 소비가 우세해지는 tipping point 존재.
특히 성인들이 Build/Craft/Socialize 등 비식량 행동을 많이 하면 forage 빈도 감소.

### 해결 방향
1. FORAGE_STOCKPILE_YIELD 증가 (2.0 → 3.0)
2. gatherer Forage score multiplier 증가 (1.50 → 2.0)
3. food scarcity 시 모든 성인에게 forage score boost 추가
4. childcare feed amount 소폭 감소 (선택적)

---

## Section 2: What to Build

### Part A: Diagnostic Harness Test (sim-test)

**File: `rust/crates/sim-test/src/main.rs`**

```rust
#[test]
fn harness_food_economy_balance_4380() {
    // Run 4380 ticks, sample food every 200 ticks
    // Track: forage completions, childcare withdrawals, birth costs, crafting costs
    // Assert: food never stays at 0 for > 200 consecutive ticks
}
```

### Part B: Config Tuning (sim-core/config.rs)

1. `FORAGE_STOCKPILE_YIELD`: 2.0 → 3.0
2. New constant: `FOOD_SCARCITY_FORAGE_BOOST: f64 = 0.40`
3. New constant: `FOOD_SCARCITY_THRESHOLD_PER_CAPITA: f64 = 1.5`

### Part C: Behavior AI — Food Scarcity Response (cognition.rs)

When `settlement.stockpile_food / population < FOOD_SCARCITY_THRESHOLD_PER_CAPITA`:
- Add `FOOD_SCARCITY_FORAGE_BOOST` (0.40) to all adults' Forage score
- Effectively makes all adults prioritize foraging when food is scarce

### Part D: Gatherer Score Increase (cognition.rs)

Gatherer job multiplier: 1.50 → 2.0 for Forage action.

---

## Section 3: Acceptance Criteria

| ID | Assertion | Threshold |
|----|-----------|-----------|
| F1 | stockpile_food > 0 at tick 4380 | > 2.0 |
| F2 | No 200-tick window where food=0 continuously after tick 500 | true |
| F3 | Population ≥ 25 at tick 4380 (no starvation collapse) | ≥ 25 |
| F4 | Forage completion count over 4380 ticks | > 500 |
| F5 | food_produced > food_consumed over full run | ratio > 1.0 |

---

## Section 4: Scope

### Files to modify:
- `rust/crates/sim-core/src/config.rs` — new constants + yield increase
- `rust/crates/sim-systems/src/runtime/cognition.rs` — food scarcity boost, gatherer multiplier
- `rust/crates/sim-test/src/main.rs` — diagnostic harness test

### Files NOT to modify:
- `rust/crates/sim-systems/src/runtime/biology.rs` — childcare logic untouched
- `rust/crates/sim-systems/src/runtime/world.rs` — forage completion deposit untouched
- Any GDScript files
- Any bridge files
