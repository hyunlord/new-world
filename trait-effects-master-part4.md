# WorldSim Trait Effects — Part 4 (Corpus #1~#12, Nous #1~#10)
> Canonical YAML effect definitions using Trait System v3 schema.
> Ops: set / add / mult / min / max / disable / enable / block / inject / override / on_event / tag / immune / replace
> All conditions use structured format: {source, key, op, value}
>
> Part 1: Archetype #1~#30 + Shadow #1~#5 ✅
> Part 2: Archetype #31~#55 ✅
> Part 3: Shadow #6~#15 + Radiance #1~#12 ✅
> **Part 4: Corpus #1~#12 + Nous #1~#10 ← this file**
> Part 5: Awakened + Bloodline
> Part 6: Mastery + Bond
> Part 7: Fate + Synergy

---

## Corpus Traits (B_) — Physical Extremes

> Body stat axes: str (Strength), agi (Agility), end (Endurance), tou (Toughness), rec (Recuperation), dr (Disease Resistance).
> Corpus traits are genetic/biological — their effects map to combat, labor, survival, injury, and aging curves.
> Academic grounding: physical anthropology, Gompertz-Makeham mortality model, sports science injury/recovery literature.

---

### #1 B_titan
> **Titan / 거인** — Built like a fortress — overwhelming physical force with diminished fine-motor precision.
> Acquisition: str↑↑ tou↑ | Rarity: epic
> Academic: Ikai & Fukunaga (1968) muscle cross-section vs force; Zatsiorsky (2006) biomechanics of strength

```yaml
- system: combat
  target: damage_mult
  op: mult
  value: 1.5
  tags: [strength, physical_dominance]

- system: combat
  target: wound_threshold
  op: mult
  value: 1.3
  tags: [toughness]

- system: skill
  target: [construction, mining, logging, smithing]
  op: mult
  value: 1.35
  tags: [heavy_labor]

- system: skill
  target: [fine_crafting, surgery, calligraphy]
  op: mult
  value: 0.7
  tags: [precision_penalty]

- system: aging
  target: str_decline_rate
  op: mult
  value: 0.75
  tags: [slow_aging]

- system: aging
  target: peak_end_age
  op: add
  value: 5
  tags: [extended_prime]
```

---

### #2 B_wraith
> **Wraith / 망령** — Ghost-fast, paper-fragile — untouchable until hit.
> Acquisition: agi↑↑ str↓ | Rarity: epic
> Academic: Schmidt & Lee (2011) motor learning and agility; Plagenhoef (1971) biomechanics of speed vs mass tradeoff

```yaml
- system: combat
  target: dodge_mult
  op: mult
  value: 1.8
  tags: [agility, evasion]

- system: combat
  target: damage_mult
  op: mult
  value: 0.65
  tags: [low_strength]

- system: body
  target: movement_speed
  op: mult
  value: 1.4
  tags: [speed]

- system: skill
  target: [stealth, pickpocket, archery, surgery]
  op: mult
  value: 1.4
  tags: [precision, dexterity]

- system: skill
  target: [construction, mining, smithing]
  op: mult
  value: 0.6
  tags: [heavy_labor_penalty]

- system: behavior
  target: heavy_labor
  op: block
  value: true
  tags: [physical_limitation]

- system: combat
  target: morale_on_hit
  op: on_event
  value:
    trigger: receives_damage
    effect: fear_spike
    intensity: 0.4
  tags: [fragility_fear]
```

---

### #3 B_mountain_body
> **Mountain Body / 산 같은 몸** — Indestructible endurance — outlasts everything.
> Acquisition: tou↑↑ end↑↑ | Rarity: epic
> Academic: Gompertz (1825) mortality curves; Noakes (2012) central governor model of endurance

```yaml
- system: combat
  target: wound_severity_mult
  op: mult
  value: 0.5
  tags: [toughness, injury_resistance]

- system: body
  target: fatigue_rate
  op: mult
  value: 0.4
  tags: [endurance]

- system: skill
  target: [long_distance_travel, endurance_labor]
  op: mult
  value: 1.5
  tags: [stamina]

- system: aging
  target: physical_decline_rate
  op: mult
  value: 0.7
  tags: [slow_aging]

- system: stress
  target: physical_hardship_stress
  op: immune
  value: true
  tags: [hardship_immunity]

- system: behavior
  target: rest_required_threshold
  op: mult
  value: 0.3
  tags: [minimal_rest]
```

