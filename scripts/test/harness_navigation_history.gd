## NavigationHistory unit tests — runs headless with no scene needed.
## Usage: godot --headless --path . --script scripts/test/harness_navigation_history.gd
## Output: prints each test result, exits 0 if all pass, exits 1 on any failure.

extends SceneTree

## Explicit preload required: class_name identifiers are not auto-resolved in --script headless mode.
const NavigationHistory = preload("res://scripts/ui/navigation_history.gd")
const EntityNavManagerScript = preload("res://scripts/ui/entity_navigation_manager.gd")


## Inner class: fake entity returned by MockEntityManager.
## `var position` must be declared so `"position" in entity` returns true.
class _MockEntity:
	var position: Vector2i = Vector2i(5, 10)


## Inner class: fake entity manager that always returns a _MockEntity at tile (5, 10).
class _MockEntityManager:
	func get_entity(_entity_id: int) -> Object:
		return _MockEntity.new()


var _passed: int = 0
var _failed: int = 0


func _initialize() -> void:
	_run_tests()
	print("[nav-history] %d passed, %d failed" % [_passed, _failed])
	quit(0 if _failed == 0 else 1)


func _assert(condition: bool, test_name: String, detail: String = "") -> void:
	if condition:
		print("[PASS] " + test_name)
		_passed += 1
	else:
		print("[FAIL] " + test_name + (" — " + detail if detail != "" else ""))
		_failed += 1


func _run_tests() -> void:
	_test_push_basic()
	_test_consecutive_dedup()
	_test_back_returns_previous()
	# Plan Assertion 3: get_current_id() pointer state after go_back()
	_test_back_pointer_state()
	# Plan Assertion 6–7: go_back() sentinel returns -1 AND state unchanged
	_test_back_sentinel_state_unchanged()
	_test_forward_after_back()
	# Plan Assertion 5: get_current_id() pointer state after go_forward()
	_test_forward_pointer_state()
	# Plan Assertion 8: go_forward() sentinel returns -1 AND state unchanged
	_test_forward_sentinel_state_unchanged()
	# Plan Assertion 5 positive: can_go_forward() true after go_back()
	_test_can_go_forward_positive()
	# Plan Assertion 9: MAX_HISTORY=5 cap
	_test_max_history_truncation()
	# Plan Assertions 11–12: push-after-go_back truncates forward stack + index
	_test_push_after_back_truncates_forward()
	_test_can_go_back_initial()
	# Plan Assertion 2: non-consecutive identical IDs are NOT deduplicated
	_test_nonconsecutive_not_deduped()
	# Plan Assertion 10: oldest entry is dropped (not newest) when MAX_HISTORY overflows
	_test_max_history_oldest_dropped()
	# Plan Assertion 14 (forward-disabled polarity): can_go_forward() false when at end
	_test_can_go_forward_negative()
	# Plan Assertions 13–15: HUD _back_button.disabled / _forward_button.disabled at boundaries
	_test_hud_button_states()
	# Plan Assertion 16: history_changed signal fires on push()
	_test_signal_history_changed()
	# Plan Assertions 16–17: SimulationBus → EntityNavigationManager → history push + focus relay
	_test_signal_focus_relay()
	# Plan Assertion 18: UI_NAV_BACK + UI_NAV_FORWARD keys present in en/ and ko/
	_test_locale_keys_present()


func _test_push_basic() -> void:
	# Type: NavigationHistory (RefCounted)
	var h := NavigationHistory.new()
	h.push(42)
	_assert(h.get_current_id() == 42, "test_push_basic:current_id",
		"expected 42 got %d" % h.get_current_id())
	_assert(h.get_size() == 1, "test_push_basic:size",
		"expected 1 got %d" % h.get_size())


func _test_consecutive_dedup() -> void:
	# Type: NavigationHistory (RefCounted)
	# Plan Assertion 1: push(42) twice → size stays 1 (consecutive dedup)
	var h := NavigationHistory.new()
	h.push(42)
	h.push(42)
	_assert(h.get_size() == 1, "test_consecutive_dedup",
		"expected 1 got %d" % h.get_size())


func _test_back_returns_previous() -> void:
	# Type: NavigationHistory (RefCounted)
	var h := NavigationHistory.new()
	h.push(10)
	h.push(20)
	h.push(30)
	_assert(h.go_back() == 20, "test_back_returns_previous:first")
	_assert(h.go_back() == 10, "test_back_returns_previous:second")
	_assert(h.go_back() == -1, "test_back_returns_previous:exhausted")


