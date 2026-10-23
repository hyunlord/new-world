# hud_topbar.gd (UI/HudTopbar in scenes/main.tscn): top bar with the date (M1.5 calendar: "3년 차 · 봄 2일째"),
#   clock (dawn/day/evening/night, aligned with the daylight tint), population, villages, shelters (V7 shelters +
#   M1 huts), sites (V7 construction sites + M1 hut blueprints), stored food and firewood stocks, logs lying in
#   stacks (M1 items), wild berries on the whole map, births, deaths, cold count, pause and display speed;
#   L toggles ko/en.
# M4 (goal 9.1): beside the calendar the era line "신석기 전환기 31 %" (get_tech `era`, `progress` ‰ as whole percent)
#   and the badge button "⚠ 기술 위험 N" (`at_risk`; red when N ≥ 1) that opens the tech map (UI/TechPanel); on the
#   lower row, while N ≥ 1, the warning "⚠ 문 만들기: 카토만 앎" naming the first at-risk node and its one holder
#   (`first_names`). This node creates the tech map (UI/TechPanel, tech_panel.gd) and the teach dialog
#   (UI/TeachPanel, teach_panel.gd) beside itself.
# M5 (goal 3.5): this node also creates the story banner (UI/StoryBanner, story_banner.gd: pressure and wolf banners,
#   the wolves' pulse, the mood legend and the drafted list) beside itself.
# M5.5 (goal §9): it also creates the neighbours panel (UI/NeighboursPanel, N), the migrant dialog (UI/MigrantPanel)
#   and the trade dialog (UI/TradePanel) beside itself, after the story banner so the dialogs draw over it.
# Read-only, localized counters from authoritative simulation snapshots.
# Bridge methods: via SimReads get_agent_snapshot, get_settlement_snapshot, get_construction_snapshot,
#   get_building_snapshot, get_time_snapshot (day_index, time_of_day, season, day_of_season, days_per_season,
#   winter_severity, year_index), get_resource_snapshot, get_tallies, get_colony_overlay, get_tech.
extends Control

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const TechPanel = preload("res://scripts/ui/panels/tech_panel.gd")
const TeachPanel = preload("res://scripts/ui/panels/teach_panel.gd")
const StoryBanner = preload("res://scripts/ui/hud/story_banner.gd")
const NeighboursPanel = preload("res://scripts/ui/panels/neighbours_panel.gd")
const MigrantPanel = preload("res://scripts/ui/panels/migrant_panel.gd")
const TradePanel = preload("res://scripts/ui/panels/trade_panel.gd")
const RISK_COLOR: Color = Color("e05a48")
const QUIET_COLOR: Color = Color("c8b89a")