---

### #4 B_phoenix_blood
> **Phoenix Blood / 불사조의 피** — Recovers from anything — illness bounces off.
> Acquisition: rec↑↑ dr↑↑ | Rarity: epic
> Academic: Eming et al. (2014) wound repair mechanisms; Casanova & Abel (2018) immunological resilience

```yaml
- system: body
  target: healing_speed
  op: mult
  value: 3.0
  tags: [recuperation]

- system: body
  target: disease_infection_chance
  op: mult
  value: 0.2
  tags: [disease_resistance]

- system: body
  target: chronic_condition_risk
  op: mult
  value: 0.15
  tags: [disease_resistance]

- system: aging
  target: disease_mortality_curve
  op: mult
  value: 0.3
  tags: [longevity]

- system: stress
  target: injury_stress
  op: mult
  value: 0.5
  tags: [quick_recovery]

- system: body
  target: recovery_on_infection
  op: on_event
  value:
    trigger: infected_with_disease
    effect: recovery_time_halved
  tags: [phoenix_recovery]
```

---

### #5 B_glass_cannon
> **Glass Cannon / 유리대포** — Maximum damage, zero defense.
> Acquisition: str↑↑ tou↓↓ | Rarity: epic
> Academic: Newton's third law applied to combat biomechanics; McArdle et al. (2010) exercise physiology — power-mass-fragility tradeoff

```yaml
- system: combat
  target: damage_mult
  op: mult
  value: 1.8
  tags: [strength, devastating_force]

- system: combat
  target: wound_threshold
  op: mult
  value: 0.4
  tags: [fragility]

- system: body
  target: injury_severity_mult
  op: mult
  value: 2.0
  tags: [fragility]

- system: aging
  target: str_decline_rate
  op: mult
  value: 1.3
  tags: [fast_aging]

- system: combat
  target: wound_amplification
  op: on_event
  value:
    trigger: receives_wound
    effect: wound_severity_amplified
    mult: 1.5
  tags: [glass_body]

- system: behavior
  target: reckless_attack
  op: inject
  value:
    priority: 0.6
  tags: [all_or_nothing]
```

---

### #6 B_iron_lungs
> **Iron Lungs / 철의 폐** — Tireless — works and travels without fatigue.
> Acquisition: end↑↑ | Rarity: rare
> Academic: Bassett & Howley (2000) maximal oxygen uptake; Coyle (1995) endurance physiology

```yaml
- system: body
  target: fatigue_rate
  op: mult
  value: 0.3
  tags: [endurance]

- system: skill
  target: [long_distance_travel, farming, mining, logging]
  op: mult
  value: 1.3
  tags: [stamina_labor]

- system: stress
  target: physical_labor_stress
  op: immune
  value: true
  tags: [tireless]

- system: behavior
  target: rest_breaks
  op: mult
  value: 0.4
  tags: [minimal_rest]
```

---

### #7 B_paper_skin
> **Paper Skin / 종이 피부** — Breaks easily, gets sick easily.
> Acquisition: tou↓↓ dr↓↓ | Rarity: epic
> Academic: Gompertz-Makeham law of accelerated mortality; Casanova (2015) human genetic immunodeficiency

```yaml
- system: combat
  target: wound_severity_mult
  op: mult
  value: 2.5
  tags: [fragility]

- system: body
  target: disease_infection_chance
  op: mult
  value: 3.0
  tags: [immunodeficiency]

- system: body
  target: chronic_condition_risk
  op: mult
  value: 2.5
  tags: [chronic_vulnerability]

- system: aging
  target: physical_decline_onset
  op: add
  value: -5
  tags: [early_aging]

- system: combat
  target: permanent_injury_risk
  op: on_event
  value:
    trigger: wounded_in_combat
    effect: high_probability_permanent_injury
    chance: 0.4
  tags: [permanent_damage]
```

---

### #8 B_perfect_form
> **Peerless Form / 완전체** — Flawless movement, quick recovery.
> Acquisition: agi↑↑ rec↑ | Rarity: epic
> Academic: Bernstein (1967) coordination and regulation of movement; Enoka (2015) neuromechanics of human movement

