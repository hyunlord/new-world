# WorldSim Trait Effects — Part 3 (Shadow #6~#15, Radiance #1~#12)
> Canonical YAML effect definitions using Trait System v3 schema.
> Ops: set / add / mult / min / max / disable / enable / block / inject / override / on_event / tag / immune / replace
> All conditions use structured format: {source, key, op, value}
>
> Part 1: Archetype #1~#30 + Shadow #1~#5 ✅
> Part 2: Archetype #31~#55 ✅
> **Part 3: Shadow #6~#15 + Radiance #1~#12 ← this file**
> Part 4: Corpus + Nous
> Part 5: Awakened + Bloodline
> Part 6: Mastery + Bond
> Part 7: Fate + Synergy

---

## Shadow: Dark Triad/Tetrad Configurations (#6~#15)

### #6 S_honeyed_venom
> **Honeyed Venom / 꿀바른 독** — Warm face, poison heart — social predator who weaponizes affection
> Acquisition: H <= 0.10 AND X >= 0.83 AND A >= 0.83 | Rarity: legendary
> Academic: Christie & Geis (1970) Machiavellianism; Jones & Paulhus (2014) Dark Triad charm-exploitation cycle

```yaml
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 1.6
  tags:
    - the_honey
    - social_predator
- system: relationship
  target: mode
  op: set
  value: instrumental
  tags:
    - dark_triad
- system: skill
  target:
    - persuasion
    - manipulation
    - seduction
  op: mult
  value: 1.5
- system: behavior
  target: manipulate_target
  op: inject
  value:
    priority: 0.8
  condition:
    source: target
    key: trust_level
    op: gte
    value: 70
- system: stress
  target: source_immunity
  op: set
  value:
    - betrayal_guilt
    - exploitation_guilt
  tags:
    - guilt_absent
- system: emotion
  target: guilt
  op: max
  value: 0.05
- system: behavior
  target:
    - authentic_care
    - genuine_empathy
    - selfless_sacrifice
  op: block
  value: true
- system: stress
  target: reputation_collapse
  op: on_event
  value:
    on: caught_manipulating
    stress_spike: 0.7
    reputation_penalty: -0.5
  tags:
    - hidden_cost
    - exposure_risk
```

---

### #7 S_throne_hunger
> **Throne Hunger / 왕좌에 굶주린 자** — Power as oxygen — will destroy anything between them and dominance
> Acquisition: value.POWER >= 0.90 AND H <= 0.17 AND A <= 0.17 | Rarity: legendary
> Academic: McClelland (1975) need for power; Rosenthal & Pittinsky (2006) narcissistic leadership; Padilla et al. (2007) destructive leadership

```yaml
- system: need
  target: power
  op: set
  value:
    decay_rate_mult: 4.0
    deprivation_stress_mult: 3.0
  tags:
    - power_as_oxygen
- system: behavior
  target:
    - undermine_superior
    - political_maneuver
    - accumulate_leverage
    - seize_opportunity
  op: inject
  value:
    priority: 0.9
- system: behavior
  target:
    - support_rival
    - defer_to_equal
    - share_credit
    - accept_subordination
  op: block
  value: true
- system: derived
  target: intimidation
  op: add
  value: 0.25
- system: derived
  target: charisma
  op: add
  value: 0.2
  condition:
    source: self
    key: is_leader
    op: eq
    value: true
- system: stress
  target: leadership_loss
  op: on_event
  value:
    on: loses_leadership
    stress_spike: 0.85
    inject_behavior: destabilize_successor
  tags:
    - catastrophic_loss
- system: stress
  target: break_types
  op: replace
  value:
    tyrannical_rage: 0.4
    paranoid_purge: 0.35
    coup_attempt: 0.25
  condition:
    source: self
    key: stress_level
    op: gt
    value: 0.8
- system: emotion
  target: guilt
  op: max
  value: 0.05
  tags:
    - guilt_absent
- system: stress
  target: source_immunity
  op: set
  value:
    - harm_caused_to_others_guilt
    - subordinate_suffering
```

---

### #8 S_ash_prophet
> **Ash Prophet / 재의 예언자** — Charismatic doomsayer who genuinely believes their own catastrophism and spreads it compellingly
> Acquisition: O >= 0.83 AND E >= 0.83 AND H <= 0.17 AND X >= 0.83 | Rarity: legendary (4-condition ultra)
> Academic: Festinger (1957) cognitive dissonance; Weber (1947) charismatic authority; Cialdini (2001) social proof under fear; Janis (1972) groupthink

