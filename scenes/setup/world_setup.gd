## WorldSetup — 게임 시작 전 세계 선택/편집 씬
## Node2D로 동작. main.gd에서 add_child()하고,
## setup_confirmed 신호를 받아 시뮬레이션 시작.
extends Node2D

## 설정 완료 시그널. spawn_data: Array[{position:Vector2i, count:int}]
signal setup_confirmed(spawn_data: Array)

## 의존 스크립트 사전 로드 (class_name 없는 RefCounted 스크립트들)
const PresetMapGeneratorScript = preload("res://scripts/core/world/preset_map_generator.gd")
const MapEditorControllerScript = preload("res://scripts/ui/map_editor/map_editor_controller.gd")
const BrushPaletteScript = preload("res://scripts/ui/map_editor/brush_palette.gd")

## 내부 참조
var _world_data = null ## WorldData (main.gd에서 주입)
var _resource_map = null ## ResourceMap (main.gd에서 주입)

var _preset_gen = null ## PresetMapGeneratorScript 인스턴스
var _editor_ctrl = null ## MapEditorControllerScript 인스턴스

var _world_renderer: Sprite2D = null
var _camera: Camera2D = null
var _brush_palette = null ## BrushPaletteScript 인스턴스 (Control)
var _canvas_layer: CanvasLayer = null

var _current_preset: String = "island"
var _current_seed: int = GameConfig.PRESET_SEED_ISLAND
var _is_painting: bool = false

## 카메라 pan 상태
var _is_panning: bool = false
var _pan_start_mouse: Vector2 = Vector2.ZERO
var _pan_start_cam: Vector2 = Vector2.ZERO

## 스폰 포인트 시각화 오버레이
var _spawn_overlay: Node2D = null


## main.gd에서 WorldData + ResourceMap 주입 후 setup() 호출
func setup(world_data, resource_map) -> void:
	_world_data = world_data
	_resource_map = resource_map


func _ready() -> void:
	_build_scene()
	## 초기 프리셋 로드 (섬)
	_apply_preset("island")


# ── 씬 구성 ─────────────────────────────────────────────────


func _build_scene() -> void:
	## WorldRenderer (Sprite2D)
	_world_renderer = Sprite2D.new()
	var wr_script = load("res://scripts/ui/renderers/world_renderer.gd")
	_world_renderer.set_script(wr_script)
	add_child(_world_renderer)

	## Camera2D (에디터 카메라 — CameraController 없이 단순 pan/zoom)
	_camera = Camera2D.new()
	_camera.enabled = true
	_camera.position = Vector2(
		GameConfig.WORLD_SIZE.x * GameConfig.TILE_SIZE / 2.0,
		GameConfig.WORLD_SIZE.y * GameConfig.TILE_SIZE / 2.0
	)
	add_child(_camera)

	## CanvasLayer (UI 오버레이)
	_canvas_layer = CanvasLayer.new()
	_canvas_layer.layer = 1
	add_child(_canvas_layer)

	## BrushPalette (왼쪽 사이드바)
	_brush_palette = BrushPaletteScript.new()
	_brush_palette.set_anchors_and_offsets_preset(Control.PRESET_LEFT_WIDE)
	_brush_palette.custom_minimum_size = Vector2(200, 0)
	_canvas_layer.add_child(_brush_palette)
	_brush_palette.preset_selected.connect(_on_preset_selected)
	_brush_palette.regenerate_requested.connect(_on_regenerate)
	_brush_palette.seed_changed.connect(_on_seed_changed)
	_brush_palette.start_simulation_requested.connect(_on_start_pressed)

	## PresetMapGenerator
	_preset_gen = PresetMapGeneratorScript.new()

	## 스폰 포인트 시각화 오버레이 (월드 좌표계)
	_spawn_overlay = Node2D.new()
	add_child(_spawn_overlay)

	## MapEditorController
	_editor_ctrl = MapEditorControllerScript.new()
	_editor_ctrl.setup(_world_data, _resource_map, _world_renderer)
	_brush_palette.set_controller(_editor_ctrl)

	## 시그널 연결: 자원 브러시 → 오버레이 갱신
	_editor_ctrl.resource_changed.connect(_on_resource_changed)
	## 시그널 연결: ResourceOverlay 토글 버튼
	_brush_palette.resource_overlay_toggled.connect(_on_resource_overlay_toggled)


# ── 프리셋 처리 ──────────────────────────────────────────────


func _apply_preset(preset_id: String) -> void:
	_current_preset = preset_id
	match preset_id:
		"island":
			_current_seed = GameConfig.PRESET_SEED_ISLAND
		"continent":
			_current_seed = GameConfig.PRESET_SEED_CONTINENT
		"archipelago":
			_current_seed = GameConfig.PRESET_SEED_ARCHIPELAGO
		"random":
			_current_seed = randi()
	_generate_and_render()


func _generate_and_render() -> void:
	if _world_data == null or _resource_map == null:
		return
	_preset_gen.generate_preset(_world_data, _resource_map, _current_preset, _current_seed)
	_world_renderer.render_world(_world_data, _resource_map)
	_editor_ctrl.setup(_world_data, _resource_map, _world_renderer)
	_editor_ctrl.clear_spawn_points()


# ── 입력 처리 ────────────────────────────────────────────────


