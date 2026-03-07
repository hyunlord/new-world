extends Control
class_name NotificationManager

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")

const NotificationCardScene = preload("res://scenes/ui/notification_card.tscn")
const CrisisBannerScene = preload("res://scenes/ui/crisis_banner.tscn")
const EventLogPanelClass = preload("res://scripts/ui/event_log_panel.gd")

signal notification_clicked(entity_id: int, position: Vector2)
signal crisis_occurred(entity_id: int, position: Vector2)
signal notification_dismissed(notification_id: int)

const POOL_SIZE: int = 30
const DRAMA_CARD_WIDTH: float = 280.0
const DRAMA_CARD_HEIGHT: float = 72.0
const DRAMA_CARD_GAP: float = 10.0
const DRAMA_LAYER_RIGHT: float = 16.0
const DRAMA_LAYER_TOP: float = 124.0

var _sim_engine: RefCounted
var _drama_layer: Control
var _milestone_panel: PanelContainer
var _milestone_message: Label
var _milestone_time: Label
var _milestone_entity_id: int = -1
var _milestone_position: Vector2 = Vector2.ZERO
var _card_pool: Array = []
var _crisis_banner = null
var _event_log = null


func init(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_build_drama_layer()
	_build_milestone_toast()
	_build_crisis_banner()
	_build_event_log()
	_build_card_pool()


func _process(_delta: float) -> void:
	var notifications: Array = SimBridge.drain_notifications()
	for raw_notification: Variant in notifications:
		if raw_notification is Dictionary:
			_consume_notification(raw_notification)


func toggle_event_log() -> void:
	if _event_log != null:
		_event_log.toggle_panel()


func is_event_log_visible() -> bool:
	return _event_log != null and _event_log.visible


func close_event_log() -> void:
	if _event_log != null:
		_event_log.visible = false


func refresh_locale() -> void:
	if _crisis_banner != null:
		_crisis_banner.refresh_locale()
	if _event_log != null:
		_event_log.refresh_locale()


func _consume_notification(notif_data: Dictionary) -> void:
	if _event_log != null:
		_event_log.add_entry(notif_data)

	match int(notif_data.get("tier", 3)):
		0:
			_show_crisis(notif_data)
		1:
			_show_drama(notif_data)
		2:
			_show_milestone(notif_data)
		_:
			pass


func _build_drama_layer() -> void:
	_drama_layer = Control.new()
	_drama_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	_drama_layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_drama_layer)


func _build_milestone_toast() -> void:
	_milestone_panel = PanelContainer.new()
	_milestone_panel.visible = false
	_milestone_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	_milestone_panel.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	_milestone_panel.offset_left = -316
	_milestone_panel.offset_top = 68
	_milestone_panel.offset_right = -16
	_milestone_panel.offset_bottom = 122
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.13, 0.18, 0.11, 0.95)
	style.border_color = Color(0.77, 0.86, 0.35, 0.7)
	style.border_width_left = 1
	style.border_width_top = 1
	style.border_width_right = 1
	style.border_width_bottom = 1
	style.corner_radius_top_left = 5
	style.corner_radius_top_right = 5
	style.corner_radius_bottom_left = 5
	style.corner_radius_bottom_right = 5
	style.content_margin_left = 10
	style.content_margin_right = 10
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	_milestone_panel.add_theme_stylebox_override("panel", style)
	add_child(_milestone_panel)

	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 4)
	_milestone_panel.add_child(box)

	_milestone_message = Label.new()
	_milestone_message.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_milestone_message.add_theme_font_size_override("font_size", GameConfig.get_font_size("popup_body"))
	_milestone_message.add_theme_color_override("font_color", Color(0.95, 0.96, 0.88))
	box.add_child(_milestone_message)

	_milestone_time = Label.new()
	_milestone_time.add_theme_font_size_override("font_size", GameConfig.get_font_size("popup_small"))
	_milestone_time.add_theme_color_override("font_color", Color(0.65, 0.75, 0.58))
	box.add_child(_milestone_time)

	_milestone_panel.gui_input.connect(_on_milestone_gui_input)


func _build_crisis_banner() -> void:
	_crisis_banner = CrisisBannerScene.instantiate()
	_crisis_banner.visible = false
	_crisis_banner.activated.connect(_on_banner_activated)
	_crisis_banner.dismissed.connect(_on_banner_dismissed)
	add_child(_crisis_banner)


