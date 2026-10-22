# Standalone generated cut-out person with articulated Bone2D hierarchy.
# Bridge methods: none. All simulation-like motion is explicitly scripted rehearsal.
extends Node2D

@export var asset_root: String = ""
var skeleton: Skeleton2D
var player: AnimationPlayer
var bones: Dictionary = {}
var sprites: Dictionary = {}
var valid: bool = true
var attributes: Dictionary = {}
static var texture_cache: Dictionary = {}
const MASK_SHADER = preload("res://wash_tint.gdshader")
const BODY: Array[String] = ["head", "torso", "upper_arm_l", "upper_arm_r", "forearm_l", "forearm_r", "upper_leg_l", "upper_leg_r", "lower_leg_l", "lower_leg_r", "foot_l", "foot_r"]
const PIVOTS: Dictionary = {"head": Vector2(0.5, 0.65), "torso": Vector2(0.5, 0.15), "upper_arm_l": Vector2(0.35, 0.12), "upper_arm_r": Vector2(0.65, 0.12), "forearm_l": Vector2(0.4, 0.09), "forearm_r": Vector2(0.6, 0.09), "hand_l": Vector2(0.43, 0.1), "hand_r": Vector2(0.57, 0.1), "upper_leg_l": Vector2(0.4, 0.09), "upper_leg_r": Vector2(0.6, 0.09), "lower_leg_l": Vector2(0.5, 0.09), "lower_leg_r": Vector2(0.5, 0.09), "foot_l": Vector2(0.5, 0.09), "foot_r": Vector2(0.5, 0.09)}

func _ready() -> void:
    if asset_root.is_empty():
        asset_root = ProjectSettings.globalize_path("res://../style05/people")
    _build()
    compose(attributes)

func _texture(part: String) -> Texture2D:
    var path: String = asset_root.path_join(part + ".png")
    if str(attributes.get("facing", "south")) in ["east", "west"] and part in BODY:
        path = asset_root.path_join("east").path_join(part + ".png")
    if not FileAccess.file_exists(path):
        path = asset_root.path_join("parts" if part in BODY else "overlays").path_join(part + ".png")
    if part == "head" and str(attributes.get("age", "adult")) != "child":
        var sex_name: String = str(attributes.get("sex", "female"))
        var facing_name: String = "front" if str(attributes.get("facing", "south")) == "south" else "east"
        path = asset_root.path_join("overlays/head_" + sex_name + "_" + facing_name + ".png")
    elif part == "head" and str(attributes.get("sex", "male")) == "female":
        path = asset_root.path_join("overlays/head_female_front.png")
    if texture_cache.has(path):
        return texture_cache[path] as Texture2D
    if not FileAccess.file_exists(path):
        push_error("MISSING_GENERATED_PART " + path)
        valid = false
        return null
    var source: Image = Image.load_from_file(path)
    if source == null or source.is_empty():
        push_error("INVALID_GENERATED_PART " + path)
        valid = false
        return null
    var texture: Texture2D = ImageTexture.create_from_image(source)
    texture_cache[path] = texture
    return texture

func _sprite(part: String, parent: Node2D, size: Vector2, center: Vector2) -> Sprite2D:
    var sprite: Sprite2D = Sprite2D.new()
    sprite.name = part
    sprite.texture = _texture(part)
    sprite.position = center
    sprite.set_meta("fit_size", size)
    if sprite.texture != null:
        var source_size: Vector2 = sprite.texture.get_size()
        sprite.scale = Vector2.ONE * minf(size.x / source_size.x, size.y / source_size.y)
        if PIVOTS.has(part):
            sprite.centered = false
            sprite.offset = -source_size * (PIVOTS[part] as Vector2)
            sprite.position = Vector2.ZERO
    parent.add_child(sprite)
    sprites[part] = sprite
    return sprite

func _bone(part: String, parent: Node2D, origin: Vector2, size: Vector2, center: Vector2) -> Bone2D:
    var bone: Bone2D = Bone2D.new()
    bone.name = part
    bone.position = origin
    bone.set_autocalculate_length_and_angle(false)
    bone.length = maxf(size.y, 1.0)
    parent.add_child(bone)
    bone.rest = bone.transform
    bones[part] = bone
    if not part.begins_with("hand_"):
        _sprite(part, bone, size, center)
    return bone

