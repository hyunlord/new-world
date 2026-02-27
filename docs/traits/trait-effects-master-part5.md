# WorldSim Trait Effects — Part 5 (Awakened #1~#18, Bloodline #1~#25)
> Canonical YAML effect definitions using Trait System v3 schema.
> Ops: set / add / mult / min / max / disable / enable / block / inject / override / on_event / tag / immune / replace
> All conditions use structured format: {source, key, op, value}
>
> Part 1: Archetype #1~#30 + Shadow #1~#5 ✅
> Part 2: Archetype #31~#55 ✅
> Part 3: Shadow #6~#15 + Radiance #1~#12 ✅
> Part 4: Corpus #1~#12 + Nous #1~#10 ✅
> **Part 5: Awakened #1~#18 + Bloodline #1~#25 ← this file**
> Part 6: Mastery + Bond
> Part 7: Fate + Synergy

---

## Awakened Traits (W_) — Event-Gated Psychological Transformations

> Awakened traits are acquired through lived experience, not stats. They represent scars,
> transformations, and awakenings that permanently alter who someone is.
> Every Awakened trait has both a **gift** (capability/resilience) and a **cost** (wound/limitation).
> Reference: post-traumatic growth theory (Tedeschi & Calhoun 1996), moral injury (Litz et al. 2009).

---

### #1 W_scarred_soul
> **Scarred Soul / 상처 입은 영혼** — Trauma has accumulated into a new baseline; heightened sensitivity and partial resilience
> Acquisition: trauma_count >= 3 | Rarity: rare
> Academic: Tedeschi & Calhoun (1996) — post-traumatic growth; Herman (1992) — complex trauma

```yaml
effects:
  - system: behavior
    target: threat_detection
    op: inject
    value: { priority: 0.7 }
  - system: stress
    target: acute_shock
    op: immune
    value: true
  - system: emotion
    target: joy
    op: max
    value: 0.8
  - system: stress
    target: accumulation_rate
    op: mult
    value: 1.2
  - system: behavior
    target: trust_strangers
    op: block
    value: true
    condition:
      source: relationship
      key: trust
      op: lt
      value: 0.4
```

---

### #2 W_battle_forged
> **Battleborn / 전장에 벼려진 자** — Combat has become familiar; fear response recalibrated
> Acquisition: combat_survived >= 5 | Rarity: rare
> Academic: Grossman (2009) — On Killing; stress inoculation theory (Meichenbaum)

```yaml
effects:
  - system: combat
    target: panic_threshold
    op: mult
    value: 0.4
  - system: skill
    target: combat_skills
    op: mult
    value: 1.25
    tags: [combat]
  - system: stress
    target: combat_stress
    op: mult
    value: 0.5
  - system: stress
    target: prolonged_peace_stress
    op: mult
    value: 1.4
  - system: emotion
    target: fear
    op: min
    value: 0.05
```

---

### #3 W_widows_frost
> **Widow's Frost / 미망인의 서리** — Grief crystallized into cold clarity
> Acquisition: spouse_or_child_death | Rarity: rare
> Academic: Kübler-Ross (1969) — grief stages; Bonanno (2009) — grief resilience

```yaml
effects:
  - system: stress
    target: grief_spiral
    op: immune
    value: true
  - system: derived
    target: wisdom
    op: mult
    value: 1.2
  - system: relationship
    target: intimacy_ceil
    op: max
    value: 75
  - system: emotion
    target: joy
    op: max
    value: 0.75
  - system: behavior
    target: form_romantic_attachment
    op: block
    value: true
    condition:
      source: self
      key: grief_months_elapsed
      op: lt
      value: 24
```

---

### #4 W_twice_born
> **Twice-Born / 두 번 태어난 자** — Faced death and returned; perspective permanently shifted
> Acquisition: lethal_injury_survived | Rarity: rare
> Academic: Yalom (2008) — existential psychotherapy; near-death experience literature

```yaml
effects:
  - system: stress
    target: death_fear_stress
    op: immune
    value: true
  - system: emotion
    target: joy
    op: min
    value: 0.1
  - system: behavior
    target: seize_opportunity
    op: inject
    value: { priority: 0.6 }
  - system: stress
    target: injury_recurrence_stress
    op: mult
    value: 1.3
  - system: body
    target: old_injury_flare_chance
    op: mult
    value: 1.5
```

---

