# WorldSim Trait Effects — Part 2 (Archetype #31~#55)
> Canonical YAML effect definitions using Trait System v3 schema.
> Ops: set / add / mult / min / max / disable / enable / block / inject / override / on_event / tag / immune / replace
> All conditions use structured format: {source, key, op, value}
> Academic references inline per trait.
>
> Part 1: Archetype #1~#30 + Shadow #56~#60 ✅
> **Part 2: Archetype #31~#55 ← this file**
> Part 3: Shadow #6~#15 + Radiance #1~#12
> Part 4: Corpus + Nous
> Part 5: Awakened + Bloodline
> Part 6: Mastery + Bond
> Part 7: Fate + Synergy

---

## Dual-Axis Epics (#31~#34)

### #31 A_mountain_blood
> **Mountain Blood / 산의 피** — Relentless executor who achieves goals without mercy
> Acquisition: C >= 0.83 AND A <= 0.17 | Rarity: epic
> Academic: Karasek (1979) Job Demands-Resources model; Roberts et al. (2005) Conscientiousness and interpersonal coldness

```yaml
- system: stress
  target: failure_stress
  op: immune
  value: true
  tags:
    - resilience
- system: behavior
  target:
    - yield
    - forgive_failure
    - accept_compromise
  op: block
  value: true
- system: skill
  target: all_work
  op: mult
  value: 1.35
  tags:
    - conscientiousness
- system: crafting
  target: quality_bonus
  op: add
  value: 1
- system: relationship
  target: intimacy
  op: max
  value: 55
  tags:
    - emotional_ceiling
- system: stress
  target: break_types
  op: replace
  value:
    obsessive_work: 0.5
    cold_rage: 0.3
    catatonic: 0.2
  condition:
    source: self
    key: stress_level
    op: gt
    value: 0.8
```

---

### #32 A_spring_wind
> **Spring Wind / 봄바람** — Warmly chaotic helper whose kindness is undermined by unreliability
> Acquisition: C <= 0.17 AND A >= 0.83 | Rarity: epic
> Academic: Graziano & Eisenberg (1997) Agreeableness and prosocial behavior; Steel (2007) procrastination and low conscientiousness

```yaml
- system: behavior
  target: spontaneous_help
  op: inject
  value:
    priority: 0.7
  condition:
    source: self
    key: nearby_agent_distressed
    op: eq
    value: true
- system: behavior
  target: comfort_others
  op: inject
  value:
    priority: 0.6
- system: skill
  target: all_work
  op: mult
  value: 0.7
  tags:
    - low_conscientiousness
- system: skill
  target:
    - mediation
    - teaching
    - negotiation
  op: mult
  value: 1.4
  tags:
    - prosocial
- system: stress
  target: own_failure_stress
  op: mult
  value: 2.0
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 1.8
- system: stress
  target: break_types
  op: replace
  value:
    overwhelmed_crying: 0.4
    flee_responsibility: 0.35
    desperate_socializing: 0.25
```

---

### #33 A_torch_bearer
> **Torch-Bearer / 횃불지기** — Visionary whose ideas spread through social energy
> Acquisition: O >= 0.83 AND X >= 0.83 | Rarity: epic
> Academic: Rogers (2003) Diffusion of Innovations; Feist (1998) Openness and creative achievement

```yaml
- system: behavior
  target: share_idea
  op: inject
  value:
    priority: 0.8
  condition:
    source: self
    key: has_new_knowledge
    op: eq
    value: true
- system: behavior
  target: recruit_followers
  op: inject
  value:
    priority: 0.5
- system: event
  target: discovery_chance
  op: mult
  value: 1.6
  tags:
    - openness
- system: aura
  target: values_drift
  op: set
  value:
    radius: 4
    intensity: 0.2
    target_filter: same_settlement
  tags:
    - cultural_diffusion
- system: derived
  target: charisma
  op: add
  value: 0.25
- system: event
  target: tech_spread
  op: on_event
  value:
    on: tech_discovered
    effect: settlement_knowledge_boost
    intensity: 0.3
  tags:
    - signature
- system: skill
  target: long_task_completion
  op: mult
  value: 0.7
  tags:
    - restless_energy
```

---

### #34 A_buried_stone
> **Buried Stone / 묻힌 돌** — Immovable recluse resistant to all change
> Acquisition: O <= 0.17 AND X <= 0.17 | Rarity: epic
> Academic: McCrae (1996) Openness and resistance to change; Ashton & Lee (2007) HEXACO low-X social withdrawal

