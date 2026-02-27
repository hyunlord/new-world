# WorldSim Trait Effects — Part 6 (Mastery #1~#20, Bond #1~#20)
> Canonical YAML effect definitions using Trait System v3 schema.
> Ops: set / add / mult / min / max / disable / enable / block / inject / override / on_event / tag / immune / replace
> All conditions use structured format: {source, key, op, value}
>
> Part 1: Archetype #1~#30 + Shadow #1~#5 ✅
> Part 2: Archetype #31~#55 ✅
> Part 3: Shadow #6~#15 + Radiance #1~#12 ✅
> Part 4: Corpus #1~#12 + Nous #1~#10 ✅
> Part 5: Awakened #1~#18 + Bloodline #1~#25 ✅
> **Part 6: Mastery #1~#20 + Bond #1~#20 ← this file**
> Part 7: Fate + Synergy

---

## Mastery Traits (M_) — Craft Identity Transformations

> Mastery traits mark the point where a skill stops being something an entity *does* and becomes something they *are*.
> They are acquired through sustained elite performance and carry compulsive craft behaviors, heightened quality sensitivity,
> and a slow decay if the craft is abandoned — the master who stops practicing feels the loss as physical grief.
> Reference: Csikszentmihalyi Flow Theory, Ericsson deliberate practice, Dreyfus skill acquisition model.

---

### #1 M_anvils_echo
> **Anvil's Echo / 모루의 메아리** — The forge has become an extension of the body; steel speaks to this smith
> Acquisition: blacksmithing >= 18 | Rarity: epic
> Loss: decay 5y of craft neglect
> Academic: Csikszentmihalyi (1990) — Flow Theory; Ericsson et al. (1993) — deliberate practice

```yaml
effects:
  - system: skill
    target: blacksmithing
    op: mult
    value: 1.4
  - system: skill
    target: metalworking
    op: mult
    value: 1.25
  - system: skill
    target: weaponsmithing
    op: mult
    value: 1.25
  - system: skill
    target: armorsmithing
    op: mult
    value: 1.25
  - system: behavior
    target: inspect_metalwork
    op: inject
    value: { priority: 0.5 }
  - system: need
    target: competence_fulfillment_rate
    op: mult
    value: 1.5
  - system: stress
    target: watching_poor_smithing_stress
    op: mult
    value: 1.6
  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.5
  - system: on_event
    target: produces_weapon
    op: inject
    value: { behavior: quality_check_event, threshold: substandard, block_output: true }
```

---

### #2 M_green_thumb
> **Verdant Touch / 푸른 손길** — The land responds to this farmer's presence; soil reads like a living text
> Acquisition: farming >= 18 | Rarity: epic
> Loss: decay 3y of crop abandonment
> Academic: Csikszentmihalyi (1990) — Flow Theory; traditional agrarian knowledge systems

```yaml
effects:
  - system: skill
    target: farming
    op: mult
    value: 1.4
  - system: skill
    target: crop_selection
    op: mult
    value: 1.4
  - system: skill
    target: irrigation
    op: mult
    value: 1.4
  - system: skill
    target: herbalism
    op: mult
    value: 1.2
  - system: skill
    target: foraging
    op: mult
    value: 1.2
  - system: behavior
    target: tend_crops_daily
    op: inject
    value: { priority: 0.6 }
  - system: need
    target: naturalistic_fulfillment
    op: mult
    value: 1.3
  - system: stress
    target: crop_waste_stress
    op: mult
    value: 1.5
  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.3
  - system: on_event
    target: planting_season
    op: inject
    value: { behavior: optimal_planting_sequence, reads: [soil, weather, timing] }
```

---

### #3 M_death_dealer
> **Death Dealer / 죽음의 상인** — Violence is no longer frightening; it is simply a language this warrior speaks fluently
> Acquisition: combat_skill >= 18 | Rarity: epic
> Loss: decay (skill < 15)
> Academic: Grossman (2009) — On Killing; stress inoculation theory (Meichenbaum 1985)

```yaml
effects:
  - system: combat
    target: damage_mult
    op: mult
    value: 1.45
  - system: combat
    target: tactical_read_speed
    op: mult
    value: 1.3
  - system: skill
    target: swordsmanship
    op: mult
    value: 1.4
  - system: skill
    target: archery
    op: mult
    value: 1.4
  - system: skill
    target: tactics
    op: mult
    value: 1.4
  - system: behavior
    target: assess_opponent_before_engaging
    op: inject
    value: { priority: 0.7 }
  - system: stress
    target: combat_stress
    op: mult
    value: 0.5
  - system: stress
    target: forced_rust_stress
    op: mult
    value: 1.6
  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.6
```

---

### #4 M_tongue_of_ages
> **Tongue of Ages / 시대의 혀** — Words become instruments; this speaker shapes rooms before a sentence ends
> Acquisition: persuasion >= 18 AND diplomacy >= 15 | Rarity: legendary
> Loss: decay 3y of social isolation
> Academic: Aristotle — Rhetoric (ethos/pathos/logos tripartite); Cialdini (1984) — Influence

```yaml
effects:
  - system: skill
    target: persuasion
    op: mult
    value: 1.5
  - system: skill
    target: diplomacy
    op: mult
    value: 1.5
  - system: skill
    target: oration
    op: mult
    value: 1.5
  - system: skill
    target: negotiation
    op: mult
    value: 1.5
  - system: derived
    target: charisma
    op: mult
    value: 1.35
  - system: derived
    target: wisdom
    op: mult
    value: 1.2
  - system: behavior
    target: teach_rhetoric
    op: inject
    value: { priority: 0.4 }
  - system: aura
    target: social_cohesion
    op: add
    value: 0.05
    radius: 3
  - system: stress
    target: watching_clumsy_negotiation_stress
    op: mult
    value: 1.4
  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.4
  - system: on_event
    target: diplomatic_encounter
    op: inject
    value: { settlement_success_rate_bonus: 0.2 }
  - system: on_event
    target: teaches_apprentice
    op: inject
    value: { apprentice_rhetoric_learning_rate: 1.5 }
```

---

### #5 M_bone_setter
> **Bone-Setter / 접골사** — The body's suffering is legible to this healer; walking away from the injured is not possible
> Acquisition: medicine >= 18 | Rarity: epic
> Loss: decay 5y of practice absence
> Academic: Ericsson et al. (1993) — deliberate practice; Hippocratic tradition; Beauchamp & Childress — biomedical ethics

