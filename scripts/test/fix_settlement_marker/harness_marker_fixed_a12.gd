## A12 — end-to-end behavioral verification for
## fix-settlement-marker-fixed-position.
##
## Every Rust-side assertion (A1–A7) verifies a PROXY (the snapshot
## `formation_x/y` field). A8–A11/A13–A15 verify only that the GDScript SOURCE
## CONTAINS a string. So breaking the GDScript half (reading `formation_xs`
## into a variable but still positioning the sprite from `centroid_xs`) would
## pass the ENTIRE Rust+static suite while the on-screen marker keeps drifting —
## the exact user-visible bug. A12 closes that hole by booting the REAL
## production scene (`main.tscn`) and asserting the actually-rendered settlement
## gathering-marker NODE's world transform is FIXED across time while a
## settlement member demonstrably moves.
##
## The CLAUDE.md VLM limitation (sprite sub-resolution) means a screenshot diff
## cannot resolve this — hence a programmatic node-transform read of
## `WorldRenderer._furniture_sprites[<settlement_id>].global_position`.
##
## Mechanism: settlement formation needs a few hundred sim ticks. Headless
## `process_frame` delta is sub-ms, so to advance the WorldSimNode Gaffer
## accumulator quickly and deterministically we set a large `Engine.time_scale`
## plus `set_sim_speed(4.0)`; combined with the MAX_ITERS_PER_FRAME=5 cap this
## advances ~5 sim ticks/frame, so a settlement forms within a bounded frame
## budget. The "+200 ticks" interval is satisfied loosely — what matters
## behaviorally is that a member MOVES over the interval, which A12 asserts
## DIRECTLY from the agent snapshot (independent of the exact tick count).
##
## Outcome is one of:
##   A12 PASS       — marker(s) present in both readings, ≥1 member moved,
##                    every marker world position IDENTICAL across the interval.
##   A12 FAIL       — a marker drifted, OR the vacuity guards tripped
##                    (no marker in both readings / no member moved).
##   A12 UNVERIFIED — the harness genuinely could not introspect the marker
##                    node transform (e.g. `_furniture_sprites` inaccessible or
##                    no settlement formed within the frame budget). Reported
##                    explicitly, never silently passed; escalate to windowed
##                    Visual Verify.
##
## Usage:
##   godot --path . --headless \
##     --script scripts/test/fix_settlement_marker/harness_marker_fixed_a12.gd

extends SceneTree

const MAIN_SCENE_PATH := "res://scenes/main.tscn"
const EVIDENCE_DIR := "res://.harness/evidence/fix-settlement-marker-fixed-position"

## Frame budget to wait for the FIRST settlement marker. At ~5 ticks/frame this
## is ~7500 sim ticks — far above the Rust harness MAX_FORM_TICKS=2000 cap, so a
## timeout here means a genuine "no formation" UNVERIFIED, not an impatient cap.
const MAX_WAIT_FORMATION_FRAMES := 1500
## Frames to advance between the two marker readings (~5 ticks/frame ⇒ ~600 sim
## ticks, comfortably more than the plan's ~200-tick interval). After this many
## frames we additionally keep going (up to MAX_MOVE_FRAMES) until ≥1 member has
## actually moved, so the movement precondition is met deterministically.
const MIN_MOVE_FRAMES := 120
const MAX_MOVE_FRAMES := 1500
## Large wall-time multiplier so the fixed-timestep accumulator fills each frame
## even though headless frame deltas are sub-millisecond.
const TIME_SCALE := 80.0
const SIM_SPEED := 4.0

enum Phase { BOOT, WAIT_FORMATION, MOVE, DONE }

var _phase: int = Phase.BOOT
var _frames_in_phase: int = 0
var _frames_total: int = 0
var _main: Node = null
var _world_sim: Node = null
var _world_renderer: Node = null
var _finalized: bool = false

# Capture-at-T state.
var _markers_t: Dictionary = {}          # settlement_id(int) -> Vector2 (global_position)
var _agent_pos_t: Dictionary = {}        # agent entity_bits(int) -> Vector2i (tile)
var _moved_seen: bool = false

var _assertions: Array = []              # {name, ok, detail}
var _unverified: bool = false
var _unverified_reason: String = ""
var _log_lines: PackedStringArray = PackedStringArray()


func _init() -> void:
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(EVIDENCE_DIR))
	Engine.time_scale = TIME_SCALE
	_log("A12 marker-fixed harness boot (time_scale=%s)" % TIME_SCALE)
	process_frame.connect(_on_frame)
	call_deferred("_boot_main_scene")


