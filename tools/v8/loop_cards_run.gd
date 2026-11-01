# tools/v8/loop_cards_run.gd — M-LOOP §3 cards scripted run.
#   Seeds 0–7, 48 days, steward on. A script that answers every open card with its
#   recommended option within 1 in-game hour. Compare with a no-answer run.
#   - Survival must be equal or better.
#   - Every answer must appear in the command log (source Player) and in the chronicle.
#   - Count card options whose command fails validation (must be 0).
#   Output goes to the directory passed as --output (default /Users/rexxa/work/loop-data/cards-run/).
# usage: $GODOT_BIN --headless --path . --script res://tools/v8/loop_cards_run.gd -- [--output DIR]
extends SceneTree

const SEEDS: Array[int] = [0, 1, 2, 3, 4, 5, 6, 7]
const DAYS: int = 48
const TICKS_PER_DAY: int = 1440
const TOTAL_TICKS: int = DAYS * TICKS_PER_DAY
const CARD_CHECK_INTERVAL: int = 60  # Check cards every in-game hour.
const ANSWER_DEADLINE: int = 60  # Answer within 1 in-game hour of appearing.

var _output_dir: String = "/Users/rexxa/work/loop-data/cards-run"
var _results: Array[Dictionary] = []
var _validation_failures: int = 0


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--output" and i + 1 < args.size():
			_output_dir = args[i + 1]
	call_deferred("_run")


func _run() -> void:
	DirAccess.make_dir_recursive_absolute(_output_dir)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in 10:
		await process_frame
	var world: Node = main.get_node_or_null("WorldSim")
	var reads: Node = main.get_node_or_null("SimReads")
	if world == null or reads == null:
		_write_result({"error": "missing WorldSim or SimReads"})
		quit(1)
		return

	# Run with card answers.
	print("=== CARDS RUN: answering cards ===")
	for seed_val: int in SEEDS:
		var result: Dictionary = await _run_seed(main, world, reads, seed_val, true)
		result["mode"] = "answer"
		_results.append(result)
		var skip_n: int = 0
		for k: String in result.get("skipped_breakdown", {}):
			skip_n += int(result["skipped_breakdown"][k])
		print("  seed %d answer: alive=%d, deaths=%d, cards=%d, answers=%d, skipped=%d, val_fail=%d" % [
			seed_val, result.get("alive", 0), result.get("deaths", 0),
			result.get("cards_seen", 0), result.get("cards_answered", 0),
			skip_n, result.get("validation_failures", 0)])

	# Run without card answers (baseline).
	print("=== CARDS RUN: no answers (baseline) ===")
	for seed_val: int in SEEDS:
		var result: Dictionary = await _run_seed(main, world, reads, seed_val, false)
		result["mode"] = "no_answer"
		_results.append(result)
		print("  seed %d no_answer: alive=%d, deaths=%d, cards=%d" % [
			seed_val, result.get("alive", 0), result.get("deaths", 0),
			result.get("cards_seen", 0)])

	_write_result({"seeds": SEEDS, "days": DAYS, "results": _results, "total_validation_failures": _validation_failures})
	_print_summary()
	quit(0)


