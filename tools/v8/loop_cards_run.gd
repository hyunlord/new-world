# tools/v8/loop_cards_run.gd — M-LOOP §3 cards scripted run.
#   Seeds 0–7, 48 days, steward on. A script that answers every open card with its
#   recommended option within 1 in-game hour. Compare with a no-answer run.
#   - Survival must be equal or better.
#   - Record synchronous native command/chronicle deltas for every answer attempt.
#     Native command logs lack source; Player and causal chronicle linkage remain unverified.
#     No-command UI answers are explicitly classified, never invented as native commands.
#   - Count card options whose command fails validation (must be 0).
#   Output goes to the directory passed as --output (default /Users/rexxa/work/loop-data/cards-run/).
# usage: $GODOT_BIN --headless --path . --script res://tools/v8/loop_cards_run.gd -- [--output DIR]
extends SceneTree

const ChronicleText = preload("res://scripts/ui/chronicle_text.gd")
# Stable ChronicleKind::KnowledgeLost code (sim-core/src/chronicle.rs).
const KNOWLEDGE_LOST_KIND: int = 28

const SEEDS: Array[int] = [0, 1, 2, 3, 4, 5, 6, 7]
const DAYS: int = 48
const TICKS_PER_DAY: int = 1440
const TOTAL_TICKS: int = DAYS * TICKS_PER_DAY
const CARD_CHECK_INTERVAL: int = 60  # Check cards every in-game hour.
const ANSWER_DEADLINE: int = 60  # Answer within 1 in-game hour of appearing.

var _output_dir: String = "/Users/rexxa/work/loop-data/cards-run"
## When non-empty, only answer cards of this kind (for per-kind isolation tests).
var _kind_filter: String = ""
## When non-empty, only run these seeds (comma-separated, e.g. "3,6").
var _seed_filter: PackedInt64Array = PackedInt64Array()
var _results: Array[Dictionary] = []
var _validation_failures: int = 0


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--output" and i + 1 < args.size():
			_output_dir = args[i + 1]
		elif args[i] == "--kind" and i + 1 < args.size():
			_kind_filter = args[i + 1]
		elif args[i] == "--seeds" and i + 1 < args.size():
			for s: String in args[i + 1].split(","):
				_seed_filter.append(int(s.strip_edges()))
	call_deferred("_run")


func _run() -> void:
	DirAccess.make_dir_recursive_absolute(_output_dir)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node_or_null("WorldSim")
	var reads: Node = main.get_node_or_null("SimReads")
	if world == null or reads == null:
		_write_result({"error": "missing WorldSim or SimReads"})
		quit(1)
		return

	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	for _i: int in 10:
		await process_frame

	# Run with card answers.
	var active_seeds: Array[int] = []
	if _seed_filter.size() > 0:
		for s: int in _seed_filter:
			active_seeds.append(s)
	else:
		active_seeds.assign(SEEDS)
	var filter_label: String = " (kind=%s)" % _kind_filter if not _kind_filter.is_empty() else ""
	print("=== CARDS RUN: answering cards%s ===" % filter_label)
	for seed_val: int in active_seeds:
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
	for seed_val: int in active_seeds:
		var result: Dictionary = await _run_seed(main, world, reads, seed_val, false)
		result["mode"] = "no_answer"
		_results.append(result)
		print("  seed %d no_answer: alive=%d, deaths=%d, cards=%d" % [
			seed_val, result.get("alive", 0), result.get("deaths", 0),
			result.get("cards_seen", 0)])

	var written: bool = _write_result({"seeds": active_seeds, "active_seeds": active_seeds, "kind_filter": _kind_filter, "full_suite": active_seeds == SEEDS and _kind_filter.is_empty(), "days": DAYS, "results": _results, "total_validation_failures": _validation_failures, "coverage": coverage(_results)})
	var passed: bool = _print_summary(active_seeds)
	quit(0 if passed and written else 1)