### #5 W_oath_breaker
> **Oath-Breaker / 맹세 파괴자** — A broken oath has fractured the self
> Acquisition: major_promise_broken | Rarity: rare
> Academic: Litz et al. (2009) — moral injury; Tangney & Dearing (2002) — shame and guilt

```yaml
effects:
  - system: stress
    target: oath_guilt_persistent
    op: set
    value: 0.15
  - system: derived
    target: trustworthiness
    op: mult
    value: 0.6
  - system: relationship
    target: trust_gain
    op: mult
    value: 0.7
  - system: behavior
    target: make_new_promises
    op: block
    value: true
  - system: behavior
    target: self_forgiveness
    op: block
    value: true
    condition:
      source: values
      key: LOYALTY
      op: gte
      value: 0.6
```

---

### #6 W_kinslayer
> **Kinslayer / 동족살해자** — Killed family/kin; unforgettable stain, dual identity
> Acquisition: family_or_kin_killed | Rarity: epic
> Academic: Litz et al. (2009) — moral injury; DSM-5 — PTSD with moral injury component

```yaml
effects:
  - system: memory
    target: kinslayer_event
    op: tag
    value: { permanent: true, compressible: false }
  - system: stress
    target: kinslayer_guilt
    op: set
    value: 0.2
  - system: emotion
    target: joy
    op: max
    value: 0.65
  - system: stress
    target: break_types
    op: replace
    value: [dissociation, violent_outburst, self_exile]
  - system: stress
    target: violence_stress
    op: immune
    value: true
  - system: combat
    target: damage_mult
    op: mult
    value: 1.2
  - system: reputation
    target: kinslayer
    op: tag
    value: true
```

---

### #7 W_exile_risen
> **Risen Exile / 되살아난 추방자** — Cast out, survived, returned stronger
> Acquisition: exile_then_success | Rarity: epic
> Academic: Park (2010) — meaning making after adversity; resilience theory

```yaml
effects:
  - system: stress
    target: isolation_stress
    op: immune
    value: true
  - system: skill
    target: survival
    op: mult
    value: 1.3
    tags: [survival, trading, diplomacy]
  - system: skill
    target: trading
    op: mult
    value: 1.3
  - system: skill
    target: diplomacy
    op: mult
    value: 1.3
  - system: derived
    target: risk_tolerance
    op: mult
    value: 1.3
  - system: behavior
    target: self_reliant_solution
    op: inject
    value: { priority: 0.6 }
  - system: relationship
    target: trust_floor
    op: set
    value: 0.15
  - system: need
    target: belonging_sensitivity
    op: mult
    value: 1.3
```

---

### #8 W_first_kill
> **Firstblood / 첫피** — First time taking a life; threshold crossed
> Acquisition: first_kill | Rarity: rare
> Academic: Grossman (2009) — On Killing; moral disengagement (Bandura 2016)

```yaml
effects:
  - system: memory
    target: first_kill_memory
    op: tag
    value: { permanent: true, compressible: false }
  - system: stress
    target: kill_stress
    op: mult
    value: 0.6
  - system: emotion
    target: guilt
    op: add
    value: 0.15
  - system: behavior
    target: avoid_killing
    op: inject
    value: { priority: 0.4 }
    condition:
      source: hexaco
      key: H
      op: gte
      value: 0.6
```

---

### #9 W_old_wolf
> **Old Wolf / 늙은 늑대** — Survived to old age through combat; danger sense refined over decades
> Acquisition: age_60_plus_combat_veteran | Rarity: epic
> Academic: Klein (1998) — recognition-primed decision making; expertise development

```yaml
effects:
  - system: combat
    target: panic_threshold
    op: mult
    value: 0.2
  - system: skill
    target: tactics
    op: mult
    value: 1.5
    tags: [tactics, command, threat_assessment]
  - system: skill
    target: command
    op: mult
    value: 1.5
  - system: skill
    target: threat_assessment
    op: mult
    value: 1.5
  - system: behavior
    target: read_danger
    op: inject
    value: { priority: 0.8 }
  - system: derived
    target: intimidation
    op: mult
    value: 1.3
  - system: body
    target: combat_recovery_rate
    op: mult
    value: 0.7
  - system: stress
    target: reckless_young_behavior_stress
    op: mult
    value: 1.3
```

---

### #10 W_broken_faith
> **Broken Faith / 부서진 믿음** — Lost religion/god; void where meaning was
> Acquisition: faith_loss_event | Rarity: rare
> Academic: Pargament (2007) — religious/spiritual struggles; Exline (2013) — religious doubt
> Special: loss_type: conversion (replaced when agent adopts new faith)

