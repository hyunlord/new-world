extends SceneTree

## Direction-2 slice 2-5c — behavioral harness for the deposit/consume HUD feed
## logic in `scripts/ui/panels/hud_status_panel.gd` (assertions A1–A6).
##
## cargo `sim-test` cannot drive GDScript, so the feed-derivation behavior is
## verified here through the established `scripts/test/<feature>/harness_*.gd`
## auto-run convention. The panel's `_ingest_snapshots()` reads its snapshots
## from `_world_sim.call(...)`; we drive it with a stub world-sim that returns
## synthetic settlement snapshots (the authorized data keys `ids` +
## `food_stocks`) and a stub Locale whose templates carry a key-distinct marker
## so deposit vs consume classification is unambiguous and decoupled from the
## actual locale TEXT (which the Rust harness A7/A8 cover).
##
## We deliberately do NOT call `_ready()` (it would re-resolve `_world_sim`/
## `_locale` from a scene tree that does not exist here and reset our stubs).
## `_ingest_snapshots()` touches no scene-tree / UI API, so it runs standalone.
##
## Outcome: prints one line per assertion + a final A1..A6 verdict, writes
## evidence, and quits with code 0 (all PASS) or 1 (any FAIL).
##
## Usage:
##   godot --path . --headless \
##     --script scripts/test/stockpile_feed_2_5c/harness_2_5c.gd

const PANEL_SCRIPT := "res://scripts/ui/panels/hud_status_panel.gd"
const EVIDENCE_DIR := "res://.harness/evidence/stockpile-feed-2-5c"

var _results: Array = []          # [{name:String, ok:bool, detail:String}]
var _owned: Array = []            # nodes to free at the end


## Stub Locale: returns a 2-`%d` template carrying a key-distinct marker so the
## pushed line is unambiguously classifiable (DEPOSIT/CONSUME) with a readable
## amount. Decoupled from real locale text on purpose — text is A7/A8's job.
class StubLocale extends Node:
	func ltr(key: String) -> String:
		if key == "HUD_FEED_STOCKPILE_DEPOSIT":
			return "DEPOSIT %d %d"
		if key == "HUD_FEED_STOCKPILE_CONSUME":
			return "CONSUME %d %d"
		return key


## Stub world-sim: feeds one synthetic settlement snapshot; construction/agent
## snapshots return null so those branches are inert (no spurious feed lines).
class StubWorldSim extends Node:
	var settlement_return: Variant = null
	func get_settlement_snapshot() -> Variant:
		return settlement_return
	func get_construction_snapshot() -> Variant:
		return null
	func get_agent_snapshot() -> Variant:
		return null


func _initialize() -> void:
	_run_all()
	_finalize()


func _run_all() -> void:
	_a1_deposit()
	_a2_consume()
	_a3_threshold_boundary()
	_a4_new_settlement_silent()
	_a5_disappeared_pruned()
	_a6_short_or_absent_array()


# ── infrastructure ───────────────────────────────────────────────────────────

func _new_panel() -> Object:
	var panel: Object = load(PANEL_SCRIPT).new()
	var sim := StubWorldSim.new()
	var loc := StubLocale.new()
	panel._world_sim = sim
	panel._locale = loc
	_owned.append(panel)
	_owned.append(sim)
	_owned.append(loc)
	return panel


func _stub_of(panel: Object) -> StubWorldSim:
	return panel._world_sim as StubWorldSim


## Build a settlement snapshot dict. `foods == null` ⇒ omit the `food_stocks`
## key entirely (A6 case b). Otherwise `food_stocks` is a PackedInt32Array.
func _snap(ids_arr: Array, foods: Variant) -> Dictionary:
	var d: Dictionary = {}
	var pi := PackedInt64Array()
	for v in ids_arr:
		pi.append(int(v))
	d["ids"] = pi
	if foods != null:
		var pf := PackedInt32Array()
		for v in foods:
			pf.append(int(v))
		d["food_stocks"] = pf
	return d


