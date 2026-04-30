class_name NavigationHistory
extends RefCounted

const MAX_HISTORY: int = 5

var _stack: Array[int] = []
var _current_index: int = -1

signal history_changed()


func push(entity_id: int) -> void:
	if entity_id < 0:
		return
	if _current_index >= 0 and _current_index < _stack.size():
		if _stack[_current_index] == entity_id:
			return
	if _current_index < _stack.size() - 1:
		_stack.resize(_current_index + 1)
	_stack.append(entity_id)
	_current_index = _stack.size() - 1
	if _stack.size() > MAX_HISTORY:
		_stack.pop_front()
		_current_index -= 1
	history_changed.emit()


func go_back() -> int:
	if _current_index <= 0:
		return -1
	_current_index -= 1
	history_changed.emit()
	return _stack[_current_index]


func go_forward() -> int:
	if _current_index >= _stack.size() - 1:
		return -1
	_current_index += 1
	history_changed.emit()
	return _stack[_current_index]


func can_go_back() -> bool:
	return _current_index > 0


func can_go_forward() -> bool:
	return _current_index < _stack.size() - 1


func get_current_id() -> int:
	if _current_index >= 0 and _current_index < _stack.size():
		return _stack[_current_index]
	return -1


func get_size() -> int:
	return _stack.size()


func clear() -> void:
	_stack.clear()
	_current_index = -1
	history_changed.emit()