```yaml
- system: behavior
  target:
    - migrate
    - adopt_new_tech
    - join_new_group
  op: block
  value: true
- system: values
  target: TRADITION
  op: min
  value: 0.7
- system: values
  target: drift_rate
  op: mult
  value: 0.2
- system: stress
  target: forced_change_stress
  op: mult
  value: 3.0
  tags:
    - change_aversion
- system: aura
  target: values_drift
  op: set
  value:
    radius: 2
    intensity: -0.1
    target_filter: same_settlement
  tags:
    - cultural_anchor
- system: derived
  target: intimidation
  op: add
  value: -0.2
- system: derived
  target: wisdom
  op: add
  value: 0.2
  condition:
    source: age
    key: years
    op: gte
    value: 50
```

---

## Triple-Axis Legendaries (#35~#42)

### #35 A_gilded_tongue
> **Gilded Tongue / 금빛 혀** — Brilliant manipulator who charms, creates, and deceives
> Acquisition: X >= 0.83 AND H <= 0.17 AND O >= 0.83 | Rarity: legendary
> Academic: Paulhus & Williams (2002) Dark Triad; Lee & Ashton (2005) H factor and manipulation

```yaml
- system: derived
  target: charisma
  op: mult
  value: 1.8
  tags:
    - defining
- system: behavior
  target:
    - deception
    - charm_manipulation
  op: enable
  value: true
- system: skill
  target:
    - persuasion
    - negotiation
    - deception
  op: mult
  value: 1.6
- system: stress
  target: guilt_from_deception
  op: immune
  value: true
- system: behavior
  target: honest_confession
  op: block
  value: true
- system: event
  target: manipulation_success
  op: on_event
  value:
    on: successful_manipulation
    effect: tag_silver_tongued
    tag: silver_tongued
    duration: 365
  tags:
    - signature
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 1.8
  tags:
    - short_term_charm
- system: relationship
  target: trust
  op: max
  value: 40
  tags:
    - trust_ceiling
- system: reputation
  target: negative_event_impact
  op: mult
  value: 2.5
  condition:
    source: self
    key: caught_in_act
    op: eq
    value: true
  tags:
    - exposure_risk
```

---

### #36 A_silent_judge
> **Silent Judge / 침묵의 심판관** — Incorruptible loner who sees everything clearly
> Acquisition: X <= 0.17 AND H >= 0.83 AND C >= 0.83 | Rarity: legendary
> Academic: Ashton & Lee (2007) Honesty-Humility as predictor of ethical behavior; Costa & McCrae (1992) Introversion and deep processing

```yaml
- system: derived
  target: trustworthiness
  op: mult
  value: 1.8
  tags:
    - defining
- system: behavior
  target:
    - gossip
    - social_flattery
    - accept_bribe
  op: block
  value: true
- system: relationship
  target: max_active_relationships
  op: set
  value: 4
  tags:
    - introvert_cap
- system: skill
  target: all_work
  op: mult
  value: 1.3
  tags:
    - conscientiousness
- system: crafting
  target: quality_bonus
  op: add
  value: 1
- system: event
  target: witness_injustice
  op: on_event
  value:
    on: witnesses_injustice
    effect: inject_report_wrongdoing
    priority: 0.95
  tags:
    - signature
- system: stress
  target: social_obligation_stress
  op: mult
  value: 2.5
- system: stress
  target: crowd_stress
  op: set
  value: 0.1
  condition:
    source: self
    key: nearby_agents
    op: gt
    value: 3
- system: derived
  target: charisma
  op: add
  value: -0.25
  tags:
    - reserved
```

---

### #37 A_embers
> **Dying Embers / 꺼져가는 불씨** — Anxious traditionalist withdrawing from the world
> Acquisition: E >= 0.83 AND X <= 0.17 AND O <= 0.17 | Rarity: legendary
> Academic: Barlow (2002) anxiety and behavioral inhibition; Carver & White (1994) BIS/BAS and Neuroticism

```yaml
- system: emotion
  target: fear
  op: min
  value: 0.25
  tags:
    - background_anxiety
- system: stress
  target: accumulation_rate
  op: mult
  value: 1.8
  tags:
    - hypervigilance
- system: behavior
  target:
    - explore_new_territory
    - try_new_skill
    - make_new_friend
  op: block
  value: true
- system: derived
  target: risk_tolerance
  op: mult
  value: 0.3
  tags:
    - defining
- system: stress
  target: break_types
  op: replace
  value:
    catatonic_withdrawal: 0.4
    obsessive_ritual: 0.35
    panic_attack: 0.25
  tags:
    - signature
- system: need
  target: safety
  op: set
  value:
    decay_rate_mult: 3.0
  tags:
    - hypersensitive
- system: values
  target: drift_rate
  op: mult
  value: 0.15
- system: skill
  target: new_skill_learning
  op: mult
  value: 0.4
- system: relationship
  target: stranger_trust
  op: set
  value: -30
  tags:
    - withdrawal
```