func _run_seed(main: Node, world: Node, reads: Node, seed_val: int, answer_cards: bool) -> Dictionary:
	# Reset world with the seed.
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	var reset_ok: bool = bool(reads.call("mutate_raw", "reset_world", [seed_val]))
	world.call("set_sim_speed", 0.0)
	for _i: int in 5:
		await process_frame
	world.call("set_sim_speed", 0.0)
	var initial_tick: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
	if not reset_ok or initial_tick != 0:
		return {"seed": seed_val, "alive": 0, "deaths": 0, "cards_seen": 0, "cards_answered": 0, "validation_failures": 0, "clock_valid": false, "initial_tick": initial_tick, "error": "reset failed or initial clock drift"}
	# Enable steward.
	SimReads.mutate(world, "cmd_set_steward", [true, 0])
	var card_panel: Node = main.find_child("CardPanel", true, false)
	if card_panel == null:
		_validation_failures += 1
		return {"seed": seed_val, "alive": 0, "deaths": 0, "cards_seen": 0, "cards_answered": 0, "validation_failures": 1, "error": "missing CardPanel"}
	var cards_seen: int = 0
	var cards_answered: int = 0
	var val_failures: int = 0
	var answer_log: Array[Dictionary] = []
	var answer_attempts: Array[Dictionary] = []
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
		if ((interval + 1) * CARD_CHECK_INTERVAL) % 1440 == 0:
			var actual_tick: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
			print("CARDS_PROGRESS " + JSON.stringify({"seed": seed_val, "mode": "answer" if answer_cards else "no_answer", "tick": actual_tick, "requested_tick": (interval + 1) * CARD_CHECK_INTERVAL, "total_ticks": TOTAL_TICKS}))
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
				# Per-kind filter: skip cards not matching the filter.
				if not _kind_filter.is_empty() and card_id != _kind_filter:
					continue
				# Use the effective recommended option (dynamic, not static).
				var active_cards: Dictionary = card_panel.get("_active")
				if not active_cards.has(card_id):
					continue
				var card_data: Dictionary = active_cards[card_id]
				var def: Dictionary = card_data.get("def", {})
				var trigger_data: Dictionary = card_data.get("trigger_data", {})
				var recommended_idx: int = int(card_panel.call("_effective_recommended", def, trigger_data))
				# Answer the card.
				var pre_active: PackedStringArray = card_panel.call("active_ids")
				var before_log: Dictionary = world.call("get_command_log", 9223372036854775807)
				var before_chronicle: Dictionary = world.call("get_chronicle_since", 0, 1)
				var before_receipts: Array = card_panel.call("command_results")
				var receipt_start: int = before_receipts.size()
				var answer_tick: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
				card_panel.call("answer_card", card_id, recommended_idx)
				var native_start: int = int(before_log.get("total", -1))
				var native_delta: Dictionary = world.call("get_command_log", maxi(0, native_start))
				var chronicle_delta: Dictionary = _chronicle_since(world, int(before_chronicle.get("total", -1)))
				var after_receipts: Array = card_panel.call("command_results")
				var dispatched: Array = after_receipts.slice(receipt_start).duplicate(true)
				var options: Array = def.get("options", [])
				var option: Dictionary = options[recommended_idx] if recommended_idx >= 0 and recommended_idx < options.size() else {}
				var attempt: Dictionary = {
					"card": card_id, "option_index": recommended_idx, "option": option.get("text_key", ""),
					"command": option.get("command", "none"), "tick": answer_tick,
					"opened_tick": card_data.get("opened_tick", -1),
					"within_deadline": int(card_data.get("opened_tick", -1)) >= 0 and answer_tick >= int(card_data.get("opened_tick", -1)) and answer_tick - int(card_data.get("opened_tick", -1)) <= ANSWER_DEADLINE,
					"native_log_start": native_start, "native_command_delta": native_delta,
					"command_results": dispatched, "chronicle_delta": chronicle_delta,
					"native_source": "UNAVAILABLE", "chronicle_causal_link": "UNVERIFIED",
					"matching": match_native_receipts(dispatched, native_delta, native_start, answer_tick),
					"synchronous_clock_valid": answer_tick == int((world.call("get_time_snapshot") as Dictionary).get("tick", -1)),
				}
				var temporal_matches: Array[int] = []
				for row: Dictionary in chronicle_delta.get("rows", []):
					if int(row.get("tick", -1)) == answer_tick:
						temporal_matches.append(int(row.get("index", -1)))
				attempt["chronicle_same_tick_indices"] = temporal_matches
				attempt["chronicle_match_status"] = "TEMPORAL_ONLY" if not temporal_matches.is_empty() else "NO_SYNCHRONOUS_ENTRY"
				if not bool(chronicle_delta.get("complete", false)):
					attempt["chronicle_match_status"] = "INCOMPLETE_READ"
				answer_attempts.append(attempt)
				var post_active: PackedStringArray = card_panel.call("active_ids")
				# Only count as answered if the card was actually consumed (removed from active).
				var was_consumed: bool = pre_active.has(card_id) and not post_active.has(card_id)
				attempt["consumed"] = was_consumed
				attempt["classification"] = "native_command" if not dispatched.is_empty() else ("presentation_only" if was_consumed else "not_sent")
				if was_consumed:
					cards_answered += 1
					var opt_text: String = str(options[recommended_idx].get("text_key", "")) if recommended_idx < options.size() else ""
					answer_log.append({"seed": seed_val, "card": card_id, "option": opt_text, "tick": answer_tick, "attempt_index": answer_attempts.size() - 1})
				await process_frame
	# Count validation failures: only commands actually sent and rejected by the bridge.
	var command_receipts: Array = []
	var rejection_breakdown: Dictionary = {}
	if card_panel != null and card_panel.has_method("command_results"):
		var results: Array = card_panel.call("command_results")
		command_receipts = results.duplicate(true)
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
	var chronicle_evidence: Dictionary = _chronicle_since(world, 0)
	var lost_rows: Array[Dictionary] = []
	for row: Dictionary in chronicle_evidence.get("rows", []):
		if int(row.get("kind", -1)) == KNOWLEDGE_LOST_KIND:
			lost_rows.append(row)
	var final_tick: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
	return {
		"initial_tick": initial_tick, "final_tick": final_tick, "requested_ticks": TOTAL_TICKS,
		"clock_valid": initial_tick == 0 and final_tick == TOTAL_TICKS,
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
		"command_results": command_receipts,
		"world_command_log": world.call("get_command_log", 0),
		"skipped_breakdown": skipped_breakdown,
		"answers_by_kind": answers_by_kind,
		"answer_log": answer_log,
		"answer_attempts": answer_attempts,
		"knowledge_lost": lost_rows.size() if bool(chronicle_evidence.get("complete", false)) else null,
		"knowledge_lost_rows": lost_rows,
		"chronicle_coverage": {"complete": chronicle_evidence.get("complete", false), "total": chronicle_evidence.get("total", -1), "read_count": (chronicle_evidence.get("rows", []) as Array).size()},
		"evidence_limitations": ["Native command log has no source field; synchronous answer window is attribution evidence, not native Player metadata.", "Chronicle delta is temporal evidence only; no causal command identifier joins the two logs.", "KnowledgeLost count is diagnostic; no acceptance threshold is inferred from this run."],
	}


