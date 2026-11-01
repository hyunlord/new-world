# Trade pane quantity and click-preflight regression with a controlled bridge read.
# Bridge surface: get_caravan and cmd_trade; no simulation is advanced.
extends SceneTree

class FakeWorld:
	extends Node
	var available: int = 2
	var include_available: bool = true
	var sent: int = 0
	func get_caravan(_id: int) -> Dictionary:
		var data: Dictionary = {"stock_x": PackedInt32Array([12]), "stock_y": PackedInt32Array([13]), "stock_kind": PackedInt64Array([0]), "stock_count": PackedInt32Array([10]), "stock_quality": PackedInt32Array([50]), "stock_value": PackedInt32Array([10]), "goods_kind": PackedInt64Array([0]), "goods_count": PackedInt32Array([1]), "goods_quality": PackedInt32Array([50]), "goods_value": PackedInt32Array([1]), "goods_unknown": PackedInt32Array([0])}
		if include_available:
			data["stock_available_count"] = PackedInt32Array([available])
		return data
	func cmd_trade(_id: int, give: PackedInt32Array, _take: PackedInt32Array) -> Dictionary:
		sent += 1
		return {"accepted": give.size() == 3 and give[2] <= available}

class TradeFixturePanel:
	extends "res://scripts/ui/panels/trade_panel.gd"
	func _ready() -> void:
		super._ready()
	func _refresh() -> void:
		_read = _world_sim.call("get_caravan", _caravan)
		_clamp_picks()
	func _toast(_text: String) -> void:
		pass

var failed: int = 0

func _init() -> void:
	call_deferred("_run")

func check(ok: bool, label: String) -> void:
	print("%s: %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		failed += 1

func _run() -> void:
	var world: FakeWorld = FakeWorld.new()
	var panel: TradeFixturePanel = TradeFixturePanel.new()
	root.add_child(world)
	root.add_child(panel)
	panel.set("_world_sim", world)
	panel.set("_caravan", 9)
	panel.call("_refresh")
	check(int(panel.stock_rows()[0]["count"]) == 2, "offered quantity excludes eight owed pieces")
	panel.pick_give(0, 10)
	check(int((panel.get("_give") as Dictionary).get("12,13", 0)) == 2, "selection clamps to available count")
	world.include_available = false
	panel.call("_refresh")
	check(panel.stock_rows().is_empty() and (panel.get("_give") as Dictionary).is_empty(), "missing availability fails closed")
	world.include_available = true
	world.available = 2
	panel.call("_refresh")
	panel.pick_give(0, 2)
	panel.pick_take(0, 1)
	world.available = 0
	var stale: Dictionary = panel.trade()
	check(world.sent == 0 and bool(stale.get("skipped", false)), "stale selection refreshes before send and emits no rejected command")
	check(not str(panel.texts()["status"]).is_empty(), "changed selection displays existing selection guidance")
	check((panel.get("_give") as Dictionary).is_empty(), "unavailable selected stock is removed before confirmation")
	world.available = 2
	panel.call("_refresh")
	panel.pick_give(0, 2)
	panel.pick_take(0, 1)
	var accepted: Dictionary = panel.trade()
	check(world.sent == 1 and bool(accepted.get("accepted", false)), "unchanged valid selection sends one accepted command")
	print("TRADE_AVAILABILITY_DONE failed=%d" % failed)
	quit(0 if failed == 0 else 1)