---

### #38 A_dawn_hammer
> **Dawn Hammer / 새벽의 망치** — Driven taskmaster who burns people out
> Acquisition: C >= 0.83 AND X >= 0.83 AND A <= 0.17 | Rarity: legendary
> Academic: Bass (1985) transformational vs transactional leadership; Schaubroeck et al. (2007) destructive leadership

```yaml
- system: skill
  target: leadership
  op: mult
  value: 1.8
  tags:
    - taskmaster
- system: behavior
  target: rally_workers
  op: inject
  value:
    priority: 0.85
  condition:
    source: self
    key: is_leader
    op: eq
    value: true
- system: behavior
  target: set_demanding_goals
  op: inject
  value:
    priority: 0.8
- system: aura
  target: work_drive
  op: set
  value:
    radius: 5
    intensity: 1.35
    target_filter: same_settlement
    side_effect:
      system: stress
      target: accumulation_rate
      op: mult
      value: 1.6
  tags:
    - signature
- system: derived
  target: charisma
  op: add
  value: 0.25
- system: derived
  target: intimidation
  op: add
  value: 0.3
- system: behavior
  target:
    - offer_rest
    - accept_excuse
    - grant_leave
  op: block
  value: true
- system: event
  target: settlement_production
  op: mult
  value: 1.3
  condition:
    source: self
    key: is_leader
    op: eq
    value: true
- system: relationship
  target: subordinate_loyalty_decay
  op: mult
  value: 1.8
  tags:
    - burnout_cost
- system: stress
  target: subordinate_failure
  op: set
  value: 0.15
  condition:
    source: self
    key: subordinate_failed_task
    op: eq
    value: true
```

---

### #39 A_woven_fate
> **Woven Fate / 엮인 운명** — Deeply feeling empathic saint
> Acquisition: E >= 0.83 AND H >= 0.83 AND A >= 0.83 | Rarity: legendary
> Academic: Figley (1995) compassion fatigue; Davis (1983) empathic concern and personal distress

```yaml
- system: behavior
  target: comfort_grieving
  op: inject
  value:
    priority: 0.9
  condition:
    source: target
    key: emotion.sadness
    op: gt
    value: 0.5
- system: behavior
  target: advocate_for_weak
  op: inject
  value:
    priority: 0.75
  condition:
    source: target
    key: social_rank
    op: lt
    value: 0.3
- system: relationship
  target: trust_gain_rate
  op: mult
  value: 1.8
  tags:
    - empathy_bond
- system: stress
  target: others_suffering
  op: set
  value: 0.12
  condition:
    source: self
    key: witnessed_suffering
    op: eq
    value: true
- system: emotion
  target: guilt
  op: min
  value: 0.25
  tags:
    - moral_sensitivity
- system: derived
  target: trustworthiness
  op: add
  value: 0.35
- system: stress
  target: break_types
  op: replace
  value:
    compassion_fatigue: 0.5
    sobbing_fit: 0.3
    catatonic: 0.2
  tags:
    - signature
- system: stress
  target: personal_loss
  op: mult
  value: 0.7
  tags:
    - meaning_in_sacrifice
- system: behavior
  target: console_effectiveness
  op: mult
  value: 2.0
- system: emotion
  target: sadness
  op: min
  value: 0.15
  condition:
    source: self
    key: nearby_agent_distressed
    op: eq
    value: true
```

---

### #40 A_tidal_mind
> **Tidal Mind / 조석의 정신** — Creative genius who can't follow through
> Acquisition: O >= 0.83 AND E >= 0.83 AND C <= 0.17 | Rarity: legendary
> Academic: Jamison (1993) creativity and mood disorders; Baas et al. (2008) mood and creativity meta-analysis

