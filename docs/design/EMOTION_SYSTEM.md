# Emotion System (Plutchik + Valence-Arousal Hybrid)

## 1. Overview

WorldSim uses a hybrid emotion model that combines:
- **Plutchik's 8 basic emotions** as the primary representational layer (Joy, Trust, Fear, Surprise, Sadness, Disgust, Anger, Anticipation)
- **Russell's Valence-Arousal circumplex** as a derived control layer for urgency and behavioral direction

This design supports three simulation goals:
- **Emergent agent behavior**: emotion blends and dyads produce non-scripted reactions
- **Personality coupling**: HEXACO traits modulate sensitivity, decay, baselines, contagion, and breakdown risk
- **Social dynamics**: local emotional contagion and stress cascades create settlement-level mood patterns

Design philosophy: academic emotion theory is translated into deterministic, bounded, real-time simulation rules. The model keeps psychologically interpretable structure while remaining computationally tractable.

## 2. Plutchik 8 Basic Emotions

| Emotion | Evolutionary Function | Behavioral Tendency | Opposite |
|---|---|---|---|
| Joy | Reproduction / social bonding | Approach, play, celebrate, reinforce affiliation | Sadness |
| Trust | Incorporation / alliance formation | Accept, cooperate, rely, maintain cohesion | Disgust |
| Fear | Protection / threat avoidance | Escape, freeze, vigilance, risk minimization | Anger |
| Surprise | Orientation to sudden change | Interrupt action, reorient attention, rapid reassessment | Anticipation |
| Sadness | Reintegration after loss | Withdraw, conserve energy, seek support, re-evaluate goals | Joy |
| Disgust | Rejection of contamination / moral violation | Repel, avoid, expel, distance from tainted targets | Trust |
| Anger | Destruction of obstacle/threat | Confront, attack, dominate, enforce boundaries | Fear |
| Anticipation | Exploration / preparation | Plan, scout, monitor, readiness for upcoming outcomes | Surprise |

## 3. Intensity Levels (24 States)

Emotion values are interpreted in three bands:
- Mild: `0-33`
- Base: `34-66`
- Intense: `67-100`

| Emotion | Mild (0-33) | Base (34-66) | Intense (67-100) |
|---|---|---|---|
| Joy | Serenity (평온) | Joy (기쁨) | Ecstasy (황홀) |
| Trust | Acceptance (수용) | Trust (신뢰) | Admiration (경외) |
| Fear | Apprehension (우려) | Fear (공포) | Terror (경악) |
| Surprise | Distraction (산만) | Surprise (놀람) | Amazement (경이) |
| Sadness | Pensiveness (수심) | Sadness (슬픔) | Grief (비통) |
| Disgust | Boredom (지루함) | Disgust (혐오) | Loathing (증오) |
| Anger | Annoyance (짜증) | Anger (분노) | Rage (격노) |
| Anticipation | Interest (흥미) | Anticipation (기대) | Vigilance (경계) |

## 4. 24 Dyad System

Dyads are computed in real time from two component emotions and are not stored.

### Primary Dyads (adjacent on wheel)

| Dyad | Components |
|---|---|
| Love | Joy + Trust |
| Submission | Trust + Fear |
| Awe | Fear + Surprise |
| Disappointment | Surprise + Sadness |
| Remorse | Sadness + Disgust |
| Contempt | Disgust + Anger |
| Aggressiveness | Anger + Anticipation |
| Optimism | Anticipation + Joy |

### Secondary Dyads (one step apart)

| Dyad | Components |
|---|---|
| Hope | Anticipation + Trust |
| Guilt | Joy + Fear |
| Curiosity | Trust + Surprise |
| Despair | Fear + Sadness |
| Unbelief | Surprise + Disgust |
| Envy | Sadness + Anger |
| Cynicism | Disgust + Anticipation |
| Pride | Anger + Joy |

### Tertiary Dyads (two steps apart)