func _build() -> void:
    skeleton = Skeleton2D.new()
    skeleton.name = "Skeleton2D"
    add_child(skeleton)
    var torso: Bone2D = _bone("torso", skeleton, Vector2.ZERO, Vector2(22, 22), Vector2.ZERO)
    for side: String in ["l", "r"]:
        var sign_x: float = -1.0 if side == "l" else 1.0
        var thigh: Bone2D = _bone("upper_leg_" + side, torso, Vector2(sign_x * 5, 15), Vector2(7, 10), Vector2(0, 5))
        var calf: Bone2D = _bone("lower_leg_" + side, thigh, Vector2(0, 8), Vector2(6, 9), Vector2(0, 4.5))
        _bone("foot_" + side, calf, Vector2(0, 4 if str(attributes.get("facing", "south")) == "south" else 7), Vector2(7, 6), Vector2(0, 3))
        var arm: Bone2D = _bone("upper_arm_" + side, torso, Vector2(sign_x * 8, 1), Vector2(7, 10), Vector2(0, 5))
        var forearm: Bone2D = _bone("forearm_" + side, arm, Vector2(0, 8), Vector2(6, 9), Vector2(0, 4.5))
        forearm.z_index = 3
        _bone("hand_" + side, forearm, Vector2(0, 7), Vector2(6, 6), Vector2(0, 3))
    var head: Bone2D = _bone("head", torso, Vector2(0, -7), Vector2(19, 19), Vector2.ZERO)
    for top: String in ["hide_wrap", "fur_cloak", "woven_tunic"]:
        _sprite("top_" + top, torso, Vector2(25, 25), Vector2(0, 9)).z_index = 2
    for hair: String in ["short", "long_tied", "braided"]:
        var hair_size: Vector2 = Vector2(22, 14) if hair == "short" else Vector2(23, 24)
        var hair_center: Vector2 = Vector2(0, -9) if hair == "short" else Vector2(1, -4)
        _sprite("hair_" + hair, head, hair_size, hair_center).z_index = 4
    _sprite("beard", head, Vector2(16, 12), Vector2(1, 1)).z_index = 4
    _sprite("hat", head, Vector2(22, 15), Vector2(0, -10)).z_index = 5
    var axe: Sprite2D = _sprite("held_stone_axe", bones["hand_r"], Vector2(20, 30), Vector2.ZERO)
    axe.z_index = 6
    axe.centered = false
    if axe.texture != null:
        axe.offset = -axe.texture.get_size() * Vector2(0.80, 0.90)
    _sprite("held_berry_basket", bones["hand_l"], Vector2(17, 15), Vector2(-2, 2)).z_index = 6
    _sprite("held_log_bundle", torso, Vector2(36, 15), Vector2(1, 2)).z_index = 6
    _sprite("hammerstone", bones["hand_r"], Vector2(8, 8), Vector2(0, -2)).z_index = 7
    _sprite("flint_core", bones["hand_l"], Vector2(9, 11), Vector2(0, -2)).z_index = 7
    _sprite("stick", bones["hand_l"], Vector2(8, 31), Vector2(-3, 8)).z_index = 6
    player = AnimationPlayer.new()
    player.name = "AnimationPlayer"
    add_child(player)
    var library: AnimationLibrary = AnimationLibrary.new()
    for state: String in ["idle", "walk", "chop", "carry", "sleep", "knap"]:
        library.add_animation(state, _animation(state))
    player.add_animation_library("", library)

func _track(animation: Animation, path: NodePath, values: Array[float]) -> void:
    var track: int = animation.add_track(Animation.TYPE_VALUE)
    animation.track_set_path(track, path)
    animation.value_track_set_update_mode(track, Animation.UPDATE_CONTINUOUS)
    for index: int in range(values.size()):
        animation.track_insert_key(track, animation.length * float(index) / float(values.size() - 1), values[index])

