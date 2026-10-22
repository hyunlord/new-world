# Full-template character lab; only structurally accepted route sets are shown.
# Bridge methods: none. Standalone visual/measurement harness, not simulation.
extends Node2D

const CELL := Vector2(104, 112)
const COLUMNS := 10
const PAGE_HEIGHT := 896
const FRAMES := ["idle", "walk_a", "walk_b", "work_a", "work_b"]
var options: Dictionary = {}
var routes: Dictionary = {}
var route: String = "A"
var layers: Dictionary = {}
var combos: Array = []
var visible_indices: Array[int] = []
var grid_columns: int = COLUMNS
var page: int = 0
var page_count: int = 1
var total_combinations: int = 0
var requested_size := Vector2i.ZERO
var source_hash_at_start: String = ""
var visibility_samples: Array[Dictionary] = []
var hidden_process_frames: int = 0
var textures: Dictionary = {}
var hashes: Dictionary = {}
var ground: String = "meadow"
var zoom: int = 1
var mode: String = "grid"
var output: String = ""
var elapsed: float = 0.0
var start_us: int = 0
var previous_us: int = 0
var rendered_frames: int = 0
var frame_times: Array[int] = []
var running: bool = false
var capture_index: int = 0
var capture_busy: bool = false
var valid: bool = true
var dataset_status: String = "NOT_COMPLETE"
var missing_inventory: Array = []

func _ready() -> void:
    source_hash_at_start = FileAccess.get_sha256("res://CharacterLab.gd")
    var args := OS.get_cmdline_user_args()
    for index: int in range(0, args.size() - 1, 2):
        options[args[index].trim_prefix("--")] = args[index + 1]
    route = str(options.get("route", "A")).to_upper()
    mode = str(options.get("mode", "grid"))
    zoom = int(options.get("zoom", "1"))
    if zoom not in [1, 2]:
        _fail("zoom must be 1 or 2")
        return
    ground = str(options.get("ground", "meadow"))
    output = str(options.get("out", "/Users/rexxa/work/arttrial3-data/lab"))
    DirAccess.make_dir_recursive_absolute(output)
    for name: String in ["A", "B"]:
        var root: String = str(options.get("root-" + name.to_lower(), ProjectSettings.globalize_path("res://../character/route_" + name.to_lower())))
        if FileAccess.file_exists(root.path_join("manifest.json")):
            var manifest: Dictionary = _read_json(root.path_join("manifest.json"))
            var checks: Dictionary = _read_json(root.path_join("CHECKS.json"))
            var bound: bool = _checks_allowed(checks) and not checks.get("hashes", {}).is_empty()
            for source: String in checks.get("hashes", {}):
                if not FileAccess.file_exists(source) or FileAccess.get_sha256(source) != str(checks["hashes"][source]):
                    bound = false
            if bound:
                routes[name] = {"root": root, "manifest": manifest, "checks": checks}
    if not routes.has(route):
        _fail("no structurally passing route " + route + "; no placeholder substituted")
        return
    _select_route(route)
    if not valid:
        get_tree().quit(1)
        return
    if DisplayServer.get_name() != "headless":
        get_window().always_on_top = true
        DisplayServer.window_move_to_foreground()
    texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
    DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
    RenderingServer.frame_post_draw.connect(_frame_drawn)
    _begin.call_deferred()

func _checks_allowed(checks: Dictionary) -> bool:
    if checks.get("passed", false):
        return true
    if str(options.get("allow-incomplete", "false")) != "true" or not checks.get("subset_passed", false):
        return false
    if checks.get("status", "") != "NOT_COMPLETE" or int(checks.get("layer_count", 0)) == 0 or checks.get("layer_count") != checks.get("layer_pass_count"):
        return false
    for result: Dictionary in checks.get("results", []):
        if not result.get("passed", false) and result.get("id", "") != "minimum_set":
            return false
    for combination: Dictionary in checks.get("combinations", []):
        if not combination.get("passed", false):
            return false
    return not checks.get("combinations", []).is_empty()

func _read_json(path: String) -> Dictionary:
    if not FileAccess.file_exists(path):
        return {}
    hashes[path] = FileAccess.get_sha256(path)
    var data: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
    return data if data is Dictionary else {}

func _fail(message: String) -> void:
    valid = false
    push_error("CHARACTER_LAB_REJECTED " + message)
    get_tree().quit(1)