const HUD_MARGIN: int = 12
const HUD_SEPARATION: int = 16
var _world_sim: Node = null
var _shelters_label: Label
var _agents_label: Label
var _settlements_label: Label
var _settlements_row: HBoxContainer
var _sites_label: Label
var _locale: Node = null
var _clock_label: Label
var _resource_labels: Array[Label] = []
var _cold_label: Label
var _cold_icon: TextureRect
var _winter_label: Label
var _wood_label: Label
var _wood_row: HBoxContainer
var _logs_label: Label
var _speed_label: Label
var _display_speed: float = 1.0
var _clock_icon: Sprite2D
var _pause_icon: Control
var _death_total: int = 0
## M4: the era line, the at-risk badge (opens the tech map) and the at-risk warning line.
var _era_label: Label
var _risk_button: Button
var _warn_label: Label
const DEATH_TALLY_KEYS: Array[String] = ["deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure"]


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")

	_locale = get_node_or_null("/root/Locale")
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var panel: PanelContainer = ArtUI.panel(self, Rect2(12, 12, 1896, 100))
	var column: VBoxContainer = VBoxContainer.new()
	panel.add_child(column)
	var hbox: HBoxContainer = HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 8)
	column.add_child(hbox)
	_clock_label = ArtUI.pair(hbox, 9)
	_clock_icon = hbox.get_child(0).get_child(0) as Sprite2D
	_agents_label = ArtUI.pair(hbox, 1)
	_settlements_row = HBoxContainer.new()
	_settlements_row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_settlements_row.add_theme_constant_override("separation", 8)
	hbox.add_child(_settlements_row)
	_settlements_label = ArtUI.pair(_settlements_row, 2)
	_shelters_label = ArtUI.pair(hbox, 3)
	_sites_label = ArtUI.pair(hbox, 4)
	_era_label = Label.new()
	_era_label.name = "EraLabel"
	_era_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_era_label.add_theme_color_override("font_color", Color("e8c46e"))
	hbox.add_child(_era_label)
	_risk_button = Button.new()
	_risk_button.name = "RiskBadge"
	_risk_button.focus_mode = Control.FOCUS_NONE
	_risk_button.flat = true
	_risk_button.mouse_filter = Control.MOUSE_FILTER_STOP
	_risk_button.pressed.connect(open_tech_map)
	hbox.add_child(_risk_button)
	var era_gap: Control = Control.new()
	era_gap.custom_minimum_size.x = 8
	era_gap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	hbox.add_child(era_gap)
	_winter_label = Label.new()
	_winter_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_winter_label.add_theme_color_override("font_color", Color("a8c0d0"))
	hbox.add_child(_winter_label)
	var lower: HBoxContainer = HBoxContainer.new()
	lower.add_theme_constant_override("separation", 8)
	column.add_child(lower)
	for index: int in [5, 6, 7, 8]:
		_resource_labels.append(ArtUI.pair(lower, index))
		if index == 5:
			_wood_row = HBoxContainer.new()
			_wood_row.mouse_filter = Control.MOUSE_FILTER_IGNORE
			_wood_row.add_theme_constant_override("separation", 8)
			lower.add_child(_wood_row)
			_wood_label = Label.new()
			_wood_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
			_wood_row.add_child(_wood_label)
			_wood_row.hide()
			var logs_gap: Control = Control.new()
			logs_gap.custom_minimum_size.x = 8
			logs_gap.mouse_filter = Control.MOUSE_FILTER_IGNORE
			lower.add_child(logs_gap)
			_logs_label = Label.new()
			_logs_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
			lower.add_child(_logs_label)
			_logs_label.hide()
			var cold_gap: Control = Control.new()
			cold_gap.custom_minimum_size.x = 8
			cold_gap.mouse_filter = Control.MOUSE_FILTER_IGNORE
			lower.add_child(cold_gap)
			_cold_label = Label.new()
			_cold_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
			lower.add_child(_cold_label)
			_cold_label.hide()
			_cold_icon = _make_cold_icon()
			lower.add_child(_cold_icon)
			_cold_icon.hide()
			var warn_gap: Control = Control.new()
			warn_gap.custom_minimum_size.x = 16
			warn_gap.mouse_filter = Control.MOUSE_FILTER_IGNORE
			lower.add_child(warn_gap)
			_warn_label = Label.new()
			_warn_label.name = "RiskWarning"
			_warn_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
			_warn_label.add_theme_color_override("font_color", RISK_COLOR)
			lower.add_child(_warn_label)
			_warn_label.hide()
	var spacer: Control = Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	lower.add_child(spacer)
	_pause_icon = ArtUI.icon(14)
	lower.add_child(_pause_icon)
	_speed_label = ArtUI.pair(lower, 15)


	for entry: Array in [["TechPanel", TechPanel], ["TeachPanel", TeachPanel], ["StoryBanner", StoryBanner], ["NeighboursPanel", NeighboursPanel],
			["MigrantPanel", MigrantPanel], ["TradePanel", TradePanel]]:
		if get_parent() != null and get_parent().get_node_or_null(str(entry[0])) == null:
			var panel_node: Control = (entry[1] as GDScript).new()
			panel_node.name = str(entry[0])
			get_parent().add_child.call_deferred(panel_node)

	# Missing WorldSim is tolerated silently — counters stay at their
	# default values. No push_error / push_warning here so the edge-case
	# (HUD instantiated before SimBridge registers) does not spam logs.

	_refresh()


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_L:
		if _locale != null:
			var current: String = str(_locale.get("_current_lang"))
			_locale.call("set_language", "en" if current == "ko" else "ko")
			_refresh()
		get_viewport().set_input_as_handled()


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	_refresh()