```yaml
- system: combat
  target: dodge_mult
  op: mult
  value: 1.5
  tags: [agility, grace]

- system: skill
  target: [surgery, archery, acrobatics, dance]
  op: mult
  value: 1.4
  tags: [precision_movement]

- system: body
  target: healing_speed
  op: mult
  value: 1.8
  tags: [recuperation]

- system: derived
  target: allure
  op: mult
  value: 1.2
  tags: [physical_grace]

- system: body
  target: movement_speed
  op: mult
  value: 1.25
  tags: [speed]
```

---

### #9 B_withered
> **Witherborn / 쇠잔한 자** — Born diminished — exhausts quickly, struggles with labor, but redirects capability.
> Acquisition: str↓↓ end↓↓ | Rarity: epic
> Academic: Sarcopenia literature — Cruz-Jentoft et al. (2019); compensatory cognitive development in physically limited populations

```yaml
- system: combat
  target: damage_mult
  op: mult
  value: 0.5
  tags: [weakness]

- system: body
  target: fatigue_rate
  op: mult
  value: 2.5
  tags: [low_endurance]

- system: skill
  target: [heavy_labor, construction, mining]
  op: mult
  value: 0.45
  tags: [physical_limitation]

- system: aging
  target: physical_decline_rate
  op: mult
  value: 1.5
  tags: [fast_aging]

- system: stress
  target: physical_hardship_stress
  op: mult
  value: 2.0
  tags: [hardship_vulnerability]

- system: need
  target: safety_need_sensitivity
  op: mult
  value: 1.4
  tags: [vulnerability_awareness]

# Compensatory: redirected capability — not helpless
- system: skill
  target: [teaching, diplomacy, writing]
  op: mult
  value: 1.1
  tags: [compensatory]
```

---

### #10 B_cat_eyes
> **Cat Eyes / 고양이 눈** — Precision sight and reaction — the sharpshooter's body.
> Acquisition: agi↑↑ | Rarity: rare
> Academic: Land & Nilsson (2012) animal eyes and visual acuity; Abernethy (1991) expert perception and reaction time

```yaml
- system: combat
  target: ranged_accuracy
  op: mult
  value: 1.6
  tags: [precision, visual_acuity]

- system: skill
  target: [archery, trapping, surgery, watchkeeping]
  op: mult
  value: 1.4
  tags: [precision_skills]

- system: behavior
  target: spot_hidden
  op: inject
  value:
    priority: 0.5
  tags: [perception]

- system: body
  target: movement_speed
  op: mult
  value: 1.15
  tags: [agility]
```

---

### #11 B_slow_healer
> **Slow Healer / 굼뜬 치유자** — Injuries linger for months — one bad wound can end a career.
> Acquisition: rec↓↓ | Rarity: rare
> Academic: Guo & DiPietro (2010) impaired wound healing factors; chronic wound pathophysiology literature

```yaml
- system: body
  target: healing_speed
  op: mult
  value: 0.25
  tags: [slow_recovery]

- system: body
  target: illness_duration_mult
  op: mult
  value: 3.0
  tags: [prolonged_illness]

- system: body
  target: wound_duration
  op: on_event
  value:
    trigger: wounded
    effect: wound_duration_mult
    mult: 4.0
  tags: [lingering_wounds]

- system: stress
  target: injury_duration_stress
  op: mult
  value: 2.5
  tags: [prolonged_suffering]

- system: aging
  target: chronic_condition_onset
  op: mult
  value: 1.4
  tags: [compounding_injuries]
```

---

### #12 B_undying
> **Deathless One / 불사자** — Near-unkillable — survives wounds that should be fatal.
> Acquisition: tou↑↑ rec↑↑ | Rarity: epic
> Academic: Gompertz-Makeham mortality deceleration at extremes; Kirkwood (2005) disposable soma theory — exceptional repair capacity

```yaml
- system: combat
  target: wound_severity_mult
  op: mult
  value: 0.35
  tags: [toughness, injury_resistance]

- system: body
  target: healing_speed
  op: mult
  value: 2.5
  tags: [recuperation]

# Signature mechanic: survival check on lethal damage
- system: combat
  target: lethal_survival
  op: on_event
  value:
    trigger: lethal_damage_received
    effect: survival_check
    base_chance: 0.35
    modified_by: tou
  tags: [deathless, signature]

- system: aging
  target: physical_decline_rate
  op: mult
  value: 0.6
  tags: [slow_aging]

- system: body
  target: disease_infection_chance
  op: mult
  value: 0.4
  tags: [disease_resistance]

- system: stress
  target: near_death_trauma
  op: mult
  value: 0.3
  tags: [death_acceptance]

# Reputation tag earned after surviving 2+ near-death events
- system: reputation
  target: tags
  op: tag
  value: unkillable
  condition:
    source: self
    key: near_death_survival_count
    op: gte
    value: 2
  tags: [earned_reputation]
```