```yaml
effects:
  - system: need
    target: meaning
    op: mult
    value: 1.6
    tags: [void_state]
  - system: stress
    target: existential_stress
    op: mult
    value: 1.4
    tags: [void_state]
  - system: emotion
    target: anticipation
    op: max
    value: 0.6
    tags: [void_state]
  - system: derived
    target: wisdom
    op: mult
    value: 1.3
    condition:
      source: self
      key: has_new_meaning_anchor
      op: eq
      value: true
    tags: [conditional_gift]
  - system: behavior
    target: question_orthodoxy
    op: inject
    value: { priority: 0.7 }
    condition:
      source: self
      key: has_new_meaning_anchor
      op: eq
      value: true
    tags: [conditional_gift]
```

---

### #11 W_touched_by_gods
> **God-Touched / 신의 손길을 입은 자** — The god directly intervened; marked for life
> Acquisition: player_direct_intervention | Rarity: legendary
> Academic: James (1902) — Varieties of Religious Experience; Otto (1917) — The Idea of the Holy

```yaml
effects:
  - system: aura
    target: presence_weight
    op: set
    value: { radius: 4, intensity: 0.3, target_filter: all }
    tags: [presence, gift]
  - system: reputation
    target: god_touched
    op: tag
    value: true
    tags: [gift]
  - system: derived
    target: charisma
    op: mult
    value: 1.4
    tags: [gift]
  - system: derived
    target: trustworthiness
    op: mult
    value: 1.3
    tags: [gift]
  - system: behavior
    target: major_life_decision
    op: on_event
    value: divine_guidance_modifier
    tags: [gift]
  - system: stress
    target: divine_burden_stress
    op: set
    value: 0.1
    tags: [cost]
  - system: need
    target: transcendence
    op: mult
    value: 1.5
    tags: [cost]
  - system: aura
    target: settlement_morale
    op: add
    value: { amount: 0.05, radius: 6 }
    tags: [gift]
  - system: behavior
    target: deny_divine_calling
    op: block
    value: true
    tags: [cost]
```

---

### #12 W_famine_survivor
> **Famine Survivor / 기근 생존자** — Knows what real hunger is
> Acquisition: prolonged_starvation_survived | Rarity: rare
> Academic: Keys et al. (1950) — Minnesota Starvation Experiment; food insecurity psychology

```yaml
effects:
  - system: stress
    target: food_insecurity_stress
    op: mult
    value: 2.0
    tags: [cost]
  - system: behavior
    target: hoard_food
    op: inject
    value: { priority: 0.7 }
    tags: [cost]
  - system: behavior
    target: waste_food
    op: block
    value: true
    tags: [cost]
  - system: stress
    target: mild_hunger_stress
    op: immune
    value: true
    tags: [gift]
  - system: body
    target: starvation_survival_threshold
    op: mult
    value: 0.7
    tags: [gift]
```

---

### #13 W_plague_walker
> **Plague-Walker / 역병을 걷는 자** — Survived plague; death-adjacent calm
> Acquisition: epidemic_survived | Rarity: rare
> Academic: Defoe (1722) — Journal of the Plague Year; epidemiological resilience

```yaml
effects:
  - system: body
    target: disease_infection_chance
    op: mult
    value: 0.4
    tags: [gift]
  - system: stress
    target: epidemic_panic_stress
    op: immune
    value: true
    tags: [gift]
  - system: behavior
    target: care_for_sick
    op: inject
    value: { priority: 0.5 }
    tags: [gift]
  - system: memory
    target: plague_deaths_witnessed
    op: tag
    value: { permanent: true }
    tags: [cost]
  - system: stress
    target: epidemic_outbreak_stress
    op: mult
    value: 0.5
    tags: [cost]
```

---

### #14 W_crown_weight
> **Crown-Burdened / 왕관을 짊어진 자** — 5+ years of leadership; wise and weary
> Acquisition: leader_5_plus_years | Rarity: epic
> Academic: Kahneman (2011) — decision fatigue; leadership loneliness literature

```yaml
effects:
  - system: derived
    target: wisdom
    op: mult
    value: 1.35
    tags: [gift]
  - system: skill
    target: [administration, diplomacy, command]
    op: mult
    value: 1.3
    tags: [gift]
  - system: behavior
    target: think_long_term
    op: inject
    value: { priority: 0.6 }
    tags: [gift]
  - system: need
    target: belonging
    op: mult
    value: 1.3
    tags: [cost]
  - system: stress
    target: responsibility_weight_stress
    op: mult
    value: 1.2
    tags: [cost]
  - system: relationship
    target: genuine_friendship_difficulty
    op: mult
    value: 0.7
    tags: [cost]
```