func _refresh() -> void:
	var agent_n: int = 0
	var settle_n: int = 0
	var site_n: int = 0
	var shelter_n: int = 0

	if _world_sim != null:
		var agent_snap: Variant = SimReads.read(_world_sim, "get_agent_snapshot")
		if agent_snap is Dictionary:
			var agent_dict: Dictionary = agent_snap
			var ids: Variant = agent_dict.get("ids", null)
			if ids is PackedInt64Array:
				var ids_arr: PackedInt64Array = ids
				agent_n = ids_arr.size()

		var settle_snap: Variant = SimReads.read(_world_sim, "get_settlement_snapshot")
		if settle_snap is Dictionary:
			var settle_dict: Dictionary = settle_snap
			var sids: Variant = settle_dict.get("ids", null)
			if sids is PackedInt64Array:
				var sids_arr: PackedInt64Array = sids
				settle_n = sids_arr.size()

		var construct_snap: Variant = SimReads.read(_world_sim, "get_construction_snapshot")
		if construct_snap is Dictionary:
			var construct_dict: Dictionary = construct_snap
			var cids: Variant = construct_dict.get("ids", null)
			if cids is PackedInt64Array:
				var cids_arr: PackedInt64Array = cids
				site_n = cids_arr.size()
		# M1 hut blueprints are ConstructionSite entities without a separate Position component, so the V7
		# construction snapshot (which joins ConstructionSite with Position) never lists them; count them here.
		var blueprint_xs: Variant = _snapshot("get_colony_overlay").get("site_xs", null)
		if blueprint_xs is PackedInt32Array:
			site_n += (blueprint_xs as PackedInt32Array).size()

		var building_snap: Variant = SimReads.read(_world_sim, "get_building_snapshot")
		if building_snap is Dictionary:
			var building_dict: Dictionary = building_snap
			var kinds: Variant = building_dict.get("kinds", null)
			if kinds is PackedByteArray:
				var kind_array: PackedByteArray = kinds
				# BuildingKind codes: 1 V7 shelter, 3 M1 hut (both are places to sleep).
				for kind: int in kind_array:
					if kind == 1 or kind == 3:
						shelter_n += 1

	if _shelters_label != null:
		_shelters_label.text = _loc("BUILDING_SHELTER") + " %d" % shelter_n
	if _agents_label != null:
		_agents_label.text = _loc("UI_POPULATION") + " %d" % agent_n
	if _settlements_label != null:
		_settlements_label.text = _loc("HUD_SETTLEMENTS_FMT") % settle_n
		_settlements_label.visible = settle_n >= 2
		_settlements_row.visible = settle_n >= 2
	if _sites_label != null:
		_sites_label.text = _loc("BUILDING_STATE_BUILDING") + " %d" % site_n

	var clock: Dictionary = _snapshot("get_time_snapshot")
	var minutes: int = clampi(roundi(float(clock.get("time_of_day", 0.0)) * 60.0), 0, 1439)
	var hour: int = int(minutes / 60.0)
	var time_text: String = "%02d:%02d · " % [hour, minutes % 60] + _loc(phase_key(hour))
	var date_text: String = _date_label(clock)
	# M1.5: the calendar date replaces the running day counter (it would contradict "봄 2일째");
	# a world without seasons keeps "N일차".
	if date_text.is_empty():
		date_text = _loc("HUD_DAY_FMT") % (int(clock.get("day_index", 0)) + 1)
	_clock_label.text = date_text + " · " + time_text
	var severity: int = int(clock.get("winter_severity", -1))
	_winter_label.visible = severity >= 0 and severity <= 2
	if _winter_label.visible:
		_winter_label.text = _loc(["HUD_WINTER_MILD", "HUD_WINTER_NORMAL", "HUD_WINTER_HARSH"][severity])
	var food: int = 0
	var settlement: Dictionary = _snapshot("get_settlement_snapshot")
	var stocks: Variant = settlement.get("food_stocks", null)
	if stocks is PackedInt32Array:
		for amount: int in stocks:
			food += amount
	var wood_stocks: Variant = settlement.get("wood_stocks", null)
	_wood_row.visible = wood_stocks is PackedInt32Array
	if wood_stocks is PackedInt32Array:
		var wood: int = 0
		for amount: int in wood_stocks:
			wood += amount
		_wood_label.text = _loc("HUD_STOCKPILE_WOOD") + " %d" % wood
	# M1 logs (ItemKind 0) lying in stacks anywhere on the map, to compare with a blueprint's "통나무 n/12".
	var logs: int = 0
	var overlay: Dictionary = _snapshot("get_colony_overlay")
	var item_kinds: Variant = overlay.get("item_kinds", null)
	var item_counts: Variant = overlay.get("item_counts", null)
	_logs_label.visible = item_counts is PackedInt32Array
	if item_kinds is PackedInt32Array and item_counts is PackedInt32Array:
		for index: int in mini(item_kinds.size(), item_counts.size()):
			if item_kinds[index] == 0:
				logs += item_counts[index]
	_logs_label.text = _loc("HUD_LOGS") + " %d" % logs
	var cold_count: int = 0
	var colds: Variant = _snapshot("get_agent_snapshot").get("colds")
	if colds is PackedByteArray:
		for marker: int in colds:
			if marker == 1:
				cold_count += 1
	_cold_label.visible = colds is PackedByteArray
	_cold_label.text = _loc("HUD_COLD") + " %d" % cold_count
	if cold_count > 0:
		_cold_label.add_theme_color_override("font_color", Color("a8c0d0"))
	else:
		_cold_label.remove_theme_color_override("font_color")
	_cold_icon.visible = cold_count > 0
	var berries: int = 0
	var resources: Dictionary = _snapshot("get_resource_snapshot")
	var kinds: Variant = resources.get("kinds", null)
	var amounts: Variant = resources.get("amounts", null)
	if kinds is PackedInt32Array and amounts is PackedInt32Array:
		for index: int in mini(kinds.size(), amounts.size()):
			if kinds[index] == 0 and amounts[index] < 255:
				berries += amounts[index]
	var tallies: Dictionary = _snapshot("get_tallies")
	var deaths: int = 0
	for key: String in DEATH_TALLY_KEYS:
		deaths += int(tallies.get(key, 0))
	_death_total = deaths
	var keys: Array[String] = ["HUD_STOCKPILE_FOOD", "HUD_BERRIES", "UI_BIRTHS", "UI_DEATHS"]
	var values: Array[int] = [food, berries, int(tallies.get("births", 0)), deaths]
	for index: int in 4:
		_resource_labels[index].text = _loc(keys[index]) + " %d" % values[index]
	# M1: the camera owns the speed (keys 1–4 = 1×–4×, 12–48 ticks per second) and the pause (P).
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	var paused: bool = camera != null and camera.has_method("is_paused") and bool(camera.call("is_paused"))
	if camera != null and camera.has_method("get_speed"):
		_display_speed = float(camera.call("get_speed"))
	_speed_label.text = _loc("UI_SPEED_PAUSED") if paused else _loc("HUD_SPEED_FMT") % str(int(_display_speed))
	_pause_icon.modulate.a = 1.0 if paused else 0.35
	_refresh_knowledge()
	ArtUI.Library.apply(_clock_icon, "U1_10" if hour >= 21 or hour < 5 else "U1_09")
	_clock_icon.offset = Vector2.ZERO
	_clock_icon.scale = Vector2(2, 2)


