# agent_layer.gd (ColonyView/AgentLayer in scenes/main.tscn): the people of the colony view (M-ART2 §3.5).
# Each person is a pooled Node2D in the y-sorted ColonyView/World, positioned at its feet — the tile's base point
#   (View.base_point, the route I contract: feet at (tile_left + 32, tile_top + 43)) — so walls, trees, structures and
#   animals sort with people by base y. Children: a 28 × 8 contact shadow (code-drawn ellipse, `people.shadow_*`) and
#   the body: the route I frame of the person's look (the bridge's `look_ids` index into manifest.json `looks`; the two
#   missing looks resolve to their fallback frames) for the state: sleep → `sleep`; carrying → `carry` (+ a 1-px bob
#   while moving); moving → `idle` / `walk_a` alternating at `people.walk_hz` in game time (+ bob on walk_a); working
#   (consuming its job) → `work_a` / `idle` at `people.work_hz`; else `idle` (HANDOVER frame plan). Facing: the last
#   horizontal step picks E or W (W = E flipped); a north/south step shows S (N = S until N art exists). The frame
#   is anchored by its canvas anchor ((40, 104) upright, (56, 72) asleep). The body's shader
#   (shaders/person_variant.gdshader) recolours skin and hair through the frame's fixed mask by the variant's Lab
#   shift (variant_transforms.json; elders get the grey hair target `people.grey_hair_rgb`) and draws the selection
#   (yellow) or drafted (red, `people.drafted_outline_px` wide) outline; otherwise, at night away from a fire, the same 1-px outline in the pale
#   `people.night_halo_rgba` at up to `people.night_halo_alpha` × DaylightController.dark_at(feet) (craft rule 5:
#   a halo where people could be lost; MART2_DECISIONS D17). Sleepers sharing a tile fan out for drawing only.
# LOD: at or below camera zoom `lod.marker_zoom_max` bodies hide; half zoom retains route-I figures and shadows.
#   Half bodies draw above scenery with a pale contour; overlapping true art is translucent. Exact coincident feet
#   get full-snapshot count annotations; colliding label/body footprints coalesce into aggregate counts.
#   Badge inspection cycles aggregate identities; body inspection stays within its original exact-foot group.
#   At quarter zoom `lod.marker_fill_px` SCREEN pixels in top colour have bright, household and dark-outline rings, plus at
#   `lod.glyph_zoom` an activity glyph (axe working, log carrying, zz sleeping, ! fleeing). The disc's ring is the
#   person's household colour (M-ART2.1 §2.3): a household is a couple and the children of its members — an adult's
#   key is the lower AgentId of themselves and a living partner, a child's is their living mother's household, else
#   their living father's, else their own (`household_keys`, from the snapshot's `mothers` / `fathers` /
#   `partners`); the key's hash picks one of `lod.household_ring_rgb`. Recomputed only when those columns change.
# Marks: a sick person shows a small green badge, a fleeing one a "!" above the head (code-drawn); M toggles the mood
#   overlay (each person tinted by `mood_bands`, StoryText.band_tint).
# The drawn rect is what clicks hit (`entity_at`); positions tween between tiles over one move step.
# Also feeds SettlementSelection.observe every frame (the inspector's village row; V8-0 DECISIONS D31).
# Bridge methods: via SimReads get_agent_snapshot (since M-ART2.1 also `mothers`, `fathers`, `partners`), get_world_info, get_society_snapshot, get_settlement_snapshot;
#   signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Tone = preload("res://scripts/world/art/tone.gd")
const SettlementSelection = preload("res://scripts/core/settlement_selection.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const OverlayLabel = preload("res://scripts/world/colony/overlay_layer.gd")
const MarkerLayout = preload("res://scripts/world/colony/marker_layout.gd")
const VARIANT_SHADER: Shader = preload("res://shaders/person_variant.gdshader")
## Snapshot state codes (bridge contract, not content): consuming; target kinds sleep and job.
const STATE_CONSUMING: int = 3
const TARGET_SLEEP: int = 3
const TARGET_JOB: int = 9
## Snapshot `life_stages` code of a child (bridge contract); the salt of the household colour hash.
const LIFE_CHILD: int = 1
const HOUSEHOLD_SALT: int = 11
const MOOD_KEY: Key = KEY_M
const INK: Color = Color8(35, 29, 23)
const BADGE: Color = Color(0.45, 0.78, 0.36)
## The contact shadow's colour (the compositor's (16, 15, 12)).
const SHADOW_RGB: Color = Color8(16, 15, 12)
const FLEE_MARK: Color = Color(0.94, 0.75, 0.25)
const LodMarkers = preload("res://scripts/world/colony/lod_markers.gd")
## Activity → glyph index of lod_markers.gd (axe working, log carrying, zz sleeping, ! fleeing).
const GLYPH_OF: Dictionary = {"work": 0, "carry": 1, "sleep": 2, "flee": 3}
## Sleepers sharing a tile fan out by these offsets (world px).
const FAN: Array[Vector2] = [Vector2(0, 0), Vector2(0, -24), Vector2(0, 24), Vector2(-8, -48), Vector2(8, 48)]

signal mood_toggled(on: bool)

## Microseconds of the last frame's update and of the last LOD marker draw (performance reports).
var last_process_usec: int = 0
var last_draw_usec: int = 0
var mux2_render_profile: Dictionary = {}
var _render_profiling: bool = OS.get_environment("WORLDSIM_MUX2_RENDER_PROFILE") == "1"

var _world_sim: Node = null
var _world: Node2D = null
var _persons: Dictionary = {}
var _selected_bits: int = -1
var _frame: int = 0
var _clock: float = 0.0
var _mood_shown: bool = false
var _lod: bool = false
var _zoom: float = 1.0
var _shadow_tex: ImageTexture = null
var _masks: Dictionary = {}
var _transforms: Dictionary = {}
var _top_colours: Dictionary = {}
var _params_version: int = -1
var _font: Font = null
var _no_outline: Color = Color(0, 0, 0, 0)
var _selection_rgba: Color = Color(0.94, 0.89, 0.67)
var _drafted_rgba: Color = Color(0.77, 0.2, 0.16)
var _outline_px: float = 1.0
var _halo_rgba: Color = Color(0.91, 0.89, 0.81)
var _halo_alpha: float = 0.0
## Halo alpha steps (an outline change re-sets one shader parameter, so the halo moves in 1/HALO_STEPS steps).
const HALO_STEPS: int = 16
var _drafted_px: float = 2.0
var _daylight: Node = null
var _markers: LodMarkers = null
var _ringed: Array[Person] = []
var _marker_px: int = 10
var _half_shader: Shader
var _half_rim: Color = Color(0.95, 0.95, 0.88, 1)
var _half_rim_px: float = 1.0
var _half_overlap_alpha: float = 0.8
var _opaque_rects: Dictionary = {}
var _coincident_badges: Array[Dictionary] = []
var _coincident_groups: Array[Dictionary] = []
var badge_layout_exhausted: int = 0
var _badge_leaders: BadgeLeaders
var _household_key: int = 0
## AgentId → household key from the last kin columns (looked up when markers refresh, so new people get theirs).
var _household_of: Dictionary = {}


class BadgeLeaders:
	extends Node2D
	var segments: PackedVector2Array = PackedVector2Array()
	var light: Color = Color.WHITE
	var scale_width: float = 1.0
	var aggregate_segments: PackedVector2Array = PackedVector2Array()
	var active_segments: PackedVector2Array = PackedVector2Array()
	var extent_style: Dictionary = {}

	func _draw() -> void:
		for entry: Dictionary in [{"points": aggregate_segments, "active": false}, {"points": active_segments, "active": true}]:
			var points: PackedVector2Array = entry["points"]
			if not points.is_empty():
				draw_multiline(points, extent_style["dark"], float(extent_style["dark_width"]) * scale_width)
				draw_multiline(points, extent_style["active_light"] if bool(entry["active"]) else extent_style["light"], float(extent_style["light_width"]) * scale_width)
		if not segments.is_empty():
			draw_multiline(segments, Color8(35, 29, 23), 2.0 * scale_width)
			draw_multiline(segments, light, scale_width)


class Person:
	extends Node2D
	var bits: int = 0
	var agent_id: int = 0
	var tile: Vector2i = Vector2i.ZERO
	var from_pos: Vector2 = Vector2.ZERO
	var to_pos: Vector2 = Vector2.ZERO
	var elapsed: float = 1.0
	var duration: float = 0.25
	var facing: String = "S"
	var west: bool = false
	var pose: String = ""
	var activity: String = "idle"
	var look: int = -1
	var frames: String = ""
	var variant: int = -1
	var elder: bool = false
	var seen: int = 0
	var bed: Vector2 = Vector2.ZERO
	var drafted: bool = false
	var selected: bool = false
	var sick: bool = false
	var fleeing: bool = false
	var top_colour: Color = Color.WHITE
	var outline: Color = Color(0, 0, 0, 0)
	var readable: bool = false
	var marker_position: Vector2 = Vector2.ZERO
	var shadow: Sprite2D
	var body: Sprite2D
	var mat: ShaderMaterial
	var marks: Node2D


func _ready() -> void:
	# Share the exact route-I recolour shader; only half zoom bypasses night lighting.
	_half_shader = Shader.new()
	_half_shader.code = VARIANT_SHADER.code.replace("shader_type canvas_item;", "shader_type canvas_item;\nrender_mode unshaded;")
	z_as_relative = false
	z_index = 4095
	var marker_material: CanvasItemMaterial = CanvasItemMaterial.new()
	marker_material.light_mode = CanvasItemMaterial.LIGHT_MODE_UNSHADED
	material = marker_material
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_badge_leaders = BadgeLeaders.new()
	_badge_leaders.z_as_relative = false
	_badge_leaders.z_index = 4093
	_badge_leaders.material = marker_material
	add_child(_badge_leaders)
	_world = get_node_or_null("../World")
	_font = load("res://assets/fonts/galmuri/Galmuri11.ttf") as Font
	_markers = LodMarkers.new()
	_markers.name = "LodMarkers"
	add_child(_markers)
	_load_people_data()
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	_daylight = get_node_or_null("/root/Main/DaylightController")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	for person: Person in _persons.values():
		person.queue_free()
	_persons.clear()
	_clear_coincident_badges()
	queue_redraw()
	_household_key = 0
	_household_of = {}


func _load_people_data() -> void:
	_transforms.clear()
	var vt: Variant = JSON.parse_string(FileAccess.get_file_as_string(ArtData.path("variant_transforms")))
	if vt is Dictionary:
		for row: Variant in (vt as Dictionary).get("transforms", []):
			if row is Dictionary:
				_transforms["%s/%s" % [str(row.get("look", "")), str(row.get("variant", ""))]] = row
	var tc: Variant = JSON.parse_string(FileAccess.get_file_as_string(ArtData.path("top_colours")))
	_top_colours = tc if tc is Dictionary else {}
	_make_shadow()


func _make_shadow() -> void:
	var size: Variant = ArtData.param("people.shadow_size", [28, 8])
	var w: int = int(size[0]) if size is Array else 28
	var h: int = int(size[1]) if size is Array else 8
	var alpha: float = float(ArtData.param("people.shadow_alpha", 0.37))
	var img: Image = Image.create(w, h, false, Image.FORMAT_RGBA8)
	for y: int in h:
		for x: int in w:
			var dx: float = (x + 0.5 - w * 0.5) / (w * 0.5)
			var dy: float = (y + 0.5 - h * 0.5) / (h * 0.5)
			if dx * dx + dy * dy <= 1.0:
				img.set_pixel(x, y, Color(SHADOW_RGB, alpha))
	_shadow_tex = ImageTexture.create_from_image(img)


## The mask texture of a frame (R skin, G hair); null when the fixed masks are missing (then nothing is recoloured).
func _mask(frames: String, facing: String, pose: String) -> Texture2D:
	var key: String = "%s/%s/%s" % [frames, facing, pose]
	if not _masks.has(key):
		var p: String = ArtData.ROOT + "people/masks/%s.png" % key
		_masks[key] = load(p) as Texture2D if ResourceLoader.exists(p) else null
	return _masks[key]


func _make_person(bits: int, agent_id: int, tile: Vector2i) -> Person:
	var person: Person = Person.new()
	person.bits = bits
	person.agent_id = agent_id
	person.tile = tile
	person.position = View.base_point(tile)
	person.from_pos = person.position
	person.to_pos = person.position
	person.shadow = Sprite2D.new()
	person.shadow.texture = _shadow_tex
	person.shadow.centered = true
	person.add_child(person.shadow)
	person.body = Sprite2D.new()
	person.body.centered = false
	person.mat = ShaderMaterial.new()
	person.mat.shader = VARIANT_SHADER
	person.mat.set_shader_parameter("outline_px", _outline_px)
	person.body.material = person.mat
	person.add_child(person.body)
	person.marks = Node2D.new()
	person.marks.draw.connect(_draw_marks.bind(person))
	person.add_child(person.marks)
	person.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	if _world != null:
		_world.add_child(person)
	else:
		add_child(person)
	return person


## Point the person at look `look` / variant `variant` (bridge appearance columns).
func _set_appearance(person: Person, look: int, variant: int, elder: bool) -> void:
	if person.look == look and person.variant == variant and person.elder == elder:
		return
	person.look = look
	person.variant = variant
	person.elder = elder
	var row: Dictionary = ArtData.look(look)
	person.frames = str(row.get("frames", ""))
	var c: Variant = _top_colours.get(person.frames, null)
	person.top_colour = Color8(int(c[0]), int(c[1]), int(c[2])) if c is Array and (c as Array).size() == 3 else Color(0.6, 0.45, 0.3)
	var variants: Array = (ArtData.manifest().get("people", {}) as Dictionary).get("variants", [])
	var vid: String = str(variants[variant]) if variant >= 0 and variant < variants.size() else ""
	var t: Dictionary = _transforms.get("%s/%s" % [person.frames, vid], {})
	var skin: Variant = t.get("skin_delta_lab", [0, 0, 0])
	var hair: Variant = t.get("hair_delta_lab", [0, 0, 0])
	var hair_delta: Vector3 = Vector3(float(hair[0]), float(hair[1]), float(hair[2]))
	if elder:
		var grey: Variant = (ArtData.bindings().get("people", {}) as Dictionary).get("grey_hair_rgb", null)
		var src: Variant = t.get("source_hair_lab", null)
		if grey is Array and src is Array:
			var target: Vector3 = Tone.rgb_to_lab(Vector3(float(grey[0]) / 255.0, float(grey[1]) / 255.0, float(grey[2]) / 255.0))
			hair_delta = target - Vector3(float(src[0]), float(src[1]), float(src[2]))
	person.mat.set_shader_parameter("skin_delta", Vector3(float(skin[0]), float(skin[1]), float(skin[2])))
	person.mat.set_shader_parameter("hair_delta", hair_delta)
	person.pose = ""


func _set_frame(person: Person, pose: String, bob: int) -> void:
	_set_frame_cached(person, pose, bob, {})


func _set_frame_cached(person: Person, pose: String, bob: int, resources: Dictionary) -> void:
	var facing: String = "E" if person.facing == "E" else "S"
	var pose_key: String = "%s/%s" % [facing, pose]
	if person.pose != pose_key:
		person.pose = pose_key
		var key: String = "%s/%s/%s" % [person.frames, facing, pose]
		if not resources.has(key):
			var tex: Texture2D = ArtData.texture_at(ArtData.frame_path(person.frames, facing, pose))
			var resolved_mask: Texture2D = _mask(person.frames, facing, pose)
			resources[key] = {"texture": tex, "mask": resolved_mask, "anchor": ArtData.anchor(key)}
		var frame: Dictionary = resources[key]
		person.body.texture = frame["texture"]
		var mask: Texture2D = frame["mask"]
		person.mat.set_shader_parameter("mask_tex", mask)
		person.mat.set_shader_parameter("has_mask", mask != null)
		person.body.set_meta("anchor", frame["anchor"])
	var anchor: Vector2i = person.body.get_meta("anchor", Vector2i(40, 104))
	person.body.flip_h = person.west and facing == "E"
	var w: int = person.body.texture.get_width() if person.body.texture != null else 80
	var ax: int = (w - anchor.x) if person.body.flip_h else anchor.x
	person.body.offset = Vector2(-ax, -anchor.y - bob)


func _process(delta: float) -> void:
	if _world_sim == null:
		return
	var t0: int = Time.get_ticks_usec()
	if _render_profiling:
		mux2_render_profile = {"frame": Engine.get_process_frames(), "people": _persons.size()}
		MarkerLayout.reset_profile()
	_update(delta)
	last_process_usec = Time.get_ticks_usec() - t0
	if _render_profiling:
		mux2_render_profile["update_total_usec"] = last_process_usec
		mux2_render_profile["badges"] = _coincident_badges.size()
		mux2_render_profile["layout_searches"] = MarkerLayout.profile_rows.duplicate(true)
		mux2_render_profile["layout_geometry"] = MarkerLayout.profile_geometry.duplicate(true)
		mux2_render_profile["profile_copy_usec"] = Time.get_ticks_usec() - t0 - last_process_usec


func _profile_span(label: String, started: int) -> int:
	if not _render_profiling:
		return 0
	var now: int = Time.get_ticks_usec()
	mux2_render_profile[label] = int(mux2_render_profile.get(label, 0)) + now - started
	return now


func _update(delta: float) -> void:
	var phase: int = Time.get_ticks_usec() if _render_profiling else 0
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_make_shadow()
		_selection_rgba = _rgba("people.selection_rgba", Color(0.94, 0.89, 0.67))
		_drafted_rgba = _rgba("people.drafted_rgba", Color(0.77, 0.2, 0.16))
		_outline_px = float(ArtData.param("people.outline_px", 1))
		_drafted_px = float(ArtData.param("people.drafted_outline_px", 2))
		_halo_rgba = _rgba("people.night_halo_rgba", Color(0.91, 0.89, 0.81))
		_halo_alpha = clampf(float(ArtData.param("people.night_halo_alpha", 0.0)), 0.0, 1.0)
		_half_rim = _rgba("people.half_rim_rgba", Color(0.95, 0.95, 0.88, 1))
		_half_rim_px = clampf(float(ArtData.param("people.half_rim_px", 1)), 0.0, 2.0)
		_half_overlap_alpha = clampf(float(ArtData.param("people.half_overlap_alpha", 0.8)), 0.5, 1.0)
		for person: Person in _persons.values():
			person.shadow.texture = _shadow_tex
			person.outline = Color(-1, -1, -1, -1)
			person.mat.set_shader_parameter("outline_px", _outline_px)
	_frame += 1
	_observe_settlements()
	var speed: float = _speed()
	_clock += delta * speed
	var snap: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var stages: PackedByteArray = snap.get("life_stages", PackedByteArray())
	var carries: PackedInt32Array = snap.get("carry_counts", PackedInt32Array())
	var states: PackedByteArray = snap.get("states", PackedByteArray())
	var targets: PackedByteArray = snap.get("target_kinds", PackedByteArray())
	var drafted: PackedByteArray = snap.get("drafted", PackedByteArray())
	var sick: PackedByteArray = snap.get("sick", PackedByteArray())
	var bands: PackedByteArray = snap.get("mood_bands", PackedByteArray())
	var fleeing: PackedByteArray = snap.get("fleeing", PackedByteArray())
	var looks: PackedInt32Array = snap.get("look_ids", PackedInt32Array())
	var variants: PackedByteArray = snap.get("variants", PackedByteArray())
	var n: int = mini(mini(bits.size(), ids.size()), mini(xs.size(), ys.size()))
	var duration: float = _step_seconds(speed)
	var walk_hz: float = float(ArtData.param("people.walk_hz", 4.0))
	var work_hz: float = float(ArtData.param("people.work_hz", 2.0))
	var bob_px: int = int(ArtData.param("people.bob_px", 1))
	var inspector: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
	_selected_bits = int(inspector.call("get_selected_entity_bits")) if inspector != null and (inspector as Control).visible else -1
	_zoom = _camera_zoom()
	var lod: bool = _zoom <= float(ArtData.param("lod.marker_zoom_max", 0.25)) + 0.001
	if lod:
		_update_households(snap)
	phase = _profile_span("params_snapshot_usec", phase)
	var phase_walk: bool = fmod(_clock * walk_hz, 1.0) >= 0.5
	var phase_work: bool = fmod(_clock * work_hz, 1.0) >= 0.5
	var readable: bool = _zoom < 1.0 and not lod
	var frame_resources: Dictionary = {}
	var any_sleep: bool = false
	for i: int in n:
		var key: int = bits[i]
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		var person: Person = _persons.get(key)
		if person == null:
			person = _make_person(key, ids[i], tile)
			_persons[key] = person
		elif tile != person.tile:
			person.from_pos = person.position
			person.to_pos = View.base_point(tile)
			if tile.x != person.tile.x:
				person.facing = "E"
				person.west = tile.x < person.tile.x
			else:
				person.facing = "S"
				person.west = false
			person.tile = tile
			person.elapsed = 0.0
			person.duration = duration
		person.seen = _frame
		var stage: int = int(stages[i]) if i < stages.size() else 0
		var look: int = looks[i] if i < looks.size() else posmod(ids[i], maxi(1, ArtData.look_count()))
		var variant: int = int(variants[i]) if i < variants.size() else posmod(ids[i], 6)
		_set_appearance(person, look, variant, stage == 2)
		var asleep: bool = i < states.size() and i < targets.size() and states[i] == STATE_CONSUMING and targets[i] == TARGET_SLEEP
		var moving: bool = person.elapsed < person.duration
		var carrying: bool = not asleep and i < carries.size() and carries[i] > 0
		var working: bool = not moving and i < states.size() and i < targets.size() and states[i] == STATE_CONSUMING and targets[i] == TARGET_JOB
		var pose: String
		var bob: int = 0
		if asleep:
			pose = "sleep"
			person.activity = "sleep"
			any_sleep = true
		elif carrying:
			pose = "carry"
			bob = bob_px if moving and phase_walk else 0
			person.activity = "carry"
		elif moving:
			pose = "walk_a" if phase_walk else "idle"
			bob = bob_px if phase_walk else 0
			person.activity = "walk"
		elif working:
			pose = "work_a" if phase_work else "idle"
			person.activity = "work"
		else:
			pose = "idle"
			person.activity = "idle"
		person.drafted = i < drafted.size() and drafted[i] != 0
		person.selected = key == _selected_bits
		var sick_now: bool = i < sick.size() and sick[i] != 0
		var flee_now: bool = not asleep and i < fleeing.size() and fleeing[i] != 0
		var marks_changed: bool = sick_now != person.sick or flee_now != person.fleeing or person.marks.visible == lod
		person.sick = sick_now
		person.fleeing = flee_now
		if person.fleeing:
			person.activity = "flee"
		person.modulate = StoryText.band_tint(int(bands[i]) if i < bands.size() else 0) if _mood_shown else Color.WHITE
		if person.visible == lod:
			person.visible = not lod
			person.body.visible = not lod
			person.marks.visible = not lod
		person.shadow.visible = not asleep and not lod
		if marks_changed:
			person.marks.queue_redraw()
		if lod:
			# Markers only (drawn by this node): no frame, mask or shader work for a hidden body.
			continue
		_set_frame_cached(person, pose, bob, frame_resources)
		if person.readable != readable:
			person.readable = readable
			person.mat.shader = _half_shader if readable else VARIANT_SHADER
			person.body.z_as_relative = not readable
			person.body.z_index = 4094 if readable else 0
			person.body.self_modulate = Color.WHITE
			person.outline = Color(-1, -1, -1, -1)
		var outline: Color = _no_outline
		if person.selected:
			outline = _selection_rgba
		elif person.drafted:
			outline = _drafted_rgba
		elif readable:
			outline = _half_rim
		elif _halo_alpha > 0.0 and _daylight != null:
			var a: float = roundf(_halo_alpha * float(_daylight.call("dark_at", person.position)) * float(HALO_STEPS)) / float(HALO_STEPS)
			if a > 0.0:
				outline = Color(_halo_rgba, a)
		if outline != person.outline:
			person.outline = outline
			person.mat.set_shader_parameter("outline_color", outline)
			# The drafted mark is wider (`people.drafted_outline_px`): a 1-px red line is lost on dark skin and fur at 1×.
			var width: float = _drafted_px if outline == _drafted_rgba else _outline_px
			if readable:
				width = maxf(width, _half_rim_px / maxf(_zoom, 0.01))
			person.mat.set_shader_parameter("outline_px", width)
	phase = _profile_span("person_reconcile_usec", phase)
	for key: int in _persons.keys():
		var person: Person = _persons[key]
		if person.seen != _frame:
			person.queue_free()
			_persons.erase(key)
	_place_sleepers(any_sleep)
	for person: Person in _persons.values():
		if person.activity == "sleep":
			person.position = person.to_pos + person.bed
		elif person.elapsed < person.duration:
			person.elapsed = minf(person.duration, person.elapsed + delta)
			person.position = person.from_pos.lerp(person.to_pos, person.elapsed / person.duration).round()
		else:
			person.position = person.to_pos
	phase = _profile_span("cleanup_placement_usec", phase)
	if not lod and _zoom < 1.0:
		_show_overlapping_figures()
	elif not _coincident_badges.is_empty():
		_clear_coincident_badges()
		queue_redraw()
	if lod or _lod:
		_lod = lod
		_refresh_markers()
	_lod = lod
	_profile_span("half_or_lod_total_usec", phase)


## At half zoom, let genuine coincident art show through; bodies keep their original feet and shadows.
## Cache opaque frame extents and use tile-size spatial buckets, avoiding all-pairs work at population1000.
func _show_overlapping_figures() -> void:
	var started: int = Time.get_ticks_usec() if _render_profiling else 0
	var rectangles: Dictionary = {}
	for person: Person in _persons.values():
		var texture: Texture2D = person.body.texture
		if texture == null:
			continue
		var key: int = texture.get_instance_id()
		if not _opaque_rects.has(key):
			var image: Image = texture.get_image()
			if image.is_compressed():
				image.decompress()
			_opaque_rects[key] = image.get_used_rect()
		var used: Rect2i = _opaque_rects[key]
		if person.body.flip_h:
			used.position.x = texture.get_width() - used.end.x
		var rectangle: Rect2 = Rect2(person.position + person.body.offset + Vector2(used.position), Vector2(used.size))
		rectangles[person.bits] = rectangle
	var overlaps: Dictionary = MarkerLayout.overlapping_people(rectangles, float(View.TILE))
	for person: Person in _persons.values():
		person.body.self_modulate = Color(1, 1, 1, _half_overlap_alpha if overlaps.has(person.bits) else 1.0)
	started = _profile_span("overlap_alpha_usec", started)
	_update_coincident_badges(rectangles)
	_profile_span("badges_total_usec", started)


## Counts come from the full person snapshot, never body visibility. Counterfactual body hides cannot alter a badge.
func _clear_coincident_badges() -> void:
	_coincident_badges.clear()
	_coincident_groups.clear()
	badge_layout_exhausted = 0
	if _badge_leaders != null:
		_badge_leaders.segments.clear()
		_badge_leaders.aggregate_segments.clear()
		_badge_leaders.active_segments.clear()
		_badge_leaders.queue_redraw()


func _update_coincident_badges(rectangles: Dictionary) -> void:
	var phase: int = Time.get_ticks_usec() if _render_profiling else 0
	var groups: Dictionary = {}
	for person: Person in _persons.values():
		var cell: Vector2i = Vector2i((person.position * _zoom).round())
		if not groups.has(cell):
			groups[cell] = []
		groups[cell].append(person.bits)
	var has_group: bool = false
	for group: Array in groups.values():
		has_group = has_group or group.size() > 1
	phase = _profile_span("badge_grouping_usec", phase)
	if not has_group:
		if not _coincident_badges.is_empty():
			_clear_coincident_badges()
			queue_redraw()
		return
	var badges: Array[Dictionary] = []
	var canvas: Transform2D = get_global_transform_with_canvas()
	var inverse: Transform2D = canvas.affine_inverse()
	var blocked: Array[Rect2] = []
	for rectangle: Rect2 in rectangles.values():
		blocked.append(canvas * rectangle)
	var ui_rectangles: Array[Rect2] = []
	var ui: Node = get_node_or_null("/root/Main/UI")
	if ui != null:
		MarkerLayout.ui_rects(ui, ui_rectangles)
	blocked.append_array(ui_rectangles)
	var unique_blocked: Array[Rect2] = []
	var seen_blocked: Dictionary = {}
	for rectangle: Rect2 in blocked:
		if not seen_blocked.has(rectangle):
			seen_blocked[rectangle] = true
			unique_blocked.append(rectangle)
	blocked = unique_blocked
	var badge_index: Dictionary = MarkerLayout.badge_index(blocked)
	phase = _profile_span("badge_obstacles_usec", phase)
	var descriptors: Array[Dictionary] = []
	var agent_ids: Dictionary = {}
	var maximum_size: Vector2 = Vector2.ONE
	var font_px: int = int(ArtData.param("labels.font_px", 24))
	var label_pad: float = floorf(float(font_px) / 4.0)
	var label_inverse: float = 1.0 / maxf(_zoom, 0.01) if bool(ArtData.param("labels.follow_camera", true)) else 1.0
	var text_sizes: Dictionary = {}
	for group: Array in groups.values():
		if group.size() < 2:
			continue
		var members: Array[int] = []
		for bits: int in group:
			members.append(bits)
			agent_ids[bits] = (_persons[bits] as Person).agent_id
		members.sort_custom(func(a: int, b: int) -> bool: return (_persons[a] as Person).agent_id < (_persons[b] as Person).agent_id)
		var bounds: Rect2 = rectangles[members[0]] if rectangles.has(members[0]) else drawn_rect(members[0])
		for bits: int in members:
			bounds = bounds.merge(rectangles[bits] if rectangles.has(bits) else drawn_rect(bits))
		var text: String = str(members.size())
		descriptors.append({"key": int(agent_ids[members[0]]), "members": members, "text": text, "size": _badge_screen_size(text, canvas, font_px, label_pad, label_inverse, text_sizes), "figure": canvas * bounds})
	descriptors.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["key"]) < int(b["key"]))
	_coincident_groups = descriptors.duplicate()
	phase = _profile_span("badge_prepare_usec", phase)
	descriptors = MarkerLayout.aggregate_badges(descriptors, agent_ids)
	var extent_style: Dictionary = _aggregate_style()
	var extent_counters: Dictionary = {"source_rectangles": 0} if _render_profiling else {}
	var extent_started: int = Time.get_ticks_usec() if _render_profiling else 0
	for descriptor: Dictionary in descriptors:
		descriptor["text"] = _aggregate_text(descriptor)
		descriptor["scope"] = MarkerLayout.aggregate_contours(descriptor["source_figures"], float(extent_style["padding"]), extent_counters) if (descriptor["source_groups"] as Array).size() > 1 else PackedVector2Array()
		descriptor["size"] = _badge_screen_size(str(descriptor["text"]), canvas, font_px, label_pad, label_inverse, text_sizes)
		maximum_size = maximum_size.max(descriptor["size"])
	if _render_profiling:
		extent_counters["build_usec"] = Time.get_ticks_usec() - extent_started
		extent_counters["cache_reuses"] = 0
		mux2_render_profile["badge_source_groups"] = _coincident_groups.size()
		mux2_render_profile["badge_aggregate_groups"] = descriptors.size()
		mux2_render_profile["badge_source_members"] = agent_ids.size()
		var aggregate_members: int = 0
		var membership: Array[Dictionary] = []
		for descriptor: Dictionary in descriptors:
			aggregate_members += (descriptor["members"] as Array).size()
			membership.append({"members": descriptor["members"], "source_groups": descriptor["source_groups"]})
		mux2_render_profile["badge_aggregate_members"] = aggregate_members
		mux2_render_profile["badge_memberships"] = membership
	phase = _profile_span("badge_aggregate_usec", phase)
	var grid: Dictionary = MarkerLayout.badge_grid(maximum_size, get_viewport_rect(), blocked)
	phase = _profile_span("badge_grid_usec", phase)
	for descriptor: Dictionary in descriptors:
		var figure: Rect2 = descriptor["figure"]
		var exhausted_before: int = int(grid["exhausted"])
		var placed: Rect2 = MarkerLayout.badge_packed(figure, descriptor["size"], get_viewport_rect(), badge_index, grid)
		MarkerLayout.badge_index_add(badge_index, placed)
		var aggregate: bool = (descriptor["source_groups"] as Array).size() > 1
		var leader: PackedVector2Array = MarkerLayout.aggregate_leader(descriptor["scope"], placed) if aggregate else MarkerLayout.badge_leader(figure, placed)
		if leader.is_empty():
			leader = PackedVector2Array([placed.get_center(), placed.get_center()])
			if _render_profiling:
				mux2_render_profile["empty_aggregate_scopes"] = int(mux2_render_profile.get("empty_aggregate_scopes", 0)) + 1
		badges.append({"members": descriptor["members"], "member_count": (descriptor["members"] as Array).size(), "aggregate": aggregate, "screen_scope": descriptor["scope"], "source_groups": descriptor["source_groups"], "anchor": inverse * Vector2(placed.get_center().x, placed.end.y), "text": descriptor["text"], "rect": inverse * placed, "leader_from": inverse * leader[0], "leader_to": inverse * leader[1], "screen_leader": leader, "exhausted": int(grid["exhausted"]) != exhausted_before})
	phase = _profile_span("badge_layout_usec", phase)
	var segments: PackedVector2Array = PackedVector2Array()
	var aggregate_segments: PackedVector2Array = PackedVector2Array()
	var active_segments: PackedVector2Array = PackedVector2Array()
	var half_stroke: float = maxf(float(extent_style["dark_width"]), float(extent_style["light_width"])) * 0.5
	var has_aggregate: bool = false
	for badge: Dictionary in badges:
		has_aggregate = has_aggregate or bool(badge["aggregate"])
	var common_rectangles: Array[Rect2] = []
	for rectangle: Rect2 in ui_rectangles:
		common_rectangles.append(rectangle.grow(float(extent_style["ui_clearance"]) + half_stroke))
	for badge: Dictionary in badges:
		common_rectangles.append((canvas * (badge["rect"] as Rect2)).grow(float(extent_style["ui_clearance"]) + half_stroke))
	var common_index: Dictionary = MarkerLayout.badge_index(common_rectangles)
	var body_obstacles: Dictionary = MarkerLayout.aggregate_body_obstacles(rectangles, canvas, float(extent_style["body_clearance"]) + half_stroke) if has_aggregate else {}
	var mouse: Vector2 = get_local_mouse_position()
	var mouse_ui: bool = false
	for rectangle: Rect2 in ui_rectangles:
		mouse_ui = mouse_ui or rectangle.has_point(canvas * mouse)
	for badge: Dictionary in badges:
		var leader: PackedVector2Array = badge["screen_leader"]
		var clipped: PackedVector2Array = MarkerLayout.clipped_leader(leader[0], leader[1], badge_index)
		if bool(badge["aggregate"]):
			clipped = MarkerLayout.clipped_aggregate(clipped, get_viewport_rect().grow(-half_stroke), common_index, body_obstacles, badge["members"], extent_counters)
			clipped.append_array(MarkerLayout.clipped_aggregate(badge["screen_scope"], get_viewport_rect().grow(-half_stroke), common_index, body_obstacles, badge["members"], extent_counters))
			badge["active"] = (badge["members"] as Array).has(_selected_bits) or (not mouse_ui and (badge["rect"] as Rect2).has_point(mouse))
			for point: Vector2 in clipped:
				if bool(badge["active"]):
					active_segments.append(inverse * point)
				else:
					aggregate_segments.append(inverse * point)
		else:
			for point: Vector2 in clipped:
				segments.append(inverse * point)
	if _render_profiling:
		mux2_render_profile["aggregate_extent"] = extent_counters
	badge_layout_exhausted = int(grid["exhausted"])
	if _render_profiling:
		mux2_render_profile["badge_layout_exhausted"] = badge_layout_exhausted
	if _badge_leaders != null:
		var width: float = 1.0 / maxf(_zoom, 0.01)
		if _badge_leaders.segments != segments or _badge_leaders.light != _half_rim or _badge_leaders.scale_width != width or _badge_leaders.aggregate_segments != aggregate_segments or _badge_leaders.active_segments != active_segments or _badge_leaders.extent_style != extent_style:
			_badge_leaders.segments = segments
			_badge_leaders.aggregate_segments = aggregate_segments
			_badge_leaders.active_segments = active_segments
			_badge_leaders.extent_style = extent_style
			_badge_leaders.light = _half_rim
			_badge_leaders.scale_width = width
			_badge_leaders.queue_redraw()
			queue_redraw()
	if badges != _coincident_badges:
		_coincident_badges = badges
		queue_redraw()
	_profile_span("badge_leader_clip_usec", phase)


