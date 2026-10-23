# Purpose: measure real completed frames with 200 people, 400 sprites and ground.
# Bridge methods called: none; the isolated prototype never changes simulation state.
extends Node2D

const Inputs = preload("res://inputs.gd")
const Markers = preload("res://markers.gd")
const VIEW := Vector2i(1600, 900)
const PEOPLE := 200
const ENVIRONMENT := 400
var inputs := Inputs.new()
var markers := Markers.new()
var output := "/Users/rexxa/work/artenv1-data/crowd/1x"
var zoom := 1.0
var duration_us := 60_000_000
var people: Array[Sprite2D] = []
var objects: Array[Sprite2D] = []
var intervals: Array[int] = []
var samples: Array[Dictionary] = []
var start_us := 0
var previous_us := 0
var sampled_us := 0
var engine_start := 0
var frames := 0
var invalid_frames := 0
var phase := -1
var running := false
var finished := false

func _ready() -> void:
    var args := OS.get_cmdline_user_args()
    for index: int in range(0, args.size() - 1, 2):
        match args[index]:
            "--out": output = args[index + 1]
            "--zoom": zoom = float(args[index + 1])
            "--seconds": duration_us = int(float(args[index + 1]) * 1_000_000.0)
    if DisplayServer.get_name() == "headless" or zoom not in [1.0, 0.5]:
        _fail("real window and zoom1/.5 required")
        return
    if not inputs.load_all(ProjectSettings.globalize_path("res://../../..").simplify_path()):
        _fail(inputs.failure)
        return
    for filename: String in ["crowd.gd", "inputs.gd", "markers.gd", "project.godot", "crowd.tscn", "axe.png", "log.png"]:
        var path := ProjectSettings.globalize_path("res://" + filename)
        inputs.hashes[path] = FileAccess.get_sha256(path)
    DirAccess.make_dir_recursive_absolute(output)
    Engine.max_fps = 0
    get_window().size = VIEW
    get_window().title = "ENV1 200 people + 400 sprites | %sx" % zoom
    get_window().always_on_top = true
    DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
    DisplayServer.window_move_to_foreground()
    texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
    for index: int in ENVIRONMENT:
        var sprite := Sprite2D.new()
        sprite.texture = inputs.environment[index % inputs.environment.size()]
        var half := sprite.texture.get_size() / 2.0
        var position_world := Vector2(40 + (index % 25) * 63, 80 + floori(float(index) / 25) * 48)
        position_world = position_world.clamp(half, Vector2(VIEW) - half)
        sprite.position = _screen(position_world)
        sprite.scale = Vector2.ONE * zoom
        add_child(sprite)
        objects.append(sprite)
    for index: int in PEOPLE:
        var sprite := Sprite2D.new()
        sprite.centered = false
        sprite.visible = zoom == 1.0
        add_child(sprite)
        people.append(sprite)
        markers.positions.append(_foot(index))
        markers.colours.append(inputs.people[index % inputs.people.size()]["colour"])
    markers.visible = zoom <= .5
    add_child(markers)
    _animate(0)
    RenderingServer.frame_post_draw.connect(_drawn)
    _begin.call_deferred()

func _screen(world: Vector2) -> Vector2:
    return (world - Vector2(VIEW) / 2.0) * zoom + Vector2(VIEW) / 2.0

func _foot(index: int) -> Vector2:
    return _screen(Vector2(56 + (index % 20) * 78, 112 + floori(float(index) / 20) * 76))

func _draw() -> void:
    if inputs.ground == null:
        return
    var tile := int(64 * zoom)
    for y: int in range(0, VIEW.y, tile):
        for x: int in range(0, VIEW.x, tile):
            draw_texture_rect(inputs.ground, Rect2(x, y, tile, tile), false)

func _animate(current: int) -> void:
    for index: int in PEOPLE:
        var frame := (current + index) % 10
        people[index].texture = inputs.people[index % inputs.people.size()]["textures"][frame]
        people[index].position = _foot(index) - (Vector2(56, 72) if frame % 5 == 4 else Vector2(40, 104))
    markers.phase = current
    markers.queue_redraw()
    queue_redraw()