func _select_route(name: String) -> void:
    route = name
    layers.clear()
    var data: Dictionary = routes[name]
    var root: String = data["root"]
    dataset_status = "PASS" if data["checks"].get("passed", false) else "NOT_COMPLETE"
    missing_inventory.clear()
    for result: Dictionary in data["checks"].get("results", []):
        if result.get("id", "") == "minimum_set" and not result.get("passed", false):
            missing_inventory.append_array(result.get("findings", []))
    for relative: String in data["manifest"]["layers"]:
        var path := root.path_join(relative)
        var layer := _read_json(path)
        var image_path := path.get_base_dir().path_join(str(layer.get("image", path.get_file().get_basename() + ".png")))
        var mask_path := path.get_base_dir().path_join(str(layer.get("mask", "")))
        layer["texture"] = _load_texture(image_path, mask_path)
        layers[str(layer["id"])] = layer
    combos = data["checks"].get("combinations", []).duplicate(true)
    for combination: Dictionary in combos:
        if not combination.get("passed", false):
            _fail("rejected combination in accepted set")
            return
        for id: String in combination["ids"]:
            if not layers.has(id):
                _fail("unknown layer " + id)
                return
    if combos.is_empty():
        _fail("empty checked combination list")
        return
    total_combinations = combos.size()
    if options.has("combo"):
        var selected := int(options["combo"])
        if selected < 0 or selected >= combos.size():
            _fail("combo index out of range")
            return
        combos = [combos[selected]]
    _load_texture(_environment("ground/" + ground + ".png"))
    if mode == "crowd":
        for asset: String in ["buildings/hut.png", "misc/hearth_0.png", "items/log_pile_small.png", "plants/oak_large.png"]:
            _load_texture(_environment(asset))
    _resize()

func _environment(relative: String) -> String:
    if relative.begins_with("ground/"):
        return str(options.get("ground-root", ProjectSettings.globalize_path("res://../style05/ground_v2"))).path_join(relative.trim_prefix("ground/"))
    return str(options.get("env-root", ProjectSettings.globalize_path("res://../style05/env"))).path_join(relative)

func _load_texture(path: String, mask_path: String = "") -> Texture2D:
    var key := path + "|" + mask_path
    if textures.has(key):
        return textures[key]
    if not FileAccess.file_exists(path) or (not mask_path.is_empty() and not FileAccess.file_exists(mask_path)):
        _fail("missing image or mask " + path)
        return null
    var image := Image.load_from_file(path)
    if image == null or image.is_empty():
        _fail("invalid image " + path)
        return null
    hashes[path] = FileAccess.get_sha256(path)
    if not mask_path.is_empty():
        var mask := Image.load_from_file(mask_path)
        if mask == null or mask.get_size() != image.get_size():
            _fail("invalid mask " + mask_path)
            return null
        hashes[mask_path] = FileAccess.get_sha256(mask_path)
        # Region tints preserve source luminance and ink. Identity defaults keep art neutral.
        var tints := {1: Color(str(options.get("skin", "#ffffff"))), 2: Color(str(options.get("hair", "#ffffff"))), 3: Color(str(options.get("cloth", "#ffffff")))}
        for y: int in image.get_height():
            for x: int in image.get_width():
                var region := roundi(mask.get_pixel(x, y).r * 255.0)
                if tints.has(region):
                    image.set_pixel(x, y, image.get_pixel(x, y) * tints[region])
    var texture := ImageTexture.create_from_image(image)
    textures[key] = texture
    return texture

func _resize() -> void:
    visible_indices.clear()
    grid_columns = maxi(1, floori(float(COLUMNS) / zoom))
    var rows := maxi(1, floori(float(PAGE_HEIGHT) / (CELL.y * zoom)))
    var capacity := grid_columns * rows
    page_count = maxi(1, ceili(float(combos.size()) / capacity))
    page = clampi(int(options.get("page", "0")), 0, page_count - 1)
    var first := page * capacity
    var last := mini(first + capacity, combos.size())
    for index: int in range(first, last):
        visible_indices.append(index)
    var size := Vector2(grid_columns * CELL.x, ceilf(float(visible_indices.size()) / grid_columns) * CELL.y)
    if mode == "crowd":
        size = Vector2(1312, 1088)
    elif combos.size() == 1:
        size = CELL
    requested_size = Vector2i(size * zoom)
    get_window().size = requested_size
    queue_redraw()