## Presentation only: each source is an exact-foot group; every member remains counted.
func _aggregate_text(descriptor: Dictionary) -> String:
	var count: int = (descriptor["members"] as Array).size()
	var groups: int = (descriptor["source_groups"] as Array).size()
	if groups <= 1:
		return str(count)
	var locale: Node = get_node_or_null("/root/Locale")
	return str(locale.call("trf", "UI_COINCIDENT_AGGREGATE_FMT", {"groups": groups, "people": count})) if locale != null else str(count)


## Re-read every invocation, so F12, UI font and camera changes need no geometry cache invalidation.
func _aggregate_style() -> Dictionary:
	return {"padding": maxf(0.0, float(ArtData.param("labels.aggregate.padding_px", 2.0))),
		"dark_width": maxf(0.0, float(ArtData.param("labels.aggregate.dark_width_px", 3.0))),
		"light_width": maxf(0.0, float(ArtData.param("labels.aggregate.light_width_px", 1.0))),
		"dark": _rgba("labels.aggregate.dark_rgba", Color8(35, 29, 23, 210)),
		"light": _rgba("labels.aggregate.light_rgba", Color8(184, 216, 224, 220)),
		"active_light": _rgba("labels.aggregate.active_light_rgba", Color8(220, 244, 250, 255)),
		"ui_clearance": maxf(0.0, float(ArtData.param("labels.aggregate.ui_clearance_px", 2.0))),
		"body_clearance": maxf(0.0, float(ArtData.param("labels.aggregate.body_clearance_px", 1.0)))}