```yaml
- system: aura
  target: morale
  op: set
  value:
    radius: 5
    intensity: -0.15
    target_filter: all
  tags:
    - dread_field
    - settlement_debuff
- system: behavior
  target:
    - prophesy_doom
    - gather_followers
    - preach_catastrophe
  op: inject
  value:
    priority: 0.85
- system: skill
  target:
    - oration
    - propaganda
    - agitation
  op: mult
  value: 1.6
  tags:
    - warped_charisma
- system: behavior
  target: recruit_follower
  op: enable
  value:
    method: fear_appeal
    susceptibility_filter: emotionality_gte_0.7
  tags:
    - cult_mechanic
- system: stress
  target: source_immunity
  op: set
  value:
    - cognitive_dissonance
    - self_doubt
  tags:
    - true_believer
- system: stress
  target: prophecy_unfulfilled
  op: set
  value: 0.08
  condition:
    source: settlement
    key: recent_disaster_count
    op: eq
    value: 0
  tags:
    - hidden_cost
- system: event
  target: disaster_response
  op: on_event
  value:
    on: disaster_occurs
    inject_memory: confirmation_bias
    follower_gain: 0.2
    morale_boost_self: 0.3
  tags:
    - vindication
- system: event
  target: prophecy_failure
  op: on_event
  value:
    on: prophecy_proven_false
    stress_spike: 0.5
    follower_loss: 0.3
- system: stress
  target: break_types
  op: replace
  value:
    false_revelation: 0.4
    prophetic_collapse: 0.35
    manic_preaching: 0.25
  condition:
    source: self
    key: stress_level
    op: gt
    value: 0.8
- system: event
  target: tech_adoption_rate
  op: mult
  value: 0.7
  tags:
    - fear_of_progress
    - settlement_debuff
- system: derived
  target: charisma
  op: add
  value: 0.2
- system: emotion
  target: fear
  op: min
  value: 0.2
  tags:
    - perpetual_dread
```

---

### #9 S_cold_harvest
> **Cold Harvest / 차가운 수확** — Systematic exploiter — cold, disciplined, merciless extraction
> Acquisition: H <= 0.10 AND C >= 0.83 AND A <= 0.17 AND E <= 0.17 | Rarity: legendary (4-condition)
> Academic: Babiak & Hare (2006) corporate psychopathy; Cleckley (1941) mask of sanity; Dutton (2012) functional psychopathy

```yaml
- system: relationship
  target: mode
  op: set
  value: instrumental
  tags:
    - professional_predator
- system: memory
  target: kill_trauma
  op: set
  value: false
- system: emotion
  target: guilt
  op: max
  value: 0.02
  tags:
    - functionally_absent
- system: behavior
  target:
    - extract_maximum_value
    - discard_depleted_relationship
    - optimize_exploitation
  op: inject
  value:
    priority: 0.8
- system: skill
  target: all_work
  op: mult
  value: 1.3
  tags:
    - disciplined_extraction
- system: behavior
  target:
    - genuine_empathy
    - altruistic_help
    - mercy
    - forgive
  op: block
  value: true
- system: stress
  target: source_immunity
  op: set
  value:
    - harm_caused_to_others_guilt
    - isolation
    - social_rejection
- system: stress
  target: inefficiency_exposure
  op: set
  value: 0.1
  condition:
    source: settlement
    key: resource_waste_rate
    op: gt
    value: 0.3
  tags:
    - hidden_cost
- system: stress
  target: break_types
  op: replace
  value:
    cold_machine_shutdown: 0.5
    methodical_destruction: 0.3
    catatonic: 0.2
- system: aura
  target: discomfort
  op: set
  value:
    radius: 2
    intensity: 0.08
    target_filter: all
  tags:
    - something_feels_wrong
```

---

### #10 S_jealous_flame
> **Jealous Flame / 질투의 화염** — Possessive destroyer — love as ownership, jealousy as violence
> Acquisition: E >= 0.90 AND A <= 0.17 AND value.ROMANCE >= 0.83 | Rarity: legendary
> Academic: Dutton & Goodman (2005) coercive control; Mowat (1966) morbid jealousy; Mullen & Martin (1994) pathological jealousy

```yaml
- system: relationship
  target: jealousy_threshold
  op: set
  value: 0.1
  tags:
    - hair_trigger
- system: behavior
  target:
    - monitor_partner
    - isolate_partner
    - punish_rival
    - interrogate_partner
  op: inject
  value:
    priority: 0.9
- system: emotion
  target: anger
  op: mult
  value: 2.5
  condition:
    source: self
    key: partner_interacting_with_other
    op: eq
    value: true
- system: event
  target: partner_bond
  op: on_event
  value:
    on: partner_forms_new_bond
    rage_intensity: 0.8
    inject_behavior: confront_rival
- system: stress
  target: partner_autonomy
  op: set
  value: 0.15
  condition:
    source: self
    key: partner_acted_independently
    op: eq
    value: true
  tags:
    - possessive_stress
- system: stress
  target: source_immunity
  op: set
  value:
    - own_jealousy_shame
  tags:
    - feels_righteous
- system: relationship
  target: partner_intimacy_decay
  op: set
  value: 0.02
  tags:
    - isolation_destroys_bonds
    - hidden_cost
- system: stress
  target: break_types
  op: replace
  value:
    obsessive_surveillance: 0.4
    violent_confrontation: 0.35
    self_destructive_rage: 0.25
- system: behavior
  target:
    - trust_partner
    - grant_partner_freedom
    - accept_partner_friends
  op: block
  value: true
```

