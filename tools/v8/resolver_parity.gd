# resolver_parity.gd — the GDScript half of the resolver parity gate step (M-ART2 §3.4, M-ART2.1 §2.2).
# Purpose: load the imported material documents with scripts/world/art/material_resolver.gd, resolve every case of
#   tools/v8/resolver_parity_fixture.json; load the flora catalogue with scripts/world/art/flora_resolver.gd and
#   resolve every case of tools/v8/flora_parity_fixture.json (a case's `fixture` species from
#   assets/v8art/flora/tests/ added by data); write {problems, results, flora_problems, flora_results} to the JSON
#   file named after `--`; then `python3 tools/v8/resolver_parity.py --check <file>` compares them with the art
#   line's resolve.py and resolve_flora.py. Prints
#   RESOLVER_PARITY_DONE and exits 0; exits 1 when the fixture cannot be read or the file cannot be written.
#   Tooling only (headless): `godot --headless --path . --script res://tools/v8/resolver_parity.gd -- <out.json>`.
# Bridge methods called: none.
extends SceneTree

const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
const FloraResolver = preload("res://scripts/world/art/flora_resolver.gd")


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.is_empty():
		push_error("resolver_parity: usage -- <out.json>")
		quit(1)
		return
	var fixture: Variant = JSON.parse_string(FileAccess.get_file_as_string("res://tools/v8/resolver_parity_fixture.json"))
	if not fixture is Dictionary:
		push_error("resolver_parity: cannot read the fixture")
		quit(1)
		return
	var resolver: MaterialResolver = MaterialResolver.new()
	resolver.load_imported()
	var results: Array = []
	for case: Variant in (fixture as Dictionary).get("cases", []):
		var c: Dictionary = case
		var material: Variant = c["material"]
		if material is float:
			material = int(material)
		var r: Dictionary = resolver.resolve(str(c["style"]), material, str(c.get("category", "")), str(c.get("kind", "wall")))
		results.append({"case": c, "result": r})
	var flora_fixture: Variant = JSON.parse_string(FileAccess.get_file_as_string("res://tools/v8/flora_parity_fixture.json"))
	if not flora_fixture is Dictionary:
		push_error("resolver_parity: cannot read the flora fixture")
		quit(1)
		return
	var flora: Dictionary = {}
	var flora_problems: Array[String] = []
	var flora_results: Array = []
	for case: Variant in (flora_fixture as Dictionary).get("cases", []):
		var c: Dictionary = case
		var key: String = str(c.get("fixture", ""))
		if not flora.has(key):
			var f: FloraResolver = FloraResolver.new()
			f.load_imported()
			if key != "":
				var record: Variant = JSON.parse_string(FileAccess.get_file_as_string("res://assets/v8art/flora/tests/%s.json" % key))
				if not (record is Dictionary and f.add_species(record)):
					f.problems.append("fixture %s did not load" % key)
			flora_problems.append_array(f.problems)
			flora[key] = f
		var habit: Variant = c.get("habit", "")
		var r: Dictionary = (flora[key] as FloraResolver).resolve(str(c["species"]), str(c["size"]), str(c["season"]), int(c["variant_hash"]), str(habit) if habit != null else "")
		flora_results.append({"case": c, "result": r})
	var out: FileAccess = FileAccess.open(args[0], FileAccess.WRITE)
	if out == null:
		push_error("resolver_parity: cannot write %s" % args[0])
		quit(1)
		return
	out.store_string(JSON.stringify({"problems": resolver.problems, "results": results, "flora_problems": flora_problems, "flora_results": flora_results}, " "))
	out.close()
	print("RESOLVER_PARITY_DONE cases=%d problems=%d flora_cases=%d flora_problems=%d" % [results.size(), resolver.problems.size(), flora_results.size(), flora_problems.size()])
	quit(0)