func _print_summary(active_seeds: Array[int]) -> bool:
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
			var worse: bool = int(r.get("deaths", 0)) > int(baseline.get("deaths", 0))
			if worse:
				per_seed_pass = false
				print("  WARN seed %d: answer deaths=%d > baseline=%d" % [int(r["seed"]), int(r["deaths"]), int(baseline.get("deaths", 0))])
	print("No seed worse than baseline: %s" % ("PASS" if per_seed_pass else "FAIL"))
	print("Total deaths <= no-answer: %s" % ("PASS" if answer_deaths <= no_answer_deaths else "FAIL"))
	print("Validation failures: %d (must be 0: %s)" % [_validation_failures, "PASS" if _validation_failures == 0 else "FAIL"])
	if not all_rejections.is_empty():
		print("Rejection breakdown: %s" % str(all_rejections))
	if not all_skipped.is_empty():
		print("Skipped breakdown: %s" % str(all_skipped))

	for row: Dictionary in _results:
		print("KnowledgeLost seed=%s mode=%s count=%s coverage=%s (diagnostic, no new bar)" % [str(row.get("seed", -1)), str(row.get("mode", "")), str(row.get("knowledge_lost", null)), JSON.stringify(row.get("chronicle_coverage", {}))])
	print("Native Player source and causal answer-to-chronicle linkage: UNVERIFIED (not exposed by existing reads)")
	print("Observed coverage (not an all-kind claim): %s" % JSON.stringify(coverage(_results)))
	var accepted: bool = results_pass(_results, active_seeds, _validation_failures)
	print("Coverage and acceptance: %s" % ("PASS" if accepted else "FAIL"))
	return accepted