---

### #11 S_carrion_comfort
> **Carrion Comfort / 썩은 위안** — Passive parasite who feeds on others' sympathy, contributes nothing
> Acquisition: E <= 0.10 AND C <= 0.10 AND O <= 0.17 | Rarity: legendary
> Academic: Millon (1981) dependent personality; Kantor (2002) passive aggression; Seligman (1975) learned helplessness as strategy

```yaml
- system: behavior
  target:
    - seek_sympathy
    - exaggerate_suffering
    - avoid_reciprocity
    - feign_helplessness
  op: inject
  value:
    priority: 0.7
- system: behavior
  target:
    - contribute_to_group
    - reciprocate_help
    - initiate_work
    - volunteer
  op: block
  value: true
- system: skill
  target: all_work
  op: mult
  value: 0.3
  tags:
    - extreme_debuff
    - passive_drain
- system: aura
  target: energy_drain
  op: set
  value:
    radius: 3
    intensity: -0.1
    target_filter: all
    affect: need.belonging
  tags:
    - emotional_parasite
- system: stress
  target: source_immunity
  op: set
  value:
    - boredom
    - guilt
    - shame
- system: event
  target: abandonment_response
  op: on_event
  value:
    on: others_stop_helping
    inject_behavior: escalate_suffering_display
    manipulation_intensity: 0.6
- system: relationship
  target: trust
  op: add
  value: -0.01
  tags:
    - slow_passive_decay
    - hidden_cost
- system: stress
  target: break_types
  op: replace
  value:
    hollow_collapse: 0.5
    desperate_cling: 0.3
    catatonic: 0.2
- system: derived
  target: trustworthiness
  op: add
  value: -0.2
  tags:
    - long_term_decay
```

---

### #12 S_web_weaver
> **Web-Weaver / 거미줄 직조자** — Master social architect who builds networks to own them
> Acquisition: H <= 0.17 AND C >= 0.83 AND X >= 0.83 AND A >= 0.83 | Rarity: legendary (4-condition near-ultra)
> Academic: Burt (2005) structural holes; Brass et al. (1998) network power; Kilduff & Tsai (2003) social network manipulation

```yaml
- system: behavior
  target:
    - cultivate_useful_relationship
    - create_obligation
    - broker_information
    - build_network_node
  op: inject
  value:
    priority: 0.8
- system: skill
  target:
    - negotiation
    - espionage
    - administration
  op: mult
  value: 1.5
- system: relationship
  target: trust_gain_rate
  op: mult
  value: 1.4
  tags:
    - appears_trustworthy
    - web_strength
- system: relationship
  target: mode
  op: set
  value: strategic
- system: event
  target: deep_trust_leverage
  op: on_event
  value:
    on: relationship_reaches_deep_trust
    enable_behavior: leverage_relationship
    threshold: 75
  tags:
    - exploitation_gate
- system: behavior
  target: information_network
  op: enable
  value:
    detection_range_mult: 2.0
    intel_quality_mult: 1.5
  tags:
    - network_intelligence
- system: stress
  target: source_immunity
  op: set
  value:
    - guilt_from_using_people
    - betrayal_guilt
- system: event
  target: network_exposure
  op: on_event
  value:
    on: network_member_discovers_manipulation
    cascade_trust_collapse: true
    reputation_penalty: -0.6
  tags:
    - hidden_cost
    - cascade_risk
- system: behavior
  target:
    - genuine_vulnerability
    - selfless_act
    - unconditional_help
  op: block
  value: true
- system: derived
  target: charisma
  op: add
  value: 0.15
  tags:
    - social_capital
```

---

### #13 S_night_bloom
> **Night Bloom / 야화** — Charming void — magnetic, warm-appearing, internally hollow
> Acquisition: X >= 0.83 AND E <= 0.17 AND H <= 0.17 AND A >= 0.83 | Rarity: legendary (4-condition)
> Academic: Cleckley (1941) mask of sanity; Hare (2003) superficial charm; Kernberg (1975) narcissistic emptiness

```yaml
- system: derived
  target: allure
  op: add
  value: 0.3
  tags:
    - magnetic_presence
- system: relationship
  target: first_impression
  op: mult
  value: 2.0
  tags:
    - extraordinary_first_contact
- system: relationship
  target: intimacy
  op: max
  value: 55
  tags:
    - hollow_exposed_at_depth
- system: behavior
  target:
    - authentic_vulnerability
    - deep_disclosure
    - genuine_emotional_sharing
  op: block
  value: true
- system: stress
  target: source_immunity
  op: set
  value:
    - loneliness
    - emotional_emptiness
  tags:
    - doesnt_feel_the_void
- system: event
  target: intimacy_attempt
  op: on_event
  value:
    on: relationship_attempts_deep_intimacy
    inject_behavior: withdraw
    cooldown_ticks: 50
- system: aura
  target: charm_then_unease
  op: set
  value:
    radius: 3
    initial_intensity: 0.15
    decay_to: -0.05
    decay_over_ticks: 200
  tags:
    - warmth_then_emptiness
- system: emotion
  target:
    - trust
    - sadness
    - joy
  op: max
  value: 0.3
  tags:
    - emotional_ceiling
- system: derived
  target: charisma
  op: add
  value: 0.2
```

