extends RefCounted

# Diagnostic harness only. Call before root.add_child(main); never from production.
static func install(main: Node, generated_path: String, manifest_path: String) -> Dictionary:
	if not OS.is_debug_build() or main.is_inside_tree():
		return {"valid": false, "error": "requires debug build and a detached scene"}
	var renderer: Node = main.get_node_or_null("AgentRenderer")
	if renderer == null:
		return {"valid": false, "error": "missing AgentRenderer"}
	var manifest_value: Variant = JSON.parse_string(FileAccess.get_file_as_string(manifest_path))
	if not manifest_value is Dictionary:
		return {"valid": false, "error": "invalid provenance manifest"}
	var manifest: Dictionary = manifest_value
	var original: Script = renderer.get_script() as Script
	var checks: Dictionary = {
		"source_sha256": original.resource_path,
		"output_sha256": generated_path,
		"clock_sha256": "res://scripts/test/art/agent_section_clock_p10.gd",
	}
	for key: String in checks:
		var actual: String = FileAccess.get_sha256(str(checks[key]))
		if actual.is_empty() or actual != str(manifest.get(key, "")):
			return {"valid": false, "error": "provenance mismatch: " + key}
	if not bool(manifest.get("ordinary_statements_preserved", false)):
		return {"valid": false, "error": "source-preservation proof missing"}
	var diagnostic: Script = load(generated_path) as Script
	if diagnostic == null or not diagnostic.can_instantiate():
		return {"valid": false, "error": "diagnostic script failed to load"}
	renderer.set_script(diagnostic)
	if not renderer.has_method("section_profile_readout"):
		return {"valid": false, "error": "diagnostic API missing"}
	return {"valid": true, "source_sha256": manifest["source_sha256"], "output_sha256": manifest["output_sha256"], "clock_sha256": manifest["clock_sha256"], "founding_groups_status": "unavailable_separate_callback"}

static func collect(renderer: Node, process_frame: int) -> Dictionary:
	if not renderer.has_method("section_profile_readout"):
		return {"valid": false, "error": "profiling_not_installed"}
	var sample: Dictionary = renderer.call("section_profile_readout")
	if int(sample.get("process_frame", -1)) != process_frame:
		sample["valid"] = false
		sample["error"] = "probe_frame_mismatch"
	return sample