static func results_pass(rows: Array[Dictionary], seeds: Array[int], failures: int) -> bool:
	if failures != 0 or seeds.is_empty() or rows.size() != seeds.size() * 2:
		return false
	var observed: Dictionary = coverage(rows)
	if observed["kinds"].is_empty() or int(observed["sent_receipts"]) == 0 or int(observed["answers"]) == 0:
		return false
	var pairs: Dictionary = {}
	var answer_deaths: int = 0
	var baseline_deaths: int = 0
	for seed: int in seeds:
		if pairs.has(seed):
			return false
		pairs[seed] = {}
	for row: Dictionary in rows:
		var seed: int = int(row.get("seed", -1))
		var mode: String = str(row.get("mode", ""))
		if not pairs.has(seed) or mode not in ["answer", "no_answer"] or pairs[seed].has(mode):
			return false
		if not bool(row.get("clock_valid", false)) or int(row.get("initial_tick", -1)) != 0 or int(row.get("final_tick", -1)) != TOTAL_TICKS or int(row.get("requested_ticks", -1)) != TOTAL_TICKS:
			return false
		if int(row.get("validation_failures", -1)) != 0 or not row.get("deaths") is int or int(row["deaths"]) < 0:
			return false
		for receipt: Dictionary in row.get("command_results", []):
			if not bool(receipt.get("accepted", false)):
				return false
		if not _answer_evidence_pass(row):
			return false
		pairs[seed][mode] = int(row["deaths"])
	for seed: int in seeds:
		var pair: Dictionary = pairs[seed]
		if not pair.has("answer") or not pair.has("no_answer") or int(pair["answer"]) > int(pair["no_answer"]):
			return false
		answer_deaths += int(pair["answer"])
		baseline_deaths += int(pair["no_answer"])
	return answer_deaths <= baseline_deaths


## Completeness and synchronous attribution are checkable; native Player metadata and
## a causal chronicle join are not exposed and remain explicitly unverified.
static func _answer_evidence_pass(row: Dictionary) -> bool:
	var attempts: Array = row.get("answer_attempts", [])
	var answers: Array = row.get("answer_log", [])
	var receipts: Array = row.get("command_results", [])
	if int(row.get("cards_answered", -1)) != answers.size():
		return false
	if str(row.get("mode", "")) == "no_answer":
		return attempts.is_empty() and answers.is_empty() and receipts.is_empty()
	var consumed: Array[int] = []
	var first_attempts: Dictionary = {}
	var collected: Array = []
	for i: int in attempts.size():
		var attempt: Dictionary = attempts[i]
		var tick: int = int(attempt.get("tick", -1))
		var opened: int = int(attempt.get("opened_tick", -1))
		if opened < 0 or tick < opened or tick > TOTAL_TICKS or not bool(attempt.get("synchronous_clock_valid", false)):
			return false
		var timely: bool = tick - opened <= ANSWER_DEADLINE
		if not attempt.has("within_deadline") or bool(attempt["within_deadline"]) != timely:
			return false
		var dispatched: Array = attempt.get("command_results", [])
		var card: String = str(attempt.get("card", ""))
		if card.is_empty():
			return false
		if not first_attempts.has(card):
			first_attempts[card] = {}
		var openings: Dictionary = first_attempts[card]
		if not openings.has(opened):
			if not timely:
				return false
			openings[opened] = tick
		elif tick < int(openings[opened]):
			return false
		# A stale target may remain open and be retried without dispatch. The first
		# attempt and every consumed/native action must still meet the deadline.
		if not timely and (bool(attempt.get("consumed", false)) or not dispatched.is_empty()):
			return false
		var matching: Dictionary = match_native_receipts(dispatched, attempt.get("native_command_delta", {}), int(attempt.get("native_log_start", -1)), tick)
		if not bool(matching.get("complete", false)) or not bool((attempt.get("matching", {}) as Dictionary).get("complete", false)):
			return false
		if not bool((attempt.get("chronicle_delta", {}) as Dictionary).get("complete", false)):
			return false
		collected.append_array(dispatched)
		if bool(attempt.get("consumed", false)):
			consumed.append(i)
	if collected != receipts or consumed.size() != answers.size():
		return false
	for i: int in answers.size():
		var answer: Dictionary = answers[i]
		if int(answer.get("attempt_index", -1)) != consumed[i]:
			return false
		var attempt: Dictionary = attempts[consumed[i]]
		if str(answer.get("card", "")).is_empty() or answer.get("card") != attempt.get("card") or answer.get("tick") != attempt.get("tick"):
			return false
	return true