---

### #14 S_broken_compass
> **Broken Compass / 부러진 나침반** — Chaos incarnate — no moral anchor, no consistency, no plan
> Acquisition: H <= 0.10 AND C <= 0.10 | Rarity: legendary
> Academic: Lykken (1995) fearlessness model; Patrick et al. (2009) triarchic psychopathy (disinhibition); Eysenck (1977) psychoticism dimension

```yaml
- system: behavior
  target: random_chaotic_action
  op: inject
  value:
    priority: 0.6
    source: full_action_pool
    selection: probability_weighted_random
  tags:
    - chaos_incarnate
- system: behavior
  target:
    - keep_promise
    - maintain_plan
    - follow_through
    - honor_agreement
  op: block
  value: true
- system: relationship
  target: trust
  op: add
  value: -0.02
  tags:
    - passive_trust_decay
    - unpredictability_is_dangerous
- system: stress
  target: source_immunity
  op: set
  value:
    - guilt
    - shame
    - regret
    - obligation
  tags:
    - no_moral_self_reference
- system: event
  target: commitment_break
  op: on_event
  value:
    on: commitment_made
    break_chance: 0.6
    check_interval_ticks: 10
- system: skill
  target: role_satisfaction
  op: set
  value:
    decay_rate: 0.1
    floor: 0.0
  tags:
    - cannot_hold_role
- system: stress
  target: break_types
  op: replace
  value:
    null_event: 0.5
    random_destruction: 0.3
    manic_episode: 0.2
  tags:
    - doesnt_break_just_stops
- system: emotion
  target: volatility
  op: mult
  value: 3.0
- system: behavior
  target: planning_horizon
  op: mult
  value: 0.1
  tags:
    - no_long_term_thought
```

---

### #15 S_iron_cradle
> **Iron Cradle / 철의 요람** — Suffocating parent/leader who controls everything "for your own good"
> Acquisition: C >= 0.90 AND A <= 0.10 AND E <= 0.17 | Rarity: legendary
> Academic: Baumrind (1966) authoritarian parenting; Bartholomew (1990) dismissive attachment; Schaubroeck et al. (2007) authoritarian leadership

```yaml
- system: behavior
  target:
    - enforce_compliance
    - correct_deviation
    - punish_independence
    - micromanage
  op: inject
  value:
    priority: 0.85
- system: aura
  target: autonomy_frustration
  op: set
  value:
    radius: 4
    intensity: 0.12
    target_filter: subordinates
    affect: need.autonomy
  tags:
    - suffocating_control
- system: stress
  target: others_mistakes
  op: set
  value: 0.12
  condition:
    source: self
    key: witnessed_incompetence
    op: eq
    value: true
  tags:
    - cannot_tolerate_error
- system: behavior
  target:
    - allow_failure
    - grant_autonomy
    - accept_others_choices
    - delegate_authority
  op: block
  value: true
- system: derived
  target: intimidation
  op: add
  value: 0.2
- system: skill
  target: administration
  op: mult
  value: 1.4
- system: event
  target: subordinate_defiance
  op: on_event
  value:
    on: subordinate_defies
    stress_spike: 0.5
    inject_behavior: punish_defiant
- system: stress
  target: break_types
  op: replace
  value:
    control_collapse: 0.5
    authoritarian_rage: 0.3
    obsessive_micromanagement: 0.2
  condition:
    source: self
    key: stress_level
    op: gt
    value: 0.8
- system: relationship
  target: subordinate_trust_decay
  op: set
  value: 0.015
  tags:
    - dependents_flee_or_break
    - hidden_cost
```

---

## Radiance: Light Triad Configurations (#1~#12)

### #1 R_golden_heart
> **Golden Heart / 금빛 심장** — Pure empathic honesty — feels everything, tells the truth, gives without calculation
> Acquisition: H >= 0.90 AND A >= 0.90 AND E >= 0.83 | Rarity: legendary
> Academic: Kaufman et al. (2019) Light Triad; Batson (2011) empathy-altruism hypothesis; Davis (1983) empathic concern