```yaml
- system: skill
  target:
    - painting
    - music
    - poetry
    - architecture
    - invention
  op: mult
  value: 1.6
  condition:
    source: self
    key: emotion.joy
    op: gt
    value: 0.6
  tags:
    - creative_peak
- system: skill
  target:
    - painting
    - music
    - poetry
    - architecture
    - invention
  op: mult
  value: 0.5
  condition:
    source: self
    key: emotion.sadness
    op: gt
    value: 0.5
  tags:
    - creative_trough
- system: skill
  target: long_task_completion
  op: mult
  value: 0.4
  tags:
    - abandonment
- system: derived
  target: creativity
  op: add
  value: 0.4
- system: stress
  target: break_types
  op: replace
  value:
    manic_episode: 0.4
    creative_collapse: 0.3
    sobbing_fit: 0.2
    wander: 0.1
- system: emotion
  target: volatility
  op: mult
  value: 2.5
- system: behavior
  target: inspired_creation
  op: on_event
  value:
    on: emotion_peak
    inject: inspired_creation
    priority: 1.0
    duration: short
  tags:
    - signature
- system: behavior
  target: abandon_task
  op: inject
  value:
    priority: 0.5
  condition:
    source: self
    key: same_task_days
    op: gt
    value: 5
- system: need
  target: competence
  op: set
  value:
    satisfaction_mult: 0.4
  condition:
    source: self
    key: same_job_years
    op: gt
    value: 1
```

---

### #41 A_unmoving_peak
> **Unmoving Peak / 부동의 봉우리** — Cold efficiency engine without emotion
> Acquisition: E <= 0.17 AND A <= 0.17 AND C >= 0.83 | Rarity: legendary
> Academic: Cleckley (1941) primary psychopathy traits; Lilienfeld & Andrews (1996) fearlessness and emotional detachment

```yaml
- system: stress
  target: source_immunity
  op: set
  value:
    - emotional_distress
    - guilt
    - grief
    - social_rejection
    - loneliness
- system: behavior
  target:
    - comfort
    - empathize
    - forgive
    - express_grief
    - console
  op: block
  value: true
- system: skill
  target: all_work
  op: mult
  value: 1.35
  tags:
    - pure_efficiency
- system: crafting
  target: quality_bonus
  op: add
  value: 1
- system: relationship
  target: intimacy
  op: max
  value: 40
  tags:
    - warmth_ceiling
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 0.3
- system: derived
  target: intimidation
  op: add
  value: 0.4
- system: aura
  target: fear
  op: set
  value:
    radius: 3
    intensity: 0.2
    target_filter: all
  tags:
    - signature
- system: stress
  target: break_types
  op: replace
  value:
    mechanical_detachment: 0.5
    cold_rage: 0.3
    catatonic: 0.2
- system: emotion
  target: intensity_mult
  op: set
  value: 0.15
```

---

### #42 A_wandering_star
> **Wandering Star / 떠도는 별** — Brilliant drifter who never finishes anything
> Acquisition: O >= 0.83 AND X >= 0.83 AND C <= 0.17 | Rarity: legendary
> Academic: Zuckerman (1994) sensation seeking; DeYoung (2015) Openness/Intellect and exploration

```yaml
- system: behavior
  target: explore
  op: inject
  value:
    priority: 0.8
  condition:
    source: self
    key: unknown_territory_nearby
    op: eq
    value: true
- system: behavior
  target: start_new_project
  op: inject
  value:
    priority: 0.7
- system: behavior
  target: make_new_friend
  op: inject
  value:
    priority: 0.6
- system: behavior
  target: migration_resistance
  op: set
  value: 0.0
  tags:
    - wanderlust
- system: skill
  target: long_task_completion
  op: mult
  value: 0.4
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 1.8
  tags:
    - fast_bonding
- system: relationship
  target: intimacy_decay_rate
  op: mult
  value: 2.5
  tags:
    - poor_maintenance
- system: derived
  target: creativity
  op: add
  value: 0.25
- system: stress
  target: boredom
  op: set
  value: 0.12
  condition:
    source: self
    key: same_task_days
    op: gt
    value: 3
- system: behavior
  target: commit_long_term
  op: block
  value: true
  condition:
    source: self
    key: same_settlement_years
    op: gt
    value: 2
  tags:
    - signature
- system: emotion
  target: joy
  op: add
  value: 0.35
  condition:
    source: self
    key: arrived_new_settlement
    op: eq
    value: true
```

---

## Value-Axis Epics (#43~#55)

### #43 A_blood_price
> **Blood Price / 피의 대가** — War-born predator obsessed with combat
> Acquisition: values.martial >= 0.88 AND A <= 0.17 | Rarity: epic
> Academic: Grossman (1995) psychology of killing; Nisbett & Cohen (1996) culture of honor

