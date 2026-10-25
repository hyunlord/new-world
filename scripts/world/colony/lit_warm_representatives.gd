# Bounded representatives of live custom canvas shader states for an isolated startup light draw.
# Bridge methods: none. Reads live nodes/resources; never changes source nodes or material uniforms.
extends RefCounted

const MAX_SCANNED: int = 4096
const MAX_REPRESENTATIVES: int = 8
const CELL: int = 64
const SPRITE_FIELDS: Array[String] = ["texture", "hframes", "vframes", "frame", "region_enabled", "region_rect", "region_filter_clip_enabled", "flip_h", "flip_v", "centered", "offset"]

static func collect(source_root: Node, light: PointLight2D) -> Dictionary:
	var states: Array[Dictionary] = []
	var report: Dictionary = {"complete": false, "scope": "one live material per shader/effective-filter-repeat-mask-z signature; not all uniforms or driver pipelines",
		"max_scanned": MAX_SCANNED, "max_representatives": MAX_REPRESENTATIVES, "scanned": 0, "eligible_sprites": 0,
		"outside_light_range": 0, "signatures": {}, "representatives": [], "errors": []}
	if source_root == null or light == null:
		(report["errors"] as Array).append("missing source root or actual light")
		return {"states": states, "report": report}
	var pending: Array[Node] = [source_root]
	var selected: Dictionary = {}
	while not pending.is_empty() and int(report["scanned"]) < MAX_SCANNED:
		var node: Node = pending.pop_back()
		report["scanned"] = int(report["scanned"]) + 1
		var children: Array[Node] = node.get_children()
		for index: int in range(children.size() - 1, -1, -1): pending.append(children[index])
		if not node is Sprite2D or node.is_queued_for_deletion(): continue
		var source: Sprite2D = node as Sprite2D
		if not source.is_visible_in_tree() or source.texture == null: continue
		var owner: CanvasItem = source
		while owner != null and owner.use_parent_material: owner = _canvas_parent(owner)
		if owner == null or not owner.material is ShaderMaterial: continue
		var material: ShaderMaterial = owner.material as ShaderMaterial
		if material.shader == null: continue
		var z: int = source.z_index
		var cursor: CanvasItem = source
		while cursor.z_as_relative and _canvas_parent(cursor) != null:
			cursor = _canvas_parent(cursor)
			z += cursor.z_index
		if z < light.range_z_min or z > light.range_z_max or (source.light_mask & light.range_item_cull_mask) == 0:
			report["outside_light_range"] = int(report["outside_light_range"]) + 1
			continue
		var filter: int = _effective_property(source, "texture_filter")
		var repeat: int = _effective_property(source, "texture_repeat")
		var signature: String = "%s/%d/%d/%d/%d" % [material.shader.get_rid().get_id(), filter, repeat, source.light_mask, z]
		report["eligible_sprites"] = int(report["eligible_sprites"]) + 1
		var signatures: Dictionary = report["signatures"]
		signatures[signature] = int(signatures.get(signature, 0)) + 1
		if selected.has(signature): continue
		selected[signature] = true
		if states.size() >= MAX_REPRESENTATIVES:
			(report["errors"] as Array).append("representative limit exceeded: " + signature)
			continue
		var state: Dictionary = {"material": material, "filter": filter, "repeat": repeat, "z": z, "light_mask": source.light_mask,
			"rect": source.get_rect(), "self_modulate": source.self_modulate, "modulate": _inherited_modulate(source)}
		for field: String in SPRITE_FIELDS: state[field] = source.get(field)
		var receipt: Dictionary = {"source_path": str(source_root.get_path_to(source)), "signature": signature,
			"shader_path": material.shader.resource_path, "shader_rid": material.shader.get_rid().get_id(),
			"material_rid": material.get_rid().get_id(), "texture_rid": source.texture.get_rid().get_id(),
			"filter": filter, "repeat": repeat, "light_mask": source.light_mask, "absolute_z": z, "source_rect": str(source.get_rect()), "uniforms": {}}
		for uniform: Dictionary in material.shader.get_shader_uniform_list():
			var name: String = str(uniform["name"])
			var value: Variant = material.get_shader_parameter(name)
			(receipt["uniforms"] as Dictionary)[name] = {"texture_rid": (value as Texture2D).get_rid().get_id()} if value is Texture2D else str(value)
		state["receipt"] = receipt
		states.append(state)
		(report["representatives"] as Array).append(receipt)
	if not pending.is_empty(): (report["errors"] as Array).append("source scan bound exceeded")
	if states.is_empty(): (report["errors"] as Array).append("no live custom shader receiver states")
	report["complete"] = (report["errors"] as Array).is_empty()
	return {"states": states, "report": report,
		"viewport_filter": source_root.get_viewport().canvas_item_default_texture_filter,
		"viewport_repeat": source_root.get_viewport().canvas_item_default_texture_repeat}