```yaml
- system: behavior
  target:
    - comfort_suffering
    - advocate_for_weak
    - give_without_asked
    - protect_vulnerable
  op: inject
  value:
    priority: 0.8
- system: derived
  target: trustworthiness
  op: add
  value: 0.3
  tags:
    - maximum_trust
- system: aura
  target: morale
  op: set
  value:
    radius: 4
    intensity: 0.08
    target_filter: all
  tags:
    - settlement_warmth
- system: stress
  target: witnessed_suffering
  op: set
  value: 0.12
  condition:
    source: self
    key: nearby_agent_distressed
    op: eq
    value: true
  tags:
    - empathic_cost
- system: behavior
  target:
    - deceive
    - manipulate
    - use_person_as_means
    - exploit
  op: block
  value: true
  tags:
    - kantianism
- system: event
  target: betrayal_response
  op: on_event
  value:
    on: betrayed_by_trusted
    grief_intensity: 0.8
    inject_behavior: forgive_betrayer
  tags:
    - forgives_anyway
- system: relationship
  target: intimacy
  op: max
  value: 100
  tags:
    - no_ceiling
- system: relationship
  target: manipulation_vulnerability
  op: set
  value:
    shadow_bonus: 0.25
  tags:
    - shadow_exploitation_target
    - hidden_cost
- system: emotion
  target: guilt
  op: min
  value: 0.3
  tags:
    - feels_deeply
```

---

### #2 R_north_star
> **North Star / 북극성** — Principled beacon — people orient themselves around this person's integrity
> Acquisition: H >= 0.90 AND C >= 0.90 AND X >= 0.83 | Rarity: legendary
> Academic: Kaufman et al. (2019) Light Triad Kantianism; Hannah et al. (2011) moral courage; Brown & Treviño (2006) ethical leadership

```yaml
- system: derived
  target: trustworthiness
  op: add
  value: 0.3
- system: derived
  target: charisma
  op: add
  value: 0.15
  tags:
    - integrity_charisma
- system: aura
  target: integrity_contagion
  op: set
  value:
    radius: 5
    intensity: 0.02
    target_filter: all
    affect: hexaco.H
  tags:
    - integrity_is_contagious
    - settlement_scale
- system: behavior
  target:
    - compromise_values
    - accept_corruption
    - look_away_from_injustice
    - cover_up_wrongdoing
  op: block
  value: true
- system: event
  target: public_integrity
  op: on_event
  value:
    on: acts_with_integrity_publicly
    settlement_value_boost:
      - TRADITION
      - LAW
    boost_amount: 0.02
- system: stress
  target: forced_compromise
  op: set
  value: 0.2
  condition:
    source: self
    key: forced_to_compromise_values
    op: eq
    value: true
  tags:
    - devastating_cost
- system: reputation
  target: tags
  op: tag
  value: beacon
- system: skill
  target: leadership
  op: mult
  value: 1.4
  condition:
    source: self
    key: is_leader
    op: eq
    value: true
- system: behavior
  target:
    - uphold_law
    - call_out_injustice
    - defend_principle
  op: inject
  value:
    priority: 0.7
```

---

### #3 R_unbreaking
> **Unbroken One / 꺾이지 않는 자** — Immovable kindness — trauma-resistant goodness, bends without breaking
> Acquisition: E <= 0.17 AND C >= 0.83 AND A >= 0.83 AND H >= 0.83 | Rarity: legendary (4-condition)
> Academic: Bonanno (2004) resilience; Masten (2001) ordinary magic; Fredrickson (2004) broaden-and-build; Kaufman et al. (2019) Light Triad

```yaml
- system: stress
  target: source_immunity
  op: set
  value:
    - grief_spiral
    - trauma_escalation
    - panic_contagion
  tags:
    - feels_but_doesnt_collapse
- system: derived
  target: wisdom
  op: add
  value: 0.25
  tags:
    - experience_without_bitterness
- system: aura
  target: stress_recovery
  op: set
  value:
    radius: 4
    intensity: 0.1
    target_filter: all
    affect: stress.recovery_rate
  tags:
    - steadiness_stabilizes
    - settlement_scale
- system: behavior
  target:
    - hold_space_for_grief
    - be_steady_for_others
    - anchor_in_crisis
  op: inject
  value:
    priority: 0.75
- system: stress
  target: slow_accumulation
  op: set
  value: 0.03
  tags:
    - absorbs_others_pain
    - hidden_cost
- system: event
  target: ally_crisis
  op: on_event
  value:
    on: ally_in_crisis
    inject_behavior: anchor_for_ally
    stress_absorption: 0.15
- system: reputation
  target: tags
  op: tag
  value: unshakeable
- system: combat
  target: panic
  op: immune
  value: true
  tags:
    - cannot_be_routed
- system: stress
  target: break_types
  op: replace
  value:
    silent_burden: 0.5
    quiet_withdrawal: 0.3
    compassion_fatigue: 0.2
  tags:
    - different_breaking
```

---

### #4 R_world_bridge
> **Bridge of Worlds / 세계의 가교** — Cultural reconciler who dissolves barriers between groups
> Acquisition: O >= 0.90 AND A >= 0.90 AND X >= 0.83 | Rarity: legendary
> Academic: Berry (2005) acculturation; Tadmor et al. (2012) multicultural identity; Putnam (2000) bridging social capital

