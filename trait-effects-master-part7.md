# WorldSim Trait Effects — Part 7 (Fate #1~#15, Synergy #1~#40)
> FINAL PART — all 242 traits complete after this file.
> Ops: set / add / mult / min / max / disable / enable / block / inject / override / on_event / tag / immune / replace
> All conditions: structured {source, key, op, value} only. No inline strings.
>
> Part 1~6: completed (187 traits)
> **Part 7: Fate #1~#15 + Synergy #1~#40 — this file**

---

## Fate Traits (#1~#15)

### #1 F_world_shaper
> World Shaper / 세계를 빚는 자 — Discovery as identity. Cannot stop reshaping civilization.

```yaml
- system: skill
  target: research_all
  op: mult
  value: 1.5
- system: derived
  target: wisdom
  op: mult
  value: 1.4
- system: derived
  target: creativity
  op: mult
  value: 1.4
- system: need
  target: knowledge
  op: mult
  value: 1.6
  note: sensitivity amplified
- system: behavior
  target: share_discovery
  op: inject
  value:
    priority: 0.7
- system: aura
  target: curiosity_contagion
  op: mult
  value: 1.3
  radius: 5
- on_event: tech_discovery
  effect: discovery_propagation_speed_doubled
- on_event: death
  effect: permanent_tech_acceleration_legacy
```

### #2 F_peoples_flame
> People's Flame / 민중의 불꽃 — Living symbol of liberation. Others follow even when not asked.

```yaml
- system: derived
  target: charisma
  op: mult
  value: 1.5
- system: derived
  target: popularity
  op: mult
  value: 1.4
- system: stress
  target: coercion_stress
  op: immune
  value: true
- system: behavior
  target: submit_to_unjust_authority
  op: block
  value: true
- system: behavior
  target: rally_others
  op: inject
  value:
    priority: 0.8
    condition:
      source: self
      key: witnessing_injustice
      op: eq
      value: true
- system: aura
  target: oppression_sensitivity
  op: mult
  value: 1.4
  radius: 5
- on_event: gives_speech
  effect: settlement_morale_surge
```

### #3 F_deathless_name
> Deathless Name / 불멸의 이름 — Known everywhere before meeting. The name precedes the person.

```yaml
- system: derived
  target: charisma
  op: mult
  value: 1.3
- system: derived
  target: trustworthiness
  op: mult
  value: 1.3
- system: reputation
  target: global_prestige
  op: set
  value: 0.8
- system: reputation
  target: tag
  op: tag
  value: deathless_name
- system: stress
  target: name_burden_stress
  op: set
  value: 0.1
- system: behavior
  target: act_unworthy_of_name
  op: block
  value: true
- system: aura
  target: stranger_recognition
  op: set
  radius: 8
  intensity: 0.3
- on_event: enters_new_settlement
  effect: reputation_precedes_arrival
- on_event: death
  effect: civilization_legend_permanent
```

### #4 F_doom_bringer
> Doom Bringer / 파멸의 전령 — Fear as fact. Not posture, actual documented destruction.

```yaml
- system: derived
  target: intimidation
  op: mult
  value: 1.6
- system: stress
  target: destruction_guilt
  op: immune
  value: true
- system: stress
  target: isolation_stress
  op: mult
  value: 1.4
- system: reputation
  target: tag
  op: tag
  value: doom_bringer
- system: combat
  target: enemy_morale_mult
  op: mult
  value: 0.6
- system: aura
  target: fear
  op: set
  radius: 6
  intensity: 0.35
- on_event: enters_enemy_settlement
  effect: settlement_fear_event
```

### #5 F_last_hope
> Last Hope / 마지막 희망 — Rallying point. Morale cannot collapse while they stand.

```yaml
- system: derived
  target: charisma
  op: mult
  value: 1.4
- system: derived
  target: trustworthiness
  op: mult
  value: 1.4
- system: need
  target: meaning
  op: mult
  value: 1.5
  note: fulfillment_rate
- system: stress
  target: others_despair_stress
  op: mult
  value: 1.6
- system: aura
  target: morale_floor
  op: min
  value: 0.3
  radius: 6
- on_event: settlement_crisis
  effect: inject_rally_settlement_override
- on_event: ally_about_to_break
  effect: inject_sustain_them
```

### #6 F_god_touched
> God-Touched / 신의 손길 — Triply marked. Purpose is undeniable, burden is crushing.

```yaml
- system: derived
  target: charisma
  op: mult
  value: 1.5
- system: reputation
  target: tag
  op: tag
  value: chosen_of_divine
- system: stress
  target: divine_burden_stress
  op: set
  value: 0.15
- system: need
  target: transcendence
  op: mult
  value: 2.0
  note: sensitivity
- system: behavior
  target: deny_purpose
  op: block
  value: true
- system: aura
  target: divine_weight
  op: set
  radius: 8
  intensity: 0.5
- on_event: major_decision
  effect: divine_guidance_modifier
```

### #7 F_curse_bearer
> Curse Bearer / 저주를 짊어진 자 — Suffering as teacher. Small hardships are invisible now.

```yaml
- system: derived
  target: wisdom
  op: mult
  value: 1.5
- system: stress
  target: accumulation_rate
  op: mult
  value: 1.4
- system: stress
  target: break_threshold
  op: mult
  value: 0.7
- system: stress
  target: mundane_hardship_stress
  op: immune
  value: true
- system: behavior
  target: recognize_others_pain
  op: inject
  value:
    priority: 0.7
- system: reputation
  target: tag
  op: tag
  value: marked_by_fate
- system: aura
  target: others_vulnerability_awareness
  op: mult
  value: 1.2
  radius: 4
- on_event: encounters_others_suffering
  effect: deep_recognition_event
```

### #8 F_bridge_of_ages
> Bridge of Ages / 세대의 다리 — Knowledge as legacy. The master who cannot stop giving it away.

```yaml
- system: derived
  target: wisdom
  op: mult
  value: 1.6
- system: skill
  target: teaching
  op: mult
  value: 1.6
- system: need
  target: meaning
  op: mult
  value: 1.6
  note: fulfillment_rate
- system: behavior
  target: hoard_knowledge
  op: block
  value: true
- system: aura
  target: learning_rate
  op: mult
  value: 1.4
  radius: 5
- on_event: teaches_student
  effect: student_learning_2x
- on_event: death
  effect: knowledge_tradition_continues
```

### #9 F_twin_crowned
> Twin-Crowned / 쌍관의 군주 — Divided duty. Cannot choose, cannot let either fall.

