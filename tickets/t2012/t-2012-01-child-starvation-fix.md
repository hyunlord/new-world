# T-2012-01: 아동 아사 면역 + 실행순서 수정

## Objective
Fix child starvation by: (1) reordering system execution so childcare feeds children BEFORE hunger decay/death, (2) increasing childcare frequency to match needs, (3) adding absolute child starvation immunity as safety net.

## Non-goals
- Do NOT modify game_config.gd (constants are already correct)
- Do NOT modify entity_data.gd, entity_manager.gd, or any other files
- Do NOT add new signals or events
- Do NOT change adult starvation behavior
- Do NOT change the childcare feeding logic itself (thresholds, amounts, stockpile withdrawal)

## Root Cause Analysis
Children starve because:
1. NeedsSystem (priority 10, tick_interval 2) runs BEFORE ChildcareSystem (priority 12, tick_interval 10)
2. NeedsSystem decays hunger AND kills entities in the same execute_tick call
3. ChildcareSystem runs 5x less frequently (every 10 ticks vs every 2)
4. Children can't self-feed (no food in inventory)
5. No absolute immunity exists — children CAN die from starvation

## Files to Modify

### 1. scripts/systems/childcare_system.gd

**Change A: Priority 12 → 8** (run BEFORE NeedsSystem priority 10)
```gdscript
# In _init():
priority = 8  # was 12 — must run BEFORE NeedsSystem (priority 10)
```

**Change B: tick_interval 10 → 2** (match NeedsSystem frequency)
```gdscript
# In _init():
tick_interval = 2  # was GameConfig.CHILDCARE_TICK_INTERVAL (10) — match NeedsSystem frequency
```
Note: Use literal `2` instead of `GameConfig.CHILDCARE_TICK_INTERVAL` since we're deliberately overriding the config value for execution frequency. Add a comment explaining why.

### 2. scripts/systems/needs_system.gd

**Change A: Child hunger floor clamp (line 44 area)**
After the existing `entity.hunger = clampf(entity.hunger, 0.0, 1.0)` line, add a child-specific floor:
```gdscript
# Existing line:
entity.hunger = clampf(entity.hunger, 0.0, 1.0)

# ADD after it — child hunger floor (children cannot reach 0):
if entity.age_stage == "infant" or entity.age_stage == "toddler" or entity.age_stage == "child" or entity.age_stage == "teen":
    entity.hunger = maxf(entity.hunger, 0.05)
```

**Change B: Absolute child starvation immunity (line 49 area)**
In the starvation check block, add child immunity BEFORE the starving_timer increment:
```gdscript
# Starvation check with grace period
if entity.hunger <= 0.0:
    # Children under 15 are immune to starvation death
    # Academic basis: Gurven & Kaplan 2007 — ~70% of child deaths in
    # hunter-gatherer societies are from infection, not starvation.
    # Child mortality is already handled by Siler model (infant_mortality).
    var age_years: float = GameConfig.get_age_years(entity.age)
    if age_years < 15.0:
        entity.hunger = 0.05
        entity.starving_timer = 0
    else:
        entity.starving_timer += 1
        var grace: int = GameConfig.CHILD_STARVATION_GRACE_TICKS.get(entity.age_stage, GameConfig.STARVATION_GRACE_TICKS)
        if entity.starving_timer >= grace:
            emit_event("entity_starved", {
                "entity_id": entity.id,
                "entity_name": entity.entity_name,
                "starving_ticks": entity.starving_timer,
                "tick": tick,
            })
            _entity_manager.kill_entity(entity.id, "starvation", tick)
            if _mortality_system != null and _mortality_system.has_method("register_death"):
                var death_age_years: float = GameConfig.get_age_years(entity.age)
                _mortality_system.register_death(death_age_years < 1.0, entity.age_stage, death_age_years, "starvation")
else:
    entity.starving_timer = 0
```

Note: The `age_years` variable is computed inside the `if entity.hunger <= 0.0` block. Use a different name (`death_age_years`) for the one inside the starving_timer check to avoid shadowing.

## Acceptance Criteria
- [ ] childcare_system.gd: priority = 8, tick_interval = 2
- [ ] needs_system.gd: child hunger floor 0.05 after clamp
- [ ] needs_system.gd: absolute child starvation immunity (age < 15 → hunger = 0.05, starving_timer = 0)
- [ ] No changes to any other files
- [ ] Code compiles without errors (GDScript syntax valid)

## Context
- GameConfig.get_age_years(age_ticks) returns float years
- age_stage values: "infant", "toddler", "child", "teen", "adult", "elder"
- Children are infant (0-3y), toddler (3-6y), child (6-12y), teen (12-15y)
- NeedsSystem has `_mortality_system` reference (set externally in main.gd)
- The CHILD_STARVATION_GRACE_TICKS dictionary in GameConfig is now only used as fallback for adults; children will never reach the starvation check