```yaml
- system: skill
  target:
    - diplomacy
    - translation
    - negotiation
    - mediation
  op: mult
  value: 1.5
- system: aura
  target: cultural_diffusion
  op: set
  value:
    radius: 6
    intensity: 0.1
    target_filter: all
    affect: cultural_barrier_reduction
  tags:
    - bridges_groups
    - settlement_scale
- system: behavior
  target:
    - find_common_ground
    - mediate_conflict
    - translate_intentions
  op: inject
  value:
    priority: 0.75
- system: event
  target: faction_mediation
  op: on_event
  value:
    on: two_hostile_factions_present
    inject_behavior: broker_peace
    success_mult: 1.5
- system: derived
  target: charisma
  op: add
  value: 0.2
  condition:
    source: self
    key: cross_cultural_context
    op: eq
    value: true
- system: stress
  target: intractable_conflict
  op: set
  value: 0.15
  condition:
    source: self
    key: mediation_failed
    op: eq
    value: true
  tags:
    - personal_cost
- system: relationship
  target: language_barrier_penalty
  op: mult
  value: 0.3
  tags:
    - barrier_reduction
- system: event
  target: foreign_relations
  op: mult
  value: 1.4
  tags:
    - settlement_scale
- system: behavior
  target:
    - discriminate
    - exclude_outsider
    - dehumanize_foreigner
  op: block
  value: true
  tags:
    - humanism
```

---

### #5 R_seed_keeper
> **Seed-Keeper / 씨앗 지기** — Civilization's memory — preserves and transmits knowledge across generations
> Acquisition: C >= 0.83 AND O >= 0.83 AND value.KNOWLEDGE >= 0.90 | Rarity: legendary
> Academic: Mokyr (2002) gifts of Athena; Henrich (2015) cumulative culture; Tomasello (1999) cultural learning

```yaml
- system: skill
  target:
    - teaching
    - writing
    - research
  op: mult
  value: 1.5
- system: event
  target: tech_documentation
  op: on_event
  value:
    on: tech_discovered
    inject_behavior: document_for_posterity
    knowledge_preservation_bonus: 0.3
- system: event
  target: settlement_destruction
  op: on_event
  value:
    on: settlement_destroyed
    knowledge_preserved: true
    tech_loss_reduction: 0.5
  tags:
    - records_survive
- system: aura
  target: learning_rate
  op: set
  value:
    radius: 6
    intensity: 0.15
    target_filter: all
    affect: skill.learning_rate
  tags:
    - living_library
    - settlement_scale
- system: event
  target: death_legacy
  op: on_event
  value:
    on: death
    permanent_bonus: true
    settlement_knowledge_mult: 1.05
  tags:
    - permanent_legacy
    - civilization_scale
- system: stress
  target: knowledge_loss
  op: set
  value: 0.25
  condition:
    source: settlement
    key: records_destroyed
    op: eq
    value: true
  tags:
    - devastating_cost
- system: behavior
  target:
    - hoard_knowledge
    - destroy_records
    - censor_information
  op: block
  value: true
- system: derived
  target: wisdom
  op: add
  value: 0.2
- system: behavior
  target:
    - teach_others
    - preserve_records
    - catalog_knowledge
  op: inject
  value:
    priority: 0.7
```

---

### #6 R_hearthfire
> **Hearthfire / 난롯불** — Community anchor — everyone gathers around this warmth
> Acquisition: A >= 0.90 AND X >= 0.83 AND E >= 0.83 | Rarity: legendary
> Academic: Baumeister & Leary (1995) need to belong; McMillan & Chavis (1986) sense of community; Putnam (2000) social capital

```yaml
- system: aura
  target: belonging_fulfillment
  op: set
  value:
    radius: 5
    intensity: 0.12
    target_filter: all
    affect: need.belonging
  tags:
    - community_warmth
    - settlement_scale
- system: behavior
  target:
    - welcome_stranger
    - organize_gathering
    - share_food
    - include_outsider
  op: inject
  value:
    priority: 0.75
- system: aura
  target: faction_cohesion
  op: set
  value:
    radius: 6
    intensity: -0.1
    target_filter: all
    affect: faction_conflict_probability
  tags:
    - settlement_cohesion
- system: need
  target: belonging
  op: set
  value:
    sensitivity_mult: 2.0
    deprivation_stress_mult: 3.0
  tags:
    - needs_community_too
    - hidden_cost
- system: event
  target: community_crisis
  op: on_event
  value:
    on: community_in_crisis
    inject_behavior: rally_community
    morale_boost: 0.15
- system: derived
  target: charisma
  op: add
  value: 0.15
  tags:
    - popularity
- system: behavior
  target:
    - exclude
    - reject_newcomer
    - ostracize
  op: block
  value: true
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 1.3
  tags:
    - makes_friends_easily
```

---

### #7 R_calm_harbor
> **Calm Harbor / 고요한 항구** — Still center — absorbs others' chaos without being moved
> Acquisition: E <= 0.17 AND A >= 0.83 AND X <= 0.17 AND H >= 0.83 | Rarity: legendary (4-condition)
> Academic: Gross (2002) emotion regulation; Neff (2003) self-compassion; Rogers (1951) unconditional positive regard

