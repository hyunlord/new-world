extends Control

# V7 Phase 14-γ — Agent click inspector panel.
#
# Right-anchored 280 px wide Control. Hidden by default until
# `display_agent(detail: Dictionary)` is called with a `found=true`
# dictionary from `WorldSimNode.get_agent_detail()`. ESC or `close()`
# hides it again.
#
# Conservative 8-field scope (P14Plan-5, locked 2026-05-25):
#   agent_id, x, y, state_tag, hunger, thirst, sleep, target_kind
#   + a `found` sentinel for stale-click safety = 9 dict keys total.
#
# Relationship / memory / combat history surfacing deferred to
# Section 16+ when the supporting systems land.

const PANEL_WIDTH := 280.0

# Top edge sits below hud_status_panel.gd (top-right, 320×180 from
# y=HUD_MARGIN(12) to y=HUD_MARGIN+PANEL_HEIGHT=192) so the two
# right-anchored panels never overlap. 180 + 12*2 = 204 → 12 px gap.
const INSPECTOR_TOP_OFFSET := 204.0

# Mirrors Rust `Hunger::SATURATION = Thirst::SATURATION = Sleep::SATURATION
# = 100.0`. ProgressBar.max_value is bound to this constant at runtime so
# the visual scale never drifts from the simulation scale (Assertion 18).
const NEED_SATURATION := 100.0

# Index-aligned with the Phase 4-γ A5 `state_tag` mapping locked in
# `collect_agent_snapshot` and propagated to `collect_agent_detail`:
#   0 = Idle, 1 = Seeking, 2 = Consuming(Agent), 3 = Consuming(other)
const STATE_LABELS: Array = ["Idle", "Seeking", "Consuming(Agent)", "Consuming(other)"]

# Index-aligned with the Phase 14-γ `target_kind` encoding in
# `collect_agent_detail`:
#   0=None, 1=Food, 2=Water, 3=Sleep, 4=ConstructionSite, 5=Agent
const TARGET_LABELS: Array = [
	"None",
	"Food",
	"Water",
	"Sleep",
	"ConstructionSite",
	"Agent",
]

var _vbox: VBoxContainer
var _label_id: Label
var _label_pos: Label
var _label_state: Label
var _label_target: Label
var _hunger_bar: ProgressBar
var _thirst_bar: ProgressBar
var _sleep_bar: ProgressBar


func _ready() -> void:
	# Right-anchored full-height panel — anchors lock the right edge of the
	# viewport, `offset_left = -PANEL_WIDTH` carves out the fixed width.
	anchor_left = 1.0
	anchor_right = 1.0
	anchor_top = 0.0
	anchor_bottom = 1.0
	offset_left = -PANEL_WIDTH
	offset_right = 0.0
	# Start below hud_status_panel.gd (top-right, bottom edge at y=192) so the
	# two right-anchored panels never overlap — INSPECTOR_TOP_OFFSET = 204.
	offset_top = INSPECTOR_TOP_OFFSET
	offset_bottom = 0.0
	# Hidden until `display_agent(...)` is called with a found=true dict.
	visible = false

	# Background fill so the panel reads as a discrete UI surface, not as a
	# transparent overlay on the world.
	var bg := ColorRect.new()
	bg.color = Color(0.08, 0.08, 0.10, 0.92)
	bg.anchor_right = 1.0
	bg.anchor_bottom = 1.0
	add_child(bg)

	_vbox = VBoxContainer.new()
	_vbox.anchor_right = 1.0
	_vbox.anchor_bottom = 1.0
	_vbox.offset_left = 8.0
	_vbox.offset_top = 8.0
	_vbox.offset_right = -8.0
	_vbox.offset_bottom = -8.0
	add_child(_vbox)

	_label_id = _make_label("Agent: -")
	_label_pos = _make_label("Pos: -")
	_label_state = _make_label("State: -")
	_label_target = _make_label("Target: -")
	_hunger_bar = _make_bar("Hunger")
	_thirst_bar = _make_bar("Thirst")
	_sleep_bar = _make_bar("Sleep")


func _make_label(text: String) -> Label:
	var lbl := Label.new()
	lbl.text = text
	_vbox.add_child(lbl)
	return lbl


func _make_bar(bar_name: String) -> ProgressBar:
	var hdr := Label.new()
	hdr.text = bar_name
	_vbox.add_child(hdr)
	var bar := ProgressBar.new()
	bar.min_value = 0.0
	# A18: runtime binding to NEED_SATURATION matches Rust SATURATION=100.0.
	bar.max_value = NEED_SATURATION
	bar.value = 0.0
	_vbox.add_child(bar)
	return bar


# V7 Phase 14-γ — populate the 8 fields from the FFI detail dict.
#
# Caller (typically WorldRenderer._try_agent_click) MUST only invoke this
# with a dictionary whose `found` key is `true`. When the FFI returns
# `found == false`, the caller should leave this panel untouched (the
# `visible` flag stays at its prior value).
func display_agent(detail: Dictionary) -> void:
	if not bool(detail.get("found", false)):
		return
	var agent_id: int = int(detail.get("agent_id", 0))
	var x: int = int(detail.get("x", 0))
	var y: int = int(detail.get("y", 0))
	var state_tag: int = clampi(int(detail.get("state_tag", 0)), 0, STATE_LABELS.size() - 1)
	var target_kind: int = clampi(int(detail.get("target_kind", 0)), 0, TARGET_LABELS.size() - 1)
	var hunger: float = float(detail.get("hunger", 0.0))
	var thirst: float = float(detail.get("thirst", 0.0))
	var sleep_val: float = float(detail.get("sleep", 0.0))
	_label_id.text = "Agent: %d" % agent_id
	_label_pos.text = "Pos: (%d, %d)" % [x, y]
	_label_state.text = "State: %s" % STATE_LABELS[state_tag]
	_label_target.text = "Target: %s" % TARGET_LABELS[target_kind]
	_hunger_bar.value = hunger
	_thirst_bar.value = thirst
	_sleep_bar.value = sleep_val
	# A19 lifecycle: panel becomes visible only after a found=true call.
	visible = true


func _unhandled_input(event: InputEvent) -> void:
	if not visible:
		return
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		close()
		get_viewport().set_input_as_handled()


func close() -> void:
	visible = false