func _boot_main_scene() -> void:
	var packed := load(MAIN_SCENE_PATH) as PackedScene
	if packed == null:
		_unverify("failed to load %s" % MAIN_SCENE_PATH)
		_finalize_and_quit(1)
		return
	_main = packed.instantiate()
	root.add_child(_main)
	_log("main.tscn instantiated")


func _on_frame() -> void:
	_frames_total += 1
	_frames_in_phase += 1
	match _phase:
		Phase.BOOT:
			_phase_boot()
		Phase.WAIT_FORMATION:
			_phase_wait_formation()
		Phase.MOVE:
			_phase_move()
		Phase.DONE:
			pass


func _phase_boot() -> void:
	if _main == null:
		if _frames_in_phase > 600:
			_unverify("scene never instantiated")
			_finalize_and_quit(1)
		return
	_world_sim = _main.get_node_or_null("WorldSim")
	_world_renderer = _main.get_node_or_null("WorldRenderer")
	if _world_sim == null or _world_renderer == null:
		if _frames_in_phase > 600:
			_unverify("WorldSim/WorldRenderer node not found")
			_finalize_and_quit(1)
		return
	# Crank the per-frame sim-tick budget so a settlement forms within the
	# bounded frame budget (see TIME_SCALE / SIM_SPEED notes).
	if _world_sim.has_method("set_sim_speed"):
		_world_sim.call("set_sim_speed", SIM_SPEED)
	_log("nodes resolved; driving sim toward first settlement marker")
	_enter(Phase.WAIT_FORMATION)


func _phase_wait_formation() -> void:
	var markers: Variant = _read_markers()
	if markers == null:
		_unverify("WorldRenderer._furniture_sprites not introspectable (got non-Dictionary)")
		_finalize_and_quit(1)
		return
	if (markers as Dictionary).size() >= 1:
		_markers_t = markers
		_agent_pos_t = _read_agent_positions()
		_log("T capture: %d marker(s), %d agent(s) — frame %d"
				% [_markers_t.size(), _agent_pos_t.size(), _frames_total])
		for sid in _markers_t.keys():
			_log("  marker[%d] @ %s" % [sid, str(_markers_t[sid])])
		_enter(Phase.MOVE)
		return
	if _frames_in_phase > MAX_WAIT_FORMATION_FRAMES:
		_unverify("no settlement marker appeared within %d frames (~%d sim ticks); "
				% [MAX_WAIT_FORMATION_FRAMES, MAX_WAIT_FORMATION_FRAMES * 5]
				+ "cannot verify marker fixedness — escalate to windowed Visual Verify")
		_finalize_and_quit(1)


func _phase_move() -> void:
	# Track whether ANY agent moved since the T capture (movement precondition).
	if not _moved_seen:
		var now := _read_agent_positions()
		for aid in _agent_pos_t.keys():
			if now.has(aid) and now[aid] != _agent_pos_t[aid]:
				_moved_seen = true
				break
	# Keep advancing until we have both: the minimum interval AND a moved member.
	if _frames_in_phase < MIN_MOVE_FRAMES:
		return
	if not _moved_seen and _frames_in_phase < MAX_MOVE_FRAMES:
		return
	# Interval elapsed — re-read and evaluate.
	_evaluate(_read_markers())
	_finalize_and_quit(0)


# Re-read markers and run the three A12 conditions (mirror Rust A3):
#   (1) non-vacuity: intersection of marker ids at T and T+Δ ≥ 1
#   (2) movement precondition: ≥1 member moved over the interval
#   (3) fixedness: every intersection marker's world position IDENTICAL
func _evaluate(markers_t2_variant: Variant) -> void:
	if markers_t2_variant == null:
		_unverify("WorldRenderer._furniture_sprites not introspectable at T+Δ")
		return
	var markers_t2 := markers_t2_variant as Dictionary

	# (1) intersection.
	var intersection: Array = []
	for sid in _markers_t.keys():
		if markers_t2.has(sid):
			intersection.append(sid)
	_record("a12_intersection_non_empty", intersection.size() >= 1,
			"|I|=%d (markers_t=%d markers_t2=%d)"
			% [intersection.size(), _markers_t.size(), markers_t2.size()])
	if intersection.is_empty():
		# Vacuity guard tripped — FAIL (handled by the recorded assertion).
		return

	# (2) movement precondition.
	_record("a12_member_moved", _moved_seen,
			"a member Position changed over %d frames (~%d sim ticks): %s"
			% [_frames_in_phase, _frames_in_phase * 5, str(_moved_seen)])

	# (3) fixedness — every intersection marker world position identical.
	var drifted: int = 0
	for sid in intersection:
		var p0: Vector2 = _markers_t[sid]
		var p1: Vector2 = markers_t2[sid]
		var same := is_equal_approx(p0.x, p1.x) and is_equal_approx(p0.y, p1.y)
		if not same:
			drifted += 1
			_log("  marker[%d] DRIFTED: %s → %s" % [sid, str(p0), str(p1)])
		else:
			_log("  marker[%d] FIXED: %s == %s" % [sid, str(p0), str(p1)])
	_record("a12_marker_world_position_fixed", drifted == 0,
			"%d of %d intersection marker(s) drifted" % [drifted, intersection.size()])


