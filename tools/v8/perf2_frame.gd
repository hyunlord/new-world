# perf2_frame.gd — PERF2-0 frame profile of the real game scene (windowed only; tools, never in the game).
# Purpose: `--load <wsave>` (a perf2 stress scene), camera on the camp at `--zoom Z`, vsync off, no frame cap, the
#   simulation at 1× (or `--paused 1`); `--warmup S` seconds, then every drawn frame for `--seconds S`:
#   frame time (post-draw to post-draw), process / physics time, render CPU and GPU time, draw calls, primitives,
#   objects, pipeline compilations (Performance.PIPELINE_COMPILATIONS_*; a frame with any is a compile hitch), the
#   SimReads getter calls and µs of that frame (Rust call + Dictionary marshal), the bridge's native tick time
#   (WORLDSIM_MUX2_PROFILE=1) and the process time of every child of Main, World3D, UI and UI/MuxHud (Probe stamps in
#   tree order: the difference between consecutive probes is the subtree's _process time that frame).
#   At the end: video/texture/buffer memory, object/node counts, distinct materials/shaders/meshes in the 3D tree and
#   the serialized size of every snapshot read this run. `--hide Env:shadows` (and other world3d_fps.gd switches)
#   for pass costs; `--overlay 1` draws an on-screen frame-time graph (for the capture video).
#   Writes `--out <json>` and prints one `PERF2_FRAME <summary json>` line.
# Bridge methods called: load_world, mux2_native_timings (direct); via SimReads get_world_info, get_agent_snapshot,
#   get_building_snapshot.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")

var _post_usec: int = -1
var _first_frame_msec: int = -1


## A process-order stamp; `order` is its index in tree order (tools only).
class Probe extends Node:
	var usec: int = -1
	var frame: int = -1
	var key: String = ""

	func _process(_delta: float) -> void:
		usec = Time.get_ticks_usec()
		frame = Engine.get_process_frames()


## The frame-time graph of the capture video: the last 240 frames, guides at 16.7 / 33.3 / 50 ms.
class Graph extends Control:
	var values: PackedFloat32Array = PackedFloat32Array()
	var caption: String = ""

	func push(ms: float, text: String) -> void:
		values.append(ms)
		if values.size() > 240:
			values = values.slice(values.size() - 240)
		caption = text
		queue_redraw()

	func _draw() -> void:
		var w: float = 480.0
		var h: float = 120.0
		draw_rect(Rect2(0, 0, w, h + 22), Color(0, 0, 0, 0.6))
		for guide: float in [16.7, 33.3, 50.0]:
			var y: float = h - guide / 60.0 * h
			draw_line(Vector2(0, y), Vector2(w, y), Color(1, 1, 1, 0.35) if guide < 20.0 else Color(1, 0.4, 0.3, 0.5), 1.0)
		for i: int in values.size():
			var v: float = values[i]
			var x: float = w - (values.size() - i) * 2.0
			var c: Color = Color(0.4, 1, 0.4) if v <= 16.7 else (Color(1, 0.9, 0.3) if v <= 33.3 else Color(1, 0.3, 0.3))
			draw_line(Vector2(x, h), Vector2(x, h - minf(v, 60.0) / 60.0 * h), c, 2.0)
		draw_string(ThemeDB.fallback_font, Vector2(6, h + 17), caption, HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color.WHITE)


func _init() -> void:
	RenderingServer.frame_post_draw.connect(_on_post_draw)
	call_deferred("_run")


func _on_post_draw() -> void:
	if _first_frame_msec < 0:
		_first_frame_msec = Time.get_ticks_msec()