| Dyad | Components |
|---|---|
| Delight | Joy + Surprise |
| Sentimentality | Trust + Sadness |
| Shame | Fear + Disgust |
| Outrage | Surprise + Anger |
| Pessimism | Sadness + Anticipation |
| Morbidness | Disgust + Joy |
| Dominance | Anger + Trust |
| Anxiety | Anticipation + Fear |

### Dyad strength

\[
D(E_1, E_2) = \sqrt{E_1 \times E_2}
\]

Geometric mean ensures dyads become strong only when **both** components are strong.

Display policy:
- Dyads are surfaced to UI/logic only when `D >= 30`.

## 5. 3-Layer Temporal Dynamics

### Fast Layer (episodic reactions)

Purpose:
- Immediate event-driven responses with rapid exponential decay.

Per-emotion half-lives (Verduyn & Brans, 2012 derived calibration):

| Emotion | Half-life |
|---|---|
| Joy | 45 min |
| Fear | 18 min |
| Surprise | 3 min |
| Sadness | 30 min |
| Disgust | 6 min |
| Anger | 24 min |
| Trust | 2 h |
| Anticipation | 3 h |

Update form:
\[
E_{fast}(t+\Delta t) = E_{fast}(t)e^{-k\Delta t} + I(t), \quad k=\ln(2)/T_{1/2}
\]

### Slow Layer (mood / baseline)

Purpose:
- Mean-reverting background mood that reflects personality and long-horizon drift.

Dynamics:
- Ornstein-Uhlenbeck style mean reversion toward personality baseline `\mu`.
- Half-life range: `6h` (surprise) to `120h` (sadness, 5 days).
- Slow-layer clamp range: `0-30` so mood contributes but does not dominate total emotion.
- Random fluctuation term: `\sigma = 0.5` per `\sqrt{\Delta t}`.

Update form:
\[
E_{slow}(t+\Delta t)=\mu + (E_{slow}(t)-\mu)e^{-k_{slow}\Delta t}+\sigma\sqrt{\Delta t}\,\mathcal{N}(0,1)
\]

### Memory Trace Layer (long-term scars)

Purpose:
- Event-specific residual effects (especially loss/trauma) that persist beyond fast and slow layers.

Rules:
- Create trace only when impulse for an emotion exceeds `20`.
- Trace intensity at creation: `0.3 \times` original impulse.
- Normal trace decay half-life: `30 days`.
- Trauma trace decay half-life: `365 days`.
- Prune traces when intensity falls below `0.5`.

Total emotion reconstruction:
\[
E_{total} = clamp(E_{fast} + E_{slow} + E_{memory}, 0, 100)
\]

## 6. Valence-Arousal Derivation

Derived each update from Plutchik totals.

Valence:
\[
V = (J + T + 0.5A_n) - (S + D + 0.5F),\quad V \in [-100,100]
\]

Arousal:
\[
A = \frac{F + S_u + A_g + A_n + 0.3J}{4.3},\quad A \in [0,100]
\]

Where:
- `J`: Joy, `T`: Trust, `F`: Fear, `S_u`: Surprise, `S`: Sadness, `D`: Disgust, `A_g`: Anger, `A_n`: Anticipation.

Interpretation:
- Positive/negative direction comes from valence.
- Activation urgency comes from arousal.
- Mapping follows Russell (1980) circumplex rationale.

## 7. Appraisal-Based Impulse Generation

Following Lazarus (1991) and Scherer (2009), each event carries appraisal dimensions:
- `goal_congruence`
- `novelty`
- `controllability`
- `agency`
- `norm_violation`
- `pathogen`
- `social_bond`
- `future_relevance`

For event intensity `I` and per-emotion personality sensitivity `S_e`:

