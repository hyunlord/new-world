# look2_data_test.gd — construction frame selection and replacement manifest contracts.
# Bridge methods called: none; checks the actual imported static meshes and presentation bindings.
extends SceneTree

const Data3D = preload("res://scripts/world3d/data3d.gd")
const Structures = preload("res://scripts/world3d/structures3d.gd")
var failures: int = 0
var checks: int = 0

func _init() -> void:
	call_deferred("_run")

func _check(label: String, condition: bool) -> void:
	checks += 1
	if not condition:
		failures += 1
		push_error("LOOK2_DATA_FAIL " + label)

func _run() -> void:
	var templates: Dictionary = Data3D.binding("sites.templates", {})
	_check("all 25 template bindings retained", templates.size() == 25)
	for identity: String in templates:
		var stages: Array = templates[identity]
		_check(identity + " foundation/frame/half", stages.size() == 3)
		if stages.size() != 3:
			continue
		for index: int in 3:
			var model: String = str(stages[index])
			_check(model + " exists", Data3D.has_model(model))
			_check(model + " selected", Structures.stage_model(stages, (float(index) + 0.5) / 3.0) == model)
			var row: Dictionary = Data3D.model_row(model)
			_check(model + " metadata", str(row.get("template", "")) == identity)
			var mesh: Mesh = Data3D.mesh(model)
			_check(model + " real surfaces", mesh != null and mesh.get_surface_count() > 0 and mesh.get_surface_count() <= 10)
		_check(identity + " frame stage", str(Data3D.model_row(str(stages[1])).get("stage", "")) == "frame")
	_check("empty stage fallback", Structures.stage_model([], 0.5) == "")
	_check("unknown model fallback", Structures.stage_model(["look2_missing_fixture"], 0.5) == "")
	print("LOOK2_DATA_DONE failures=%d checks=%d" % [failures, checks])
	quit(0 if failures == 0 else 1)