```yaml
effects:
  - system: skill
    target: medicine
    op: mult
    value: 1.45
  - system: skill
    target: surgery
    op: mult
    value: 1.45
  - system: skill
    target: wound_treatment
    op: mult
    value: 1.45
  - system: behavior
    target: examine_injured
    op: inject
    value: { priority: 0.8 }
  - system: behavior
    target: refuse_to_treat
    op: block
    value: true
  - system: need
    target: meaning_fulfillment_rate
    op: mult
    value: 1.4
  - system: stress
    target: untreated_injury_nearby_stress
    op: mult
    value: 1.5
  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.4
  - system: on_event
    target: treating_patient
    op: inject
    value: { diagnosis_accuracy_mult: 1.4 }
```

---

### #6 M_wall_maker
> **Wall-Maker / 벽을 세우는 자** — Structures speak to this builder; weakness in stone is felt before it is seen
> Acquisition: architecture >= 18 AND fortification >= 15 | Rarity: legendary
> Loss: decay 5y of construction absence
> Academic: Dreyfus & Dreyfus (1986) — expert skill acquisition; Vitruvius — De Architectura (firmitas, utilitas, venustas)

```yaml
effects:
  - system: skill
    target: architecture
    op: mult
    value: 1.5
  - system: skill
    target: fortification
    op: mult
    value: 1.5
  - system: skill
    target: engineering
    op: mult
    value: 1.5
  - system: behavior
    target: inspect_settlement_defenses
    op: inject
    value: { priority: 0.5 }
  - system: derived
    target: wisdom
    op: mult
    value: 1.15
  - system: settlement
    target: fortification_decay_rate
    op: mult
    value: 0.5
    condition:
      source: self
      key: role
      op: eq
      value: resident
  - system: stress
    target: flawed_construction_stress
    op: mult
    value: 1.6
  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.5
  - system: on_event
    target: designing_structure
    op: inject
    value: { quality_mult: 1.45, duration_bonus: true }
  - system: on_event
    target: teaches_apprentice
    op: inject
    value: { apprentice_architecture_learning_rate: 1.5 }
```

---

### #7 M_thread_weaver
> **Thread-Weaver / 실을 엮는 자** — Pattern and texture are a private language; crude cloth is an affront to the world
> Acquisition: weaving >= 18 | Rarity: epic
> Loss: decay 5y of loom abandonment
> Academic: Csikszentmihalyi (1990) — Flow Theory; textile anthropology (craft-as-identity literature)

```yaml
effects:
  - system: skill
    target: weaving
    op: mult
    value: 1.45
  - system: skill
    target: textile_work
    op: mult
    value: 1.45
  - system: skill
    target: dyeing
    op: mult
    value: 1.45
  - system: skill
    target: pattern_design
    op: mult
    value: 1.2
  - system: skill
    target: garment_making
    op: mult
    value: 1.2
  - system: behavior
    target: examine_cloth_quality
    op: inject
    value: { priority: 0.4 }
  - system: need
    target: aesthetic_fulfillment
    op: mult
    value: 1.4
  - system: stress
    target: coarse_cloth_stress
    op: mult
    value: 1.3
  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.4
  - system: on_event
    target: produces_cloth
    op: inject
    value: { quality_mult: 1.4 }
```

---

### #8 M_song_keeper
> **Song-Keeper / 노래지기** — Memory lives in melody; this artist carries the community's grief, joy, and history in their voice
> Acquisition: music >= 18 AND poetry >= 15 | Rarity: legendary
> Loss: decay 3y of silence
> Academic: Oral tradition studies (Ong 1982 — Orality and Literacy); Csikszentmihalyi (1990) — Flow Theory

```yaml
effects:
  - system: skill
    target: music
    op: mult
    value: 1.5
  - system: skill
    target: poetry
    op: mult
    value: 1.5
  - system: skill
    target: storytelling
    op: mult
    value: 1.5
  - system: skill
    target: performance
    op: mult
    value: 1.5
  - system: derived
    target: creativity
    op: mult
    value: 1.4
  - system: derived
    target: wisdom
    op: mult
    value: 1.2
  - system: behavior
    target: memorize_important_events_as_song
    op: inject
    value: { priority: 0.5 }
  - system: aura
    target: morale_recovery_rate
    op: mult
    value: 1.4
    radius: 4
  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.5
  - system: on_event
    target: performs_during_grief
    op: inject
    value: { settlement_grief_recovery: major }
  - system: on_event
    target: composes_epic
    op: inject
    value: { effect: permanent_cultural_memory_added }
  - system: on_event
    target: teaches_apprentice
    op: inject
    value: { apprentice_music_learning_rate: 1.5 }
```

---

### #9 M_shadow_step
> **Shadow Step / 그림자 걸음** — Invisibility is a discipline this agent has perfected; they see the watchers because they were one
> Acquisition: espionage >= 18 | Rarity: epic
> Loss: decay 5y of tradecraft inactivity
> Academic: Intelligence tradecraft literature (Heuer 1999 — Psychology of Intelligence Analysis); situational awareness theory (Endsley 1995)

```yaml
effects:
  - system: skill
    target: espionage
    op: mult
    value: 1.45
  - system: skill
    target: stealth
    op: mult
    value: 1.45
  - system: skill
    target: tracking
    op: mult
    value: 1.45
  - system: skill
    target: pickpocket
    op: mult
    value: 1.45
  - system: behavior
    target: scan_environment_for_threats
    op: inject
    value: { priority: 0.6 }
  - system: behavior
    target: leave_no_trace
    op: inject
    value: { priority: 0.5 }
  - system: stress
    target: surveillance_detection_stress
    op: immune
    value: true
  - system: stress
    target: paranoia_from_own_methods
    op: set
    value: 0.08
  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.4
  - system: on_event
    target: information_gathered
    op: inject
    value: { quality_mult: 1.4 }
```

---

### #10 M_kings_hand
> **King's Hand / 왕의 손** — Systems disorder is physically painful; this administrator sees the optimal solution before the problem is finished being described
> Acquisition: administration >= 18 AND logistics >= 15 | Rarity: legendary
> Loss: decay 3y of administrative disengagement
> Academic: Weber (1922) — bureaucratic theory; systems management theory (Forrester 1961 — Industrial Dynamics)