```yaml
- system: combat
  target: damage_mult
  op: mult
  value: 1.5
- system: combat
  target: morale_floor
  op: set
  value: 0.5
  tags:
    - unbreakable_will
- system: behavior
  target:
    - surrender
    - show_mercy
    - negotiate_peace
  op: block
  value: true
  condition:
    source: self
    key: combat_advantage
    op: gt
    value: 0.3
- system: stress
  target: kill_trauma
  op: immune
  value: true
- system: derived
  target: intimidation
  op: add
  value: 0.35
- system: stress
  target: forced_peace
  op: set
  value: 0.1
  condition:
    source: self
    key: no_combat_days
    op: gt
    value: 20
- system: memory
  target: kill_trauma
  op: on_event
  value:
    on: combat_kill
    inject_memory: honor_kill
    valence: positive
    intensity: 0.6
- system: reputation
  target: tags
  op: tag
  value: feared_warrior
```

---

### #44 A_golden_scale
> **Golden Scale / 황금 저울** — Honest merchant for whom fair trade is a moral covenant
> Acquisition: values.commerce >= 0.88 AND H >= 0.83 | Rarity: epic
> Academic: Henrich et al. (2001) cooperation and market integration; Ashton & Lee (2007) H-factor in economic behavior

```yaml
- system: behavior
  target:
    - price_manipulation
    - fraud
    - break_trade_agreement
    - smuggle
  op: block
  value: true
  tags:
    - fair_trade_oath
- system: skill
  target:
    - trading
    - negotiation
    - appraisal
  op: mult
  value: 1.4
- system: stress
  target: forced_dishonest_trade
  op: set
  value: 0.25
  condition:
    source: self
    key: forced_to_cheat
    op: eq
    value: true
- system: reputation
  target: tags
  op: tag
  value: fair_dealer
  condition:
    source: self
    key: successful_trades
    op: gte
    value: 10
- system: derived
  target: trustworthiness
  op: add
  value: 0.3
- system: relationship
  target: trust_gain_rate
  op: mult
  value: 1.6
  condition:
    source: target
    key: is_trade_partner
    op: eq
    value: true
  tags:
    - merchant_bond
- system: behavior
  target: trade_pricing
  op: override
  value: fair_price_only
```

---

### #45 A_green_covenant
> **Green Covenant / 녹색 서약** — Nature mystic who prioritizes environmental harmony over civilization
> Acquisition: values.nature >= 0.88 AND O >= 0.83 | Rarity: epic
> Academic: Wilson (1984) biophilia hypothesis; Kellert & Wilson (1993) nature and human development

```yaml
- system: skill
  target:
    - gathering
    - farming
    - herbalism
    - animal_husbandry
    - naturalism
  op: mult
  value: 1.45
  tags:
    - nature_mastery
- system: behavior
  target:
    - overmine
    - clear_forest
    - pollute_water
    - strip_harvest
    - drain_wetland
  op: block
  value: true
  tags:
    - ecological_ethic
- system: stress
  target: nature_destruction_stress
  op: set
  value: 0.2
  condition:
    source: settlement
    key: environmental_damage
    op: gt
    value: 0.3
- system: emotion
  target: joy
  op: on_event
  value:
    on: entered_pristine_wilderness
    intensity: 0.5
    duration: 48
  tags:
    - commune_with_nature
- system: behavior
  target: accept_heavy_industry
  op: set
  value:
    weight_mult: 0.1
  condition:
    source: settlement
    key: industry_level
    op: gt
    value: 2
- system: derived
  target: naturalistic_intelligence
  op: mult
  value: 1.6
- system: need
  target: meaning
  op: set
  value:
    satisfaction_mult: 2.0
  condition:
    source: self
    key: current_biome
    op: eq
    value: wilderness
```

---

### #46 A_eternal_student
> **Eternal Student / 영원한 학도** — Knowledge addict for whom stagnation is death
> Acquisition: values.knowledge >= 0.88 AND O >= 0.83 | Rarity: epic
> Academic: Cacioppo & Petty (1982) need for cognition; Kashdan et al. (2004) curiosity and intrinsic motivation

