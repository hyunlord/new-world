# WorldSim Trait Effects - Part 1 (Archetype #1~#30, Shadow #56~#60)
> Canonical YAML definitions using Trait System v3 schema.
> All ops use v3 canonical names. All conditions use structured format.

## Archetype: Single-Axis (#1~#12)

### #1 A_incorruptible
```yaml
- system: behavior
  target:
    - accept_bribe
    - steal
    - fraud
    - embezzle
  op: block
  value: true
  tags:
    - integrity
- system: behavior
  target: trade_pricing
  op: override
  value: fair_price_only
- system: derived
  target: trustworthiness
  op: add
  value: 0.25
- system: reputation
  target: tags
  op: tag
  value: untarnished
- system: behavior
  target:
    - compromise
    - bluff
    - deceive
  op: block
  value: true
- system: behavior
  target: diplomacy_filter
  op: set
  value: reject_if_unfair
- system: stress
  target: corruption_exposure
  op: set
  value: 0.1
  condition:
    source: leader
    key: tags
    op: has
    value: corrupt
```

### #2 A_serpent_tongue
```yaml
- system: behavior
  target:
    - fraud
    - smuggle
    - bribe
    - forge_document
  op: enable
  value: true
- system: skill
  target:
    - deception
    - persuasion
  op: mult
  value: 1.4
- system: relationship
  target: betrayal_cooldown
  op: mult
  value: 0.0
- system: stress
  target: betrayal_guilt
  op: immune
  value: true
- system: reputation
  target: negative_event_impact
  op: mult
  value: 2.0
  condition:
    source: self
    key: caught_in_act
    op: eq
    value: true
- system: relationship
  target: trust_repair_rate
  op: set
  value: 0.0
  condition:
    source: target
    key: knows_betrayal
    op: eq
    value: true
- system: derived
  target: deception_resistance
  op: add
  value: 0.3
```

### #3 A_glass_heart
```yaml
- system: emotion
  target: intensity_mult
  op: set
  value: 2.0
- system: emotion
  target: decay_rate
  op: mult
  value: 0.5
- system: stress
  target: accumulation_rate
  op: mult
  value: 1.8
- system: stress
  target: mental_break_threshold
  op: add
  value: -0.15
- system: stress
  target: break_types
  op: replace
  value:
    crying_fit: 0.4
    creative_frenzy: 0.3
    catatonic: 0.2
    berserk: 0.1
- system: skill
  target:
    - music
    - poetry
    - painting
    - dance
    - theater
  op: mult
  value: 1.3
  tags:
    - art_bonus
- system: memory
  target: trauma_intensity
  op: mult
  value: 1.5
- system: memory
  target: positive_intensity
  op: mult
  value: 1.5
- system: combat
  target: morale
  op: add
  value: -0.3
- system: combat
  target: flee_threshold
  op: add
  value: 0.15
- system: relationship
  target: bond_event_impact
  op: mult
  value: 2.0
```

### #4 A_stone_blood
```yaml
- system: stress
  target: accumulation_rate
  op: mult
  value: 0.2
- system: stress
  target: mental_break_threshold
  op: add
  value: 0.3
- system: emotion
  target: fear
  op: max
  value: 0.1
- system: emotion
  target:
    - joy
    - sadness
    - trust
    - surprise
  op: max
  value: 0.5
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 0.5
- system: relationship
  target: intimacy
  op: max
  value: 70
- system: behavior
  target:
    - console
    - empathize
    - cry
    - express_grief
  op: block
  value: true
- system: combat
  target: morale_floor
  op: set
  value: 0.4
```

### #5 A_bonfire
```yaml
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 2.0
- system: relationship
  target: first_impression
  op: add
  value: 25
- system: aura
  target: joy
  op: set
  value:
    radius: 3
    intensity: 0.15
    target_filter: all
- system: need
  target: belonging
  op: set
  value:
    decay_rate_mult: 3.0
- system: need
  target: intimacy
  op: set
  value:
    decay_rate_mult: 2.5
- system: stress
  target: isolation_stress
  op: mult
  value: 4.0
- system: stress
  target: break_types
  op: replace
  value:
    sobbing_fit: 0.5
    desperate_socializing: 0.3
    wander: 0.2
  condition:
    source: self
    key: isolation_days
    op: gt
    value: 3
- system: skill
  target:
    - persuasion
    - negotiation
    - teaching
  op: mult
  value: 1.2
- system: event
  target: festival_effect
  op: mult
  value: 1.5
```