```yaml
effects:
  - system: skill
    target: administration
    op: mult
    value: 1.5
  - system: skill
    target: logistics
    op: mult
    value: 1.5
  - system: skill
    target: law
    op: mult
    value: 1.5
  - system: skill
    target: command
    op: mult
    value: 1.5
  - system: derived
    target: wisdom
    op: mult
    value: 1.3
  - system: behavior
    target: optimize_resource_allocation
    op: inject
    value: { priority: 0.6 }
  - system: behavior
    target: delegate_correctly
    op: inject
    value: { priority: 0.5 }
  - system: stress
    target: disorganization_stress
    op: mult
    value: 1.5
  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.5
  - system: settlement
    target: efficiency_mult
    op: mult
    value: 1.3
    condition:
      source: self
      key: role
      op: eq
      value: administrative
  - system: on_event
    target: crisis_event
    op: inject
    value: { behavior: coordinate_response, priority: 0.8 }
  - system: on_event
    target: teaches_apprentice
    op: inject
    value: { apprentice_administration_learning_rate: 1.5 }
```
### #11 M_fire_tamer
> **Fire-Tamer / 불을 길들이는 자** — One who feeds not just bodies but belonging
> Acquisition: cooking ≥ 18 | Rarity: epic
> Loss: decay 3y (craft_neglect_stress source)
> Academic: Csikszentmihalyi Flow Theory; commensality research (Dunbar 2017)

```yaml
effects:
  - system: skill
    target: [cooking, food_preservation, brewing]
    op: mult
    value: 1.45

  - system: settlement
    target: food_satisfaction_mult
    op: mult
    value: 1.3
    condition:
      source: behavior
      key: cooking_for_community
      op: eq
      value: true

  - system: on_event
    target: feast_event
    op: inject
    value:
      behavior: settlement_morale_boost
      magnitude: major

  - system: need
    target: belonging.fulfillment_rate
    op: mult
    value: 1.3

  - system: behavior
    target: taste_and_adjust
    op: inject
    value:
      priority: 0.6

  - system: stress
    target: wasted_food_stress
    op: mult
    value: 1.4

  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.3
```

---

### #12 M_star_reader
> **Star-Reader / 별을 읽는 자** — One who measures time by fire that died a thousand years ago
> Acquisition: astronomy ≥ 18 AND mathematics ≥ 15 | Rarity: legendary
> Loss: decay 5y (craft_neglect_stress source)
> Academic: History of astronomical observation; Dreyfus expert-stage model

```yaml
effects:
  - system: skill
    target: [astronomy, mathematics, navigation, calendar_keeping]
    op: mult
    value: 1.5

  - system: settlement
    target: seasonal_prediction_accuracy
    op: mult
    value: 1.5
    condition:
      source: entity
      key: is_active_resident
      op: eq
      value: true

  - system: behavior
    target: observe_celestial_events
    op: inject
    value:
      priority: 0.7

  - system: derived
    target: wisdom
    op: mult
    value: 1.3

  - system: on_event
    target: predicts_eclipse_or_event
    op: inject
    value:
      behavior: announce_prediction
      side_effects:
        - reputation_boost
        - settlement_awe_event

  - system: need
    target: knowledge.fulfillment_rate
    op: mult
    value: 1.5

  - system: behavior
    target: record_observations
    op: inject
    value:
      priority: 0.5

  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.3
```

---

### #13 M_horse_whisperer
> **Horse-Whisperer / 말에게 속삭이는 자** — Speaks a language older than words
> Acquisition: animal_training ≥ 18 | Rarity: epic
> Loss: decay 3y (craft_neglect_stress source)
> Academic: Ethology; human-animal bond research (Serpell 1996)

```yaml
effects:
  - system: skill
    target: [animal_training, horse_riding, veterinary]
    op: mult
    value: 1.45

  - system: on_event
    target: encountering_animal
    op: inject
    value:
      behavior: read_and_calm_animal
      hostility_reduction: major

  - system: behavior
    target: read_animal_state
    op: inject
    value:
      priority: 0.6

  - system: on_event
    target: training_animal
    op: inject
    value:
      behavior: patient_conditioning
      modifier:
        target: training_speed
        op: mult
        value: 1.5

  - system: stress
    target: animal_mistreatment_nearby_stress
    op: mult
    value: 1.6

  - system: need
    target: naturalistic_fulfillment
    op: mult
    value: 1.4

  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.3
```

---

### #14 M_law_speaker
> **Law-Speaker / 법을 말하는 자** — Memory of the people, voice of what must hold
> Acquisition: law ≥ 18 AND persuasion ≥ 15 | Rarity: legendary
> Loss: decay 5y (craft_neglect_stress source)
> Academic: Legal philosophy; jurisprudence oral tradition (Gagarin 2008)

```yaml
effects:
  - system: skill
    target: [law, persuasion, arbitration, negotiation]
    op: mult
    value: 1.5

  - system: derived
    target: trustworthiness
    op: mult
    value: 1.35

  - system: derived
    target: wisdom
    op: mult
    value: 1.25

  - system: on_event
    target: legal_dispute
    op: inject
    value:
      behavior: arbitrate_dispute
      priority: 0.8

  - system: settlement
    target: legal_dispute_resolution_rate
    op: mult
    value: 1.4
    condition:
      source: entity
      key: is_active_resident
      op: eq
      value: true

  - system: behavior
    target: ignore_injustice
    op: block

  - system: aura
    target: LAW_value_drift
    op: add
    value: 0.01
    radius: 3

  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.3
```

---

### #15 M_stone_singer
> **Stone-Singer / 돌에게 노래하는 자** — Hears the seams the mountain tries to hide
> Acquisition: mining ≥ 18 | Rarity: epic
> Loss: decay 3y (craft_neglect_stress source)
> Academic: Geological intuition; mining tradition knowledge (Knapp 1999)

```yaml
effects:
  - system: skill
    target: [mining, geology, gem_cutting]
    op: mult
    value: 1.45

  - system: behavior
    target: read_rock_formations
    op: inject
    value:
      priority: 0.5

  - system: on_event
    target: surveying_terrain
    op: inject
    value:
      behavior: deep_formation_read
      modifier:
        target: mineral_discovery_chance
        op: mult
        value: 1.6

  - system: need
    target: competence.fulfillment_rate
    op: mult
    value: 1.4

  - system: stress
    target: shoddy_tunneling_stress
    op: mult
    value: 1.4

  - system: stress
    target: mine_collapse_risk_awareness_stress
    op: set
    value: 0.06

  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.3
```

---

### #16 M_root_finder
> **Root-Finder / 뿌리를 찾는 자** — Knows what the forest offers before hunger speaks
> Acquisition: herbalism ≥ 18 AND foraging ≥ 15 | Rarity: legendary
> Loss: decay 3y (craft_neglect_stress source)
> Academic: Ethnobotany; traditional medicine systems (Moerman 1998)