func _badge_screen_size(text: String, canvas: Transform2D, px: int, pad: float, inv: float, sizes: Dictionary) -> Vector2:
	if sizes.has(text):
		return sizes[text]
	var size: Vector2 = _font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, px) if _font != null else Vector2(px, px)
	var screen_size: Vector2 = (canvas * Rect2(Vector2.ZERO, Vector2(size.x + pad * 2.0, px + pad) * inv)).size
	sizes[text] = screen_size
	return screen_size


func _rgba(path: String, fallback: Color) -> Color:
	var v: Variant = ArtData.param(path, null)
	if v is Array and (v as Array).size() == 4:
		return Color8(int(v[0]), int(v[1]), int(v[2]), int(v[3]))
	return fallback


func _place_sleepers(any_sleep: bool) -> void:
	var by_tile: Dictionary = {}
	for person: Person in _persons.values():
		person.bed = Vector2.ZERO
		if any_sleep and person.activity == "sleep":
			if not by_tile.has(person.tile):
				by_tile[person.tile] = []
			(by_tile[person.tile] as Array).append(person)
	for tile: Vector2i in by_tile.keys():
		var group: Array = by_tile[tile]
		if group.size() < 2:
			continue
		group.sort_custom(func(a: Person, b: Person) -> bool: return a.agent_id < b.agent_id)
		for k: int in group.size():
			(group[k] as Person).bed = FAN[k % FAN.size()]


