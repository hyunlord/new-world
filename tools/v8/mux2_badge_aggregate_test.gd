# M-UX2 collision-component count conservation and original-body/aggregate-badge picking boundary.
# Bridge methods: none. Independent quadratic graph reference and synthetic production AgentLayer fixture.
extends SceneTree

const Layout = preload("res://scripts/world/colony/marker_layout.gd")
const Agents = preload("res://scripts/world/colony/agent_layer.gd")
var _checks: int = 0
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	_failures += 0 if valid else 1
	print("MUX2_BADGE_AGGREGATE_%s %s" % ["PASS" if valid else "FAIL", label])


func _source(key: int, figure: Rect2, count: int = 2, size: Vector2 = Vector2(24, 30)) -> Dictionary:
	var members: Array[int] = []
	for i: int in count:
		members.append(key * 100 + i)
	return {"key": members[0], "members": members, "text": str(count), "size": size, "figure": figure}


func _ids(sources: Array[Dictionary]) -> Dictionary:
	var result: Dictionary = {}
	for source: Dictionary in sources:
		for bits: int in source["members"]:
			result[bits] = bits
	return result


## Frozen rule: two separate reserved rectangles, never their enclosing bounding box.
func _reference_shapes(source: Dictionary) -> Array[Rect2]:
	var figure: Rect2 = source["figure"]
	var size: Vector2 = source["size"]
	var label: Rect2 = Rect2(Vector2(figure.position.x + figure.size.x / 2.0 - size.x / 2.0, figure.position.y - size.y - 4.0).round(), size)
	return [label.grow(4.0), figure.grow(1.0)]


func _reference(sources: Array[Dictionary]) -> Array:
	var ordered: Array[Dictionary] = sources.duplicate()
	ordered.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["key"]) < int(b["key"]))
	var visited: Dictionary = {}
	var result: Array = []
	for start: int in ordered.size():
		if visited.has(start):
			continue
		var pending: Array[int] = [start]
		visited[start] = true
		var members: Array[int] = []
		while not pending.is_empty():
			var current: int = pending.pop_front()
			members.append_array(ordered[current]["members"])
			for other: int in ordered.size():
				if visited.has(other):
					continue
				var linked: bool = false
				for a: Rect2 in _reference_shapes(ordered[current]):
					for b: Rect2 in _reference_shapes(ordered[other]):
						linked = linked or a.intersects(b)
				if linked:
					visited[other] = true
					pending.append(other)
		members.sort()
		result.append(members)
	return result


func _signature(components: Array[Dictionary]) -> Array:
	var result: Array = []
	for component: Dictionary in components:
		result.append(component["members"])
	return result


func _compare(label: String, sources: Array[Dictionary]) -> void:
	var ids: Dictionary = _ids(sources)
	var components: Array[Dictionary] = Layout.aggregate_badges(sources, ids)
	_check(label + "_quadratic_reference", _signature(components) == _reference(sources))
	var source_groups: int = 0
	var member_count: int = 0
	var unique: Dictionary = {}
	var count_text_exact: bool = true
	for component: Dictionary in components:
		source_groups += (component["source_groups"] as Array).size()
		member_count += (component["members"] as Array).size()
		count_text_exact = count_text_exact and int(component["text"]) == (component["members"] as Array).size()
		for bits: int in component["members"]:
			unique[bits] = true
	_check(label + "_partition_conserves_every_member", source_groups == sources.size() and member_count == ids.size() and unique.size() == ids.size() and count_text_exact)
	var reversed: Array[Dictionary] = sources.duplicate()
	reversed.reverse()
	_check(label + "_reordered_components_identical", Layout.aggregate_badges(reversed, ids) == components)


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


