extends "res://scripts/core/simulation/simulation_system.gd"


func _init() -> void:
	system_name = "personality_maturation_system"
	priority = 49
	tick_interval = GameConfig.TICKS_PER_YEAR