---

## Nous Traits (N_) — Cognitive Extremes

> Gardner intelligence axes: logical, linguistic, spatial, musical, kinesthetic, interpersonal, intrapersonal, naturalistic.
> Nous traits affect skill learning rates, role suitability, decision-making quality, and derived cognitive stats.
> Academic grounding: Gardner (1983) Frames of Mind; Cattell-Horn-Carroll (CHC) theory of intelligence.

---

### #1 N_polymath
> **Polymath / 박식가** — Masters both reason and language — learns everything, explains everything.
> Acquisition: logical↑↑ linguistic↑↑ | Rarity: epic
> Academic: Gardner (1983) logical-mathematical + linguistic intelligences; Simonton (2009) scientific polymathy

```yaml
- system: skill
  target: [research, mathematics, writing, teaching, rhetoric, administration]
  op: mult
  value: 1.5
  tags: [cognitive_excellence]

- system: derived
  target: wisdom
  op: mult
  value: 1.3
  tags: [accumulated_knowledge]

- system: behavior
  target: seek_knowledge
  op: inject
  value:
    priority: 0.6
  tags: [intellectual_drive]

- system: need
  target: competence
  op: mult
  value: 1.3
  tags: [mastery_hunger]

- system: skill
  target: cross_domain_insight
  op: on_event
  value:
    trigger: learns_new_skill
    effect: cross_domain_insight_chance
    chance: 0.2
  tags: [polymath_connection]

- system: skill
  target: learning_rate_all
  op: mult
  value: 1.2
  tags: [fast_learner]
```

---

### #2 N_silver_voice
> **Silver Voice / 은빛 목소리** — Words land exactly right — speaker, diplomat, teacher.
> Acquisition: linguistic↑↑ interpersonal↑↑ | Rarity: epic
> Academic: Gardner (1983) linguistic + interpersonal intelligences; Goleman (1995) social intelligence and influence

```yaml
- system: skill
  target: [persuasion, diplomacy, teaching, negotiation, oration]
  op: mult
  value: 1.6
  tags: [social_mastery]

- system: derived
  target: charisma
  op: mult
  value: 1.35
  tags: [natural_speaker]

- system: behavior
  target: mediate_conflict
  op: inject
  value:
    priority: 0.5
  tags: [peacemaker]

- system: aura
  target: trust_gain_rate
  op: mult
  value: 1.2
  tags: [trustworthy_presence]

- system: relationship
  target: intimacy_gain
  op: mult
  value: 1.25
  tags: [social_warmth]

- system: behavior
  target: broker_resolution
  op: on_event
  value:
    trigger: conflict_between_agents
    effect: inject_mediation
    priority: 0.6
  tags: [conflict_resolution]
```

---

### #3 N_architects_eye
> **Architect's Eye / 건축가의 눈** — Sees structures in space — builder and planner.
> Acquisition: spatial↑↑ logical↑ | Rarity: epic
> Academic: Gardner (1983) spatial intelligence; Newcombe & Shipley (2015) spatial cognition in STEM

```yaml
- system: skill
  target: [architecture, engineering, fortification, navigation, mapmaking]
  op: mult
  value: 1.6
  tags: [spatial_mastery]

- system: skill
  target: [sculpture, painting]
  op: mult
  value: 1.3
  tags: [artistic_spatial]

- system: behavior
  target: survey_land
  op: inject
  value:
    priority: 0.4
  tags: [spatial_awareness]

- system: event
  target: building_quality
  op: on_event
  value:
    trigger: designs_building
    effect: quality_mult
    mult: 1.4
  tags: [master_builder]

- system: derived
  target: creativity
  op: mult
  value: 1.2
  tags: [spatial_creativity]
```

---

### #4 N_beast_tongue
> **Beast Tongue / 짐승의 말** — Reads animals and nature like text — farmer, hunter, ranger.
> Acquisition: naturalistic↑↑ kinesthetic↑ | Rarity: epic
> Academic: Gardner (1983) naturalistic + bodily-kinesthetic intelligences; Kellert (1997) biophilia and naturalistic cognition

