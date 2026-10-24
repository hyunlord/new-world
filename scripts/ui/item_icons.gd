# item_icons.gd — item icons for panels (M-ART2 §3.6, 10.3).
# Purpose: an item kind code → its imported sprite (assets/v8art/bindings.json `items`: the row's `asset`, or the
#   first of its `assets`; an unknown code falls back to the family's `fallback`) drawn at `size` px (32 by default)
#   beside the item's name in the trade dialog and the workshop recipes. Linear filtering (a 64-px sprite shown at
#   32 px). Names stay text (EconomyText.item); the icon is extra.
# Bridge methods called: none.
extends RefCounted

const ArtData = preload("res://scripts/world/art/art_data.gd")


## The manifest id of item kind `code`'s icon ("" when none).
static func asset_of(code: int) -> String:
	var row: Dictionary = ArtData.binding("items", code)
	if row.has("asset") and row["asset"] is String:
		return str(row["asset"])
	var list: Variant = row.get("assets", [])
	if list is Array and not (list as Array).is_empty():
		return str(list[0])
	return ""


## A TextureRect showing item kind `code` at `size` × `size` (an empty rect when there is no art).
static func icon(code: int, size: int = 32) -> TextureRect:
	var rect: TextureRect = TextureRect.new()
	rect.name = "ItemIcon"
	rect.custom_minimum_size = Vector2(size, size)
	rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	rect.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	rect.texture = ArtData.texture(asset_of(code))
	rect.set_meta("item_kind", code)
	return rect
