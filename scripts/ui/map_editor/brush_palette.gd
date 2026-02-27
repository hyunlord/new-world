## BrushPalette — 에디터 왼쪽 사이드바 UI
## Control 노드로 동작. WorldSetup이 add_child()하고
## controller 참조를 set_controller()로 주입.
extends Control

## 현재 선택 상태를 알리는 시그널 (WorldSetup이 listen)
signal brush_mode_changed(mode: int)
signal brush_biome_changed(biome: int)
signal brush_size_changed(size: int)
signal brush_strength_changed(strength: float)
signal spawn_count_changed(count: int)
signal preset_selected(preset_id: String)
signal seed_changed(seed_value: int)
signal regenerate_requested
signal start_simulation_requested
signal resource_overlay_toggled

var _controller = null ## MapEditorController ref (untyped, injected by WorldSetup)

## 바이옴 이름 → 버튼 색 (BIOME_COLORS와 대응)
const _BIOME_COLORS: Array[Color] = [
	Color(0.05, 0.1, 0.4),   ## 0 DEEP_WATER
	Color(0.2, 0.4, 0.7),    ## 1 SHALLOW_WATER
	Color(0.85, 0.82, 0.55), ## 2 BEACH
	Color(0.3, 0.7, 0.3),    ## 3 GRASSLAND
	Color(0.15, 0.5, 0.15),  ## 4 FOREST
	Color(0.05, 0.3, 0.1),   ## 5 DENSE_FOREST
	Color(0.55, 0.5, 0.35),  ## 6 HILL
	Color(0.6, 0.6, 0.65),   ## 7 MOUNTAIN
	Color(0.9, 0.95, 1.0),   ## 8 SNOW
]

const _BIOME_KEYS: Array[String] = [
	"UI_MAP_BIOME_DEEP_WATER",
	"UI_MAP_BIOME_SHALLOW_WATER",
	"UI_MAP_BIOME_BEACH",
	"UI_MAP_BIOME_GRASSLAND",
	"UI_MAP_BIOME_FOREST",
	"UI_MAP_BIOME_DENSE_FOREST",
	"UI_MAP_BIOME_HILL",
	"UI_MAP_BIOME_MOUNTAIN",
	"UI_MAP_BIOME_SNOW",
]

## 현재 선택된 바이옴 인덱스
var _selected_biome: int = 3 ## GRASSLAND

## seed LineEdit 참조
var _seed_edit: LineEdit = null
## 스폰 총 인원 표시 Label
var _spawn_total_label: Label = null


func _ready() -> void:
	_build_ui()


## Description
## MapEditorController를 주입한다.
func set_controller(controller) -> void:
	_controller = controller


# ── UI 구성 ─────────────────────────────────────────────────