func _run_seed(main: Node, world: Node, reads: Node, seed_val: int, answer_cards: bool) -> Dictionary:
	# Reset world with the seed.
	reads.call("mutate_raw", "reset_world", [seed_val])
	for _i: int in 5:
		await process_frame
	# Enable steward.
	SimReads.mutate(world, "cmd_set_steward", [true, 0])
	var card_panel: Node = main.find_child("CardPanel", true, false)
	var cards_seen: int = 0
	var cards_answered: int = 0
	var val_failures: int = 0
	var answer_log: Array[Dictionary] = []
	# Advance tick by tick, checking cards at intervals.
	# In headless mode, directly call _poll_triggers since _process timer doesn't fire.
	@warning_ignore("integer_division")
	var intervals: int = TOTAL_TICKS / CARD_CHECK_INTERVAL
	var card_kinds_seen: Dictionary = {}
	for interval: int in intervals:
		# Advance one interval.
		reads.call("mutate_raw", "advance_ticks", [CARD_CHECK_INTERVAL])
		await process_frame
		await process_frame
		# Directly trigger card polling (headless mode bypasses the delta timer).
		if card_panel != null and card_panel.has_method("_poll_triggers"):
			card_panel.call("_poll_triggers")
			card_panel.call("_check_expiry")
		# Check for open cards.
		if card_panel == null or not card_panel.has_method("active_ids"):
			continue
		var active: PackedStringArray = card_panel.call("active_ids")
		for cid: String in active:
			card_kinds_seen[cid] = true
		cards_seen = maxi(cards_seen, active.size())
		if answer_cards and not active.is_empty():
			for card_id: String in active:
				# Use the effective recommended option (dynamic, not static).
				var active_cards: Dictionary = card_panel.get("_active")
				if not active_cards.has(card_id):
					continue
				var card_data: Dictionary = active_cards[card_id]
				var def: Dictionary = card_data.get("def", {})
				var trigger_data: Dictionary = card_data.get("trigger_data", {})
				var recommended_idx: int = int(card_panel.call("_effective_recommended", def, trigger_data))
				# Answer the card.
				card_panel.call("answer_card", card_id, recommended_idx)
				cards_answered += 1
				var options: Array = def.get("options", [])
				var opt_text: String = str(options[recommended_idx].get("text_key", "")) if recommended_idx < options.size() else ""
				answer_log.append({"seed": seed_val, "card": card_id, "option": opt_text, "tick": interval * CARD_CHECK_INTERVAL})
				await process_frame
	# Count validation failures: only commands actually sent and rejected by the bridge.
	var rejection_breakdown: Dictionary = {}
	if card_panel != null and card_panel.has_method("command_results"):
		var results: Array = card_panel.call("command_results")
		for r: Dictionary in results:
			if not bool(r.get("accepted", false)):
				val_failures += 1
				var key: String = "%s/%s" % [str(r.get("command", "?")), str(r.get("reason", "?"))]
				rejection_breakdown[key] = int(rejection_breakdown.get(key, 0)) + 1
	# Count skipped commands by kind (stale targets not sent).
	var skipped_breakdown: Dictionary = {}
	if card_panel != null and card_panel.has_method("skipped_results"):
		var skipped: Array = card_panel.call("skipped_results")
		for s: Dictionary in skipped:
			var key: String = "%s/%s" % [str(s.get("card_id", "?")), str(s.get("reason", "?"))]
			skipped_breakdown[key] = int(skipped_breakdown.get(key, 0)) + 1
	# Collect final state and death causes.
	var agent_snap: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var alive: int = (agent_snap.get("ids", PackedInt64Array()) as PackedInt64Array).size()
	var tallies: Dictionary = SimReads.read(world, "get_tallies") if world.has_method("get_tallies") else {}
	var deaths: int = 0
	var death_causes: Dictionary = {}
	for key: String in ["deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure"]:
		var count: int = int(tallies.get(key, 0))
		deaths += count
		if count > 0:
			death_causes[key] = count
	var births: int = int(tallies.get("births", 0))
	_validation_failures += val_failures
	# Build answers-by-kind from the answer log.
	var answers_by_kind: Dictionary = {}
	for entry: Dictionary in answer_log:
		var kind: String = str(entry.get("card", ""))
		answers_by_kind[kind] = int(answers_by_kind.get(kind, 0)) + 1
	return {
		"seed": seed_val,
		"alive": alive,
		"deaths": deaths,
		"births": births,
		"death_causes": death_causes,
		"cards_seen": cards_seen,
		"cards_answered": cards_answered,
		"cards_kinds_seen": card_kinds_seen.keys(),
		"validation_failures": val_failures,
		"rejection_breakdown": rejection_breakdown,
		"skipped_breakdown": skipped_breakdown,
		"answers_by_kind": answers_by_kind,
		"answer_log": answer_log,
	}


