extends SceneTree

const Topbar = preload("res://scripts/ui/panels/hud_topbar.gd")
const News = preload("res://scripts/ui/panels/hud_status_panel.gd")
const DeathViz = preload("res://scripts/ui/death_viz_renderer.gd")
const FONT = preload("res://assets/fonts/galmuri/Galmuri11.ttf")
const SIZE := Vector2i(1920, 1080)
const SCENARIO := "SYNTHETIC CALIBRATION / NOT NATURAL SIMULATION EVIDENCE"
const STEM := "calibration_deaths_p7"
var _main: Node2D
var _locale: Node
var _out: String = "res://docs/art/production7/calibration"
var _validate_fixture: bool = false

class SnapshotWorld extends Node:
	signal world_reset
	func get_tallies() -> Dictionary:
		return {"deaths_starvation": 1, "deaths_dehydration": 1, "deaths_combat": 1, "deaths_old_age": 1, "deaths_exposure": 1}
	func get_agent_snapshot() -> Dictionary: return {}
	func get_settlement_snapshot() -> Dictionary: return {}
	func get_society_snapshot() -> Dictionary: return {}
	func get_construction_snapshot() -> Dictionary: return {}
	func get_building_snapshot() -> Dictionary: return {}
	func get_resource_snapshot() -> Dictionary: return {}
	func get_time_snapshot() -> Dictionary: return {"tick": 100, "time_of_day": 12.0}
	func get_recent_deaths() -> Dictionary:
		return {"xs": PackedInt32Array([0, 12, 24, 36, 48]), "ys": PackedInt32Array([20, 20, 20, 20, 20]), "reasons": PackedInt32Array([0, 1, 2, 3, 4]), "ticks": PackedInt64Array([100, 100, 100, 100, 100]), "current_tick": 100}