---

### #15 W_mothers_fury
> **Parent's Fury / 부모의 분노** — Child threatened or killed; protective instinct crossed into primal
> Acquisition: child_endangered_or_killed | Rarity: rare
> Academic: Hrdy (1999) — Mother Nature; parental defense aggression literature

```yaml
effects:
  - system: combat
    target: damage_mult
    op: mult
    value: 1.6
    condition:
      source: self
      key: child_threatened
      op: eq
      value: true
    tags: [gift, conditional]
  - system: stress
    target: fear_when_child_threatened
    op: immune
    value: true
    tags: [gift]
  - system: behavior
    target: protect_child_at_any_cost
    op: inject
    value: { priority: 1.0 }
    tags: [gift]
  - system: stress
    target: child_safety_anxiety
    op: mult
    value: 2.0
    tags: [cost]
  - system: behavior
    target: let_child_take_risks
    op: block
    value: true
    tags: [cost]
```

---

### #16 W_dreaming_prophet
> **Dreaming Prophet / 꿈꾸는 예언자** — Genuine visionary experience; carries the burden
> Acquisition: intrapersonal_high + religious_experience | Rarity: epic
> Academic: Hobson (2002) — dream neuroscience; prophetic tradition in comparative religion

```yaml
effects:
  - system: skill
    target: [oration, ritual, counseling]
    op: mult
    value: 1.4
    tags: [gift]
  - system: behavior
    target: share_vision
    op: inject
    value: { priority: 0.5 }
    tags: [gift]
  - system: behavior
    target: major_settlement_event
    op: on_event
    value: prophetic_insight_event
    tags: [gift]
  - system: stress
    target: vision_burden_stress
    op: set
    value: 0.12
    tags: [cost]
  - system: need
    target: sleep
    op: mult
    value: 1.5
    condition:
      source: self
      key: dream_disturbance
      op: eq
      value: true
    tags: [cost]
  - system: aura
    target: spiritual_weight
    op: set
    value: { radius: 3, intensity: 0.15 }
    tags: [cost]
```

---

### #17 W_chain_breaker
> **Chain-Breaker / 사슬을 끊는 자** — Broke free from oppression; autonomy is sacred
> Acquisition: escaped_oppression | Rarity: rare
> Academic: Fromm (1941) — Escape from Freedom; self-determination theory (Deci & Ryan)

```yaml
effects:
  - system: stress
    target: coercion_stress
    op: immune
    value: true
    tags: [gift]
  - system: behavior
    target: resist_authority
    op: inject
    value: { priority: 0.7 }
    tags: [gift]
  - system: behavior
    target: help_others_escape
    op: inject
    value: { priority: 0.5 }
    tags: [gift]
  - system: behavior
    target: submit_to_authority
    op: block
    value: true
    tags: [cost]
  - system: stress
    target: witnessing_oppression_stress
    op: mult
    value: 1.6
    tags: [cost]
```

---

### #18 W_wanderers_return
> **Returned Wanderer / 돌아온 방랑자** — Wandered long, then returned/settled
> Acquisition: long_wandering_then_settled | Rarity: epic
> Academic: Campbell (1949) — Hero's Journey; cross-cultural adaptation (Berry 1997)

```yaml
effects:
  - system: skill
    target: [navigation, trading, diplomacy, language]
    op: mult
    value: 1.3
    tags: [gift]
  - system: derived
    target: wisdom
    op: mult
    value: 1.25
    tags: [gift]
  - system: behavior
    target: cross_cultural_empathy
    op: inject
    value: { priority: 0.4 }
    tags: [gift]
  - system: stress
    target: isolation_stress
    op: immune
    value: true
    tags: [gift]
  - system: stress
    target: rootlessness_residual_stress
    op: set
    value: 0.08
    tags: [cost]
  - system: need
    target: belonging
    op: mult
    value: 1.4
    tags: [cost]
```

---

## Bloodline Traits (L_) — Genetic Predispositions

> Bloodline traits are inherited genetically, present from birth. They represent latent biological
> predispositions passed from parent to child. Effects are modifiers layered on top of normal stats —
> tendencies that shape development without replacing the base.
> Inheritance types: dominant | recessive | maternal | paternal | founder