### #6 A_deep_well
```yaml
- system: skill
  target: all_work
  op: mult
  value: 1.3
  condition:
    source: self
    key: is_alone
    op: eq
    value: true
- system: derived
  target: wisdom
  op: add
  value: 0.15
  tags:
    - introspection
- system: stress
  target: introspection_recovery
  op: mult
  value: 2.0
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 0.0
  condition:
    source: self
    key: in_social_event
    op: eq
    value: true
- system: derived
  target: charisma
  op: add
  value: -0.3
- system: stress
  target: crowd_stress
  op: set
  value: 0.08
  condition:
    source: self
    key: nearby_agents
    op: gt
    value: 5
- system: stress
  target: isolation_stress
  op: set
  value: 0.0
- system: need
  target: belonging
  op: set
  value:
    decay_rate_mult: 0.3
```

### #7 A_open_palm
```yaml
- system: skill
  target:
    - negotiation
    - mediation
  op: mult
  value: 1.4
- system: relationship
  target: alliance_decay
  op: mult
  value: 0.7
- system: behavior
  target:
    - refuse_request
    - reject_proposal
    - deny_entry
  op: block
  value: true
- system: behavior
  target: exploitation_detection
  op: mult
  value: 0.5
- system: need
  target: autonomy
  op: set
  value:
    satisfaction_mult: 0.5
- system: stress
  target: manipulation_resistance
  op: mult
  value: 0.5
```

### #8 A_iron_wall
```yaml
- system: stress
  target: manipulation_resistance
  op: set
  value: 1.0
- system: behavior
  target: intimidation_resistance
  op: add
  value: 0.5
- system: skill
  target: debate
  op: mult
  value: 1.3
- system: behavior
  target: cooperation_accept
  op: mult
  value: 0.6
- system: relationship
  target: intimacy_decay_rate
  op: mult
  value: 2.0
- system: behavior
  target:
    - compromise
    - concede
    - apologize
  op: set
  value:
    weight_mult: 0.2
- system: derived
  target: intimidation
  op: add
  value: 0.2
```

### #9 A_clockwork
```yaml
- system: skill
  target: all_work
  op: mult
  value: 1.25
- system: crafting
  target: quality_bonus
  op: add
  value: 1
- system: behavior
  target:
    - break_rule
    - skip_task
    - shirk_duty
  op: block
  value: true
- system: behavior
  target: improvisation
  op: mult
  value: 0.6
- system: stress
  target: unexpected_event_stress
  op: mult
  value: 2.5
- system: behavior
  target: accept_change
  op: mult
  value: 0.3
  condition:
    source: self
    key: change_type
    op: eq
    value: schedule
    note: also triggers on method
- system: stress
  target: routine_disruption
  op: set
  value: 0.15
```

### #10 A_wildfire
```yaml
- system: behavior
  target: improvisation
  op: mult
  value: 1.4
- system: behavior
  target: crisis_response_speed
  op: mult
  value: 1.3
- system: skill
  target: long_task_completion
  op: mult
  value: 0.5
- system: behavior
  target: promise_fulfillment
  op: mult
  value: 0.6
- system: behavior
  target: rule_compliance
  op: mult
  value: 0.3
- system: stress
  target: repetition_stress
  op: set
  value: 0.1
  condition:
    source: self
    key: same_task_days
    op: gt
    value: 3
- system: emotion
  target: joy
  op: add
  value: 0.3
  tags:
    - novelty_burst
  condition:
    source: self
    key: new_task_started
    op: eq
    value: true
```

### #11 A_horizon_seeker
```yaml
- system: skill
  target: new_skill_learning
  op: mult
  value: 1.4
- system: event
  target: discovery_chance
  op: mult
  value: 2.0
- system: behavior
  target: migration_resistance
  op: set
  value: 0.0
- system: need
  target: competence
  op: set
  value:
    satisfaction_mult: 0.5
  condition:
    source: self
    key: same_job_years
    op: gt
    value: 1
- system: stress
  target: tradition_enforcement
  op: set
  value: 0.1
  condition:
    source: settlement
    key: values.TRADITION
    op: gt
    value: 0.7
- system: behavior
  target: explore_unknown
  op: inject
  value:
    priority: 0.7
  condition:
    source: self
    key: unknown_entity_nearby
    op: eq
    value: true
    note: also triggers on new_technology_available
```

