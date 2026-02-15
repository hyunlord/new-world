class_name PopupManager
extends CanvasLayer

var _sim_engine: RefCounted
var _was_paused: bool = false
var _dim_bg: ColorRect

var _stats_panel: Control
var _entity_panel: Control
var _building_panel: Control


func init(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine


func _ready() -> void:
	layer = 100
	_dim_bg = ColorRect.new()
	_dim_bg.color = Color(0, 0, 0, 0.7)
	_dim_bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	_dim_bg.mouse_filter = Control.MOUSE_FILTER_STOP
	_dim_bg.visible = false
	add_child(_dim_bg)
	_dim_bg.gui_input.connect(_on_bg_input)


func add_stats_panel(panel: Control) -> void:
	_stats_panel = panel
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	panel.visible = false
	_dim_bg.add_child(panel)


func add_entity_panel(panel: Control) -> void:
	_entity_panel = panel
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	panel.visible = false
	_dim_bg.add_child(panel)


func add_building_panel(panel: Control) -> void:
	_building_panel = panel
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	panel.visible = false
	_dim_bg.add_child(panel)


func _on_bg_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		close_all()


func open_stats() -> void:
	if _stats_panel == null:
		return
	if _stats_panel.visible:
		close_all()
		return
	if not _dim_bg.visible:
		_pause_sim()
	_hide_all_panels()
	_stats_panel.visible = true
	_dim_bg.visible = true
	_center_panel(_stats_panel, 0.75, 0.8)


func open_entity(entity_id: int) -> void:
	if _entity_panel == null:
		return
	if _entity_panel.visible:
		close_all()
		return
	if not _dim_bg.visible:
		_pause_sim()
	_hide_all_panels()
	_entity_panel.set_entity_id(entity_id)
	_entity_panel.visible = true
	_dim_bg.visible = true
	_center_panel(_entity_panel, 0.55, 0.85)


func open_building(building_id: int) -> void:
	if _building_panel == null:
		return
	if _building_panel.visible:
		close_all()
		return
	if not _dim_bg.visible:
		_pause_sim()
	_hide_all_panels()
	_building_panel.set_building_id(building_id)
	_building_panel.visible = true
	_dim_bg.visible = true
	_center_panel(_building_panel, 0.45, 0.5)


func close_all() -> void:
	_hide_all_panels()
	_dim_bg.visible = false
	_resume_sim()


func is_any_visible() -> bool:
	return _dim_bg.visible


func is_detail_visible() -> bool:
	if _entity_panel != null and _entity_panel.visible:
		return true
	if _building_panel != null and _building_panel.visible:
		return true
	return false


func _process(_delta: float) -> void:
	if not _dim_bg.visible:
		return
	# Auto-close if all panels hid themselves (e.g. entity died)
	var any: bool = false
	if _stats_panel != null and _stats_panel.visible:
		any = true
	if _entity_panel != null and _entity_panel.visible:
		any = true
	if _building_panel != null and _building_panel.visible:
		any = true
	if not any:
		close_all()


func _hide_all_panels() -> void:
	if _stats_panel != null:
		_stats_panel.visible = false
	if _entity_panel != null:
		_entity_panel.visible = false
	if _building_panel != null:
		_building_panel.visible = false


func _pause_sim() -> void:
	if _sim_engine != null:
		_was_paused = _sim_engine.is_paused
		if not _was_paused:
			_sim_engine.is_paused = true
			SimulationBus.pause_changed.emit(true)


func _resume_sim() -> void:
	if _sim_engine != null and not _was_paused:
		_sim_engine.is_paused = false
		SimulationBus.pause_changed.emit(false)


func _center_panel(panel: Control, w_ratio: float, h_ratio: float) -> void:
	var vp := get_viewport().get_visible_rect().size
	var pw: float = vp.x * w_ratio
	var ph: float = vp.y * h_ratio
	panel.position = Vector2((vp.x - pw) * 0.5, (vp.y - ph) * 0.5)
	panel.size = Vector2(pw, ph)