func _process(_delta: float) -> void:
    var now := Time.get_ticks_usec()
    var current := floori(float(now) / 125_000.0)
    if current != phase:
        _animate(current)
        phase = current
    if not running:
        return
    var visible_people := _visible_people()
    var visible_environment := _visible_environment()
    var valid := DisplayServer.window_can_draw() and get_window().visible and DisplayServer.window_get_mode() != DisplayServer.WINDOW_MODE_MINIMIZED and get_window().size == VIEW and visible_people == PEOPLE and visible_environment == ENVIRONMENT
    if not valid:
        invalid_frames += 1
    if now - sampled_us >= 1_000_000:
        samples.append({"elapsed_us": now - start_us, "valid": valid, "people": visible_people, "environment": visible_environment})
        sampled_us = now

func _visible_people() -> int:
    var count := 0
    if zoom <= .5:
        for point: Vector2 in markers.positions:
            if markers.is_visible_in_tree() and get_viewport_rect().encloses(Rect2(point - Vector2(5, 12), Vector2(25, 22))):
                count += 1
        return count
    for sprite: Sprite2D in people:
        if sprite.is_visible_in_tree() and get_viewport_rect().encloses(Rect2(sprite.position, sprite.texture.get_size())):
            count += 1
    return count

func _visible_environment() -> int:
    var count := 0
    for sprite: Sprite2D in objects:
        var size := sprite.texture.get_size() * zoom
        if sprite.is_visible_in_tree() and get_viewport_rect().encloses(Rect2(sprite.position - size / 2, size)):
            count += 1
    return count

func _begin() -> void:
    await get_tree().create_timer(2.0).timeout
    await RenderingServer.frame_post_draw
    if get_viewport().get_texture().get_image().save_png(output.path_join("start.png")) != OK:
        _fail("viewport capture failed")
        return
    start_us = Time.get_ticks_usec()
    previous_us = start_us
    engine_start = Engine.get_frames_drawn()
    running = true
    print("ENV1_CROWD_BEGIN zoom=%s people=200 environment=400" % zoom)

func _drawn() -> void:
    if not running or finished:
        return
    var now := Time.get_ticks_usec()
    intervals.append(now - previous_us)
    previous_us = now
    frames += 1
    if now - start_us >= duration_us:
        running = false
        finished = true
        _finish.call_deferred(now - start_us, Engine.get_frames_drawn() - engine_start)

func _finish(elapsed: int, engine_frames: int) -> void:
    var captured := get_viewport().get_texture().get_image().save_png(output.path_join("final.png")) == OK
    var unchanged := inputs.unchanged()
    var fps := float(frames) * 1_000_000.0 / float(elapsed)
    var verified := elapsed >= 60_000_000 and invalid_frames == 0 and unchanged and captured and absi(engine_frames - frames) <= 1 and objects.size() == ENVIRONMENT and people.size() == PEOPLE
    var result := {"measurement_verified": verified, "passed": verified and fps >= 60, "fps": fps, "zoom": zoom,
        "elapsed_us": elapsed, "rendered_frames": frames, "engine_frames": engine_frames, "counter": "frame_post_draw",
        "people_count": people.size(), "environment_count": objects.size(), "environment_variants": inputs.environment_paths,
        "lod": zoom <= .5, "marker_diameter": 10 if zoom <= .5 else 0, "viewport": [VIEW.x, VIEW.y],
        "vsync": DisplayServer.window_get_vsync_mode(), "max_fps": Engine.max_fps, "invalid_frames": invalid_frames,
        "godot": Engine.get_version_info(), "display_server": DisplayServer.get_name(),
        "renderer": RenderingServer.get_current_rendering_method(), "adapter": RenderingServer.get_video_adapter_name(),
        "hashes": inputs.hashes, "inputs_unchanged": unchanged, "frame_intervals_us": intervals, "visibility": samples,
        "limitations": "Isolated art rendering load; shared host; completed renders are not physical scanout."}
    var file := FileAccess.open(output.path_join("measurement.json"), FileAccess.WRITE)
    file.store_string(JSON.stringify(result, "  ") + "\n")
    file.close()
    print("ENV1_CROWD_DONE verified=%s fps=%.3f elapsed_us=%d" % [verified, fps, elapsed])
    get_tree().quit(0 if verified and fps >= 60 else 1)

func _fail(message: String) -> void:
    push_error("ENV1_CROWD_NOT_VERIFIED " + message)
    get_tree().quit(1)