# CanvasItem's internal parent-item helper is not script-bound in Godot4.6.
# Match its immediate-parent/top_level boundary using public Node/CanvasItem properties.
static func _canvas_parent(source: CanvasItem) -> CanvasItem:
	if source.top_level: return null
	return source.get_parent() as CanvasItem

static func _effective_property(source: CanvasItem, field: String) -> int:
	var cursor: CanvasItem = source
	while cursor != null:
		var value: int = int(cursor.get(field))
		if value != 0: return value
		cursor = _canvas_parent(cursor)
	# Inherit the same explicitly copied viewport default when every ancestor inherits.
	return 0

static func _inherited_modulate(source: CanvasItem) -> Color:
	var tint: Color = Color.WHITE
	var cursor: CanvasItem = source
	while cursor != null:
		tint *= cursor.modulate
		cursor = _canvas_parent(cursor)
	return tint

static func append_to(viewport: SubViewport, collection: Dictionary) -> Dictionary:
	var report: Dictionary = (collection.get("report", {}) as Dictionary).duplicate(true)
	if not bool(report.get("complete", false)): return report
	var states: Array = collection.get("states", [])
	var light: PointLight2D = viewport.get_node_or_null("Mux2WarmLight") as PointLight2D
	if light == null:
		report["complete"] = false
		(report["errors"] as Array).append("private actual light missing")
		return report
	var old_size: Vector2i = viewport.size
	var columns: int = mini(4, states.size())
	var rows: int = ceili(float(states.size()) / float(columns))
	var size: Vector2i = Vector2i(maxi(old_size.x, columns * CELL), old_size.y + rows * CELL)
	var radius: float = minf(light.texture.get_width(), light.texture.get_height()) * absf(light.texture_scale) * 0.5
	if size.x > 2048 or size.y > 2048 or size.x * size.y > 4194304 or Vector2(size).length() * 0.5 + light.offset.length() >= radius:
		report["complete"] = false
		(report["errors"] as Array).append("representative packing exceeds viewport/light footprint bound")
		return report
	viewport.size = size
	viewport.set("canvas_item_default_texture_filter", collection["viewport_filter"])
	viewport.set("canvas_item_default_texture_repeat", collection["viewport_repeat"])
	light.position = Vector2(size) * 0.5
	var proxies: Array[Dictionary] = []
	for index: int in states.size():
		var state: Dictionary = states[index]
		var rect: Rect2 = state["rect"]
		if rect.size.x <= 0.0 or rect.size.y <= 0.0:
			report["complete"] = false
			(report["errors"] as Array).append("empty representative geometry")
			return report
		var sprite: Sprite2D = Sprite2D.new()
		sprite.name = "LitRepresentative%d" % index
		for field: String in SPRITE_FIELDS: sprite.set(field, state[field])
		sprite.material = state["material"] as ShaderMaterial
		sprite.set("texture_filter", state["filter"])
		sprite.set("texture_repeat", state["repeat"])
		sprite.light_mask = int(state["light_mask"])
		sprite.z_as_relative = false
		sprite.z_index = int(state["z"])
		sprite.modulate = state["modulate"]
		sprite.self_modulate = state["self_modulate"]
		var factor: float = minf(1.0, float(CELL - 8) / maxf(rect.size.x, rect.size.y))
		sprite.scale = Vector2.ONE * factor
		var top: Vector2 = Vector2((index % columns) * CELL + 4, old_size.y + floori(float(index) / columns) * CELL + 4)
		sprite.position = top - rect.position * factor
		viewport.add_child(sprite)
		var pixel_rect: Rect2i = Rect2i(Vector2i(top), Vector2i(ceilf(rect.size.x * factor), ceilf(rect.size.y * factor)))
		proxies.append({"node": str(sprite.name), "pixel_rect": [pixel_rect.position.x, pixel_rect.position.y, pixel_rect.size.x, pixel_rect.size.y],
			"proxy_scale": factor, "source": state["receipt"]})
	report["proxies"] = proxies
	report["viewport_size"] = [size.x, size.y]
	report["coverage_limits"] = "Proxy geometry transforms differ; actual texture/material/sampler RIDs and selected render state are shared read-only. Future/unseen states and distinct uniforms within a signature are not exhaustively warmed."
	return report
