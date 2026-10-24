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
# LOD: below camera zoom `lod.people_below_zoom` bodies hide and this node draws each person as a filled circle of
#   `lod.marker_px` SCREEN pixels in its top's colour (people/top_colours.json) with a 1-px ink ring, plus at
#   `lod.glyph_zoom` an activity glyph (axe working, log carrying, zz sleeping, ! fleeing).
# Marks: a sick person shows a small green badge, a fleeing one a "!" above the head (code-drawn); M toggles the mood
#   overlay (each person tinted by `mood_bands`, StoryText.band_tint).
# The drawn rect is what clicks hit (`entity_at`); positions tween between tiles over one move step.
# Also feeds SettlementSelection.observe every frame (the inspector's village row; V8-0 DECISIONS D31).
# Bridge methods: via SimReads get_agent_snapshot, get_world_info, get_society_snapshot, get_settlement_snapshot;
#   signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Tone = preload("res://scripts/world/art/tone.gd")
const SettlementSelection = preload("res://scripts/core/settlement_selection.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const VARIANT_SHADER: Shader = preload("res://shaders/person_variant.gdshader")
## Snapshot state codes (bridge contract, not content): consuming; target kinds sleep and job.
const STATE_CONSUMING: int = 3
const TARGET_SLEEP: int = 3
const TARGET_JOB: int = 9
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
	var shadow: Sprite2D
	var body: Sprite2D
	var mat: ShaderMaterial
	var marks: Node2D


func _ready() -> void:
	z_index = 6
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
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
	var facing: String = "E" if person.facing == "E" else "S"
	if person.pose != "%s/%s" % [facing, pose]:
		person.pose = "%s/%s" % [facing, pose]
		var tex: Texture2D = ArtData.texture_at(ArtData.frame_path(person.frames, facing, pose))
		person.body.texture = tex
		var mask: Texture2D = _mask(person.frames, facing, pose)
		person.mat.set_shader_parameter("mask_tex", mask)
		person.mat.set_shader_parameter("has_mask", mask != null)
		var a: Vector2i = ArtData.anchor("%s/%s/%s" % [person.frames, facing, pose])
		person.body.set_meta("anchor", a)
	var anchor: Vector2i = person.body.get_meta("anchor", Vector2i(40, 104))
	person.body.flip_h = person.west and facing == "E"
	var w: int = person.body.texture.get_width() if person.body.texture != null else 80
	var ax: int = (w - anchor.x) if person.body.flip_h else anchor.x
	person.body.offset = Vector2(-ax, -anchor.y - bob)


func _process(delta: float) -> void:
	if _world_sim == null:
		return
	var t0: int = Time.get_ticks_usec()
	_update(delta)
	last_process_usec = Time.get_ticks_usec() - t0


func _update(delta: float) -> void:
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_make_shadow()
		_selection_rgba = _rgba("people.selection_rgba", Color(0.94, 0.89, 0.67))
		_drafted_rgba = _rgba("people.drafted_rgba", Color(0.77, 0.2, 0.16))
		_outline_px = float(ArtData.param("people.outline_px", 1))
		_drafted_px = float(ArtData.param("people.drafted_outline_px", 2))
		_halo_rgba = _rgba("people.night_halo_rgba", Color(0.91, 0.89, 0.81))
		_halo_alpha = clampf(float(ArtData.param("people.night_halo_alpha", 0.0)), 0.0, 1.0)
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
	var lod: bool = _zoom < float(ArtData.param("lod.people_below_zoom", 1.0)) - 0.001
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
		var phase_walk: bool = fmod(_clock * walk_hz, 1.0) >= 0.5
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
			pose = "work_a" if fmod(_clock * work_hz, 1.0) >= 0.5 else "idle"
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
		_set_frame(person, pose, bob)
		var outline: Color = _no_outline
		if person.selected:
			outline = _selection_rgba
		elif person.drafted:
			outline = _drafted_rgba
		elif _halo_alpha > 0.0 and _daylight != null:
			var a: float = roundf(_halo_alpha * float(_daylight.call("dark_at", person.position)) * float(HALO_STEPS)) / float(HALO_STEPS)
			if a > 0.0:
				outline = Color(_halo_rgba, a)
		if outline != person.outline:
			person.outline = outline
			person.mat.set_shader_parameter("outline_color", outline)
			# The drafted mark is wider (`people.drafted_outline_px`): a 1-px red line is lost on dark skin and fur at 1×.
			person.mat.set_shader_parameter("outline_px", _drafted_px if outline == _drafted_rgba else _outline_px)
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
	if lod or _lod:
		_lod = lod
		_refresh_markers()
	_lod = lod


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


## LOD markers (below `lod.people_below_zoom`): discs and glyphs by the MultiMesh child (lod_markers.gd); this node
## draws only the selection and drafted rings around their discs.
func _refresh_markers() -> void:
	var t0: int = Time.get_ticks_usec()
	if not _lod:
		_markers.hide_markers()
		queue_redraw()
		last_draw_usec = Time.get_ticks_usec() - t0
		return
	var sizes: Dictionary = ArtData.param("lod.marker_px", {})
	var key: String = "0.25" if _zoom < 0.375 else "0.5"
	var px: int = int(sizes.get(key, 10 if key == "0.5" else 6))
	var glyphs: bool = absf(_zoom - float(ArtData.param("lod.glyph_zoom", 0.5))) < 0.01
	var points: PackedVector2Array = PackedVector2Array()
	var colours: PackedColorArray = PackedColorArray()
	var ids: PackedInt32Array = PackedInt32Array()
	points.resize(_persons.size())
	colours.resize(_persons.size())
	ids.resize(_persons.size())
	var k: int = 0
	_ringed.clear()
	for person: Person in _persons.values():
		points[k] = person.position
		colours[k] = person.top_colour
		ids[k] = GLYPH_OF.get(person.activity, -1)
		if person.selected or person.drafted:
			_ringed.append(person)
		k += 1
	_markers.show_markers(points, colours, ids, px, _zoom, glyphs, _rgba("lod.ring_rgba", INK))
	_marker_px = px
	queue_redraw()
	last_draw_usec = Time.get_ticks_usec() - t0


func _draw() -> void:
	if not _lod:
		return
	var onepx: float = 1.0 / maxf(_zoom, 0.01)
	var r: float = _marker_px * 0.5 * onepx
	for person: Person in _ringed:
		draw_arc(person.position, r + onepx, 0.0, TAU, 16, _selection_rgba if person.selected else _drafted_rgba, 1.5 * onepx)


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
		var r: float = 6.0 / maxf(_zoom, 0.01)
		return Rect2(person.position - Vector2(r, r), Vector2(2 * r, 2 * r))
	if person.activity == "sleep":
		return Rect2(person.position + Vector2(-28, -36), Vector2(56, 40))
	return Rect2(person.position + Vector2(-20, -60), Vector2(40, 64))


## Entity bits of the person drawn at world position `p` (the lowest on screen wins), or -1.
func entity_at(p: Vector2) -> int:
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
