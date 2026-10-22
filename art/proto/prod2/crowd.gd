# Native-size route I accepted-only crowd; actual frame_post_draw timing.
# Bridge methods: none. External PNGs are runtime lab data, never game resources.
extends Node2D

const COUNT := 200
const VIEW := Vector2i(1600, 960)
const STATES := ["idle", "walk_a", "work_a", "carry", "sleep"]
var output := "/Users/rexxa/work/artprod2-data/godot-crowd"
var route := ""
var duration_us := 60_000_000
var hashes: Dictionary = {}
var variants: Array[Dictionary] = []
var people: Array[Sprite2D] = []
var identities: Array[Dictionary] = []
var intervals: Array[int] = []
var visibility: Array[Dictionary] = []
var start_us := 0
var previous_us := 0
var last_sample_us := 0
var rendered_frames := 0
var hidden_frames := 0
var running := false
var finished := false
var last_phase := -1
var engine_start_frames := 0

func _ready() -> void:
    route = ProjectSettings.globalize_path("res://../../character/route_i").simplify_path()
    var args := OS.get_cmdline_user_args()
    for index: int in range(0, args.size() - 1, 2):
        match args[index]:
            "--out": output = args[index + 1]
            "--route": route = args[index + 1]
            "--seconds": duration_us = int(float(args[index + 1]) * 1_000_000.0)
    if DisplayServer.get_name() == "headless":
        _fail("windowed renderer required")
        return
    DirAccess.make_dir_recursive_absolute(output)
    hashes[ProjectSettings.globalize_path("res://crowd.gd")] = FileAccess.get_sha256("res://crowd.gd")
    hashes[ProjectSettings.globalize_path("res://project.godot")] = FileAccess.get_sha256("res://project.godot")
    Engine.max_fps = 0
    get_window().size = VIEW
    get_window().title = "ART-PROD2 | route I accepted | 200 people at 1x"
    get_window().always_on_top = true
    DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
    DisplayServer.window_move_to_foreground()
    texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
    if not _load_inputs():
        return
    for index: int in COUNT:
        var sprite := Sprite2D.new()
        sprite.centered = false
        add_child(sprite)
        people.append(sprite)
        var variant: Dictionary = variants[index % variants.size()]
        identities.append({"id": index, "look": variant["look"], "age": variant["age"],
            "foot": [_foot(index).x, _foot(index).y], "scale": 1})
    _animate(0)
    RenderingServer.frame_post_draw.connect(_drawn)
    _begin.call_deferred()

func _load_inputs() -> bool:
    var directories := DirAccess.get_directories_at(route)
    directories.sort()
    for look: String in directories:
        var manifest_path := route.path_join(look).path_join("MANIFEST.json")
        if not FileAccess.file_exists(manifest_path):
            continue
        var manifest: Variant = JSON.parse_string(FileAccess.get_file_as_string(manifest_path))
        if not manifest is Dictionary or manifest.get("frames", []).size() != 10:
            _fail("invalid accepted manifest: " + look)
            return false
        hashes[manifest_path] = FileAccess.get_sha256(manifest_path)
        for row: Dictionary in manifest["frames"]:
            if not row.get("passed", false) or not row.get("judge", {}).get("numeric_pass", false):
                _fail("unaccepted structural/numeric frame: " + look)
                return false
        var textures: Array[ImageTexture] = []
        for facing: String in ["S", "E"]:
            for state: String in STATES:
                var relative := facing + "/" + state + ".png"
                var path := route.path_join(look).path_join(relative)
                var digest := FileAccess.get_sha256(path)
                if digest.is_empty() or digest != str(manifest.get("files", {}).get(relative, "")):
                    _fail("input digest mismatch: " + path)
                    return false
                hashes[path] = digest
                var image := Image.load_from_file(path)
                var expected := Vector2i(112,80) if state == "sleep" else Vector2i(80,112)
                if image == null or image.get_size() != expected:
                    _fail("invalid input canvas: " + path)
                    return false
                textures.append(ImageTexture.create_from_image(image))
        variants.append({"look": look, "age": manifest["age"], "textures": textures})
    if variants.is_empty():
        _fail("no accepted route I looks")
        return false
    return true

func _foot(index: int) -> Vector2:
    return Vector2(56 + (index % 20) * 78, 112 + floori(float(index) / 20.0) * 88)

func _draw() -> void:
    draw_rect(Rect2(Vector2.ZERO, Vector2(VIEW)), Color("737b57"))
    for index: int in COUNT:
        draw_set_transform(_foot(index), 0.0, Vector2(14,4))
        draw_circle(Vector2.ZERO, 1.0, Color(0,0,0,0.24))
    draw_set_transform(Vector2.ZERO)