func _unhandled_input(event: InputEvent) -> void:
	## 마우스 버튼 시작
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT:
			_is_painting = event.pressed
			if event.pressed:
				var tile = _editor_ctrl.screen_to_tile(_camera, event.position)
				_editor_ctrl.paint(tile)
				_refresh_spawn_visual()
		elif event.button_index == MOUSE_BUTTON_MIDDLE:
			_is_panning = event.pressed
			if event.pressed:
				_pan_start_mouse = event.position
				_pan_start_cam = _camera.position
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_camera.zoom *= 1.1
			_camera.zoom = _camera.zoom.clamp(Vector2(0.5, 0.5), Vector2(4.0, 4.0))
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_camera.zoom *= 0.9
			_camera.zoom = _camera.zoom.clamp(Vector2(0.5, 0.5), Vector2(4.0, 4.0))

	## 마우스 드래그
	elif event is InputEventMouseMotion:
		if _is_panning:
			var delta: Vector2 = event.position - _pan_start_mouse
			_camera.position = _pan_start_cam - delta / _camera.zoom
		elif _is_painting:
			var tile = _editor_ctrl.screen_to_tile(_camera, event.position)
			_editor_ctrl.paint(tile)
			_refresh_spawn_visual()


# ── 시그널 핸들러 ────────────────────────────────────────────


func _on_preset_selected(preset_id: String) -> void:
	_apply_preset(preset_id)


func _on_regenerate() -> void:
	_generate_and_render()


func _on_seed_changed(seed_val: int) -> void:
	_current_seed = seed_val


func _on_start_pressed() -> void:
	var spawn_data: Array = _editor_ctrl.get_spawn_points()
	## 스폰 포인트 없으면 PresetMapGenerator 추천 사용
	if spawn_data.is_empty():
		var suggestions: Array = _preset_gen.get_spawn_suggestions(_world_data, _resource_map)
		for s in suggestions:
			spawn_data.append({position = s, count = GameConfig.MAP_EDITOR_SPAWN_DEFAULT})
	## 그래도 없으면 맵 중앙 walkable 탐색
	if spawn_data.is_empty():
		var center: Vector2i = _find_center_walkable()
		if center != Vector2i(-1, -1):
			spawn_data.append({position = center, count = GameConfig.MAP_EDITOR_SPAWN_DEFAULT})
		else:
			## walkable 타일 없음 — 경고 (시작 금지)
			push_warning("WorldSetup: No walkable tiles. Cannot start simulation.")
			return
	setup_confirmed.emit(spawn_data)


func _on_resource_changed() -> void:
	if _world_renderer != null and _world_renderer.has_method("update_resource_overlay"):
		_world_renderer.update_resource_overlay()


func _on_resource_overlay_toggled() -> void:
	if _world_renderer != null and _world_renderer.has_method("toggle_resource_overlay"):
		_world_renderer.toggle_resource_overlay()


## 스폰 포인트 시각화 갱신 — 마커 + 인원 수 표시
func _refresh_spawn_visual() -> void:
	if _spawn_overlay == null:
		return
	## 기존 마커 전부 제거
	for child in _spawn_overlay.get_children():
		child.queue_free()

	var spawn_points: Array = _editor_ctrl.get_spawn_points()
	var tile_px: int = GameConfig.TILE_SIZE

	for sp in spawn_points:
		var pos: Vector2i = sp.position
		var count: int = sp.get("count", GameConfig.MAP_EDITOR_SPAWN_DEFAULT)

		## 노란색 ✕ 마커
		var marker: Label = Label.new()
		marker.text = "X"
		marker.add_theme_color_override("font_color", Color.YELLOW)
		marker.add_theme_font_size_override("font_size", 14)
		marker.position = Vector2(
			pos.x * tile_px - 5,
			pos.y * tile_px - 7
		)
		_spawn_overlay.add_child(marker)

		## 인원 수 레이블
		var count_lbl: Label = Label.new()
		count_lbl.text = str(count)
		count_lbl.add_theme_color_override("font_color", Color.WHITE)
		count_lbl.add_theme_font_size_override("font_size", 10)
		count_lbl.position = Vector2(
			pos.x * tile_px - 3,
			pos.y * tile_px + 5
		)
		_spawn_overlay.add_child(count_lbl)

	## 총 인원 표시 갱신
	var total_count: int = 0
	for sp in spawn_points:
		total_count += sp.get("count", GameConfig.MAP_EDITOR_SPAWN_DEFAULT)
	if _brush_palette != null:
		_brush_palette.update_spawn_total(total_count)


## 맵 중앙에서 가장 가까운 walkable 타일 탐색
func _find_center_walkable() -> Vector2i:
	if _world_data == null:
		return Vector2i(-1, -1)
	var cx: int = GameConfig.WORLD_SIZE.x / 2
	var cy: int = GameConfig.WORLD_SIZE.y / 2
	for r in range(0, 50):
		for dy in range(-r, r + 1):
			for dx in range(-r, r + 1):
				var x: int = cx + dx
				var y: int = cy + dy
				if _world_data.is_valid(x, y):
					var biome: int = _world_data.get_biome(x, y)
					if biome >= 2: ## BEACH=2 이상은 walkable
						return Vector2i(x, y)
	return Vector2i(-1, -1)