---

### #19 L_giants_marrow
> **Giant's Marrow / 거인의 골수** — Physical development skews large and strong
> Inheritance: dominant | Rarity: rare
> Academic: Acromegaly-adjacent growth factor predisposition; somatotype heritability studies

```yaml
effects:
  - system: body
    target: str_growth_rate
    op: mult
    value: 1.25
  - system: body
    target: str_decline_rate
    op: mult
    value: 0.85
  - system: body
    target: height_modifier
    op: add
    value: 0.1
  - system: skill
    target: [heavy_labor, construction]
    op: mult
    value: 1.15
```

---

### #20 L_hawks_gaze
> **Hawk's Gaze / 매의 시선** — Visual acuity and spatial processing exceptional
> Inheritance: dominant | Rarity: rare
> Academic: Visual cortex volume heritability; spatial reasoning twin studies (Linn & Petersen)

```yaml
effects:
  - system: skill
    target: [archery, tracking, navigation, mapmaking]
    op: mult
    value: 1.25
  - system: behavior
    target: spot_hidden
    op: inject
    value: {priority: 0.4}
  - system: combat
    target: ranged_accuracy
    op: mult
    value: 1.2
```

---

### #21 L_winter_blood
> **Winter Blood / 겨울의 피** — Cold tolerance, slow metabolism, enduring
> Inheritance: recessive | Rarity: rare
> Academic: Arctic adaptation genetics (ACTN3, UCP1 variants); Bergmann's rule heritability

```yaml
effects:
  - system: stress
    target: cold_environment_stress
    op: immune
    value: true
  - system: body
    target: cold_endurance
    op: mult
    value: 1.5
  - system: body
    target: heat_tolerance
    op: mult
    value: 0.75
  - system: body
    target: metabolism_rate
    op: mult
    value: 0.85
```

---

### #22 L_summer_veins
> **Summer Veins / 여름의 핏줄** — Heat tolerance, quick energy, fertile
> Inheritance: recessive | Rarity: rare
> Academic: Tropical climate adaptation genetics; thyroid hormone activity heritability

```yaml
effects:
  - system: stress
    target: heat_environment_stress
    op: immune
    value: true
  - system: body
    target: energy_recovery_rate
    op: mult
    value: 1.2
  - system: fertility
    target: fertility_rate
    op: mult
    value: 1.15
  - system: body
    target: cold_tolerance
    op: mult
    value: 0.75
```

---

### #23 L_iron_liver
> **Iron Liver / 철의 간** — Disease resistance and toxin processing unusually strong
> Inheritance: dominant | Rarity: rare
> Academic: CYP450 enzyme polymorphisms; innate immune system heritability (Tian et al.)

```yaml
effects:
  - system: body
    target: disease_infection_chance
    op: mult
    value: 0.65
  - system: body
    target: toxin_resistance
    op: mult
    value: 1.6
  - system: body
    target: chronic_condition_risk
    op: mult
    value: 0.7
```

---

### #24 L_mothers_intuition
> **Mother's Intuition / 어머니의 직감** — Interpersonal reading; matrilineal
> Inheritance: maternal | Rarity: rare
> Academic: Interpersonal sensitivity research; Baron-Cohen empathizing-systemizing theory

```yaml
effects:
  - system: skill
    target: [interpersonal_reading, negotiation, childcare]
    op: mult
    value: 1.2
  - system: derived
    target: charisma
    op: mult
    value: 1.1
  - system: behavior
    target: read_others_emotional_state
    op: inject
    value: {priority: 0.4}
  - system: genetics
    target: maternal_only_inheritance
    op: tag
    value: true
```

---

### #25 L_war_seed
> **War Seed / 전쟁의 씨앗** — Aggression and combat instinct; patrilineal
> Inheritance: paternal | Rarity: rare
> Academic: Testosterone-mediated aggression predisposition; MAOA behavioral genetics studies

```yaml
effects:
  - system: body
    target: combat_aggression_threshold
    op: mult
    value: 0.8
  - system: skill
    target: [combat_skills]
    op: mult
    value: 1.15
  - system: emotion
    target: anger_sensitivity
    op: mult
    value: 1.2
  - system: genetics
    target: paternal_only_inheritance
    op: tag
    value: true
```

---

