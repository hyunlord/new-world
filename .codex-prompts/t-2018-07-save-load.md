# T-2018-07: Save/Load EmotionData + EntityData Extension

## Objective
Add `emotion_data` field to EntityData, extend the binary save/load system to serialize EmotionData (3 layers + stress + habituation + mental break state), bump save version, and handle legacy migration.

## Godot 4.6 Headless Compatibility (CRITICAL)
- **NO `class_name`** on RefCounted scripts
- Use `preload()` / `load()` for script references
- Use `var x = dict.get(...)` (untyped), NOT `var x := dict.get(...)`

## Files to Modify

### 1. `scripts/core/entity_data.gd` (EDIT — add emotion_data field)
### 2. `scripts/core/save_manager.gd` (EDIT — extend save/load, bump version)

## Dependencies (already exist)
- `scripts/core/emotion_data.gd` — EmotionData with `to_dict()`, `from_dict()`, `from_legacy()` static methods

---

## Changes to `scripts/core/entity_data.gd`

### Add field after line 44 (after `emotions` Dictionary)

After the existing `emotions` Dictionary declaration (line 38-44), add:

```gdscript
## Phase 2-A3: Plutchik emotion data (EmotionData RefCounted)
var emotion_data: RefCounted = null
```

### Update `to_dict()` (around line 106)

After the `"emotions": emotions.duplicate(),` line (line 140), add:

```gdscript
		"emotion_data": emotion_data.to_dict() if emotion_data != null else {},
```

### Update `from_dict()` (around line 146)

After the existing emotions loading block (lines 210-217), add:

```gdscript
	# Plutchik emotion data (Phase 2-A3)
	var EmotionDataScript = load("res://scripts/core/emotion_data.gd")
	var ed_data = data.get("emotion_data", {})
	if not ed_data.is_empty() and ed_data.has("fast"):
		e.emotion_data = EmotionDataScript.from_dict(ed_data)
	else:
		# Legacy migration: create EmotionData from old 5-emotion values
		e.emotion_data = EmotionDataScript.from_legacy(e.emotions)
```

---

## Changes to `scripts/core/save_manager.gd`

### Bump SAVE_VERSION (line 8)

```gdscript
# Before:
const SAVE_VERSION: int = 5
# After:
const SAVE_VERSION: int = 6
```

### Extend `_save_entities()` — add EmotionData after existing 5 emotion floats

After the 5 emotion floats block (lines 153-158, the `f.store_float(e.emotions.get("love", 0.0))` line), add the EmotionData serialization:

```gdscript
		# EmotionData (v6+): JSON string for flexible serialization
		if e.emotion_data != null:
			var ed_json: String = JSON.stringify(e.emotion_data.to_dict())
			f.store_pascal_string(ed_json)
		else:
			f.store_pascal_string("")
```

### Extend `_load_entities()` — add EmotionData loading after existing 5 emotion floats

After the emotions block (lines 238-244, right after `e.emotions = { ... }`), add:

```gdscript
		# EmotionData (v6+)
		var EmotionDataScript = load("res://scripts/core/emotion_data.gd")
		if _load_version >= 6:
			var ed_json_str: String = f.get_pascal_string()
			if ed_json_str != "":
				var ed_json: JSON = JSON.new()
				if ed_json.parse(ed_json_str) == OK and ed_json.data is Dictionary:
					e.emotion_data = EmotionDataScript.from_dict(ed_json.data)
				else:
					e.emotion_data = EmotionDataScript.from_legacy(e.emotions)
			else:
				e.emotion_data = EmotionDataScript.from_legacy(e.emotions)
		else:
			# Pre-v6 saves: migrate from legacy 5-emotion values
			e.emotion_data = EmotionDataScript.from_legacy(e.emotions)
```

**IMPORTANT**: The `EmotionDataScript` load must be outside the entity loop for efficiency. Move it next to the existing `EntityDataScript` and `PersonalityDataScript` loads at the top of `_load_entities()` (around line 197-198):

```gdscript
	var EntityDataScript = load("res://scripts/core/entity_data.gd")
	var PersonalityDataScript = load("res://scripts/core/personality_data.gd")
	var EmotionDataScript = load("res://scripts/core/emotion_data.gd")  # ADD THIS
```

Then in the loop, use just:
```gdscript
		if _load_version >= 6:
			var ed_json_str: String = f.get_pascal_string()
			if ed_json_str != "":
				var ed_json: JSON = JSON.new()
				if ed_json.parse(ed_json_str) == OK and ed_json.data is Dictionary:
					e.emotion_data = EmotionDataScript.from_dict(ed_json.data)
				else:
					e.emotion_data = EmotionDataScript.from_legacy(e.emotions)
			else:
				e.emotion_data = EmotionDataScript.from_legacy(e.emotions)
		else:
			e.emotion_data = EmotionDataScript.from_legacy(e.emotions)
```

---

## Non-goals
- Do NOT modify EmotionData itself (T-2018-01)
- Do NOT modify EmotionSystem (T-2018-02)
- Do NOT modify entity_detail_panel.gd (T-2018-06)
- Do NOT modify entity_manager.gd (emotion_data init happens in EmotionSystem.execute_tick)
- Do NOT modify main.gd
- Do NOT change the existing 5 emotion floats in the binary format (backward compat)
- Do NOT modify building, relationship, settlement, or resource map save/load

## Save Format Change Summary

### entities.bin v5 → v6
```
[existing v5 fields...]
emotions: 5 × float (happiness, loneliness, stress, grief, love)
emotion_data: pascal_string (JSON of EmotionData.to_dict(), or "" if null)  ← NEW
[remaining v5 fields: job, settlement_id, AI state, inventory, stats, parent_ids, children_ids]
```

The new `emotion_data` pascal_string is inserted right after the 5 legacy emotion floats and before the job byte.

## Acceptance Criteria
- [ ] `entity_data.gd` has `var emotion_data: RefCounted = null` field
- [ ] `entity_data.gd` `to_dict()` includes `emotion_data` key
- [ ] `entity_data.gd` `from_dict()` loads EmotionData or migrates from legacy
- [ ] `save_manager.gd` SAVE_VERSION bumped to 6
- [ ] `save_manager.gd` `_save_entities()` writes EmotionData as JSON pascal_string after 5 emotion floats
- [ ] `save_manager.gd` `_load_entities()` reads EmotionData JSON when version >= 6
- [ ] `save_manager.gd` `_load_entities()` migrates from legacy when version < 6
- [ ] `EmotionDataScript` loaded once outside the entity loop (not per-entity)
- [ ] v5 saves still load correctly (legacy migration)
- [ ] v6 saves include full EmotionData state
- [ ] No GDScript parse errors
- [ ] No changes to buildings, relationships, settlements, or resource map save/load