func _build_event_log() -> void:
	_event_log = EventLogPanelClass.new()
	_event_log.entry_clicked.connect(_on_event_log_entry_clicked)
	add_child(_event_log)


func _build_card_pool() -> void:
	for _index: int in range(POOL_SIZE):
		var card = NotificationCardScene.instantiate()
		card.visible = false
		card.clicked.connect(_on_card_clicked)
		_drama_layer.add_child(card)
		_card_pool.append(card)


func _show_crisis(notif_data: Dictionary) -> void:
	_crisis_banner.show_notification(notif_data)
	_crisis_banner.position.y = -float(_crisis_banner.size.y)
	var tween: Tween = create_tween()
	tween.tween_property(_crisis_banner, "position:y", 0.0, 0.28).from(-80.0)
	crisis_occurred.emit(
		int(notif_data.get("primary_entity", -1)),
		Vector2(float(notif_data.get("position_x", 0.0)), float(notif_data.get("position_y", 0.0)))
	)


func _show_drama(notif_data: Dictionary) -> void:
	var card = _get_pooled_card()
	if card == null:
		return
	card.reset()
	card.setup(notif_data)
	card.size = Vector2(DRAMA_CARD_WIDTH, DRAMA_CARD_HEIGHT)
	card.visible = true
	_reflow_drama_cards()
	var target_position: Vector2 = card.position
	card.position = target_position + Vector2(320.0, 0.0)
	var tween: Tween = create_tween()
	tween.tween_property(card, "position:x", target_position.x, 0.30).from(card.position.x)
	get_tree().create_timer(30.0).timeout.connect(_return_to_pool.bind(card))


func _show_milestone(notif_data: Dictionary) -> void:
	_milestone_entity_id = int(notif_data.get("primary_entity", -1))
	_milestone_position = Vector2(
		float(notif_data.get("position_x", 0.0)),
		float(notif_data.get("position_y", 0.0))
	)
	_milestone_message.text = str(notif_data.get("message", ""))
	_milestone_time.text = GameCalendar.format_short_datetime(int(notif_data.get("tick", 0)))
	_milestone_panel.visible = true
	_milestone_panel.modulate.a = 0.0
	var tween: Tween = create_tween()
	tween.tween_property(_milestone_panel, "modulate:a", 1.0, 0.25)
	tween.tween_interval(8.0)
	tween.tween_property(_milestone_panel, "modulate:a", 0.0, 0.45)
	tween.tween_callback(Callable(self, "_hide_milestone"))


func _hide_milestone() -> void:
	_milestone_panel.visible = false
	_milestone_entity_id = -1
	_milestone_position = Vector2.ZERO


func _get_pooled_card():
	for card in _card_pool:
		if not card.visible:
			return card
	return null


func _return_to_pool(card) -> void:
	if card == null or not is_instance_valid(card) or not card.visible:
		return
	var notification_id: int = card.notification_id()
	card.reset()
	_reflow_drama_cards()
	notification_dismissed.emit(notification_id)


func _reflow_drama_cards() -> void:
	var visible_cards: Array = []
	for card in _card_pool:
		if card.visible:
			visible_cards.append(card)
	for index: int in range(visible_cards.size()):
		var target_position := Vector2(
			get_viewport().get_visible_rect().size.x - DRAMA_CARD_WIDTH - DRAMA_LAYER_RIGHT,
			DRAMA_LAYER_TOP + index * (DRAMA_CARD_HEIGHT + DRAMA_CARD_GAP)
		)
		visible_cards[index].position = target_position


func _on_card_clicked(entity_id: int, target_position: Vector2) -> void:
	notification_clicked.emit(entity_id, target_position)


func _on_banner_activated(entity_id: int, target_position: Vector2) -> void:
	notification_clicked.emit(entity_id, target_position)
	if _crisis_banner != null:
		var notification_id: int = _crisis_banner.notification_id()
		_crisis_banner.hide_banner()
		notification_dismissed.emit(notification_id)


func _on_banner_dismissed(notification_id: int) -> void:
	notification_dismissed.emit(notification_id)


func _on_event_log_entry_clicked(entity_id: int, target_position: Vector2) -> void:
	notification_clicked.emit(entity_id, target_position)


func _on_milestone_gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		notification_clicked.emit(_milestone_entity_id, _milestone_position)
		accept_event()