\[
Joy = I \times max(0, goal\_congruence) \times (1+0.5\times novelty) \times S_{joy}
\]
\[
Sadness = I \times max(0, -goal\_congruence) \times (1-controllability) \times S_{sadness}
\]
\[
Anger = I \times max(0,-goal\_congruence) \times controllability \times max(0,-agency+norm\_violation) \times S_{anger}
\]
\[
Fear = I \times max(0,-goal\_congruence) \times (1-controllability) \times (0.5+0.5\times novelty) \times S_{fear}
\]
\[
Disgust = I \times (pathogen+0.7\times norm\_violation) \times (0.5+0.5\times max(0,-goal\_congruence)) \times S_{disgust}
\]
\[
Surprise = I \times novelty \times S_{surprise}
\]
\[
Trust = I \times max(0,social\_bond) \times (1-pathogen) \times (1-norm\_violation) \times S_{trust}
\]
\[
Anticipation = I \times future\_relevance \times (0.5+0.5\times max(0,goal\_congruence)) \times S_{anticipation}
\]

Habituation is applied to impulse intensity before personality scaling.

## 8. HEXACO-Emotion Coupling

| HEXACO Axis | Emotion Effect | Academic Source |
|---|---|---|
| `E` (Emotionality) ↑ | Fear/Sadness sensitivity ↑, half-life ↑, baseline ↑ | Verduyn: neuroticism-negative affect `r≈.48` |
| `X` (Extraversion) ↑ | Joy sensitivity ↑, half-life ↑, baseline ↑ | Aghababaei: `X`-happiness `r≈.57` |
| `A` (Agreeableness) ↓ | Anger sensitivity ↑, half-life ↑, baseline ↑ | HEXACO framing: `A` as "versus Anger" |
| `H` (Honesty-Humility) ↑ | Moral-disgust sensitivity ↑ | Cohen: guilt proneness × `H` `r≈.50` |
| `O` (Openness) ↑ | Surprise/Anticipation sensitivity ↑ | Conceptual mapping |
| `C` (Conscientiousness) ↑ | Mental-break threshold ↑ | Self-control literature |

Core transform formulas (using trait z-scores):

Sensitivity:
\[
S = e^{c\times z}
\]

Half-life adjustment:
\[
T'_{1/2} = T_{1/2,base} \times e^{c\times z}
\]

Baseline:
\[
B = clampf(B_{base}+slope\times z, min, max)
\]

Representative implementation coefficients:
- Fear/Sadness sensitivity: `exp(0.4*z_E)`
- Joy sensitivity: `exp(0.3*z_X)`
- Anger sensitivity: `exp(-0.35*z_A)`
- Disgust sensitivity: `exp(0.25*z_H)`
- Surprise sensitivity: `exp(0.2*z_O)`
- Anticipation sensitivity: `exp(0.2*z_O + 0.15*z_C)`
- Trust sensitivity: `exp(0.2*z_X + 0.15*z_A)`

## 9. Opposite Emotion Inhibition

Opposite pairs:
- Joy ↔ Sadness
- Trust ↔ Disgust
- Fear ↔ Anger
- Surprise ↔ Anticipation

Inhibition update:
\[
fast[e] = max(0,\, fast[e] - \gamma \times total[opposite(e)])
\]

With:
- `\gamma = 0.3`

Purpose:
- Damps contradictory affective states and stabilizes readable emotional profiles.

## 10. Emotional Contagion

Based on emotional contagion literature (Hatfield et al., 1993; Fan et al., 2016), contagion is local, weighted, and asymmetric.

Contagion coefficients:

| Emotion | Coefficient |
|---|---|
| Anger | 0.12 |
| Fear | 0.10 |
| Joy | 0.08 |
| Disgust | 0.06 |
| Trust | 0.06 |
| Sadness | 0.04 |
| Surprise | 0.03 |
| Anticipation | 0.03 |

Rules:
- Distance decay: `exp(-distance / d0)`, with `d0 = 5` tiles.
- Relationship strength multiplies transmission.
- Susceptibility: `exp(0.2*z_E + 0.1*z_A)`.
- Only source emotions `> 10` are contagious.
- Scope is **within the same settlement only** to avoid global `O(n^2)` contagion.

Per-pair contribution (source to target):
\[
\Delta e = \kappa_e \times E_{source,e} \times e^{-distance/d_0} \times relationship \times susceptibility \times \Delta t
\]

## 11. Stress & Mental Break System