func _build_ui() -> void:
	## 배경 패널
	custom_minimum_size = Vector2(200, 0)
	var panel: PanelContainer = PanelContainer.new()
	panel.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(panel)

	var vbox: VBoxContainer = VBoxContainer.new()
	vbox.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	panel.add_child(vbox)

	## ── 프리셋 섹션 ──
	var preset_lbl: Label = Label.new()
	preset_lbl.text = Locale.ltr("UI_MAP_EDITOR_TITLE")
	vbox.add_child(preset_lbl)

	var preset_hbox: HBoxContainer = HBoxContainer.new()
	vbox.add_child(preset_hbox)
	for preset_id: String in ["island", "continent", "archipelago", "random"]:
		var key: String = "UI_MAP_PRESET_" + preset_id.to_upper()
		var preset_btn: Button = Button.new()
		preset_btn.text = Locale.ltr(key)
		preset_btn.pressed.connect(_on_preset_btn.bind(preset_id))
		preset_hbox.add_child(preset_btn)

	## Seed
	var seed_hbox: HBoxContainer = HBoxContainer.new()
	vbox.add_child(seed_hbox)
	var seed_lbl: Label = Label.new()
	seed_lbl.text = Locale.ltr("UI_MAP_SEED_LABEL")
	seed_hbox.add_child(seed_lbl)
	_seed_edit = LineEdit.new()
	_seed_edit.placeholder_text = "0"
	_seed_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	seed_hbox.add_child(_seed_edit)
	var regen_btn: Button = Button.new()
	regen_btn.text = Locale.ltr("UI_MAP_REGENERATE")
	regen_btn.pressed.connect(_on_regenerate)
	seed_hbox.add_child(regen_btn)

	## ── 브러시 모드 섹션 ──
	var brush_lbl: Label = Label.new()
	brush_lbl.text = Locale.ltr("UI_MAP_BRUSH_TERRAIN")
	vbox.add_child(brush_lbl)

	var mode_hbox: HBoxContainer = HBoxContainer.new()
	vbox.add_child(mode_hbox)
	var modes: Array = [
		["UI_MAP_BRUSH_TERRAIN", 0],
		["UI_MAP_BRUSH_FOOD", 1],
		["UI_MAP_BRUSH_WOOD", 2],
		["UI_MAP_BRUSH_STONE", 3],
		["UI_MAP_BRUSH_SPAWN", 4],
		["UI_MAP_BRUSH_ERASE", 5],
	]
	for entry: Array in modes:
		var mode_btn: Button = Button.new()
		mode_btn.text = Locale.ltr(entry[0])
		mode_btn.toggle_mode = true
		mode_btn.pressed.connect(_on_mode_btn.bind(entry[1]))
		mode_hbox.add_child(mode_btn)

	## ── 바이옴 팔레트 (TERRAIN 모드 전용) ──
	var biome_grid: GridContainer = GridContainer.new()
	biome_grid.columns = 3
	vbox.add_child(biome_grid)
	for i: int in range(9):
		var biome_btn: Button = Button.new()
		biome_btn.text = Locale.ltr(_BIOME_KEYS[i])
		biome_btn.add_theme_color_override("font_color", _BIOME_COLORS[i])
		biome_btn.pressed.connect(_on_biome_btn.bind(i))
		biome_grid.add_child(biome_btn)

	## ── 브러시 크기 슬라이더 ──
	var size_lbl: Label = Label.new()
	size_lbl.text = Locale.ltr("UI_MAP_BRUSH_SIZE")
	vbox.add_child(size_lbl)
	var size_slider: HSlider = HSlider.new()
	size_slider.min_value = GameConfig.MAP_EDITOR_BRUSH_MIN
	size_slider.max_value = GameConfig.MAP_EDITOR_BRUSH_MAX
	size_slider.value = GameConfig.MAP_EDITOR_BRUSH_DEFAULT
	size_slider.step = 1
	size_slider.value_changed.connect(_on_size_changed)
	vbox.add_child(size_slider)

	## ── 강도 슬라이더 ──
	var str_lbl: Label = Label.new()
	str_lbl.text = Locale.ltr("UI_MAP_BRUSH_STRENGTH")
	vbox.add_child(str_lbl)
	var str_slider: HSlider = HSlider.new()
	str_slider.min_value = GameConfig.MAP_EDITOR_STRENGTH_MIN
	str_slider.max_value = GameConfig.MAP_EDITOR_STRENGTH_MAX
	str_slider.value = 1.0
	str_slider.step = 0.1
	str_slider.value_changed.connect(_on_strength_changed)
	vbox.add_child(str_slider)

	## ── 스폰 인원 수 ──
	var spawn_lbl: Label = Label.new()
	spawn_lbl.text = Locale.ltr("UI_MAP_SPAWN_COUNT")
	vbox.add_child(spawn_lbl)
	var spawn_spin: SpinBox = SpinBox.new()
	spawn_spin.min_value = GameConfig.MAP_EDITOR_SPAWN_MIN
	spawn_spin.max_value = GameConfig.MAP_EDITOR_SPAWN_MAX
	spawn_spin.value = GameConfig.MAP_EDITOR_SPAWN_DEFAULT
	spawn_spin.value_changed.connect(_on_spawn_count_changed)
	vbox.add_child(spawn_spin)

	## ── ResourceOverlay 토글 ──
	var overlay_btn: Button = Button.new()
	overlay_btn.text = Locale.ltr("UI_MAP_OVERLAY_TOGGLE")
	overlay_btn.toggle_mode = true
	overlay_btn.pressed.connect(_on_overlay_toggle)
	vbox.add_child(overlay_btn)

	## ── 스폰 총 인원 표시 ──
	_spawn_total_label = Label.new()
	_spawn_total_label.text = Locale.trf("UI_MAP_SPAWN_TOTAL", {"count": 0})
	_spawn_total_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(_spawn_total_label)

	## ── 시작 버튼 ──
	var start_btn: Button = Button.new()
	start_btn.text = Locale.ltr("UI_MAP_START_SIM")
	start_btn.pressed.connect(_on_start_pressed)
	vbox.add_child(start_btn)


# ── 시그널 핸들러 ────────────────────────────────────────────


func _on_preset_btn(preset_id: String) -> void:
	preset_selected.emit(preset_id)


func _on_regenerate() -> void:
	var seed_val: int = int(_seed_edit.text) if _seed_edit.text.is_valid_int() else randi()
	seed_changed.emit(seed_val)
	regenerate_requested.emit()


func _on_mode_btn(mode: int) -> void:
	if _controller != null:
		_controller.brush_mode = mode
	brush_mode_changed.emit(mode)


func _on_biome_btn(biome: int) -> void:
	_selected_biome = biome
	if _controller != null:
		_controller.brush_biome = biome
	brush_biome_changed.emit(biome)


func _on_size_changed(value: float) -> void:
	if _controller != null:
		_controller.brush_size = int(value)
	brush_size_changed.emit(int(value))


func _on_strength_changed(value: float) -> void:
	if _controller != null:
		_controller.brush_strength = value
	brush_strength_changed.emit(value)


func _on_spawn_count_changed(value: float) -> void:
	if _controller != null:
		_controller.spawn_count = int(value)
	spawn_count_changed.emit(int(value))


func _on_overlay_toggle() -> void:
	resource_overlay_toggled.emit()


func _on_start_pressed() -> void:
	start_simulation_requested.emit()


## world_setup.gd에서 스폰 포인트 변경 시 호출
func update_spawn_total(total: int) -> void:
	if _spawn_total_label != null:
		_spawn_total_label.text = Locale.trf("UI_MAP_SPAWN_TOTAL", {"count": total})