# Plan Assertion 3–4: After go_back(), go_back() returns correct ID and
# get_current_id() reflects the new position.
func _test_back_pointer_state() -> void:
	# Type: NavigationHistory (RefCounted)
	# Setup: push(10)→push(20)→push(30)→go_back() → get_current_id() must be 20
	var h := NavigationHistory.new()
	h.push(10)
	h.push(20)
	h.push(30)
	h.go_back()
	_assert(h.get_current_id() == 20, "test_back_pointer_state",
		"expected get_current_id()==20 after go_back(), got %d" % h.get_current_id())


# Plan Assertion 6–7: go_back() at index 0 returns -1 AND leaves get_current_id() unchanged.
func _test_back_sentinel_state_unchanged() -> void:
	# Type: NavigationHistory (RefCounted)
	# Setup: push(10) only → index is 0 → go_back() must return -1 AND get_current_id() stays 10
	var h := NavigationHistory.new()
	h.push(10)
	var ret: int = h.go_back()
	_assert(ret == -1, "test_back_sentinel_state_unchanged:returns_minus1",
		"expected -1 got %d" % ret)
	_assert(h.get_current_id() == 10, "test_back_sentinel_state_unchanged:state_unchanged",
		"expected get_current_id()==10 (unchanged) after sentinel, got %d" % h.get_current_id())


func _test_forward_after_back() -> void:
	# Type: NavigationHistory (RefCounted)
	# Plan Assertion 5: go_forward() returns the correct next ID after go_back()
	var h := NavigationHistory.new()
	h.push(10)
	h.push(20)
	h.push(30)
	h.go_back()
	_assert(h.go_forward() == 30, "test_forward_after_back",
		"expected 30 got %d" % h.get_current_id())


# Plan Assertion 5: After go_forward(), get_current_id() reflects the new position.
func _test_forward_pointer_state() -> void:
	# Type: NavigationHistory (RefCounted)
	# Setup: push(10)→push(20)→push(30)→go_back()→go_forward() → get_current_id() must be 30
	var h := NavigationHistory.new()
	h.push(10)
	h.push(20)
	h.push(30)
	h.go_back()
	h.go_forward()
	_assert(h.get_current_id() == 30, "test_forward_pointer_state",
		"expected get_current_id()==30 after go_forward(), got %d" % h.get_current_id())


# Plan Assertion 8: go_forward() at end returns -1 AND leaves get_current_id() unchanged.
func _test_forward_sentinel_state_unchanged() -> void:
	# Type: NavigationHistory (RefCounted)
	# Setup: push(10)→push(20)→push(30) (at end) → go_forward() must return -1 AND get_current_id() stays 30
	var h := NavigationHistory.new()
	h.push(10)
	h.push(20)
	h.push(30)
	var ret: int = h.go_forward()
	_assert(ret == -1, "test_forward_sentinel_state_unchanged:returns_minus1",
		"expected -1 got %d" % ret)
	_assert(h.get_current_id() == 30, "test_forward_sentinel_state_unchanged:state_unchanged",
		"expected get_current_id()==30 (unchanged) after sentinel, got %d" % h.get_current_id())


# Plan Assertion 5 positive: can_go_forward() returns true after go_back() when forward stack is non-empty.
func _test_can_go_forward_positive() -> void:
	# Type: NavigationHistory (RefCounted)
	# Setup: push(10)→push(20)→push(30)→go_back() → can_go_forward() must be true
	var h := NavigationHistory.new()
	h.push(10)
	h.push(20)
	h.push(30)
	h.go_back()
	_assert(h.can_go_forward(), "test_can_go_forward_positive",
		"expected can_go_forward()==true after go_back() with items ahead")


# Plan Assertion 9: MAX_HISTORY=5 cap
func _test_max_history_truncation() -> void:
	# Type: NavigationHistory (RefCounted)
	var h := NavigationHistory.new()
	for i in range(10):
		h.push(i)
	_assert(h.get_size() == NavigationHistory.MAX_HISTORY, "test_max_history_truncation",
		"expected %d got %d" % [NavigationHistory.MAX_HISTORY, h.get_size()])


