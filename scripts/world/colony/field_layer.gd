# Authoritative cultivated tiles; drawing never advances jobs or crops.
# Bridge reads via FieldFacts/SimReads: get_field_tiles, get_crop_defs, get_time_snapshot; no mutations.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const Facts = preload("res://scripts/ui/field_facts.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
var _world: Node
var _rows: Array[Dictionary] = []
var _elapsed: float = 0.0
var _signature: int = -1
var _season: int = 0
var _categories: Dictionary = {}

func _ready() -> void:
	_world = get_node("/root/Main/WorldSim")
	_world.connect("world_reset", _reset)
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST

func _reset() -> void:
	_rows.clear()
	_signature = -1
	queue_redraw()

func _process(delta: float) -> void:
	_elapsed += delta
	if _elapsed < float(ArtData.param("plants.poll_seconds", 0.5)):
		return
	_elapsed = 0.0
	var camera: Camera2D = get_viewport().get_camera_2d()
	if camera == null:
		return
	var half: Vector2 = get_viewport_rect().size / camera.zoom / 2.0
	var centre: Vector2 = camera.get_screen_center_position()
	var first: Vector2i = View.world_to_tile(centre - half) - Vector2i.ONE
	var last: Vector2i = View.world_to_tile(centre + half) + Vector2i.ONE
	var window := Rect2i(first, last - first + Vector2i.ONE)
	var rows: Array[Dictionary] = []
	for row: Dictionary in Facts.rows(_world):
		if window.has_point(Vector2i(int(row["x"]), int(row["y"]))):
			rows.append(row)
	_season = int(SimReads.read(_world, "get_time_snapshot").get("season", 0))
	var signature: int = hash([rows, _season, ArtData.params_version()])
	if signature != _signature:
		_rows = rows
		_categories.clear()
		for definition: Dictionary in Facts.definitions(_world):
			_categories[int(definition["code"])] = str(definition.get("category", ""))
		_signature = signature
		queue_redraw()

func _draw() -> void:
	var style: Dictionary = ArtData.params().get("plants", {})
	var colors: Array = style.get("phase_colors", ["777461", "718253", "9c9b56", "77704f"])
	var earth := Color(str(colors[0]))
	var green := Color(str(colors[1]))
	var ripe := Color(str(style.get("ripe_accent", "c6a26a")))
	if _season == 3:
		var season_style: Dictionary = ArtData.params().get("season", {})
		var saturation: float = float(season_style.get("winter_saturation", 0.35))
		green = Color.from_hsv(green.h, green.s * saturation, green.v)
		ripe = Color.from_hsv(ripe.h, ripe.s * saturation, ripe.v)
	var outline := Color(str(style.get("outline", "141210")))
	for row: Dictionary in _rows:
		var origin: Vector2 = View.tile_to_world(Vector2i(int(row["x"]), int(row["y"])))
		var phase: int = int(row["phase"])
		var alpha: float = 0.8 if bool(row["active"]) else 0.4
		draw_rect(Rect2(origin + Vector2(4, 4), Vector2(56, 56)), Color(earth, alpha * 0.35))
		if phase == 0:
			draw_rect(Rect2(origin + Vector2(5, 5), Vector2(54, 54)), Color(earth, alpha), false, 2)
			continue
		for y: int in [16, 32, 48]:
			draw_line(origin + Vector2(8, y), origin + Vector2(56, y), Color(outline, alpha), 2)
			for x: int in [16, 32, 48]:
				var point: Vector2 = origin + Vector2(x, y)
				match phase:
					2:
						draw_circle(point, 2, Color(ripe, alpha))
					3, 4:
						draw_line(point, point - Vector2(0, 10 if phase == 4 else 6), Color(green, alpha), 2)
						draw_line(point - Vector2(0, 4), point - Vector2(4, 7), Color(green, alpha), 2)
						if phase == 4:
							if str(_categories.get(int(row["crop"]), "")) == "fibre":
								draw_line(point - Vector2(3, 11), point - Vector2(-3, 11), Color(ripe, alpha), 2)
							else:
								draw_circle(point - Vector2(0, 11), 3, Color(ripe, alpha))
					5:
						draw_line(point, point - Vector2(0, 3), Color(earth, alpha), 2)
					6:
						draw_circle(point, 1, Color(earth, alpha))