```yaml
- system: skill
  target: [animal_training, falconry, veterinary, hunting, trapping, herbalism, farming]
  op: mult
  value: 1.5
  tags: [nature_mastery]

- system: behavior
  target: commune_with_nature
  op: inject
  value:
    priority: 0.5
  tags: [nature_bond]

- system: skill
  target: [weather_reading, foraging]
  op: mult
  value: 1.4
  tags: [environmental_reading]

- system: event
  target: animal_encounter
  op: on_event
  value:
    trigger: encountering_wild_animal
    effect: reduced_hostility_chance
    chance: 0.4
  tags: [animal_affinity]

- system: derived
  target: naturalistic_intelligence
  op: mult
  value: 1.4
  tags: [nature_cognition]
```

---

### #5 N_war_savant
> **War Savant / 전쟁의 천재** — Legendary 3-axis — reads battlefield as geometry, body as weapon.
> Acquisition: kinesthetic↑↑ spatial↑ logical↑ | Rarity: legendary
> Academic: Gardner (1983) bodily-kinesthetic + spatial + logical intelligences; Clausewitz (1832) On War — coup d'oeil; Boyd (1976) OODA loop

```yaml
- system: skill
  target: [swordsmanship, archery, tactics, command, siege_warfare]
  op: mult
  value: 1.7
  tags: [military_mastery]

- system: combat
  target: tactical_command_mult
  op: mult
  value: 1.5
  tags: [tactical_genius]

# Signature: settlement-wide combat aura when leading
- system: aura
  target: settlement_combat_power
  op: mult
  value: 1.2
  condition:
    source: self
    key: is_leading_in_battle
    op: eq
    value: true
  tags: [command_aura, signature]

- system: behavior
  target: read_battlefield
  op: inject
  value:
    priority: 0.8
  tags: [tactical_awareness, signature]

- system: combat
  target: battle_analysis
  op: on_event
  value:
    trigger: battle_begins
    effect: analyze_enemy_formation_event
  tags: [strategic_mind]

- system: derived
  target: intimidation
  op: mult
  value: 1.3
  tags: [warrior_presence]

- system: stress
  target: non_combat_boredom
  op: mult
  value: 1.5
  tags: [war_mind]

- system: derived
  target: wisdom
  op: mult
  value: 1.2
  condition:
    source: self
    key: in_military_context
    op: eq
    value: true
  tags: [military_wisdom]
```

---

### #6 N_inner_eye
> **Inner Eye / 내면의 눈** — Profound self-knowledge — emotional regulation, personal growth mastery.
> Acquisition: intrapersonal↑↑ | Rarity: rare
> Academic: Gardner (1983) intrapersonal intelligence; Gross (2015) emotion regulation handbook; Kabat-Zinn (1990) mindfulness

```yaml
- system: stress
  target: recovery_rate
  op: mult
  value: 1.8
  tags: [emotional_regulation]

- system: stress
  target: break_threshold
  op: mult
  value: 1.4
  tags: [mental_resilience]

- system: derived
  target: wisdom
  op: mult
  value: 1.5
  tags: [self_knowledge]

- system: skill
  target: [meditation, counseling, self_discipline]
  op: mult
  value: 1.5
  tags: [introspective_skills]

- system: need
  target: autonomy_fulfillment_rate
  op: mult
  value: 1.3
  tags: [self_direction]

- system: memory
  target: trauma_processing
  op: on_event
  value:
    trigger: trauma_event
    effect: reduced_intensity_mult
    mult: 0.6
  tags: [trauma_resilience]
```

---

### #7 N_natures_child
> **Nature's Child / 자연의 아이** — Born of the wild — nature is home, civilization is foreign.
> Acquisition: naturalistic↑↑ | Rarity: rare
> Academic: Gardner (1983) naturalistic intelligence; Wilson (1984) biophilia hypothesis; Kaplan & Kaplan (1989) restorative environments

```yaml
- system: skill
  target: [foraging, farming, animal_training, herbalism, weather_reading]
  op: mult
  value: 1.5
  tags: [nature_skills]

- system: stress
  target: urban_settlement_stress
  op: mult
  value: 1.5
  tags: [civilization_discomfort]

- system: stress
  target: wilderness_stress
  op: immune
  value: true
  tags: [wild_comfort]

- system: behavior
  target: migrate_to_wilderness
  op: inject
  value:
    priority: 0.4
  condition:
    source: self
    key: in_dense_settlement
    op: eq
    value: true
  tags: [wilderness_pull]

- system: emotion
  target: joy
  op: on_event
  value:
    trigger: in_pristine_nature
    effect: happiness_boost
    intensity: 0.3
  tags: [nature_joy]
```

