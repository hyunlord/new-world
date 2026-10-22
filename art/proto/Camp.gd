# Standalone visual/performance harness: generated textures, articulated people.
# Bridge methods: none. Movement is a scripted loop, not game simulation.
extends Node2D

const COMPOSER = preload("res://PersonComposer.gd")
const COMPANION = preload("res://Companion.gd")
const SEASON = preload("res://season.gdshader")
var options: Dictionary = {}
var people: Array[Node2D] = []
var starts: Array[Vector2] = []
var camera: Camera2D
var output_dir: String = "/Users/rexxa/work/arttrial2-data/proto"
var env_root: String = ""
var autotile_root: String = ""
var people_root: String = ""
var valid: bool = true
var running: bool = false
var elapsed: float = 0.0
var measurement_start: int = 0
var frame_us: Array[int] = []
var rendered_frames: int = 0
var previous_frame: int = 0
var mode: String = "camp"
var count: int = 24
var all_walk: bool = false
var zoom_level: float = 1.0
var textures: Dictionary = {}
var asset_hashes: Dictionary = {}
var answer_key: Array[Dictionary] = []
var animation_samples: Array[Dictionary] = []
var chop_grips: Array[Dictionary] = []
var source_hashes: Dictionary = {}

func _ready() -> void:
    for source: String in ["Camp.gd", "PersonComposer.gd", "Companion.gd", "season.gdshader", "tint_mask.gdshader", "wash_tint.gdshader", "project.godot", "scenes/camp.tscn"]:
        source_hashes[source] = FileAccess.get_sha256("res://" + source)
    var arguments: PackedStringArray = OS.get_cmdline_user_args()
    var index: int = 0
    while index < arguments.size():
        if arguments[index].begins_with("--") and index + 1 < arguments.size():
            options[arguments[index].trim_prefix("--")] = arguments[index + 1]
            index += 2
        else:
            index += 1
    output_dir = str(options.get("out", output_dir))
    env_root = str(options.get("env-root", ProjectSettings.globalize_path("res://../style05/env")))
    people_root = str(options.get("people-root", ProjectSettings.globalize_path("res://../style05/people")))
    autotile_root = str(options.get("autotile-root", ProjectSettings.globalize_path("res://../style05/autotile")))
    mode = str(options.get("mode", "camp"))
    all_walk = mode == "camp" and str(options.get("all-walk", "false")) == "true"
    count = int(options.get("count", "24"))
    zoom_level = float(options.get("zoom", "1"))
    if mode == "camp" and zoom_level == 2.0:
        get_window().size = Vector2i(2816, 1664)
    elif mode == "camp":
        get_window().size = Vector2i(1408, 832)
    if mode == "people":
        count = 12
    DirAccess.make_dir_recursive_absolute(output_dir)
    DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
    camera = Camera2D.new()
    camera.position = Vector2(640, 360)
    if mode == "camp":
        camera.position = Vector2(704, 416)
    camera.zoom = Vector2.ONE * zoom_level
    add_child(camera)
    if mode == "season_compare":
        var base: Sprite2D = _place(env_root, str(options.get("ground", "meadow")) + ".png", Vector2(320, 360), Vector2(512, 512), -10)
        base.material = _season_material()
        _place(str(options.get("reference-root", "")), str(options.get("reference", "")), Vector2(960, 360), Vector2(512, 512), -10)
    else:
        _ground(str(options.get("ground", "meadow")))
    if mode not in ["people", "season", "season_compare"]:
        _environment()
    if mode not in ["season", "season_compare"]:
        _people()
    else:
        count = 0
    if not valid:
        push_error("PROTO_ASSETS_MISSING: no placeholder body or environment substituted")
        get_tree().quit(1)
        return
    if mode == "people":
        _json("answer_key.json", answer_key)
    print("PROTO_READY people=%d mode=%s renderer=Sprite2D bones=Bone2D zoom=%s" % [count, mode, zoom_level])
    RenderingServer.frame_post_draw.connect(_frame_drawn)
    _begin.call_deferred()

func _find_file(root: String, filename: String) -> String:
    if FileAccess.file_exists(root.path_join(filename)):
        return root.path_join(filename)
    var directories: PackedStringArray = DirAccess.get_directories_at(root)
    directories.sort()
    for directory: String in directories:
        if FileAccess.file_exists(root.path_join(directory).path_join(filename)):
            return root.path_join(directory).path_join(filename)
    return root.path_join(filename)