```yaml
effects:
  - system: skill
    target: [herbalism, foraging, medicine, alchemy]
    op: mult
    value: 1.5

  - system: behavior
    target: identify_plants_automatically
    op: inject
    value:
      priority: 0.6

  - system: on_event
    target: treating_with_herbs
    op: inject
    value:
      behavior: compound_optimal_remedy
      modifier:
        target: medicine_effectiveness
        op: mult
        value: 1.4

  - system: derived
    target: wisdom
    op: mult
    value: 1.2

  - system: settlement
    target: disease_resistance
    op: mult
    value: 1.2
    condition:
      source: entity
      key: is_active_resident
      op: eq
      value: true

  - system: need
    target: naturalistic_fulfillment
    op: mult
    value: 1.5

  - system: on_event
    target: new_plant_found
    op: inject
    value:
      behavior: document_and_test
      priority: 0.7

  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.3
```

---

### #17 M_death_midwife
> **Death's Midwife / 죽음의 산파** — Has sat with the dying so often that death no longer startles
> Acquisition: medicine ≥ 15 AND deaths_witnessed ≥ 10 | Rarity: epic
> Loss: PERMANENT — this is not a skill, it is what witnessing death does to a healer
> Academic: Kubler-Ross grief stages; palliative care psychology; moral injury (Litz et al. 2009)

```yaml
effects:
  - system: stress
    target: death_fear_stress
    op: immune

  - system: stress
    target: grief_witnessing_stress
    op: mult
    value: 0.4

  - system: skill
    target: [medicine, palliative_care, grief_counseling]
    op: mult
    value: 1.4

  - system: behavior
    target: sit_with_dying
    op: inject
    value:
      priority: 0.9

  - system: emotion
    target: sadness.processing_rate
    op: mult
    value: 1.5

  - system: need
    target: meaning.fulfillment_rate
    op: mult
    value: 1.5
```

---

### #18 M_bridge_builder
> **Bridge-Builder / 다리를 놓는 자** — Cannot give a biased ruling; the trait will not allow it
> Acquisition: negotiation ≥ 18 AND mediations_completed ≥ 5 | Rarity: legendary
> Loss: betrayal — 3 biased mediations trigger catastrophic allostatic collapse and trait removal
> Academic: Conflict resolution theory (Fisher & Ury 1981); moral psychology (Haidt 2012)

```yaml
effects:
  - system: skill
    target: [negotiation, diplomacy, mediation]
    op: mult
    value: 1.5

  - system: behavior
    target: biased_mediation
    op: block

  - system: derived
    target: trustworthiness
    op: mult
    value: 1.4

  - system: on_event
    target: faction_conflict
    op: inject
    value:
      behavior: broker_peace
      priority: 0.85

  - system: aura
    target: inter_faction_tension
    op: mult
    value: 0.7
    radius: 4

  - system: on_event
    target: biased_mediation_given
    op: inject
    value:
      behavior: moral_reckoning
      side_effects:
        - target: allostatic_load
          op: add
          value: 0.5
        - target: trait_betrayal_counter
          op: add
          value: 1

  - system: derived
    target: wisdom
    op: mult
    value: 1.3
```

---

### #19 M_edge_walker
> **Edge-Walker / 경계를 걷는 자** — Lives where the rules stop being clear; finds that comfortable
> Acquisition: combat ≥ 15 AND espionage ≥ 15 | Rarity: epic
> Loss: decay 5y (craft_neglect_stress source)
> Academic: Dual-domain expertise; liminal identity theory (Turner 1969)

```yaml
effects:
  - system: skill
    target: [combat_skills, espionage, stealth, tracking]
    op: mult
    value: 1.3

  - system: behavior
    target: threat_assess_before_act
    op: inject
    value:
      priority: 0.7

  - system: combat
    target: ambush_detection
    op: mult
    value: 1.5

  - system: stress
    target: surveillance_stress
    op: immune

  - system: behavior
    target: operate_in_gray_area
    op: inject
    value:
      priority: 0.5

  - system: stress
    target: certainty_requirement_stress
    op: immune

  - system: need
    target: autonomy.fulfillment_rate
    op: mult
    value: 1.3

  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.2
```

---

### #20 M_word_carver
> **Word-Carver / 글자를 새기는 자** — Their words outlive them; they know it and write accordingly
> Acquisition: reading ≥ 18 AND poetry ≥ 15 | Rarity: legendary
> Loss: decay 3y (craft_neglect_stress source)
> Academic: Oral-to-literate transition theory (Ong 1982); cultural memory studies (Assmann 1995)

```yaml
effects:
  - system: skill
    target: [reading, writing, poetry, history, law]
    op: mult
    value: 1.5

  - system: behavior
    target: record_important_events
    op: inject
    value:
      priority: 0.6

  - system: on_event
    target: entity_death
    op: inject
    value:
      behavior: compose_permanent_written_legacy
      condition:
        source: entity
        key: has_trait
        op: eq
        value: M_word_carver
      side_effects:
        - target: collective_memory
          op: add
          value: permanent_legacy_entry

  - system: settlement
    target: collective_memory_preservation
    op: mult
    value: 1.4
    condition:
      source: entity
      key: is_active_resident
      op: eq
      value: true

  - system: derived
    target: wisdom
    op: mult
    value: 1.35

  - system: need
    target: meaning.fulfillment_rate
    op: mult
    value: 1.5

  - system: on_event
    target: witnesses_important_event
    op: inject
    value:
      behavior: write_it_down
      priority: 0.75

  - system: stress
    target: craft_neglect_stress
    op: mult
    value: 1.3
```

---

## Bond Traits (D_) — Relational History Written Into the Self

> Bond traits are acquired through specific relational events with other agents. They are the
> relationship layer made permanent: deep intimacy becomes soul-tethering, shared combat becomes
> blood oath, betrayal becomes eternal grudge. Many have loss conditions tied to the target agent.
> Reference: Bowlby attachment theory, Hazan & Shaver (1987) adult attachment, social identity theory.

---

### #1 D_soul_tethered
> **Soul-Tethered / 영혼결박** — When two lives grow so intertwined that separation becomes a kind of dying
> Acquisition: intimacy ≥ 95 sustained for 10 years | Rarity: legendary
> Loss: converts to W_widows_frost on target death
> Academic: Bowlby attachment theory; Hazan & Shaver (1987) adult attachment styles