```yaml
- system: derived
  target: charisma
  op: mult
  value: 1.4
- system: derived
  target: wisdom
  op: mult
  value: 1.3
- system: skill
  target: administration
  op: mult
  value: 1.5
- system: stress
  target: dual_responsibility_stress
  op: set
  value: 0.15
- system: behavior
  target: abandon_one_settlement
  op: block
  value: true
- system: reputation
  target: tag
  op: tag
  value: twin_crowned
- on_event: conflict_between_two_settlements
  effect: unique_arbitration_access
```

### #10 F_seasons_child
> Season's Child / 계절의 아이 — Impossible reconciliation. Opposites in one body.

```yaml
- system: stress
  target: climate_stress
  op: immune
  value: true
- system: body
  target: cold_endurance
  op: mult
  value: 1.5
- system: body
  target: heat_endurance
  op: mult
  value: 1.5
- system: body
  target: energy_recovery_rate
  op: mult
  value: 1.3
- system: fertility
  target: fertility_rate
  op: mult
  value: 1.2
- system: derived
  target: wisdom
  op: mult
  value: 1.2
- system: aura
  target: climate_comfort
  op: mult
  value: 1.1
  radius: 3
- on_event: seasonal_crisis
  effect: environmental_resilience_bonus
```

### #11 F_ender_of_lines
> Ender of Lines / 혈통의 끝 — Targeted extinction. Not rage, systematic.

```yaml
- system: derived
  target: intimidation
  op: mult
  value: 1.5
- system: combat
  target: damage_mult
  op: mult
  value: 1.35
- system: stress
  target: violence_guilt
  op: immune
  value: true
- system: stress
  target: bloodline_obsession_stress
  op: set
  value: 0.12
- system: behavior
  target: target_bloodlines
  op: inject
  value:
    priority: 0.6
- system: reputation
  target: tag
  op: tag
  value: line_ender
- system: aura
  target: enemy_bloodlines_fear
  op: set
  radius: 5
  intensity: 0.4
- on_event: encounters_known_bloodline_enemy
  effect: combat_advantage_event
```

### #12 F_silent_founder
> Silent Founder / 침묵의 건설자 — Power refused. Shapes without holding.

```yaml
- system: derived
  target: wisdom
  op: mult
  value: 1.4
- system: derived
  target: trustworthiness
  op: mult
  value: 1.4
- system: behavior
  target: accept_formal_leadership
  op: block
  value: true
- system: behavior
  target: guide_from_shadow
  op: inject
  value:
    priority: 0.6
- system: need
  target: recognition
  op: mult
  value: 0.5
  note: sensitivity reduced — does not need credit
- system: aura
  target: civic_trust
  op: mult
  value: 1.3
  radius: 5
- on_event: settlement_crisis
  effect: inject_advise_not_lead
```

### #13 F_prophet_of_ruin
> Prophet of Ruin / 파멸의 예언자 — Proven prophet. People fear their words now.

```yaml
- system: derived
  target: wisdom
  op: mult
  value: 1.5
- system: stress
  target: prophetic_burden_stress
  op: set
  value: 0.15
- system: behavior
  target: withhold_prophecy
  op: block
  value: true
- system: reputation
  target: tag
  op: tag
  value: true_prophet
- system: aura
  target: dread
  op: set
  radius: 6
  intensity: 0.2
- on_event: makes_prophecy
  effect: higher_accuracy_chance
- on_event: false_prophecy
  effect: catastrophic_credibility_collapse
```

### #14 F_uncrowned_king
> Uncrowned King / 왕관 없는 왕 — The crown that cannot be forced on. People follow the title-less.

```yaml
- system: derived
  target: charisma
  op: mult
  value: 1.5
- system: derived
  target: popularity
  op: mult
  value: 1.5
- system: derived
  target: trustworthiness
  op: mult
  value: 1.3
- system: stress
  target: unwanted_authority_stress
  op: set
  value: 0.12
- system: behavior
  target: accept_formal_crown
  op: block
  value: true
- system: reputation
  target: tag
  op: tag
  value: uncrowned
- system: aura
  target: follower_loyalty
  op: mult
  value: 1.4
  radius: 6
- on_event: settlement_crisis
  effect: people_turn_to_them_regardless
```

### #15 F_memory_keeper
> Memory Keeper / 기억의 수호자 — Living archive. History does not die when they die.

```yaml
- system: derived
  target: wisdom
  op: mult
  value: 1.6
- system: skill
  target: writing
  op: mult
  value: 1.5
- system: skill
  target: teaching
  op: mult
  value: 1.4
- system: need
  target: meaning
  op: mult
  value: 1.6
  note: fulfillment_rate
- system: behavior
  target: let_history_be_forgotten
  op: block
  value: true
- system: behavior
  target: record_important_events
  op: inject
  value:
    priority: 0.9
- system: aura
  target: cultural_continuity
  op: mult
  value: 1.4
  radius: 5
- on_event: important_event
  effect: compelled_to_record
- on_event: death
  effect: permanent_chronicle_legacy
```

---

## Synergy Traits (#1~#40)

### #1 Y_frozen_fury
> Frozen Fury / 얼어붙은 격노 — Stone-cold suppression fused with battle instinct creates a controlled combat trance beyond rage.

```yaml
- system: combat
  target: cold_berserk
  op: inject
  value:
    trigger: stress_threshold_70
    effects:
      - attack_power_mult_1.8
      - pain_threshold_mult_3.0
      - emotion_volatility_zero
      - accuracy_add_15
  note: SIGNATURE — enters controlled combat trance with no emotional noise, retains tactical awareness unlike rage
- system: combat
  target: fear_effect
  op: immune
  value: true
  note: the frozen core does not flinch
- system: combat
  target: panic_effect
  op: immune
  value: true
- system: stress
  target: combat_stress_rate
  op: mult
  value: 0.5
  condition:
    source: context
    key: in_combat
    op: eq
    value: true
- system: combat
  target: damage_output
  op: mult
  value: 1.4
  condition:
    source: self
    key: health_pct
    op: lte
    value: 0.3
  note: wounded fury — as body breaks, frozen core burns colder
- system: need
  target: social
  op: mult
  value: 1.5
  note: sensitivity amplified — frozen state bleeds into peace
- on_event: enters_combat
  effect: emotional_suppression_total
```

### #2 Y_burning_glass
> Burning Glass / 타오르는 유리 — Fragile emotions shattered by inner storms converge into devastating clarity.