## M4: the era line, the badge and the at-risk warning from get_tech (hidden without knowledge).
func _refresh_knowledge() -> void:
	var tech: Dictionary = _snapshot("get_tech")
	var known: bool = not (tech.get("ids", PackedInt64Array()) as PackedInt64Array).is_empty()
	_era_label.visible = known
	_risk_button.visible = known
	_warn_label.visible = false
	if not known:
		return
	_era_label.text = KnowledgeText.era_line(_locale, tech)
	var at_risk: int = int(tech.get("at_risk", 0))
	_risk_button.text = KnowledgeText.trf(_locale, "M4_HUD_AT_RISK_FMT", {"count": at_risk})
	var colour: Color = RISK_COLOR if at_risk > 0 else QUIET_COLOR
	for state: String in ["font_color", "font_hover_color", "font_pressed_color", "font_focus_color"]:
		_risk_button.add_theme_color_override(state, colour)
	var first: Dictionary = KnowledgeText.first_at_risk(tech)
	if at_risk > 0 and not first.is_empty():
		_warn_label.text = warning_line(_locale, tech, int(first["id"]), int(first["holder"]))
		_warn_label.visible = true


## "⚠ 문 만들기: 카토만 앎" (`M4_WARN_AT_RISK`) for node `id` and its one holder `holder` (an AgentId).
static func warning_line(locale: Node, tech: Dictionary, id: int, holder: int) -> String:
	var name: String = Names.present(holder, KnowledgeText.language(locale)) if holder >= 0 else KnowledgeText.ltr(locale, "PANEL_FAMILY_NONE")
	return KnowledgeText.trf(locale, "M4_WARN_AT_RISK", {"node": KnowledgeText.node_name(locale, id, tech), "name": name})


