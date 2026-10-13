extends SceneTree
const GodPanel = preload("res://scripts/ui/panels/god_hand_panel.gd")
const Effects = preload("res://scripts/rendering/art/god_hand_effects.gd")
var _failures: int = 0

class SnapshotWorld extends Node:
	var settlements: Dictionary = {}
	var state: Dictionary = {}
	var resources: Dictionary = {}
	var calls: Array[Array] = []
	var succeeds: bool = true
	func get_settlement_snapshot() -> Dictionary: return settlements
	func get_intervention_state() -> Dictionary: return state
	func get_resource_snapshot() -> Dictionary: return resources
	func apply_intervention(kind: String, settlement: int, amount: int) -> Dictionary:
		calls.append([kind, settlement, amount])
		return {"ok": succeeds, "applied": amount, "tick": 4}

func _initialize() -> void:
	call_deferred("_run")

func _check(value: bool, message: String) -> void:
	if not value:
		_failures += 1
		push_error(message)

func _run() -> void:
	var world := SnapshotWorld.new()
	var panel := GodPanel.new()
	root.add_child(panel)
	panel.set_process(false)
	panel.configure(world)
	_check(panel._buttons.size() == 7, "exactly seven buttons")
	for button: Button in panel._buttons.values():
		_check(button.disabled, "no settlement disables buttons")
	world.settlements = {"settlement_ids": PackedInt32Array([7]), "formation_xs": PackedInt32Array([3]), "formation_ys": PackedInt32Array([4])}
	var locale: Node = root.get_node("Locale")
	var original: String = str(locale.get("_current_lang"))
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		panel.refresh_state()
		for kind: String in GodPanel.PRESETS:
			var button: Button = panel._buttons[kind]
			_check(not button.disabled, "settlement enables buttons")
			_check(button.text == locale.call("ltr", "GOD_HAND_" + kind.to_upper()), "localized label")
			_check(button.tooltip_text.contains(str(GodPanel.PRESETS[kind])), "tooltip preset")
			button.pressed.emit()
			_check(world.calls.back() == [kind, -1, GodPanel.PRESETS[kind]], "home preset command " + kind)
			_check(panel._toast.text == panel.reply_text(kind, GodPanel.PRESETS[kind], GodPanel.PRESETS[kind]), "localized success reply")
		_check(panel.amount_text("rain", 13).ends_with(str(locale.call("ltr", "GOD_HAND_UNIT_HOURS"))), "rain hours")
		_check(panel.amount_text("cold_snap", 1).ends_with(str(locale.call("ltr", "GOD_HAND_UNIT_NIGHTS"))), "cold snap nights")
	world.succeeds = false
	panel.cast_kind("rain")
	_check(panel._toast.text == locale.call("ltr", "GOD_HAND_FAILED"), "failed reply")
	world.state = {"log_kinds": PackedStringArray(["gift_food", "rain"]), "log_ticks": PackedInt64Array([1, 2]), "log_amounts": PackedInt64Array([60, 13]), "log_applied": PackedInt64Array([60, 13]), "cold_snap_tonight": true, "cold_snap_nights_left": 1}
	panel.refresh_state()
	_check(not panel._log.visible and panel._snap.visible, "collapsed log and active snap line")
	panel.set_log_open(true)
	_check(panel._log.visible and panel._log.get_child(1).visible and not panel._log.get_child(2).visible, "bounded log rows")
	var effects := Effects.new()
	root.add_child(effects)
	effects.set_process(false)
	var objects := Node2D.new()
	root.add_child(objects)
	var pile := Sprite2D.new()
	pile.set_meta("art_kind", "deadwood")
	pile.set_meta("tile_x", 3)
	pile.set_meta("tile_y", 4)
	objects.add_child(pile)
	var buildings := Node2D.new()
	root.add_child(buildings)
	var stock := Label.new()
	stock.set_meta("art_kind", "stockpile_label")
	stock.set_meta("art_id", (3 << 32) | 4)
	buildings.add_child(stock)
	effects._objects = objects
	effects._buildings = buildings
	world.state = {}
	world.resources = {"xs": PackedInt32Array([3]), "ys": PackedInt32Array([4]), "kinds": PackedInt32Array([3]), "amounts": PackedInt32Array([2])}
	effects.configure(world)
	effects._process(0.0)
	_check(effects.metadata()["rain_settlement_ids"].is_empty(), "natural state has no rain")
	_check(effects.metadata()["windfall_dust"].is_empty(), "natural state has no windfall dust")
	world.state = {"rain_settlement_ids": PackedInt32Array([7]), "rain_ticks_left": PackedInt32Array([20]), "cold_snap_tonight": true, "log_kinds": PackedStringArray(["windfall", "gift_wood"]), "log_ticks": PackedInt64Array([2, 3]), "log_amounts": PackedInt64Array([20, 40]), "log_applied": PackedInt64Array([20, 40])}
	world.resources["amounts"] = PackedInt32Array([4])
	effects._process(0.2)
	var metadata: Dictionary = effects.metadata()
	_check(metadata["rain_settlement_ids"] == [7], "active rain state starts effect")
	_check(metadata["cold_snap_tonight"], "snap presentation state")
	_check(metadata["windfall_falls"].size() == 1, "increased deadwood falls")
	_check(metadata["windfall_dust"].size() == 1 and metadata["windfall_dust"][0]["tile"] == [3, 4], "dust ring at the spot that actually rose")
	_check(metadata["windfall_dust"][0]["width"] == 2.0 and metadata["windfall_dust"][0]["duration"] == 0.6 and metadata["windfall_dust"][0]["color"] == "E8D8B8", "two-pixel pale dust with explicit lifetime")
	_check(metadata["stockpile_flashes"].size() == 1, "gift flashes")
	_check(is_equal_approx(pile.offset.y, -6.0), "pile actually falls above its atlas pivot")
	_check(stock.self_modulate == Effects.GIFT_COLOR, "stockpile label actually flashes green")
	_check(is_equal_approx(Effects.fall_offset(0.0), -6.0) and is_equal_approx(Effects.fall_offset(0.7), -1.0), "six-pixel fall and one-pixel bounce")
	_check(Effects.dust_alpha(0.5) > 0.0 and Effects.dust_alpha(0.6) == 0.0, "dust stops at .6 seconds")
	effects._process(0.61)
	_check(effects.metadata()["windfall_falls"].size() == 1 and effects.metadata()["windfall_dust"].is_empty(), "drop lasts beyond dust at .61 seconds")
	effects._process(0.2)
	_check(effects.metadata()["windfall_falls"].is_empty() and Effects.fall_offset(0.8) == 0.0, "drop ends at .8 seconds")
	effects._process(0.4)
	_check(effects.metadata()["stockpile_flashes"].is_empty(), "flash ends at 1.2 real seconds")
	_check(is_zero_approx(pile.offset.y) and stock.self_modulate == Color.WHITE, "fall and flash restore rendered nodes")
	world.state["rain_ticks_left"] = PackedInt32Array([0])
	world.state["cold_snap_tonight"] = false
	effects._process(0.1)
	_check(effects.metadata()["rain_settlement_ids"].is_empty(), "expired rain stops effect")
	_check(not effects.metadata()["cold_snap_tonight"], "snap stops with state")
	effects.configure(world)
	_check(effects.metadata()["windfall_falls"].is_empty(), "reset clears transient memory")
	_check(effects.metadata()["windfall_dust"].is_empty(), "reset clears dust")
	locale.call("set_language", original)
	effects.free()
	objects.free()
	buildings.free()
	panel.free()
	world.free()
	print("GOD_HAND failures=", _failures)
	quit(1 if _failures else 0)