func _texture(root: String, filename: String) -> Texture2D:
    var path: String = _find_file(root, filename)
    if textures.has(path):
        return textures[path] as Texture2D
    if not FileAccess.file_exists(path):
        push_error("MISSING_GENERATED_ENV " + path)
        valid = false
        return null
    var source: Image = Image.load_from_file(path)
    if source == null or source.is_empty():
        valid = false
        push_error("INVALID_GENERATED_ENV " + path)
        return null
    var texture: Texture2D = ImageTexture.create_from_image(source)
    textures[path] = texture
    asset_hashes[path] = FileAccess.get_sha256(path)
    return texture

func _place(root: String, filename: String, point: Vector2, size: Vector2, layer: int = 0) -> Sprite2D:
    var sprite: Sprite2D = Sprite2D.new()
    sprite.texture = _texture(root, filename)
    sprite.position = point
    sprite.z_index = layer
    if sprite.texture != null:
        sprite.scale = size / sprite.texture.get_size()
    add_child(sprite)
    return sprite

func _season_material() -> ShaderMaterial:
    var material: ShaderMaterial = ShaderMaterial.new()
    material.shader = SEASON
    material.set_shader_parameter("season", int(options.get("season", "1")))
    return material

func _ground(kind: String) -> void:
    var material: ShaderMaterial = _season_material()
    for row: int in range(-1, 3):
        for col: int in range(-1, 4):
            var ground: Sprite2D = _place(env_root, kind + ".png", Vector2(col * 512 + 256, row * 512 + 256), Vector2(512, 512), -10)
            ground.material = material

func _environment() -> void:
    _place(env_root, "hut.png", Vector2(900, 220), Vector2(192, 192), 1)
    _place(env_root, "hearth_0.png", Vector2(790, 420), Vector2(64, 64), 1)
    _place(env_root, "log_pile_small.png", Vector2(900, 440), Vector2(64, 64), 1)
    for n: int in range(5):
        var tree: Sprite2D = _place(env_root, "oak_large.png", Vector2(100 + n * 230, 72), Vector2(128, 128), 2)
        tree.material = _season_material()
    _enclosure()
    var companion: Node2D = COMPANION.new()
    companion.position = Vector2(760, 580)
    companion.z_index = 10
    add_child(companion)

func _enclosure() -> void:
    var occupied: Dictionary = {}
    for y: int in range(1, 7):
        for x: int in range(1, 9):
            var floor_name: String = "floor_01.png" if y == 6 else "floor_00.png"
            var floor_sprite: Sprite2D = _place(autotile_root, floor_name, Vector2(64, 112) + Vector2(x, y) * 64, Vector2(64, 64), -5)
            if y == 6:
                floor_sprite.flip_v = true
    for y: int in range(8):
        for x: int in range(10):
            if y == 0 or y == 7 or x == 0 or x == 9 or (x == 5 and y >= 2 and y <= 6):
                occupied[Vector2i(x, y)] = true
    occupied.erase(Vector2i(4, 7))
    occupied.erase(Vector2i(5, 7))
    var offsets: Array[Vector2i] = [Vector2i.UP, Vector2i.RIGHT, Vector2i.DOWN, Vector2i.LEFT]
    var cells: Array = occupied.keys()
    cells.sort_custom(func(a: Vector2i, b: Vector2i) -> bool: return a.y < b.y or (a.y == b.y and a.x < b.x))
    var seam_pairs: Array[Dictionary] = []
    for cell: Vector2i in cells:
        var mask: int = 0
        for direction: int in range(4):
            if occupied.has(cell + offsets[direction]):
                mask |= 1 << direction
        _place(autotile_root, "wall_%02d.png" % mask, Vector2(64, 112) + Vector2(cell) * 64, Vector2(64, 64), 1)
        seam_pairs.append({"cell": [cell.x, cell.y], "mask": mask})
    _json("enclosure_cells.json", seam_pairs)