## Open the tech map (the badge's action).
func open_tech_map() -> void:
	var panel: Node = get_parent().get_node_or_null("TechPanel") if get_parent() != null else null
	if panel != null and panel.has_method("open"):
		panel.call("open")


## The era line's text (for tests).
func era_text() -> String:
	return _era_label.text if _era_label != null else ""


## The badge's text (for tests).
func risk_text() -> String:
	return _risk_button.text if _risk_button != null else ""


## The warning line's text, "" when hidden (for tests).
func warning_text() -> String:
	return _warn_label.text if _warn_label != null and _warn_label.visible else ""


## The clock word for `hour`, aligned with daylight_controller.gd's tint: dawn 5–7, day 7–17, evening 17–21
## (the tint turns towards dusk from 17 and reaches night at 21), night 21–5.
static func phase_key(hour: int) -> String:
	if hour >= 21 or hour < 5:
		return "HUD_PHASE_NIGHT"
	if hour < 7:
		return "HUD_PHASE_DAWN"
	if hour >= 17:
		return "HUD_PHASE_EVENING"
	return "HUD_PHASE_DAY"


func _snapshot(method: String) -> Dictionary:
	if _world_sim == null:
		return {}
	var value: Variant = SimReads.read(_world_sim, method)
	return value if value is Dictionary else {}


func _loc(key: String) -> String:
	return _locale.ltr(key) if _locale != null else key





## The calendar date, e.g. "3년 차 · 봄 2일째" / "Year 3 · Spring, day 2"; "" without a calendar.
## Bridge contract: year_index and season are zero-based; day_of_season is one-based.
func _date_label(clock: Dictionary) -> String:
	if not clock.has("season") or not clock.has("day_of_season") or int(clock.get("days_per_season", 0)) <= 0:
		return ""
	var season: int = int(clock["season"])
	var day: int = int(clock["day_of_season"])
	var keys: Array[String] = ["HUD_SEASON_SPRING", "HUD_SEASON_SUMMER", "HUD_SEASON_AUTUMN", "HUD_SEASON_WINTER"]
	if season < 0 or season >= keys.size() or day < 1 or day > int(clock["days_per_season"]):
		return ""
	var year: int = maxi(0, int(clock.get("year_index", 0))) + 1
	var season_text: String
	if int(clock.get("winter_severity", -1)) >= 0 and season != 3:
		# From 00:00 to the burn end the night still belongs to the evening's winter; when the day has
		# rolled into spring the year has too, so the night belongs to the previous year.
		if season == 0:
			year = maxi(1, year - 1)
		season_text = _loc("HUD_SEASON_LAST_NIGHT_FMT") % [_loc("HUD_SEASON_WINTER"), int(clock["days_per_season"])]
	else:
		season_text = _loc("HUD_SEASON_DAY_FMT") % [_loc(keys[season]), day]
	var year_text: String = _loc("UI_TIME_YEAR_FMT").format({"year": year}) if clock.has("year_index") else ""
	return season_text if year_text.is_empty() else year_text + " · " + season_text


func _make_cold_icon() -> TextureRect:
	var pixels: Image = Image.create(9, 9, false, Image.FORMAT_RGBA8)
	pixels.fill(Color.TRANSPARENT)
	for coordinate: int in range(1, 8):
		pixels.set_pixel(4, coordinate, Color("a8c0d0"))
		pixels.set_pixel(coordinate, 4, Color("a8c0d0"))
	for coordinate: int in [2, 3, 5, 6]:
		pixels.set_pixel(coordinate, coordinate, Color("a8c0d0"))
		pixels.set_pixel(coordinate, 8 - coordinate, Color("a8c0d0"))
	var icon: TextureRect = TextureRect.new()
	icon.texture = ImageTexture.create_from_image(pixels)
	icon.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	icon.custom_minimum_size = Vector2(18, 18)
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return icon


func get_death_total() -> int:
	return _death_total