```yaml
- system: special
  target: emotional_detonation
  op: enable
  value: true
  note: SIGNATURE — at stress peak, all emotion collapses into single devastating action then resets
- system: stress
  target: emotional_intensity
  op: mult
  value: 2.0
  note: all emotions burn at double intensity
- system: derived
  target: creativity
  op: mult
  value: 1.5
  condition:
    source: self
    key: stress_level
    op: gte
    value: 40
- system: derived
  target: charisma
  op: mult
  value: 1.4
  note: raw emotional intensity becomes magnetic
- system: stress
  target: recovery_rate
  op: mult
  value: 0.4
  note: each crack in the glass takes time to seal
- system: body
  target: damage_taken
  op: mult
  value: 1.3
  note: glass heart means pain hits harder
- on_event: stress_exceeds_90
  effect: emotional_detonation_3x_action_then_reset
```

### #3 Y_iron_sun
> Iron Sun / 철의 태양 — Titanic physical power anchored to unshakable moral compass becomes immovable righteous force.

```yaml
- system: combat
  target: immovable_guardian
  op: inject
  value:
    trigger: ally_in_danger
    effects:
      - displacement_resistance_absolute
      - defense_mult_2.5
      - movement_zero
      - threat_draw_3x
  note: SIGNATURE — physically cannot be moved from protecting someone
- system: combat
  target: defense
  op: mult
  value: 1.5
- system: derived
  target: charisma
  op: mult
  value: 1.3
  note: moral weight made physical
- system: aura
  target: ally_morale
  op: mult
  value: 1.4
  radius: 5
  condition:
    source: context
    key: in_combat
    op: eq
    value: true
- system: stress
  target: injustice_witness_stress
  op: set
  value: 0.1
  note: cannot look away from injustice
- system: combat
  target: damage_output
  op: mult
  value: 1.5
  condition:
    source: context
    key: defending_innocent
    op: eq
    value: true
- on_event: ally_takes_lethal_damage_nearby
  effect: immovable_guardian_auto_activate
```

### #4 Y_velvet_knife
> Velvet Knife / 비단 칼날 — Honeyed venom and mastery of language combine into words that cut deeper than any blade.

```yaml
- system: special
  target: social_assassination
  op: inject
  value:
    priority: 0.8
    cooldown: 500
  note: SIGNATURE — destroys target reputation and relationships through conversation alone
- system: skill
  target: persuasion
  op: mult
  value: 1.8
- system: skill
  target: deception
  op: mult
  value: 1.7
- system: derived
  target: empathy
  op: mult
  value: 0.3
  note: understanding others became a tool, not a feeling
- system: stress
  target: mask_fatigue_stress
  op: set
  value: 0.08
  note: constant performance grinds the mind
- system: skill
  target: information_extraction
  op: mult
  value: 2.0
  note: others reveal more than they intend
- on_event: extended_conversation
  effect: social_assassination_opportunity
```

### #5 Y_storm_crown
> Storm Crown / 폭풍의 왕관 — Emotional tempest channeled through command creates authority that bends will like wind bends trees.

```yaml
- system: special
  target: tempest_authority
  op: enable
  value: true
  note: SIGNATURE — decisions made in emotional storm carry overwhelming compliance
- system: derived
  target: charisma
  op: mult
  value: 1.5
- system: skill
  target: leadership
  op: mult
  value: 1.6
- system: stress
  target: leadership_stress_rate
  op: mult
  value: 1.5
  note: every decision adds pressure — the crown is heavy
- system: aura
  target: fear_and_awe
  op: set
  radius: 5
  intensity: 0.3
  condition:
    source: self
    key: stress_level
    op: gte
    value: 50
- system: derived
  target: risk_tolerance
  op: mult
  value: 1.5
  note: the storm crown does not fear consequences
- on_event: gives_order_while_stressed
  effect: overwhelming_compliance_surge
```

### #6 Y_silent_forge
> Silent Forge / 침묵의 대장간 — Mechanical precision fused with ancestral craft mastery produces creations that transcend current knowledge.

```yaml
- system: special
  target: impossible_crafting
  op: enable
  value: true
  note: SIGNATURE — can craft items one technology tier beyond current civilization level
- system: skill
  target: crafting
  op: mult
  value: 1.8
- system: skill
  target: construction
  op: mult
  value: 1.5
- system: derived
  target: focus_duration
  op: mult
  value: 2.0
  note: clockwork mind does not wander
- system: need
  target: social
  op: mult
  value: 0.6
  note: sensitivity reduced — solitude is workshop, not loneliness
- system: stress
  target: craft_deprivation_stress
  op: set
  value: 0.1
  condition:
    source: self
    key: ticks_since_crafting
    op: gte
    value: 100
- on_event: enters_deep_focus
  effect: tech_tier_bypass_crafting
```

### #7 Y_broken_mirror
> Broken Mirror / 깨진 거울 — Fractured self-image polished by repeated betrayal becomes a dark lens that sees treachery before it arrives.

```yaml
- system: special
  target: betrayal_precognition
  op: enable
  value: true
  note: SIGNATURE — detects incoming betrayal with high accuracy before it happens
- system: skill
  target: deception_detection
  op: mult
  value: 2.0
- system: derived
  target: trustworthiness
  op: mult
  value: 0.8
  note: trust starts near zero — assumes betrayal is default
- system: stress
  target: social_stress_rate
  op: mult
  value: 1.6
  note: every conversation is a search for the hidden knife
- system: derived
  target: pattern_recognition
  op: mult
  value: 1.5
- system: need
  target: security
  op: mult
  value: 1.6
  note: sensitivity amplified — chronic hypervigilance
- on_event: detects_betrayal
  effect: preemptive_countermeasure_opportunity
```

### #8 Y_holy_fire
> Holy Fire / 성스러운 불꽃 — Pure compassion ignited by divine touch becomes a flame that heals others by consuming the healer.

```yaml
- system: special
  target: sacrificial_healing
  op: enable
  value: true
  note: SIGNATURE — absorbs others trauma and stress into own body and mind
- system: aura
  target: stress_recovery
  op: mult
  value: 1.4
  radius: 4
  note: passive warmth — those nearby heal faster
- system: derived
  target: trustworthiness
  op: mult
  value: 1.5
- system: need
  target: rest
  op: mult
  value: 1.6
  note: sensitivity amplified — divine work demands mortal payment
- system: behavior
  target: self_preservation_over_others
  op: block
  value: true
  note: prioritizes others pain over own survival
- system: combat
  target: willingness_to_harm
  op: mult
  value: 0.4
  note: golden heart recoils from violence
- on_event: ally_stress_critical
  effect: absorb_trauma_into_self
```

