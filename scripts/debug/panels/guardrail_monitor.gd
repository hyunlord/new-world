class_name DebugGuardrailMonitor
extends ScrollContainer

## Guardrails tab: shows active/inactive state of all simulation guardrails.
## Refreshes every 100 frames (~1.7s at 60fps).

var _provider: DebugDataProvider
var _vbox: VBoxContainer
var _tick_counter: int = 0

const GUARDRAIL_LOCALE_KEYS: Dictionary = {
	"emotion_runaway": "DEBUG_GUARDRAIL_EMOTION_RUNAWAY",
	"genetic_collapse": "DEBUG_GUARDRAIL_GENETIC_COLLAPSE",
	"luddite_loop": "DEBUG_GUARDRAIL_LUDDITE_LOOP",
	"event_flood": "DEBUG_GUARDRAIL_EVENT_FLOOD",
	"death_spiral": "DEBUG_GUARDRAIL_DEATH_SPIRAL",
	"faction_explosion": "DEBUG_GUARDRAIL_FACTION_EXPLOSION",
	"permanent_dictatorship": "DEBUG_GUARDRAIL_PERMANENT_DICTATORSHIP",
	"religious_war_loop": "DEBUG_GUARDRAIL_RELIGIOUS_WAR_LOOP",
	"famine_spiral": "DEBUG_GUARDRAIL_FAMINE_SPIRAL",
}


func init_provider(provider: DebugDataProvider) -> void:
	_provider = provider


func _ready() -> void:
	_vbox = VBoxContainer.new()
	_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(_vbox)


func _process(_delta: float) -> void:
	if _provider == null:
		return
	_tick_counter += 1
	if _tick_counter < 100:
		return
	_tick_counter = 0
	_refresh()


func _refresh() -> void:
	for child: Node in _vbox.get_children():
		child.queue_free()

	var guardrails: Array = _provider.get_guardrails()
	for entry: Dictionary in guardrails:
		var name_key: String = entry.get("name", "")
		var active: bool = entry.get("active", false)
		var locale_key: String = GUARDRAIL_LOCALE_KEYS.get(name_key, name_key)

		var lbl := Label.new()
		lbl.text = "%s: %s" % [
			Locale.ltr(locale_key),
			Locale.ltr("DEBUG_ACTIVE") if active else Locale.ltr("DEBUG_INACTIVE")
		]
		lbl.add_theme_color_override("font_color", Color.GREEN if active else Color.GRAY)
		_vbox.add_child(lbl)