```yaml
effects:
  - system: relationship
    target: target_intimacy_ceil
    op: max
    value: 100
    # No ceiling exists for this person — the bond grows without limit

  - system: emotion
    target: target_bleed_sensitivity
    op: set
    value: 0.3
    # Target's emotional state partially bleeds into self; their joy is felt, their pain is shared

  - system: need
    target: belonging.fulfillment_source
    op: set
    value: target_presence
    # This person IS home — their presence alone resolves the belonging need

  - system: derived
    target: wisdom
    op: mult
    value: 1.2
    condition:
      source: target
      key: alive
      op: eq
      value: true
    # Seeing through bonded eyes deepens understanding of the world

  - system: stress
    target: target_absence_stress
    op: mult
    value: 1.8
    # Separation is not loneliness — it is incompleteness

  - system: stress
    target: loneliness_while_bonded
    op: immune
    # Never truly alone while target lives

  - system: on_event
    target: target_in_danger
    op: inject
    value:
      behavior: panic_override
      priority: 0.95
      description: Overrides all other behavior when bonded target is threatened

  - system: on_event
    target: target_death
    op: inject
    value:
      behavior: trait_converts_to
      converts_to: W_widows_frost
      secondary_event: catastrophic_grief_event
      description: The bond does not break — it inverts. Soul-Tethered becomes Widow's Frost.
```

---

### #2 D_blood_oath
> **Blood Oath / 피의 맹세** — Forged in shared survival; the kind of loyalty that makes death feel like betrayal
> Acquisition: survived 3 combats alongside the same agent | Rarity: epic
> Loss: permanent — cannot be revoked
> Academic: Military unit cohesion research (Shils & Janowitz 1948); band-of-brothers effect

```yaml
effects:
  - system: combat
    target: damage_mult
    op: mult
    value: 1.2
    condition:
      source: target
      key: in_same_battle
      op: eq
      value: true
    # Fighting beside the oath-brother sharpens every strike

  - system: combat
    target: morale
    op: min
    value: 0.4
    condition:
      source: target
      key: alive_in_battle
      op: eq
      value: true
    # Will not break while the oath-brother still stands

  - system: relationship
    target: target_betrayal_possible
    op: set
    value: false
    # This person cannot be betrayed — the blood oath forecloses it

  - system: stress
    target: target_death_stress
    op: mult
    value: 2.0
    # Their death is not just grief — it is the loss of a self-extension

  - system: on_event
    target: target_in_danger
    op: inject
    value:
      behavior: protect_oath_brother
      priority: 0.9
      description: Drops current task to defend the oath-brother when threatened
```

---

### #3 D_eternal_grudge
> **Eternal Grudge / 영원한 원한** — The wound that never closes; hatred as a second identity
> Acquisition: intimacy falls to ≤ −80 after a significant harm event | Rarity: epic
> Loss: resolves on successful revenge OR target death; closure event fires either way
> Academic: Revenge motivation (McCullough et al. 2001); moral injury (Litz 2009)

```yaml
effects:
  - system: behavior
    target: harm_grudge_target
    op: inject
    value:
      priority: 0.6
      description: Active drive to damage, undermine, or destroy the grudge target

  - system: behavior
    target: forgive_grudge_target
    op: block
    # Forgiveness is not inaccessible — it is structurally impossible while this trait persists

  - system: stress
    target: target_thriving_stress
    op: mult
    value: 1.8
    # Watching the grudge target succeed is experienced as a repeated wound

  - system: skill
    target: tracking
    op: mult
    value: 1.3
    condition:
      source: target
      key: is_grudge_target
      op: eq
      value: true

  - system: skill
    target: espionage
    op: mult
    value: 1.3
    condition:
      source: target
      key: is_grudge_target
      op: eq
      value: true
    # Hatred focuses the mind toward the object of hate

  - system: on_event
    target: grudge_resolved
    op: inject
    value:
      secondary_event: closure_event
      behavior: reflect_on_cost_of_hate
      description: Resolution does not mean peace — it means looking at what the hate cost you

  - system: on_event
    target: target_dies_without_revenge
    op: inject
    value:
      event: hollow_victory_event
      description: Death without confrontation leaves the wound unaddressed; grief and rage with nowhere to go
```

---

### #4 D_shepherds_heart
> **Shepherd's Heart / 목자의 마음** — Those who have raised children see the world differently; they carry futures in their hands
> Acquisition: successfully raised 5 children to adulthood | Rarity: epic
> Loss: permanent
> Academic: Parental investment theory (Trivers 1972); Erikson's generativity vs. stagnation

```yaml
effects:
  - system: behavior
    target: protect_young
    op: inject
    value:
      priority: 0.8
      description: Automatically prioritizes protecting children and young agents in range

  - system: behavior
    target: teach_young
    op: inject
    value:
      priority: 0.5
      description: Seeks opportunities to pass on knowledge and skills to younger agents

  - system: skill
    target: childcare
    op: mult
    value: 1.3

  - system: skill
    target: teaching
    op: mult
    value: 1.3

  - system: skill
    target: medicine_basic
    op: mult
    value: 1.3
    # Years of tending small bodies builds a particular kind of practical care

  - system: need
    target: meaning.fulfillment_rate
    op: mult
    value: 1.4
    # The work of raising others fills the meaning-need more efficiently

  - system: stress
    target: child_in_danger_stress
    op: mult
    value: 2.0

  - system: derived
    target: trustworthiness
    op: mult
    value: 1.15
    # Those who shepherd others are read as reliably safe
```

---

### #5 D_twice_betrayed
> **Twice-Betrayed / 두 번 버림받은 자** — The wall is not cruelty; it is the only architecture that survived
> Acquisition: betrayed twice by agents with intimacy ≥ 60 at time of betrayal | Rarity: epic
> Loss: heals after 10 years of secure attachment without betrayal
> Academic: Betrayal trauma theory (Freyd 1996); anxious-avoidant attachment (Bowlby)

```yaml
effects:
  - system: relationship
    target: intimacy_ceil
    op: max
    value: 70
    # The wall is load-bearing — intimacy cannot pass this threshold while the trait persists

  - system: relationship
    target: intimacy_gain_rate
    op: mult
    value: 0.5
    # Trust accumulates slowly; the mechanism for closeness is damaged, not destroyed

  - system: behavior
    target: betray_detection_scan
    op: inject
    value:
      priority: 0.6
      description: Continuously monitors social signals for early warning of betrayal; interprets ambiguity as threat

  - system: stress
    target: unexpected_kindness_stress
    op: mult
    value: 1.3
    # Kindness from others reads as a setup; the nervous system no longer trusts warmth

  - system: need
    target: belonging.sensitivity
    op: mult
    value: 1.4
    # The need for connection did not disappear — it intensified. The wound and the longing are the same wound.

  - system: on_event
    target: trait_heals
    op: inject
    value:
      behavior: vulnerability_attempt
      description: After healing, they try again — tentatively, watching for exits
```