## Run ONE ingest with the given snapshot, measuring only the lines pushed by
## THIS ingest (entries are cleared first). Returns the stockpile-classified
## feed lines (DEPOSIT/CONSUME), excluding formed/dissolved/born lines.
func _ingest_stockpile_lines(panel: Object, snap: Dictionary) -> Array:
	_stub_of(panel).settlement_return = snap
	panel._entries.clear()
	panel._ingest_snapshots()
	var lines: Array = []
	for e in panel._entries:
		var t: String = String(e.get("text", ""))
		if t.begins_with("DEPOSIT ") or t.begins_with("CONSUME "):
			lines.append(t)
	return lines


func _is_deposit(line: String) -> bool:
	return line.begins_with("DEPOSIT ")


func _is_consume(line: String) -> bool:
	return line.begins_with("CONSUME ")


## amount == 3rd whitespace token of "DEPOSIT <sid> <amount>".
func _amount(line: String) -> int:
	var parts: PackedStringArray = line.split(" ")
	if parts.size() >= 3:
		return int(parts[2])
	return -999999


func _record(name: String, ok: bool, detail: String) -> void:
	_results.append({ "name": name, "ok": ok, "detail": detail })
	var tag: String = "PASS" if ok else "FAIL"
	print("[2-5c %s] %s — %s" % [name, tag, detail])


# ── A1: deposit, exactly one line, amount == delta == 5 ──────────────────────

func _a1_deposit() -> void:
	var panel := _new_panel()
	# Baseline id 7 at food=10 (prior snapshot), then +5 ⇒ food=15.
	var _b: Array = _ingest_stockpile_lines(panel, _snap([7], [10]))
	var lines: Array = _ingest_stockpile_lines(panel, _snap([7], [15]))
	var deposits: int = 0
	var consumes: int = 0
	var amt: int = -1
	for l in lines:
		if _is_deposit(l):
			deposits += 1
			amt = _amount(l)
		elif _is_consume(l):
			consumes += 1
	var ok: bool = lines.size() == 1 and deposits == 1 and consumes == 0 and amt == 5
	_record("A1", ok, "lines=%d deposits=%d consumes=%d amount=%d (want 1/1/0/5)"
		% [lines.size(), deposits, consumes, amt])


# ── A2: consume, exactly one line, amount == |delta| == 4 (positive) ─────────

func _a2_consume() -> void:
	var panel := _new_panel()
	var _b: Array = _ingest_stockpile_lines(panel, _snap([7], [10]))
	var lines: Array = _ingest_stockpile_lines(panel, _snap([7], [6]))  # delta = -4
	var deposits: int = 0
	var consumes: int = 0
	var amt: int = -1
	for l in lines:
		if _is_consume(l):
			consumes += 1
			amt = _amount(l)
		elif _is_deposit(l):
			deposits += 1
	var ok: bool = lines.size() == 1 and consumes == 1 and deposits == 0 and amt == 4
	_record("A2", ok, "lines=%d consumes=%d deposits=%d amount=%d (want 1/1/0/4, abs not -4)"
		% [lines.size(), consumes, deposits, amt])


# ── A3: threshold boundary is exactly 3 (>= THRESHOLD), sub-threshold ────────
#       does NOT advance the last-notified baseline.

func _a3_threshold_boundary() -> void:
	# case lo: baseline 0 → +2 (no line, baseline unchanged) → then +1 reaching
	# cumulative +3 vs the UNCHANGED baseline DOES push 1 line.
	var lo := _new_panel()
	var _b0: Array = _ingest_stockpile_lines(lo, _snap([1], [0]))   # baseline id1 = 0
	var at2: Array = _ingest_stockpile_lines(lo, _snap([1], [2]))   # delta +2 ⇒ 0 lines
	var baseline_unchanged: bool = int(lo._last_notified_food[1]) == 0
	var at3: Array = _ingest_stockpile_lines(lo, _snap([1], [3]))   # delta +3 vs 0 ⇒ 1 line
	var lo_ok: bool = at2.size() == 0 and baseline_unchanged and at3.size() == 1 \
		and (at3.size() == 0 or _is_deposit(at3[0]))

	# case hi: fresh baseline 0 → +3 directly ⇒ exactly 1 line.
	var hi := _new_panel()
	var _b1: Array = _ingest_stockpile_lines(hi, _snap([1], [0]))
	var hi3: Array = _ingest_stockpile_lines(hi, _snap([1], [3]))
	var hi_ok: bool = hi3.size() == 1

	var ok: bool = lo_ok and hi_ok
	_record("A3", ok,
		"lo: +2 lines=%d baseline_unchanged=%s then +3 lines=%d | hi: +3 lines=%d (want 0/true/1 | 1)"
		% [at2.size(), str(baseline_unchanged), at3.size(), hi3.size()])