### #12 A_root_bound
```yaml
- system: skill
  target: traditional_work
  op: mult
  value: 1.2
- system: values
  target: drift_rate
  op: mult
  value: 0.2
- system: skill
  target: new_skill_learning
  op: mult
  value: 0.5
- system: behavior
  target:
    - adopt_innovation
    - experiment
    - research
  op: block
  value: true
- system: behavior
  target: migration_resistance
  op: set
  value: 1.0
- system: stress
  target: displacement_stress
  op: set
  value: 0.2
  condition:
    source: self
    key: not_in_birth_settlement
    op: eq
    value: true
```

## Archetype: Dual-Axis (#13~#30)

### #13 A_silver_mask
```yaml
- system: relationship
  target: first_impression
  op: add
  value: 40
- system: relationship
  target: trust
  op: max
  value: 50
  condition:
    source: self
    key: relationship_duration
    op: gt
    value: 365
- system: behavior
  target: maintain_cover
  op: enable
  value: true
  tags:
    - espionage
- system: behavior
  target: fake_emotion
  op: enable
  value: true
- system: skill
  target:
    - espionage
    - diplomacy
    - deception
  op: mult
  value: 1.5
- system: behavior
  target:
    - express_true_feelings
    - confess
    - genuine_apology
  op: block
  value: true
```

### #14 A_true_mirror
```yaml
- system: derived
  target: charisma
  op: add
  value: 0.3
- system: behavior
  target:
    - lie
    - deceive
    - bluff
    - fake_emotion
  op: block
  value: true
- system: skill
  target:
    - espionage
    - deception
  op: mult
  value: 0.0
- system: relationship
  target: trust_gain_rate
  op: mult
  value: 2.0
- system: derived
  target: trustworthiness
  op: add
  value: 0.3
- system: behavior
  target: suspicion
  op: mult
  value: 0.3
```

### #15 A_phantom
```yaml
- system: reputation
  target: spread_speed
  op: mult
  value: 0.1
- system: skill
  target:
    - assassination
    - theft
    - infiltration
  op: mult
  value: 1.5
- system: relationship
  target: memory_decay_rate
  op: mult
  value: 3.0
- system: derived
  target: charisma
  op: add
  value: -0.5
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 0.2
- system: stress
  target: isolation_stress
  op: set
  value: 0.0
- system: need
  target: recognition
  op: set
  value:
    decay_rate_mult: 0.1
```

### #16 A_confessor
```yaml
- system: relationship
  target: secret_disclosure_chance
  op: mult
  value: 2.0
- system: behavior
  target: console_effectiveness
  op: mult
  value: 2.0
- system: aura
  target: stress_relief
  op: set
  value:
    radius: 1
    intensity: 0.1
    target_filter: interacting_with
- system: relationship
  target: max_active_relationships
  op: set
  value: 5
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
  target: trustworthiness
  op: add
  value: 0.35
```

### #17 A_tempest
```yaml
- system: derived
  target: intimidation
  op: add
  value: 0.4
- system: crafting
  target: quality_bonus
  op: add
  value: 1
  condition:
    source: self
    key: emotion.anger
    op: gt
    value: 0.5
    note: "also triggers on emotion.sadness > 0.5"
- system: behavior
  target: argument_style
  op: set
  value: explosive
- system: relationship
  target: conflict_damage
  op: mult
  value: 2.0
- system: emotion
  target: anger
  op: min
  value: 0.2
- system: behavior
  target: rational_decision
  op: mult
  value: 0.3
  condition:
    source: self
    key: emotion.anger
    op: gt
    value: 0.6
- system: behavior
  target: apologize
  op: set
  value:
    weight_mult: 0.1
```

### #18 A_still_water
```yaml
- system: skill
  target: mediation
  op: mult
  value: 1.5
- system: emotion
  target: anger
  op: max
  value: 0.15
  condition:
    source: self
    key: trigger_type
    op: eq
    value: insult
    note: also triggers on provocation
- system: aura
  target: anger
  op: set
  value:
    radius: 2
    intensity: -0.15
    target_filter: all
- system: behavior
  target: crisis_response_speed
  op: mult
  value: 0.6
- system: emotion
  target: intensity_mult
  op: set
  value: 0.5
- system: combat
  target: initial_panic_check
  op: override
  value: skip
```