func _attributes(index: int) -> Dictionary:
    var states: Array[String] = ["idle", "walk", "chop", "carry", "sleep", "knap", "walk", "carry", "sleep", "idle", "chop", "walk"]
    var state: String = states[index % 12]
    var held: String = "none"
    if state == "carry":
        held = "log_bundle" if index % 12 == 3 else "berry_basket"
    elif state == "chop":
        held = "stone_axe"
    elif state == "knap":
        held = "knapping_stones"
    if all_walk:
        state = "walk"
    var age: String = "child" if index % 12 in [6, 9] else ("elder" if index % 12 == 11 else "adult")
    return {"facing": ["south", "east", "west"][index % 3], "age": age, "sex": "female" if index % 2 == 0 else "male", "sex_rendered": true, "hair": ["short", "long_tied", "braided"][index % 3], "top": ["hide_wrap", "fur_cloak", "woven_tunic"][index % 3], "skin": ["#bc916a", "#8b6045", "#d6af88"][index % 3], "top_tint": ["#9f8461", "#798065", "#9a7054"][index % 3], "held": held, "state": state, "hat": index % 5 == 0, "phase": float(index % 8) / 8.0}

func _people() -> void:
    for index: int in range(count):
        var person: Node2D = COMPOSER.new()
        person.asset_root = people_root
        person.attributes = _attributes(index)
        var point: Vector2
        if mode == "people":
            point = Vector2(370 + (index % 4) * 180, 220 + floori(float(index) / 4.0) * 140)
        else:
            var columns: int = 20 if count > 24 else 8
            point = Vector2(96 + (index % columns) * 64, 160 + floori(float(index) / float(columns)) * 64)
        person.position = point
        person.z_index = 5
        add_child(person)
        valid = valid and person.valid
        people.append(person)
        starts.append(point)
        answer_key.append({"index": index + 1, "world_position": [point.x, point.y], "attributes": person.attributes})
        if person.attributes["state"] == "chop":
            chop_grips.append({"person": index + 1, "samples": person.chop_grip_samples()})

func _begin() -> void:
    await get_tree().process_frame
    await get_tree().process_frame
    if DisplayServer.get_name() != "headless":
        await RenderingServer.frame_post_draw
    running = true
    measurement_start = Time.get_ticks_usec()
    previous_frame = measurement_start

func _frame_drawn() -> void:
    if running:
        rendered_frames += 1

func _process(delta: float) -> void:
    if not running:
        return
    elapsed += delta
    if animation_samples.size() < 4 and elapsed >= float(animation_samples.size()) * 0.25:
        var snapshot: Dictionary = {"elapsed": elapsed, "people": []}
        for person: Node2D in people:
            var transforms: Dictionary = {}
            for bone_name: String in person.bones:
                var bone: Bone2D = person.bones[bone_name] as Bone2D
                transforms[bone_name] = {"rotation": bone.rotation, "scale": [bone.scale.x, bone.scale.y]}
            snapshot["people"].append({"animation": person.player.current_animation, "position": person.player.current_animation_position, "bones": transforms})
        animation_samples.append(snapshot)
    var now: int = Time.get_ticks_usec()
    frame_us.append(now - previous_frame)
    previous_frame = now
    if mode != "people":
        for index: int in range(people.size()):
            var phase: float = elapsed * 0.4 + float(index) * 0.3
            if str(people[index].attributes["state"]) in ["walk", "carry"]:
                people[index].position = starts[index] + Vector2(sin(phase) * 8.0, cos(phase) * 8.0)
    var duration: float = float(options.get("seconds", "60"))
    if float(now - measurement_start) / 1000000.0 >= duration:
        running = false
        _finish.call_deferred()