## Badge (sick) and "!" (fleeing) above the head, in the person's own coordinates (feet at 0, 0).
func _draw_marks(person: Person) -> void:
	var head: float = -float(person.body.get_meta("anchor", Vector2i(40, 104)).y) + 40.0
	if person.activity == "sleep":
		head = -40.0
	if person.sick:
		person.marks.draw_circle(Vector2(10, head), 4.0, INK)
		person.marks.draw_circle(Vector2(10, head), 3.0, BADGE)
	if person.fleeing:
		person.marks.draw_rect(Rect2(-3, head - 14, 7, 15), Color(INK, 0.95))
		person.marks.draw_rect(Rect2(-1, head - 12, 3, 8), FLEE_MARK)
		person.marks.draw_rect(Rect2(-1, head - 2, 3, 2), FLEE_MARK)


## LOD markers (at or below `lod.marker_zoom_max`): batched discs and glyphs; this node
## draws only the selection and drafted rings around their discs.
func _refresh_markers() -> void:
	var t0: int = Time.get_ticks_usec()
	if not _lod:
		_markers.hide_markers()
		queue_redraw()
		last_draw_usec = Time.get_ticks_usec() - t0
		return
	var px: int = int(ArtData.param("lod.marker_fill_px", 14))
	var glyphs: bool = absf(_zoom - float(ArtData.param("lod.glyph_zoom", 0.25))) < 0.01
	var points: PackedVector2Array = PackedVector2Array()
	var placement_points: PackedVector2Array = PackedVector2Array()
	var colours: PackedColorArray = PackedColorArray()
	var rings: PackedColorArray = PackedColorArray()
	var ids: PackedInt32Array = PackedInt32Array()
	var stable_ids: PackedInt64Array = PackedInt64Array()
	var palette: Array = ArtData.param("lod.household_ring_rgb", [])
	points.resize(_persons.size())
	placement_points.resize(_persons.size())
	colours.resize(_persons.size())
	rings.resize(_persons.size())
	ids.resize(_persons.size())
	stable_ids.resize(_persons.size())
	var k: int = 0
	_ringed.clear()
	for person: Person in _persons.values():
		points[k] = person.position
		placement_points[k] = person.to_pos + person.bed if person.activity == "sleep" else person.to_pos
		colours[k] = person.top_colour
		rings[k] = _ring_colour(int(_household_of.get(person.agent_id, -1)), palette)
		ids[k] = GLYPH_OF.get(person.activity, -1)
		stable_ids[k] = person.agent_id
		if person.selected or person.drafted:
			_ringed.append(person)
		k += 1
	_markers.show_markers(points, colours, ids, px, _zoom, glyphs, _rgba("lod.ring_rgba", INK), rings, ArtData.param("lod", {}), stable_ids, placement_points)
	k = 0
	for person: Person in _persons.values():
		person.marker_position = _markers.point_at(k)
		k += 1
	_marker_px = _markers.total_px()
	queue_redraw()
	last_draw_usec = Time.get_ticks_usec() - t0