### #9 Y_wolves_pact
> Wolves' Pact / 늑대의 서약 — Blood oath loyalty fused with pack instinct creates a bond where the group becomes one predatory mind.

```yaml
- system: special
  target: pack_resonance
  op: enable
  value: true
  note: SIGNATURE — bonded group shares combat awareness, acts as single predatory unit
- system: combat
  target: coordination
  op: mult
  value: 1.8
  condition:
    source: self
    key: bonded_allies_nearby
    op: gte
    value: 2
- system: combat
  target: attack_power
  op: mult
  value: 1.4
  condition:
    source: self
    key: bonded_allies_nearby
    op: gte
    value: 1
- system: behavior
  target: abandon_pack_member
  op: block
  value: true
  note: will not retreat while a pack member is in danger
- system: need
  target: social
  op: override
  value: bonded_allies_only
  note: social needs only satisfied by pack members
- system: stress
  target: pack_death_stress
  op: set
  value: 0.3
  note: pack member death is devastating
- on_event: bonded_ally_enters_combat
  effect: pack_resonance_auto_engage
```

### #10 Y_poisoned_well
> Poisoned Well / 독이 든 우물 — Web-weaving manipulation and silver-tongued persuasion combine to corrupt entire social networks from within.

```yaml
- system: special
  target: network_corruption
  op: enable
  value: true
  note: SIGNATURE — can turn an entire factions relationships toxic from within
- system: skill
  target: manipulation
  op: mult
  value: 2.0
- system: skill
  target: rumor_spreading
  op: mult
  value: 2.5
  note: rumors are 3x more believed and harder to trace
- system: skill
  target: faction_infiltration
  op: mult
  value: 1.8
- system: behavior
  target: form_genuine_connection
  op: block
  value: true
  note: every relationship is a node in the web
- system: stress
  target: web_management_stress
  op: set
  value: 0.08
  note: managing multiple webs grinds the mind
- on_event: joins_faction
  effect: slow_trust_erosion_begins
```
### #11 Y_bleeding_root
> Bleeding Root / 피뿌리 — Ancestral suffering flows through the veins, manifesting as both power and wound.

```yaml
- system: combat
  target: damage_output
  op: mult
  value: 1.4
  condition:
    source: self
    key: health_pct
    op: lt
    value: 0.5
- system: body
  target: hp_regen
  op: mult
  value: 0.6
  note: ancient wounds never fully close
- system: special
  target: ancestral_wound_channel
  op: enable
  value: true
  note: SIGNATURE — can convert own HP into burst damage by channeling ancestor pain
- system: stress
  target: grief_stress
  op: immune
  value: true
  note: already saturated with generational sorrow
- system: need
  target: social
  op: mult
  value: 1.3
  note: sensitivity amplified — weight of bloodline isolates
- system: derived
  target: intimidation
  op: mult
  value: 1.3
- on_event: wounded_in_combat
  effect: ancestral_power_surge
```

### #12 Y_scarred_diamond
> Scarred Diamond / 상흔금강 — A body that cannot die encasing a soul that cannot heal.

```yaml
- system: body
  target: damage_reduction
  op: mult
  value: 1.5
- system: body
  target: hp_regen
  op: mult
  value: 1.4
- system: special
  target: trauma_armor
  op: enable
  value: true
  note: SIGNATURE — each scar increases damage_reduction but decreases emotional_recovery proportionally
- system: stress
  target: emotional_recovery_rate
  op: mult
  value: 0.4
  note: emotional wounds calcify rather than heal
- system: need
  target: comfort
  op: mult
  value: 1.5
  note: sensitivity amplified — perpetual tension
- system: derived
  target: stoicism
  op: mult
  value: 1.4
- on_event: survives_lethal_damage
  effect: new_scar_permanent_dr_bonus
```

### #13 Y_autumn_hymn
> Autumn Hymn / 가을찬가 — A song warm enough to thaw the frost that grief leaves on the heart.

```yaml
- system: special
  target: frost_thaw_song
  op: enable
  value: true
  note: SIGNATURE — musical performance can remove W_widows_frost from OTHER entities (unique cure in system)
- system: skill
  target: performance
  op: mult
  value: 1.8
- system: aura
  target: grief_reduction
  op: mult
  value: 1.4
  radius: 5
  condition:
    source: self
    key: is_performing
    op: eq
    value: true
- system: stress
  target: emotional_range
  op: mult
  value: 0.7
  note: must contain own emotions to channel others
- system: need
  target: social
  op: mult
  value: 1.3
  note: sensitivity amplified — giving warmth depletes reserves
- system: stress
  target: creative_pressure_stress
  op: set
  value: 0.1
  condition:
    source: self
    key: days_since_performance
    op: gte
    value: 10
- on_event: performs_song
  effect: aoe_grief_thaw_chance
```

### #14 Y_night_garden
> Night Garden / 밤의 정원 — Where shadow falls, green things wake.

```yaml
- system: special
  target: shadow_cultivation
  op: enable
  value: true
  note: SIGNATURE — can grow resources in barren/hostile tiles that others cannot use
- system: skill
  target: gathering
  op: mult
  value: 1.6
  condition:
    source: world
    key: is_night
    op: eq
    value: true
- system: skill
  target: gathering
  op: mult
  value: 0.7
  condition:
    source: world
    key: is_night
    op: eq
    value: false
- system: body
  target: night_move_speed
  op: mult
  value: 1.3
- system: stress
  target: sunlight_stress
  op: set
  value: 0.08
  condition:
    source: world
    key: is_day
    op: eq
    value: true
- system: reputation
  target: tag
  op: tag
  value: shadow_gardener
- on_event: enters_barren_tile
  effect: shadow_growth_begins
```

### #15 Y_living_monument
> Living Monument / 살아있는 기념비 — Their words do not fade; they become the stone on which culture is built.

```yaml
- system: special
  target: word_permanence
  op: enable
  value: true
  note: SIGNATURE — spoken declarations become binding cultural laws persisting beyond death
- system: derived
  target: charisma
  op: mult
  value: 1.5
- system: skill
  target: writing
  op: mult
  value: 1.6
- system: stress
  target: word_burden_stress
  op: set
  value: 0.12
  note: every word may become permanent law
- system: behavior
  target: retract_declaration
  op: block
  value: true
  note: cannot take back declarations once spoken
- system: aura
  target: cultural_output
  op: mult
  value: 1.5
  radius: 5
- on_event: makes_declaration
  effect: declaration_becomes_cultural_law
```

