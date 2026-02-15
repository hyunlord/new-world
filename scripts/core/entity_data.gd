extends RefCounted

var id: int = -1
var entity_name: String = ""
var position: Vector2i = Vector2i.ZERO
var is_alive: bool = true

## Needs (0.0 = critical, 1.0 = full)
var hunger: float = 1.0
var energy: float = 1.0
var social: float = 1.0

## Attributes
var age: int = 0
var speed: float = 1.0
var strength: float = 1.0

## AI State
var current_action: String = "idle"
var current_goal: String = ""
var action_target: Vector2i = Vector2i(-1, -1)
var action_timer: int = 0


## Serialize to dictionary for save/load
func to_dict() -> Dictionary:
	return {
		"id": id,
		"entity_name": entity_name,
		"position_x": position.x,
		"position_y": position.y,
		"is_alive": is_alive,
		"hunger": hunger,
		"energy": energy,
		"social": social,
		"age": age,
		"speed": speed,
		"strength": strength,
		"current_action": current_action,
		"current_goal": current_goal,
		"action_target_x": action_target.x,
		"action_target_y": action_target.y,
		"action_timer": action_timer,
	}


## Deserialize from dictionary
static func from_dict(data: Dictionary) -> RefCounted:
	var script = load("res://scripts/core/entity_data.gd")
	var e = script.new()
	e.id = data.get("id", -1)
	e.entity_name = data.get("entity_name", "")
	e.position = Vector2i(data.get("position_x", 0), data.get("position_y", 0))
	e.is_alive = data.get("is_alive", true)
	e.hunger = data.get("hunger", 1.0)
	e.energy = data.get("energy", 1.0)
	e.social = data.get("social", 1.0)
	e.age = data.get("age", 0)
	e.speed = data.get("speed", 1.0)
	e.strength = data.get("strength", 1.0)
	e.current_action = data.get("current_action", "idle")
	e.current_goal = data.get("current_goal", "")
	e.action_target = Vector2i(data.get("action_target_x", -1), data.get("action_target_y", -1))
	e.action_timer = data.get("action_timer", 0)
	return e
