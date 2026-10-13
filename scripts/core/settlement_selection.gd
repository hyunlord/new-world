extends RefCounted
## Shared presentation-only selection; -1 means no selected settlement.

static var selected_id: int = -1


static func select(id: int) -> void:
	selected_id = id


static func clear() -> void:
	selected_id = -1


static func reset() -> void:
	clear()