func _finish() -> void:
    var wall_seconds: float = float(Time.get_ticks_usec() - measurement_start) / 1000000.0
    var sorted_us: Array[int] = frame_us.duplicate()
    sorted_us.sort()
    var result: Dictionary = {"asset_status": "CANDIDATE_ONLY_UNLESS_SEPARATE_JUDGE_EVIDENCE", "mode": mode, "people": count, "zoom": zoom_level, "wall_seconds": wall_seconds, "frames": frame_us.size(), "fps": float(frame_us.size()) / wall_seconds, "frame_p95_ms": float(sorted_us[mini(floori(float(sorted_us.size()) * 0.95), sorted_us.size() - 1)]) / 1000.0, "renderer": "Sprite2D cutouts under Bone2D/Skeleton2D", "driver": RenderingServer.get_current_rendering_driver_name(), "display": DisplayServer.get_name(), "vsync": DisplayServer.window_get_vsync_mode(), "headless": DisplayServer.get_name() == "headless", "animations": ["idle", "walk", "chop", "carry", "sleep", "knap"], "asset_roots": {"people": people_root, "environment": env_root, "autotile": autotile_root}, "environment_hashes": asset_hashes}
    var part_hashes: Dictionary = {}
    for path: String in COMPOSER.texture_cache:
        part_hashes[path] = FileAccess.get_sha256(path)
    result["person_hashes"] = part_hashes
    result["all_walk"] = all_walk
    if mode == "people" and options.has("capture-time"):
        var capture_time: float = float(options["capture-time"])
        for person: Node2D in people:
            person.player.pause()
            person.player.seek(fmod(capture_time + float(person.attributes.get("phase", 0.0)), person.player.current_animation_length), true)
        result["capture_animation_time"] = capture_time
    result["process_frames"] = frame_us.size()
    result["process_fps"] = float(frame_us.size()) / wall_seconds
    result["rendered_frames"] = rendered_frames
    result["rendered_fps"] = float(rendered_frames) / wall_seconds
    result["measurement_start_event"] = "first_frame_post_draw" if DisplayServer.get_name() != "headless" else "headless_process_only"
    var playing: Array[String] = []
    for person: Node2D in people:
        var animation_name: String = str(person.player.current_animation)
        if animation_name not in playing:
            playing.append(animation_name)
    result["animations"] = playing
    result["viewport_size"] = [get_viewport_rect().size.x, get_viewport_rect().size.y]
    result["population_audit"] = _population_audit()
    result["source_hashes_at_start"] = source_hashes
    if DisplayServer.get_name() != "headless":
        await RenderingServer.frame_post_draw
        var capture: Image = get_viewport().get_texture().get_image()
        var error: Error = capture.save_png(output_dir.path_join("capture.png"))
        if error != OK:
            push_error("CAPTURE_SAVE_FAILED %d" % error)
            get_tree().quit(1)
            return
        result["capture_sha256"] = FileAccess.get_sha256(output_dir.path_join("capture.png"))
    _json("runtime.json", result)
    _json("animation_samples.json", animation_samples)
    _json("chop_grips.json", chop_grips)
    _json("frame_times_us.json", frame_us)
    print("PROTO_DONE " + JSON.stringify(result))
    get_tree().quit(0)

func _population_audit() -> Dictionary:
    var occupancy: Dictionary = {}
    var fully_visible: int = 0
    var positions: Array[Dictionary] = []
    for index: int in range(people.size()):
        var person: Node2D = people[index]
        var cell: Vector2i = Vector2i(floori(person.position.x / 64.0), floori(person.position.y / 64.0))
        var key: String = "%d,%d" % [cell.x, cell.y]
        occupancy[key] = int(occupancy.get(key, 0)) + 1
        var bounds: Rect2 = Rect2()
        var first: bool = true
        for part: String in person.sprites:
            var sprite: Sprite2D = person.sprites[part] as Sprite2D
            if sprite.is_visible_in_tree():
                var rect: Rect2 = sprite.get_global_transform_with_canvas() * sprite.get_rect()
                bounds = rect if first else bounds.merge(rect)
                first = false
        if not first and get_viewport_rect().encloses(bounds):
            fully_visible += 1
        positions.append({"person": index + 1, "position": [person.position.x, person.position.y], "tile": [cell.x, cell.y], "screen_bounds": [bounds.position.x, bounds.position.y, bounds.size.x, bounds.size.y]})
    var max_occupancy: int = 0
    for value: int in occupancy.values():
        max_occupancy = maxi(max_occupancy, value)
    return {"max_people_per_tile": max_occupancy, "unique_tiles": occupancy.size(), "fully_visible_people": fully_visible, "positions": positions}

func _json(filename: String, value: Variant) -> void:
    var file: FileAccess = FileAccess.open(output_dir.path_join(filename), FileAccess.WRITE)
    if file == null:
        push_error("EVIDENCE_WRITE_FAILED " + filename)
        valid = false
        return
    file.store_string(JSON.stringify(value, "  ") + "\n")
