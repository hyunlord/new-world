extends RefCounted

const Library = preload("res://scripts/ui/art_sprite_library.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")

static func panel(owner: Control, rect: Rect2) -> PanelContainer:
	owner.theme = THEME
	owner.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	var box: PanelContainer = PanelContainer.new()
	box.position = rect.position
	box.size = rect.size
	box.mouse_filter = Control.MOUSE_FILTER_IGNORE
	box.set_meta("art_panel", true)
	owner.add_child(box)
	return box

static func icon(index: int) -> Control:
	var holder: Control = Control.new()
	holder.custom_minimum_size = Vector2(32, 32)
	holder.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var sprite: Sprite2D = Sprite2D.new()
	holder.add_child(sprite)
	if Library.apply(sprite, "U1_%02d" % index):
		sprite.offset = Vector2.ZERO
		sprite.scale = Vector2(2, 2)
	return holder

static func pair(row: HBoxContainer, index: int) -> Label:
	row.add_child(icon(index))
	var label: Label = Label.new()
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(label)
	var gap: Control = Control.new()
	gap.custom_minimum_size.x = 16
	row.add_child(gap)
	return label