### #16 Y_ember_prophet
> Ember Prophet / 잿불예언자 — Visions arrive wreathed in flame, showing what must burn so renewal can begin.

```yaml
- system: special
  target: fire_prophecy
  op: enable
  value: true
  note: SIGNATURE — prophetic visions about what must be destroyed for renewal
- system: derived
  target: wisdom
  op: mult
  value: 1.5
- system: skill
  target: demolition
  op: mult
  value: 1.8
- system: stress
  target: prophecy_suppression_stress
  op: set
  value: 0.12
  condition:
    source: self
    key: prophecy_suppressed_count
    op: gte
    value: 3
- system: reputation
  target: tag
  op: tag
  value: ember_prophet
- system: aura
  target: renewal_readiness
  op: mult
  value: 1.3
  radius: 4
- on_event: prophecy_fulfilled
  effect: stress_purge_and_settlement_renewal
```

### #17 Y_gentle_cage
> Gentle Cage / 다정한 새장 — A prison made of warmth, with bars you never want to touch.

```yaml
- system: special
  target: loving_imprisonment
  op: enable
  value: true
  note: SIGNATURE — protected entities cannot voluntarily leave but gain contentment
- system: skill
  target: caretaking
  op: mult
  value: 2.0
- system: aura
  target: need_decay_reduction
  op: mult
  value: 0.6
  radius: 3
  note: those in care have slower need decay
- system: aura
  target: autonomy_suppression
  op: mult
  value: 0.3
  radius: 3
  note: protected entities lose autonomous decision-making
- system: stress
  target: guilt_accumulation_stress
  op: set
  value: 0.08
  condition:
    source: self
    key: protected_count
    op: gte
    value: 3
- system: reputation
  target: outsider_perception
  op: mult
  value: 0.7
  note: those outside the cage see the bars clearly
- on_event: protected_entity_attempts_leave
  effect: emotional_persuasion_to_stay
```

### #18 Y_blood_architect
> Blood Architect / 핏줄건축가 — Giant's bone and mason's eye converge to raise what lesser hands cannot conceive.

```yaml
- system: special
  target: megalithic_construction
  op: enable
  value: true
  note: SIGNATURE — can build structures requiring 3x normal workforce alone
- system: skill
  target: construction
  op: mult
  value: 2.0
- system: skill
  target: crafting
  op: mult
  value: 1.5
- system: need
  target: food
  op: mult
  value: 1.6
  note: sensitivity amplified — giant blood demands enormous calories
- system: body
  target: move_speed
  op: mult
  value: 0.85
  note: massive frame moves deliberately
- system: aura
  target: construction_inspiration
  op: mult
  value: 1.3
  radius: 4
- on_event: completes_megalithic_build
  effect: settlement_building_capacity_bonus
```

### #19 Y_truth_serum
> Truth Serum / 진실의 혈청 — A voice so clean it dissolves the membrane between thought and speech.

```yaml
- system: special
  target: compelled_honesty
  op: enable
  value: true
  note: SIGNATURE — direct questions impose severe deception penalty on targets
- system: behavior
  target: lie
  op: block
  value: true
  note: incorruptible integrity is absolute
- system: skill
  target: interrogation
  op: mult
  value: 2.0
- system: skill
  target: diplomacy
  op: mult
  value: 0.6
  note: diplomacy requires selective truth — their gift ruins negotiation
- system: stress
  target: truth_burden_stress
  op: set
  value: 0.1
  note: hearing every truth unfiltered
- system: aura
  target: corruption_suppression
  op: mult
  value: 0.5
  radius: 5
  note: corruption withers where no lie goes undetected
- on_event: asks_direct_question
  effect: target_deception_severely_penalized
```

### #20 Y_chain_march
> Chain March / 사슬행진 — Every chain broken forges a new bond; every freed soul finds a family.

```yaml
- system: special
  target: liberation_bonds
  op: enable
  value: true
  note: SIGNATURE — freed entities automatically form high-loyalty bonds with each other
- system: skill
  target: liberation
  op: mult
  value: 1.8
- system: derived
  target: charisma
  op: mult
  value: 1.3
- system: need
  target: belonging
  op: mult
  value: 1.4
  note: sensitivity amplified — creates families for others but never fully belongs
- system: behavior
  target: submit_to_authority
  op: block
  value: true
  note: instinctive defiance of hierarchical control
- system: aura
  target: migration_attraction
  op: mult
  value: 1.4
  radius: 6
  note: the oppressed migrate toward their settlements
- on_event: frees_entity
  effect: instant_loyalty_bond_formation
```
### #21 Y_fading_star
> Fading Star / 지는 별 — A beautiful presence that dims everywhere it lingers; must keep moving or watch the world wither.

```yaml
- system: special
  target: entropic_presence
  op: enable
  value: true
  note: SIGNATURE — the longer they stay, the more vitality drains from the location; must keep moving
- system: derived
  target: charisma
  op: mult
  value: 1.6
  condition:
    source: self
    key: ticks_in_settlement
    op: lt
    value: 30
  note: immense social influence on arrival that decays rapidly
- system: derived
  target: charisma
  op: mult
  value: 0.6
  condition:
    source: self
    key: ticks_in_settlement
    op: gt
    value: 200
- system: body
  target: move_speed
  op: mult
  value: 1.3
  note: compounding speed bonus while continuously traveling
- system: stress
  target: bond_formation_rate
  op: mult
  value: 0.4
  note: emotional frost prevents quick attachment
- system: aura
  target: settlement_cultural_richness
  op: add
  value: 0.03
  radius: 0
  note: each visited settlement gains a sliver of permanent cultural richness on departure
- on_event: departs_settlement
  effect: morale_bloom_then_fade
```

### #22 Y_red_garden
> Red Garden / 붉은 정원 — Nurtures through violence, protects through terror; love and cruelty bloom from the same root.

```yaml
- system: special
  target: protective_terror
  op: enable
  value: true
  note: SIGNATURE — enemies who threaten loved ones suffer permanent fear debuff
- system: combat
  target: damage_output
  op: mult
  value: 1.3
  condition:
    source: self
    key: dependents_threatened
    op: eq
    value: true
  note: maternal fury scales with threats to loved ones
- system: aura
  target: crime_suppression
  op: mult
  value: 0.4
  radius: 5
  note: settlement crime drops dramatically while present
- system: stress
  target: violence_satisfaction
  op: set
  value: -0.1
  condition:
    source: self
    key: defeated_threat_to_loved_one
    op: eq
    value: true
  note: stress relief from punishing threats
- system: behavior
  target: loved_ones_leave
  op: block
  value: true
  note: loved ones migration desire suppressed — protected and trapped
- system: aura
  target: tile_fertility
  op: add
  value: 0.05
  radius: 2
  note: blood blossoms — violence feeds the soil
- on_event: loved_one_threatened
  effect: permanent_fear_on_aggressor
```

