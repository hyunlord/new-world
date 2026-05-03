# healing-system-v1: 3-Layer Healing System

## Feature Description

Extends HealthRuntimeSystem with natural clotting + natural healing (Layers 1-2)
and adds a Treat social action (Layer 3) to prevent population collapse after A3 wildlife combat.

### Layer 1: Natural Clotting (health.rs)
Inside `process_parts()`, after each BLEEDING drain tick where hp > 0:
- `bleed_rate` decrements by 1 (saturating_sub)
- When `bleed_rate == 0`: BLEEDING flag is cleared

### Layer 2: Natural Healing (health.rs)
After all bleed/infection logic, per part:
- If NOT BLEEDING, NOT INFECTED, hp > 0, hp < 100 → `hp += NATURAL_HEAL_RATE` (capped at 100)

### Layer 3: Treat Action (world.rs + cognition.rs)
New `ActionType::Treat` social care action:
- **cognition.rs**: Treat scored when `has_nearby_injured` (pre-collected Vec<(f64,f64)>) and energy >= 0.30; RD axis bonus
- **world.rs**: Two-phase completion — Phase 1 computes heal amount + pushes `treat_completions`, Phase 2 (after `drop(query)`) finds nearest injured target within `TREAT_RANGE` and applies heal to most-damaged living part; also accelerates clotting (`bleed_rate -= 2`)
- Heal formula: `TREAT_BASE_HEAL_AMOUNT + healing_level × TREAT_SKILL_HEAL_PER_LEVEL [+ TREAT_KNOWLEDGE_BONUS if knowledge_first_aid]`, capped at 80

## Config Constants Added (sim-core/config.rs)

- `NATURAL_HEAL_RATE: u8 = 1`
- `ACTION_TIMER_TREAT: i32 = 10`
- `TREAT_BASE_HEAL_AMOUNT: u8 = 30`
- `TREAT_SKILL_HEAL_PER_LEVEL: u8 = 3`
- `TREAT_KNOWLEDGE_BONUS: u8 = 20`
- `TREAT_RANGE: f64 = 1.5`
- `TREAT_XP_GAIN: f64 = 0.5`
- `TREAT_TARGET_PART_HP_THRESHOLD: u8 = 80`

## Files Changed

- `sim-core/src/enums.rs`: `ActionType::Treat` variant
- `sim-core/src/config.rs`: 8 new constants
- `sim-systems/src/runtime/health.rs`: natural clotting + natural healing in `process_parts()`
- `sim-systems/src/runtime/cognition.rs`: `injured_positions` pre-collection, `has_nearby_injured` param, Treat scoring, timer, bias
- `sim-systems/src/runtime/world.rs`: `treat_completions` Vec, Treat action handler, Phase 2 apply heal
- `sim-engine/src/frame_snapshot.rs`: `ActionType::Treat => 31` in `action_state_code()`
- `localization/en/actions.json`: `"ACTION_TREAT": "Treat"`
- `localization/ko/actions.json`: `"ACTION_TREAT": "치료"`
- `sim-test/src/main.rs`: 9 harness tests in `mod harness_b1_healing_system`

## Harness Tests (9)

1. `harness_b1_natural_clot_reduces_bleed_rate` — bleed_rate decrements 5→2 after 3 ticks (Type A)
2. `harness_b1_natural_clot_clears_bleeding_flag` — BLEEDING cleared when bleed_rate reaches 0 (Type A)
3. `harness_b1_natural_heal_restores_hp` — HP restores from 70 to >70 after 10 ticks (Type B)
4. `harness_b1_skills_api_xp_accumulates` — Skills.add_xp accumulates, get_level returns 0 (Type A)
5. `harness_b1_knowledge_api_has_knowledge` — AgentKnowledge.has_knowledge correct before/after learn() (Type A)
6. `harness_b1_treat_heals_injured_target` — Treat action raises target HP from 50 to >50 (Type B)
7. `harness_b1_treat_awards_xp_to_treater` — treater gains SKILL_HEALING XP > 0 (Type A)
8. `harness_b1_treat_skilled_heals_more` — level=10 treater heals more HP than level=0 (Type B)
9. `harness_b1_population_survives_clottable_wounds` — ≥50% of agents with BLEEDING brain hp=3 survive (Type D)

## Crates: sim-core, sim-systems, sim-engine, sim-test
