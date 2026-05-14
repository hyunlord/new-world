extends Control

# CausalPanel — "왜?" UI panel (V7 Phase 3-γ, γ-2-α scaffold).
# γ-2-α scope: empty panel scaffold + Q-key toggle.
# γ-2-β (separate land) will wire SimBridge.get_tile_causal_history /
# get_event_chain into the body and add tile-click activation.
#
# Mount: scenes/main.tscn → UI (CanvasLayer) → CausalPanel (Control + this script).
# Default visibility: hidden. Q toggles via _unhandled_input.

const PANEL_TITLE_KEY := "UI_CAUSAL_PANEL_TITLE"
const PANEL_PLACEHOLDER_KEY := "UI_CAUSAL_PANEL_PLACEHOLDER"
const PANEL_WIDTH := 320.0
const PANEL_HEIGHT := 200.0
const PANEL_MARGIN := 16.0

var _title_label: Label
var _placeholder_label: Label

func _ready() -> void:
	visible = false
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_build_layout()

func _build_layout() -> void:
	var bg := ColorRect.new()
	bg.color = Color(0.0, 0.0, 0.0, 0.78)
	bg.position = Vector2(PANEL_MARGIN, PANEL_MARGIN)
	bg.size = Vector2(PANEL_WIDTH, PANEL_HEIGHT)
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(bg)

	_title_label = Label.new()
	_title_label.text = _ltr(PANEL_TITLE_KEY)
	_title_label.position = Vector2(PANEL_MARGIN + 12.0, PANEL_MARGIN + 8.0)
	_title_label.add_theme_color_override("font_color", Color(1.0, 0.92, 0.5, 1.0))
	add_child(_title_label)

	_placeholder_label = Label.new()
	_placeholder_label.text = _ltr(PANEL_PLACEHOLDER_KEY)
	_placeholder_label.position = Vector2(PANEL_MARGIN + 12.0, PANEL_MARGIN + 40.0)
	_placeholder_label.size = Vector2(PANEL_WIDTH - 24.0, PANEL_HEIGHT - 56.0)
	_placeholder_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_placeholder_label.add_theme_color_override("font_color", Color(0.85, 0.85, 0.85, 1.0))
	add_child(_placeholder_label)

func _ltr(key: String) -> String:
	var locale := get_node_or_null("/root/Locale")
	if locale != null and locale.has_method("ltr"):
		return locale.call("ltr", key)
	return key

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_Q:
			toggle_visible()

func toggle_visible() -> void:
	visible = not visible

func is_panel_visible() -> bool:
	return visible
