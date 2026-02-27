# scripts/ai/ — CLAUDE.md

> AI decision-making: Utility AI, behavior selection, action execution.
> Currently a single BehaviorSystem. Will expand as AI complexity grows.

---

## Current Architecture

```
BehaviorSystem (SimulationSystem, prio=20, interval=5)
  └─ For each entity:
     1. Generate action candidates
     2. Score each action (Utility AI)
     3. Select highest-scoring action
     4. Execute action → emit events via SimulationBus
```

### Utility AI Scoring

Each action candidate receives a score based on weighted factors:

```gdscript
func score_action(entity: EntityData, action: ActionCandidate) -> float:
    var score: float = 0.0

    # Need fulfillment (highest weight)
    score += action.need_fulfillment * _get_need_urgency(entity, action.target_need)

    # Personality alignment
    score += _personality_modifier(entity, action) * GameConfig.AI_PERSONALITY_WEIGHT

    # Value alignment
    score += _value_modifier(entity, action) * GameConfig.AI_VALUE_WEIGHT

    # Emotion influence
    score += _emotion_modifier(entity, action) * GameConfig.AI_EMOTION_WEIGHT

    # Social context
    score += _social_modifier(entity, action) * GameConfig.AI_SOCIAL_WEIGHT

    return score
```

### Need Urgency Curve
```
urgency = (1.0 - need_value) ^ GameConfig.AI_URGENCY_EXPONENT
# Exponent > 1.0 → low needs are disproportionately urgent
# Default exponent: 2.0 → 0.3 need → urgency 0.49, 0.1 need → urgency 0.81
```

---

## Action Candidates

Actions are data-driven. Each action defines:
```gdscript
class ActionCandidate:
    var id: String           # "gather_food", "socialize", "rest", "build"
    var target_need: String  # primary need this fulfills
    var need_fulfillment: float  # how much it would restore
    var required_skill: String   # optional
    var target_position: Vector2i
    var target_entity_id: int    # for social actions
```

### Available Action Types (Current)
- **gather_food** → fulfills hunger
- **drink_water** → fulfills thirst
- **rest/sleep** → fulfills sleep
- **socialize** → fulfills belonging, intimacy
- **work** → fulfills competence, produces resources
- **build** → fulfills competence, creates buildings
- **explore** → fulfills autonomy, discovers resources
- **flee** → safety, triggered by fear emotion

---

## Personality → Behavior Mapping

HEXACO axes influence action scoring:

| HEXACO Axis | High → Behavior Bias | Low → Behavior Bias |
|-------------|----------------------|---------------------|
| H (Honesty) | Fair trade, sharing | Theft, deception |
| E (Emotionality) | Risk avoidance, social seeking | Risk taking, independence |
| X (Extraversion) | Socialize, lead | Solitary work, avoid crowds |
| A (Agreeableness) | Cooperate, compromise | Compete, dominate |
| C (Conscientiousness) | Work, organize, plan | Leisure, spontaneous |
| O (Openness) | Explore, learn new skills | Familiar routines |

---

## AI Evolution Roadmap

```
Phase 0 (current):  Utility AI — weighted scoring
Phase 2:            GOAP — goal-oriented action planning
Phase 3:            Behavior Trees — complex multi-step behaviors
Phase 4:            ML (ONNX) — learned behavior patterns
Phase 5:            Local LLM — natural language decisions for leaders
```

Each phase adds to, not replaces, the previous. Utility AI remains the fallback.

---

## Rust Migration Notes

AI decision-making is a strong Rust candidate at scale:

| Component | Migration Priority | Reason |
|-----------|-------------------|--------|
| Action scoring loop | 🟡 MEDIUM | N entities × M actions per tick |
| Pathfinding (used by AI) | 🔴 HIGH | Already in core/world/ |
| GOAP planner | 🟡 MEDIUM | Graph search, parallelizable |
| ONNX inference | 🔴 HIGH | `ort` crate, GPU-accelerated |

### Preparing for Rust (Do Now)
- Keep scoring functions pure (input: EntityData + ActionCandidate → output: float)
- No side effects in scoring — all effects happen after selection via SimulationBus
- Use PackedFloat64Array for batch scoring when entity count grows

---

## Adding a New Action Type

1. Add ActionCandidate definition in `behavior_system.gd`
2. Add scoring logic (need fulfillment, personality modifier, etc.)
3. Add execution logic → emit appropriate SimulationBus signal
4. Add constants to GameConfig (weights, thresholds)
5. Localize action name: `Locale.ltr("ACTION_" + action_id.to_upper())`

---

## Do NOT

- Execute actions directly (modify EntityData) — always go through SimulationBus
- Make AI decisions based on information the entity wouldn't know (fog of war)
- Hard-code action priorities — everything flows through the scoring function
- Skip personality/emotion influence — every action must consider agent psychology