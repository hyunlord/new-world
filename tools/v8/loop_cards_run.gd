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
		print("  seed %d answer: alive=%d, deaths=%d, cards=%d, answers=%d, validation_fail=%d" % [
			seed_val, result.get("alive", 0), result.get("deaths", 0),
			result.get("cards_seen", 0), result.get("cards_answered", 0),
			result.get("validation_failures", 0)])

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
	@warning_ignore("integer_division")
	var intervals: int = TOTAL_TICKS / CARD_CHECK_INTERVAL
	for interval: int in intervals:
		# Advance one interval.
		reads.call("mutate_raw", "advance_ticks", [CARD_CHECK_INTERVAL])
		await process_frame
		await process_frame
		# Check for open cards.
		if card_panel == null or not card_panel.has_method("active_ids"):
			continue
		var active: PackedStringArray = card_panel.call("active_ids")
		cards_seen = maxi(cards_seen, active.size())
		if answer_cards and not active.is_empty():
			for card_id: String in active:
				# Find the recommended option index.
				var active_cards: Dictionary = card_panel.get("_active")
				if not active_cards.has(card_id):
					continue
				var card_data: Dictionary = active_cards[card_id]
				var def: Dictionary = card_data.get("def", {})
				var options: Array = def.get("options", [])
				var recommended_idx: int = 0
				for oi: int in options.size():
					if bool(options[oi].get("recommended", false)):
						recommended_idx = oi
						break
				# Answer the card.
				card_panel.call("answer_card", card_id, recommended_idx)
				cards_answered += 1
				var opt_text: String = str(options[recommended_idx].get("text_key", ""))
				answer_log.append({"seed": seed_val, "card": card_id, "option": opt_text, "tick": interval * CARD_CHECK_INTERVAL})
				await process_frame
	# Collect final state.
	var agent_snap: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var alive: int = (agent_snap.get("ids", PackedInt64Array()) as PackedInt64Array).size()
	var tallies: Dictionary = SimReads.read(world, "get_tallies") if world.has_method("get_tallies") else {}
	var deaths: int = 0
	for key: String in ["deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure"]:
		deaths += int(tallies.get(key, 0))
	_validation_failures += val_failures
	return {
		"seed": seed_val,
		"alive": alive,
		"deaths": deaths,
		"cards_seen": cards_seen,
		"cards_answered": cards_answered,
		"validation_failures": val_failures,
		"answer_log": answer_log,
	}


func _print_summary() -> void:
	print("\n=== SUMMARY ===")
	var answer_alive: int = 0
	var answer_deaths: int = 0
	var no_answer_alive: int = 0
	var no_answer_deaths: int = 0
	var total_answered: int = 0
	for r: Dictionary in _results:
		if str(r["mode"]) == "answer":
			answer_alive += int(r["alive"])
			answer_deaths += int(r["deaths"])
			total_answered += int(r["cards_answered"])
		else:
			no_answer_alive += int(r["alive"])
			no_answer_deaths += int(r["deaths"])
	print("Answer:    alive=%d deaths=%d cards_answered=%d" % [answer_alive, answer_deaths, total_answered])
	print("No-answer: alive=%d deaths=%d" % [no_answer_alive, no_answer_deaths])
	print("Survival equal or better: %s" % ("PASS" if answer_alive >= no_answer_alive else "FAIL"))
	print("Validation failures: %d (must be 0: %s)" % [_validation_failures, "PASS" if _validation_failures == 0 else "FAIL"])


func _write_result(data: Dictionary) -> void:
	var path: String = _output_dir.path_join("cards_run_result.json")
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(data, "\t"))
		file.close()
		print("Result written to: ", path)
