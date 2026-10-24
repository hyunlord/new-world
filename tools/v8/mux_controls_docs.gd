# mux_controls_docs.gd: exports the help overlay's single controls registry as README Markdown.
# Bridge methods: none. Usage: --script res://tools/v8/mux_controls_docs.gd -- /absolute/output.md /absolute/ko.json
extends SceneTree

const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")

func _initialize() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() != 2:
		push_error("Expected output Markdown path and flat locale JSON path")
		quit(2)
		return
	var locale_file: FileAccess = FileAccess.open(args[1], FileAccess.READ)
	if locale_file == null:
		quit(2)
		return
	var parsed: Variant = JSON.parse_string(locale_file.get_as_text())
	if not parsed is Dictionary:
		quit(2)
		return
	var strings: Dictionary = parsed
	var catalog: Catalog = Catalog.new()
	if not catalog.error.is_empty():
		push_error(catalog.error)
		quit(2)
		return
	for entry: Dictionary in catalog.entries:
		for key: String in [str(entry["text_key"]), str(entry["input_key"])]:
			if not strings.has(key):
				push_error("Missing translation: " + key)
				quit(2)
				return
	var output: FileAccess = FileAccess.open(args[0], FileAccess.WRITE)
	if output == null:
		quit(2)
		return
	output.store_string(catalog.markdown_table(func(key: String) -> String: return str(strings.get(key, key))) + "\n")
	quit(0)