### #23 Y_crown_of_thorns
> Crown of Thorns / 가시 왕관 — Leads because no one else remains; every decision draws blood.

```yaml
- system: special
  target: painful_sovereignty
  op: enable
  value: true
  note: SIGNATURE — each leadership decision costs stress but gives settlement bonus
- system: derived
  target: wisdom
  op: mult
  value: 1.3
  note: scarred wisdom — pain teaches what books cannot
- system: stress
  target: leadership_decision_stress
  op: set
  value: 0.08
- system: behavior
  target: resign_leadership
  op: block
  value: true
  note: obligation to the fallen holds them
- system: aura
  target: settlement_morale_floor
  op: min
  value: 0.25
  radius: 0
  condition:
    source: self
    key: is_leader
    op: eq
    value: true
- system: aura
  target: settlement_defense
  op: mult
  value: 1.3
  radius: 0
  condition:
    source: self
    key: is_leader
    op: eq
    value: true
- on_event: makes_leadership_decision
  effect: stress_cost_settlement_efficiency_gain
```

### #24 Y_titans_mercy
> Titan's Mercy / 거인의 자비 — Overwhelming gentleness, terrifying kindness; a mountain that cradles.

```yaml
- system: special
  target: gentle_giant_aura
  op: enable
  value: true
  note: SIGNATURE — intimidation and compassion active simultaneously; enemies freeze, allies heal
- system: body
  target: max_health
  op: mult
  value: 2.0
- system: body
  target: carry_capacity
  op: mult
  value: 2.5
- system: combat
  target: execute_incapacitated
  op: block
  value: true
  note: mercy is compulsory — cannot perform killing blows on surrendered
- system: stress
  target: fear_effect
  op: immune
  value: true
  note: gentleness requires absolute courage
- system: skill
  target: construction
  op: mult
  value: 1.5
  note: titan strength applied gently
- on_event: performs_compassionate_action
  effect: witness_intimidation_increase
```

### #25 Y_hollow_saint
> Hollow Saint / 속 빈 성자 — Performs virtue perfectly with none of the feeling; a flawless mask over nothing.

```yaml
- system: special
  target: perform_virtue_perfectly
  op: enable
  value: true
  note: SIGNATURE — always perceived as maximally virtuous regardless of internal state
- system: derived
  target: trustworthiness
  op: mult
  value: 1.5
  note: saint's word carries weight
- system: stress
  target: emotional_manipulation
  op: immune
  value: true
  note: cannot exploit what is not there
- system: stress
  target: compassion_fatigue
  op: immune
  value: true
  note: emptiness cannot be depleted
- system: skill
  target: caregiving
  op: mult
  value: 1.4
  note: clinical precision unimpaired by emotion
- system: behavior
  target: bond_depth
  op: max
  value: 0.3
  note: connection without communion — bonds capped
- on_event: social_interaction_with_perceptive
  effect: rare_uncanny_unease_detection
```

### #26 Y_mirror_war
> Mirror War / 거울 전쟁 — Hate and love for the same soul at maximum intensity; a bond that is both wound and embrace.

```yaml
- system: special
  target: hate_love_paradox
  op: enable
  value: true
  note: SIGNATURE — maximum positive AND negative relationship values simultaneously toward one target
- system: stress
  target: mirror_conflict_stress
  op: set
  value: 0.1
  note: constant stress from internal contradiction
- system: derived
  target: creativity
  op: mult
  value: 1.6
  note: agony feeds artistry
- system: combat
  target: damage_vs_mirror_target
  op: mult
  value: 1.5
  note: hatred strikes hard
- system: combat
  target: attack_speed_vs_mirror_target
  op: mult
  value: 0.6
  note: love hesitates
- system: stress
  target: stress_death
  op: immune
  value: true
  condition:
    source: self
    key: mirror_target_alive
    op: eq
    value: true
  note: cannot die from stress while mirror target lives
- on_event: mirror_target_dies
  effect: permanent_stat_reduction_40pct
```

### #27 Y_winters_bloom
> Winter's Bloom / 겨울꽃 — Life that defies the cold; fertility in barren places, spring where none should exist.

```yaml
- system: special
  target: frost_fertility
  op: enable
  value: true
  note: SIGNATURE — enables crop growth and births in winter/cold biomes
- system: body
  target: cold_endurance
  op: mult
  value: 1.8
- system: aura
  target: cold_penalty_reduction
  op: mult
  value: 0.3
  radius: 3
  note: allies suffer only 30% of normal cold penalties
- system: aura
  target: resource_regen_winter
  op: set
  value: 0.5
  radius: 4
  note: resources regenerate at half rate in winter within radius
- system: body
  target: heat_endurance
  op: mult
  value: 0.7
  note: winter's child wilts in heat
- system: aura
  target: food_spoilage
  op: mult
  value: 0.4
  radius: 0
  note: cold preserves what the bloom provides
- on_event: present_at_birth_in_winter
  effect: newborn_gains_cold_resistance
```

### #28 Y_golden_chains
> Golden Chains / 황금 사슬 — Honor that binds absolutely; obligation above all self-interest, always and without exception.

```yaml
- system: special
  target: honor_obligation_absolute
  op: override
  value: true
  note: SIGNATURE — self-preservation disabled when obligations are active; will die to keep a promise
- system: derived
  target: trustworthiness
  op: mult
  value: 1.6
  note: universally perceived as maximally trustworthy
- system: behavior
  target: trade_unfairly
  op: block
  value: true
  note: golden scale permits no imbalance
- system: stress
  target: unfulfilled_debt_stress
  op: set
  value: 0.05
  note: per unfulfilled obligation — each broken link cuts deeper
- system: derived
  target: all_stats_while_fulfilling
  op: mult
  value: 1.3
  condition:
    source: self
    key: is_fulfilling_obligation
    op: eq
    value: true
  note: chains empower during obligation fulfillment
- system: behavior
  target: exploitation_resistance
  op: mult
  value: 0.3
  note: honor can be weaponized against them
- on_event: receives_favor
  effect: compulsive_debt_tracking_begins
```