func _begin() -> void:
    await get_tree().process_frame
    if DisplayServer.get_name() != "headless":
        await RenderingServer.frame_post_draw
    if mode != "crowd" and get_window().size != requested_size:
        _fail("window size clamped; requested %s actual %s" % [requested_size, get_window().size])
        return
    start_us = Time.get_ticks_usec()
    previous_us = start_us
    running = true
    print("CHARACTER_LAB_READY route=%s combinations=%d mode=%s" % [route, combos.size(), mode])

func _frame_drawn() -> void:
    if running:
        rendered_frames += 1

func _unhandled_key_input(event: InputEvent) -> void:
    if event is InputEventKey and event.pressed:
        if event.keycode == KEY_TAB and mode == "grid":
            var next := "B" if route == "A" else "A"
            if routes.has(next):
                _select_route(next)
        elif event.keycode in [KEY_LEFT, KEY_RIGHT] and mode == "grid":
            options["page"] = str(clampi(page + (1 if event.keycode == KEY_RIGHT else -1), 0, page_count - 1))
            _resize()
        elif event.keycode == KEY_G:
            ground = "forest_floor" if ground == "meadow" else "meadow"
            _load_texture(_environment("ground/" + ground + ".png"))
        elif event.keycode in [KEY_1, KEY_2] and mode == "grid":
            zoom = 1 if event.keycode == KEY_1 else 2
            _resize()

func _body_for(body: Dictionary, frame: String) -> Dictionary:
    for candidate: Dictionary in layers.values():
        if candidate["kind"] == "body" and candidate["body_type"] == body["body_type"] and candidate["direction"] == body["direction"] and candidate["frame"] == frame:
            return candidate
    _fail("missing required animation frame " + frame)
    return body

func _person(ids: Array, point: Vector2, phase: int) -> void:
    var body: Dictionary = layers[str(ids[0])]
    var activity := str(options.get("activity", "loop"))
    var frame := str(body["frame"])
    if activity == "sleep":
        for candidate: Dictionary in layers.values():
            if candidate["kind"] == "body" and candidate["body_type"] == body["body_type"] and candidate["frame"] == "sleep":
                body = candidate
                frame = "sleep"
                ids = [candidate["id"]]
                break
        if frame != "sleep":
            _fail("missing sleep body")
            return
    if mode == "crowd":
        activity = "walk"
    if frame != "sleep":
        if activity in ["walk", "carry"]:
            frame = "walk_a" if phase % 2 == 0 else "walk_b"
        elif activity == "work":
            frame = "work_a" if phase % 2 == 0 else "work_b"
        elif activity == "idle":
            frame = "idle"
            point.y -= phase % 2
        elif activity == "loop":
            frame = FRAMES[phase % FRAMES.size()]
        body = _body_for(body, frame)
    var shadow_size := Vector2(22, 6) if body["body_type"] == "child" else Vector2(28, 8)
    if body["frame"] != "sleep":
        draw_set_transform((point + Vector2(32, 90)) * zoom, 0, shadow_size * zoom / 2)
        draw_circle(Vector2.ZERO, 1, Color(0, 0, 0, 110.0 / 255.0))
    draw_set_transform(Vector2.ZERO)
    for id: String in ids:
        var layer: Dictionary = body if layers[id]["kind"] == "body" else layers[id]
        var offset := Vector2.ZERO
        if layer["kind"] in ["item", "carry"]:
            var child: bool = body["body_type"] == "child"
            var elder: bool = body["body_type"] == "elder"
            var target := Vector2(32, 60 if child else (55 if elder else 52))
            var source: Array = layer.get("shoulder", [32, 52])
            if layer["kind"] == "item":
                source = layer["grip"]
                if body["direction"] == "E":
                    target = Vector2(41 if child else 44, 68 if child else (66 if elder else 64))
                else:
                    target = Vector2((47 if child else 50) if body["direction"] == "S" else (17 if child else 14), 70 if child else (68 if elder else 66))
            offset = target - Vector2(source[0], source[1])
        var texture: Texture2D = layer["texture"]
        if texture != null:
            draw_texture_rect(texture, Rect2((point + offset) * zoom, texture.get_size() / float(layer["scale"]) * zoom), false)

