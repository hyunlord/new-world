@tool
class_name FileBasedDebugProvider
extends DebugDataProvider

## Extends DebugDataProvider so existing typed panels (init_provider) work unchanged.
## Reads from user://debug_snapshot.json written by SimBridge every 60 ticks.
## Overrides all methods that would reach _sim_bridge (null in editor context).

const SNAPSHOT_PATH := "user://debug_snapshot.json"
const COMMAND_PATH := "user://debug_command.json"
const MAX_SNAPSHOT_AGE_MSEC := 3000

var _snapshot: Dictionary = {}
var _snapshot_time_msec: int = 0

func _init() -> void:
	# Pass null — we override all bridge-reaching methods so _sim_bridge is never called.
	super._init(null)

## Update the provider with a freshly parsed snapshot dictionary.
func update_snapshot(data: Dictionary) -> void:
	_snapshot = data
	_snapshot_time_msec = Time.get_ticks_msec()

## Returns true if the snapshot is recent enough to consider the simulation connected.
func is_snapshot_fresh() -> bool:
	if _snapshot.is_empty():
		return false
	var age := Time.get_ticks_msec() - _snapshot_time_msec
	return age < MAX_SNAPSHOT_AGE_MSEC

# --- DebugDataProvider overrides (never touch _sim_bridge) ---

func get_system_perf() -> Dictionary:
	return _snapshot.get("system_perf", {})

func get_debug_summary() -> Dictionary:
	return _snapshot.get("debug_summary", {})

func get_config_all() -> Dictionary:
	return _snapshot.get("config", {})

func get_guardrails() -> Array:
	return _snapshot.get("guardrails", [])

func get_current_tick() -> int:
	return int(get_debug_summary().get("tick", 0))

func get_tick_history() -> PackedFloat32Array:
	# Not available via file IPC — return empty array; panels handle empty gracefully.
	return PackedFloat32Array()

func query_entities(_condition: String, _threshold: float) -> PackedInt32Array:
	return PackedInt32Array()

func get_entity_detail(_entity_id: int) -> Dictionary:
	return {}

func enable_debug(_enabled: bool) -> void:
	pass  # No-op: debug mode is controlled by the running game, not the editor.

## Command file IPC is not yet polled by Rust — always returns false.
## When Rust polls debug_command.json, this can be re-enabled.
func set_config_value(_key: String, _value: float) -> bool:
	return false  # No-op: Rust does not yet poll user://debug_command.json.