### #26 L_silver_tongue_blood
> **Silver Tongue / 은빛 혀의 혈통** — Linguistic fluency comes naturally
> Inheritance: dominant | Rarity: rare
> Academic: FOXP2 language gene heritability; verbal ability twin studies (Plomin et al.)

```yaml
effects:
  - system: skill
    target: [linguistic_skills, persuasion, teaching]
    op: mult
    value: 1.2
  - system: body
    target: voice_quality
    op: add
    value: 0.1
  - system: derived
    target: charisma
    op: mult
    value: 1.1
```

---

### #27 L_deep_roots
> **Deep Roots / 깊은 뿌리** — Settlement attachment; hard to move, thrives in place
> Inheritance: recessive | Rarity: rare
> Academic: Sedentism evolutionary psychology; place attachment research (Lewicka 2011)

```yaml
effects:
  - system: stress
    target: migration_stress
    op: mult
    value: 1.5
  - system: need
    target: belonging_in_home_settlement
    op: mult
    value: 1.3
  - system: skill
    target: [local_farming, settlement_administration]
    op: mult
    value: 1.15
  - system: behavior
    target: migrate_probability
    op: mult
    value: 0.3
```

---

### #28 L_starlit_mind
> **Starlit Mind / 별빛의 정신** — Logical-mathematical aptitude
> Inheritance: recessive | Rarity: rare
> Academic: General intelligence heritability (g-factor); spatial-mathematical aptitude twin studies

```yaml
effects:
  - system: skill
    target: [mathematics, research, astronomy]
    op: mult
    value: 1.25
  - system: body
    target: logical_development_rate
    op: mult
    value: 1.2
  - system: derived
    target: wisdom
    op: mult
    value: 1.1
```

---

### #29 L_beast_affinity
> **Beastkin / 짐승핏줄** — Animal rapport and naturalistic intelligence
> Inheritance: maternal | Rarity: rare
> Academic: Gardner's naturalistic intelligence; human-animal bond heritability research

```yaml
effects:
  - system: skill
    target: [animal_training, veterinary, hunting]
    op: mult
    value: 1.25
  - system: behavior
    target: encountering_animal
    op: on_event
    value: hostility_reduced
  - system: genetics
    target: maternal_only_inheritance
    op: tag
    value: true
```

---

### #30 L_stone_bones
> **Stone Bones / 돌의 뼈** — Dense bones; heavy, tough, slower
> Inheritance: dominant | Rarity: rare
> Academic: Bone mineral density heritability (Peacock et al.); body density vs. mobility trade-off

```yaml
effects:
  - system: body
    target: toughness_modifier
    op: mult
    value: 1.2
  - system: body
    target: movement_speed
    op: mult
    value: 0.9
  - system: body
    target: swim_speed
    op: mult
    value: 0.75
  - system: combat
    target: wound_severity_mult
    op: mult
    value: 0.85
```

---

### #31 L_dawn_blessed
> **Dawn-Blessed / 새벽의 축복** — Founding lineage of exceptional vitality and charisma
> Inheritance: founder | Rarity: legendary
> Academic: Founding effect in population genetics; charismatic authority (Weber); heterosis in small founding populations

```yaml
effects:
  - system: body
    target: physical_peak_duration
    op: add
    value: 5
  - system: derived
    target: charisma
    op: mult
    value: 1.25
  - system: body
    target: disease_infection_chance
    op: mult
    value: 0.6
  - system: aging
    target: decline_rate
    op: mult
    value: 0.8
  - system: reputation
    target: bloodline_prestige
    op: set
    value: 0.3
  - system: genetics
    target: dawn_blessed_propagation
    op: mult
    value: 1.0
  - system: aura
    target: morale
    op: add
    value: 0.05
    condition:
      source: self
      key: radius
      op: lte
      value: 3
```

---

### #32 L_thin_blood
> **Thin Blood / 엷은 피** — Weak immune response, prone to illness
> Inheritance: recessive | Rarity: rare
> Academic: Immunodeficiency genetics; compensatory sensitivity hypothesis (Boyce & Ellis 2005)

```yaml
effects:
  - system: body
    target: disease_infection_chance
    op: mult
    value: 1.8

  - system: body
    target: chronic_condition_risk
    op: mult
    value: 1.6

  - system: skill
    target: intrapersonal_skills
    op: mult
    value: 1.1
    tags: [compensatory, vulnerability_sharpens_self]

  - system: skill
    target: empathy_based_skills
    op: mult
    value: 1.1
    tags: [compensatory]
```

---