func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	var arguments: PackedStringArray = OS.get_cmdline_user_args()
	if arguments.size() == 2 and arguments[0] == "--out":
		_out = arguments[1]
	elif arguments == PackedStringArray(["--validate-fixture"]):
		_validate_fixture = true
	elif not arguments.is_empty():
		push_error("Usage: -- --out /absolute/output/directory")
		quit(2)
		return
	if DisplayServer.get_name() == "headless" and not _validate_fixture:
		push_error("Death calibration requires a real window")
		quit(2)
		return
	var destination: String = ProjectSettings.globalize_path(_out)
	if DirAccess.make_dir_recursive_absolute(destination) != OK:
		quit(2)
		return
	root.content_scale_size = SIZE
	root.content_scale_mode = Window.CONTENT_SCALE_MODE_VIEWPORT
	root.content_scale_aspect = Window.CONTENT_SCALE_ASPECT_IGNORE
	root.content_scale_factor = 1.0
	root.size = SIZE
	root.title = SCENARIO
	_locale = root.get_node("Locale")
	_locale.call("set_language", "ko")
	_main = Node2D.new()
	_main.name = "Main"
	root.add_child(_main)
	var background := ColorRect.new()
	background.size = Vector2(SIZE)
	background.color = Color("202B32")
	_main.add_child(background)
	var world := SnapshotWorld.new()
	world.name = "WorldSim"
	_main.add_child(world)
	var topbar: Control = Topbar.new()
	_main.add_child(topbar)
	topbar.set_process(false)
	var news: Control = News.new()
	_main.add_child(news)
	news.set_process(false)
	news.call("_ingest_snapshots")
	news.call("_refresh")
	var viz: Node2D = DeathViz.new()
	_main.add_child(viz)
	viz.set_process(false)
	viz.call("_process", 0.0)
	_label(SCENARIO, Rect2(64, 150, 1800, 42), 28)
	_label("실제 사망 렌더러 · 실제 HUD / 입력만 합성 fixture · 각 원인 1명, 합계 5명", Rect2(64, 208, 1780, 36), 22)
	_label("실제 마커 (원래 크기 1×)", Rect2(64, 292, 370, 32), 22)
	for index: int in 5:
		var x: float = 456.0 + float(index) * 192.0
		var color: Color = viz.call("_reason_color", index)
		_label("%d" % index, Rect2(x - 20, 390, 40, 32), 20)
		_label("#" + color.to_html(false).to_upper(), Rect2(x - 68, 430, 160, 32), 20)
	var log: Array = news.call("get_feed_log")
	_label("진단용 전체 feed log: 5줄 (실제 UI는 좌하단의 4줄)", Rect2(760, 556, 1090, 40), 24)
	for index: int in log.size():
		var record: Dictionary = log[index]
		var translated: String = str(_locale.call("ltr", record["key"]))
		_label("%d  %s" % [index, translated], Rect2(760, 618 + index * 58, 1080, 42), 24)
	_label("실제 HUD 알림: 최대 4줄 · 마지막 줄에 추위 사망", Rect2(24, 786, 720, 34), 20)
	_label("합성 화면은 자연 사망 발생률·A26 자연 캡처 합격의 증거가 아닙니다.", Rect2(760, 990, 1130, 40), 22)
	if _validate_fixture:
		var fixture_valid: bool = log.size() == 5 and news.call("get_visible_feed").size() == 4 and topbar.call("get_death_total") == 5 and viz.get("_xs").size() == 5
		print("DEATH CALIBRATION FIXTURE %s; drawing and capture not tested" % ["PASS" if fixture_valid else "FAIL"])
		_main.free()
		quit(0 if fixture_valid else 1)
		return
	for frame: int in 3:
		await process_frame
	await RenderingServer.frame_post_draw
	var deaths: Array = viz.call("capture_metadata")
	var visible_feed: Array = news.call("get_visible_feed")
	var total: int = int(topbar.call("get_death_total"))
	var valid: bool = deaths.size() == 5 and visible_feed.size() == 4 and log.size() == 5 and total == 5
	for index: int in mini(5, deaths.size()):
		valid = valid and int(deaths[index]["reason"]) == index
	var screenshot: Image = root.get_texture().get_image()
	valid = valid and screenshot.get_size() == SIZE
	var png_path: String = destination.path_join(STEM + ".png")
	if screenshot.save_png(png_path) != OK:
		push_error("Failed to save death calibration PNG")
		quit(1)
		return
	var metadata: Dictionary = {
		"schema_version": 1, "scenario": SCENARIO, "natural": false,
		"simulation_instantiated": false, "god_intervention": false,
		"input_source": "SnapshotWorld synthetic five-reason fixture",
		"production_scripts": ["res://scripts/ui/panels/hud_topbar.gd", "res://scripts/ui/panels/hud_status_panel.gd", "res://scripts/ui/death_viz_renderer.gd"],
		"source_sha": OS.get_environment("WORLDSIM_GIT_SHA"),
		"locale": "ko", "viewport": [SIZE.x, SIZE.y], "file": STEM + ".png",
		"png_sha256": FileAccess.get_sha256(png_path),
		"death_tallies": SimReads.read(world, "get_tallies"), "hud_death_total": total,
		"drawn_deaths": deaths, "visible_feed": visible_feed, "feed_log": log,
		"validation_pass": valid,
	}
	var output: FileAccess = FileAccess.open(destination.path_join(STEM + ".json"), FileAccess.WRITE)
	if output == null:
		quit(1)
		return
	output.store_string(JSON.stringify(metadata, "\t") + "\n")
	output.close()
	print("DEATH CALIBRATION %s markers=%d visible=%d log=%d total=%d synthetic=true" % ["PASS" if valid else "FAIL", deaths.size(), visible_feed.size(), log.size(), total])
	_main.free()
	quit(0 if valid else 1)

func _label(text: String, bounds: Rect2, font_size: int) -> void:
	var label := Label.new()
	label.text = text
	label.position = bounds.position
	label.size = bounds.size
	label.add_theme_font_override("font", FONT)
	label.add_theme_font_size_override("font_size", font_size)
	label.add_theme_color_override("font_color", Color("EEEAD2"))
	_main.add_child(label)