```yaml
- system: aura
  target: calming_field
  op: set
  value:
    radius: 4
    intensity: -0.12
    target_filter: all
    affect:
      - emotion.anxiety
      - emotion.fear
  tags:
    - calming_presence
    - settlement_scale
- system: behavior
  target:
    - de_escalate
    - absorb_anger
    - hold_space
    - listen_without_judgment
  op: inject
  value:
    priority: 0.7
- system: derived
  target: trustworthiness
  op: add
  value: 0.25
- system: stress
  target: source_immunity
  op: set
  value:
    - others_anger_directed_at_self
    - social_conflict_exposure
  tags:
    - unshakeable_calm
- system: stress
  target: slow_accumulation_from_holding
  op: set
  value: 0.04
  tags:
    - gives_endlessly
    - pays_quietly
    - hidden_cost
- system: event
  target: nearby_conflict
  op: on_event
  value:
    on: conflict_in_nearby_agents
    inject_behavior: mediate
    de_escalation_mult: 1.5
- system: relationship
  target: sought_during_stress
  op: set
  value:
    passive_relationship_gain: 0.01
  tags:
    - others_seek_them_out
- system: emotion
  target: anger
  op: max
  value: 0.15
  tags:
    - personal_calm
```

---

### #8 R_dawn_singer
> **Dawn Singer / 새벽을 노래하는 자** — Creative healer whose art transforms collective suffering into meaning
> Acquisition: O >= 0.90 AND E >= 0.83 AND skill.artwork >= 0.90 | Rarity: legendary
> Academic: Pennebaker (1997) expressive writing; Hass-Cohen & Carr (2008) art therapy; Csikszentmihalyi (1996) creativity and flow

```yaml
- system: aura
  target: morale_recovery
  op: set
  value:
    radius: 6
    intensity: 0.1
    target_filter: all
    affect: stress.recovery_rate
  tags:
    - art_heals_community
    - settlement_scale
- system: skill
  target:
    - art
    - music
    - performance
    - poetry
  op: mult
  value: 1.7
  tags:
    - extraordinary_talent
- system: event
  target: community_trauma
  op: on_event
  value:
    on: community_trauma
    inject_behavior: create_memorial_art
    healing_mult: 1.4
- system: event
  target: masterwork
  op: on_event
  value:
    on: masterwork_created
    settlement_morale_event: true
    morale_boost: 0.2
  tags:
    - settlement_scale
- system: stress
  target: creative_block
  op: set
  value: 0.15
  condition:
    source: self
    key: unable_to_create
    op: eq
    value: true
  tags:
    - devastating_cost
- system: event
  target: tragedy_creation
  op: on_event
  value:
    on: great_tragedy_witnessed
    suffering_intensity: 0.5
    masterwork_chance_mult: 2.0
  tags:
    - greatest_works_from_greatest_pain
- system: derived
  target: creativity
  op: add
  value: 0.35
  tags:
    - highest_of_all_traits
- system: reputation
  target: tags
  op: tag
  value: voice_of_the_people
- system: behavior
  target:
    - create_art
    - perform
    - inspire_through_beauty
  op: inject
  value:
    priority: 0.7
```

---

### #9 R_iron_promise
> **Iron Promise / 철의 약속** — Living oath — their word is absolute, inspires others to keep theirs
> Acquisition: H >= 0.90 AND value.LOYALTY >= 0.90 AND C >= 0.83 | Rarity: legendary
> Academic: Kant (1785) categorical imperative; Frank (1988) commitment device; Ostrom (1990) credible commitment

```yaml
- system: behavior
  target:
    - break_promise
    - renegotiate_commitment
    - betray_trusted
    - abandon_ally
  op: block
  value: true
  tags:
    - absolute_word
- system: event
  target: promise_lock
  op: on_event
  value:
    on: promise_made
    lock_behavior: must_fulfill
    override_priority: 0.95
- system: stress
  target: unfulfilled_promise
  op: set
  value: 0.2
  condition:
    source: self
    key: has_unfulfilled_promise
    op: eq
    value: true
  tags:
    - enormous_burden
- system: derived
  target: trustworthiness
  op: add
  value: 0.35
  tags:
    - maximum_trust
- system: aura
  target: commitment_contagion
  op: set
  value:
    radius: 4
    intensity: 0.06
    target_filter: all
    affect: promise_keeping_rate
  tags:
    - commitment_is_contagious
    - settlement_scale
- system: relationship
  target: exploitation_vulnerability
  op: set
  value:
    extracted_promise_compulsion: 0.8
  tags:
    - can_be_exploited
    - hidden_cost
- system: event
  target: forced_break
  op: on_event
  value:
    on: forced_to_break_promise
    allostatic_load_spike: 0.7
    stress_event: catastrophic
  tags:
    - soul_breaking
- system: reputation
  target: tags
  op: tag
  value: oath_keeper
- system: behavior
  target:
    - honor_commitment
    - fulfill_oath
    - keep_word
  op: inject
  value:
    priority: 0.85
```

---