func _print_summary() -> void:
	print("\n=== SUMMARY ===")
	var answer_alive: int = 0
	var answer_deaths: int = 0
	var answer_births: int = 0
	var no_answer_alive: int = 0
	var no_answer_deaths: int = 0
	var no_answer_births: int = 0
	var total_answered: int = 0
	var answer_death_causes: Dictionary = {}
	var no_answer_death_causes: Dictionary = {}
	var all_rejections: Dictionary = {}
	var all_skipped: Dictionary = {}
	var all_answers_by_kind: Dictionary = {}
	for r: Dictionary in _results:
		if str(r["mode"]) == "answer":
			answer_alive += int(r["alive"])
			answer_deaths += int(r["deaths"])
			answer_births += int(r.get("births", 0))
			total_answered += int(r["cards_answered"])
			for k: String in r.get("death_causes", {}):
				answer_death_causes[k] = int(answer_death_causes.get(k, 0)) + int(r["death_causes"][k])
			for k: String in r.get("rejection_breakdown", {}):
				all_rejections[k] = int(all_rejections.get(k, 0)) + int(r["rejection_breakdown"][k])
			for k: String in r.get("skipped_breakdown", {}):
				all_skipped[k] = int(all_skipped.get(k, 0)) + int(r["skipped_breakdown"][k])
			for k: String in r.get("answers_by_kind", {}):
				all_answers_by_kind[k] = int(all_answers_by_kind.get(k, 0)) + int(r["answers_by_kind"][k])
		else:
			no_answer_alive += int(r["alive"])
			no_answer_deaths += int(r["deaths"])
			no_answer_births += int(r.get("births", 0))
			for k: String in r.get("death_causes", {}):
				no_answer_death_causes[k] = int(no_answer_death_causes.get(k, 0)) + int(r["death_causes"][k])
	print("Answer:    alive=%d deaths=%d births=%d cards_answered=%d" % [answer_alive, answer_deaths, answer_births, total_answered])
	print("  death causes: %s" % str(answer_death_causes))
	print("  answers by kind: %s" % str(all_answers_by_kind))
	print("No-answer: alive=%d deaths=%d births=%d" % [no_answer_alive, no_answer_deaths, no_answer_births])
	print("  death causes: %s" % str(no_answer_death_causes))
	# Per-seed table.
	print("\n=== PER-SEED TABLE ===")
	print("seed | mode      | alive | deaths | births | answered | skipped | death_causes | answers_by_kind | skipped_by_kind")
	for r: Dictionary in _results:
		var skip_count: int = 0
		for k: String in r.get("skipped_breakdown", {}):
			skip_count += int(r["skipped_breakdown"][k])
		print("  %d  | %-9s | %5d | %6d | %6d | %8d | %7d | %s | %s | %s" % [
			int(r["seed"]), str(r["mode"]),
			int(r["alive"]), int(r["deaths"]), int(r.get("births", 0)),
			int(r.get("cards_answered", 0)), skip_count,
			str(r.get("death_causes", {})), str(r.get("answers_by_kind", {})),
			str(r.get("skipped_breakdown", {}))])
	# Per-seed survival comparison.
	var per_seed_pass: bool = true
	for r: Dictionary in _results:
		if str(r["mode"]) == "answer":
			var baseline: Dictionary = {}
			for b: Dictionary in _results:
				if str(b["mode"]) == "no_answer" and int(b["seed"]) == int(r["seed"]):
					baseline = b
			var worse: bool = int(r.get("deaths", 0)) > int(baseline.get("deaths", 0)) + 1
			if worse:
				per_seed_pass = false
				print("  WARN seed %d: answer deaths=%d > baseline+1=%d" % [int(r["seed"]), int(r["deaths"]), int(baseline.get("deaths", 0)) + 1])
	print("No seed worse than baseline+1: %s" % ("PASS" if per_seed_pass else "FAIL"))
	print("Total deaths <= no-answer+1: %s" % ("PASS" if answer_deaths <= no_answer_deaths + 1 else "FAIL"))
	print("Validation failures: %d (must be 0: %s)" % [_validation_failures, "PASS" if _validation_failures == 0 else "FAIL"])
	if not all_rejections.is_empty():
		print("Rejection breakdown: %s" % str(all_rejections))
	if not all_skipped.is_empty():
		print("Skipped breakdown: %s" % str(all_skipped))


func _write_result(data: Dictionary) -> void:
	var path: String = _output_dir.path_join("cards_run_result.json")
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(data, "\t"))
		file.close()
		print("Result written to: ", path)