func _draw() -> void:
    if not valid or combos.is_empty():
        return
    var bg: Texture2D = textures.get(_environment("ground/" + ground + ".png") + "|")
    if bg != null:
        for y: int in range(0, get_window().size.y, 512 * zoom):
            for x: int in range(0, get_window().size.x, 512 * zoom):
                draw_texture_rect(bg, Rect2(x, y, 512 * zoom, 512 * zoom), false)
    var phase := capture_index if mode == "capture" else floori(elapsed * 8)
    if mode == "crowd":
        var assets := ["buildings/hut.png", "misc/hearth_0.png", "items/log_pile_small.png", "plants/oak_large.png"]
        for index: int in assets.size():
            var texture: Texture2D = textures[_environment(assets[index]) + "|"]
            draw_texture_rect(texture, Rect2(64 + index * 320, 0, 128, 128), false)
        var upright: Array = combos.filter(func(c: Dictionary) -> bool: return layers[str(c["ids"][0])]["frame"] != "sleep")
        if upright.is_empty():
            return
        for index: int in 200:
            var point := Vector2(16 + (index % 20) * 64, 128 + floori(float(index) / 20) * 96)
            point.x += sin(elapsed * 2 + index) * 3
            _person(upright[index % upright.size()]["ids"], point, phase + index)
    else:
        for slot: int in visible_indices.size():
            _person(combos[visible_indices[slot]]["ids"], Vector2(slot % grid_columns, floori(float(slot) / grid_columns)) * CELL, phase)

func _process(delta: float) -> void:
    if not running:
        return
    elapsed += delta
    var now := Time.get_ticks_usec()
    var wall_elapsed := float(now - start_us) / 1000000.0
    var drawable := DisplayServer.window_can_draw()
    if not drawable:
        hidden_process_frames += 1
    if visibility_samples.size() <= floori(wall_elapsed):
        visibility_samples.append({"wall_seconds": wall_elapsed, "can_draw": drawable, "focused": DisplayServer.window_is_focused(), "rendered_frames": rendered_frames, "process_frames": frame_times.size()})
    frame_times.append(now - previous_us)
    previous_us = now
    queue_redraw()
    if mode == "capture" and not capture_busy:
        _capture_frame.call_deferred()
    elif mode != "capture" and float(now - start_us) / 1000000.0 >= float(options.get("seconds", "60")):
        running = false
        _finish.call_deferred()

func _capture_frame() -> void:
    if capture_busy or not running:
        return
    capture_busy = true
    if DisplayServer.get_name() == "headless":
        _fail("capture requires a real window")
        return
    await RenderingServer.frame_post_draw
    var path := output.path_join("frame_%02d.png" % capture_index)
    if get_viewport().get_texture().get_image().save_png(path) != OK:
        _fail("capture write failed")
        return
    capture_index += 1
    capture_busy = false
    if capture_index == 8:
        running = false
        _finish.call_deferred()

func _finish() -> void:
    var seconds := float(Time.get_ticks_usec() - start_us) / 1000000.0
    var headless := DisplayServer.get_name() == "headless"
    if not headless:
        await RenderingServer.frame_post_draw
        if get_viewport().get_texture().get_image().save_png(output.path_join("lab_%s_%dx.png" % [route.to_lower(), zoom])) != OK:
            _fail("board write failed")
            return
    var report := {"dataset_status": dataset_status, "diagnostic_partial_inventory": dataset_status == "NOT_COMPLETE", "missing_inventory": missing_inventory, "route": route, "mode": mode, "zoom": zoom, "ground": ground, "combinations": total_combinations, "page": page, "page_count": page_count, "global_combination_indices": [int(options["combo"])] if options.has("combo") else visible_indices, "draw_count": 200 if mode == "crowd" else visible_indices.size(), "population": 200 if mode == "crowd" else visible_indices.size(), "requested_viewport": [requested_size.x, requested_size.y], "wall_seconds": seconds, "rendered_frames": rendered_frames, "rendered_fps": rendered_frames / seconds, "headless": headless, "p1_eligible": dataset_status == "PASS" and not headless and mode == "crowd" and zoom == 1 and seconds >= 60, "frame_times_us": frame_times, "visibility_samples": visibility_samples, "hidden_process_frames": hidden_process_frames, "always_on_top": get_window().always_on_top, "asset_hashes": hashes, "script_sha256": source_hash_at_start, "source_unchanged_during_run": source_hash_at_start == FileAccess.get_sha256("res://CharacterLab.gd"), "viewport": [get_window().size.x, get_window().size.y], "renderer": RenderingServer.get_current_rendering_driver_name()}
    var file := FileAccess.open(output.path_join("runtime.json"), FileAccess.WRITE)
    if file == null:
        _fail("runtime evidence write failed")
        return
    file.store_string(JSON.stringify(report, "  ") + "\n")
    print("CHARACTER_LAB_DONE " + JSON.stringify({"route": route, "dataset_status": dataset_status, "headless": headless, "rendered_fps": report["rendered_fps"]}))
    get_tree().quit(0)