### #33 L_moon_sickness
> **Moon-Sickness / 달의 병** — Emotional volatility tied to stress cycles
> Inheritance: recessive | Rarity: rare
> Academic: Cyclothymia spectrum; creativity-mood disorder link (Jamison 1993)

```yaml
effects:
  - system: emotion
    target: emotional_swing_amplitude
    op: mult
    value: 1.4
    tags: [highs_higher, lows_lower]

  - system: stress
    target: accumulation_rate
    op: mult
    value: 1.2

  - system: skill
    target: art
    op: mult
    value: 1.15
    tags: [compensatory, emotional_range_feeds_creativity]

  - system: skill
    target: music
    op: mult
    value: 1.15
    tags: [compensatory]

  - system: skill
    target: empathy_based_skills
    op: mult
    value: 1.15
    tags: [compensatory]
```

---

### #34 L_hollow_bones
> **Hollow Bones / 텅 빈 뼈** — Light and agile but fragile
> Inheritance: recessive | Rarity: rare
> Academic: Bone density genetics; gracile vs robust morphology (Ruff 2000)

```yaml
effects:
  - system: body
    target: toughness_modifier
    op: mult
    value: 0.75

  - system: body
    target: movement_speed
    op: mult
    value: 1.15
    tags: [lighter_frame]

  - system: skill
    target: acrobatics
    op: mult
    value: 1.15

  - system: skill
    target: archery
    op: mult
    value: 1.15

  - system: skill
    target: dance
    op: mult
    value: 1.15

  - system: combat
    target: wound_severity_mult
    op: mult
    value: 1.3
    tags: [fragile_under_impact]
```

---

### #35 L_blood_fury
> **Blood Fury / 피의 광기** — Anger escalation; rage comes fast and hard
> Inheritance: paternal | Rarity: rare
> Academic: MAOA gene variants and aggression; intermittent explosive disorder genetics (Brunner et al. 1993)

```yaml
effects:
  - system: emotion
    target: anger_escalation_rate
    op: mult
    value: 1.6
    tags: [anger_spikes_fast]

  - system: combat
    target: berserk_trigger_threshold
    op: mult
    value: 0.7
    tags: [enters_berserk_sooner]

  - system: combat
    target: berserk_damage_mult
    op: mult
    value: 1.3
    tags: [compensatory]

  - system: genetics
    target: paternal_only_inheritance
    op: tag
    value: true
```

---

### #36 L_cursed_womb
> **Cursed Womb / 저주받은 자궁** — Fertility penalty, but surviving children often exceptional
> Inheritance: maternal | Rarity: rare
> Academic: Reproductive trade-off theory (r/K selection); quality vs quantity (MacArthur & Wilson 1967)

```yaml
effects:
  - system: fertility
    target: fertility_rate
    op: mult
    value: 0.5

  - system: on_event
    target: child_survives_to_age_5
    op: inject
    value: exceptional_development_bonus
    tags: [rare_but_exceptional_offspring, quality_over_quantity]

  - system: genetics
    target: maternal_only_inheritance
    op: tag
    value: true
```

---

### #37 L_short_wick
> **Quickflame / 불붙는 성미** — Rapid emotional ignition; quick to passion, quick to fade
> Inheritance: dominant | Rarity: rare
> Academic: Emotional reactivity and regulation (Gross 2002); temperament research (Kagan 1994)

```yaml
effects:
  - system: emotion
    target: emotional_ignition_speed
    op: mult
    value: 1.6
    tags: [fast_to_passion]

  - system: emotion
    target: emotional_duration
    op: mult
    value: 0.6
    tags: [fades_fast]

  - system: relationship
    target: first_impression_volatility
    op: mult
    value: 1.4
```

---

### #38 L_wandering_mind
> **Wandering Mind / 떠도는 정신** — Attention drifts; easily distracted, drawn to novelty
> Inheritance: recessive | Rarity: rare
> Academic: ADHD genetics; mind-wandering and creativity (Baird et al. 2012)

```yaml
effects:
  - system: skill
    target: research
    op: mult
    value: 0.8
    tags: [distracted]

  - system: skill
    target: long_tasks
    op: mult
    value: 0.8
    tags: [distracted]

  - system: behavior
    target: task_abandonment_probability
    op: mult
    value: 1.4

  - system: skill
    target: exploration
    op: mult
    value: 1.2
    tags: [compensatory, novelty_seeker]

  - system: skill
    target: novelty_tasks
    op: mult
    value: 1.2
    tags: [compensatory]
```

