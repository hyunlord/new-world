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

## 4.1 Stress Zone Semantics

The effective gameplay range is 0–1,000. Values above 1,000 are the **overflow zone**, reached only during extreme cascade events.

| Zone | Range | `stress_state` | Gameplay Effect |
|------|-------|---------------|-----------------|
| Normal | 0–200 | 0 | No penalty; eustress boosts work efficiency up to +9% |
| Tense | 200–350 | 1 | Work efficiency begins declining (Yerkes-Dodson descent) |
| Crisis | 350–500 | 2 | Work efficiency notably reduced |
| Break Risk | 500–900 | 3 | Mental break probability check active each tick |
| Overflow | 1,000+ | 3 | Cascade surge zone; UI gauge maxed with red overlay |

### STRESS_CLAMP_MAX = 2,000: Design Rationale

The hard cap is 2,000 rather than 1,000 to accommodate worst-case cascade stacking without clipping:

- `partner_death` (base `instant=450`) × COR loss aversion `2.5` = **1,125** effective stress
- Pre-existing stress at `300` → single-tick peak = **1,425**
- 2,000 provides overflow buffer for multi-event chains within the same tick

The 0–1,000 range covers all meaningful simulation states. The 1,000–2,000 range is a safety margin that prevents hard truncation during extreme cascades.

### UI Display Strategy (Phase 5)

The UI renders stress on a **0–1,000 visual scale**:
- `stress ≤ 1,000`: gauge position = `stress / 1,000`
- `stress > 1,000`: gauge remains full; a red overlay pattern is applied
- Internal simulation accumulates beyond 1,000 until natural decay returns it to the visible range

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

---

## Phase 2: 멘탈 브레이크 시스템

### 개요

스트레스가 임계점을 넘으면 에이전트에게 **멘탈 브레이크**가 확률적으로 발동된다.
발동 시 행동이 강제 오버라이드되고, 종료 후 카타르시스(스트레스 감소)와 Shaken 후유증이 적용된다.

### 멘탈 브레이크 10종

| ID | 한국어 | 영어 | 심각도 | 행동 모드 | 카타르시스 |
|----|--------|------|--------|-----------|-----------|
| panic | 공황 | Panic | minor | flee_hide | 80% |
| rage | 분노 폭발 | Rage | major | attack_smash | 65% |
| outrage_violence | 폭력 난동 | Outrage Violence | extreme | seek_and_destroy | 60% |
| shutdown | 셧다운 | Shutdown | major | freeze_in_place | 90% |
| purge | 폭식/낭비 | Purge | minor | binge_consume | 75% |
| grief_withdrawal | 애도 칩거 | Grief Withdrawal | major | withdraw_to_home | 85% |
| fugue | 해리성 둔주 | Dissociative Fugue | major | wander_away | 80% |
| paranoia | 편집증 | Paranoia | major | distrust_isolate | 95% |
| compulsive_ritual | 강박 의식 | Compulsive Ritual | minor | repeat_action | 85% |
| hysterical_bonding | 불안 집착 | Hysterical Bonding | minor | cling_to_target | 80% |

### 발동 역치 계산

```
base_threshold = 520.0
threshold_min  = 420.0
threshold_max  = 900.0

threshold *= (1 + 0.40 * resilience_z)   # Connor-Davidson resilience
threshold *= (1 + 0.25 * C_z)            # 성실성 (Conscientiousness)
threshold *= (1 - 0.35 * E_z)            # 신경증 (Emotionality)
threshold *= (1 + 0.15 * support_z)      # 사회적 지지
threshold *= (1 - 0.25 * allostatic/100) # 알로스태틱 부하
# GAS Exhaustion 보정: reserve < 30 → -40, reserve < 15 → -80
```

### 발동 확률 (per tick)

```
if stress <= threshold: p = 0
else: p = clamp((stress - threshold) / 6000.0, 0, 0.25)
# reserve < 30: p *= 1.3
# allostatic > 60: p *= 1.2
```

### 유형 선택 (HEXACO 가중치)

각 브레이크 유형의 `personality_weights`(H/E/X/A/C/O 축)를 이용해 가중치를 계산하고
softmax 방식으로 유형을 선택한다. E↑ → Panic/Shutdown, A↓ → Rage/Outrage.

### Shaken 후유증

브레이크 종료 후 Shaken 상태가 시작된다:
- `shaken_work_penalty`: 작업 효율 감소 (심각도별 -5%~-20%)
- `shaken_remaining`: 남은 틱 수 (24~120틱)
- 매 틱 1씩 감소, 0이 되면 해제

### 학술 근거

| 모델 | 적용 |
|------|------|
| Lazarus & Folkman (1984) | 대처 실패 → 붕괴 |
| Selye GAS (1956) | Exhaustion 단계 취약성 |
| DSM-5 Panic Disorder | Panic 유형 |
| IED (Intermittent Explosive Disorder) | Rage/Outrage 유형 |
| Learned Helplessness (Seligman 1967) | Shutdown 유형 |
| Dissociative Fugue (DSM-5) | Fugue 유형 |
| Connor-Davidson Resilience Scale (2003) | 역치 개인화 |
| RimWorld Mental Break | Minor/Major/Extreme 단계, 확률 모델 |