---

### #6 D_pack_alpha
> **Pack Alpha / 무리의 우두머리** — Power held through others; the identity that depends on their faces looking back
> Acquisition: 10 agents with highest-tier trust orientation toward self | Rarity: epic
> Loss: follower count drops below 5
> Academic: Social identity theory (Tajfel & Turner 1979); leadership emergence research

```yaml
effects:
  - system: aura
    target: follower_morale
    op: add
    value: 0.08
    radius: 4
    target_filter: trusted_agents
    # Presence alone lifts those who follow

  - system: behavior
    target: protect_follower
    op: inject
    value:
      priority: 0.7
      description: Actively moves to shield followers from harm; identity is bound to their survival

  - system: derived
    target: charisma
    op: mult
    value: 1.25

  - system: derived
    target: intimidation
    op: mult
    value: 1.15
    # The alpha communicates through presence — both warmth and weight

  - system: stress
    target: follower_loss_stress
    op: mult
    value: 1.5
    # Each lost follower is a piece of the self that walked away

  - system: on_event
    target: follower_count_drops_below_5
    op: inject
    value:
      event: identity_crisis_stress_event
      behavior: trait_lost
      description: Without the pack, the alpha is structurally undefined; the identity collapses with the role
```

---

### #7 D_lone_wolf
> **Lone Wolf / 외로운 늑대** — Solitude chosen so many times it stopped feeling like a choice
> Acquisition: 5 consecutive years without any close relationship (intimacy ≥ 40) | Rarity: rare
> Loss: intimacy ≥ 60 formed and sustained
> Academic: Avoidant attachment style (Bowlby); self-sufficiency as defensive structure

```yaml
effects:
  - system: stress
    target: isolation_stress
    op: immune
    # Aloneness is no longer experienced as deprivation; it has become baseline safety

  - system: skill
    target: survival
    op: mult
    value: 1.25
    # Years of solo resource management compound into genuine competence

  - system: skill
    target: self_reliance_composite
    op: mult
    value: 1.25

  - system: derived
    target: risk_tolerance
    op: mult
    value: 1.2
    # With no one depending on you, personal risk calculus shifts

  - system: relationship
    target: intimacy_gain_rate
    op: mult
    value: 0.6
    # The machinery for closeness has grown rusty; connection is possible but slow

  - system: need
    target: belonging.sensitivity
    op: mult
    value: 1.3
    # The need was never gone — it was buried. It still aches, quietly, at the edges.

  - system: on_event
    target: first_genuine_intimacy_after_trait_loss
    op: inject
    value:
      event: vulnerability_stress_event
      description: The first real closeness after years alone triggers acute disorientation and fear
```

---

### #8 D_unrequited
> **Unreturned Vow / 돌아오지 않은 맹세** — To love without answer; the light kept on in an empty window
> Acquisition: self love ≥ 80 toward target, unreciprocated for 3 continuous years | Rarity: rare
> Loss: decays naturally after 5 years without reciprocation
> Academic: Unrequited love psychology (Baumeister & Wotman 1992); Tennov's limerence theory

```yaml
effects:
  - system: emotion
    target: joy.max
    op: max
    value: 0.75
    # Joy is available but capped — the absence of this one thing shadows all other happiness

  - system: behavior
    target: target_proximity_seeking
    op: inject
    value:
      priority: 0.4
      description: Gravitates toward the target's presence without fully understanding the pull

  - system: stress
    target: target_with_others_stress
    op: mult
    value: 1.6
    # Seeing the target choose someone else reopens the wound with each occurrence

  - system: need
    target: romance.sensitivity
    op: mult
    value: 1.5
    # Unrequited love doesn't kill the longing — it amplifies it; the wound keeps the desire alive

  - system: on_event
    target: five_year_decay_end
    op: inject
    value:
      event: relief_event
      behavior: move_on_behavior
      description: The decay end is not healing exactly — it is the exhaustion of hope, which feels like peace
```

---

### #9 D_kingmaker
> **Kingmaker / 옹립자** — Power exercised through elevation of others; the throne has never been the point
> Acquisition: successfully supported 3 leaders who achieved and held significant power | Rarity: legendary
> Loss: all supported leaders fall or fail
> Academic: Political influence dynamics; behind-the-throne power analysis

```yaml
effects:
  - system: skill
    target: diplomacy
    op: mult
    value: 1.35

  - system: skill
    target: negotiation
    op: mult
    value: 1.35

  - system: skill
    target: politics
    op: mult
    value: 1.35

  - system: skill
    target: persuasion
    op: mult
    value: 1.35
    # Three successful kingmakings represent deep mastery of social leverage

  - system: derived
    target: wisdom
    op: mult
    value: 1.25
    # Those who raise others to power see patterns in human ambition others miss

  - system: behavior
    target: identify_strong_candidate
    op: inject
    value:
      priority: 0.5
      description: Perpetually scouts the social landscape for the next worthy leader to back

  - system: behavior
    target: seek_own_leadership
    op: block
    # The kingmaker does not want the crown — this is not humility but structural self-definition

  - system: aura
    target: political_network_access
    op: mult
    value: 1.3
    radius: 6
    target_filter: political_actors

  - system: stress
    target: supported_leader_failing_stress
    op: mult
    value: 1.8
    # Their failure is experienced as personal failure — the identity is co-extensive with their success

  - system: on_event
    target: all_supported_leaders_fail
    op: inject
    value:
      event: identity_crisis_event
      behavior: trait_lost
      description: Without a leader to uphold, the kingmaker has no frame for the self; the role collapses
```

---

### #10 D_mirror_bond
> **Mirror Bond / 거울의 유대** — To find yourself reflected in another; the self made legible through shared shape
> Acquisition: shares 3+ active traits with target AND intimacy ≥ 85 | Rarity: legendary
> Loss: target death OR 5 years of continuous separation
> Academic: Self-other overlap theory (Aron et al. 1991); self-expansion model of intimacy

