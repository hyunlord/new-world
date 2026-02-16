# T-2013-01: Conditional Child Starvation Protection

## Objective
Change child starvation from absolute immunity to conditional protection: children are protected when settlement food exists, but can starve during true famine (food = 0).

## Non-goals
- Do NOT change ChildcareSystem (priority, tick_interval, feeding logic)
- Do NOT change MortalitySystem (Siler model, monthly pop log)
- Do NOT change GameConfig constants
- Do NOT change EntityData fields (starving_timer already exists and is serialized)
- Do NOT change execution order of systems

## Files to modify

### 1. `scripts/systems/needs_system.gd`

#### 1a. Add `_building_manager` variable (after line 4)

Current:
```gdscript
var _entity_manager: RefCounted
var _mortality_system: RefCounted
```

Change to:
```gdscript
var _entity_manager: RefCounted
var _building_manager: RefCounted
var _mortality_system: RefCounted
```

#### 1b. Update `init()` to accept building_manager (line 14)

Current:
```gdscript
func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
```

Change to:
```gdscript
func init(entity_manager: RefCounted, building_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
```

#### 1c. Make hunger floor conditional (lines 45-46)

Current:
```gdscript
		if entity.age_stage == "infant" or entity.age_stage == "toddler" or entity.age_stage == "child" or entity.age_stage == "teen":
			entity.hunger = maxf(entity.hunger, 0.05)
```

Change to:
```gdscript
		if entity.age_stage == "infant" or entity.age_stage == "toddler" or entity.age_stage == "child" or entity.age_stage == "teen":
			if entity.settlement_id > 0 and _get_settlement_food(entity.settlement_id) > 0.0:
				entity.hunger = maxf(entity.hunger, 0.05)
```

When settlement has food, children keep the 0.05 hunger floor (preventing starvation trigger).
When settlement has NO food (true famine), the floor is removed and hunger can reach 0.

#### 1d. Replace absolute immunity with conditional protection (lines 51-57)

Current:
```gdscript
		if entity.hunger <= 0.0:
			# Children under 15 are immune to starvation death.
			# Child mortality is handled by the Siler model (infant_mortality).
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

Replace the ENTIRE block (from `if entity.hunger <= 0.0:` to `entity.starving_timer = 0`) with:
```gdscript
		if entity.hunger <= 0.0:
			var age_years: float = GameConfig.get_age_years(entity.age)
			if age_years < 15.0:
				# Child conditional protection: check settlement food
				var sett_food: float = 0.0
				if entity.settlement_id > 0:
					sett_food = _get_settlement_food(entity.settlement_id)
				if sett_food > 0.0:
					# Food exists but child is starving → emergency feed from stockpile
					var feed_amount: float = minf(0.3, sett_food)
					var withdrawn: float = _withdraw_food(entity.settlement_id, feed_amount)
					if withdrawn > 0.0:
						entity.hunger = withdrawn * GameConfig.FOOD_HUNGER_RESTORE
					entity.starving_timer = 0
				else:
					# True famine (no settlement food) → grace period, then starvation allowed
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
						entity.hunger = 0.01  # Keep barely alive during grace
			else:
				# Adult starvation: grace period then death
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

#### 1e. Add helper functions at the end of the file (after line 74)

Add these two functions (same pattern as childcare_system.gd):

```gdscript


## Get total food in stockpiles belonging to a settlement
func _get_settlement_food(settlement_id: int) -> float:
	if _building_manager == null:
		return 0.0
	var total_food: float = 0.0
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		var stockpile: RefCounted = stockpiles[i]
		if stockpile.settlement_id != settlement_id or not stockpile.is_built:
			continue
		total_food += float(stockpile.storage.get("food", 0.0))
	return total_food


## Withdraw food from stockpiles belonging to a settlement
func _withdraw_food(settlement_id: int, amount: float) -> float:
	if _building_manager == null or amount <= 0.0:
		return 0.0
	var remaining: float = amount
	var withdrawn: float = 0.0
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		if remaining <= 0.0:
			break
		var stockpile: RefCounted = stockpiles[i]
		if stockpile.settlement_id != settlement_id or not stockpile.is_built:
			continue
		var available: float = float(stockpile.storage.get("food", 0.0))
		if available <= 0.0:
			continue
		var take: float = minf(available, remaining)
		stockpile.storage["food"] = available - take
		remaining -= take
		withdrawn += take
	return withdrawn
```

### 2. `scenes/main/main.gd`

Find the line:
```gdscript
	needs_system.init(entity_manager)
```

Change to:
```gdscript
	needs_system.init(entity_manager, building_manager)
```

This is around line 115. The `building_manager` variable already exists in main.gd (used by other systems).

## Behavior Summary

| Situation | Child (< 15y) | Adult |
|-----------|---------------|-------|
| Food available + hunger → 0 | Emergency feed from stockpile, **survives** | Uses grace period (auto-eat usually saves them) |
| Food = 0 + hunger → 0 | Grace period (CHILD_STARVATION_GRACE_TICKS), **can die** | Grace period (STARVATION_GRACE_TICKS), **can die** |
| Famine ends (food returns) | Timer resets via hunger floor, **recovers** | Timer resets if hunger > 0 |

## Acceptance Criteria
- [ ] LSP diagnostics: 0 errors on needs_system.gd
- [ ] LSP diagnostics: 0 errors on main.gd
- [ ] Children with settlement food > 0 never reach starvation death
- [ ] Children with settlement food = 0 can die after grace period
- [ ] Adult starvation path unchanged
- [ ] No changes to childcare_system.gd, mortality_system.gd, or game_config.gd

## Context
- EntityData already has `starving_timer: int` field (serialized in to_dict/from_dict)
- GameConfig.CHILD_STARVATION_GRACE_TICKS = {"infant": 50, "toddler": 40, "child": 30, "teen": 20}
- GameConfig.STARVATION_GRACE_TICKS = 25 (adult default)
- GameConfig.FOOD_HUNGER_RESTORE = 0.3
- BuildingManager.get_buildings_by_type("stockpile") returns Array of BuildingData
- BuildingData has: settlement_id, is_built, storage (Dictionary with "food", "wood", "stone")
- ChildcareSystem already uses identical _get_settlement_food/_withdraw_food pattern