```yaml
- system: skill
  target: all_learning
  op: mult
  value: 1.4
  tags:
    - knowledge_hunger
- system: behavior
  target:
    - seek_teacher
    - read_available_texts
    - ask_questions
    - attend_lecture
  op: inject
  value:
    priority: 0.7
- system: need
  target: competence
  op: set
  value:
    decay_rate_mult: 2.5
- system: stress
  target: knowledge_stagnation_stress
  op: set
  value: 0.15
  condition:
    source: self
    key: days_without_learning
    op: gt
    value: 10
- system: emotion
  target: joy
  op: on_event
  value:
    on: skill_level_unlocked
    intensity: 0.6
    duration: 24
- system: derived
  target: wisdom
  op: mult
  value: 1.3
  condition:
    source: self
    key: age
    op: gt
    value: 40
- system: behavior
  target: decision_speed
  op: mult
  value: 0.7
  tags:
    - analysis_paralysis
```

---

### #47 A_broken_chain
> **Broken Chain / 끊어진 사슬** — Unchainable rebel who resists all authority at any cost
> Acquisition: values.independence >= 0.88 AND A <= 0.17 | Rarity: epic
> Academic: Deci & Ryan (2000) self-determination theory; Brehm (1966) psychological reactance theory

```yaml
- system: behavior
  target:
    - submit_to_authority
    - follow_orders_against_values
    - join_hierarchy
    - accept_punishment_quietly
  op: block
  value: true
  tags:
    - unchainable
- system: stress
  target: coercion_stress
  op: mult
  value: 3.0
- system: behavior
  target: resist_order
  op: inject
  value:
    priority: 0.9
  condition:
    source: leader
    key: gave_direct_order
    op: eq
    value: true
- system: behavior
  target: obedience
  op: override
  value: near_zero
  condition:
    source: self
    key: order_source
    op: eq
    value: authority
- system: behavior
  target: migration_resistance
  op: set
  value: 0.0
  condition:
    source: self
    key: autonomy_threatened
    op: eq
    value: true
- system: derived
  target: risk_tolerance
  op: mult
  value: 1.6
- system: relationship
  target: subordinate_loyalty
  op: mult
  value: 0.0
  tags:
    - refuses_hierarchy
```

---

### #48 A_ancestor_voice
> **Ancestor's Voice / 선조의 목소리** — Living monument for whom the past is sacred and change is corruption
> Acquisition: values.tradition >= 0.88 AND O <= 0.17 | Rarity: epic
> Academic: Schwartz (1992) tradition value domain; Jost et al. (2003) political conservatism as motivated social cognition

```yaml
- system: behavior
  target:
    - adopt_foreign_custom
    - abandon_tradition
    - try_new_technique
    - accept_cultural_reform
  op: block
  value: true
  tags:
    - tradition_guardian
- system: values
  target: drift_rate
  op: mult
  value: 0.15
  tags:
    - cultural_anchor
- system: stress
  target: cultural_change_stress
  op: set
  value: 0.2
  condition:
    source: settlement
    key: tradition_abandoned
    op: eq
    value: true
- system: emotion
  target: joy
  op: on_event
  value:
    on: participated_traditional_ritual
    intensity: 0.7
    duration: 72
- system: aura
  target: cultural_conservatism
  op: set
  value:
    radius: 3
    intensity: 0.1
    target_filter: same_settlement
  tags:
    - tradition_aura
- system: derived
  target: trustworthiness
  op: mult
  value: 1.4
  condition:
    source: target
    key: culture
    op: eq
    value: same_as_self
- system: skill
  target: new_skill_learning
  op: mult
  value: 0.5
  tags:
    - closed_mind
```

---

### #49 A_last_stand
> **Last Stand / 최후의 보루** — Self-immolating protector driven by anxiety about others' safety
> Acquisition: values.sacrifice >= 0.88 AND E >= 0.83 | Rarity: epic
> Academic: Batson (2011) altruism and empathic concern; Oliner & Oliner (1988) altruistic personality rescuers

```yaml
- system: behavior
  target:
    - shield_ally
    - take_hit_for_other
    - volunteer_dangerous_task
  op: inject
  value:
    priority: 0.9
  tags:
    - self_sacrifice
- system: combat
  target: ally_damage_redirect
  op: override
  value: absorb_adjacent_ally_hit
  tags:
    - living_shield
- system: stress
  target: others_in_danger_stress
  op: mult
  value: 3.0
- system: stress
  target: own_death_fear
  op: immune
  value: true
- system: emotion
  target: sadness
  op: on_event
  value:
    on: ally_died
    intensity: 0.9
    duration: 120
- system: behavior
  target: avenge_fallen
  op: inject
  value:
    priority: 0.95
  condition:
    source: self
    key: ally_died_in_combat
    op: eq
    value: true
- system: aura
  target: combat_morale
  op: set
  value:
    radius: 4
    intensity: 0.2
    target_filter: allies
  condition:
    source: self
    key: in_combat
    op: eq
    value: true
```