```yaml
effects:
  - system: relationship
    target: target_emotional_resonance
    op: set
    value: 0.5
    # Emotional states partially synchronize — not merger, but harmonic overlap

  - system: skill
    target: shared_skill_domains
    op: mult
    value: 1.2
    # Skills held in common sharpen through the tacit dialogue of shared practice

  - system: stress
    target: disagreement_with_target_stress
    op: mult
    value: 0.3
    # Friction with the mirror-bonded other is far less threatening than with anyone else

  - system: stress
    target: target_suffering_stress
    op: mult
    value: 2.0
    # Their pain arrives as one's own; the resonance cuts both ways

  - system: derived
    target: wisdom
    op: mult
    value: 1.2
    # Knowing another so well — and being known — builds a particular kind of understanding

  - system: on_event
    target: target_death
    op: inject
    value:
      event: catastrophic_mirror_shatter_event
      secondary: extended_grief_sequence
      description: The mirror breaks. What was visible only through them becomes inaccessible; a part of the self goes dark.

  - system: on_event
    target: five_year_separation
    op: inject
    value:
      event: fading_grief_event
      behavior: trait_loss
      description: Distance slowly dissolves the resonance; the bond fades like a reflected image losing its source
```

---

### #11 D_debt_of_life
> **Debt of Life / 목숨의 빚** — A life owed cannot be forgotten; it reshapes every choice
> Acquisition: saved from lethal danger by a specific agent | Rarity: epic
> Loss: resolved when debt repaid by saving the benefactor in kind
> Academic: Reciprocity theory (Gouldner 1960), honor systems and moral debt literature

```yaml
effects:
  - system: behavior
    target: target_protection
    op: inject
    value:
      priority: 0.8
      description: "Prioritizes safety of the one who saved them"

  - system: relationship
    target: life_saver.trust_floor
    op: set
    value: 0.6
    # Base trust in life-saver cannot fall below this floor

  - system: behavior
    target: betray_life_saver
    op: block
    # Cannot act against the one they owe their life to

  - system: stress
    target: life_saver_in_danger
    op: mult
    value: 1.8
    # Extreme distress when benefactor is threatened

  - system: on_event
    target: debt_repaid
    op: inject
    value:
      emit: [trait_resolved, relationship_deepened]
      description: "Debt discharged; bond transforms from obligation to chosen loyalty"
```

---

### #12 D_bitter_mentor
> **Bitter Mentor / 쓰라린 스승** — Pride and envy braided together; the student became the lesson
> Acquisition: student surpasses master in shared skill domain | Rarity: epic
> Loss: permanent
> Academic: Erikson's generativity vs. stagnation (1950), mentor-protege rivalry dynamics

```yaml
effects:
  - system: emotion
    target: pride_envy_coactive
    op: inject
    value:
      pride_weight: 0.5
      envy_weight: 0.5
      description: "Simultaneous pride and bitterness; cannot fully resolve into either"

  - system: skill
    target: teaching
    op: mult
    value: 1.3
    # Still teaches — it is all they know how to give

  - system: behavior
    target: acknowledge_student_superiority_aloud
    op: block
    # Cannot say it out loud, even when everyone knows

  - system: stress
    target: student_public_success
    op: set
    value: 0.1
    # Low-grade ambient ache whenever the student is praised

  - system: derived
    target: wisdom
    op: mult
    value: 1.2
    # Being surpassed is its own kind of teaching
```

---

### #13 D_orphans_resolve
> **Orphan's Resolve / 고아의 결의** — Learned to stand alone before learning to walk far
> Acquisition: both parents died before the agent reached age 15 | Rarity: rare
> Loss: permanent
> Academic: Childhood bereavement studies (Worden 2009), resilience theory (Rutter 1987)

```yaml
effects:
  - system: stress
    target: isolation_stress
    op: mult
    value: 0.6
    # gift — learned to be alone early; solitude does not sting as sharply

  - system: derived
    target: risk_tolerance
    op: mult
    value: 1.2
    # gift — when you have lost the most important things, lesser risks feel manageable

  - system: behavior
    target: self_reliance
    op: inject
    value:
      priority: 0.5
      description: "Defaults to solving problems alone before seeking help"
    # gift

  - system: need
    target: belonging.sensitivity
    op: mult
    value: 1.4
    # cost — the wound that never closes; hungers for belonging precisely because it was taken

  - system: relationship
    target: parental_figure_attachment_intensity
    op: mult
    value: 1.6
    # cost — latches hard onto anyone who takes a parental role; the need is disproportionate
```

---

### #14 D_last_of_line
> **Last of the Line / 마지막 혈족** — When the name dies with you, every day is an ending
> Acquisition: all direct kin confirmed dead | Rarity: epic
> Loss: converts to null on birth of own child
> Academic: Existential psychology (Yalom 1980), lineage and legacy psychology

```yaml
effects:
  - system: emotion
    target: joy.max
    op: max
    value: 0.7
    # The weight of being the last mutes peak happiness

  - system: behavior
    target: preserve_lineage_memory
    op: inject
    value:
      priority: 0.6
      description: "Driven to record, tell, or monument the names of those who came before"

  - system: stress
    target: lineage_extinction
    op: set
    value: 0.15
    # Ambient existential dread; the silence where family used to be

  - system: behavior
    target: take_suicidal_risks
    op: block
    condition:
      source: self
      key: has_living_kin
      op: eq
      value: false
    # The line must not end here; death feels like erasure of everyone

  - system: on_event
    target: child_born
    op: inject
    value:
      emit: [trait_converts_to_null, cathartic_joy]
      description: "The line continues; grief unlocks into relief and overwhelming joy"
```

---

### #15 D_forged_family
> **Forged Family / 만들어진 가족** — Blood is one kind of bond; chosen loyalty is another, and sometimes fiercer
> Acquisition: 5 or more non-kin agents each at intimacy ≥ 85 | Rarity: epic
> Loss: lost when 3 or more chosen members die or permanently depart
> Academic: Fictive kinship studies (Stack 1974), chosen family research (Weston 1991)

```yaml
effects:
  - system: behavior
    target: protect_chosen_family
    op: inject
    value:
      priority: 0.8
      description: "Defends chosen family with the same urgency as blood kin"

  - system: need
    target: belonging.fulfillment_rate
    op: mult
    value: 1.5
    # Chosen family satisfies belonging more efficiently than strangers; sometimes more than blood

  - system: stress
    target: chosen_family_member_loss
    op: mult
    value: 2.0
    # Each loss hits harder because each member was a deliberate choice

  - system: aura
    target: belonging_need_fulfillment
    op: mult
    value: 1.2
    radius: intimate
    target_filter: chosen_family_members
    # Presence of this agent meaningfully fills the belonging need of those they have chosen

  - system: on_event
    target: three_members_lost
    op: inject
    value:
      emit: [trait_lost, grief_cascade]
      description: "The family has collapsed below the threshold that made it real"
```

---