func _animation(state: String) -> Animation:
    var animation: Animation = Animation.new()
    animation.length = 1.0 if state in ["walk", "carry"] else 1.5
    animation.loop_mode = Animation.LOOP_LINEAR
    var frames: int = 12 if state == "chop" else 8
    for part: String in BODY:
        var values: Array[float] = []
        for frame: int in range(frames + 1):
            var wave: float = sin(TAU * float(frame) / float(frames))
            var angle: float = 0.0
            var side: float = -1.0 if part.ends_with("_l") else 1.0
            if state in ["walk", "carry"] and "leg" in part:
                angle = -(0.55 + wave * 0.30) * side if part.begins_with("upper") else 0.30 + wave * 0.25
            if state == "walk" and "arm" in part:
                angle = -(0.45 + wave * 0.30) * side
            if state == "carry" and "arm" in part:
                angle = side * (-1.9 if str(attributes.get("held", "none")) == "log_bundle" else -0.35) + wave * 0.04
            if state == "chop" and "arm" in part:
                var target: Vector2 = Vector2(side * 1.5, -4.0 - wave * 5.0)
                var shoulder: Vector2 = Vector2(side * 8.0, 1.0)
                var reach: Vector2 = target - shoulder
                var elbow: float = side * acos(clampf((reach.length_squared() - 64.0 - 49.0) / 112.0, -1.0, 1.0))
                var upper: float = reach.angle() - atan2(7.0 * sin(elbow), 8.0 + 7.0 * cos(elbow)) - PI * 0.5
                angle = upper if part.begins_with("upper") else elbow
            if state == "sleep" and part == "torso":
                angle = PI * 0.5
            if state == "knap":
                if "arm" in part:
                    angle = side * (0.85 + wave * 0.18) if part.begins_with("upper") else side * 0.65
                if "leg" in part:
                    angle = side * (-1.2 if part.begins_with("upper") else 1.8)
            if part == "torso" and str(attributes.get("age", "adult")) == "elder":
                angle += deg_to_rad(-8.0)
            values.append(angle)
        var bone: Bone2D = bones[part] as Bone2D
        _track(animation, NodePath(str(get_path_to(bone)) + ":rotation"), values)
    if state == "chop":
        var axe_angles: Array[float] = []
        for frame: int in range(frames + 1):
            var wave: float = sin(TAU * float(frame) / float(frames))
            var reach: Vector2 = Vector2(1.5, -4.0 - wave * 5.0) - Vector2(8.0, 1.0)
            var elbow: float = acos(clampf((reach.length_squared() - 113.0) / 112.0, -1.0, 1.0))
            var upper: float = reach.angle() - atan2(7.0 * sin(elbow), 8.0 + 7.0 * cos(elbow)) - PI * 0.5
            axe_angles.append(-0.35 + wave * 0.65 - upper - elbow)
        _track(animation, NodePath(str(get_path_to(sprites["held_stone_axe"])) + ":rotation"), axe_angles)
    var breath: Array[float] = [1.0, 1.0]
    if state == "idle":
        breath.insert(1, 1.025)
    _track(animation, NodePath("Skeleton2D/torso:scale:y"), breath)
    return animation

func _tint(part: String, color: Color, neutral: float) -> void:
    var sprite: Sprite2D = sprites[part] as Sprite2D
    var material: ShaderMaterial = ShaderMaterial.new()
    material.shader = MASK_SHADER
    material.set_shader_parameter("replacement", color)
    material.set_shader_parameter("reference_luma", neutral)
    material.set_shader_parameter("skin_only", part in BODY)
    sprite.material = material

