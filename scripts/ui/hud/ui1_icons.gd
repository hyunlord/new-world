# Shared bone pictograms for UI1. Bridge methods: none.
extends RefCounted

const ICONS: Texture2D = preload("res://assets/ui/ui1_icons.svg")
const SYMBOLS: Texture2D = preload("res://assets/ui/ui1_symbols.svg")
const TILE: Vector2 = Vector2(24, 24)
const ROLES: Dictionary = {
	"calendar": Vector2(0, 0), "people": Vector2(1, 0), "house": Vector2(2, 0), "build": Vector2(3, 0),
	"food": Vector2(4, 0), "hunger": Vector2(4, 0), "wood": Vector2(5, 0), "speed": Vector2(6, 0), "pause": Vector2(7, 0),
	"chop": Vector2(0, 1), "quarry": Vector2(1, 1), "dig": Vector2(2, 1), "stockpile": Vector2(3, 1),
	"remove_zone": Vector2(4, 1), "deconstruct": Vector2(5, 1), "cancel": Vector2(6, 1), "rooms": Vector2(7, 1),
	"goals": Vector2(0, 2), "why": Vector2(1, 2), "priorities": Vector2(2, 2), "book": Vector2(3, 2),
	"settings": Vector2(4, 2), "family": Vector2(5, 2), "discovery": Vector2(6, 2), "material": Vector2(7, 2),
}
const SYMBOL_ROLES: Dictionary = {"generic": 0, "chevron": 1, "up-chevron": 2, "sleep": 3, "warmth": 4, "thirst": 5, "search": 6}
static var _textures: Dictionary = {}


static func texture(role: String) -> AtlasTexture:
	if not _textures.has(role):
		var icon := AtlasTexture.new()
		icon.filter_clip = true
		if ROLES.has(role):
			icon.atlas = ICONS
			icon.region = Rect2((ROLES[role] as Vector2) * TILE, TILE)
		else:
			icon.atlas = SYMBOLS
			icon.region = Rect2(Vector2(int(SYMBOL_ROLES.get(role, 0)), 0) * TILE, TILE)
		_textures[role] = icon
	return _textures[role] as AtlasTexture