func _arg(name: String, fallback: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else fallback


## Probes before every child of `parent`, and one after the last.
func _bracket(parent: Node) -> void:
	if parent == null:
		return
	for child: Node in parent.get_children():
		if child is Probe:
			continue
		var probe: Probe = Probe.new()
		probe.name = "Perf2Probe" + str(child.name)
		probe.key = str(parent.name) + "/" + str(child.name)
		parent.add_child(probe)
		parent.move_child(probe, child.get_index())
	var tail: Probe = Probe.new()
	tail.name = "Perf2ProbeEnd"
	tail.key = str(parent.name) + "/<end>"
	parent.add_child(tail)


func _probes_in_order(node: Node, out: Array) -> void:
	for child: Node in node.get_children():
		if child is Probe:
			out.append(child)
		_probes_in_order(child, out)


func _monitor(name: String) -> float:
	# Monitors added after 4.3 are looked up by name so the tool still parses on older builds.
	if ClassDB.class_has_integer_constant("Performance", name):
		return Performance.get_monitor(ClassDB.class_get_integer_constant("Performance", name))
	return -1.0


func _compilations() -> float:
	var total: float = 0.0
	for n: String in ["PIPELINE_COMPILATIONS_CANVAS", "PIPELINE_COMPILATIONS_MESH", "PIPELINE_COMPILATIONS_SURFACE",
			"PIPELINE_COMPILATIONS_DRAW", "PIPELINE_COMPILATIONS_SPECIALIZATION"]:
		total += maxf(0.0, _monitor(n))
	return total


func _pct(sorted: PackedFloat32Array, p: float) -> float:
	if sorted.is_empty():
		return 0.0
	var rank: int = clampi(int(ceil(p / 100.0 * sorted.size())), 1, sorted.size())
	return sorted[rank - 1]


func _scene_census(root_node: Node) -> Dictionary:
	var materials: Dictionary = {}
	var shaders: Dictionary = {}
	var meshes: Dictionary = {}
	var instances: int = 0
	var multimesh_instances: int = 0
	var lights: int = 0
	var shadow_lights: int = 0
	var stack: Array = [root_node]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		stack.append_array(n.get_children())
		if n is Light3D:
			lights += 1
			if (n as Light3D).shadow_enabled and (n as Light3D).visible:
				shadow_lights += 1
		var mats: Array = []
		if n is GeometryInstance3D:
			var gi: GeometryInstance3D = n
			if gi.material_override != null:
				mats.append(gi.material_override)
		if n is MeshInstance3D:
			var mi: MeshInstance3D = n
			if mi.mesh != null:
				meshes[mi.mesh.get_rid()] = true
				instances += 1
				for s: int in mi.mesh.get_surface_count():
					var m: Material = mi.get_active_material(s)
					if m != null:
						mats.append(m)
		elif n is MultiMeshInstance3D:
			var mm: MultiMeshInstance3D = n
			if mm.multimesh != null and mm.multimesh.mesh != null:
				meshes[mm.multimesh.mesh.get_rid()] = true
				multimesh_instances += mm.multimesh.visible_instance_count if mm.multimesh.visible_instance_count >= 0 else mm.multimesh.instance_count
				for s: int in mm.multimesh.mesh.get_surface_count():
					var m2: Material = mm.multimesh.mesh.surface_get_material(s)
					if m2 != null:
						mats.append(m2)
		for m3: Material in mats:
			materials[m3.get_rid()] = true
			if m3 is ShaderMaterial and (m3 as ShaderMaterial).shader != null:
				shaders[(m3 as ShaderMaterial).shader.resource_path if (m3 as ShaderMaterial).shader.resource_path != "" else str((m3 as ShaderMaterial).shader.get_rid())] = true
	return {"materials": materials.size(), "shaders": shaders.size(), "shader_names": shaders.keys(), "meshes": meshes.size(),
		"mesh_instances": instances, "multimesh_instances_drawn": multimesh_instances, "lights": lights, "shadow_lights": shadow_lights}


func _run() -> void:
	var boot_msec: int = Time.get_ticks_msec()
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	var camera: Camera2D = main.get_node("Camera2D")
	for _i: int in 60:
		await process_frame
	var game_first_frame_msec: int = _first_frame_msec
	# Probes go in before the load so the rebuild frames after it are attributed too.
	var w3d_early: Node = main.get_node_or_null("World3D")
	_bracket(main)
	if w3d_early != null:
		_bracket(w3d_early)
	_bracket(main.get_node_or_null("UI"))
	_bracket(main.get_node_or_null("UI/MuxHud"))
	var probes_early: Array = []
	_probes_in_order(main, probes_early)
	var after_load_top: Array = []
	var save: String = _arg("--load", "")
	var load_ms: float = -1.0
	if save != "":
		var t: int = Time.get_ticks_usec()
		if not bool(world.call("load_world", save)):
			push_error("perf2_frame: load_world failed " + save)
			quit(1)
			return
		load_ms = float(Time.get_ticks_usec() - t) / 1000.0
	# First frames after a load rebuild every view: time them separately (the "load hitch").
	var after_load: PackedFloat32Array = PackedFloat32Array()
	var last: int = Time.get_ticks_usec()
	for _i: int in 30:
		await RenderingServer.frame_post_draw
		var now: int = Time.get_ticks_usec()
		after_load.append(float(now - last) / 1000.0)
		if float(now - last) > 100000.0:
			var pf0: int = Engine.get_process_frames()
			var top0: Array = []
			for k0: int in probes_early.size() - 1:
				var a0: Probe = probes_early[k0]
				var b0: Probe = probes_early[k0 + 1]
				if a0.frame == pf0 and b0.frame == pf0 and b0.usec >= a0.usec:
					top0.append([a0.key, float(b0.usec - a0.usec) / 1000.0])
			top0.sort_custom(func(x: Array, y: Array) -> bool: return float(x[1]) > float(y[1]))
			after_load_top.append({"frame": after_load.size() - 1, "ms": float(now - last) / 1000.0, "compiles": _compilations(), "top": top0.slice(0, 5)})
		last = now
	var w3d: Node = main.get_node_or_null("World3D")
	if _arg("--paused", "") == "1":
		world.process_mode = Node.PROCESS_MODE_DISABLED
	for name: String in _arg("--hide", "").split(",", false):
		if w3d == null:
			break
		if name.begins_with("Env:"):
			var env: Environment = w3d.get_node("Sky").get("environment")
			for f: String in name.trim_prefix("Env:").split("+"):
				if f == "shadows":
					(w3d.get_node("Sky").get("sun") as DirectionalLight3D).shadow_enabled = false
				else:
					env.set(f + "_enabled", false)
			continue
		var hidden: Node = w3d.get_node_or_null(name)
		if hidden is Node3D:
			(hidden as Node3D).visible = false
			hidden.process_mode = Node.PROCESS_MODE_DISABLED
	# PERF2-1: `--speed N` runs the simulation at N× (camera_controller.set_speed, 1–4).
	if _arg("--speed", "") != "":
		camera.call("set_speed", float(_arg("--speed", "1")))
	var zoom: float = float(_arg("--zoom", "1"))
	camera.zoom = Vector2(zoom, zoom)
	var info: Dictionary = SimReads.read(world, "get_world_info")
	var centre: Vector2i = Vector2i(int(info.get("width", 64)) >> 1, int(info.get("height", 64)) >> 1)
	var b: Dictionary = SimReads.read(world, "get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	for i: int in kinds.size():
		if bool(ArtData.binding("buildings", kinds[i]).get("light", false)):
			centre = Vector2i(b["xs"][i], b["ys"][i])
			break
	camera.position = View.tile_centre(centre)
	camera.call("_clamp_position")
	var graph: Graph = null
	if _arg("--overlay", "") == "1":
		var layer: CanvasLayer = CanvasLayer.new()
		layer.layer = 100
		graph = Graph.new()
		graph.position = Vector2(12, 12)
		layer.add_child(graph)
		root.add_child(layer)
	var late: Probe = Probe.new()
	late.name = "Perf2ProbeLate"
	late.key = "<late priority>"
	late.process_priority = 100000
	main.add_child(late)
	var reads: Node = main.get_node("SimReads")
	var people_now: int = (SimReads.read(world, "get_agent_snapshot").get("ids", PackedInt64Array()) as PackedInt64Array).size()
	var warmup: float = float(_arg("--warmup", "10"))
	var seconds: float = float(_arg("--seconds", "30"))
	var t0: int = Time.get_ticks_msec()
	var warm_compiles: float = _compilations()
	while Time.get_ticks_msec() - t0 < int(warmup * 1000.0):
		await process_frame
	var vp: RID = root.get_viewport_rid()
	RenderingServer.viewport_set_measure_render_time(vp, true)
	# `--freeze 1`: stop every _process under Main (simulation, views, UI) so the frame time is the renderer's alone
	# (CPU submit + GPU; vsync off). Used where the GPU timer is unavailable.
	if _arg("--freeze", "") == "1":
		main.process_mode = Node.PROCESS_MODE_DISABLED
	var rd: RenderingDevice = RenderingServer.get_rendering_device()
	var probes: Array = []
	_probes_in_order(main, probes)
	probes.append(late)
	var has_native: bool = world.has_method("mux2_native_timings") and str(world.call("mux2_native_timings").get("status", "")) == "MEASURED"
	var native_prev: Dictionary = world.call("mux2_native_timings") if has_native else {}
	var frames: Array = []
	var frame_ms: PackedFloat32Array = PackedFloat32Array()
	var seg_sum: Dictionary = {}
	var seg_max: Dictionary = {}
	var getter_calls: Dictionary = {}
	var getter_usec: Dictionary = {}
	var compiles_prev: float = _compilations()
	var compiles_measure_start: float = compiles_prev
	var s0: int = Time.get_ticks_usec()
	_post_usec = -1
	var sample_frames: int = 0
	# `--frames N` (capture video with --write-movie): stop after N drawn frames instead of S wall seconds.
	var frame_limit: int = int(_arg("--frames", "0"))
	print("PERF2_MOVIE_START %d" % Engine.get_frames_drawn())
	while (frame_limit > 0 and sample_frames < frame_limit) or (frame_limit <= 0 and Time.get_ticks_usec() - s0 < int(seconds * 1e6)):
		await RenderingServer.frame_post_draw
		var now_usec: int = Time.get_ticks_usec()
		var pf: int = Engine.get_process_frames()
		if _post_usec < 0:
			_post_usec = now_usec
			continue
		var ms: float = float(now_usec - _post_usec) / 1000.0
		_post_usec = now_usec
		var row: Dictionary = {"ms": ms}
		row["process_ms"] = Performance.get_monitor(Performance.TIME_PROCESS) * 1000.0
		row["physics_ms"] = Performance.get_monitor(Performance.TIME_PHYSICS_PROCESS) * 1000.0
		row["draw_calls"] = Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)
		row["primitives"] = Performance.get_monitor(Performance.RENDER_TOTAL_PRIMITIVES_IN_FRAME)
		row["objects"] = Performance.get_monitor(Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)
		row["gpu_ms"] = RenderingServer.viewport_get_measured_render_time_gpu(vp)
		row["render_cpu_ms"] = RenderingServer.viewport_get_measured_render_time_cpu(vp) + RenderingServer.get_frame_setup_time_cpu()
		if rd != null and sample_frames % 60 == 5:
			var passes: Dictionary = {}
			for ti: int in rd.get_captured_timestamps_count():
				passes[rd.get_captured_timestamp_name(ti)] = float(rd.get_captured_timestamp_gpu_time(ti)) / 1e6
			row["gpu_timestamps_ms"] = passes
		var c: float = _compilations()
		row["compiles"] = c - compiles_prev
		compiles_prev = c
		var segs: Dictionary = {}
		for k: int in probes.size() - 1:
			var a: Probe = probes[k]
			var bb: Probe = probes[k + 1]
			if a.frame == pf and bb.frame == pf and bb.usec >= a.usec:
				var key: String = a.key
				segs[key] = float(bb.usec - a.usec) / 1000.0
				seg_sum[key] = float(seg_sum.get(key, 0.0)) + segs[key]
				seg_max[key] = maxf(float(seg_max.get(key, 0.0)), segs[key])
		row["segments"] = segs
		var calls: Dictionary = reads.call("frame_call_counts")
		var usec: Dictionary = reads.call("frame_call_usec")
		var bridge_ms: float = 0.0
		var bridge_calls: int = 0
		for g: String in calls:
			getter_calls[g] = int(getter_calls.get(g, 0)) + int(calls[g])
			getter_usec[g] = int(getter_usec.get(g, 0)) + int(usec.get(g, 0))
			bridge_calls += int(calls[g])
			bridge_ms += float(usec.get(g, 0)) / 1000.0
		row["bridge_calls"] = bridge_calls
		row["bridge_ms"] = bridge_ms
		var people_node: Node = main.get_node_or_null("World3D/People")
		if people_node != null and people_node.get("perf_split") is Dictionary:
			var split: Dictionary = people_node.get("perf_split")
			if int(split.get("frame", -1)) == pf:
				row["people_split"] = split
		if has_native:
			var nat: Dictionary = world.call("mux2_native_timings")
			var tot: Dictionary = nat.get("totals", {})
			var prev_tot: Dictionary = native_prev.get("totals", {})
			row["sim_tick_ms"] = float(int(tot.get("tick", 0)) - int(prev_tot.get("tick", 0))) / 1e6
			row["sim_ticks"] = int(nat.get("executed_ticks", 0)) - int(native_prev.get("executed_ticks", 0))
			native_prev = nat
		frames.append(row)
		frame_ms.append(ms)
		sample_frames += 1
		if graph != null:
			var sorted_live: PackedFloat32Array = frame_ms.duplicate()
			sorted_live.sort()
			graph.push(ms, "%s z%.1f  %.1f ms  p50 %.1f  p99 %.1f  people %d" % [save.get_file().get_basename(), zoom, ms,
				_pct(sorted_live, 50.0), _pct(sorted_live, 99.0), people_now])
	var elapsed: float = float(Time.get_ticks_usec() - s0) / 1e6
	var sorted: PackedFloat32Array = frame_ms.duplicate()
	sorted.sort()
	var over33: int = 0
	var over50: int = 0
	var hitches: Array = []
	for i: int in frames.size():
		var f: Dictionary = frames[i]
		if float(f["ms"]) > 33.3:
			over33 += 1
		if float(f["ms"]) > 50.0:
			over50 += 1
		if float(f["ms"]) > 33.3 or float(f["compiles"]) > 0.0:
			var top: Array = []
			for key2: String in f["segments"]:
				top.append([key2, f["segments"][key2]])
			top.sort_custom(func(x: Array, y: Array) -> bool: return float(x[1]) > float(y[1]))
			hitches.append({"frame": i, "ms": f["ms"], "compiles": f["compiles"], "gpu_ms": f["gpu_ms"], "render_cpu_ms": f["render_cpu_ms"],
				"sim_tick_ms": f.get("sim_tick_ms", -1), "bridge_ms": f["bridge_ms"], "top_segments": top.slice(0, 5)})
	var mean: float = 0.0
	for v: float in frame_ms:
		mean += v
	mean /= maxf(1.0, float(frame_ms.size()))
	var avg := func(key: String) -> float:
		var s: float = 0.0
		for f2: Dictionary in frames:
			s += float(f2.get(key, 0.0))
		return s / maxf(1.0, float(frames.size()))
	var seg_mean: Dictionary = {}
	for key3: String in seg_sum:
		seg_mean[key3] = float(seg_sum[key3]) / maxf(1.0, float(frames.size()))
	var getters: Dictionary = {}
	for g2: String in getter_calls:
		var snap_bytes: int = -1
		if g2.begins_with("get_"):
			snap_bytes = var_to_bytes(SimReads.read(world, g2)).size()
		getters[g2] = {"calls_per_frame": float(getter_calls[g2]) / maxf(1.0, float(frames.size())),
			"usec_per_frame": float(getter_usec[g2]) / maxf(1.0, float(frames.size())),
			"usec_per_call": float(getter_usec[g2]) / maxf(1.0, float(getter_calls[g2])), "bytes": snap_bytes}
	# Save through the bridge (as the game's save menu does), timed; the file goes to user:// and is removed.
	var save_ms: float = -1.0
	var save_bytes: int = -1
	if _arg("--time-save", "1") == "1" and world.has_method("save_world"):
		var tmp: String = ProjectSettings.globalize_path("user://perf2_frame_tmp.wsave")
		var ts: int = Time.get_ticks_usec()
		if bool(world.call("save_world", tmp)):
			save_ms = float(Time.get_ticks_usec() - ts) / 1000.0
			save_bytes = FileAccess.get_file_as_bytes(tmp).size()
			DirAccess.remove_absolute(tmp)
	var people: int = (SimReads.read(world, "get_agent_snapshot").get("ids", PackedInt64Array()) as PackedInt64Array).size()
	var summary: Dictionary = {"scene": save.get_file(), "zoom": zoom, "paused": _arg("--paused", "") == "1", "hide": _arg("--hide", ""),
		"label": _arg("--label", ""), "speed": float(_arg("--speed", "1")), "people": people, "seconds": elapsed, "frames": frames.size(), "fps": float(frames.size()) / maxf(0.001, elapsed),
		"frame_ms": {"mean": mean, "p50": _pct(sorted, 50.0), "p90": _pct(sorted, 90.0), "p99": _pct(sorted, 99.0), "max": _pct(sorted, 100.0)},
		"over_33ms_per_min": over33 / elapsed * 60.0, "over_50ms_per_min": over50 / elapsed * 60.0,
		"process_ms_mean": avg.call("process_ms"), "physics_ms_mean": avg.call("physics_ms"), "gpu_ms_mean": avg.call("gpu_ms"),
		"render_cpu_ms_mean": avg.call("render_cpu_ms"), "draw_calls_mean": avg.call("draw_calls"), "primitives_mean": avg.call("primitives"),
		"objects_mean": avg.call("objects"), "bridge_ms_mean": avg.call("bridge_ms"), "bridge_calls_mean": avg.call("bridge_calls"),
		"sim_tick_ms_mean": avg.call("sim_tick_ms") if has_native else -1.0, "sim_ticks_per_frame": avg.call("sim_ticks") if has_native else -1.0,
		"segments_ms_mean": seg_mean, "segments_ms_max": seg_max, "getters": getters,
		"compiles_warmup": compiles_measure_start - warm_compiles, "compiles_measured": compiles_prev - compiles_measure_start,
		"hitches": hitches.slice(0, 60), "hitch_count": hitches.size(),
		"load_world_ms": load_ms, "save_world_ms": save_ms, "save_world_bytes": save_bytes, "after_load_frames_ms": after_load, "after_load_slow": after_load_top,
		"startup": {"engine_to_first_frame_msec": game_first_frame_msec, "engine_to_tool_start_msec": boot_msec},
		"memory": {"video_mem": Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED), "texture_mem": _monitor("RENDER_TEXTURE_MEM_USED"),
			"buffer_mem": _monitor("RENDER_BUFFER_MEM_USED"), "static_mem": Performance.get_monitor(Performance.MEMORY_STATIC),
			"objects": Performance.get_monitor(Performance.OBJECT_COUNT), "nodes": Performance.get_monitor(Performance.OBJECT_NODE_COUNT),
			"resources": Performance.get_monitor(Performance.OBJECT_RESOURCE_COUNT), "orphans": Performance.get_monitor(Performance.OBJECT_ORPHAN_NODE_COUNT)},
		"census": _scene_census(w3d) if w3d != null else {}, "native_profile": has_native,
		"size": [root.size.x, root.size.y], "focus": root.has_focus(), "renderer": RenderingServer.get_current_rendering_method(), "rd_timestamps_supported": rd != null and rd.get_captured_timestamps_count() > 0, "freeze": _arg("--freeze", "") == "1",
		"adapter": RenderingServer.get_video_adapter_name()}
	var out: String = _arg("--out", "")
	if out != "":
		var fh: FileAccess = FileAccess.open(out, FileAccess.WRITE)
		fh.store_string(JSON.stringify({"summary": summary, "frame_ms": frame_ms, "frames": frames}))
		fh.close()
	var brief: Dictionary = summary.duplicate()
	brief.erase("hitches")
	brief.erase("getters")
	brief.erase("segments_ms_max")
	print("PERF2_FRAME " + JSON.stringify(brief))
	quit(0)