Stress accumulation:
\[
stress_{t+\Delta t} = stress_t\,e^{-\Delta t/\tau_S} + neg\_input\times\Delta t,\quad \tau_S=48h
\]

Negative input:
\[
neg\_input = 1.0\times Fear + 0.9\times Anger + 1.1\times Sadness + 0.6\times Disgust
\]

Break threshold:
\[
threshold = 300 + 50\times z_C
\]

Break probability:
\[
p = \frac{1}{1+e^{-(stress-threshold)/\beta}},\quad \beta=60
\]

Break types:

| Break Type | Trigger Pattern | Duration | Energy Drain |
|---|---|---|---|
| Panic | Fear dominant | 2h | x3 |
| Rage | Anger dominant | 1h | x5 |
| Shutdown | Sadness dominant | 6h | x0.5 |
| Purge | Disgust dominant | 2h | x2 |
| Outrage Violence | Surprise+Anger dyad (`outrage`) > 60 | 0.5h | x4 |

Post-break rule:
- Stress is reduced to **50%**, not zero, to prevent immediate repeated triggering loops and preserve consequence continuity.

## 12. Habituation

Repeated exposure to the same event category reduces impulse amplitude:
\[
habituation\_factor = e^{-\eta n},\quad \eta=0.2
\]

Where:
- `n` = exposure count for that category.

Application order:
1. Compute appraisal-derived impulse.
2. Multiply by habituation factor.
3. Apply personality sensitivity scaling.

## 13. Event Presets (`data/emotions/event_presets.json`)

The following table enumerates all current presets and their appraisal vectors.

| Event ID | Description | Cat. | Int. | g | n | c | a | m | p | b | f | Trauma |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `food_acquired` | Food acquired | resource | 25 | 0.7 | 0.1 | 0.8 | 0.5 | 0.0 | 0.0 | 0.0 | 0.3 | No |
| `wood_acquired` | Wood acquired | resource | 20 | 0.5 | 0.05 | 0.8 | 0.5 | 0.0 | 0.0 | 0.0 | 0.2 | No |
| `stone_acquired` | Stone acquired | resource | 20 | 0.5 | 0.05 | 0.8 | 0.5 | 0.0 | 0.0 | 0.0 | 0.2 | No |
| `severe_hunger` | Severe hunger | survival | 50 | -0.8 | 0.1 | 0.3 | 0.0 | 0.0 | 0.0 | 0.0 | 0.7 | No |
| `starvation_warning` | Starvation imminent | survival | 75 | -1.0 | 0.2 | 0.1 | 0.0 | 0.0 | 0.0 | 0.0 | 0.9 | No |
| `ate_food` | Ate food successfully | survival | 20 | 0.6 | 0.0 | 0.9 | 0.5 | 0.0 | 0.0 | 0.0 | 0.1 | No |
| `partner_found` | Found a partner | social | 40 | 0.9 | 0.6 | 0.5 | 0.5 | 0.0 | 0.0 | 0.8 | 0.7 | No |
| `partner_death` | Partner died | loss | 90 | -1.0 | 0.8 | 0.0 | -0.5 | 0.0 | 0.0 | 1.0 | 0.9 | Yes |
| `child_death` | Child died | loss | 95 | -1.0 | 0.7 | 0.0 | -0.5 | 0.0 | 0.0 | 1.0 | 0.8 | Yes |
| `parent_death` | Parent died | loss | 70 | -0.8 | 0.5 | 0.0 | -0.3 | 0.0 | 0.0 | 0.7 | 0.5 | Yes |
| `friend_death` | Friend died | loss | 50 | -0.6 | 0.5 | 0.0 | -0.2 | 0.0 | 0.0 | 0.5 | 0.3 | No |
| `child_born` | Child born | family | 60 | 0.9 | 0.7 | 0.3 | 0.5 | 0.0 | 0.0 | 0.9 | 0.9 | No |
| `building_completed` | Building completed | achievement | 30 | 0.8 | 0.3 | 0.8 | 0.7 | 0.0 | 0.0 | 0.2 | 0.5 | No |
| `social_interaction` | Positive social interaction | social | 15 | 0.4 | 0.1 | 0.6 | 0.3 | 0.0 | 0.0 | 0.5 | 0.1 | No |
| `new_territory` | New territory discovered | exploration | 35 | 0.5 | 0.9 | 0.6 | 0.7 | 0.0 | 0.0 | 0.0 | 0.8 | No |
| `migration_started` | Migration to new settlement | exploration | 40 | 0.3 | 0.8 | 0.4 | 0.3 | 0.0 | 0.0 | -0.3 | 0.9 | No |
| `settlement_founded` | New settlement founded | achievement | 45 | 0.8 | 0.7 | 0.5 | 0.6 | 0.0 | 0.0 | 0.6 | 0.8 | No |
| `theft_victim` | Victim of theft | conflict | 55 | -0.7 | 0.5 | 0.4 | -0.8 | 0.8 | 0.0 | -0.5 | 0.3 | No |
| `betrayal` | Betrayed by trusted person | conflict | 80 | -0.9 | 0.7 | 0.2 | -1.0 | 0.9 | 0.0 | -1.0 | 0.5 | Yes |
| `combat_threat` | Combat threat | danger | 70 | -0.8 | 0.4 | 0.3 | -0.5 | 0.0 | 0.0 | -0.7 | 0.6 | No |
| `community_festival` | Community festival | social | 40 | 0.6 | 0.3 | 0.7 | 0.3 | 0.0 | 0.0 | 0.8 | 0.2 | No |
| `job_assigned` | New job assigned | work | 15 | 0.3 | 0.3 | 0.2 | -0.2 | 0.0 | 0.0 | 0.0 | 0.4 | No |
| `rested_well` | Well rested (energy recovered) | survival | 15 | 0.5 | 0.0 | 0.7 | 0.3 | 0.0 | 0.0 | 0.0 | 0.1 | No |