---

### #50 A_merrymaker
> **Merrymaker / 흥꾼** — Joy engine who broadcasts infectious celebration
> Acquisition: values.merriment >= 0.88 AND X >= 0.83 | Rarity: epic
> Academic: Fredrickson (2001) broaden-and-build theory; Gervais & Wilson (2005) evolution of laughter

```yaml
- system: aura
  target: joy
  op: set
  value:
    radius: 4
    intensity: 0.12
    target_filter: all
  tags:
    - infectious_joy
- system: behavior
  target:
    - organize_festival
    - tell_joke
    - celebrate_small_wins
    - start_dance
  op: inject
  value:
    priority: 0.6
- system: stress
  target: others_persistent_sadness_stress
  op: set
  value: 0.1
  condition:
    source: self
    key: nearby_sad_agents
    op: gt
    value: 2
- system: stress
  target: boredom_stress
  op: immune
  value: true
- system: aura
  target: joy
  op: set
  value:
    radius: 8
    intensity: 0.25
    target_filter: all
  condition:
    source: self
    key: in_festival
    op: eq
    value: true
  tags:
    - festival_amplification
- system: stress
  target: break_types
  op: replace
  value:
    hollow_laughter: 0.5
    sobbing_fit: 0.3
    catatonic: 0.2
  tags:
    - duchenne_collapse
- system: need
  target: belonging
  op: set
  value:
    satisfaction_mult: 1.8
```

---

### #51 A_veiled_blade
> **Veiled Blade / 숨은 칼날** — Shadow operator for whom manipulation is an art form
> Acquisition: values.cunning >= 0.88 AND H <= 0.17 | Rarity: epic
> Academic: Christie & Geis (1970) Machiavellianism; Jones & Paulhus (2014) Dark Triad

```yaml
- system: skill
  target:
    - espionage
    - deception
    - negotiation
  op: mult
  value: 1.5
  tags:
    - shadow_arts
- system: behavior
  target:
    - plant_false_information
    - manipulate_relationship
    - sow_discord
  op: inject
  value:
    priority: 0.7
- system: stress
  target:
    - guilt_from_manipulation
    - guilt_from_deception
  op: immune
  value: true
- system: derived
  target: charisma
  op: mult
  value: 1.4
  condition:
    source: self
    key: interaction_type
    op: eq
    value: manipulation
  tags:
    - instrumental_charm
- system: relationship
  target: trust_gain_rate
  op: mult
  value: 1.8
  tags:
    - false_trustworthiness
- system: memory
  target: outwitted_target
  op: on_event
  value:
    on: successful_manipulation
    intensity: 0.5
    valence: positive
  tags:
    - artisan_pride
- system: behavior
  target: reveal_true_motive
  op: block
  value: true
```

---

### #52 A_scales_keeper
> **Scales-Keeper / 저울의 수호자** — Justice embodied, cannot witness injustice without acting
> Acquisition: values.fairness >= 0.88 AND H >= 0.83 | Rarity: epic
> Academic: Tyler (2006) procedural justice; Lind & Tyler (1988) social psychology of procedural justice

```yaml
- system: behavior
  target: intervene_in_injustice
  op: inject
  value:
    priority: 0.9
  condition:
    source: self
    key: witnessed_injustice
    op: eq
    value: true
  tags:
    - justice
- system: behavior
  target:
    - report_corruption
    - advocate_for_fair_treatment
  op: inject
  value:
    priority: 0.75
  tags:
    - justice
- system: behavior
  target:
    - ignore_injustice
    - accept_unfair_deal
    - give_preferential_treatment
  op: block
  value: true
- system: stress
  target: witnessed_injustice_stress
  op: mult
  value: 2.5
  tags:
    - moral_injury
- system: derived
  target: trustworthiness
  op: mult
  value: 1.4
- system: event
  target: injustice_resolved
  op: on_event
  value:
    on: successfully_resolved_injustice
    effects:
      - system: emotion
        target: joy
        op: add
        value: 0.5
      - system: reputation
        target: tags
        op: tag
        value: arbiter
- system: reputation
  target: tags
  op: tag
  value: just
- system: relationship
  target: trust_gain_rate
  op: mult
  value: 1.8
  condition:
    source: target
    key: was_defended_by_self
    op: eq
    value: true
```

---

