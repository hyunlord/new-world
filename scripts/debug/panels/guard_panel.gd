class_name DebugGuardPanel
extends VBoxContainer

## Guard tab: 9 guardrails status display.

const GUARDRAIL_NAMES: Array[String] = [
	"Emotion Runaway",
	"Genetic Collapse",
	"Luddite Loop",
	"Event Flood",
	"Death Spiral",
	"Faction Explosion",
	"Permanent Dictatorship",
	"Religious War Loop",
	"Famine Spiral",
]

var _provider: DebugDataProvider
var _update_counter: int = 0
var _status_labels: Array[Label] = []


func init(provider: DebugDataProvider) -> void:
	_provider = provider
	_build_ui()


func _build_ui() -> void:
	var title := Label.new()
	title.text = "Guardrail Monitor"
	title.add_theme_font_size_override("font_size", 13)
	add_child(title)

	add_child(HSeparator.new())

	_status_labels.clear()
	for name in GUARDRAIL_NAMES:
		var hbox := HBoxContainer.new()
		var lbl := Label.new()
		lbl.text = name
		lbl.custom_minimum_size = Vector2(170, 0)
		lbl.add_theme_font_size_override("font_size", 11)
		var status := Label.new()
		status.text = "..."
		status.add_theme_font_size_override("font_size", 11)
		hbox.add_child(lbl)
		hbox.add_child(status)
		add_child(hbox)
		_status_labels.append(status)


func _process(_delta: float) -> void:
	_update_counter += 1
	if _update_counter % 60 != 0:
		return
	_update_data()


func _update_data() -> void:
	var statuses: Array = _provider.get_guardrail_status()

	for i in _status_labels.size():
		var lbl := _status_labels[i]
		if i < statuses.size():
			var entry = statuses[i]
			if entry is Dictionary:
				var active: bool = bool(entry.get("active", false))
				lbl.text = "ACTIVE" if active else "OK"
				lbl.add_theme_color_override("font_color",
					Color(1.0, 0.3, 0.3) if active else Color(0.3, 1.0, 0.3))
			else:
				lbl.text = "?"
		else:
			lbl.text = "OK"
			lbl.add_theme_color_override("font_color", Color(0.3, 1.0, 0.3))