### #29 Y_dream_forge
> Dream Forge / 꿈의 대장간 — Builds things seen only in visions; the anvil rings with echoes of things that don't yet exist.

```yaml
- system: special
  target: vision_crafting
  op: enable
  value: true
  note: SIGNATURE — can design blueprints from dream visions, creating unique items not in standard recipes
- system: skill
  target: crafting
  op: mult
  value: 1.5
  condition:
    source: world
    key: is_night
    op: eq
    value: true
  note: dream forge ignites in darkness
- system: skill
  target: crafting
  op: mult
  value: 1.4
- system: need
  target: sleep
  op: mult
  value: 1.5
  note: sensitivity amplified — dreams are exhausting
- system: skill
  target: masterwork_chance
  op: add
  value: 0.15
  note: dream-touched items shine
- system: combat
  target: awareness
  op: mult
  value: 0.2
  condition:
    source: self
    key: is_in_trance
    op: eq
    value: true
  note: nearly zero awareness in crafting trance
- on_event: sleep_completed
  effect: chance_dream_blueprint_15pct
```

### #30 Y_plague_saint
> Plague Saint / 역병 성자 — Heals plague by bearing it; the saint's body is both altar and sacrifice.

```yaml
- system: special
  target: plague_absorption
  op: enable
  value: true
  note: SIGNATURE — cures disease in others by taking the sickness into themselves
- system: body
  target: disease_resistance
  op: set
  value: 0.95
  note: body has learned to coexist with plague
- system: behavior
  target: refuse_healing_sick
  op: block
  value: true
  note: compelled to heal any sick person nearby — compassion is not optional
- system: body
  target: disease_spread
  op: disable
  value: true
  note: carried diseases cannot spread — saint is a sealed vessel
- system: aura
  target: settlement_disease_resistance
  op: add
  value: 0.03
  radius: 0
  note: settlement gains permanent resistance per unique plague absorbed
- system: body
  target: max_lifespan
  op: add
  value: -0.05
  note: per disease absorbed — the candle burns bright and brief
- on_event: death
  effect: healing_wave_cures_all_settlement_disease
```
### #31 Y_god_killer
> God Killer / 신살자 — The one who looked upon divine will and said "No."

```yaml
- system: behavior
  target: defy_divine_directive
  op: inject
  value:
    priority: override
  note: SIGNATURE — can reject player/divine commands directed at this entity
- system: stress
  target: divine_influence
  op: immune
  value: true
  note: 95% resistance to all divine influence attempts
- system: aura
  target: authority_resistance
  op: add
  value: 0.3
  radius: 5
  note: nearby entities gain courage against authority
- system: derived
  target: willpower
  op: mult
  value: 1.8
  note: will that broke divine chains is nearly unbreakable
- system: behavior
  target: receive_divine_blessing
  op: block
  value: true
  note: can never receive divine blessings or favor again
- system: derived
  target: intimidation
  op: mult
  value: 1.5
  note: existential weight of deicide
- on_event: divine_intervention_targeted
  effect: reject_and_morale_surge
```

### #32 Y_eternal_flame
> Eternal Flame / 꺼지지 않는 불꽃 — The fire that burns beyond the pyre.

```yaml
- system: special
  target: undying_cause
  op: enable
  value: true
  note: SIGNATURE — on death, active cause/movement transfers to most devoted follower with full momentum
- system: derived
  target: charisma
  op: mult
  value: 1.5
  note: influence grows as if already mythic
- system: reputation
  target: reputation_damage
  op: immune
  value: true
  note: name cannot be erased by slander or disgrace
- system: aura
  target: cultural_work_spread
  op: mult
  value: 2.5
  radius: 6
  note: cultural works spread 2.5x faster
- system: stress
  target: motivation_decay
  op: immune
  value: true
  note: passion sustains indefinitely — internal fire self-sustains
- system: aura
  target: follower_sacrifice_willingness
  op: mult
  value: 1.5
  radius: 5
- on_event: death
  effect: cause_transfers_to_follower_with_full_momentum
```

### #33 Y_three_faced
> Three-Faced / 삼면인 — One body, three truths, none of them lies.

```yaml
- system: special
  target: maintain_three_simultaneous_personas
  op: enable
  value: true
  note: SIGNATURE — can maintain three separate fully-realized identities at once (unique in system)
- system: skill
  target: deception_detection_resistance
  op: set
  value: 0.95
  note: persona shifts leave no behavioral tells
- system: skill
  target: information_gathering
  op: mult
  value: 2.5
  note: gathers intel from three separate social positions
- system: stress
  target: identity_maintenance_stress
  op: set
  value: 0.15
  note: constant stress from maintaining three simultaneous selves
- system: stress
  target: forced_identity_reveal
  op: immune
  value: true
  note: cannot be forcibly unmasked by any single investigation
- system: reputation
  target: tracks
  op: set
  value: 3
  note: three independent reputation scores, one per persona
- on_event: persona_denounces_another_persona
  effect: strategic_self_betrayal_for_advantage
```

### #34 Y_pain_weaver
> Pain Weaver / 고통직공 — Fingers that turn scars into silk.

```yaml
- system: special
  target: suffering_transmutation
  op: enable
  value: true
  note: SIGNATURE — converts accumulated stress into crafting/art quality bonus
- system: skill
  target: crafting
  op: mult
  value: 1.5
  condition:
    source: self
    key: stress_level
    op: gte
    value: 50
- system: skill
  target: crafting
  op: mult
  value: 0.3
  condition:
    source: self
    key: stress_level
    op: lt
    value: 20
  note: contentment kills the muse
- system: aura
  target: stress_relief_via_art
  op: add
  value: 15
  radius: 4
  note: entities who experience their art lose 15 stress points
- system: skill
  target: counseling
  op: mult
  value: 2.0
  note: understands suffering deeply
- system: skill
  target: hidden_trauma_detection
  op: enable
  value: true
  note: can perceive others hidden trauma through observation
- on_event: creates_art_while_stressed
  effect: masterwork_threshold_reduced
```

### #35 Y_ashes_dawn
> Ashes of Dawn / 재의 여명 — Everything burned. The hands still build.