func _animate(phase: int) -> void:
    for index: int in people.size():
        # Every look cycles both facings and all five source frames at 8 fps.
        var frame := (phase + index) % 10
        people[index].texture = variants[index % variants.size()]["textures"][frame]
        people[index].position = _foot(index) - (Vector2(56,72) if frame % 5 == 4 else Vector2(40,104))

func _begin() -> void:
    await get_tree().create_timer(2.0).timeout
    await RenderingServer.frame_post_draw
    if get_viewport().get_texture().get_image().save_png(output.path_join("crowd_start.png")) != OK:
        _fail("cannot capture actual viewport")
        return
    start_us = Time.get_ticks_usec()
    previous_us = start_us
    engine_start_frames = Engine.get_frames_drawn()
    running = true
    print("PROD2_CROWD_BEGIN count=200 scale=1 duration_us=%d" % duration_us)

func _process(_delta: float) -> void:
    var now := Time.get_ticks_usec()
    var phase := floori(float(now) / 125_000.0)
    if phase != last_phase:
        _animate(phase)
        last_phase = phase
    if not running:
        return
    var valid := DisplayServer.window_can_draw() and get_window().visible and DisplayServer.window_get_mode() != DisplayServer.WINDOW_MODE_MINIMIZED and _visible_count() == COUNT and get_window().size == VIEW
    if not valid:
        hidden_frames += 1
    if now - last_sample_us >= 1_000_000:
        visibility.append({"elapsed_us": now-start_us, "valid": valid, "visible_people": _visible_count(), "window_size": [get_window().size.x,get_window().size.y]})
        last_sample_us = now

func _visible_count() -> int:
    var count := 0
    for sprite: Sprite2D in people:
        if sprite.is_visible_in_tree() and get_viewport_rect().encloses(Rect2(sprite.position,sprite.texture.get_size())):
            count += 1
    return count

func _drawn() -> void:
    if not running or finished:
        return
    var now := Time.get_ticks_usec()
    intervals.append(now - previous_us)
    previous_us = now
    rendered_frames += 1
    if now-start_us >= duration_us:
        running = false
        finished = true
        _finish.call_deferred(now-start_us, Engine.get_frames_drawn()-engine_start_frames)

func _finish(elapsed_us: int, engine_frames: int) -> void:
    var capture_error := get_viewport().get_texture().get_image().save_png(output.path_join("crowd_final.png"))
    var unchanged := true
    for path: String in hashes:
        unchanged = unchanged and FileAccess.get_sha256(path) == str(hashes[path])
    var fps := float(rendered_frames) * 1_000_000.0 / float(elapsed_us)
    var verified := elapsed_us >= 60_000_000 and hidden_frames == 0 and unchanged and capture_error == OK and absi(engine_frames-rendered_frames) <= 1 and DisplayServer.window_get_vsync_mode() == DisplayServer.VSYNC_DISABLED
    var result := {"measurement_verified": verified, "P1_pass": verified and fps >= 60.0,
        "fps": fps, "elapsed_us": elapsed_us, "rendered_frames": rendered_frames,
        "engine_frames_drawn_delta": engine_frames, "counter": "RenderingServer.frame_post_draw",
        "count": COUNT, "scale": 1, "viewport": [VIEW.x,VIEW.y], "vsync_mode": DisplayServer.window_get_vsync_mode(),
        "max_fps": Engine.max_fps, "hidden_process_frames": hidden_frames, "visible_final": _visible_count(),
        "godot": Engine.get_version_info(), "display_server": DisplayServer.get_name(),
        "rendering_method": RenderingServer.get_current_rendering_method(), "video_adapter": RenderingServer.get_video_adapter_name(),
        "input_hashes": hashes, "inputs_unchanged": unchanged, "people": identities,
        "frame_intervals_us": intervals, "visibility_samples": visibility, "animation_hz": 8,
        "source_frames": ["S/idle","S/walk_a","S/work_a","S/carry","S/sleep","E/idle","E/walk_a","E/work_a","E/carry","E/sleep"],
        "capture_error": capture_error, "limitations": "Isolated rendering lab, no simulation; rendered frames are not display scanout; shared host."}
    var file := FileAccess.open(output.path_join("measurement.json"),FileAccess.WRITE)
    file.store_string(JSON.stringify(result,"  ")+"\n")
    file.close()
    print("PROD2_CROWD_DONE verified=%s frames=%d elapsed_us=%d fps=%.3f" % [verified,rendered_frames,elapsed_us,fps])
    get_tree().quit(0 if (verified or duration_us < 60_000_000) and hidden_frames == 0 and unchanged else 1)

func _fail(message: String) -> void:
    push_error("PROD2_CROWD_NOT_VERIFIED " + message)
    get_tree().quit(1)