---

### #39 L_twin_souled
> **Twin-Souled / 쌍둥이 영혼** — Twins more likely; twins share something uncanny
> Inheritance: recessive | Rarity: rare
> Academic: Dizygotic twinning genetics; twin bond research (Segal 1999)

```yaml
effects:
  - system: fertility
    target: twin_birth_probability
    op: mult
    value: 3.0

  - system: on_event
    target: is_a_twin
    op: inject
    value: sibling_bond_intensity_mult_1.5
    condition:
      source: self
      key: is_twin
      op: eq
      value: true

  - system: relationship
    target: twin_sibling_intimacy_ceil
    op: max
    value: 100
    condition:
      source: self
      key: is_twin
      op: eq
      value: true
    tags: [uncapped_bond]
```

---

### #40 L_old_blood
> **Old Blood / 오래된 피** — Ancient lineage; accumulated wisdom and slow-burning authority
> Inheritance: founder | Rarity: legendary
> Academic: Founder effect; institutional memory in lineages; Weber's traditional authority (1922)

```yaml
effects:
  - system: derived
    target: wisdom
    op: mult
    value: 1.3

  - system: derived
    target: trustworthiness
    op: mult
    value: 1.2

  - system: reputation
    target: bloodline_prestige
    op: set
    value: 0.4
    tags: [lineage_recognition]

  - system: aging
    target: wisdom_growth_rate
    op: mult
    value: 1.3
    tags: [wiser_with_age_faster]

  - system: genetics
    target: old_blood_propagation
    op: mult
    value: 1.0
    tags: [generational_persistence]

  - system: aura
    target: trust
    op: add
    value: 0.05
    tags: [radius_3]
```

---

### #41 L_echo_face
> **Echo Visage / 잔향의 얼굴** — Strong facial resemblance across generations; dynasty recognition
> Inheritance: dominant | Rarity: rare
> Academic: Familial resemblance genetics; kin recognition cues (Bressan & Zucchi 2009)

```yaml
effects:
  - system: reputation
    target: dynasty_recognition
    op: set
    value: 0.3
    tags: [visibly_carries_lineage]

  - system: relationship
    target: lineage_member_trust_bonus
    op: add
    value: 0.1
    condition:
      source: target
      key: shares_bloodline
      op: eq
      value: true

  - system: on_event
    target: meets_ancestor_descendant
    op: inject
    value: recognition_event
    tags: [dynasty_continuity]
```

---

### #42 L_fey_touched
> **Fey-Touched / 요정에게 닿은** — Otherworldly bloodline; ethereal beauty, strange luck, unsettling
> Inheritance: founder | Rarity: legendary
> Academic: Uncanny valley hypothesis (Mori 1970); liminal entity folklore

```yaml
effects:
  - system: derived
    target: allure
    op: mult
    value: 1.5
    tags: [otherworldly_beauty]

  - system: derived
    target: charisma
    op: mult
    value: 1.2

  - system: stress
    target: mundane_life_stress
    op: mult
    value: 1.2
    tags: [ordinary_world_chafes]

  - system: aura
    target: uncanny_presence
    op: set
    value: 0.2
    tags: [radius_3, beautiful_and_unsettling]

  - system: genetics
    target: fey_touched_propagation
    op: mult
    value: 1.0
    tags: [generational_persistence]

  - system: on_event
    target: unusual_event_in_settlement
    op: inject
    value: drawn_toward_it
    tags: [fey_are_drawn_to_strange]
```

---

### #43 L_ember_heart
> **Ember Heart / 잔불의 심장** — Passion ignites late but burns long
> Inheritance: recessive | Rarity: rare
> Academic: Slow-to-warm temperament (Thomas & Chess 1977); grit research (Duckworth 2016)

```yaml
effects:
  - system: behavior
    target: slow_commitment_start
    op: inject
    value: true
    tags: [takes_time_to_engage]

  - system: skill
    target: chosen_craft
    op: mult
    value: 1.5
    condition:
      source: self
      key: dedication_months
      op: gte
      value: 6
    tags: [long_burn_mastery]

  - system: stress
    target: abandonment_of_craft_stress
    op: mult
    value: 1.8
    condition:
      source: self
      key: dedication_months
      op: gte
      value: 6
    tags: [cannot_easily_stop]

  - system: behavior
    target: abandon_long_term_pursuit
    op: block
    value: true
    condition:
      source: self
      key: dedication_months
      op: gte
      value: 6
```
