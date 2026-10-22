# Diagnostic 200-sprite TRIAL3B crowd on a real visible window, never lab admission.
# Bridge methods: none. Loads actual rejected B composite PNGs; no simulation.
extends Node2D

const COUNT := 200
const COLUMNS := 20
const CELL := Vector2i(64, 96)
const VIEW := Vector2i(1280, 960)
const MEASURE_US := 60_000_000
var output: String = ""
var lab: String = ""
var render_root: String = ""
var input_hashes: Dictionary = {}
var variants: Array[Array] = []
var people: Array[Sprite2D] = []
var intervals_us: Array[int] = []
var visibility: Array[Dictionary] = []
var start_us: int = 0
var previous_us: int = 0
var last_visibility_us: int = 0
var rendered_frames: int = 0
var hidden_frames: int = 0
var running: bool = false
var finishing: bool = false
var source_hash: String = ""

func _ready() -> void:
    source_hash = FileAccess.get_sha256("res://Trial3BCrowd.gd")
    var args := OS.get_cmdline_user_args()
    var options: Dictionary = {}
    for index: int in range(0, args.size() - 1, 2):
        options[args[index].trim_prefix("--")] = args[index + 1]
    output = str(options.get("out", "/Users/rexxa/work/arttrial3-data/b/crowd-diagnostic"))
    lab = str(options.get("lab", "/Users/rexxa/work/arttrial3-data/b/lab"))
    render_root = str(options.get("render", "/Users/rexxa/work/arttrial3-data/b/render"))
    DirAccess.make_dir_recursive_absolute(output)
    if DisplayServer.get_name() == "headless":
        _fail("visible display required; headless FPS is forbidden")
        return
    get_window().size = VIEW
    get_window().title = "TRIAL3B DIAGNOSTIC_REJECTED | 200 at 1x | P1 ineligible"
    get_window().always_on_top = true
    DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
    DisplayServer.window_move_to_foreground()
    texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
    if not _load_inputs():
        return
    for index: int in COUNT:
        var sprite := Sprite2D.new()
        sprite.texture = variants[index % variants.size()][0]
        sprite.centered = false
        sprite.position = Vector2((index % COLUMNS) * CELL.x, floori(float(index) / COLUMNS) * CELL.y)
        add_child(sprite)
        people.append(sprite)
    RenderingServer.frame_post_draw.connect(_drawn)
    _begin.call_deferred()

func _json(path: String) -> Variant:
    if not FileAccess.file_exists(path):
        return null
    input_hashes[path] = FileAccess.get_sha256(path)
    return JSON.parse_string(FileAccess.get_file_as_string(path))

func _load_inputs() -> bool:
    var manifest: Variant = _json(render_root.path_join("manifest.json"))
    var provenance: Variant = _json(lab.path_join("provenance.json"))
    var answers: Variant = _json(lab.path_join("answer_key.private.json"))
    if not manifest is Dictionary or not provenance is Dictionary or not answers is Array:
        _fail("missing actual B manifest/composite provenance/answer key")
        return false
    if manifest.get("route", "") != "B" or provenance.get("status", "") != "DIAGNOSTIC_REJECTED":
        _fail("diagnostic expects explicitly rejected B inputs")
        return false
    for path: String in provenance.get("inputs", {}):
        if FileAccess.get_sha256(path) != str(provenance["inputs"][path]):
            _fail("stale composite source: " + path)
            return false
        input_hashes[path] = str(provenance["inputs"][path])
    for row: Dictionary in answers:
        if int(row.get("zoom", 0)) != 1 or row.get("activity", "") not in ["walk", "carry"]:
            continue
        var frames: Array[ImageTexture] = []
        for frame: int in 8:
            var image := Image.create(64, 96, false, Image.FORMAT_RGBA8)
            image.fill(Color.TRANSPARENT)
            var frame_name: String = str(row["frames"][frame])
            for original_id: String in row["layer_ids"]:
                var layer_id := original_id
                if original_id.begins_with("BODY_"):
                    layer_id = "BODY_adult_%s_%s" % [str(row["direction"]),frame_name]
                var path := render_root.path_join("layers/"+layer_id+".png")
                var source := Image.load_from_file(path)
                if source == null or source.get_size() != CELL or not input_hashes.has(path):
                    _fail("missing/wrong-size/unbound original B pass: " + path)
                    return false
                source.convert(Image.FORMAT_RGBA8)
                image.blend_rect(source, Rect2i(Vector2i.ZERO,CELL),Vector2i.ZERO)
            var frame_path := output.path_join("sample_%02d_frame_%02d.png" % [int(row["number"]),frame])
            if image.save_png(frame_path) != OK:
                _fail("cannot retain exact composited frame")
                return false
            input_hashes[frame_path] = FileAccess.get_sha256(frame_path)
            frames.append(ImageTexture.create_from_image(image))
        variants.append(frames)
    if variants.is_empty():
        _fail("no upright actual B composites")
        return false
    return true