## The ring colour of household key `key` from `palette` (RGB 0–255 rows); alpha 0 (the ink ring) without either.
static func _ring_colour(key: int, palette: Array) -> Color:
	if key < 0 or palette.is_empty():
		return Color(0, 0, 0, 0)
	var row: Variant = palette[View.key_hash(key, 0, 0, HOUSEHOLD_SALT) % palette.size()]
	if not (row is Array and (row as Array).size() == 3):
		return Color(0, 0, 0, 0)
	return Color8(int(row[0]), int(row[1]), int(row[2]))


## The household key of every person in a snapshot (AgentId → key; see the header). Pure (tests).
static func household_keys(ids: PackedInt64Array, stages: PackedByteArray, mothers: PackedInt64Array, fathers: PackedInt64Array, partners: PackedInt64Array) -> Dictionary:
	var alive: Dictionary = {}
	for id: int in ids:
		alive[id] = true
	var adult: Dictionary = {}
	for i: int in ids.size():
		var p: int = partners[i] if i < partners.size() else -1
		adult[ids[i]] = mini(ids[i], p) if p >= 0 and alive.has(p) else ids[i]
	var out: Dictionary = {}
	for i: int in ids.size():
		var key: int = adult[ids[i]]
		if i < stages.size() and stages[i] == LIFE_CHILD:
			var m: int = mothers[i] if i < mothers.size() else -1
			var f: int = fathers[i] if i < fathers.size() else -1
			key = adult[m] if alive.has(m) else (adult[f] if alive.has(f) else ids[i])
		out[ids[i]] = key
	return out