### #19 A_war_drum
```yaml
- system: skill
  target:
    - agitation
    - propaganda
  op: mult
  value: 1.5
- system: aura
  target: combat_morale
  op: set
  value:
    radius: 5
    intensity: 0.2
    target_filter: allies
  condition:
    source: self
    key: in_combat
    op: eq
    value: true
- system: skill
  target:
    - peace_negotiation
    - diplomacy
  op: mult
  value: 0.6
- system: relationship
  target: hostility_escalation
  op: mult
  value: 2.0
- system: behavior
  target: pre_battle_speech
  op: enable
  value:
    morale_boost: 0.3
    duration: combat
- system: stress
  target: peace_boredom
  op: set
  value: 0.05
  condition:
    source: self
    key: no_conflict_days
    op: gt
    value: 30
```

### #20 A_hearth_keeper
```yaml
- system: aura
  target: joy
  op: set
  value:
    radius: 5
    intensity: 0.1
    target_filter: same_settlement
- system: aura
  target: conflict_suppression
  op: set
  value:
    radius: 3
    intensity: 0.3
- system: derived
  target: intimidation
  op: add
  value: -0.3
- system: behavior
  target: harsh_decision
  op: set
  value:
    weight_mult: 0.2
- system: relationship
  target: intimacy_decay_rate
  op: mult
  value: 0.5
- system: behavior
  target:
    - punish
    - enforce_rule
    - exile
  op: set
  value:
    weight_mult: 0.1
```

### #21 A_obsidian_edge
```yaml
- system: skill
  target:
    - tactics
    - strategy
    - logistics
  op: mult
  value: 1.4
- system: combat
  target: crit_chance
  op: add
  value: 0.15
- system: behavior
  target:
    - console
    - empathize
    - grant_leave
  op: block
  value: true
  condition:
    source: target
    key: role
    op: eq
    value: subordinate
- system: relationship
  target: subordinate_loyalty_decay
  op: mult
  value: 2.0
- system: behavior
  target: punish_failure
  op: inject
  value:
    priority: 0.9
  condition:
    source: self
    key: subordinate_failed_task
    op: eq
    value: true
- system: emotion
  target: expression_mult
  op: set
  value: 0.1
```

### #22 A_powder_keg
```yaml
- system: stress
  target: mental_break_threshold
  op: add
  value: -0.3
- system: stress
  target: break_types
  op: replace
  value:
    creative_frenzy: 0.3
    emotional_outburst: 0.3
    berserk: 0.2
    sobbing: 0.2
- system: crafting
  target: quality_bonus
  op: add
  value: 3
  condition:
    source: self
    key: mental_state
    op: eq
    value: creative_frenzy
- system: emotion
  target: volatility
  op: mult
  value: 3.0
- system: behavior
  target: action_delay
  op: set
  value: 0
- system: behavior
  target: consequence_evaluation
  op: mult
  value: 0.2
- system: behavior
  target: promise_fulfillment
  op: mult
  value: 0.4
```

### #23 A_living_library
```yaml
- system: skill
  target:
    - research
    - mathematics
    - astronomy
    - law
    - medicine
  op: mult
  value: 1.5
- system: event
  target: cross_discipline_discovery
  op: enable
  value: true
- system: behavior
  target: decision_speed
  op: mult
  value: 0.6
- system: memory
  target: knowledge_retention
  op: mult
  value: 2.0
- system: behavior
  target: plan_vs_act
  op: set
  value:
    plan_weight: 0.8
    act_weight: 0.2
- system: need
  target: competence
  op: set
  value:
    decay_rate_mult: 2.0
```

### #24 A_autumn_leaf
```yaml
- system: skill
  target:
    - painting
    - poetry
    - music
    - exploration
  op: mult
  value: 1.3
- system: stress
  target: settlement_boredom
  op: set
  value: 0.15
  condition:
    source: self
    key: same_settlement_years
    op: gt
    value: 3
- system: behavior
  target: wander
  op: inject
  value:
    priority: 0.6
  condition:
    source: self
    key: same_settlement_years
    op: gt
    value: 2
- system: need
  target: materialism
  op: set
  value:
    satisfaction_mult: 0.2
- system: emotion
  target: joy
  op: add
  value: 0.4
  condition:
    source: self
    key: arrived_new_settlement
    op: eq
    value: true
- system: relationship
  target: long_distance_decay
  op: mult
  value: 3.0
```

