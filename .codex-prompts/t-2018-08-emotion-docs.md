# T-2018-08: Emotion System Documentation

## Objective
Create comprehensive documentation for the Plutchik emotion system with full academic references, design rationale, and implementation details.

## File to Create

### `docs/EMOTION_SYSTEM.md` (NEW)

## Content Outline

Write a complete markdown document covering ALL of the following sections. Be thorough — this is the reference document for the emotion system.

### 1. Overview
- Hybrid model: Plutchik 8 basic emotions + Russell Valence-Arousal
- Purpose in WorldSim: emergent agent behavior, personality coupling, social dynamics
- Design philosophy: academic rigor adapted for real-time simulation

### 2. Plutchik 8 Basic Emotions
Table with all 8 emotions including:
| Emotion | Evolutionary Function | Behavioral Tendency | Opposite |
- Joy, Trust, Fear, Surprise, Sadness, Disgust, Anger, Anticipation
- Each with Plutchik's evolutionary purpose (protection, destruction, reproduction, etc.)

### 3. Intensity Levels (24 States)
Table mapping each emotion to 3 intensity levels:
| Emotion | Mild (0-33) | Base (34-66) | Intense (67-100) |
| Joy | Serenity | Joy | Ecstasy |
| Trust | Acceptance | Trust | Admiration |
... etc for all 8

Include Korean labels.

### 4. 24 Dyad System
- Primary Dyads (8, adjacent on wheel): love, submission, awe, disappointment, remorse, contempt, aggressiveness, optimism
- Secondary Dyads (8, one apart): hope, guilt, curiosity, despair, unbelief, envy, cynicism, pride
- Tertiary Dyads (8, two apart): delight, sentimentality, shame, outrage, pessimism, morbidness, dominance, anxiety
- Calculation: geometric mean sqrt(E1 × E2) — strong only when both component emotions are high
- Display threshold: 30+

### 5. 3-Layer Temporal Dynamics
- **Fast Layer** (episodic): Immediate emotional reactions, rapid exponential decay
  - Half-lives from Verduyn & Brans (2012): Joy 45min, Fear 18min, Surprise 3min, Sadness 30min, Disgust 6min, Anger 24min, Trust 2h, Anticipation 3h