func _update_households(snap: Dictionary) -> void:
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var mothers: PackedInt64Array = snap.get("mothers", PackedInt64Array())
	var fathers: PackedInt64Array = snap.get("fathers", PackedInt64Array())
	var partners: PackedInt64Array = snap.get("partners", PackedInt64Array())
	var key: int = hash([ids, mothers, fathers, partners])
	if key == _household_key:
		return
	_household_key = key
	if mothers.size() != ids.size() or partners.size() != ids.size():
		_household_of = {}
		return
	_household_of = household_keys(ids, snap.get("life_stages", PackedByteArray()), mothers, fathers, partners)


## The household key of a person (−1 without one; tests).
func household_of(agent_id: int) -> int:
	return int(_household_of.get(agent_id, -1))


func _draw() -> void:
	if not _lod:
		if _zoom < 1.0 and _font != null:
			for badge: Dictionary in _coincident_badges:
				OverlayLabel.draw_label(self, _font, str(badge["text"]), badge["anchor"], _zoom)
				if bool(badge.get("aggregate", false)):
					var style: Dictionary = _badge_leaders.extent_style
					var width: float = float(style["light_width"]) / maxf(_zoom, 0.01)
					draw_rect((badge["rect"] as Rect2).grow(-width * 0.5), style["active_light"] if bool(badge.get("active", false)) else style["light"], false, width)
		return
	var onepx: float = 1.0 / maxf(_zoom, 0.01)
	var r: float = _marker_px * 0.5 * onepx
	for person: Person in _ringed:
		if person.drafted:
			draw_arc(person.marker_position, r + onepx, 0.0, TAU, 32, _drafted_rgba, 2.0 * onepx)
		if person.selected:
			draw_arc(person.marker_position, r + (3.0 if person.drafted else 1.0) * onepx, 0.0, TAU, 32, _selection_rgba, 2.0 * onepx)