# ── A4: first observation of a NEW settlement is a silent baseline ───────────

func _a4_new_settlement_silent() -> void:
	var panel := _new_panel()
	# Never-seen id 42, nonzero food. Stockpile lines must be 0 (baseline only).
	var lines: Array = _ingest_stockpile_lines(panel, _snap([42], [99]))
	var ok: bool = lines.size() == 0
	_record("A4", ok, "first-sight stockpile lines=%d (want 0; formed/dissolved excluded)"
		% [lines.size()])


# ── A5: a disappeared settlement id is pruned — no stale diff on reappearance ─

func _a5_disappeared_pruned() -> void:
	var panel := _new_panel()
	var _b: Array = _ingest_stockpile_lines(panel, _snap([5], [10]))   # baseline S=10
	var _gone: Array = _ingest_stockpile_lines(panel, _snap([], null)) # S absent ⇒ pruned
	var pruned: bool = not panel._last_notified_food.has(5)
	var reappear: Array = _ingest_stockpile_lines(panel, _snap([5], [20]))  # +10 vs stale
	var ok: bool = pruned and reappear.size() == 0
	_record("A5", ok, "pruned=%s reappearance stockpile lines=%d (want true/0, no phantom +10)"
		% [str(pruned), reappear.size()])


# ── A6: short / absent food_stocks array is safe (no crash, defensive skip) ──

func _a6_short_or_absent_array() -> void:
	# case a: food_stocks shorter than ids. Baseline id1; id2 has no food entry.
	var a := _new_panel()
	var _ba: Array = _ingest_stockpile_lines(a, _snap([1, 2], [10]))   # id1 baselined; id2 skipped
	var a_lines: Array = _ingest_stockpile_lines(a, _snap([1, 2], [15]))  # id1 +5 ⇒ 1; id2 skipped
	var a_dep: int = 0
	for l in a_lines:
		if _is_deposit(l):
			a_dep += 1
	var case_a_ok: bool = a_lines.size() == 1 and a_dep == 1

	# case b: food_stocks absent entirely ⇒ every id skipped, no crash, 0 lines.
	var b := _new_panel()
	var _bb: Array = _ingest_stockpile_lines(b, _snap([1, 2], null))
	var b_lines: Array = _ingest_stockpile_lines(b, _snap([1, 2], null))
	var case_b_ok: bool = b_lines.size() == 0

	var ok: bool = case_a_ok and case_b_ok
	_record("A6", ok,
		"short-array: id1 lines=%d (want 1, id2 skipped) | absent-array: lines=%d (want 0, no crash)"
		% [a_lines.size(), b_lines.size()])


# ── finalize ─────────────────────────────────────────────────────────────────

func _finalize() -> void:
	var passed: int = 0
	for r in _results:
		if bool(r.get("ok", false)):
			passed += 1
	var total: int = _results.size()
	var all_ok: bool = passed == total and total == 6

	var lines: PackedStringArray = PackedStringArray()
	lines.append("# stockpile-feed-2-5c — GDScript behavioral harness (A1–A6)")
	for r in _results:
		var tag: String = "PASS" if bool(r.get("ok", false)) else "FAIL"
		lines.append("%s %s — %s" % [tag, String(r.get("name", "")), String(r.get("detail", ""))])
	var verdict: String = "A1_A6_VERDICT PASS" if all_ok else "A1_A6_VERDICT FAIL"
	lines.append("%s (%d/%d)" % [verdict, passed, total])

	var body: String = "\n".join(lines)
	print("\n" + body)
	_write_evidence(body)

	for n in _owned:
		if is_instance_valid(n):
			n.free()

	quit(0 if all_ok else 1)


func _write_evidence(body: String) -> void:
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(EVIDENCE_DIR))
	var path: String = EVIDENCE_DIR + "/feature_harness_assertions.txt"
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if f != null:
		f.store_string(body + "\n")
		f.close()