### #25 A_iron_oath
```yaml
- system: behavior
  target:
    - break_promise
    - renegotiate
    - abandon_task
  op: block
  value: true
- system: stress
  target: promise_failure
  op: set
  value: 0.6
  condition:
    source: self
    key: promise_broken_by_external
    op: eq
    value: true
- system: stress
  target: break_types
  op: replace
  value:
    self_punishment: 0.4
    rage: 0.3
    catatonic: 0.3
  condition:
    source: self
    key: trigger
    op: eq
    value: promise_failure
- system: relationship
  target: subordinate_loyalty
  op: add
  value: 0.3
- system: relationship
  target: oathbreaker_response
  op: set
  value: permanent_hostility
- system: values
  target: LAW
  op: min
  value: 0.7
```

### #26 A_quicksilver
```yaml
- system: behavior
  target:
    - promise_fulfillment
    - rule_compliance
    - duty_compliance
  op: mult
  value: 0.0
- system: behavior
  target: faction_loyalty
  op: set
  value: 0.0
- system: behavior
  target: switch_faction
  op: set
  value:
    weight_mult: 5.0
- system: behavior
  target: faction_entry_resistance
  op: set
  value: 0.0
- system: stress
  target: betrayal_guilt
  op: immune
  value: true
- system: relationship
  target: trust
  op: max
  value: 40
- system: behavior
  target: danger_detection
  op: mult
  value: 1.5
```

### #27 A_dreamer
```yaml
- system: skill
  target:
    - poetry
    - painting
    - music
    - theology
    - mythology
  op: mult
  value: 1.5
- system: skill
  target:
    - farming
    - construction
    - mining
    - logging
  op: mult
  value: 0.7
- system: behavior
  target: danger_detection
  op: mult
  value: 0.6
- system: event
  target: dream_inspiration
  op: enable
  value:
    chance_per_night: 0.05
- system: memory
  target: distortion_rate
  op: mult
  value: 2.0
- system: need
  target: transcendence
  op: set
  value:
    decay_rate_mult: 2.0
```

### #28 A_fortress_mind
```yaml
- system: stress
  target:
    - propaganda_resistance
    - brainwash_resistance
  op: set
  value: 1.0
- system: values
  target: drift_rate
  op: set
  value: 0.0
- system: behavior
  target:
    - adopt_innovation
    - learn_new_skill
    - accept_change
    - experiment
  op: block
  value: true
- system: skill
  target: existing_skills
  op: mult
  value: 1.2
- system: relationship
  target: stranger_trust
  op: set
  value: -20
- system: emotion
  target: volatility
  op: mult
  value: 0.2
```

### #29 A_zealot
```yaml
- system: skill
  target:
    - proselytize
    - persuasion
  op: mult
  value: 1.4
- system: behavior
  target: persecute_heretic
  op: inject
  value:
    priority: 0.8
  condition:
    source: target
    key: values_diff
    op: gt
    value: 0.5
- system: behavior
  target:
    - compromise
    - concede
    - tolerance
  op: block
  value: true
- system: stress
  target: source_immunity
  op: set
  value:
    - doubt
    - heresy_exposure
- system: relationship
  target: value_conflict_damage
  op: mult
  value: 3.0
- system: relationship
  target: value_alignment_bonus
  op: mult
  value: 2.0
- system: event
  target: religious_conflict_chance
  op: mult
  value: 2.0
```

### #30 A_fox_path
```yaml
- system: relationship
  target: first_impression
  op: add
  value: 30
- system: derived
  target: charisma
  op: add
  value: 0.2
- system: skill
  target:
    - espionage
    - conspiracy
    - manipulation
  op: mult
  value: 1.4
- system: behavior
  target: maintain_cover
  op: enable
  value: true
- system: relationship
  target: betrayal_damage
  op: mult
  value: 3.0
- system: behavior
  target:
    - express_true_feelings
    - genuine_apology
  op: block
  value: true
- system: derived
  target: wisdom
  op: add
  value: -0.2
```

## Shadow (#56~#60)