func _write_result(data: Dictionary) -> bool:
	var path: String = _output_dir.path_join("cards_run_result.json")
	var written: bool = write_result_file(path, data)
	print("Result %s: %s" % ["written" if written else "WRITE FAILED", path])
	return written


static func write_result_file(path: String, data: Dictionary) -> bool:
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		return false
	file.store_string(JSON.stringify(data, "\t"))
	file.flush()
	var written: bool = file.get_error() == OK
	file.close()
	return written


## Report natural coverage separately: absent kinds need focused fixture evidence.
static func coverage(rows: Array[Dictionary]) -> Dictionary:
	var kinds: Dictionary = {}
	var answered_kinds: Dictionary = {}
	var receipts: int = 0
	var answers: int = 0
	for row: Dictionary in rows:
		for kind: String in row.get("cards_kinds_seen", []):
			kinds[kind] = true
		if str(row.get("mode", "")) == "answer":
			receipts += (row.get("command_results", []) as Array).size()
			answers += int(row.get("cards_answered", 0))
			for answer: Dictionary in row.get("answer_log", []):
				answered_kinds[str(answer.get("card", ""))] = true
	var sorted_kinds: Array = kinds.keys()
	sorted_kinds.sort()
	var sorted_answered: Array = answered_kinds.keys()
	sorted_answered.sort()
	var missing: Array[String] = []
	var registry: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/cards.json"))
	for definition: Dictionary in registry.get("cards", []):
		if not bool(definition.get("test_only", false)) and not kinds.has(str(definition["id"])):
			missing.append(str(definition["id"]))
	missing.sort()
	return {"kinds": sorted_kinds, "answered_kinds": sorted_answered, "unobserved_kinds": missing, "sent_receipts": receipts, "answers": answers}


## Read every retained native chronicle row; missing/malformed pagination is not zero loss.
static func _chronicle_since(world: Node, start: int) -> Dictionary:
	var rows: Array[Dictionary] = []
	if start < 0 or not world.has_method("get_chronicle_since"):
		return {"complete": false, "rows": rows, "total": -1}
	var cursor: int = start
	while true:
		var page: Dictionary = world.call("get_chronicle_since", cursor, 512)
		var total: int = int(page.get("total", -1))
		var next_index: int = int(page.get("next", -1))
		var chunk: Array[Dictionary] = ChronicleText.rows(page)
		if total < cursor or next_index < cursor or next_index > total or chunk.size() != next_index - cursor:
			return {"complete": false, "rows": rows, "total": total}
		for row: Dictionary in chunk:
			if int(row.get("index", -1)) != cursor or int(row.get("kind", -1)) < 0:
				return {"complete": false, "rows": rows, "total": total}
			rows.append(row)
			cursor += 1
		if cursor == total:
			return {"complete": true, "rows": rows, "total": total}
		if chunk.is_empty():
			return {"complete": false, "rows": rows, "total": total}
	return {"complete": false, "rows": rows, "total": -1}


## A receipt matches only its exact native index, kind, acceptance and synchronous tick.
static func match_native_receipts(receipts: Array, native: Dictionary, start: int, tick: int) -> Dictionary:
	var kinds: PackedStringArray = native.get("kinds", PackedStringArray())
	var ticks: PackedInt64Array = native.get("ticks", PackedInt64Array())
	var accepted: PackedByteArray = native.get("accepted", PackedByteArray())
	var valid: bool = start >= 0 and int(native.get("total", -1)) == start + kinds.size() and ticks.size() == kinds.size() and accepted.size() == kinds.size()
	var matched: Array[int] = []
	var unmatched: Array[int] = []
	for i: int in receipts.size():
		var receipt: Dictionary = receipts[i]
		var offset: int = int(receipt.get("log_index", -1)) - start
		if valid and offset >= 0 and offset < kinds.size() and not matched.has(offset) and str(receipt.get("log_kind", "")) == kinds[offset] and bool(receipt.get("accepted", false)) == (accepted[offset] != 0) and ticks[offset] == tick:
			matched.append(offset)
		else:
			unmatched.append(i)
	return {"native_shape_valid": valid, "matched": matched.size(), "unmatched_receipt_indices": unmatched, "unmatched_native_count": kinds.size() - matched.size(), "complete": valid and unmatched.is_empty() and matched.size() == kinds.size()}
