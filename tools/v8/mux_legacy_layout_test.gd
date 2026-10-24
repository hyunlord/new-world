# Legacy scaling integration checks against real Control transforms and menu bounds.
# Bridge methods: none. Parent runs with Godot --headless --path . --script this file.
extends SceneTree

const LegacyLayout = preload("res://scripts/ui/mux/legacy_layout.gd")
var _failures: int = 0
var _checks: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, passed: bool) -> void:
	_checks += 1
	if not passed:
		_failures += 1
	print("MUX_LEGACY_%s %s" % ["PASS" if passed else "FAIL", label])


func _owner(ui: Node, node_name: String) -> Control:
	var control: Control = Control.new()
	control.name = node_name
	ui.add_child(control)
	return control


func _panel(owner: Control, node_name: String, minimum: Vector2) -> PanelContainer:
	var panel: PanelContainer = PanelContainer.new()
	panel.name = node_name
	panel.custom_minimum_size = minimum
	panel.size = minimum
	owner.add_child(panel)
	return panel


func _run() -> void:
	root.size = Vector2i(1920, 1080)
	var ui: CanvasLayer = CanvasLayer.new()
	root.add_child(ui)
	var top: Control = _owner(ui, "HudTopbar")
	_panel(top, "Panel", Vector2(1000, 100))
	var orders: Control = _owner(ui, "OrderBar")
	var bar: PanelContainer = _panel(orders, "Bar", Vector2(1400, 100))
	var row: HBoxContainer = HBoxContainer.new()
	bar.add_child(row)
	for i: int in 8:
		var button: Button = Button.new()
		button.name = "Mode%d" % i
		button.custom_minimum_size = Vector2(140, 50)
		row.add_child(button)
	var build: PanelContainer = _panel(orders, "BuildMenu", Vector2(400, 800))
	build.position = Vector2(300, 100)
	build.hide()
	var inspector: Control = _owner(ui, "AgentInspectorPanel")
	inspector.position = Vector2(1548, 124)
	_panel(inspector, "Panel", Vector2(360, 480))
	var tech: Control = _owner(ui, "TechPanel")
	tech.position = Vector2(12, 124)
	_panel(tech, "Panel", Vector2(1896, 830))
	var story: Control = _owner(ui, "StoryBanner")
	var column: VBoxContainer = VBoxContainer.new()
	column.name = "Column"
	column.custom_minimum_size = Vector2(880, 100)
	story.add_child(column)
	var layout: LegacyLayout = LegacyLayout.new()
	root.add_child(layout)
	layout.setup(ui)
	layout.set_process(false)
	await process_frame
	layout.apply_scale(1.5, Vector2(1920, 1080), 1450)
	await process_frame
	layout.apply_scale(1.5, Vector2(1920, 1080), 1450)
	_check("bar API coordinate roots remain unscaled", top.scale == Vector2.ONE and orders.scale == Vector2.ONE)
	_check("world indicator root remains unscaled", story.scale == Vector2.ONE)
	_check("inspector uses requested scale", is_equal_approx(inspector.scale.x, 1.5))
	_check("inspector reserves MUX right column", (layout.layout_report()["AgentInspectorPanel"]["rect"] as Rect2).end.x <= 1450.01)
	_check("fixed tech canvas reports fit cap", layout.layout_report()["TechPanel"]["capped"])
	var initial: Dictionary = layout.layout_report()
	layout.apply_scale(1.0, Vector2(1920, 1080), 1450)
	layout.apply_scale(1.25, Vector2(1920, 1080), 1450)
	layout.apply_scale(1.5, Vector2(1920, 1080), 1450)
	_check("scale round trip does not compound", initial["AgentInspectorPanel"]["rect"] == layout.layout_report()["AgentInspectorPanel"]["rect"])
	build.show()
	build.position = Vector2(1800, 900)
	layout.apply_scale(1.5, Vector2(1920, 1080), 1450)
	_check("opened menu fits above scaled bar", build.get_global_rect().end.y <= layout.bar_top() - 7.99)
	_check("opened menu retains scale-aware hit rectangle", build.get_global_rect().size.is_equal_approx(build.size * build.scale))
	var screen: Rect2 = Rect2(Vector2.ZERO, Vector2(1920, 1080))
	for key: String in layout.layout_report():
		var rect: Rect2 = layout.layout_report()[key]["rect"]
		_check(key + " inside viewport", screen.encloses(rect))
	layout.apply_scale(1.5, Vector2(1280, 720), 950)
	screen.size = Vector2(1280, 720)
	for key: String in layout.layout_report():
		var rect: Rect2 = layout.layout_report()[key]["rect"]
		_check(key + " fits reduced viewport", screen.grow(0.01).encloses(rect))
	var topbar_edge: float = layout.top_bottom()
	layout.set_content_top(topbar_edge + 60.0)
	layout.apply_scale(1.5, Vector2(1280, 720), 950)
	_check("content reservation preserves topbar measurement", is_equal_approx(layout.top_bottom(), topbar_edge))
	_check("inspector clears full action header", inspector.get_global_transform().origin.y >= topbar_edge + 60.0)
	_check("story clears full action header", column.get_global_position().y >= topbar_edge + 60.0)
	ui.free()
	layout.free()
	print("MUX_LEGACY_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
