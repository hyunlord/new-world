# Phase 2 Tickets: Time + Personality + Emotion + Relationships + Family

## Dependency Order

```
T-1000 (time constants)
  ├→ T-1010 (entity data extensions)
  │    ├→ T-1030 (personality)
  │    ├→ T-1040 (emotion system)
  │    ├→ T-1080 (age system)
  │    └→ T-1110 (renderer gender/age)
  ├→ T-1020 (needs/resource rebalance)
  └→ T-1050 (chunk spatial index)
       └→ T-1060 (relationship manager)
            └→ T-1070 (social events)
                 └→ T-1090 (family system)
                      └→ T-1100 (binary save/load)
                           └→ T-1120 (detail panel UI)
                                └→ T-1130 (stats panel + docs)

T-1110 (renderer) can parallel with T-1060+
T-1120 (UI) can parallel with T-1100
```

## Tickets

### T-1000: Time System Constants
- GameConfig: TICK_HOURS=2, TICKS_PER_DAY=12, TICKS_PER_YEAR=4380, DAYS_PER_YEAR=365
- Remove TICK_MINUTES, AGE_DAYS_DIVISOR
- Add tick_to_date(tick) -> Dictionary
- Add age constants: AGE_CHILD_END, AGE_TEEN_END, AGE_ADULT_END, AGE_MAX
- SimEngine.get_game_time() rewrite for new calendar
- HUD time display: "Y3 M7 D15 14:00"
- Status: PENDING

### T-1010: EntityData Extensions
- Add: gender, age_stage, partner_id, parent_ids, children_ids
- Add: pregnancy_tick, birth_tick
- Add: personality dict (5 traits), emotions dict (5 emotions)
- Remove from_dict/to_dict (replaced by binary save later)
- Status: PENDING

### T-1020: Needs/Resource/Building Rebalance
- NeedsSystem: hunger decay=0.002/tick, energy=0.003, social=0.001
- ResourceRegenSystem: interval 120 ticks (10 days)
- Building build_ticks: day-unit conversion (stockpile=36, shelter=60, campfire=24)
- Starvation grace: recalculate for new tick rate
- Auto-eat threshold adjustment
- Day/night: TICKS_PER_DAY=12 based hour calculation
- DO NOT change BehaviorSystem/MovementSystem/GatheringSystem/ConstructionSystem tick_interval
- Status: PENDING

### T-1030: Personality System
- EntityData personality dict initialized randf_range(0.1, 0.9)
- GameConfig.personality_compatibility(a, b) -> float
- BehaviorSystem: diligence affects work efficiency (0.8~1.2x)
- BehaviorSystem: extraversion affects socialize utility weight
- Status: PENDING

### T-1040: Emotion System
- New: EmotionSystem (priority=32, tick_interval=12)
- happiness = lerp toward (hunger+energy+social)/3
- loneliness, stress, grief, love updates
- Effects on behavior utilities
- Status: PENDING

### T-1050: Chunk Spatial Index
- New: ChunkIndex class (16x16 tiles per chunk)
- Methods: update_entity(id, old_pos, new_pos), get_entities_in_chunk(cx, cy), get_nearby_entities(pos, radius)
- Integrated into EntityManager
- O(1) chunk lookup, O(chunk_size) neighbor scan
- Status: PENDING

### T-1060: Relationship Manager
- New: Relationship class (affinity, trust, romantic_interest, interaction_count, last_tick, type)
- New: RelationshipManager (Dictionary key="min_id:max_id")
- Stage transitions: stranger→acquaintance→friend→close_friend→romantic→partner
- Natural decay: affinity -0.1 per 100 ticks without interaction
- Cleanup: delete acquaintance with affinity<=5
- Status: PENDING

### T-1070: Social Event System
- New: SocialEventSystem (priority=37, tick_interval=30)
- Chunk-based proximity check (same chunk only)
- Events: CASUAL_TALK, DEEP_TALK, SHARE_FOOD, WORK_TOGETHER, FLIRT, GIVE_GIFT, PROPOSAL, CONSOLE, ARGUMENT
- Weighted random selection based on personality + situation
- Proposal: compatibility-based acceptance
- Toast notifications for proposals
- Status: PENDING

### T-1080: Age System
- New: AgeSystem (priority=48, tick_interval=50)
- Age stage transitions: child→teen→adult→elder
- Job restrictions by age stage
- Movement speed multipliers by age
- Gathering efficiency by age
- Elder death probability (5%/year after 60)
- Growth toast notifications
- Status: PENDING

### T-1090: Family System + Disable Asexual
- New: FamilySystem (priority=52, tick_interval=50)
- Pregnancy checks (8 conditions)
- Birth process (consume food, create child, assign parents)
- Disable PopulationSystem birth (keep death logic)
- Initial relationship bootstrap (3-4 friend pairs, 1-2 close_friend)
- Partner behavior: move_to_partner, shared shelter preference
- Spouse death: grief, remarriage cooldown
- Status: PENDING

### T-1100: Binary Save/Load
- Save path: user://saves/quicksave/
- Files: meta.json, entities.bin, buildings.bin, relationships.bin, settlements.bin, world.bin, stats.json
- Binary serialization with store_32/store_float/store_pascal_string
- Load: clear_all → read → rebuild indices
- Drop JSON save compatibility
- Status: PENDING

### T-1110: Entity Renderer Updates
- Gender tint: male=blue tint, female=red tint
- Age size: child=0.6x, teen=0.8x, elder=0.95x
- Elder: white dot overlay
- Selected entity: heart marker on partner
- Status: PENDING

### T-1120: Entity Detail Panel Extensions
- Personality section: 5 bars
- Emotions section: 5 bars
- Family section: partner, children, parents, family tree button
- Relationships section: top 5 relationships
- Status: PENDING

### T-1130: Stats Panel + Docs
- Stats: couples count, singles count, age distribution bar, avg happiness
- Recent events: marriages, births, deaths
- Docs: GAME_BALANCE, VISUAL_GUIDE, SYSTEMS, ARCHITECTURE, CHANGELOG
- Status: PENDING