Column legend:
- `g`: goal_congruence
- `n`: novelty
- `c`: controllability
- `a`: agency
- `m`: norm_violation
- `p`: pathogen
- `b`: social_bond
- `f`: future_relevance

## 14. Save/Load

Persistence model:
- `EmotionData` is serialized as JSON inside per-entity save data.

Backward compatibility:
- Legacy 5-emotion saves are migrated via `from_legacy()`.
- Migration maps old fields into Plutchik + layered representation, then recomputes derived state.

Saved fields:
- `fast`
- `slow`
- `memory_traces`
- `stress`
- `habituation`
- `mental_break_type`
- `mental_break_remaining`

Load post-processing:
- Recalculate derived values (including valence-arousal) after restoration.

## 15. References

- Plutchik, R. (1980). *Emotion: A Psychoevolutionary Synthesis*. Harper & Row.
- Plutchik, R. (2001). The nature of emotions. *American Scientist*, 89(4), 344-350.
- Russell, J. A. (1980). A circumplex model of affect. *Journal of Personality and Social Psychology*, 39(6), 1161-1178.
- Lazarus, R. S. (1991). *Emotion and Adaptation*. Oxford University Press.
- Scherer, K. R. (2009). The dynamic architecture of emotion: Evidence for the component process model. *Cognition and Emotion*, 23(7), 1307-1351.
- Verduyn, P., & Brans, K. (2012). The relationship between extraversion, neuroticism, and aspects of trait affect. *Personality and Individual Differences*, 52(6), 664-669.
- Verduyn, P., et al. (2015). Intensity profiles of emotional experience over time. *Cognition and Emotion*, 29(4), 751-763.
- Hatfield, E., Cacioppo, J. T., & Rapson, R. L. (1993). Emotional contagion. *Current Directions in Psychological Science*, 2(3), 96-100.
- Fan, R., et al. (2016). Anger is more influential than joy: Sentiment correlation in Weibo. *PLoS ONE*, 9(10), e110184.
- Aghababaei, N., & Arji, A. (2014). Well-being and the HEXACO model of personality. *Personality and Individual Differences*, 56, 139-142.
- Festinger, L. (1957). *A Theory of Cognitive Dissonance*. Stanford University Press.