### #56 S_hollow_crown
```yaml
- system: emotion
  target: guilt
  op: set
  value: 0.0
- system: emotion
  target:
    - trust
    - sadness
  op: max
  value: 0.15
- system: emotion
  target: contempt
  op: min
  value: 0.2
- system: stress
  target: accumulation_rate
  op: mult
  value: 0.3
- system: stress
  target: source_immunity
  op: set
  value:
    - guilt
    - grief
    - social_rejection
    - loneliness
- system: relationship
  target: mode
  op: set
  value: instrumental
- system: relationship
  target: betrayal_cooldown
  op: set
  value: 0
- system: relationship
  target: betrayal_stress
  op: set
  value: 0.0
- system: memory
  target: kill_trauma
  op: set
  value: false
- system: combat
  target: kill_stress
  op: set
  value: 0.0
- system: behavior
  target:
    - manipulate
    - exploit
    - charm_offensive
    - cold_calculation
  op: enable
  value: true
- system: behavior
  target:
    - genuine_empathy
    - altruistic_help
    - self_sacrifice
    - console
  op: block
  value: true
- system: derived
  target: charisma
  op: add
  value: 0.2
- system: derived
  target: trustworthiness
  op: add
  value: -0.3
  tags:
    - long_term_decay
```

### #57 S_puppet_master
```yaml
- system: skill
  target:
    - conspiracy
    - manipulation
    - espionage
  op: mult
  value: 1.6
- system: behavior
  target: recruit_minion
  op: enable
  value:
    loyalty_bonus: 0.4
    method: calculated_charm
- system: relationship
  target: mode
  op: set
  value: strategic
- system: behavior
  target: plausible_deniability
  op: enable
  value:
    detection_reduction: 0.8
- system: behavior
  target: delegate_dirty_work
  op: inject
  value:
    priority: 0.9
- system: behavior
  target: planning_horizon
  op: mult
  value: 3.0
- system: behavior
  target: improvisation
  op: mult
  value: 0.5
```

### #58 S_mirror_throne
```yaml
- system: skill
  target:
    - propaganda
    - agitation
    - leadership
  op: mult
  value: 1.5
- system: emotion
  target: anger
  op: on_event
  value:
    on: receives_criticism
    intensity: 0.8
- system: relationship
  target: critic_response
  op: set
  value: permanent_hostility
- system: need
  target: recognition
  op: set
  value:
    decay_rate_mult: 5.0
- system: stress
  target: break_types
  op: replace
  value:
    narcissistic_rage: 0.5
    grandiose_speech: 0.3
    self_destruction: 0.2
  condition:
    source: self
    key: need.recognition
    op: lt
    value: 0.2
- system: derived
  target: charisma
  op: add
  value: 0.3
  condition:
    source: self
    key: is_leader
    op: eq
    value: true
- system: stress
  target: role_loss
  op: set
  value: 0.8
  condition:
    source: self
    key: lost_leadership
    op: eq
    value: true
```

### #59 S_cracked_mirror
```yaml
- system: behavior
  target: sympathy_manipulation
  op: enable
  value:
    effectiveness: 0.6
- system: need
  target: recognition
  op: set
  value:
    decay_rate_mult: 4.0
- system: emotion
  target: volatility
  op: mult
  value: 3.0
- system: behavior
  target: revenge
  op: inject
  value:
    priority: 1.0
  condition:
    source: target
    key: prior_intimacy
    op: gt
    value: 50
- system: behavior
  target: revenge_proportionality
  op: set
  value: 0.0
- system: stress
  target: coping
  op: set
  value:
    primary: self_pity
    secondary: blame_others
- system: stress
  target: isolation_stress
  op: mult
  value: 5.0
```

### #60 S_red_smile
```yaml
- system: emotion
  target: joy
  op: add
  value: 0.3
  condition:
    source: self
    key: witnessed_suffering
    op: eq
    value: true
- system: skill
  target:
    - interrogation
    - torture
  op: mult
  value: 1.6
- system: combat
  target: kill_morale_boost
  op: set
  value: 0.2
- system: aura
  target: fear
  op: set
  value:
    radius: 3
    intensity: 0.25
    target_filter: all
- system: skill
  target:
    - animal_training
    - veterinary
  op: mult
  value: 0.0
- system: behavior
  target:
    - mercy
    - spare_life
    - forgive
  op: block
  value: true
- system: stress
  target: violence_withdrawal
  op: set
  value: 0.1
  condition:
    source: self
    key: no_violence_days
    op: gt
    value: 7
```
