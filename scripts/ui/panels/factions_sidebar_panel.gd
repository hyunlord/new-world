extends PanelContainer

var _sim_engine: RefCounted
var _ui_built: bool = false
var _refresh_timer: float = 0.0

var _scroll: ScrollContainer
var _content: VBoxContainer
var _title_label: Label
var _bands_container: VBoxContainer

const COLOR_BG: Color = Color(0.05, 0.07, 0.10, 0.92)
const COLOR_SECTION: Color = Color(0.16, 0.22, 0.28)
const COLOR_LABEL: Color = Color(0.50, 0.58, 0.65)
const COLOR_VALUE: Color = Color(0.85, 0.82, 0.75)
const COLOR_BAND: Color = Color(0.78, 0.56, 0.19)


func init(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine


func _ensure_ui() -> void:
	if _ui_built:
		return
	_build_ui()
	_ui_built = true


func _ready() -> void:
	_ensure_ui()


func _process(delta: float) -> void:
	if not visible:
		return
	_ensure_ui()
	_refresh_timer += delta
	if _refresh_timer >= 2.0:
		_refresh_timer = 0.0
		_refresh()


func _build_ui() -> void:
	var style := StyleBoxFlat.new()
	style.bg_color = COLOR_BG
	style.content_margin_left = 10
	style.content_margin_right = 10
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	add_theme_stylebox_override("panel", style)

	_scroll = ScrollContainer.new()
	_scroll.set_anchors_preset(Control.PRESET_FULL_RECT)
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(_scroll)

	_content = VBoxContainer.new()
	_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_content.add_theme_constant_override("separation", 4)
	_scroll.add_child(_content)

	_title_label = Label.new()
	_title_label.add_theme_font_size_override("font_size", 12)
	_title_label.add_theme_color_override("font_color", Color.WHITE)
	_content.add_child(_title_label)

	_bands_container = VBoxContainer.new()
	_bands_container.add_theme_constant_override("separation", 6)
	_content.add_child(_bands_container)


func _refresh() -> void:
	if _sim_engine == null or _bands_container == null:
		return
	for child in _bands_container.get_children():
		child.queue_free()

	var bands: Array = []
	if _sim_engine.has_method("get_band_list"):
		bands = _sim_engine.get_band_list()

	_title_label.text = "%s (%d)" % [Locale.ltr("UI_TAB_FACTIONS_BANDS"), bands.size()]

	if bands.is_empty():
		var empty := Label.new()
		empty.text = Locale.ltr("UI_NO_BANDS")
		empty.add_theme_font_size_override("font_size", 10)
		empty.add_theme_color_override("font_color", COLOR_LABEL)
		_bands_container.add_child(empty)
		return

	for band_raw: Variant in bands:
		if not (band_raw is Dictionary):
			continue
		var band: Dictionary = band_raw
		var band_name: String = str(band.get("name", Locale.ltr("UI_UNKNOWN")))
		var member_count: int = int(band.get("member_count", 0))
		var leader_name: String = str(band.get("leader_name", ""))
		var is_promoted: bool = bool(band.get("is_promoted", false))

		var card := PanelContainer.new()
		var card_style := StyleBoxFlat.new()
		card_style.bg_color = Color(0.08, 0.10, 0.14, 0.6)
		card_style.corner_radius_top_left = 3
		card_style.corner_radius_top_right = 3
		card_style.corner_radius_bottom_left = 3
		card_style.corner_radius_bottom_right = 3
		card_style.content_margin_left = 6
		card_style.content_margin_right = 6
		card_style.content_margin_top = 4
		card_style.content_margin_bottom = 4
		card.add_theme_stylebox_override("panel", card_style)
		_bands_container.add_child(card)

		var vbox := VBoxContainer.new()
		vbox.add_theme_constant_override("separation", 1)
		card.add_child(vbox)

		# Band name + status
		var name_label := Label.new()
		var status: String = Locale.ltr("UI_BAND_PROMOTED") if is_promoted else Locale.ltr("UI_BAND_PROVISIONAL")
		name_label.text = "%s [%s] %d%s" % [band_name, status, member_count, Locale.ltr("UI_MEMBERS_SUFFIX")]
		name_label.add_theme_font_size_override("font_size", 10)
		name_label.add_theme_color_override("font_color", COLOR_BAND)
		vbox.add_child(name_label)

		# Leader
		if not leader_name.is_empty():
			var leader_label := Label.new()
			leader_label.text = "  %s: %s" % [Locale.ltr("UI_LEADER"), leader_name]
			leader_label.add_theme_font_size_override("font_size", 9)
			leader_label.add_theme_color_override("font_color", COLOR_LABEL)
			vbox.add_child(leader_label)
