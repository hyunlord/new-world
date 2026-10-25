# M-UX2 exact-overlap count and inspection regression; synthetic geometry is not visual acceptance.
# Bridge methods: none. The full snapshot owns badges even while the visibility oracle hides one body.
extends SceneTree

const Agents = preload("res://scripts/world/colony/agent_layer.gd")
const Layout = preload("res://scripts/world/colony/marker_layout.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
var _checks: int = 0
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, ok: bool) -> void:
	_checks += 1
	if not ok:
		_failures += 1
	print("MUX2_COINCIDENT_%s %s" % ["PASS" if ok else "FAIL", label])


func _person(bits: int, agent_id: int, at: Vector2) -> Agents.Person:
	var person: Agents.Person = Agents.Person.new()
	person.bits = bits
	person.agent_id = agent_id
	person.position = at
	person.body = Sprite2D.new()
	person.body.centered = false
	person.body.offset = Vector2(-16, -64)
	var image: Image = Image.create(32, 64, false, Image.FORMAT_RGBA8)
	image.fill(Color.WHITE)
	person.body.texture = ImageTexture.create_from_image(image)
	person.add_child(person.body)
	return person


## Seed a real clipping result to make stale draw-state observable even when a nearby badge needs no leader.
func _seed_leader(agents: Node2D) -> bool:
	var layer: Node2D = agents.get("_badge_leaders") as Node2D
	var segments: PackedVector2Array = Layout.clipped_leader(Vector2(10, 10), Vector2(50, 10), {})
	layer.set("segments", segments)
	layer.queue_redraw()
	return not (layer.get("segments") as PackedVector2Array).is_empty()


func _zoom_transition(zoom: float) -> void:
	var camera: Camera2D = Camera2D.new()
	root.add_child(camera)
	camera.zoom = Vector2.ONE * zoom
	camera.make_current()
	var agents: Node2D = Agents.new()
	root.add_child(agents)
	agents.set("_zoom", 0.5)
	var first: Agents.Person = _person(200, 20, Vector2(400, 400))
	var second: Agents.Person = _person(100, 10, Vector2(400, 400))
	agents.add_child(first)
	agents.add_child(second)
	agents.set("_persons", {200: first, 100: second})
	agents.call("_show_overlapping_figures")
	_check("zoom_%.2f_fixture_has_badge_and_leader" % zoom, not (agents.get("_coincident_badges") as Array).is_empty() and _seed_leader(agents))
	# The empty read-only snapshot removes fixture people; the actual _update zoom branch must clear cached draws.
	agents.set("_params_version", ArtData.params_version())
	agents.call("_update", 0.0)
	_check("zoom_%.2f_used_actual_camera_transition" % zoom, is_equal_approx(float(agents.get("_zoom")), zoom) and bool(agents.get("_lod")) == (zoom == 0.25))
	var layer: Node2D = agents.get("_badge_leaders") as Node2D
	_check("zoom_%.2f_clears_badges_and_leader_segments" % zoom, (agents.get("_coincident_badges") as Array).is_empty() and (layer.get("segments") as PackedVector2Array).is_empty())
	agents.free()
	camera.free()


func _run() -> void:
	var agents: Node2D = Agents.new()
	root.add_child(agents)
	agents.set("_zoom", 0.5)
	var leader_layer: Node2D = agents.get("_badge_leaders") as Node2D
	_check("leader_draw_child_owned_by_agents", leader_layer != null and leader_layer.get_parent() == agents)
	_check("leader_layer_above_scenery_below_half_bodies", not leader_layer.z_as_relative and leader_layer.z_index == 4093 and agents.z_index == 4095)
	_check("leader_material_unshaded", leader_layer.material is CanvasItemMaterial and (leader_layer.material as CanvasItemMaterial).light_mode == CanvasItemMaterial.LIGHT_MODE_UNSHADED)
	var first: Agents.Person = _person(200, 20, Vector2(400, 400))
	var second: Agents.Person = _person(100, 10, Vector2(400, 400))
	var separate: Agents.Person = _person(300, 30, Vector2(700, 500))
	for person: Agents.Person in [first, second, separate]:
		agents.add_child(person)
	agents.set("_persons", {200: first, 100: second, 300: separate})
	agents.call("_show_overlapping_figures")
	var badges: Array = agents.get("_coincident_badges")
	_check("one_badge_for_two_coincident_people", badges.size() == 1 and badges[0]["text"] == "2")
	_check("stable_membership", badges[0]["members"] == [100, 200])
	_check("unmoved_feet_and_art_anchor", first.position == Vector2(400, 400) and second.position == first.position and first.body.offset == Vector2(-16, -64))
	_check("separate_person_stays_opaque", separate.body.self_modulate.a == 1.0)
	var badge_rect: Rect2 = badges[0]["rect"]
	_check("badge_above_actual_body", badge_rect.end.y < first.position.y - 64.0)
	var before: Array = badges.duplicate(true)
	first.body.hide()
	agents.call("_show_overlapping_figures")
	_check("counterfactual_hide_cannot_change_badge", agents.get("_coincident_badges") == before)
	first.body.show()
	var hit: Vector2 = Vector2(400, 390)
	_check("default_pick_retains_existing_priority", int(agents.call("entity_at", hit)) == 200)
	_check("inspection_starts_at_stable_id", int(agents.call("entity_at", hit, true)) == 100)
	agents.set("_selected_bits", 100)
	_check("inspection_cycles_to_other_member", int(agents.call("entity_at", hit, true)) == 200)
	_check("default_pick_ignores_selection_cycle", int(agents.call("entity_at", hit)) == 200)
	agents.set("_selected_bits", 200)
	_check("inspection_wraps", int(agents.call("entity_at", hit, true)) == 100)
	_check("badge_hit_reaches_group", int(agents.call("entity_at", badge_rect.get_center(), true)) == 100)
	_check("badge_not_attack_target", int(agents.call("entity_at", badge_rect.get_center())) == -1)
	separate.position = Vector2(400, 330)
	agents.call("_show_overlapping_figures")
	badges = agents.get("_coincident_badges")
	var moved: Rect2 = badges[0]["rect"]
	var neighbor: Rect2 = Rect2(separate.position + separate.body.offset, Vector2(32, 64))
	_check("badge_avoids_neighboring_person", not moved.intersects(neighbor))
	_check("badge_stays_inside_viewport", agents.get_viewport_rect().encloses(moved))
	_check("displaced_badge_keeps_group_members", badges[0]["members"] == [100, 200])
	_check("separation_fixture_has_leader", _seed_leader(agents))
	second.position = Vector2(440, 400)
	agents.call("_show_overlapping_figures")
	_check("badge_disappears_when_group_separates", (agents.get("_coincident_badges") as Array).is_empty())
	_check("group_separation_clears_leader_segments", (leader_layer.get("segments") as PackedVector2Array).is_empty())
	second.position = first.position
	agents.call("_show_overlapping_figures")
	_check("world_reset_fixture_has_badge_and_leader", not (agents.get("_coincident_badges") as Array).is_empty() and _seed_leader(agents))
	agents.call("_on_world_reset")
	_check("world_reset_clears_badges_and_leader_segments", (agents.get("_coincident_badges") as Array).is_empty() and (leader_layer.get("segments") as PackedVector2Array).is_empty())
	agents.free()
	_zoom_transition(0.25)
	_zoom_transition(1.0)
	print("MUX2_COINCIDENT_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