func _draw() -> void:
    draw_rect(Rect2(Vector2.ZERO,Vector2(VIEW)),Color("677054"))
    for index: int in COUNT:
        var center := Vector2((index % COLUMNS)*CELL.x+32, floori(float(index)/COLUMNS)*CELL.y+88)
        draw_set_transform(center,0.0,Vector2(14,4))
        draw_circle(Vector2.ZERO,1.0,Color(0,0,0,110.0/255.0))
    draw_set_transform(Vector2.ZERO)

func _begin() -> void:
    await get_tree().create_timer(2.0).timeout
    start_us = Time.get_ticks_usec()
    previous_us = start_us
    running = true
    print("B_CROWD_BEGIN count=200 duration=60 visible=true diagnostic_rejected=true")

func _process(_delta: float) -> void:
    var now := Time.get_ticks_usec()
    for index: int in people.size():
        var frame := (floori(float(now) / 125_000.0) + index) % 8
        people[index].texture = variants[index % variants.size()][frame]
    if not running:
        return
    var visible_now := _visible_count()
    var can_draw := DisplayServer.window_can_draw() and get_window().visible
    var minimized := DisplayServer.window_get_mode() == DisplayServer.WINDOW_MODE_MINIMIZED
    if not can_draw or minimized or visible_now != COUNT:
        hidden_frames += 1
    if now - last_visibility_us >= 1_000_000:
        visibility.append({"elapsed_us": now-start_us, "can_draw": can_draw,
            "minimized": minimized, "visible_sprites": visible_now,
            "viewport": [get_viewport_rect().size.x, get_viewport_rect().size.y]})
        last_visibility_us = now

func _visible_count() -> int:
    var count: int = 0
    for sprite: Sprite2D in people:
        if sprite.is_visible_in_tree() and get_viewport_rect().encloses(Rect2(sprite.position, Vector2(CELL))):
            count += 1
    return count

func _drawn() -> void:
    if not running or finishing:
        return
    var now := Time.get_ticks_usec()
    intervals_us.append(now-previous_us)
    previous_us = now
    rendered_frames += 1
    if now-start_us >= MEASURE_US:
        running = false
        finishing = true
        _finish.call_deferred(now-start_us)

func _finish(elapsed_us: int) -> void:
    var image := get_viewport().get_texture().get_image()
    var capture_error := image.save_png(output.path_join("crowd_final.png"))
    var unchanged := source_hash == FileAccess.get_sha256("res://Trial3BCrowd.gd")
    for path: String in input_hashes:
        unchanged = unchanged and FileAccess.get_sha256(path) == str(input_hashes[path])
    var fps := float(rendered_frames)*1_000_000.0/float(elapsed_us)
    var measured := elapsed_us >= MEASURE_US and rendered_frames > 0 and hidden_frames == 0 and unchanged and capture_error == OK
    var result := {"status": "DIAGNOSTIC_REJECTED", "P1_eligible": false,
        "reason": "B structural gate failed; standalone diagnostic does not bypass CharacterLab admission",
        "measurement_verified": measured, "elapsed_us": elapsed_us,
        "rendered_frames": rendered_frames, "fps": fps, "measured_60fps_bar": fps >= 60.0,
        "hidden_process_frames": hidden_frames, "visible_sprites_final": _visible_count(),
        "count": COUNT, "sprite_scale": 1, "viewport": [VIEW.x,VIEW.y],
        "composite_method": "original RGBA layer alpha composition at 0,0, no crop or repair",
        "activities": ["walk", "carry"], "other_activities_excluded": true,
        "display_server": DisplayServer.get_name(), "godot": Engine.get_version_info(),
        "rendering_method": RenderingServer.get_current_rendering_method(),
        "frame_intervals_us": intervals_us, "visibility_samples": visibility,
        "input_hashes": input_hashes, "script_sha256": source_hash,
        "inputs_unchanged": unchanged, "capture_error": capture_error,
        "limitations": "Shared-host diagnostic; no quiet-session admission and no production simulation load"}
    var file := FileAccess.open(output.path_join("measurement.json"), FileAccess.WRITE)
    file.store_string(JSON.stringify(result, "  ")+"\n")
    file.close()
    print("B_CROWD_DONE verified=%s frames=%d elapsed_us=%d fps=%.3f P1_eligible=false" % [measured,rendered_frames,elapsed_us,fps])
    get_tree().quit(0 if measured else 1)

func _fail(message: String) -> void:
    push_error("B_CROWD_NOT_VERIFIED " + message)
    get_tree().quit(1)
