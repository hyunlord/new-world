# Stress System Reference

## 1. Overview

The stress system models how agents accumulate, regulate, and carry stress over time. It is grounded in established psychology and physiology research and converted into deterministic game rules.

## 2. Academic References

| Model | Author / Year | Game Mapping |
|---|---|---|
| Transactional Model of Stress | Lazarus & Folkman, 1984 | `appraisal_scale`: demand vs. resource imbalance |
| General Adaptation Syndrome (GAS) | Selye, 1956 | `reserve` -> Alarm / Resistance / Exhaustion |
| Allostatic Load | McEwen, 1998 | Chronic wear -> persistent personality change |
| Conservation of Resources (COR) | Hobfoll, 1989 | Resource loss is 2.5x more sensitive |
| Social Readjustment Rating Scale | Holmes & Rahe, 1967 | Stressors are converted into scored events |
| Yerkes-Dodson Law | Yerkes & Dodson, 1908 | Eustress -> work efficiency curve |
| Emotional Contagion | Hatfield et al., 1993 | Group stress spreads between agents |
| Cognitive Dissonance | Festinger, 1957 | Trait-incongruent actions add stress |
| Connor-Davidson Resilience Scale | Connor & Davidson, 2003 | `resilience` represented as a numeric factor |

Game references:
- RimWorld (Mood / Mental Break)
- CK3 (Stress 0-400, Coping)
- Dwarf Fortress (Tantrum Spiral)

## 3. Three-Layer Hybrid Model

- Lazarus appraisal: demand/resource imbalance -> `appraisal_scale`.
- Selye GAS: `reserve` drains and recovers through Alarm -> Resistance -> Exhaustion.
- McEwen allostatic load: chronic wear via EMA-style accumulation.
- Hobfoll COR: loss aversion at 2.5x in `inject_stress_event`.

## 4. Constants Table

| Constant | Value | Purpose |
|---|---|---|
| `STRESS_CLAMP_MAX` | `2000.0` | Maximum stress |
| `STRESS_EPSILON` | `0.05` | Micro-oscillation cutoff |
| `BASE_DECAY_PER_TICK` | `1.2` | Base natural decay |
| `DECAY_FRAC` | `0.006` | Proportional decay from current stress |
| `SAFE_DECAY_BONUS` | `0.8` | Extra decay in safe zones |
| `SLEEP_DECAY_BONUS` | `1.5` | Extra decay during sleep |
| `SUPPORT_DECAY_MULT` | `0.12` | Social support decay multiplier |
| `THRESHOLD_TENSE` | `200.0` | `stress_state = 1` threshold |
| `THRESHOLD_CRISIS` | `350.0` | `stress_state = 2` threshold |
| `THRESHOLD_BREAK_RISK` | `500.0` | `stress_state = 3` threshold |
| `RESERVE_MAX` | `100.0` | Maximum reserve |
| `ALLO_RATE` | `0.035` | Allostatic increase rate |
| `ALLO_STRESS_THRESHOLD` | `250.0` | Allostatic increase starts at this stress |
| `ALLO_RECOVERY_THRESHOLD` | `120.0` | Allostatic recovery starts at this stress |
| `ALLO_RECOVERY_RATE` | `0.003` | Natural allostatic recovery rate |
| `EMOTION_STRESS_THRESHOLD` | `20.0` | Minimum emotion contribution |
| `VA_GAMMA` | `3.0` | V/A composite max contribution |
| `EUSTRESS_OPTIMAL` | `150.0` | Yerkes-Dodson optimal stress |

## 5. Stressor Values

Need deficiency curves:
- `hunger` starts below `0.35`, at `0.0` -> ~12 / tick
- `energy` starts below `0.40`, at `0.0` -> ~12 / tick
- `social` starts below `0.25`, representing social isolation

Event stress:
- `partner_death`: `instant=450`, `per_tick=10`, `decay_rate=0.01`, `is_loss=true` (COR 2.5x -> effective `1125`)
- `parent_death`: `instant=650`, `per_tick=15`, `decay_rate=0.008`, `is_loss=true`
- `combat_engaged`: `instant=80`, `per_tick=20`, `decay_rate=0.15`

## 6. Emotion Integration

Emotion -> stress contributions (`EMOTION_WEIGHTS`):
- `fear: +0.09`, `anger: +0.06`, `sadness: +0.05`, `disgust: +0.04`, `surprise: +0.03`
- `joy: -0.05`, `trust: -0.04`, `anticipation: -0.02`
- V/A composite: `VA_GAMMA * arousal * (-valence) / 100`

Stress -> emotion feedback:
- Stress `100-500`: OU target shifts (`sadness+`, `anger+`, `fear+`, `joy-`, `trust-`)
- Stress `300-700`: fast-gain sensitivity changes (negative `x1.7`, positive `x0.5`)
- Allostatic `60+`: Emotional Blunting (global emotion dampening)

## 7. Resilience Formula

```text
r = 0.35*(1-E) + 0.25*C + 0.15*X + 0.10*O + 0.10*A + 0.05*H
    + 0.25*support - 0.30*(allostatic/100) - 0.20*fatigue_penalty
clamp(r, 0.05, 1.0)
```

HEXACO:
- `E`: Emotionality
- `C`: Conscientiousness
- `X`: Extraversion
- `O`: Openness
- `A`: Agreeableness
- `H`: Honesty-Humility

## 8. Yerkes-Dodson Work Efficiency

```text
stress < 150: perf = 1.0 + 0.0006 * stress         # max 1.09
stress 150-350: perf = 1.09 - 0.0004 * (stress-150) # 1.09 -> 1.01
stress > 350: perf = 1.01 - 0.0012 * (stress-350)   # 1.01 -> 0.35
clamp(perf, 0.35, 1.10)
```

## 9. Phase Roadmap

- Phase 1 (current): core data + baseline pipeline
- Phase 2: mental break trigger + 10 types + behavior override
- Phase 3: trauma scars + CK3-style trait-incongruent actions
- Phase 4: 15 learned coping traits + group contagion
- Phase 5: child stress + ACE + era evolution + UI

## 10. Validation Scenarios

1. Typical agent: stress stays within `10-80`.
2. Starving agent: `hunger < 0.2` causes fast stress rise.
3. Bereaved agent: instant `1125` -> multi-day `300-500`.
4. Reserve depletion: sustained `stress 400+` -> `reserve < 30` slows recovery.
5. Allostatic accumulation: sustained `stress 300+` for one month -> `allostatic 10-15`.
