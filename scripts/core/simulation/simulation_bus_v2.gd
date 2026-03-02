extends Node

## Runtime event payload channel.
## event_type_id: stable numeric event kind from Rust runtime.
## payload: event data dictionary.
## tick: -1 when event has no explicit tick.
@warning_ignore("unused_signal")
signal event_emitted(event_type_id: int, payload: Dictionary, tick: int)

## Runtime UI command channel.
@warning_ignore("unused_signal")
signal ui_command(command_id: StringName, payload: Dictionary)

const EVENT_TICK_COMPLETED: int = 1
const EVENT_SIMULATION_PAUSED: int = 2
const EVENT_SIMULATION_RESUMED: int = 3
const EVENT_UI_COMMAND: int = 1000


## Emits a v2 runtime event. UI command events are also forwarded to ui_command.
func emit_runtime_event(event_type_id: int, payload: Dictionary, tick: int) -> void:
	event_emitted.emit(event_type_id, payload, tick)
	if event_type_id != EVENT_UI_COMMAND:
		return
	var command_id: StringName = StringName(str(payload.get("command_id", "")))
	var command_payload_raw: Variant = payload.get("payload", {})
	var command_payload: Dictionary = {}
	if command_payload_raw is Dictionary:
		command_payload = command_payload_raw
	ui_command.emit(command_id, command_payload)