func compose(attrs: Dictionary) -> void:
    attributes = attrs.duplicate(true)
    if skeleton == null:
        return
    var age: String = str(attrs.get("age", "adult"))
    scale = Vector2.ONE * (0.75 if age == "child" else 1.0)
    skeleton.rotation = 0.0
    skeleton.scale.x = -1.0 if str(attrs.get("facing", "south")) == "west" else 1.0
    queue_redraw()
    (bones["head"] as Bone2D).scale = Vector2.ONE * (1.15 if age == "child" else 1.0)
    for part: String in BODY:
        var body_sprite: Sprite2D = sprites[part] as Sprite2D
        body_sprite.texture = _texture(part)
        if body_sprite.texture != null:
            var source_size: Vector2 = body_sprite.texture.get_size()
            var fit_size: Vector2 = body_sprite.get_meta("fit_size") as Vector2
            body_sprite.scale = Vector2.ONE * minf(fit_size.x / source_size.x, fit_size.y / source_size.y)
            body_sprite.offset = -source_size * (PIVOTS[part] as Vector2)
        if part not in ["foot_l", "foot_r"]:
            _tint(part, Color(str(attrs.get("skin", "#bc916a"))), 0.65)
    for side: String in ["l", "r"]:
        (bones["foot_" + side] as Bone2D).position.y = 4.0 if str(attrs.get("facing", "south")) == "south" else 7.0
    var top_name: String = "top_" + str(attrs.get("top", "hide_wrap"))
    var hair_name: String = "hair_" + str(attrs.get("hair", "short"))
    for part: String in sprites:
        var sprite: Sprite2D = sprites[part] as Sprite2D
        if part.begins_with("top_"):
            sprite.visible = part == top_name
            _tint(part, Color(str(attrs.get("top_tint", "#aa8965"))), 0.60)
        elif part.begins_with("hair_"):
            sprite.visible = part == hair_name
            sprite.material = null
            if age == "elder":
                _tint(part, Color("#c7bfae"), 0.35)
        elif part.begins_with("held_"):
            sprite.visible = part == "held_" + str(attrs.get("held", "none"))
    (sprites["beard"] as Sprite2D).visible = false
    (sprites["beard"] as Sprite2D).material = null
    if age == "elder":
        _tint("beard", Color("#c7bfae"), 0.35)
    (sprites["stick"] as Sprite2D).visible = age == "elder"
    (sprites["hat"] as Sprite2D).visible = bool(attrs.get("hat", false))
    var state: String = str(attrs.get("state", "idle"))
    (bones["torso"] as Bone2D).position.y = 6.0 if state == "knap" else 0.0
    (sprites["held_stone_axe"] as Sprite2D).rotation = 0.0
    (sprites["hammerstone"] as Sprite2D).visible = state == "knap"
    (sprites["flint_core"] as Sprite2D).visible = state == "knap"
    if not player.has_animation(state):
        push_error("UNKNOWN_PERSON_STATE " + state)
        valid = false
        return
    player.stop()
    var library: AnimationLibrary = player.get_animation_library("")
    for animation_name: String in ["idle", "walk", "chop", "carry", "sleep", "knap"]:
        library.remove_animation(animation_name)
        library.add_animation(animation_name, _animation(animation_name))
    player.play(state)
    player.advance(float(attrs.get("phase", 0.0)))

func chop_grip_samples() -> Array[Dictionary]:
    var state: String = str(player.current_animation)
    var old_position: float = player.current_animation_position
    var samples: Array[Dictionary] = []
    player.play("chop")
    for frame: int in range(13):
        player.seek(float(frame) * 1.5 / 12.0, true)
        var left: Vector2 = (bones["hand_l"] as Bone2D).global_position
        var right: Vector2 = (bones["hand_r"] as Bone2D).global_position
        samples.append({"frame": frame, "left": [left.x, left.y], "right": [right.x, right.y], "distance_px": left.distance_to(right)})
    player.play(state)
    player.seek(old_position, true)
    return samples

func _draw() -> void:
    # Contact shadow is a ground mark, never a substitute for generated anatomy.
    var sleeping: bool = str(attributes.get("state", "idle")) == "sleep"
    var facing_sign: float = -1.0 if str(attributes.get("facing", "south")) == "west" else 1.0
    var shadow_center: Vector2 = Vector2(-8.0 * facing_sign, 4.0) if sleeping else Vector2(0, 31)
    draw_set_transform(shadow_center, 0.0, Vector2(1.0, 0.3))
    draw_circle(Vector2.ZERO, 26.0 if sleeping else 13.0, Color(0.12, 0.10, 0.08, 0.20))
    draw_set_transform(Vector2.ZERO)