```yaml
- system: special
  target: phoenix_rebuilding
  op: enable
  value: true
  note: SIGNATURE — after catastrophic personal loss, gains temporary massive stat boost to rebuild
- system: stress
  target: morale_floor
  op: min
  value: 0.15
  note: something always survives in the ashes
- system: skill
  target: construction
  op: mult
  value: 2.5
  condition:
    source: settlement
    key: recently_destroyed
    op: eq
    value: true
  note: rebuilder instinct — construction speed surges after destruction
- system: stress
  target: stress_floor
  op: set
  value: 0.2
  note: the ashes never fully cool — permanent minimum stress
- system: aura
  target: refugee_attraction
  op: mult
  value: 1.8
  radius: 6
  note: displaced entities drawn to their location
- system: derived
  target: wisdom
  op: mult
  value: 1.3
  note: dawn-sight — can see opportunity in devastation
- on_event: catastrophic_personal_loss
  effect: phoenix_surge_all_stats_2x_for_200_ticks
```

### #36 Y_unmoving_storm
> Unmoving Storm / 부동뇌 — The mountain that contains the hurricane.

```yaml
- system: special
  target: calm_fury_paradox
  op: enable
  value: true
  note: SIGNATURE — combat power increases proportionally to emotional control, not intensity
- system: combat
  target: fear_effect
  op: immune
  value: true
- system: combat
  target: panic_effect
  op: immune
  value: true
- system: combat
  target: rage_effect
  op: immune
  value: true
  note: immune to all combat-induced emotional states
- system: combat
  target: first_strike_damage
  op: mult
  value: 2.0
  note: first strike from stillness — the mountain moves once
- system: aura
  target: dual_presence
  op: enable
  value: true
  radius: 4
  note: allies within radius lose stress; enemies lose morale
- system: stress
  target: peace_accumulation_stress
  op: set
  value: 0.05
  condition:
    source: self
    key: days_since_combat
    op: gte
    value: 30
  note: storm demands release — stress builds during extended peace
- on_event: maintains_calm_in_combat
  effect: combat_power_scaling_with_control
```

### #37 Y_bleeding_compass
> Bleeding Compass / 피묻은 나침반 — The oath-breaker who swears harder each time, and means it less.

```yaml
- system: derived
  target: trustworthiness
  op: mult
  value: 0.7
  note: SIGNATURE — the ONLY synergy trait that REDUCES a core social stat
- system: skill
  target: oath_persuasiveness
  op: mult
  value: 1.8
  note: oaths sound incredibly convincing despite the paradox
- system: behavior
  target: refuse_commitment
  op: block
  value: true
  note: compulsively makes oaths and promises
- system: skill
  target: betrayal_detection
  op: mult
  value: 2.5
  note: takes one to know one — a traitor knows the signs
- system: body
  target: pathfinding_discovery
  op: mult
  value: 1.6
  note: finds hidden paths others cannot, but 30% wrong direction
- system: stress
  target: oath_broken_stress
  op: set
  value: 0.15
  note: gains stress each time oath broken — knows the pattern but cannot stop
- on_event: oath_made
  effect: 30pct_chance_iron_phase_absolute_reliability_50_ticks
```

### #38 Y_moons_cradle
> Moon's Cradle / 달의 요람 — Mad hands that still hold gently.

```yaml
- system: behavior
  target: lucid_protection
  op: inject
  value:
    priority: 0.9
    condition:
      source: self
      key: moon_sickness_active
      op: eq
      value: true
  note: SIGNATURE — protective behavior toward dependents continues even during moon-sickness episodes
- system: derived
  target: perception
  op: mult
  value: 1.8
  condition:
    source: self
    key: moon_sickness_active
    op: eq
    value: true
  note: madness opens other senses
- system: aura
  target: calming_presence
  op: mult
  value: 1.3
  radius: 3
  condition:
    source: self
    key: moon_sickness_active
    op: eq
    value: true
  note: calms others even while in altered state
- system: behavior
  target: dependent_danger_sense
  op: set
  value: unlimited
  note: can sense dependents in danger regardless of distance
- system: reputation
  target: tag
  op: tag
  value: lunar_guardian
- system: stress
  target: post_episode_exhaustion
  op: set
  value: 0.2
  note: needs extended rest after each episode
- on_event: moon_sickness_begins
  effect: protection_override_activates
```

### #39 Y_blood_remembers
> Blood Remembers / 피의 기억 — The veins know the way the mind has forgotten.

```yaml
- system: special
  target: ancestral_navigation
  op: enable
  value: true
  note: SIGNATURE — can find ancestral locations without maps; blood literally guides them
- system: derived
  target: all_stats
  op: mult
  value: 1.25
  condition:
    source: location
    key: is_ancestral_territory
    op: eq
    value: true
  note: all stats boosted on ancestral territory — the blood sings
- system: combat
  target: combat_power
  op: mult
  value: 1.4
  condition:
    source: location
    key: ancestral_battle_site
    op: eq
    value: true
  note: the blood remembers how to win here
- system: skill
  target: ancestor_skill_access
  op: enable
  value: 0.4
  note: can use skills known by ancestors at 40% proficiency without training
- system: stress
  target: inherited_trauma
  op: set
  value: 0.05
  note: permanent baseline stress from echoes of ancestral suffering
- system: aura
  target: bloodline_attraction
  op: mult
  value: 1.4
  radius: 6
  note: entities sharing bloodline feel drawn
- on_event: sleep_completed
  effect: 20pct_chance_ancestral_memory_dream
```

### #40 Y_dawn_of_war
> Dawn of War / 전쟁의 여명 — The mind that wages war on everything, including peace.

```yaml
- system: behavior
  target: optimize_for_war_constantly
  op: inject
  value:
    priority: 0.7
    scope: all_interactions
  note: SIGNATURE — tactical assessment runs on ALL interactions, not just combat
- system: combat
  target: reaction_speed
  op: mult
  value: 1.8
  note: body permanently in combat readiness
- system: combat
  target: dawn_attack_power
  op: mult
  value: 2.0
  condition:
    source: world
    key: time_of_day
    op: eq
    value: dawn
  note: hammer falls at first light
- system: skill
  target: negotiation
  op: mult
  value: 1.6
  note: treats diplomacy as warfare by other means
- system: aura
  target: settlement_resource_efficiency
  op: mult
  value: 1.3
  radius: 0
  note: military logistics thinking applied to settlement economy
- system: behavior
  target: relationship_quality_cap
  op: max
  value: 0.7
  note: cannot stop evaluating loved ones as tactical assets
- system: stress
  target: peacetime_stress
  op: set
  value: 0.08
  condition:
    source: settlement
    key: days_since_conflict
    op: gte
    value: 60
  note: the war mind has no off switch
- system: aura
  target: nearby_combat_skill
  op: add
  value: 0.15
  radius: 5
  note: combatants gain tactical skill from proximity to war genius
- on_event: enters_any_area
  effect: instant_terrain_tactical_assessment
```
