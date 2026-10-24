# panel_tooltips.gd — explicit panel button tooltips using the shared input catalog.
# Bridge methods: none. Refresh on hover reads current localized button labels.
extends RefCounted

const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
static var _catalog: Catalog = null

static func attach(button: Button, panel_action: String = "", description_key: String = "") -> void:
	if _catalog == null: _catalog = Catalog.new()
	var detail: String = button.tooltip_text
	var update: Callable = _refresh.bind(button, panel_action, description_key, detail)
	button.mouse_entered.connect(update)
	button.focus_entered.connect(update)
	update.call()

static func _refresh(button: Button, panel_action: String, description_key: String, detail: String) -> void:
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	var locale: Node = tree.root.get_node_or_null("Locale") if tree != null else null
	var lines: PackedStringArray = PackedStringArray()
	var heading: String = str(locale.call("ltr", description_key)) if locale != null and description_key != "" else button.text
	if heading != "": lines.append(heading)
	if detail != "" and detail != heading: lines.append(detail)
	lines.append(_catalog.tooltip("ui_activate"))
	if panel_action != "":
		var prefix: String = str(locale.call("ltr", "MUX_TIP_PANEL_SHORTCUT")) if locale != null else ""
		lines.append(prefix + _catalog.tooltip(panel_action))
	button.tooltip_text = "\n".join(lines)