### #16 D_cursed_lover
> **Cursed Lover / 저주받은 연인** — Three times love; three times funeral; the pattern has become superstition
> Acquisition: 3 romantic partners died while in active relationship | Rarity: epic
> Loss: permanent
> Academic: Complicated grief (Shear 2015), superstitious conditioning and learned helplessness

```yaml
effects:
  - system: stress
    target: new_romantic_attachment_forming
    op: mult
    value: 1.5
    # Loving again feels like signing a death warrant for the beloved

  - system: behavior
    target: form_romantic_attachment
    op: block
    condition:
      source: self
      key: superstition_value
      op: gte
      value: 0.4
    # High-superstition agents refuse to love again; they believe they are the cause

  - system: emotion
    target: joy.max
    op: max
    value: 0.72
    # The shadow of past losses caps happiness; grief has calcified into the personality

  - system: stress
    target: current_partner_illness_or_injury
    op: mult
    value: 2.5
    # Every cough sounds like the beginning of the pattern repeating

  - system: derived
    target: wisdom
    op: mult
    value: 1.2
    # gift — grief-earned; they understand impermanence and love's cost with unusual depth

  - system: behavior
    target: cherish_present_moment
    op: inject
    value:
      priority: 0.4
      description: "Attentive to the present; knows it can be taken without warning"
    # gift
```

---

### #17 D_sworn_enemy
> **Sworn Enemy / 맹세의 원수** — The hatred was given a name, and the name became a compass
> Acquisition: mutual intimacy ≤ −90 with sworn declaration between both agents | Rarity: epic
> Loss: converts on enemy's death, leaving hollow emptiness
> Academic: Enemy psychology (Holt 1989), revenge and purpose literature (Stuckless & Goranson 1992)

```yaml
effects:
  - system: behavior
    target: monitor_enemy_status
    op: inject
    value:
      priority: 0.6
      description: "Regularly checks on enemy's location, health, and actions"

  - system: combat
    target: damage_mult
    op: mult
    value: 1.3
    condition:
      source: target
      key: is_sworn_enemy
      op: eq
      value: true
    # Fights with exceptional ferocity against this specific agent

  - system: behavior
    target: forgive_or_ignore_enemy
    op: block
    # Cannot set the hatred aside; the oath is structural

  - system: stress
    target: enemy_thriving
    op: mult
    value: 1.5
    # Enemy's success is experienced as personal injury

  - system: need
    target: meaning.fulfillment_source
    op: add
    value: defeating_enemy
    # The enemy gives life direction; the hatred is also a purpose

  - system: on_event
    target: enemy_dies
    op: inject
    value:
      emit: [hollow_emptiness, identity_disruption]
      description: "The organizing purpose of hatred is gone; what remains when the enemy is silent?"
```

---

### #18 D_foster_bond
> **Foster Bond / 양육의 끈** — Not of my blood, but raised by my hands; the bond is no less real
> Acquisition: raised another agent's child for 3 or more continuous years | Rarity: rare
> Loss: permanent
> Academic: Alloparenting research (Hrdy 2009), foster care attachment studies (Dozier et al. 2009)

```yaml
effects:
  - system: behavior
    target: protect_foster_child
    op: inject
    value:
      priority: 0.8
      description: "Treats the foster child's safety as a primary behavioral imperative"

  - system: skill
    target: childcare
    op: mult
    value: 1.2
    # Years of practice have refined these skills

  - system: skill
    target: teaching
    op: mult
    value: 1.2
    # Raising a child across years builds patient instruction capacity

  - system: need
    target: meaning.fulfillment_rate
    op: mult
    value: 1.3
    # Watching another's child grow is a deep source of meaning

  - system: relationship
    target: foster_child.intimacy_ceil
    op: max
    value: 95
    # The ceiling of closeness approaches that of birth-kin; nearly as deep
```

---

### #19 D_river_between
> **River Between / 사이의 강** — Loved on both banks; unable to cross to either without leaving the other behind
> Acquisition: intimacy ≥ 75 with agents from two mutually hostile factions | Rarity: epic
> Loss: lost when one faction collapses or the agent is formally expelled from either
> Academic: Cross-cutting cleavages theory (Lipset 1960), bridging social capital (Putnam 2000)

```yaml
effects:
  - system: skill
    target: diplomacy
    op: mult
    value: 1.3
    # gift — navigating between hostile parties builds real diplomatic fluency

  - system: skill
    target: negotiation
    op: mult
    value: 1.3

  - system: skill
    target: mediation
    op: mult
    value: 1.3

  - system: stress
    target: faction_conflict_escalation
    op: mult
    value: 1.6
    # Caught in the middle; every escalation is a personal crisis

  - system: behavior
    target: pick_a_side_permanently
    op: block
    # Cannot abandon one bond to fully commit to the other

  - system: behavior
    target: relay_mutual_information
    op: inject
    value:
      priority: 0.4
      description: "Naturally acts as informal channel between the two factions"
    # gift — the river flows both ways

  - system: on_event
    target: forced_to_choose_faction
    op: inject
    value:
      emit: [identity_crisis]
      description: "Being made to choose violates the core of who they are between these people"

  - system: on_event
    target: one_faction_collapses
    op: inject
    value:
      emit: [grief, trait_lost]
      description: "One bank has washed away; the river no longer has two sides to hold"
```

---

### #20 D_chain_of_grief
> **Chain of Grief / 슬픔의 사슬** — Every new loss arrives with the weight of all the ones before it
> Acquisition: 3 or more agents at intimacy ≥ 70 have died | Rarity: epic
> Loss: permanent
> Academic: Cumulative grief theory (Rando 1993), bereavement overload (Kastenbaum 1969)

```yaml
effects:
  - system: stress
    target: new_loss_event
    op: mult
    value: 1.5
    # Each death echoes every prior death; grief is not experienced in isolation

  - system: emotion
    target: joy.max
    op: max
    value: 0.75
    # Accumulated loss places a ceiling on peak happiness; something is always slightly muted

  - system: stress
    target: grief_spiral_risk
    op: mult
    value: 1.3
    # Vulnerable to cascading grief if multiple losses occur in close succession

  - system: skill
    target: grief_counseling
    op: mult
    value: 1.4
    # gift — intimate knowledge of grief makes them unusually capable with the bereaved

  - system: skill
    target: comforting_bereaved
    op: mult
    value: 1.4

  - system: behavior
    target: sit_with_grieving
    op: inject
    value:
      priority: 0.5
      description: "Naturally drawn to those in mourning; knows how to be present without fixing"
    # gift

  - system: stress
    target: witnessing_others_loss
    op: mult
    value: 0.6
    # Has processed enough grief to hold space for others without being destroyed by it
```