---

### #8 N_muse_touched
> **Muse-Marked / 뮤즈의 총아** — Music and words entwined — bard, healer of grief.
> Acquisition: musical↑↑ linguistic↑ | Rarity: epic
> Academic: Gardner (1983) musical + linguistic intelligences; Juslin & Sloboda (2010) music and emotion; Koelsch (2014) music therapy

```yaml
- system: skill
  target: [music, poetry, storytelling, ritual_performance]
  op: mult
  value: 1.7
  tags: [artistic_mastery]

# Aura: music heals grief
- system: aura
  target: morale_recovery_rate
  op: mult
  value: 1.3
  tags: [healing_music]

- system: event
  target: grief_healing
  op: on_event
  value:
    trigger: performs_during_grief_event
    effect: settlement_morale_recovery_boost
    mult: 1.5
  tags: [bard_healing]

- system: event
  target: cultural_legacy
  op: on_event
  value:
    trigger: composes_masterwork
    effect: permanent_cultural_memory_event
  tags: [artistic_legacy]

- system: need
  target: aesthetic_fulfillment
  op: mult
  value: 1.4
  tags: [artistic_need]

- system: stress
  target: creative_block_stress
  op: mult
  value: 1.6
  tags: [artistic_vulnerability]
```

---

### #9 N_dim
> **Dullard / 둔재** — Struggles with abstraction and expression — slow learner, but vulnerability creates genuine connection.
> Acquisition: logical↓↓ linguistic↓↓ | Rarity: epic
> Academic: Gardner (1983) on non-hierarchical intelligences; Edgerton (1993) The Cloak of Competence — social resilience in intellectual disability

```yaml
- system: skill
  target: [research, mathematics, rhetoric, teaching]
  op: mult
  value: 0.3
  tags: [intellectual_limitation]

- system: skill
  target: learning_rate_all
  op: mult
  value: 0.6
  tags: [slow_learner]

- system: derived
  target: wisdom
  op: mult
  value: 0.7
  tags: [cognitive_limitation]

# Compensatory: community matters more — vulnerability creates connection
- system: need
  target: belonging_sensitivity
  op: mult
  value: 1.3
  tags: [compensatory, community_bond]

# No penalty to physical work — body works fine
- system: skill
  target: [physical_labor, farming, gathering]
  op: mult
  value: 1.0
  tags: [physical_capable]

- system: relationship
  target: trust_on_help
  op: on_event
  value:
    trigger: helped_by_others
    effect: trust_gain_spike
    mult: 1.8
  tags: [vulnerability_connection]

- system: stress
  target: being_mocked_stress
  op: mult
  value: 1.5
  tags: [shame_sensitivity]
```

---

### #10 N_feral_mind
> **Feral Mind / 야수의 정신** — Body is brilliant, social reading is broken — force without tact. Not malice (no H-axis component), just social blindness.
> Acquisition: kinesthetic↑↑ interpersonal↓↓ | Rarity: epic
> Academic: Gardner (1983) bodily-kinesthetic vs interpersonal dissociation; Baron-Cohen (2011) empathy spectrum — low cognitive empathy with intact affective empathy

```yaml
- system: skill
  target: [combat, hunting, tracking, survival]
  op: mult
  value: 1.5
  tags: [physical_mastery]

- system: skill
  target: [diplomacy, negotiation, leadership]
  op: mult
  value: 0.35
  tags: [social_blindness]

# Unintentional social damage — not malice, just can't read the room
- system: behavior
  target: act_without_reading_room
  op: inject
  value:
    priority: 0.5
  tags: [social_misreading]

- system: derived
  target: charisma
  op: mult
  value: 0.5
  tags: [low_social_intelligence]

- system: stress
  target: social_misreading_confusion_stress
  op: mult
  value: 1.6
  tags: [social_confusion]

- system: emotion
  target: confusion_on_social_failure
  op: on_event
  value:
    trigger: social_conflict_caused_unintentionally
    effect: confusion_emotion_spike
    intensity: 0.5
  tags: [bewilderment]

- system: combat
  target: damage_mult
  op: mult
  value: 1.3
  tags: [physical_prowess]
```