- **Slow Layer** (mood/baseline): Ornstein-Uhlenbeck mean-reverting process
  - Personality-dependent baselines
  - Half-lives: 6h (surprise) to 120h/5 days (sadness)
  - Range: 0-30 (contributes to but doesn't dominate total)
  - Random fluctuation: σ=0.5 per √(dt)
- **Memory Trace Layer** (long-term scars): Event-specific persistent effects
  - Threshold: only impulses > 20 create traces
  - Trace intensity: 30% of original impulse
  - Normal decay: half-life 30 days
  - Trauma decay: half-life 365 days
  - Pruning: remove traces with intensity < 0.5

### 6. Valence-Arousal Derivation
- Valence = (Joy + Trust + 0.5×Anticipation) - (Sadness + Disgust + 0.5×Fear), clamped -100 to +100
- Arousal = (Fear + Surprise + Anger + Anticipation + 0.3×Joy) / 4.3, clamped 0-100
- Based on Russell (1980) Circumplex Model mapping

### 7. Appraisal-Based Impulse Generation
Based on Lazarus (1991) and Scherer (2009):
- 8 appraisal dimensions: goal_congruence, novelty, controllability, agency, norm_violation, pathogen, social_bond, future_relevance
- Mapping formulas for each emotion:
  - Joy = intensity × max(0, goal_congruence) × (1 + 0.5 × novelty) × sensitivity
  - Sadness = intensity × max(0, -goal_congruence) × (1 - controllability) × sensitivity
  - Anger = intensity × max(0, -goal_congruence) × controllability × max(0, -agency + norm_violation) × sensitivity
  - Fear = intensity × max(0, -goal_congruence) × (1 - controllability) × (0.5 + 0.5 × novelty) × sensitivity
  - Disgust = intensity × (pathogen + 0.7 × norm_violation) × (0.5 + 0.5 × max(0, -goal_congruence)) × sensitivity
  - Surprise = intensity × novelty × sensitivity
  - Trust = intensity × max(0, social_bond) × (1 - pathogen) × (1 - norm_violation) × sensitivity
  - Anticipation = intensity × future_relevance × (0.5 + 0.5 × max(0, goal_congruence)) × sensitivity

### 8. HEXACO-Emotion Coupling
Table of personality → emotion connections:
| HEXACO Axis | Emotion Effect | Academic Source |
| E (Emotionality) ↑ | Fear/Sadness sensitivity ↑, half-life ↑, baseline ↑ | Verduyn: neuroticism↔negative affect r≈.48 |
| X (Extraversion) ↑ | Joy sensitivity ↑, half-life ↑, baseline ↑ | Aghababaei: X↔happiness r≈.57 |
| A (Agreeableness) ↓ | Anger sensitivity ↑, half-life ↑, baseline ↑ | HEXACO: A = "versus Anger" |
| H (Honesty-Humility) ↑ | Disgust(moral) sensitivity ↑ | Cohen: guilt proneness × H r≈.50 |
| O (Openness) ↑ | Surprise/Anticipation sensitivity ↑ | — |
| C (Conscientiousness) ↑ | Mental Break threshold ↑ | Self-control literature |

Sensitivity formula: exp(coefficient × z-score)
Half-life adjustment: base × exp(coefficient × z-score)
Baseline calculation: clampf(base + slope × z-score, min, max)

### 9. Opposite Emotion Inhibition
- Joy ↔ Sadness, Trust ↔ Disgust, Fear ↔ Anger, Surprise ↔ Anticipation
- Inhibition: fast[emo] -= γ × opposite_total, γ = 0.3
- Prevents contradictory emotional states

### 10. Emotional Contagion
Based on Hatfield et al. (1993) and Fan et al. (2016):
- Contagion coefficients (anger > fear > joy): anger 0.12, fear 0.10, joy 0.08, disgust 0.06, trust 0.06, sadness 0.04, surprise 0.03, anticipation 0.03
- Distance decay: exp(-distance / d0), d0 = 5 tiles
- Relationship strength multiplier
- Susceptibility: exp(0.2×z_E + 0.1×z_A)
- Threshold: only emotions > 10 are contagious
- Scope: within settlement only (avoids O(n²) global)

### 11. Stress & Mental Break System
- Stress accumulation: stress = stress × exp(-dt/τ_S) + neg_input × dt, τ_S = 48h
- Negative input weights: Fear ×1.0, Anger ×0.9, Sadness ×1.1, Disgust ×0.6
- Break threshold: 300 + 50 × z_C (Conscientiousness raises threshold)
- Break probability: sigmoid p = 1/(1+exp(-(stress-threshold)/β)), β = 60
- Break types:
  - **Panic** (Fear dominant): flee random, 2h, energy drain ×3
  - **Rage** (Anger dominant): attack/destroy, 1h, energy drain ×5
  - **Shutdown** (Sadness dominant): idle, 6h, energy drain ×0.5
  - **Purge** (Disgust dominant): exile/destroy contaminated, 2h, energy drain ×2
  - **Outrage Violence** (Surprise+Anger dyad > 60): immediate violence, 0.5h, energy drain ×4
- Post-break: stress reduced to 50% (not 0, prevents immediate re-break)

### 12. Habituation
- Repeated same-category events reduce emotional impact
- Formula: habituation_factor = exp(-η × n), η = 0.2, n = exposure count
- Applied to impulse intensity before personality sensitivity

### 13. Event Presets
Full table of all event presets from `data/emotions/event_presets.json` with their appraisal vectors.

### 14. Save/Load
- EmotionData serialized as JSON within entity save data
- Backward compatible: legacy 5-emotion saves migrated via `from_legacy()`
- Fields saved: fast, slow, memory_traces, stress, habituation, mental_break_type, mental_break_remaining

### 15. References
Full academic bibliography:
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

## Non-goals
- Do NOT create or modify any code files
- Do NOT include implementation details beyond what's needed for understanding the system
- Do NOT reference internal Godot APIs — this is a design document

## Acceptance Criteria
- [ ] `docs/EMOTION_SYSTEM.md` exists
- [ ] All 15 sections covered with substantive content
- [ ] All formulas from the implementation included
- [ ] All academic references listed with full citation
- [ ] Tables for emotion intensities, Dyads, contagion coefficients, HEXACO coupling
- [ ] No broken markdown formatting
