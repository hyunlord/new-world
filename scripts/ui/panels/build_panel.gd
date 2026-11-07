# Catalogue-driven template groups, material gates, housing zones and construction sources.
extends PanelContainer

signal template_chosen(row: Dictionary, station: String)
signal housing_requested
signal source_requested(site: int, record: int)

const ItemIcons = preload("res://scripts/ui/item_icons.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
const GROUPS: Array[String] = ["BUILD_GROUP_BUILT", "BUILD_GROUP_AVAILABLE", "BUILD_GROUP_NEXT"]

var _world: Node
var _locale: Node
var _catalog: Array[Dictionary] = []
var _groups: Array[VBoxContainer] = []
var _group_labels: Array[Label] = []
var _heading: Label
var _housing: Button
var _zone_controls: HFlowContainer
var _zone_buttons: Array[Button] = []
var _close: Button
var _last_result: Dictionary = {}
var _detail: VBoxContainer
var _stations: OptionButton
var _place: Button
var _upgrade: Button
var _why: Button
var _materials: Label
var _gate: Label
var _title: Label
var _result: Label
var _snapshot: Dictionary = {}
var _selected: String = ""
var _site: int = -1
var _station: String = ""
var _elapsed: float = 0.0
var _signature: String = ""


func _ready() -> void:
	theme = preload("res://assets/ui/worldsim_theme.tres")
	mouse_filter = Control.MOUSE_FILTER_STOP
	_world = get_node("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	var column := VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	add_child(column)
	var heading := HBoxContainer.new()
	column.add_child(heading)
	var title := Label.new()
	title.text = _tr("UI_ORDER_BUILD")
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	heading.add_child(title)
	_heading = title
	var close_button := Button.new()
	close_button.text = _tr("UI_CLOSE")
	close_button.pressed.connect(hide)
	heading.add_child(close_button)
	_close = close_button
	_zone_controls = HFlowContainer.new()
	column.add_child(_zone_controls)
	_build_zones()
	var body := HBoxContainer.new()
	body.add_theme_constant_override("separation", 12)
	column.add_child(body)
	var scroll := ScrollContainer.new()
	scroll.custom_minimum_size = Vector2(496, 560)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	body.add_child(scroll)
	var listings := VBoxContainer.new()
	listings.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(listings)
	for key: String in GROUPS:
		var label := Label.new()
		label.text = _tr(key)
		listings.add_child(label)
		_group_labels.append(label)
		var group := VBoxContainer.new()
		listings.add_child(group)
		_groups.append(group)
	_detail = VBoxContainer.new()
	_detail.custom_minimum_size.x = 376
	_detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(_detail)
	_title = _label(_detail)
	_stations = OptionButton.new()
	_stations.item_selected.connect(_choose_station)
	_detail.add_child(_stations)
	_materials = _label(_detail)
	_gate = _label(_detail)
	_place = Button.new()
	_place.text = _tr("BUILD_PLACE")
	_place.pressed.connect(_choose_template)
	_detail.add_child(_place)
	_upgrade = Button.new()
	_upgrade.text = _tr("BUILD_UPGRADE")
	_upgrade.pressed.connect(_upgrade_selected)
	_detail.add_child(_upgrade)
	_why = Button.new()
	_why.text = _tr("BUILD_SOURCE")
	_why.pressed.connect(_open_source)
	_detail.add_child(_why)
	_result = _label(column)
	_world.connect("world_reset", _reset)
	if _locale != null:
		_locale.connect("language_changed", _on_language_changed)
	hide()


func _tr(key: String) -> String:
	return str(_locale.call("ltr", key)) if _locale != null else key


func _on_language_changed(_language: String) -> void:
	_heading.text = _tr("UI_ORDER_BUILD")
	_refresh_zone_labels()
	_close.text = _tr("UI_CLOSE")
	for index: int in GROUPS.size():
		_group_labels[index].text = _tr(GROUPS[index])
	_place.text = _tr("BUILD_PLACE")
	_upgrade.text = _tr("BUILD_UPGRADE")
	_why.text = _tr("BUILD_SOURCE")
	for index: int in _stations.item_count:
		var station: String = str(_stations.get_item_metadata(index))
		_stations.set_item_text(index, EconomyText.building(_locale, EconomyText.build_code(station)))
	_rebuild_lists()
	_fill_detail()
	_fill_result()


func _build_zones() -> void:
	var content: Dictionary = SimReads.read(_world, "get_content_catalog")
	for value: Variant in content.get("building_zones", []):
		if value is not Dictionary:
			continue
		var row: Dictionary = value
		var button := Button.new()
		button.name = "Zone_" + str(row["id"])
		button.set_meta("zone", row)
		_zone_controls.add_child(button)
		_zone_buttons.append(button)
		if str(row["id"]) == "housing":
			_housing = button
			if str(row.get("status", "")) == "Connected":
				button.pressed.connect(func() -> void: housing_requested.emit(); hide())
	_refresh_zone_labels()


func _refresh_zone_labels() -> void:
	for button: Button in _zone_buttons:
		var row: Dictionary = button.get_meta("zone")
		button.disabled = str(row.get("status", "")) != "Connected" or str(row["id"]) != "housing"
		button.text = _tr(str(row["name_key"]))
		button.tooltip_text = str(row.get("debt", ""))
		if button.disabled:
			button.text += " · " + _tr("BUILD_LOCK_INERT")
			button.tooltip_text = _tr("BUILD_LOCK_INERT") + "\n" + str(row.get("debt", ""))
			button.add_theme_color_override("font_disabled_color", Color("807970"))


func _fill_result() -> void:
	_result.text = ""
	if not _last_result.is_empty():
		_result.text = _tr("BUILD_UPGRADE_ACCEPTED") if bool(_last_result.get("accepted", false)) else EconomyText.reason(_locale, str(_last_result.get("reason", "nothing")))


func _label(parent: Node) -> Label:
	var label := Label.new()
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(label)
	return label


func open() -> void:
	var content: Dictionary = SimReads.read(_world, "get_content_catalog")
	_catalog.clear()
	for value: Variant in content.get("buildings", []):
		if value is Dictionary:
			_catalog.append(value)
	_signature = ""
	_refresh()
	position = (get_viewport_rect().size - Vector2(920, 672)) * 0.5
	show()


func _reset() -> void:
	_selected = ""
	_site = -1
	_station = ""
	_signature = ""
	_last_result.clear()
	_fill_result()
	hide()


func open_site(id: int) -> void:
	open()
	for value: Variant in _snapshot.get("template_sites", []):
		if value is Dictionary and int(value.get("id", -1)) == id:
			_select(str(value["template_id"]), id)
			return


func _process(delta: float) -> void:
	if not visible:
		return
	_elapsed += delta
	if _elapsed >= 0.5:
		_elapsed = 0.0
		_refresh()


func _refresh() -> void:
	_snapshot = SimReads.read(_world, "get_building_snapshot")
	var signature: String = JSON.stringify([_snapshot.get("template_sites", []), _snapshot.get("template_options", [])])
	if signature != _signature:
		_signature = signature
		_rebuild_lists()
	_fill_detail()


func _definition(id: String) -> Dictionary:
	for row: Dictionary in _catalog:
		if str(row["id"]) == id:
			return row
	return {}


func _option(id: String, station: String = "") -> Dictionary:
	var fallback: Dictionary = {}
	for value: Variant in _snapshot.get("template_options", []):
		if value is Dictionary and str(value.get("id", "")) == id:
			if not station.is_empty():
				if str(value.get("station", "")) == station:
					return value
				continue
			if fallback.is_empty():
				fallback = value
			if bool(value.get("known", false)) and bool(value.get("affordable", false)):
				return value
	return fallback


func _site_row() -> Dictionary:
	for value: Variant in _snapshot.get("template_sites", []):
		if value is Dictionary and int(value.get("id", -1)) == _site:
			return value
	return {}


func _rebuild_lists() -> void:
	for group: VBoxContainer in _groups:
		for child: Node in group.get_children():
			group.remove_child(child)
			child.queue_free()
	for value: Variant in _snapshot.get("template_sites", []):
		if value is not Dictionary:
			continue
		var site: Dictionary = value
		if str(site.get("status", "")) != "complete":
			continue
		var row: Dictionary = _definition(str(site.get("template_id", "")))
		var button: Button = _card(row, "%s · %s" % [site.get("completed_pieces", 0), site.get("total_pieces", 0)])
		button.pressed.connect(_select.bind(str(site.get("template_id", "")), int(site["id"])))
		_groups[0].add_child(button)
	for row: Dictionary in _catalog:
		var option: Dictionary = _option(str(row["id"]))
		var admitted: bool = bool(row.get("connected", false)) and bool(option.get("known", false)) and bool(option.get("affordable", false))
		var group_index: int = 1 if admitted else 2
		var button: Button = _card(row, _tr("BUILD_TIER").format({"tier": row.get("tier", 0)}))
		button.name = "Template_" + str(row["id"])
		if not admitted:
			button.add_theme_color_override("font_color", Color("807970"))
			button.tooltip_text = str(row.get("debt", "")) + "\n" + _gate_text(option)
		# Grey rows remain inspectable; only the action is disabled. Debt must be readable on click, too.
		button.pressed.connect(_choose_card.bind(str(row["id"])))
		_groups[group_index].add_child(button)


# Single-choice available cards enter placement; multi-station workshops keep the chooser.
func _choose_card(id: String) -> void:
	_refresh()
	_select(id, -1)
	if not _place.disabled and _stations.item_count <= 1:
		_choose_template()


func _card(row: Dictionary, suffix: String) -> Button:
	var button := Button.new()
	button.text = "%s · %s" % [_tr(str(row.get("name_key", "UI_UNKNOWN"))), suffix]
	button.custom_minimum_size = Vector2(472, 64)
	button.icon = ItemIcons.building_texture(row, Vector2i(48, 40), true)
	button.expand_icon = true
	button.icon_max_width = 48
	button.set_meta("template_id", str(row.get("id", "")))
	return button


func _select(id: String, site: int) -> void:
	_selected = id
	_site = site
	_station = str(_site_row().get("station", _option(id).get("station", "")))
	_stations.clear()
	for value: Variant in _snapshot.get("template_options", []):
		if value is not Dictionary or str(value.get("id", "")) != id:
			continue
		var station: String = str(value.get("station", ""))
		if station.is_empty():
			continue
		_stations.add_item(EconomyText.building(_locale, EconomyText.build_code(station)))
		_stations.set_item_metadata(_stations.item_count - 1, station)
		if station == _station:
			_stations.select(_stations.item_count - 1)
	_fill_detail()


func _choose_station(index: int) -> void:
	_station = str(_stations.get_item_metadata(index))
	_fill_detail()


func _fill_detail() -> void:
	var row: Dictionary = _definition(_selected)
	_detail.visible = not row.is_empty()
	if row.is_empty():
		return
	var option: Dictionary = _option(_selected, _station)
	var site: Dictionary = _site_row()
	_title.text = _tr(str(row["name_key"])) + " · " + _tr("BUILD_TIER").format({"tier": row.get("tier", 0)})
	_stations.visible = _stations.item_count > 0 and _site < 0
	var lines := PackedStringArray()
	for value: Variant in option.get("requirements", []):
		if value is Dictionary:
			lines.append("%s %s/%s" % [EconomyText.item(_locale, int(value["item"])), value["have"], value["need"]])
	_materials.text = "\n".join(lines)
	_gate.text = str(row.get("debt", "")) if not bool(row.get("connected", false)) else _gate_text(option)
	_place.visible = _site < 0
	_place.disabled = not bool(row.get("connected", false)) or not bool(option.get("known", false)) or not bool(option.get("affordable", false))
	_upgrade.visible = not site.is_empty() and str(site.get("status", "")) == "complete" and not str(site.get("upgrade_to", "")).is_empty()
	_upgrade.disabled = not bool(site.get("upgrade_available", false))
	var reason_key: String = str(site.get("upgrade_reason_key", ""))
	var reason: String = str(site.get("upgrade_reason", ""))
	_upgrade.tooltip_text = _tr(reason_key) if not reason_key.is_empty() else (EconomyText.reason(_locale, reason) if not reason.is_empty() else "")
	_why.visible = not site.is_empty()
	if not site.is_empty():
		_gate.text = _tr("BUILD_CAUSE_" + str(site.get("cause", "command")).to_upper())
		_gate.text += "\n" + _tr("BUILD_STATUS_" + str(site.get("status", "building")).to_upper())
		_gate.text += "\n%s/%s" % [site.get("completed_pieces", 0), site.get("total_pieces", 0)]


func _gate_text(option: Dictionary) -> String:
	var lines := PackedStringArray()
	var key: String = str(option.get("reason_key", ""))
	if not key.is_empty():
		lines.append(_tr(key))
	for value: Variant in option.get("missing_knowledge", []):
		lines.append(KnowledgeText.node_name(_locale, int(value)))
	return "\n".join(lines)


func _choose_template() -> void:
	if _place.disabled:
		return
	template_chosen.emit(_definition(_selected), _station)
	hide()


func _upgrade_selected() -> void:
	if _upgrade.disabled or _site < 0:
		return
	var raw: Variant = SimReads.mutate(_world, "cmd_upgrade_building", [_site])
	if raw is Dictionary:
		_last_result = raw
		_fill_result()
	_signature = ""
	_refresh()


func _open_source() -> void:
	var site: Dictionary = _site_row()
	if not site.is_empty():
		source_requested.emit(_site, int(site.get("source_record", -1)))


func _unhandled_input(event: InputEvent) -> void:
	if visible and event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_ESCAPE:
		hide()
		get_viewport().set_input_as_handled()