### #10 R_gentle_thunder
> **Gentle Thunder / 온화한 천둥** — Quiet force — warm but immovable, changes things without conflict
> Acquisition: A >= 0.83 AND X >= 0.83 AND E <= 0.17 AND C >= 0.83 | Rarity: legendary (4-condition)
> Academic: Greenleaf (1977) servant leadership; Collins (2001) Level 5 leadership; Owens & Hekman (2012) humble leadership

```yaml
- system: derived
  target: intimidation
  op: add
  value: 0.15
  tags:
    - unexpected_for_agreeable
    - steadiness_is_power
- system: derived
  target: charisma
  op: add
  value: 0.2
- system: behavior
  target:
    - lead_by_example
    - set_standard_quietly
    - mentor_through_action
  op: inject
  value:
    priority: 0.7
- system: aura
  target: work_quality
  op: set
  value:
    radius: 4
    intensity: 0.1
    target_filter: all
    affect: skill.work_quality
  tags:
    - standards_elevation
    - settlement_scale
- system: behavior
  target:
    - back_down_from_values
    - compromise_quality
    - accept_mediocrity
  op: block
  value: true
- system: stress
  target: mediocrity_exposure
  op: set
  value: 0.08
  condition:
    source: self
    key: witnessed_mediocrity_persisting
    op: eq
    value: true
  tags:
    - hidden_cost
- system: event
  target: group_excellence
  op: on_event
  value:
    on: group_achieves_excellence
    joy_intensity: 0.6
    morale_boost_self: 0.3
- system: skill
  target: leadership
  op: mult
  value: 1.35
```

---

### #11 R_wellspring
> **Living Wellspring / 생명의 샘** — Emotional abundance — gives without depletion, others bloom nearby
> Acquisition: A >= 0.90 AND E >= 0.83 AND O >= 0.83 | Rarity: legendary
> Academic: Fredrickson (2001) broaden-and-build; Reis et al. (2000) intimacy process model; Ryan & Deci (2000) self-determination and relatedness

```yaml
- system: aura
  target: belonging_fulfillment
  op: set
  value:
    radius: 5
    intensity: 0.15
    target_filter: all
    affect: need.belonging
  tags:
    - most_powerful_in_game
    - settlement_scale
- system: behavior
  target:
    - give_emotional_support
    - nurture_growth
    - celebrate_others_success
  op: inject
  value:
    priority: 0.75
- system: relationship
  target: intimacy_gain_rate
  op: mult
  value: 1.5
  tags:
    - everyone_goes_deeper
- system: derived
  target: trustworthiness
  op: add
  value: 0.2
- system: derived
  target: charisma
  op: add
  value: 0.15
  tags:
    - popularity
- system: stress
  target: care_refused
  op: set
  value: 0.12
  condition:
    source: self
    key: offering_care_rejected
    op: eq
    value: true
  tags:
    - rejection_hurts_deeply
    - hidden_cost
- system: event
  target: community_loss
  op: on_event
  value:
    on: community_loss_event
    absorb_grief: true
    inject_behavior: comfort_community
    personal_stress: 0.15
- system: behavior
  target:
    - withhold_care_strategically
    - use_care_as_leverage
    - conditional_support
  op: block
  value: true
  tags:
    - cannot_help_instrumentally
- system: emotion
  target: joy
  op: min
  value: 0.2
  tags:
    - baseline_positivity
```

---

### #12 R_first_fire
> **First Fire / 최초의 불꽃** — Civilization spark — discovers, systematizes, shares freely
> Acquisition: O >= 0.90 AND C >= 0.83 AND H >= 0.83 | Rarity: legendary
> Academic: Mokyr (2002) gifts of Athena; Henrich (2015) cumulative culture; Merton (1973) communalism norm of science

```yaml
- system: event
  target: discovery_mult
  op: mult
  value: 1.6
  tags:
    - best_in_game
    - civilization_scale
- system: behavior
  target:
    - document_discovery
    - teach_finding
    - replicate_process
    - systematize_knowledge
  op: inject
  value:
    priority: 0.8
- system: behavior
  target:
    - hoard_knowledge
    - patent_discovery
    - restrict_access
  op: block
  value: true
  tags:
    - shares_freely
- system: event
  target: tech_sharing
  op: on_event
  value:
    on: tech_discovered
    mandatory_event: share_knowledge
    spread_rate_mult: 2.0
- system: event
  target: death_legacy
  op: on_event
  value:
    on: death
    permanent_bonus: true
    settlement_tech_research_speed_mult: 1.08
  tags:
    - permanent_legacy
    - civilization_scale
- system: aura
  target: research_rate
  op: set
  value:
    radius: 6
    intensity: 0.12
    target_filter: all
    affect: event.discovery_mult
  tags:
    - settlement_scale
- system: stress
  target: discovery_stolen
  op: set
  value: 0.15
  condition:
    source: self
    key: credit_stolen_for_discovery
    op: eq
    value: true
  tags:
    - personal_cost
- system: derived
  target: creativity
  op: add
  value: 0.25
- system: derived
  target: wisdom
  op: add
  value: 0.15
  condition:
    source: age
    key: years
    op: gte
    value: 40
  tags:
    - age_gated
```