# Plan Assertions 11–12: push() after go_back() truncates forward stack and updates index.
func _test_push_after_back_truncates_forward() -> void:
	# Type: NavigationHistory (RefCounted)
	var h := NavigationHistory.new()
	h.push(10)
	h.push(20)
	h.push(30)
	h.go_back()
	h.push(40)
	_assert(h.get_size() == 3, "test_push_after_back_truncates_forward:size",
		"expected 3 got %d" % h.get_size())
	_assert(not h.can_go_forward(), "test_push_after_back_truncates_forward:no_forward")


func _test_can_go_back_initial() -> void:
	# Type: NavigationHistory (RefCounted)
	var h := NavigationHistory.new()
	_assert(not h.can_go_back(), "test_can_go_back_initial:empty")
	h.push(42)
	_assert(not h.can_go_back(), "test_can_go_back_initial:single_entry")
	h.push(43)
	_assert(h.can_go_back(), "test_can_go_back_initial:two_entries")


# Plan Assertion 2: Non-consecutive identical IDs must NOT be deduplicated.
# (A set-based implementation would fail this — push(A)→push(B)→push(A) would collapse to {A,B}.)
func _test_nonconsecutive_not_deduped() -> void:
	# Type: NavigationHistory (RefCounted)
	# push(42) → push(99) → push(42): only consecutive dupes are skipped, so size must be 3.
	var h := NavigationHistory.new()
	h.push(42)
	h.push(99)
	h.push(42)
	_assert(h.get_size() == 3, "test_nonconsecutive_not_deduped",
		"expected 3 (A→B→A not collapsed) got %d" % h.get_size())


# Plan Assertion 10: MAX_HISTORY overflow drops the OLDEST entry, not the newest.
func _test_max_history_oldest_dropped() -> void:
	# Type: NavigationHistory (RefCounted)
	# Push MAX_HISTORY+1 entries (0..MAX_HISTORY). The oldest (0) is popped from front.
	# get_current_id() must equal MAX_HISTORY (the newest push), not 0 (the oldest).
	var h := NavigationHistory.new()
	for i: int in range(NavigationHistory.MAX_HISTORY + 1):
		h.push(i)
	_assert(h.get_current_id() == NavigationHistory.MAX_HISTORY,
		"test_max_history_oldest_dropped",
		"expected current_id==%d (newest) got %d" % [NavigationHistory.MAX_HISTORY, h.get_current_id()])


# Plan Assertion 14 (forward-disabled polarity): can_go_forward() must be false when at the last entry.
func _test_can_go_forward_negative() -> void:
	# Type: NavigationHistory (RefCounted)
	# After push(10)→push(20) the index is at end → can_go_forward() must be false.
	var h := NavigationHistory.new()
	h.push(10)
	h.push(20)
	_assert(not h.can_go_forward(), "test_can_go_forward_negative:at_end",
		"expected can_go_forward()==false when index is at last entry")


# Plan Assertions 13–15: HUD _back_button.disabled / _forward_button.disabled at boundaries.
# Tests ACTUAL Button.disabled property, not the NavigationHistory predicates.
func _test_hud_button_states() -> void:
	# Type: Button (Node), NavigationHistory (RefCounted)
	# Replicates the history_changed → button.disabled wiring in hud.gd.
	var h := NavigationHistory.new()
	var back_btn := Button.new()
	var fwd_btn := Button.new()
	back_btn.disabled = true
	fwd_btn.disabled = true
	h.history_changed.connect(func() -> void:
		back_btn.disabled = not h.can_go_back()
		fwd_btn.disabled = not h.can_go_forward()
	)

	# Assertion 13 (back-disabled polarity): _back_button.disabled == true when history is empty.
	_assert(back_btn.disabled,
		"test_hud_button:back_disabled_empty",
		"_back_button.disabled must be true when history is empty")

	# Push one entry — single entry means index 0, still can't go back.
	h.push(10)
	_assert(back_btn.disabled,
		"test_hud_button:back_disabled_single_entry",
		"_back_button.disabled must be true with only one entry")

	# Push second entry — back is now available.
	h.push(20)
	# Assertion 15 (back-enabled polarity): _back_button.disabled == false when can go back.
	_assert(not back_btn.disabled,
		"test_hud_button:back_enabled_two_entries",
		"_back_button.disabled must be false when 2+ entries exist")

	# Assertion 14 (forward-disabled polarity): _forward_button.disabled == true when at end.
	_assert(fwd_btn.disabled,
		"test_hud_button:forward_disabled_at_end",
		"_forward_button.disabled must be true when index is at last entry")

	# Navigate back — forward stack is now non-empty.
	h.go_back()
	# Assertion 15 (forward-enabled polarity): _forward_button.disabled == false after go_back().
	_assert(not fwd_btn.disabled,
		"test_hud_button:forward_enabled_after_back",
		"_forward_button.disabled must be false after go_back()")

	back_btn.free()
	fwd_btn.free()