func _picking_fixture() -> void:
	var agents: Node2D = Agents.new()
	root.add_child(agents)
	agents.set("_zoom", 0.5)
	var a: Agents.Person = _person(100, 10, Vector2(400, 400))
	var b: Agents.Person = _person(400, 40, Vector2(400, 400))
	var c: Agents.Person = _person(200, 20, Vector2(432, 464))
	var d: Agents.Person = _person(300, 30, Vector2(432, 464))
	var single: Agents.Person = _person(500, 50, Vector2(410, 432))
	for person: Agents.Person in [a, b, c, d, single]:
		agents.add_child(person)
	agents.set("_persons", {400: b, 100: a, 300: d, 200: c, 500: single})
	agents.call("_show_overlapping_figures")
	var badges: Array = agents.get("_coincident_badges")
	_check("production_two_sources_one_aggregate", badges.size() == 1 and (agents.get("_coincident_groups") as Array).size() == 2)
	if badges.size() != 1:
		agents.free()
		return
	_check("aggregate_counts_people_not_groups_or_singleton", badges[0]["text"] == root.get_node("Locale").call("trf", "UI_COINCIDENT_AGGREGATE_FMT", {"groups": 2, "people": 4}) and badges[0]["member_count"] == 4 and badges[0]["members"] == [100, 200, 300, 400])
	var badge_hit: Vector2 = (badges[0]["rect"] as Rect2).get_center()
	for selected: int in [100, 200, 300, 400]:
		agents.set("_selected_bits", selected)
		_check("aggregate_badge_cycle_%d" % selected, int(agents.call("entity_at", badge_hit, true)) == (selected + 100 if selected < 400 else 100))
	agents.set("_selected_bits", 100)
	_check("first_body_cycles_only_original_group", int(agents.call("entity_at", Vector2(385, 390), true)) == 400)
	agents.set("_selected_bits", 400)
	_check("first_body_wrap_skips_distant_aggregate_ids", int(agents.call("entity_at", Vector2(385, 390), true)) == 100)
	agents.set("_selected_bits", 200)
	_check("second_body_cycles_only_original_group", int(agents.call("entity_at", Vector2(442, 454), true)) == 300)
	agents.set("_selected_bits", 300)
	_check("second_body_wrap_skips_distant_aggregate_ids", int(agents.call("entity_at", Vector2(442, 454), true)) == 200)
	_check("unrelated_single_inside_aggregate_bounds_stays_single", int(agents.call("entity_at", Vector2(410, 410), true)) == 500)
	_check("default_pick_unchanged", int(agents.call("entity_at", Vector2(385, 390))) == 400)
	_check("badge_not_default_attack_target", int(agents.call("entity_at", badge_hit)) == -1)
	# Settle the intentional selection accent before comparing the counterfactual body hide.
	agents.call("_show_overlapping_figures")
	badges = agents.get("_coincident_badges")
	var before: Array = badges.duplicate(true)
	var original_groups: Array = (agents.get("_coincident_groups") as Array).duplicate(true)
	a.body.hide()
	agents.call("_show_overlapping_figures")
	_check("body_hide_preserves_aggregate_and_source_membership", agents.get("_coincident_badges") == before and agents.get("_coincident_groups") == original_groups)
	a.body.show()
	_check("all_true_feet_and_anchors_unchanged", a.position == Vector2(400, 400) and b.position == a.position and c.position == Vector2(432, 464) and d.position == c.position and a.body.offset == Vector2(-16, -64))
	c.position = Vector2(800, 464)
	d.position = c.position
	agents.call("_show_overlapping_figures")
	badges = agents.get("_coincident_badges")
	_check("movement_recomputes_components", badges.size() == 2 and badges[0]["text"] == "2" and badges[1]["text"] == "2")
	agents.call("_on_world_reset")
	_check("reset_clears_original_and_aggregate_membership", (agents.get("_coincident_groups") as Array).is_empty() and (agents.get("_coincident_badges") as Array).is_empty())
	agents.free()


func _run() -> void:
	var pair: Dictionary = _source(1, Rect2(100, 100, 16, 32))
	var isolated: Array[Dictionary] = [pair]
	_compare("isolated_pair", isolated)
	var alone: Array[Dictionary] = Layout.aggregate_badges(isolated, _ids(isolated))
	_check("single_source_retains_count_figure_size", alone.size() == 1 and alone[0]["text"] == "2" and alone[0]["figure"] == pair["figure"] and alone[0]["size"] == pair["size"])
	var label_collision: Array[Dictionary] = [pair, _source(2, Rect2(130, 100, 16, 32))]
	_compare("label_label_collision", label_collision)
	_check("label_clearance_merges", Layout.aggregate_badges(label_collision, _ids(label_collision)).size() == 1)
	var gap: Array[Dictionary] = [pair, _source(2, Rect2(132, 100, 16, 32))]
	_compare("exact_margin_gap", gap)
	_check("touching_reserved_edges_do_not_merge", Layout.aggregate_badges(gap, _ids(gap)).size() == 2)
	var resized: Array[Dictionary] = gap.duplicate(true)
	resized[1]["size"] = Vector2(40, 30)
	_check("changed_label_dimensions_recompute_collision_graph", Layout.aggregate_badges(resized, _ids(resized)).size() == 1)
	var mixed: Array[Dictionary] = [pair, _source(2, Rect2(100, 138, 16, 32))]
	_compare("label_figure_collision", mixed)
	_check("mixed_rectangles_connect", Layout.aggregate_badges(mixed, _ids(mixed)).size() == 1)
	var bodies: Array[Dictionary] = [_source(1, Rect2(100, 100, 40, 32), 2, Vector2(2, 2)), _source(2, Rect2(135, 100, 40, 32), 2, Vector2(2, 2))]
	_compare("figure_figure_collision", bodies)
	_check("body_contours_connect", Layout.aggregate_badges(bodies, _ids(bodies)).size() == 1)
	var corner: Array[Dictionary] = [_source(1, Rect2(100, 100, 16, 32), 2, Vector2(80, 10)), _source(2, Rect2(136, 119, 2, 2), 2, Vector2(2, 2))]
	_compare("empty_envelope_corner", corner)
	_check("enclosing_box_does_not_invent_edge", Layout.aggregate_badges(corner, _ids(corner)).size() == 2)
	var chain: Array[Dictionary] = [_source(1, Rect2(100, 100, 16, 32), 2), _source(2, Rect2(130, 100, 16, 32), 3), _source(3, Rect2(160, 100, 16, 32), 4)]
	_compare("transitive_mixed_counts", chain)
	var total: Array[Dictionary] = Layout.aggregate_badges(chain, _ids(chain))
	_check("transitive_count_is_nine_people", total.size() == 1 and total[0]["text"] == "9" and (total[0]["source_groups"] as Array).size() == 3)
	_compare("empty_sources", [])
	for sample: int in 16:
		var sources: Array[Dictionary] = []
		for i: int in 32:
			sources.append(_source(i + 1, Rect2(float((i * 37 + sample * 11) % 401 - 200) * 0.5, float((i * 53 + sample * 17) % 307 - 150) * 0.5, 12.5 + i % 5, 30.5), 2 + i % 3, Vector2(24 + sample % 3 * 4, 30)))
		_compare("fractional_negative_generated_%d" % sample, sources)
	_picking_fixture()
	print("MUX2_BADGE_AGGREGATE_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