# Returns a Dictionary {settlement_id:int -> Vector2 global_position} or null if
# the renderer's marker store is not introspectable as a Dictionary.
func _read_markers() -> Variant:
	if _world_renderer == null:
		return null
	var store: Variant = _world_renderer.get("_furniture_sprites")
	if not (store is Dictionary):
		return null
	var out: Dictionary = {}
	for key in (store as Dictionary).keys():
		var node: Variant = (store as Dictionary)[key]
		if node is Node2D:
			out[int(key)] = (node as Node2D).global_position
	return out


# Returns {agent entity_bits:int -> Vector2i tile} from the live agent snapshot.
func _read_agent_positions() -> Dictionary:
	var out: Dictionary = {}
	if _world_sim == null or not _world_sim.has_method("get_agent_snapshot"):
		return out
	var snap: Variant = _world_sim.call("get_agent_snapshot")
	if not (snap is Dictionary):
		return out
	var d := snap as Dictionary
	var ids: PackedInt64Array = d.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = d.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = d.get("ys", PackedInt32Array())
	var n: int = mini(ids.size(), mini(xs.size(), ys.size()))
	for i in n:
		out[int(ids[i])] = Vector2i(xs[i], ys[i])
	return out


func _enter(next: int) -> void:
	_phase = next
	_frames_in_phase = 0


func _record(name: String, ok: bool, detail: String) -> void:
	_assertions.append({"name": name, "ok": ok, "detail": detail})
	var prefix := "PASS" if ok else "FAIL"
	_log("%s %s — %s" % [prefix, name, detail])


func _unverify(reason: String) -> void:
	_unverified = true
	_unverified_reason = reason
	_log("UNVERIFIED — %s" % reason)


func _finalize_and_quit(code: int) -> void:
	if _finalized:
		return
	_finalized = true
	_enter(Phase.DONE)
	_capture_screenshot()
	_emit_results()
	_write_file("console_log.txt", "\n".join(_log_lines) + "\n")
	_log("A12 harness done (exit=%d, frames=%d)" % [code, _frames_total])
	quit(code)


func _capture_screenshot() -> void:
	var abs_path := ProjectSettings.globalize_path(EVIDENCE_DIR.path_join("a12_marker.png"))
	var vp := root.get_viewport()
	if vp != null:
		var tex := vp.get_texture()
		if tex != null:
			var img := tex.get_image()
			if img != null and img.get_width() > 0:
				if img.save_png(abs_path) == OK:
					return
	var fallback := Image.create(1, 1, false, Image.FORMAT_RGB8)
	fallback.fill(Color.BLACK)
	fallback.save_png(abs_path)


func _emit_results() -> void:
	var lines := PackedStringArray()
	lines.append("SCENARIO: a12_settlement_marker_fixed")
	var verdict := ""
	if _unverified:
		verdict = "UNVERIFIED"
		lines.append("  UNVERIFIED\treason\t%s" % _unverified_reason)
	else:
		var all_pass := _assertions.size() > 0
		for a in _assertions:
			var d := a as Dictionary
			var ok := bool(d.get("ok", false))
			if not ok:
				all_pass = false
			var tag := "PASS" if ok else "FAIL"
			lines.append("  %s\t%s\t%s"
					% [tag, String(d.get("name", "?")), String(d.get("detail", ""))])
		verdict = "PASS" if all_pass else "FAIL"
	# Machine-readable verdict line the evaluator/pipeline can grep. Distinct
	# tokens for PASS vs UNVERIFIED so they are never conflated.
	lines.append("A12_VERDICT: %s" % verdict)
	lines.append("RESULT: %s" % verdict)
	lines.append("OVERALL: %s" % ("PASS" if verdict == "PASS" else "FAIL"))
	_write_file("a12_results.txt", "\n".join(lines) + "\n")
	# Also write the canonical interactive_results.txt the visual gate greps.
	_write_file("interactive_results.txt", "\n".join(lines) + "\n")


func _write_file(rel: String, body: String) -> void:
	var abs := ProjectSettings.globalize_path(EVIDENCE_DIR.path_join(rel))
	var f := FileAccess.open(abs, FileAccess.WRITE)
	if f == null:
		_log("WARNING: failed to open " + abs)
		return
	f.store_string(body)
	f.close()


func _log(msg: String) -> void:
	print("[a12-harness] " + msg)
	_log_lines.append(msg)