func _camera_zoom() -> float:
	var camera: Camera2D = get_viewport().get_camera_2d()
	return camera.zoom.x if camera != null else 1.0


func _speed() -> float:
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	return float(camera.call("get_speed")) if camera != null and camera.has_method("get_speed") else 1.0


## Seconds one step lasts at the current speed: move_interval_ticks / (12 × speed).
func _step_seconds(speed: float) -> float:
	var info: Dictionary = SimReads.read(_world_sim, "get_world_info")
	var interval: float = float(info.get("move_interval_ticks", 3))
	return maxf(0.02, interval / (12.0 * maxf(1.0, speed)))


func _observe_settlements() -> void:
	var society: Dictionary = SimReads.read(_world_sim, "get_society_snapshot")
	var settle_snap: Variant = SimReads.read(_world_sim, "get_settlement_snapshot")
	var settlements: Dictionary = settle_snap.duplicate() if settle_snap is Dictionary else {}
	if not settlements.has("settlement_ids"):
		settlements["settlement_ids"] = PackedInt32Array(Array(settlements.get("ids", PackedInt64Array())))
	SettlementSelection.observe(society, settlements)


## The drawn rect of a person in world coordinates (what a click hits).
func drawn_rect(bits: int) -> Rect2:
	var person: Person = _persons.get(bits)
	if person == null:
		return Rect2()
	if _lod:
		var mark_px: float = 4.0 if person.selected and person.drafted else (2.0 if person.selected or person.drafted else 0.0)
		var r: float = (_marker_px * 0.5 + mark_px) / maxf(_zoom, 0.01)
		return Rect2(person.marker_position - Vector2(r, r), Vector2(2 * r, 2 * r))
	if person.activity == "sleep":
		return Rect2(person.position + Vector2(-28, -36), Vector2(56, 40))
	return Rect2(person.position + Vector2(-20, -60), Vector2(40, 64))