# Plan Assertion 16: NavigationHistory.history_changed signal fires on push().
func _test_signal_history_changed() -> void:
	# Type: NavigationHistory (RefCounted)
	# GDScript lambdas capture bool by value; use Array[bool] as a reference container.
	var h := NavigationHistory.new()
	var fired: Array[bool] = [false]
	h.history_changed.connect(func() -> void: fired[0] = true)
	h.push(42)
	_assert(fired[0], "test_signal_history_changed",
		"expected history_changed to emit on push()")


# Plan Assertions 16–17: Full signal chain via EntityNavigationManager + SimulationBus.
# Assert 16: SimulationBus.entity_navigation_requested.emit(id) causes history.push(id).
# Assert 17: Same emit causes entity_focus_requested(id, pos) to fire on the manager.
# Uses _MockEntityManager (returns entity at tile (5,10)) so focus() does not return early.
func _test_signal_focus_relay() -> void:
	# Type: EntityNavigationManager (Node), _MockEntityManager (RefCounted)
	var fake_em := _MockEntityManager.new()
	var mgr = EntityNavManagerScript.new()
	mgr.setup(fake_em)

	# Track entity_focus_requested emissions.
	var focus_id: Array[int] = [-1]
	var focus_pos: Array[Vector2] = [Vector2(-1.0, -1.0)]
	mgr.entity_focus_requested.connect(func(eid: int, pos: Vector2) -> void:
		focus_id[0] = eid
		focus_pos[0] = pos
	)

	# Trigger the navigation chain via SimulationBus (as wildlife click / panel link would do).
	# Use get_root().get_node() for runtime lookup — bare "SimulationBus" identifier fails
	# compile-time resolution in --script entry-point context (autoloads not yet registered).
	var sim_bus: Node = get_root().get_node_or_null("SimulationBus")
	if sim_bus == null:
		_assert(false, "test_signal_focus_relay:simbus_unavailable",
			"SimulationBus autoload not found at /root/SimulationBus")
		mgr.free()
		return
	sim_bus.entity_navigation_requested.emit(42)

	# Assertion 16: history received the push.
	_assert(mgr.history.get_current_id() == 42,
		"test_signal_focus_relay:history_push",
		"expected history.get_current_id()==42 after bus emit, got %d" % mgr.history.get_current_id())

	# Assertion 17: entity_focus_requested(42, pos) was emitted by the manager.
	_assert(focus_id[0] == 42,
		"test_signal_focus_relay:focus_emitted_id",
		"expected entity_focus_requested(42, _) to fire, got id=%d" % focus_id[0])
	_assert(focus_pos[0].x >= 0.0,
		"test_signal_focus_relay:focus_emitted_pos",
		"expected valid tile position (x>=0), got %s" % str(focus_pos[0]))

	mgr.free()


# Plan Assertion 18: UI_NAV_BACK and UI_NAV_FORWARD keys exist in both en/ and ko/ locale files.
func _test_locale_keys_present() -> void:
	for locale_path: String in ["res://localization/en/ui.json", "res://localization/ko/ui.json"]:
		var f := FileAccess.open(locale_path, FileAccess.READ)
		_assert(f != null, "test_locale_keys_present:open:" + locale_path)
		if f == null:
			continue
		var data: Variant = JSON.parse_string(f.get_as_text())
		f.close()
		var tag: String = locale_path.get_slice("/", 3)  # "en" or "ko"
		_assert(data is Dictionary and (data as Dictionary).has("UI_NAV_BACK"),
			"test_locale_keys_present:UI_NAV_BACK:" + tag,
			"key UI_NAV_BACK missing from " + locale_path)
		_assert(data is Dictionary and (data as Dictionary).has("UI_NAV_FORWARD"),
			"test_locale_keys_present:UI_NAV_FORWARD:" + tag,
			"key UI_NAV_FORWARD missing from " + locale_path)
