# main_bootstrap.gd — selected content preflight before gameplay children enter the tree.
# M3D-1: when renderer_switch.gd wants 3D (`--renderer=3d` or the `renderer_3d` setting) it adds
#   scenes/world3d/World3D.tscn under Main and hides ColonyView (its layers keep running for hit-tests and orders).
# Bridge methods: WorldSimNode.bootstrap_catalog, WorldSimNode.create.
extends Node2D

var boot_ok: bool = false
var boot_error: String = ""


func _enter_tree() -> void:
	if boot_ok:
		return
	var receipt: Dictionary = WorldSimNode.bootstrap_catalog()
	if not bool(receipt.get("ok", false)):
		_fail(str(receipt.get("error", "Content bootstrap failed")))
		return
	var world: WorldSimNode = WorldSimNode.create()
	if world == null:
		_fail("WorldSim factory rejected content bootstrap")
		return
	world.name = "WorldSim"
	add_child(world)
	move_child(world, 0)
	boot_ok = true


# Gameplay children are ready before the root; keep bridge preflight in _enter_tree unchanged.
func _ready() -> void:
	if not boot_ok:
		return
	var panel: Control = preload("res://scripts/ui/panels/build_panel.gd").new()
	panel.name = "BuildPanel"
	get_node("UI").add_child(panel)
	panel.connect("template_chosen", Callable(get_node("ColonyView/OrderTool"), "set_template"))
	panel.connect("housing_requested", Callable(get_node("UI/OrderBar"), "select_mode").bind(15))
	panel.connect("source_requested", _open_building_source)
	if preload("res://scripts/world3d/renderer_switch.gd").wants_3d():
		_enable_3d()


## M3D-1: draw the world in 3D; the 2D view stays in the tree, hidden.
func _enable_3d() -> void:
	var world3d: Node = (load("res://scenes/world3d/World3D.tscn") as PackedScene).instantiate()
	add_child(world3d)
	move_child(world3d, get_node("ColonyView").get_index())
	(get_node("ColonyView") as CanvasItem).visible = false


func _open_building_source(site: int, record: int) -> void:
	var why: Node = get_node_or_null("UI/WhyPanel")
	if why != null:
		(get_node("UI/BuildPanel") as Control).hide()
		why.call("open_building", site, record)


func _fail(message: String) -> void:
	boot_error = message
	for child: Node in get_children():
		remove_child(child)
		child.queue_free()
	var error_label: Label = Label.new()
	error_label.text = "WorldSim startup failed\n" + message
	error_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	error_label.position = Vector2(24, 24)
	error_label.size = Vector2(900, 300)
	add_child(error_label)
	push_error(message)