### #53 A_craft_soul
> **Craft-Soul / 장인의 혼** — Master artisan for whom quality is sacred and mediocrity is sin
> Acquisition: values.craftsmanship >= 0.88 AND C >= 0.83 | Rarity: epic
> Academic: Csikszentmihalyi (1990) flow and optimal experience; Sennett (2008) The Craftsman

```yaml
- system: crafting
  target: quality_bonus
  op: add
  value: 3
  tags:
    - master_artisan
- system: skill
  target:
    - crafting
    - construction
    - smithing
    - woodworking
  op: mult
  value: 1.4
- system: skill
  target: all_work
  op: mult
  value: 0.75
  tags:
    - refuses_to_rush
- system: behavior
  target:
    - cut_corners
    - produce_low_quality_deliberately
  op: block
  value: true
- system: stress
  target: forced_rush_stress
  op: set
  value: 0.15
  condition:
    source: self
    key: task_deadline_pressure
    op: eq
    value: true
- system: event
  target: masterwork_created
  op: on_event
  value:
    on: produced_masterwork_item
    effects:
      - system: emotion
        target: joy
        op: add
        value: 0.6
      - system: behavior
        target: protect_masterwork
        op: inject
        value:
          priority: 0.85
- system: derived
  target: creativity
  op: mult
  value: 1.35
  condition:
    source: self
    key: is_crafting
    op: eq
    value: true
- system: need
  target: competence
  op: set
  value:
    satisfaction_mult: 1.6
```

---

### #54 A_lotus_eater
> **Lotus Eater / 연화탐닉자** — Pleasure sovereign who maximizes comfort and minimizes effort
> Acquisition: values.leisure >= 0.88 AND C <= 0.17 | Rarity: epic
> Academic: Kahneman et al. (1999) hedonic psychology; Csikszentmihalyi (1990) passive vs active leisure

```yaml
- system: need
  target: warmth
  op: set
  value:
    decay_rate_mult: 2.5
  tags:
    - comfort_dependent
- system: stress
  target: discomfort_sensitivity
  op: mult
  value: 2.0
- system: skill
  target: all_work
  op: mult
  value: 0.65
  tags:
    - minimum_effort
- system: behavior
  target: task_abandonment
  op: mult
  value: 1.8
  condition:
    source: self
    key: task_difficulty
    op: gt
    value: 0.6
- system: behavior
  target:
    - seek_comfort
    - avoid_effort
    - find_entertainment
  op: inject
  value:
    priority: 0.7
- system: stress
  target: boredom_stress
  op: immune
  value: true
  tags:
    - self_entertainer
- system: stress
  target: hard_labor_stress
  op: mult
  value: 2.5
- system: behavior
  target:
    - volunteer_difficult_task
    - persist_through_hardship
  op: block
  value: true
- system: need
  target: autonomy
  op: set
  value:
    satisfaction_mult: 1.5
```

---

### #55 A_rival_seeker
> **Rival-Seeker / 맞수 사냥꾼** — Glory hunter who needs worthy opponents
> Acquisition: values.competition >= 0.88 AND X >= 0.83 | Rarity: epic
> Academic: Reeve (2014) competence motivation; Elliot & Church (1997) achievement motivation

```yaml
- system: behavior
  target:
    - challenge_strongest
    - seek_competition
    - boast_victory
  op: inject
  value:
    priority: 0.75
  tags:
    - glory_seeker
- system: combat
  target: damage_mult
  op: mult
  value: 1.5
  condition:
    source: target
    key: combat_strength
    op: gte
    value: self.combat_strength
  tags:
    - worthy_opponent
- system: skill
  target: all_work
  op: mult
  value: 1.3
  condition:
    source: target
    key: skill_level
    op: gte
    value: self.skill_level
- system: stress
  target: no_worthy_rival_stress
  op: set
  value: 0.1
  condition:
    source: self
    key: days_without_challenge
    op: gt
    value: 30
- system: derived
  target: charisma
  op: mult
  value: 1.3
  tags:
    - showmanship
- system: event
  target: worthy_rival_defeated
  op: on_event
  value:
    on: defeated_equal_or_stronger_opponent
    effects:
      - system: emotion
        target: joy
        op: add
        value: 0.6
      - system: behavior
        target: seek_next_rival
        op: inject
        value:
          priority: 0.8
- system: behavior
  target: attack_weak_target
  op: block
  value: true
  tags:
    - honor_code
- system: reputation
  target: tags
  op: tag
  value: champion
  condition:
    source: self
    key: worthy_victories
    op: gte
    value: 5
```