## Default picking retains lowest-on-screen priority. Explicit inspection cycles exact coincident members by stable ID.
func entity_at(p: Vector2, cycle_selection: bool = false) -> int:
	if cycle_selection and not _lod and _zoom < 1.0:
		for badge: Dictionary in _coincident_badges:
			if (badge["rect"] as Rect2).has_point(p):
				var members: Array[int] = badge["members"]
				return members[(members.find(_selected_bits) + 1) % members.size()]
		for group: Dictionary in _coincident_groups:
			var members: Array[int] = group["members"]
			for bits: int in members:
				if drawn_rect(bits).has_point(p):
					return members[(members.find(_selected_bits) + 1) % members.size()]
	var best: int = -1
	var best_y: float = -INF
	for person: Person in _persons.values():
		if drawn_rect(person.bits).has_point(p) and person.position.y > best_y:
			best = person.bits
			best_y = person.position.y
	return best


## The drawn top-left of the person's tile cell (feet minus View.BASE) for tools, or (-1, -1).
func drawn_position(bits: int) -> Vector2:
	var person: Person = _persons.get(bits)
	return person.position - Vector2(View.BASE) if person != null else Vector2(-1, -1)


## The pose, facing, look, variant and marks of the person with entity bits `bits` (for tools), or {}.
func person_look(bits: int) -> Dictionary:
	var person: Person = _persons.get(bits)
	if person == null:
		return {}
	return {"frame": person.pose, "pose": person.pose, "activity": person.activity, "west": person.west,
		"look": person.look, "frames": person.frames, "variant": person.variant, "elder": person.elder,
		"carry": person.activity == "carry", "drafted": person.drafted, "sick": person.sick,
		"feet": person.position, "anchor": person.body.get_meta("anchor", Vector2i.ZERO), "offset": person.body.offset,
		"flip": person.body.flip_h, "lod": _lod}


## M5: show or hide the mood overlay (M).
func set_mood_shown(on: bool) -> void:
	if on == _mood_shown:
		return
	_mood_shown = on
	mood_toggled.emit(on)


func mood_shown() -> bool:
	return _mood_shown


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == MOOD_KEY:
		set_mood_shown(not _mood_shown)
		get_viewport().set_input_as_handled()


## M5: the drafted outline, sick badge, M5.5 fleeing "!" and tint of the person with entity bits `bits` (for tools), or {}.
func person_marks(bits: int) -> Dictionary:
	var person: Person = _persons.get(bits)
	if person == null:
		return {}
	return {"drafted": person.drafted, "sick": person.sick, "fleeing": person.fleeing, "tint": person.modulate}


## How many people are drawn.
func person_count() -> int:
	return _persons.size()


## Whether people are drawn as LOD markers now.
func lod_active() -> bool:
	return _lod


## The tint shader parameters of a person: [skin_delta, hair_delta] (tests: a pure function of look and variant).
func tint_params(bits: int) -> Array:
	var person: Person = _persons.get(bits)
	if person == null:
		return []
	return [person.mat.get_shader_parameter("skin_delta"), person.mat.get_shader_parameter("hair_delta")]


## Turn a person to `facing` ("S", "E", "W", "N") and redraw its idle frame (tests and tools).
func face_for_test(bits: int, facing: String) -> void:
	var person: Person = _persons.get(bits)
	if person == null:
		return
	person.facing = "E" if facing == "E" or facing == "W" else "S"
	person.west = facing == "W"
	person.pose = ""
	_set_frame(person, "idle", 0)
