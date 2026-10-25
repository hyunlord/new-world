# art_ui.gd: static UI builders shared by the HUD and panels: themed PanelContainer, 32 px U1 atlas icon and
#   icon+label pair rows; M4: a proficiency bar (‰ 0–1000) with a 2-px mark at the practitioner threshold.
# Bridge methods: none.
extends RefCounted

const ArtData = preload("res://scripts/world/art/art_data.gd")
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

static func icon(role: String) -> Control:
	var holder: Control = Control.new()
	holder.custom_minimum_size = Vector2(32, 32)
	holder.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var sprite: Sprite2D = Sprite2D.new()
	holder.add_child(sprite)
	if apply_icon(sprite, role):
		sprite.offset = Vector2.ZERO
		sprite.scale = Vector2(2, 2)
	return holder

static func pair(row: HBoxContainer, role: String) -> Label:
	row.add_child(icon(role))
	var label: Label = Label.new()
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(label)
	var gap: Control = Control.new()
	gap.custom_minimum_size.x = 16
	row.add_child(gap)
	return label

## M4: a 12-px bar of `value` ‰ (0–1000) at least `width` px wide, green at or above `threshold` ‰ (amber below), with
## a light 2-px mark where the threshold lies.
static func proficiency_bar(value: int, threshold: int, width: int) -> ProgressBar:
	var bar: ProgressBar = ProgressBar.new()
	bar.min_value = 0.0
	bar.max_value = 1000.0
	bar.value = clampi(value, 0, 1000)
	bar.show_percentage = false
	bar.custom_minimum_size = Vector2(width, 12)
	bar.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	var fill: StyleBoxFlat = StyleBoxFlat.new()
	fill.bg_color = Color("8ee07a") if value >= threshold else Color("d8a050")
	bar.add_theme_stylebox_override("fill", fill)
	var background: StyleBoxFlat = StyleBoxFlat.new()
	background.bg_color = Color8(41, 38, 33)
	bar.add_theme_stylebox_override("background", background)
	var mark: ColorRect = ColorRect.new()
	mark.name = "Threshold"
	mark.color = Color("f2e6c8")
	mark.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var at: float = clampf(float(threshold) / 1000.0, 0.0, 1.0)
	mark.anchor_left = at
	mark.anchor_right = at
	mark.anchor_top = 0.0
	mark.anchor_bottom = 1.0
	mark.offset_left = -1.0
	mark.offset_right = 1.0
	mark.offset_top = -2.0
	mark.offset_bottom = 2.0
	bar.add_child(mark)
	return bar


static func apply_icon(sprite: Sprite2D, role: String) -> bool:
	var definition: Dictionary = ArtData.bindings().get("ui_icons", {}).get(role, {})
	if definition.is_empty():
		return false
	var texture: Texture2D = ArtData.texture_at(str(definition["path"]))
	if texture == null:
		return false
	var region: Array = definition["rect"]
	sprite.texture = texture
	sprite.region_enabled = true
	sprite.region_rect = Rect2(float(region[0]), float(region[1]), float(region[2]), float(region[3]))
	sprite.centered = false
	sprite.offset = Vector2.ZERO
	sprite.scale = Vector2(2, 2)
	sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	return true
